use anchor_lang::prelude::*;

/// Number of seats in one game. 8 players, 2 of them moles.
pub const MAX_SEATS: usize = 8;
/// Number of moles picked per round.
pub const NUM_MOLES: usize = 2;
/// Points every player starts with.
pub const STARTING_BALANCE: u64 = 1_000;
/// Points in the shared pot at game start (8 players x 1,000 buy-in).
pub const STARTING_POT: u64 = 8_000;
/// Index price at game start, 1e4 scale (10_000 = 100.00).
pub const STARTING_PRICE: u64 = 10_000;
/// Price cannot fall below this (1e4 scale).
pub const PRICE_FLOOR: u64 = 1_000;
/// Price cannot rise above this (1e4 scale).
pub const PRICE_CAP: u64 = 100_000;
/// Leak rate in basis points: 200 bps = 2% of every trade leaks from the pot.
pub const LEAK_BPS: u64 = 200;
/// Seconds after a VRF request before the demo-randomness fallback is allowed.
pub const VRF_TIMEOUT_SECS: i64 = 180;
/// Devnet EU Ephemeral Rollup validator (validator 1.x line).
pub const ER_VALIDATOR_EU_DEVNET: Pubkey = pubkey!("MEUGGrYPxKk17hCr7wpT6s8dtNokZj5U2L57vjYMS8e");

/// Game status values.
pub const STATUS_LOBBY: u8 = 0;
pub const STATUS_AWAITING_RANDOMNESS: u8 = 1;
pub const STATUS_TRADING: u8 = 2;
pub const STATUS_REVEALED: u8 = 3;

/// Randomness source values. Stored on chain so the UI can show it honestly.
pub const SOURCE_NONE: u8 = 0;
/// Real MagicBlock VRF (verified on-chain randomness + proof).
pub const SOURCE_VRF: u8 = 1;
/// Demo fallback: hash of caller seed + current slot. NOT verifiable randomness.
pub const SOURCE_DEMO: u8 = 2;
