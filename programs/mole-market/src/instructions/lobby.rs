use anchor_lang::prelude::*;

use crate::{constants::*, error::MoleError, state::*};

/// Create a game. The creator takes seat 0. The pot starts at 8,000 points
/// (8 x 1,000 buy-in) and the index price at 100.00.
pub fn handler(ctx: Context<CreateGame>, game_id: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    game.creator = ctx.accounts.creator.key();
    game.game_id = game_id;
    game.status = STATUS_LOBBY;
    game.randomness_source = SOURCE_NONE;
    game.pot = STARTING_POT;
    game.mole_pool = 0;
    game.price = STARTING_PRICE;
    game.tick = 0;
    game.mole_a = u8::MAX;
    game.mole_b = u8::MAX;
    game.vrf_nonce = 0;
    game.pending_vrf = false;
    game.vrf_requested_at = 0;
    game.bump = ctx.bumps.game;
    game.seats[0].owner = ctx.accounts.creator.key();
    game.seats[0].is_bot = false;
    game.seats[0].balance = STARTING_BALANCE;
    msg!("Mole Market game {} created, pot {}", game_id, game.pot);
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct CreateGame<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + Game::SIZE,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// A human takes an empty seat. Balance starts at 1,000 points.
pub fn join_handler(ctx: Context<JoinGame>, game_id: u64, seat: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_LOBBY, MoleError::NotInLobby);
    require!((seat as usize) < MAX_SEATS, MoleError::SeatOutOfRange);
    let slot = &mut game.seats[seat as usize];
    require!(slot.is_empty(), MoleError::SeatTaken);
    slot.owner = ctx.accounts.player.key();
    slot.is_bot = false;
    slot.balance = STARTING_BALANCE;
    msg!("Seat {} joined game {}", seat, game_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64, seat: u8)]
pub struct JoinGame<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

/// Fill every empty seat with a bot so a solo demo has a full table.
/// Bot owners are deterministic PDAs: ["bot", game_id, seat]. Anyone can
/// call this; bots never sign, their trades run through `bot_trade`.
pub fn fill_bots_handler(ctx: Context<FillBots>, game_id: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_LOBBY, MoleError::NotInLobby);
    let mut filled = 0u8;
    for (i, seat) in game.seats.iter_mut().enumerate() {
        if seat.is_empty() {
            let (bot, _) = Pubkey::find_program_address(
                &[b"bot", game_id.to_le_bytes().as_ref(), &[i as u8]],
                &crate::ID,
            );
            seat.owner = bot;
            seat.is_bot = true;
            seat.balance = STARTING_BALANCE;
            filled += 1;
        }
    }
    msg!("Filled {} bot seats in game {}", filled, game_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct FillBots<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub payer: Signer<'info>,
}

/// Authorize a throwaway session key for one seat, so the seat owner signs
/// once on the base layer and the session key signs gasless ER trades.
pub fn create_session_handler(
    ctx: Context<CreateSession>,
    game_id: u64,
    seat: u8,
    session_key: Pubkey,
    expiry_slot: u64,
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!((seat as usize) < MAX_SEATS, MoleError::SeatOutOfRange);
    let slot = &mut game.seats[seat as usize];
    require!(!slot.is_empty(), MoleError::SeatEmpty);
    require_keys_eq!(slot.owner, ctx.accounts.owner.key(), MoleError::NotSeatOwner);
    slot.session_key = session_key;
    slot.session_expiry = expiry_slot;
    msg!("Session key set for seat {} in game {}", seat, game_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64, seat: u8, session_key: Pubkey, expiry_slot: u64)]
pub struct CreateSession<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub owner: Signer<'info>,
}
