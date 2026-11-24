# Mole Market — submission checklist

Event: MagicBlock Solana Blitz v10, Entertainment Finance
Deadline: **Fri 27 Nov 2026, 15:00 WAT** (working date from Luma; not shown on the site).
Platform: submit form at https://build.magicblock.app/?stage=blitz#submit
Submit at least 30 minutes before the deadline; do not touch the repo, live app or video after submitting until winners are announced.

## Requirements

- [x] DONE — Program deployed on devnet: `73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK` (deploy tx `2HpwJnUjeQ19uSDLrjV3iWVFBE9sqbZc7spHAyJkYmoPPM4n2xNgDMmfVE1E1nAM4eqzjUfW1hDCundJcGfbzGuZ`).
- [x] DONE — Full game 4245 played live on devnet (game PDA `6VJbtHcSD3Pnp8bJUm9Rkq1ben5zpPA7c1QVcmYce6Qk`): create, joins, bots, demo mole pick (moles 5 and 7), trades, delegation to devnet-eu ER, reveal on the ER, undelegate + settle to base. Pot 8000 → 7974, mole pool 0 → 26.
- [x] DONE — Tests: 9 unit + 3 LiteSVM integration tests, 12/12 pass.
- [x] DONE — Frontend builds and serves; verified 2026-11-23 reading game 4245 live (status Revealed, pot 7974, pool 26, moles 5/7, source DEMO).
- [ ] TODO (Eddie's step) — Create the public GitHub repo (e.g. `mole-market`) and publish the code (Mabel cannot create repos).
- [ ] TODO (Eddie's step) — Film the demo video. Script: `VIDEO-SCRIPT.md`. Use the DEMO mole pick only — NEVER the REAL VRF button (callback never landed in 3 tries).
- [ ] TODO (Eddie's step) — Re-check the submit form in a live browser: it was CLOSED on 2026-11-20 ("No open events") and again on 2026-11-23. Do not assume it is open — check it before the deadline.
- [ ] TODO (Eddie's step) — Submit the form with: public repo link, demo video link, program ID above, and the game 4245 live-game evidence.
- [ ] TODO (Mabel's step) — After the repo exists, check every submitted link opens (repo, video, program on explorer).

## Prize / payout

- Prize split / payout route / KYC: not verified from the form in this task (the form was closed when checked) — read the form when it opens and note what it asks for before submitting.

## Honest limits (state these in the submission, do not hide them)

- VRF mole pick: requested live three times (games 4244, 4245, 4246), callback NEVER landed. The demo uses the labelled DEMO pick. Do not claim live VRF works.
- Role hiding is UI-level only in this build. Anyone who fetches the game account on chain can read `mole_a` / `mole_b` before reveal. True secrecy needs a Private Ephemeral Rollup.
- Points are virtual. No real money, devnet only.
- Deployer balance after three demo games: ~0.11 SOL (VRF retry 2026-11-23 cost ~0.01 SOL). Do not start a new funded run without checking the balance first.

## Links

- Submit form: https://build.magicblock.app/?stage=blitz#submit
- Luma (deadline source): https://luma.com/hedv5sgh
- Program: `73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK` (Solana devnet)
- Demo video: TODO — not filmed yet.
- Public repo: TODO — not created yet.
