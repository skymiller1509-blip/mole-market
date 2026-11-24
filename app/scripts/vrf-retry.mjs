// VRF retry test: create a fresh game, request REAL MagicBlock VRF, and poll
// for the callback for 10 minutes. NO demo fallback (pick_moles_demo is never
// called). Usage: NODE_USE_ENV_PROXY=1 node scripts/vrf-retry.mjs [gameId]
// Exits 0 only if a real VRF callback lands (status Trading, source REAL VRF).
import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const GAME_ID = Number(process.argv[2] || "4246");
const RPC = "https://rpc.magicblock.app/devnet";
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
  const balBefore = await connection.getBalance(payer.publicKey);
  console.log(`Payer: ${payer.publicKey.toBase58()}`);
  console.log(`Balance before: ${balBefore} lamports (${(balBefore / 1e9).toFixed(9)} SOL)`);
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, { commitment: "confirmed" });
  const program = new anchor.Program(idl, provider);
  const game = gamePda(GAME_ID);
  console.log(`Game ${GAME_ID} PDA: ${game.toBase58()}`);

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
        console.log(`[tx ${label}] ${sig} ${v.confirmationStatus} slot=${v.slot}`);
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
      `[${label}] status=${STATUS[g.status]} source=${SOURCE[g.randomnessSource]} pendingVrf=${g.pendingVrf} vrfNonce=${g.vrfNonce} moles=${g.moleA},${g.moleB}`
    );
    return g;
  };

  // 1. Create game.
  await sendAndPoll(program.methods.createGame(new anchor.BN(GAME_ID)).accounts({ game, creator: payer.publicKey }), "createGame");

  // 2. Two throwaway humans join (funded just enough for tx fees), then bots fill the rest.
  const alice = Keypair.generate();
  const bob = Keypair.generate();
  for (const [who, seat] of [[alice, 1], [bob, 2]]) {
    const fund = SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: who.publicKey, lamports: 2_000_000 });
    const fundSig = await connection.sendTransaction(new Transaction().add(fund), [payer]);
    console.log(`[fund seat ${seat}] ${fundSig.slice(0, 16)}...`);
    await new Promise((r) => setTimeout(r, 1000));
    await sendAndPoll(
      program.methods.joinGame(new anchor.BN(GAME_ID), seat).accounts({ game, player: who.publicKey }).signers([who]),
      `joinGame seat ${seat}`
    );
  }
  await sendAndPoll(program.methods.fillBots(new anchor.BN(GAME_ID)).accounts({ game, payer: payer.publicKey }), "fillBots");
  await show("table full");

  // 3. REAL VRF request.
  const vrfReqSig = await sendAndPoll(
    program.methods.startRound(new anchor.BN(GAME_ID)).accounts({ payer: payer.publicKey, game, oracleQueue: VRF_DEFAULT_QUEUE }),
    "startRound (VRF request)"
  );
  console.log(`[vrf] request tx: ${vrfReqSig}`);
  console.log(`[vrf] request accepted. Polling for the callback for 600s...`);

  // 4. Poll for 10 minutes.
  const pollStart = Date.now();
  let landed = false;
  let lastStatus = null;
  while (Date.now() - pollStart < 600_000) {
    await new Promise((r) => setTimeout(r, 10_000));
    const g = await fetch();
    const elapsed = Math.round((Date.now() - pollStart) / 1000);
    const summary = `status=${STATUS[g.status]} source=${SOURCE[g.randomnessSource]} pendingVrf=${g.pendingVrf}`;
    if (summary !== lastStatus) {
      console.log(`[poll t+${elapsed}s] ${summary}`);
      lastStatus = summary;
    }
    if (g.status === 2 && g.randomnessSource === 1) { landed = true; break; }
  }

  const g = await show(landed ? "VRF CALLBACK LANDED" : "after 600s poll");
  const balAfter = await connection.getBalance(payer.publicKey);
  console.log(`Balance after: ${balAfter} lamports (${(balAfter / 1e9).toFixed(9)} SOL)`);
  console.log(`Spent: ${((balBefore - balAfter) / 1e9).toFixed(9)} SOL`);

  if (landed) {
    console.log(`\nVRF CALLBACK LANDED: game ${GAME_ID}, moles seats ${g.moleA} and ${g.moleB}, source REAL VRF`);
    // Find the callback tx in the game PDA history.
    const sigs = await connection.getSignaturesForAddress(game, { limit: 20 });
    console.log(`Recent txs on game PDA:`);
    for (const s of sigs) console.log(`  ${s.signature} slot=${s.slot} ${s.err ? "ERR " + JSON.stringify(s.err) : "ok"}`);
    console.log(`VRF request tx: ${vrfReqSig}`);
    console.log("VRF RETRY OK");
  } else {
    console.log(`\nVRF CALLBACK DID NOT LAND in 600s for game ${GAME_ID}. status=${STATUS[g.status]} source=${SOURCE[g.randomnessSource]} pendingVrf=${g.pendingVrf}`);
    console.log(`VRF request tx: ${vrfReqSig}`);
    console.log("VRF RETRY FAILED (no callback)");
    process.exit(2);
  }
}

main().catch((e) => {
  console.error("VRF RETRY ERROR:", e);
  process.exit(1);
});
