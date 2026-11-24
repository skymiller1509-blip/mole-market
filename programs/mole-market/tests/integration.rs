//! Full game-loop integration tests on LiteSVM (in-process Solana runtime).
//! Covers: create -> join -> fill bots -> demo mole pick -> human trade ->
//! session-key trade -> bot trades -> reveal + payouts, plus the failure
//! boundaries for VRF and delegation (neither program is deployed on
//! LiteSVM, so those paths must fail, not silently pass).

use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use mole_market::{accounts as acc, instruction as ix, state::Game, ID};
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use solana_transaction::Transaction;

const GAME_ID: u64 = 777;

fn game_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"game", &GAME_ID.to_le_bytes()], &ID)
}

fn bot_pda(seat: u8) -> Pubkey {
    Pubkey::find_program_address(&[b"bot", &GAME_ID.to_le_bytes(), &[seat]], &ID).0
}

fn load_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let so_path = format!(
        "{}/../../target/deploy/mole_market.so",
        env!("CARGO_MANIFEST_DIR")
    );
    svm.add_program_from_file(ID, &so_path)
        .expect("add mole-market program");
    svm
}

fn send(
    svm: &mut LiteSVM,
    payer: &Keypair,
    ixs: Vec<Instruction>,
    extra_signers: &[&Keypair],
) -> Result<litesvm::types::TransactionMetadata, String> {
    let blockhash = svm.latest_blockhash();
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&payer.pubkey()),
        &signers,
        blockhash,
    );
    svm.send_transaction(tx).map_err(|e| format!("{e:?}"))
}

fn ix_create_game(creator: &Pubkey) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::CreateGame {
            game,
            creator: *creator,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: ix::CreateGame { game_id: GAME_ID }.data(),
    }
}

fn ix_join_game(player: &Pubkey, seat: u8) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::JoinGame {
            game,
            player: *player,
        }
        .to_account_metas(None),
        data: ix::JoinGame {
            game_id: GAME_ID,
            seat,
        }
        .data(),
    }
}

fn ix_fill_bots(payer: &Pubkey) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::FillBots {
            game,
            payer: *payer,
        }
        .to_account_metas(None),
        data: ix::FillBots { game_id: GAME_ID }.data(),
    }
}

fn ix_pick_demo(creator: &Pubkey, seed: u64) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::PickMolesDemo {
            game,
            creator: *creator,
        }
        .to_account_metas(None),
        data: ix::PickMolesDemo {
            game_id: GAME_ID,
            seed,
        }
        .data(),
    }
}

fn ix_trade(
    signer: &Pubkey,
    seat: u8,
    is_buy: bool,
    amount: u64,
) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::Trade {
            game,
            signer: *signer,
        }
        .to_account_metas(None),
        data: ix::Trade {
            game_id: GAME_ID,
            seat,
            is_buy,
            amount,
        }
        .data(),
    }
}

fn ix_bot_trade(payer: &Pubkey, seat: u8) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::BotTrade {
            game,
            payer: *payer,
        }
        .to_account_metas(None),
        data: ix::BotTrade {
            game_id: GAME_ID,
            seat,
        }
        .data(),
    }
}

fn ix_end_round(payer: &Pubkey) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::EndRound {
            game,
            payer: *payer,
        }
        .to_account_metas(None),
        data: ix::EndRound { game_id: GAME_ID }.data(),
    }
}

fn ix_create_session(
    owner: &Pubkey,
    seat: u8,
    session_key: Pubkey,
    expiry: u64,
) -> Instruction {
    let (game, _) = game_pda();
    Instruction {
        program_id: ID,
        accounts: acc::CreateSession {
            game,
            owner: *owner,
        }
        .to_account_metas(None),
        data: ix::CreateSession {
            game_id: GAME_ID,
            seat,
            session_key,
            expiry_slot: expiry,
        }
        .data(),
    }
}

fn read_game(svm: &LiteSVM) -> Game {
    let (game, _) = game_pda();
    let account = svm.get_account(&game).expect("game account exists");
    Game::try_deserialize(&mut account.data.as_slice()).expect("decode game")
}

fn setup_full_table() -> (LiteSVM, Keypair, Keypair, Keypair) {
    let mut svm = load_svm();
    let creator = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&alice.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&bob.pubkey(), 10_000_000_000).unwrap();

    send(&mut svm, &creator, vec![ix_create_game(&creator.pubkey())], &[]).unwrap();
    send(&mut svm, &alice, vec![ix_join_game(&alice.pubkey(), 1)], &[]).unwrap();
    send(&mut svm, &bob, vec![ix_join_game(&bob.pubkey(), 2)], &[]).unwrap();
    send(&mut svm, &creator, vec![ix_fill_bots(&creator.pubkey())], &[]).unwrap();

    let game = read_game(&svm);
    assert_eq!(game.status, 0, "lobby");
    assert!(game.seats_filled(), "all 8 seats filled after fill_bots");
    assert_eq!(game.pot, 8_000);
    assert_eq!(game.price, 10_000);
    assert_eq!(game.seats[0].owner, creator.pubkey());
    assert_eq!(game.seats[1].owner, alice.pubkey());
    assert_eq!(game.seats[2].owner, bob.pubkey());
    for seat in 3..8 {
        assert!(game.seats[seat as usize].is_bot, "seat {seat} is a bot");
        assert_eq!(game.seats[seat as usize].owner, bot_pda(seat));
        assert_eq!(game.seats[seat as usize].balance, 1_000);
    }
    (svm, creator, alice, bob)
}

#[test]
fn full_game_loop_with_bots() {
    let (mut svm, creator, alice, _bob) = setup_full_table();

    // Demo mole pick (VRF is not deployed on LiteSVM; demo path is the one
    // under test here).
    send(
        &mut svm,
        &creator,
        vec![ix_pick_demo(&creator.pubkey(), 42)],
        &[],
    )
    .unwrap();
    let game = read_game(&svm);
    assert_eq!(game.status, 2, "trading");
    assert_eq!(game.randomness_source, 2, "DEMO source recorded on chain");
    assert_ne!(game.mole_a, game.mole_b, "two distinct moles");
    assert!(game.mole_a < 8 && game.mole_b < 8);
    let (mole_a, mole_b) = (game.mole_a, game.mole_b);

    // Human trade: creator buys 500. Pot must leak 2% (10 pts).
    send(
        &mut svm,
        &creator,
        vec![ix_trade(&creator.pubkey(), 0, true, 500)],
        &[],
    )
    .unwrap();
    let game = read_game(&svm);
    assert_eq!(game.seats[0].balance, 500);
    assert!(game.seats[0].shares > 0, "long position opened");
    assert_eq!(game.pot, 8_000 - 10, "pot leaked 2% of 500");
    assert_eq!(game.mole_pool, 10);
    assert!(game.price > 10_000, "buy moved the price up");

    // Wrong signer cannot trade someone else's seat.
    let err = send(
        &mut svm,
        &alice,
        vec![ix_trade(&alice.pubkey(), 0, true, 100)],
        &[],
    );
    assert!(err.is_err(), "non-owner trade must fail");

    // Session key: creator authorizes a throwaway key, then trades with it.
    // Expiry is far in the future (LiteSVM slots start near 0).
    let session = Keypair::new();
    send(
        &mut svm,
        &creator,
        vec![ix_create_session(
            &creator.pubkey(),
            0,
            session.pubkey(),
            1_000_000_000,
        )],
        &[],
    )
    .unwrap();
    // The session key signs the trade instruction, but the creator still
    // pays the transaction fee (the session key holds no lamports).
    send(
        &mut svm,
        &creator,
        vec![ix_trade(&session.pubkey(), 0, false, 100)],
        &[&session],
    )
    .unwrap();
    let game = read_game(&svm);
    assert_eq!(game.seats[0].balance, 400, "session-key trade settled");

    // Bots trade: every bot seat takes its deterministic turn.
    for seat in 3..8u8 {
        send(
            &mut svm,
            &creator,
            vec![ix_bot_trade(&creator.pubkey(), seat)],
            &[],
        )
        .unwrap();
    }
    let game = read_game(&svm);
    assert!(game.tick >= 7, "tick advanced (2 human + 5 bot trades)");
    assert!(game.mole_pool > 10, "bots leaked more into the pool");
    assert!(game.pot < 8_000 - 10, "pot kept leaking");

    // A human cannot use bot_trade, and a bot seat cannot be traded by hand.
    let err = send(
        &mut svm,
        &creator,
        vec![ix_bot_trade(&creator.pubkey(), 0)],
        &[],
    );
    assert!(err.is_err(), "bot_trade on a human seat must fail");

    // Reveal: payouts split the surviving pot among the honest and the
    // skimmed pool between the two moles.
    let pot_before = game.pot;
    let pool_before = game.mole_pool;
    send(
        &mut svm,
        &creator,
        vec![ix_end_round(&creator.pubkey())],
        &[],
    )
    .unwrap();
    let game = read_game(&svm);
    assert_eq!(game.status, 3, "revealed");
    let honest_share = pot_before / 6;
    let mole_share = pool_before / 2;
    for (i, seat) in game.seats.iter().enumerate() {
        let is_mole = i as u8 == mole_a || i as u8 == mole_b;
        let base = seat.balance as i128
            + mole_market::logic::position_pnl(seat.shares, seat.entry_price, game.price);
        let expected = (base.max(0) as u64)
            + if is_mole { mole_share } else { honest_share };
        assert_eq!(
            seat.payout, expected,
            "seat {i} payout (mole={is_mole}) must match the formula"
        );
    }
    // Moles really did get the skimmed pool.
    assert!(game.seats[mole_a as usize].payout >= mole_share);
    assert!(game.seats[mole_b as usize].payout >= mole_share);
}

#[test]
fn vrf_request_fails_without_vrf_program() {
    // Documents the boundary: on LiteSVM the VRF program is not deployed,
    // so start_round must fail instead of silently picking moles.
    // Accounts are hand-built because #[vrf] injects program_identity,
    // vrf_program, slot_hashes and system_program into the struct.
    use std::str::FromStr;
    let (mut svm, creator, _, _) = setup_full_table();
    let (game, _) = game_pda();
    let (program_identity, _) = Pubkey::find_program_address(&[b"identity"], &ID);
    let vrf_program =
        Pubkey::from_str("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz").unwrap();
    let slot_hashes =
        Pubkey::from_str("SysvarS1otHashes111111111111111111111111111").unwrap();
    let ix = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(game, false),
            AccountMeta::new(
                Pubkey::from_str("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh").unwrap(),
                false,
            ),
            AccountMeta::new_readonly(program_identity, false),
            AccountMeta::new_readonly(vrf_program, false),
            AccountMeta::new_readonly(slot_hashes, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix::StartRound { game_id: GAME_ID }.data(),
    };
    let err = send(&mut svm, &creator, vec![ix], &[]);
    assert!(err.is_err(), "start_round without a VRF program must fail");
    let game = read_game(&svm);
    assert_eq!(game.status, 0, "game still in lobby after failed VRF request");
}

#[test]
fn delegate_fails_without_delegation_program() {
    // Documents the boundary: delegation needs the MagicBlock Delegation
    // Program, which is not on LiteSVM. Must fail, not silently pass.
    // Accounts are hand-built because #[delegate] injects buffer_game,
    // delegation_record_game and delegation_metadata_game (before `game`)
    // plus owner_program, delegation_program and system_program.
    use ephemeral_rollups_sdk::pda::{
        DELEGATE_BUFFER_TAG, DELEGATION_METADATA_TAG, DELEGATION_RECORD_TAG,
    };
    let (mut svm, creator, _, _) = setup_full_table();
    let (game, _) = game_pda();
    let delegation_program = ephemeral_rollups_sdk::id();
    let (buffer_game, _) = Pubkey::find_program_address(
        &[DELEGATE_BUFFER_TAG, game.as_ref()],
        &ID,
    );
    let (delegation_record_game, _) = Pubkey::find_program_address(
        &[DELEGATION_RECORD_TAG, game.as_ref()],
        &delegation_program,
    );
    let (delegation_metadata_game, _) = Pubkey::find_program_address(
        &[DELEGATION_METADATA_TAG, game.as_ref()],
        &delegation_program,
    );
    let ix = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new(buffer_game, false),
            AccountMeta::new(delegation_record_game, false),
            AccountMeta::new(delegation_metadata_game, false),
            AccountMeta::new(game, false),
            AccountMeta::new_readonly(ID, false),
            AccountMeta::new_readonly(delegation_program, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: ix::DelegateGame { game_id: GAME_ID }.data(),
    };
    let err = send(&mut svm, &creator, vec![ix], &[]);
    assert!(err.is_err(), "delegate without the Delegation program must fail");
}
