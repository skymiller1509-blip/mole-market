use anchor_lang::prelude::*;

use crate::{constants::*, error::MoleError, logic::*, state::*};

/// Apply one trade to a seat. Shared by human `trade` and `bot_trade`.
/// - `amount` points leave the seat balance and open/extend a position.
/// - Price moves with the trade (buy up, sell down).
/// - 2% of the amount leaks out of the shared pot into the mole pool:
///   everyone can watch the pot shrink, nobody sees who collects it.
fn apply_trade(game: &mut Game, seat_index: usize, is_buy: bool, amount: u64) -> Result<()> {
    let seat = &mut game.seats[seat_index];
    require!(amount > 0, MoleError::AmountZero);
    require!(amount <= seat.balance, MoleError::InsufficientBalance);

    let price_before = game.price;
    let new_shares = shares_for(amount, price_before);
    seat.entry_price = blended_entry(seat.shares, seat.entry_price, if is_buy { new_shares } else { -new_shares }, price_before);
    seat.shares = seat
        .shares
        .checked_add(if is_buy { new_shares } else { -new_shares })
        .ok_or(MoleError::MathOverflow)?;
    seat.balance = seat.balance.checked_sub(amount).ok_or(MoleError::MathOverflow)?;

    let impact = price_impact(amount);
    game.price = if is_buy {
        price_before.saturating_add(impact).min(PRICE_CAP)
    } else {
        price_before.saturating_sub(impact).max(PRICE_FLOOR)
    };

    let leak = leak_for(amount);
    game.pot = game.pot.saturating_sub(leak);
    game.mole_pool = game.mole_pool.saturating_add(leak);
    game.tick = game.tick.saturating_add(1);
    Ok(())
}

/// Check the signer may act for a seat: the seat owner, or an authorized
/// session key that has not expired yet.
fn authorize(game: &Game, seat_index: usize, signer: &Pubkey, slot: u64) -> Result<()> {
    let seat = &game.seats[seat_index];
    if signer == &seat.owner {
        return Ok(());
    }
    if !seat.session_key.eq(&Pubkey::default()) && signer == &seat.session_key {
        require!(slot <= seat.session_expiry, MoleError::SessionExpired);
        return Ok(());
    }
    Err(MoleError::NotSeatOwner.into())
}

/// Human trade: buy (long) or sell (short) `amount` points at the index.
/// Runs on the base layer, or gasless on the Ephemeral Rollup once the
/// game is delegated. The signer is the seat owner or its session key.
pub fn trade_handler(
    ctx: Context<Trade>,
    game_id: u64,
    seat: u8,
    is_buy: bool,
    amount: u64,
) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_TRADING, MoleError::NotTrading);
    require!((seat as usize) < MAX_SEATS, MoleError::SeatOutOfRange);
    require!(!game.seats[seat as usize].is_empty(), MoleError::SeatEmpty);
    let slot = Clock::get()?.slot;
    authorize(game, seat as usize, &ctx.accounts.signer.key(), slot)?;
    apply_trade(game, seat as usize, is_buy, amount)?;
    msg!(
        "Game {} seat {} {} {} pts, price {}, pot {}, mole pool {}",
        game_id,
        seat,
        if is_buy { "bought" } else { "sold" },
        amount,
        game.price,
        game.pot,
        game.mole_pool
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64, seat: u8, is_buy: bool, amount: u64)]
pub struct Trade<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub signer: Signer<'info>,
}

/// Bot trade: anyone (the demo runner, a crank, any player) can trigger a
/// bot seat's next move. The move is deterministic from (game_id, tick,
/// seat), so the table keeps playing while the human watches. Bots are the
/// reason the demo works solo.
pub fn bot_trade_handler(ctx: Context<BotTrade>, game_id: u64, seat: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_TRADING, MoleError::NotTrading);
    require!((seat as usize) < MAX_SEATS, MoleError::SeatOutOfRange);
    let bot_seat = &game.seats[seat as usize];
    require!(!bot_seat.is_empty(), MoleError::SeatEmpty);
    require!(bot_seat.is_bot, MoleError::NotABot);
    if bot_seat.balance == 0 {
        // Broke bot: nothing to do, but not an error (keeps cranks simple).
        return Ok(());
    }
    let (is_buy, want) = bot_decision(game_id, game.tick, seat);
    let amount = want.min(bot_seat.balance);
    apply_trade(game, seat as usize, is_buy, amount)?;
    msg!("Game {} bot seat {} traded {}", game_id, seat, amount);
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64, seat: u8)]
pub struct BotTrade<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub payer: Signer<'info>,
}

/// End the round: mark every position to the final price, pay the moles
/// their skimmed pool, split the surviving pot among the honest players,
/// and publish the mole seats. The reveal is the payoff of the whole game.
pub fn end_round_handler(ctx: Context<EndRound>, game_id: u64) -> Result<()> {
    let game = &mut ctx.accounts.game;
    require!(game.status == STATUS_TRADING, MoleError::NotTrading);
    let price = game.price;
    let pot = game.pot;
    let mole_pool = game.mole_pool;
    for i in 0..MAX_SEATS {
        let seat = game.seats[i];
        game.seats[i].payout = payout_for(
            seat.balance,
            seat.shares,
            seat.entry_price,
            price,
            game.is_mole(i as u8),
            pot,
            mole_pool,
        );
    }
    game.status = STATUS_REVEALED;
    msg!(
        "Game {} revealed: moles were seats {} and {}. Pot {} survived, mole pool {} paid out.",
        game_id, game.mole_a, game.mole_b, pot, mole_pool
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct EndRound<'info> {
    #[account(
        mut,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump = game.bump
    )]
    pub game: Account<'info, Game>,
    pub payer: Signer<'info>,
}
