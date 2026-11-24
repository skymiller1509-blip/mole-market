// Demo loop runner: plays a full Mole Market game end to end from the CLI.
// Usage:
//   node scripts/demo-loop.mjs [localnet|devnet] [gameId]
// - localnet: http://127.0.0.1:8899, payer = ./deployer-keypair.json (airdrop first)
// - devnet:  https://rpc.magicblock.app/devnet, payer = ./deployer-keypair.json
// Prints every step and the final table. Exit 0 only if the loop completes.
//
// NOTE: websockets are broken in this VM (wss TLS handshake fails against
// every endpoint), so confirmations are polled over plain HTTP
// (getSignatureStatuses) instead of Anchor's .rpc(), which waits on a
// websocket subscription and times out after 30s even when the tx landed.
import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const CLUSTER = process.argv[2] || "localnet";
const GAME_ID = Number(process.argv[3] || "4242");
const RPC =
  CLUSTER === "devnet"
    ? "https://rpc.magicblock.app/devnet"
    : "http://127.0.0.1:8899";

const PROGRAM_ID = new PublicKey("73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK");
const VRF_DEFAULT_QUEUE = new PublicKey("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");
const STATUS = ["Lobby", "Awaiting randomness", "Trading", "Revealed"];
const SOURCE = ["none", "REAL VRF", "DEMO (not VRF)"];

const idl = JSON.parse(readFileSync(new URL("../src/idl/mole_market.json", import.meta.url)));

function gamePda(gameId) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("game"), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  )[0];
}

async function main() {
  const connection = new Connection(RPC, "confirmed");
  const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(new URL("../../deployer-keypair.json", import.meta.url))))
  );
  console.log(`Cluster: ${CLUSTER} (${RPC})`);
  console.log(`Payer: ${payer.publicKey.toBase58()}`);
  console.log(`Balance: ${await connection.getBalance(payer.publicKey)} lamports`);
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const program = new anchor.Program(idl, provider);
  const game = gamePda(GAME_ID);

  // Send an Anchor MethodsBuilder tx and poll its status over HTTP
  // (no websocket). Returns the signature. Throws if the tx errors.
  async function sendAndPoll(builder, label) {
    const tx = await builder.transaction();
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    tx.feePayer = payer.publicKey;
    await wallet.signTransaction(tx);
    for (const s of (builder._signers ?? [])) tx.partialSign(s);
    const sig = await connection.sendRawTransaction(tx.serialize());
    const start = Date.now();
    while (Date.now() - start < 90000) {
      const st = await connection.getSignatureStatuses([sig]);
      const v = st.value[0];
      if (v && (v.confirmationStatus === "confirmed" || v.confirmationStatus === "finalized")) {
        if (v.err) throw new Error(`${label}: tx ${sig} failed on chain: ${JSON.stringify(v.err)}`);
        console.log(`[tx ${label}] ${sig.slice(0, 16)}... ${v.confirmationStatus} slot=${v.slot}`);
        return sig;
      }
      await new Promise((r) => setTimeout(r, 2000));
    }
    throw new Error(`${label}: tx ${sig} not confirmed in 90s`);
  }

  const fetch = async () => program.account.game.fetch(game);
  const show = async (label) => {
    const g = await fetch();
    console.log(
      `[${label}] status=${STATUS[g.status]} source=${SOURCE[g.randomnessSource]} pot=${g.pot} molePool=${g.molePool} price=${(g.price.toNumber() / 100).toFixed(2)} tick=${g.tick} moles=${g.moleA},${g.moleB}`
    );
    return g;
  };

  // 1. Create game (creator takes seat 0).
  await sendAndPoll(program.methods.createGame(new anchor.BN(GAME_ID)).accounts({ game, creator: payer.publicKey }), "createGame");
  await show("created");

  // 2. Two more humans join, then bots fill the rest.
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  for (const [who, seat] of [[alice, 1], [bob, 2]]) {
    // Devnet RPC airdrop is closed (HTTP 410), so fund players from the
    // deployer instead of requestAirdrop.
    const fund = SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: who.publicKey, lamports: 100_000_000 });
    await connection.sendTransaction(new Transaction().add(fund), [payer]);
    await new Promise((r) => setTimeout(r, 500));
    await sendAndPoll(
      program.methods
        .joinGame(new anchor.BN(GAME_ID), seat)
        .accounts({ game, player: who.publicKey })
        .signers([who]),
      `joinGame seat ${seat}`
    );
  }
  await sendAndPoll(program.methods.fillBots(new anchor.BN(GAME_ID)).accounts({ game, payer: payer.publicKey }), "fillBots");
  const g0 = await show("table full (3 humans + 5 bots)");

  // 3. Try REAL VRF first. On localnet there is no VRF program, so this is
  // expected to fail there; on devnet it should succeed.
  let vrfWorked = false;
  try {
    await sendAndPoll(
      program.methods
        .startRound(new anchor.BN(GAME_ID))
        .accounts({ payer: payer.publicKey, game, oracleQueue: VRF_DEFAULT_QUEUE }),
      "startRound (VRF request)"
    );
    console.log("[vrf] request accepted, waiting up to 200s for the callback...");
    for (let i = 0; i < 100; i++) {
      await new Promise((r) => setTimeout(r, 2000));
      const g = await fetch();
      if (g.status === 2) { vrfWorked = true; break; }
    }
    console.log(vrfWorked ? "[vrf] callback landed: moles picked by REAL VRF" : "[vrf] callback did not land in 200s");
  } catch (e) {
    console.log(`[vrf] request failed (expected on localnet): ${String(e).slice(0, 160)}`);
  }

  // 4. Demo fallback if VRF did not deliver.
  if (!vrfWorked) {
    await sendAndPoll(
      program.methods
        .pickMolesDemo(new anchor.BN(GAME_ID), new anchor.BN(424242))
        .accounts({ game, creator: payer.publicKey }),
      "pickMolesDemo"
    );
    console.log("[demo] moles picked with DEMO randomness (labelled on chain)");
  }
  const g1 = await show("trading open");

  // 5. Human trades: creator buys 500, alice sells 200.
  await sendAndPoll(
    program.methods
      .trade(new anchor.BN(GAME_ID), 0, true, new anchor.BN(500))
      .accounts({ game, signer: payer.publicKey }),
    "trade creator buy 500"
  );
  await sendAndPoll(
    program.methods
      .trade(new anchor.BN(GAME_ID), 1, false, new anchor.BN(200))
      .accounts({ game, signer: alice.publicKey })
      .signers([alice]),
    "trade alice sell 200"
  );
  await show("after 2 human trades");

  // 6. Session key: creator authorizes a throwaway key and trades with it.
  const session = Keypair.generate();
  const slot = await connection.getSlot();
  await sendAndPoll(
    program.methods
      .createSession(new anchor.BN(GAME_ID), 0, session.publicKey, new anchor.BN(slot + 5000))
      .accounts({ game, owner: payer.publicKey }),
    "createSession"
  );
  await sendAndPoll(
    program.methods
      .trade(new anchor.BN(GAME_ID), 0, true, new anchor.BN(100))
      .accounts({ game, signer: session.publicKey })
      .signers([session]),
    "trade via session key"
  );
  console.log("[session] trade signed by session key ok");
  await show("after session-key trade");

  // 7. Bot round: every bot seat takes its deterministic turn.
  for (let seat = 0; seat < 8; seat++) {
    if (!g1.seats[seat].isBot) continue;
    await sendAndPoll(
      program.methods
        .botTrade(new anchor.BN(GAME_ID), seat)
        .accounts({ game, payer: payer.publicKey }),
      `botTrade seat ${seat}`
    );
  }
  await show("after bot round");

  // 8. Try delegation (works on devnet, fails on localnet: no program).
  try {
    await sendAndPoll(
      program.methods
        .delegateGame(new anchor.BN(GAME_ID))
        .accounts({ payer: payer.publicKey, game }),
      "delegateGame"
    );
    console.log("[delegate] game delegated to the Ephemeral Rollup");
  } catch (e) {
    console.log(`[delegate] failed (expected on localnet): ${String(e).slice(0, 160)}`);
  }

  // 9. Reveal.
  await sendAndPoll(program.methods.endRound(new anchor.BN(GAME_ID)).accounts({ game, payer: payer.publicKey }), "endRound");
  const g2 = await show("REVEALED");
  console.log("\nFinal table:");
  g2.seats.forEach((s, i) => {
    const isMole = i === g2.moleA || i === g2.moleB;
    console.log(
      `  seat ${i} ${s.isBot ? "BOT  " : "human"} ${isMole ? "MOLE  " : "honest"} balance=${s.balance} shares=${s.shares} payout=${s.payout}`
    );
  });
  console.log(`\nMoles were seats ${g2.moleA} and ${g2.moleB}. Randomness source: ${SOURCE[g2.randomnessSource]}.`);
  console.log(`Pot survived: ${g2.pot} pts. Mole pool paid out: ${g2.molePool} pts.`);
  console.log("DEMO LOOP OK");
}

main().catch((e) => {
  console.error("DEMO LOOP FAILED:", e);
  process.exit(1);
});
