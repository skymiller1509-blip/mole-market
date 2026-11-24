use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{
    anchor::{vrf, vrf_callback},
    vrf::{
        self,
        consts::program_identity_pubkey,
        instructions::{create_request_randomness_ix_const, RequestRandomnessParams},
        types::SerializableAccountMeta,
    },
};

use crate::{constants::*, error::MoleError, logic::*, state::*};

/// Requester identity PDA, derived at compile time (SDK 0.18+).
const PROGRAM_IDENTITY: Pubkey = program_identity_pubkey(&crate::ID);

/// Start the round: request verifiable randomness from the MagicBlock VRF
/// oracle (base-layer queue). The oracle calls back `callback_pick_moles`,
/// which secretly picks the two mole seats. Costs 500,000 lamports on the
/// base queue, paid by the caller.
pub fn start_round_handler(ctx: Context<StartRound>, game_id: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_LOBBY, MoleError::NotInLobby);
    require!(game.seats_filled(), MoleError::SeatsNotFull);
    require!(!game.pending_vrf, MoleError::VrfAlreadyPending);

    game.vrf_nonce = game.vrf_nonce.checked_add(1).ok_or(MoleError::MathOverflow)?;
    let nonce = game.vrf_nonce;
    game.pending_vrf = true;
    game.status = STATUS_AWAITING_RANDOMNESS;
    game.vrf_requested_at = Clock::get()?.unix_timestamp;

    let mut caller_seed = [0u8; 32];
    caller_seed[..8].copy_from_slice(&nonce.to_le_bytes());
    let params = RequestRandomnessParams {
        payer: ctx.accounts.payer.key(),
        oracle_queue: ctx.accounts.oracle_queue.key(),
        callback_program_id: crate::ID,
        callback_discriminator: crate::instruction::CallbackPickMoles::DISCRIMINATOR.to_vec(),
        caller_seed,
        accounts_metas: Some(vec![SerializableAccountMeta {
            pubkey: ctx.accounts.game.key(),
            is_signer: false,
            is_writable: true,
        }]),
        callback_args: Some(nonce.to_le_bytes().to_vec()),
        ..Default::default()
    };
    let ix = create_request_randomness_ix_const(params, PROGRAM_IDENTITY);
    ctx.accounts
        .invoke_signed_vrf(&ctx.accounts.payer.to_account_info(), &ix)?;
    msg!("VRF requested for game {} (nonce {})", game_id, nonce);
    Ok(())
}

#[vrf]
#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct StartRound<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    /// CHECK: The VRF oracle queue. Base-layer queue for base-layer requests.
    #[account(mut, address = vrf::consts::DEFAULT_QUEUE)]
    pub oracle_queue: UncheckedAccount<'info>,
}

/// VRF callback: the oracle delivers verified randomness (with an on-chain
/// proof checked by the VRF program before this is invoked). Derives the
/// two mole seats and opens trading. Idempotent: a duplicate callback for
/// the same nonce is a no-op. A callback for a superseded nonce is rejected.
pub fn callback_pick_moles_handler(
    ctx: Context<CallbackPickMoles>,
    randomness: [u8; 32],
    nonce: u64,
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.pending_vrf, MoleError::VrfNotPending);
    require!(nonce == game.vrf_nonce, MoleError::StaleCallback);
    if game.status == STATUS_TRADING {
        // Duplicate fulfillment: already settled, do nothing twice.
        return Ok(());
    }
    require!(game.status == STATUS_AWAITING_RANDOMNESS, MoleError::NotInLobby);
    let (a, b) = derive_moles(&randomness);
    game.mole_a = a;
    game.mole_b = b;
    game.randomness_source = SOURCE_VRF;
    game.status = STATUS_TRADING;
    game.pending_vrf = false;
    msg!("VRF moles picked for game {}: seats {} and {}", game.game_id, a, b);
    Ok(())
}

#[vrf_callback]
#[derive(Accounts)]
pub struct CallbackPickMoles<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
}

/// Demo fallback: pick the moles from a hash of (caller seed, current slot,
/// game id). This is NOT verifiable randomness and the game records
/// SOURCE_DEMO on chain so nobody can mistake it for VRF. Allowed only
/// while still in the lobby, or after a VRF request has gone unanswered
/// for VRF_TIMEOUT_SECS. Creator-only.
pub fn pick_moles_demo_handler(ctx: Context<PickMolesDemo>, game_id: u64, seed: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require_keys_eq!(game.creator, ctx.accounts.creator.key(), MoleError::NotCreator);
    require!(
        game.status == STATUS_LOBBY
            || (game.status == STATUS_AWAITING_RANDOMNESS
                && Clock::get()?.unix_timestamp - game.vrf_requested_at >= VRF_TIMEOUT_SECS),
        MoleError::VrfNotTimedOut
    );
    require!(game.status != STATUS_TRADING, MoleError::MolesAlreadyPicked);
    let slot = Clock::get()?.slot;
    let randomness = demo_randomness(seed, slot, game_id);
    let (a, b) = derive_moles(&randomness);
    game.mole_a = a;
    game.mole_b = b;
    game.randomness_source = SOURCE_DEMO;
    game.status = STATUS_TRADING;
    game.pending_vrf = false;
    msg!(
        "DEMO randomness moles picked for game {}: seats {} and {} (NOT VRF)",
        game_id, a, b
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64, seed: u64)]
pub struct PickMolesDemo<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub creator: Signer<'info>,
}
