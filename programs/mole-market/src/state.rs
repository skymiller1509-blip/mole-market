use anchor_lang::prelude::*;

use crate::constants::*;

/// One seat at the table. A seat is either a human (wallet) or a bot.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct Seat {
    /// Wallet that owns the seat, or the bot PDA for bot seats.
    pub owner: Pubkey,
    /// True for bot-filled seats.
    pub is_bot: bool,
    /// Tradable points balance.
    pub balance: u64,
    /// Net shares: positive = long, negative = short.
    pub shares: i64,
    /// Volume-weighted average entry price of the current position (1e4 scale).
    pub entry_price: u64,
    /// Final payout after end_round (0 until revealed).
    pub payout: u64,
    /// Authorized session key (gasless ER trades). Pubkey::default() = none.
    pub session_key: Pubkey,
    /// Slot after which the session key stops working.
    pub session_expiry: u64,
}

impl Seat {
    pub const SIZE: usize = 32 + 1 + 8 + 8 + 8 + 8 + 32 + 8; // 105

    pub fn is_empty(&self) -> bool {
        self.owner == Pubkey::default()
    }
}

/// The whole game lives in one PDA so it can be delegated to an Ephemeral
/// Rollup as a single unit (and, in future, gated by a Private ER permission).
#[account]
pub struct Game {
    pub creator: Pubkey,
    pub game_id: u64,
    pub status: u8,
    /// SOURCE_NONE / SOURCE_VRF / SOURCE_DEMO. On-chain honesty label.
    pub randomness_source: u8,
    /// Shared pot, in points. Every trade leaks 2% of its size out of this.
    pub pot: u64,
    /// Points skimmed by the moles so far. Paid out to the moles at reveal.
    pub mole_pool: u64,
    /// Index price, 1e4 scale.
    pub price: u64,
    /// Trade counter. Drives the deterministic bot strategy.
    pub tick: u64,
    /// Mole seat indices. 255 = not picked yet.
    pub mole_a: u8,
    pub mole_b: u8,
    /// Increments on every VRF request; the callback must echo it back.
    pub vrf_nonce: u64,
    /// True while a VRF request is outstanding.
    pub pending_vrf: bool,
    /// Unix timestamp of the last VRF request (for the demo fallback timeout).
    pub vrf_requested_at: i64,
    pub bump: u8,
    pub seats: [Seat; MAX_SEATS],
}

impl Game {
    pub const SIZE: usize = 32 + 8 + 1 + 1 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + 1 + 8 + 1
        + MAX_SEATS * Seat::SIZE; // 934

    pub fn seats_filled(&self) -> bool {
        self.seats.iter().all(|s| !s.is_empty())
    }

    pub fn is_mole(&self, seat: u8) -> bool {
        seat == self.mole_a || seat == self.mole_b
    }
}
