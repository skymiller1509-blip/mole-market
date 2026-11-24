pub mod constants;
pub mod error;
pub mod instructions;
pub mod logic;
pub mod state;

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK");

#[ephemeral]
#[program]
pub mod mole_market {
    use super::*;

    pub fn create_game(ctx: Context<CreateGame>, game_id: u64) -> Result<()> {
        instructions::lobby::handler(ctx, game_id)
    }

    pub fn join_game(ctx: Context<JoinGame>, game_id: u64, seat: u8) -> Result<()> {
        instructions::lobby::join_handler(ctx, game_id, seat)
    }

    pub fn fill_bots(ctx: Context<FillBots>, game_id: u64) -> Result<()> {
        instructions::lobby::fill_bots_handler(ctx, game_id)
    }

    pub fn create_session(
        ctx: Context<CreateSession>,
        game_id: u64,
        seat: u8,
        session_key: Pubkey,
        expiry_slot: u64,
    ) -> Result<()> {
        instructions::lobby::create_session_handler(ctx, game_id, seat, session_key, expiry_slot)
    }

    pub fn start_round(ctx: Context<StartRound>, game_id: u64) -> Result<()> {
        instructions::round::start_round_handler(ctx, game_id)
    }

    pub fn callback_pick_moles(
        ctx: Context<CallbackPickMoles>,
        randomness: [u8; 32],
        nonce: u64,
    ) -> Result<()> {
        instructions::round::callback_pick_moles_handler(ctx, randomness, nonce)
    }

    pub fn pick_moles_demo(ctx: Context<PickMolesDemo>, game_id: u64, seed: u64) -> Result<()> {
        instructions::round::pick_moles_demo_handler(ctx, game_id, seed)
    }

    pub fn trade(
        ctx: Context<Trade>,
        game_id: u64,
        seat: u8,
        is_buy: bool,
        amount: u64,
    ) -> Result<()> {
        instructions::trade::trade_handler(ctx, game_id, seat, is_buy, amount)
    }

    pub fn bot_trade(ctx: Context<BotTrade>, game_id: u64, seat: u8) -> Result<()> {
        instructions::trade::bot_trade_handler(ctx, game_id, seat)
    }

    pub fn end_round(ctx: Context<EndRound>, game_id: u64) -> Result<()> {
        instructions::trade::end_round_handler(ctx, game_id)
    }

    pub fn delegate_game(ctx: Context<DelegateGame>, game_id: u64) -> Result<()> {
        instructions::delegation::delegate_game_handler(ctx, game_id)
    }

    pub fn commit_game(ctx: Context<CommitGame>) -> Result<()> {
        instructions::delegation::commit_game_handler(ctx)
    }

    pub fn undelegate_game(ctx: Context<UndelegateGame>) -> Result<()> {
        instructions::delegation::undelegate_game_handler(ctx)
    }
}
