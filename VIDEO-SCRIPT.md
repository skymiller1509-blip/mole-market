# Mole Market — demo video script (up to 3 minutes)

Film the web app in a browser on the phone (screen recording) or on a
borrowed desktop. Read the lines aloud in plain words, at a steady
pace. The app opens on game 4245, the full game already played live —
but film a FRESH game for the demo, so the judge watches the whole flow
happen, not a finished screen.

One sentence a judge should repeat afterwards: **"Eight traders, one
pot, two secret moles — and you can watch the pot leak."**

## ⚠️ DO NOT PRESS THIS BUTTON ON CAMERA

**Never press "Start round (REAL VRF)" in the demo.** The VRF oracle
accepted the request three times (games 4244, 4245, 4246) and the
callback never landed — game 4246 waited 10 minutes and still said
"Awaiting randomness". Pressing it stalls the demo.

**Use the "Pick moles (DEMO)" button instead.** It is labelled DEMO on
chain and on screen. Say so aloud when you press it.

## Hook options — pick one, say it aloud, keep the clearest

1. "Watch this pot. Every trade leaks money — and nobody knows who takes it." (pot meter draining on screen)
2. "Two of these eight traders are stealing from the rest."
3. "Everyone sees the leak. Nobody sees the leaker."

## 0:00–0:05 — Hook (sound off: it must work with no audio)

On screen before you say anything: the game table, pot at 8,000, and a
trade going through while the pot meter visibly drops.

On-screen text (large, bottom third): **THE POT IS LEAKING. WHO IS TAKING IT?**

Say: "Eight traders share one pot. Two of them are secretly skimming
it. This is Mole Market."

## 0:05–0:20 — Problem, one sentence, one named user

Say: "Ada plays trading games with friends, but every game either
shows everyone's cards or trusts one person to keep the secret. She
wants a game where the secret is kept by the code, not by a person."

On-screen text: **ADA WANTS A GAME WHERE THE CODE KEEPS THE SECRET.**

## 0:20–2:00 — The one live flow (about 90 seconds)

Narrate what is happening, not the labels on screen. Keep the pot
meter visible the whole time.

1. Say: "I create a game. I take seat zero. The pot starts at 8,000
   points." Create the game on screen.
2. Say: "I fill the empty seats with bots, so the table is full."
   Tap "Fill bots". Eight seats fill.
3. Say: "Now the moles are picked. I use the DEMO pick here — the live
   randomness oracle has not answered in three tries, so the demo uses
   a labelled DEMO pick and says so on screen." Tap "Pick moles
   (DEMO)". Trading opens. Do NOT name any mole — the UI does not show
   them, and neither do you.
4. Say: "I buy. Watch the pot." Buy once. Point at the pot meter:
   "Every trade leaks two percent of its size out of the pot into a
   hidden pool. The pot drains, but nobody is told who collects it."
5. Say: "The bots play their turns." Tap "Run bot round". The pot
   drains a little more.
6. Say: "Now I end the round, and the secret comes out." Tap "End
   round + reveal". The two moles are published on screen, the skimmed
   pool is split between them, and the rest of the pot is split among
   the six honest players. Say: "There they are. The moles get the
   skim. Everyone else splits what is left."

End the flow on the reveal screen. That is the loud moment — the
reveal is what a judge remembers.

## 2:00–2:30 — How it works (30 seconds)

Say: "Underneath, this is a real program on Solana devnet — program
73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK. Trades run on a MagicBlock
Ephemeral Rollup, so they are gasless, and the game settles back to
base at the end. A full game, game 4245, already ran live on devnet:
the pot went from 8,000 to 7,974 and the mole pool from zero to 26."

On-screen text: **SOLANA DEVNET · MAGICBLOCK EPHEMERAL ROLLUP · GAME 4245 PLAYED LIVE**

## 2:30–3:00 — What's next (30 seconds)

Say: "Next is the real fix for the secret: a Private Ephemeral Rollup,
so even reading the game account on chain cannot reveal the moles
before the reveal. And the live randomness pick, once the oracle
answers. Mole Market is built for the MagicBlock Solana Blitz,
Entertainment Finance."

On-screen text: **MOLE MARKET — MAGICBLOCK SOLANA BLITZ V10**

End on the reveal screen. Stop recording.

## Honest limits — say them if asked, do not hide them

- The VRF mole pick has never worked live: three requests accepted,
  zero callbacks. The demo uses the labelled DEMO pick.
- Role hiding is UI-level only in this build. Anyone who fetches the
  game account on chain can read the mole seats before reveal. True
  secrecy needs a Private Ephemeral Rollup — that is the next step,
  not a shipped feature.
- Points are virtual. No real money, devnet only. Do not run this
  with real money.

## Recording notes (phone)

- Turn on Do Not Disturb on the phone first, so no notification lands
  mid-take.
- Film in landscape. Browser zoom 100–125%, full screen, bookmarks bar
  hidden, extra tabs closed.
- Use the phone's built-in Android screen recorder: swipe down twice,
  tap Screen record, microphone on, wait for the countdown.
- Before filming, open the app and check the demo game path works on
  that phone and network. Keep a fallback recording ready.
- Record one long take of about 3 minutes, then trim two edits in
  CapCut (free): a judge cut at 60–75 seconds and a social cut at
  25–35 seconds, from the same footage.
- Add captions — the video may be watched without sound.
- Record a separate 60-second insurance capture of the flow working,
  so a dead network on submission day still leaves something to submit.
- Never show a real private key on screen. Use "Make demo key" (a
  throwaway in-browser keypair, devnet only) if you connect a wallet.
- Rehearse out loud at least five times before the take you keep.
