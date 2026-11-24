//! Pure game logic: no accounts, no sysvars, fully unit-testable.
//! The on-chain instructions call these helpers so the math is identical
//! in tests and on chain.

use sha2::{Digest, Sha256};

use crate::constants::*;

/// Pick two distinct mole seats (0..8) from 32 bytes of randomness.
/// Uniform: first seat is r[0] % 8, second is r[1] % 7 mapped to skip the
/// first seat, so every unordered pair is equally likely.
pub fn derive_moles(randomness: &[u8; 32]) -> (u8, u8) {
    let a = randomness[0] % 8;
    let mut b = randomness[1] % 7;
    if b >= a {
        b += 1;
    }
    (a, b)
}

/// Demo entropy: SHA-256 of (seed || slot || game_id). This is NOT
/// verifiable randomness: anyone can recompute it. It exists only so the
/// game is playable when the VRF oracle does not answer, and the game
/// records SOURCE_DEMO on chain when it is used.
pub fn demo_randomness(seed: u64, slot: u64, game_id: u64) -> [u8; 32] {
    let mut input = Vec::with_capacity(24);
    input.extend_from_slice(&seed.to_le_bytes());
    input.extend_from_slice(&slot.to_le_bytes());
    input.extend_from_slice(&game_id.to_le_bytes());
    let digest = Sha256::digest(&input);
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&digest);
    hash_bytes
}

/// Points that leak from the pot into the mole pool on a trade of `amount`.
pub fn leak_for(amount: u64) -> u64 {
    amount * LEAK_BPS / 10_000
}

/// Price impact of a trade: 1 point of price per 10 points traded, min 1.
pub fn price_impact(amount: u64) -> u64 {
    (amount / 10).max(1)
}

/// New shares bought/sold for `amount` points at `price` (1e4 scale).
pub fn shares_for(amount: u64, price: u64) -> i64 {
    ((amount as u128 * 10_000u128) / price as u128) as i64
}

/// Volume-weighted average entry price after adding `new_shares` at `price`
/// to a position of `old_shares` at `old_entry`. All values 1e4 scale.
pub fn blended_entry(old_shares: i64, old_entry: u64, new_shares: i64, price: u64) -> u64 {
    if old_shares == 0 {
        return price;
    }
    let old_abs = old_shares.unsigned_abs() as u128;
    let new_abs = new_shares.unsigned_abs() as u128;
    if old_abs + new_abs == 0 {
        return old_entry;
    }
    ((old_entry as u128 * old_abs + price as u128 * new_abs) / (old_abs + new_abs)) as u64
}

/// Mark-to-market P&L of a position, in points (can be negative).
/// shares > 0 is long, shares < 0 is short.
pub fn position_pnl(shares: i64, entry_price: u64, price: u64) -> i128 {
    if shares == 0 || entry_price == 0 {
        return 0;
    }
    shares as i128 * (price as i128 - entry_price as i128) / 10_000
}

/// Final payout for one seat at reveal.
/// Honest players keep balance + P&L and split the surviving pot.
/// Moles keep balance + P&L and split the skimmed mole pool.
pub fn payout_for(
    balance: u64,
    shares: i64,
    entry_price: u64,
    price: u64,
    is_mole: bool,
    pot: u64,
    mole_pool: u64,
) -> u64 {
    let base = balance as i128 + position_pnl(shares, entry_price, price);
    let bonus = if is_mole {
        mole_pool as i128 / NUM_MOLES as i128
    } else {
        pot as i128 / (MAX_SEATS - NUM_MOLES) as i128
    };
    (base + bonus).max(0) as u64
}

/// Deterministic bot strategy from (game_id, tick, seat).
/// Returns (is_buy, amount). Amount is 50..200 points.
pub fn bot_decision(game_id: u64, tick: u64, seat: u8) -> (bool, u64) {
    let mut input = Vec::with_capacity(17);
    input.extend_from_slice(&game_id.to_le_bytes());
    input.extend_from_slice(&tick.to_le_bytes());
    input.push(seat);
    let bytes = Sha256::digest(&input);
    let is_buy = bytes[0] % 2 == 0;
    let amount = 50 + (bytes[1] as u64 % 151);
    (is_buy, amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moles_are_always_two_distinct_seats() {
        for seed in 0..10_000u64 {
            let r = demo_randomness(seed, 12345, 7);
            let (a, b) = derive_moles(&r);
            assert!(a < 8 && b < 8, "seat out of range: {a} {b}");
            assert_ne!(a, b, "moles must differ");
        }
    }

    #[test]
    fn moles_cover_all_pairs() {
        // Over many draws every unordered pair (28 of them) must appear.
        let mut seen = [[false; 8]; 8];
        for seed in 0..20_000u64 {
            let r = demo_randomness(seed, 999, 1);
            let (a, b) = derive_moles(&r);
            seen[a as usize][b as usize] = true;
        }
        let mut pairs = 0;
        for a in 0..8 {
            for b in 0..8 {
                if a != b && seen[a][b] {
                    pairs += 1;
                }
            }
        }
        // 56 ordered pairs (28 unordered x 2 directions).
        assert_eq!(pairs, 56, "not all ordered mole pairs reachable");
    }

    #[test]
    fn leak_is_two_percent() {
        assert_eq!(leak_for(1_000), 20);
        assert_eq!(leak_for(100), 2);
        assert_eq!(leak_for(50), 1);
        assert_eq!(leak_for(49), 0);
    }

    #[test]
    fn price_impact_and_shares() {
        assert_eq!(price_impact(1_000), 100);
        assert_eq!(price_impact(5), 1);
        assert_eq!(shares_for(1_000, 10_000), 1_000);
        assert_eq!(shares_for(500, 20_000), 250);
    }

    #[test]
    fn pnl_long_and_short() {
        // Long 1000 shares entered at 100.00, price now 110.00 -> +100.
        assert_eq!(position_pnl(1_000, 10_000, 11_000), 100);
        // Same long, price fell to 90.00 -> -100.
        assert_eq!(position_pnl(1_000, 10_000, 9_000), -100);
        // Short 1000 shares entered at 100.00, price now 90.00 -> +100.
        assert_eq!(position_pnl(-1_000, 10_000, 9_000), 100);
        // Flat position -> 0.
        assert_eq!(position_pnl(0, 10_000, 11_000), 0);
    }

    #[test]
    fn payouts_split_pot_and_pool() {
        // Honest player, flat, balance 1000, pot 8000, pool 0 -> 1000 + 8000/6.
        assert_eq!(
            payout_for(1_000, 0, 0, 10_000, false, 8_000, 0),
            1_000 + 8_000 / 6
        );
        // Mole, flat, balance 1000, pool 400 -> 1000 + 200.
        assert_eq!(
            payout_for(1_000, 0, 0, 10_000, true, 8_000, 400),
            1_000 + 200
        );
        // Losing long can never pay out negative.
        assert_eq!(payout_for(0, 10_000, 10_000, 1_000, false, 0, 0), 0);
    }

    #[test]
    fn bot_decisions_are_deterministic_and_bounded() {
        let first = bot_decision(42, 7, 3);
        assert_eq!(first, bot_decision(42, 7, 3));
        assert_ne!(first, bot_decision(42, 8, 3));
        for tick in 0..500u64 {
            let (_, amount) = bot_decision(42, tick, 3);
            assert!((50..=200).contains(&amount));
        }
    }

    #[test]
    fn demo_randomness_differs_by_input() {
        assert_ne!(demo_randomness(1, 1, 1), demo_randomness(2, 1, 1));
        assert_ne!(demo_randomness(1, 1, 1), demo_randomness(1, 2, 1));
        assert_ne!(demo_randomness(1, 1, 1), demo_randomness(1, 1, 2));
    }
}
