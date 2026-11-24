use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{
    anchor::{commit, delegate},
    cpi::DelegateConfig,
    ephem::{FoldableIntentBuilder, MagicIntentBundleBuilder},
};

use crate::{constants::*, state::*};

/// Delegate the game PDA to the Ephemeral Rollup (devnet EU validator).
/// Runs on the base layer. After this, trades execute on the rollup:
/// ~10 ms blocks, zero-fee transactions. The delegation deposit
/// (~0.003 SOL) is paid by the caller.
pub fn delegate_game_handler(ctx: Context<DelegateGame>, game_id: u64) -> Result<()> {
    ctx.accounts.delegate_game(
        &ctx.accounts.payer,
        &[b"game", game_id.to_le_bytes().as_ref()],
        DelegateConfig {
            validator: Some(ER_VALIDATOR_EU_DEVNET),
            ..Default::default()
        },
    )?;
    msg!("Game {} delegated to the Ephemeral Rollup", game_id);
    Ok(())
}

#[delegate]
#[derive(Accounts)]
#[instruction(game_id: u64)]
pub struct DelegateGame<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: The game PDA to delegate.
    #[account(
        mut,
        del,
        seeds = [b"game", game_id.to_le_bytes().as_ref()],
        bump
    )]
    pub game: UncheckedAccount<'info>,
}

/// Commit the rollup state back to the base layer, keeping the game
/// delegated so play can continue. Runs on the Ephemeral Rollup.
pub fn commit_game_handler(ctx: Context<CommitGame>) -> Result<()> {
    MagicIntentBundleBuilder::new(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.magic_context.to_account_info(),
        ctx.accounts.magic_program.to_account_info(),
    )
    .commit(&[ctx.accounts.game.to_account_info()])
    .build_and_invoke()?;
    msg!("Game committed to the base layer");
    Ok(())
}

#[commit]
#[derive(Accounts)]
pub struct CommitGame<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
}

/// Commit the final state and return the game PDA to the base layer.
/// Runs on the Ephemeral Rollup. Always allowed, even after the plain
/// commit limit is hit, so a game can always exit.
pub fn undelegate_game_handler(ctx: Context<UndelegateGame>) -> Result<()> {
    MagicIntentBundleBuilder::new(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.magic_context.to_account_info(),
        ctx.accounts.magic_program.to_account_info(),
    )
    .commit_and_undelegate(&[ctx.accounts.game.to_account_info()])
    .build_and_invoke()?;
    msg!("Game committed and undelegated");
    Ok(())
}

#[commit]
#[derive(Accounts)]
pub struct UndelegateGame<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
}
