# Mole Market — Among Us meets day trading

**Eight traders, one shared pot, two secret moles skimming it — and everyone can see the leak.**

> **Demo video:** TODO — not filmed yet. See `VIDEO-SCRIPT.md` for the script. Film the DEMO mole pick only — never the REAL VRF button (see "What is real" below).
>
> **Submission:** https://build.magicblock.app/?stage=blitz#submit — deadline Fri 27 Nov 2026, 15:00 WAT. The form was CLOSED when last checked (2026-11-23); re-check it in a live browser before submitting.

**MagicBlock Solana Blitz v10 — Entertainment Finance.**
Eight players trade one shared pot while verifiable randomness secretly picks
two moles who skim it. Everyone sees the leak, nobody sees the leaker.
Bots fill empty seats so the demo works solo.

> **Honest grade (Eddie's standing rule):** captivating yes, fantastic yes,
> wow partly (the leak meter + reveal land, but the moles are only hidden by
> the UI in this build — see "What is real" below), unique yes, original yes,
> mind-blowing partly (the VRF pick and the gasless rollup trades are real
> MagicBlock tech; true role secrecy needs a Private ER, which is the
> documented next step, not a shipped feature).

## The one flow

1. **Create a game** — you take seat 0. The shared pot starts at 8,000 points.
2. **Join / fill bots** — humans join seats 1–7, or one click fills every empty
   seat with a bot (deterministic bot PDAs, no keypairs needed).
3. **Start the round** — the program asks the MagicBlock VRF oracle for
   randomness. The oracle's callback secretly picks 2 mole seats and opens
   trading.
4. **Trade** — buy (long) or sell (short) the index. Every trade moves the
   price and **leaks 2% of its size out of the shared pot into the mole
   pool**. The pot meter visibly drains; the UI never says who collects it.
5. **Bots trade** — one click runs every bot seat's deterministic turn, so a
   solo demo keeps moving.
6. **Reveal** — anyone ends the round. The two moles are published, the
   skimmed pool is split between them, and the surviving pot is split among
   the six honest players. Every payout is computed on chain from the final
   price.

Points are virtual. No real money, devnet only.

## What is real vs demo / simulated

| Piece | Status | Evidence |
|---|---|---|
| Anchor program (`mole_market`) | **Real, built** | `target/deploy/mole_market.so` (482,120 bytes), IDL in `target/idl/mole_market.json` |
| Game logic, leak, payouts, bots | **Real, tested** | 9 unit + 3 LiteSVM integration tests, 12/12 pass (see Tests) |
| VRF mole pick (`start_round` + `callback_pick_moles`) | **Requested live, callback NEVER observed** | `start_round` ran three times on devnet (games 4244 and 4245 on 21 Nov, game 4246 on 23 Nov): the VRF program accepted all three requests (tx `5AGxygA4bW7bv1gdSrthwCSJhGs5HSGxqdCLpBq6PR9ogruw7rKY2tL9Uu26biJ8in8sKrcWKRZyAU5SyBKRUkVk`, `3D7hnduVgpEX8HFE2PcknpakXETqkhCGMCkknkjQCsZERjDM18XWPGumyvvXA63cDa1YfTi2rjSCKFfZ5LqYN4tS` and `4ooJzHcCsrTLXafXcU5QruedRgXDzDmtZ7yTbAHmvuanAVYb5UBvQw4DvjN8244Cv9LjjGAkXooAPJyjramwYLgA`). No callback landed on any of the three games (game 4246 polled for a full 10 minutes, still "Awaiting randomness"). The demo fallback (`pick_moles_demo`, labelled DEMO on chain) opened trading instead. Do NOT claim live VRF works until a callback is seen on chain. |
| Demo mole pick (`pick_moles_demo`) | **Real, tested, labelled** | Hash of caller seed + slot. The game stores `randomness_source = DEMO` on chain and the UI shows "DEMO (not VRF)". Only usable in the lobby, or 180s after an unanswered VRF request. Never presented as VRF. |
| Ephemeral Rollup delegation | **PROVEN live on devnet 2026-11-21** | Game 4245 delegated mid-game (tx `4i4dxP1eWqkWoYj5LAhAo82P3H2b2Hh9c2Ry6GHLKUQEyerEuW2nrRHXJxoGHjWVsx8ZtBHdAm7jxvhAB5oNyZCt`); router `getDelegationStatus` returned `isDelegated: true`, fqdn `https://devnet-eu.magicblock.app/`; `endRound` ran ON the ER (tx `5d1yXfRExp4gZofYCxecGhP9XK5k2xsuP8UjndGFpm9sUja5aEZXFQs8UiTZxbamFtTyzAhkkLLftVQvjtwHQary`); `undelegateGame` settled back to base (commit tx `3cg3z8beWv2UqwLiYaUGDDS7qiZt3ZbUsiqV2WGuPeJxDnpn2SWH7REwVjRwwZCrDo7YQ4oSdtgapMif6uEa9AyV`, base owner restored, status Revealed). Note: after delegation, base-layer txs on the game account fail with `AccountOwnedByWrongProgram`; game ops must go through the ER endpoint. |
| Role hiding ("nobody sees the leaker") | **UI-level only in this build — NOT cryptographic** | The UI never renders mole identities until reveal, and the game stores them in the delegated game account. **Anyone who fetches the game account on chain can read `mole_a` / `mole_b` before reveal.** True secrecy needs a Private Ephemeral Rollup (TEE): delegate the game account to the TEE validator and gate it with an `EphemeralPermission` (`access-control` feature, `CreateEphemeralPermissionCpi`). That is the next step, not a shipped feature. Do not run this with real money. |
| Session keys (gasless trades) | **Real, tested** | `create_session` authorizes a throwaway key per seat; trades accept the owner or an unexpired session key. Covered by the integration test. |
| Bots | **Real** | Deterministic strategy from `(game_id, tick, seat)`; `bot_trade` is permissionless so anyone can crank the table. |
| Frontend | **Real, builds, serves, reads live devnet state** | React + Vite + `@anchor-lang/core` 1.0.2 + `@magicblock-labs/ephemeral-rollups-sdk` 0.18.0. `npm run build` passes; `vite preview` serves index (HTTP 200) and bundle (HTTP 200). Verified 2026-11-23: the frontend's exact data path (Anchor `game.fetch` over `https://rpc.magicblock.app/devnet`, account-owner check, router `getDelegationStatus`, 3 s HTTP polling, no websockets) reads game 4245 live — status Revealed, pot 7974, mole pool 26, moles 5/7, source DEMO — and game 4246 still "Awaiting randomness". The app opens on game 4245 and loads its state before any wallet is connected (read-only program). Honest limit: no real browser click-through was possible in this VM (no display), so button flows were verified at code + data-path level, not by clicking in a browser. |

## Tech stack

- Anchor program (`mole_market`) on Solana devnet, Rust; MagicBlock Ephemeral Rollups SDK 0.18.0 and VRF SDK 0.18.0
- Frontend: React + Vite + `@anchor-lang/core` 1.0.2, wallet adapter, "Make demo key" throwaway keypair
- Tests: 9 unit + 3 LiteSVM integration tests, 12/12 pass

## Verified constants (read from the installed SDK / live network, not guessed)

- Program ID: `73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK`
- VRF program: `Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz` (from
  `ephemeral-vrf-sdk 0.18.0` `src/consts.rs`; the legacy global identity
  `9irBy75QS2BN81FUgXuHcjqceJJRuc9oDkAe8TKVvvAw` is deprecated and unused)
- VRF base queue (devnet): `Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh` —
  verified live on devnet 2026-11-20 (exists, owned by the VRF program,
  210,255,079 lamports)
- VRF ephemeral queue: `5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc`
- Delegation program: `DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh` (verified
  live on devnet, executable)
- Permission program: `ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1`
- Devnet EU ER: `https://devnet-eu.magicblock.app` (`magicblock-core 1.1.1`,
  commit `8886db2`); devnet US: `https://devnet-us.magicblock.app` (2.0);
  base RPC: `https://rpc.magicblock.app/devnet`
- VRF fee on the base queue: 500,000 lamports per request (paid by the caller).
  Requests inside an Ephemeral Rollup are fee-exempt.
- Delegation deposit: ~0.003 SOL per session (paid at undelegation).

## Tests

```bash
cargo test -p mole-market --lib        # 9 unit tests (logic.rs)
cargo test -p mole-market --test integration  # 3 LiteSVM integration tests
```

Last run 2026-11-20: **12 passed, 0 failed.**

- `full_game_loop_with_bots`: create → join → fill bots (8 seats) → demo
  mole pick → human buy 500 (pot leaks exactly 10) → wrong-signer trade
  rejected → session-key trade → 5 bot trades → reveal; every seat's payout
  checked against the formula; moles get the skimmed pool.
- `vrf_request_fails_without_vrf_program`: `start_round` fails on LiteSVM
  (no VRF program there) and the game stays in the lobby.
- `delegate_fails_without_delegation_program`: `delegate_game` fails on
  LiteSVM (no Delegation Program there).

## Try it yourself

### Program

```bash
# Toolchain (pinned): Solana CLI 3.1.9, Rust 1.89.0, Anchor CLI 1.0.2, Node 24
anchor build
anchor test          # runs cargo test (LiteSVM)
```

### Frontend

```bash
cd app
npm install
npm run build        # tsc + vite
npm run dev          # http://localhost:5173
```

The UI defaults to devnet. Connect Phantom, or "Make demo key" for a
throwaway in-browser keypair (devnet only, no real funds). Pick a game ID,
Create game → Fill bots → Start round (REAL VRF) or Pick moles (DEMO) →
Buy/Sell → Run bot round → End round + reveal. Delegate to rollup moves
trades onto the Ephemeral Rollup (gasless); Commit writes state back to base.

### Demo loop (CLI, no browser)

```bash
cd app
node scripts/demo-loop.mjs localnet 4242   # needs a local validator + deploy
node scripts/demo-loop.mjs devnet 4242     # needs the program deployed + SOL
```

## Deploy to devnet (when funded)

```bash
anchor deploy --provider.cluster https://rpc.magicblock.app/devnet
# then: node app/scripts/demo-loop.mjs devnet <gameId>
```

Program deploy rent for the 482,120-byte binary is ~3.36 SOL.

## Live verification (devnet, 2026-11-21)

Program deployed: `73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK`
(ProgramData `EoAoCiZWfn5EKQx1HSaTvhnHUb4a534E1h9VuNcD99eq`, slot 524638473).
Deploy tx: `2HpwJnUjeQ19uSDLrjV3iWVFBE9sqbZc7spHAyJkYmoPPM4n2xNgDMmfVE1E1nAM4eqzjUfW1hDCundJcGfbzGuZ`.
Deployer funded by Eddie via faucet.solana.com (4.02 SOL); ~2.47 SOL spent on
deploy, ~0.12 SOL left after three demo games.

Full game 4245 played live on devnet (game PDA
`6VJbtHcSD3Pnp8bJUm9Rkq1ben5zpPA7c1QVcmYce6Qk`): create, 2 human joins,
bots fill 5 seats, VRF request accepted but callback never landed in 200s,
demo fallback opened trading (moles seats 5 and 7, labelled DEMO), 3 human
trades + 5 bot trades (pot 8000 -> 7974, mole pool 0 -> 26, price
100.00 -> 100.03, tick 8), session-key trade, delegate to devnet-eu ER,
reveal on the ER, undelegate + settle to base (status Revealed on base).
Runner: `node app/scripts/demo-loop.mjs devnet <gameId>` then
`node app/scripts/finish-game.mjs <gameId>`. Note: websockets are broken in
this VM, so the runner polls confirmations over HTTP; and after delegation
the reveal must run on the ER endpoint, not the base RPC.

## Blockers (2026-11-20, updated 2026-11-21)

1. **Devnet SOL for the program deploy — RESOLVED 2026-11-21.** Eddie funded
   the deployer with 4.02 SOL via faucet.solana.com (the site blocks AI
   agents; the CLI airdrop fails on both RPCs). Deploy done, program live.
2. **`solana-test-validator` 3.1.9 cannot start in this VM** — it panics on
   `assertion failed: io_uring_supported()` (the sandbox blocks the
   `io_uring_setup` syscall, errno 1). Localnet demo-loop runs are blocked
   by this; LiteSVM tests are the verified loop instead. (Solana 2.3.13
   installer also failed: its GitHub release asset URL errors.)
3. **No MAGICBLOCK_AUTH_TOKEN was needed.** Delegation uses the on-chain
   Delegation Program + a public validator; no auth token step exists in
   this flow. (The token in old community guides was for managed rollup
   creation, not for delegation.)
4. `devnet-router.magicblock.app` answered `getDelegationStatus` fine on
   2026-11-21 (1.1s). Earlier timeouts from this VM were intermittent.

## Layout

```
programs/mole-market/src/
  lib.rs            program entry points (#[ephemeral] + #[program])
  state.rs          Game + Seat accounts (934 bytes, under the 10 KiB commit cap)
  logic.rs          pure math + unit tests (moles, leak, price, payouts, bots)
  constants.rs      seats, pot, leak rate, validator, status/source codes
  error.rs          MoleError
  instructions/
    lobby.rs        create_game / join_game / fill_bots / create_session
    round.rs        start_round (VRF) / callback_pick_moles / pick_moles_demo
    trade.rs        trade / bot_trade / end_round
    delegation.rs   delegate_game / commit_game / undelegate_game
  tests/integration.rs   LiteSVM full-loop + boundary tests
app/                React + Vite frontend (wallet adapter + demo key)
app/scripts/demo-loop.mjs   CLI full-loop runner
```

## Submission

Submit at https://build.magicblock.app/?stage=blitz#submit before
**Fri 27 Nov 2026, 15:00 WAT** (working date from Luma; not shown on the
site). The v10 checklist (repo/demo/video) was not confirmable by text
fetch, and the submit form was CLOSED on 2026-11-20 ("No open events") —
re-check the form in a live browser before submitting. Program is deployed
and the demo loop has run live on devnet (game 4245). Still missing before
submit: public repo + demo video.
