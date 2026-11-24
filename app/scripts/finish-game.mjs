// Finish game 4245 on the Ephemeral Rollup: the game was delegated mid-game,
// so endRound must run on the ER endpoint, then undelegate to settle back.
// Usage: node scripts/finish-game.mjs [gameId]
// Reads the game state on the ER, ends the round there, undelegates, then
// verifies the final state on the base layer.
import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const GAME_ID = Number(process.argv[2] || "4245");
const ER_RPC = "https://devnet-eu.magicblock.app/";
const BASE_RPC = "https://rpc.magicblock.app/devnet";
const PROGRAM_ID = new PublicKey("73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK");
const MAGIC_PROGRAM = new PublicKey("Magic11111111111111111111111111111111111111");
const MAGIC_CONTEXT = new PublicKey("MagicContext1111111111111111111111111111111");
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
  const er = new Connection(ER_RPC, "confirmed");
  const base = new Connection(BASE_RPC, "confirmed");
  const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(readFileSync(new URL("../../deployer-keypair.json", import.meta.url))))
  );
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(er, wallet, { commitment: "confirmed" });
  const program = new anchor.Program(idl, provider);
  const game = gamePda(GAME_ID);

  async function sendAndPoll(builder, label) {
    const tx = await builder.transaction();
    tx.recentBlockhash = (await er.getLatestBlockhash()).blockhash;
    tx.feePayer = payer.publicKey;
    await wallet.signTransaction(tx);
    for (const s of (builder._signers ?? [])) tx.partialSign(s);
    const sig = await er.sendRawTransaction(tx.serialize());
    const start = Date.now();
    while (Date.now() - start < 90000) {
      const st = await er.getSignatureStatuses([sig]);
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

  const show = async (conn, label) => {
    const p = new anchor.Program(idl, new anchor.AnchorProvider(conn, wallet, { commitment: "confirmed" }));
    const g = await p.account.game.fetch(game);
    console.log(
      `[${label}] status=${STATUS[g.status]} source=${SOURCE[g.randomnessSource]} pot=${g.pot} molePool=${g.molePool} price=${(g.price.toNumber() / 100).toFixed(2)} tick=${g.tick} moles=${g.moleA},${g.moleB}`
    );
    return g;
  };

  // 1. State on the ER before the reveal.
  const g0 = await show(er, "ER before reveal");

  // 2. End the round on the ER.
  await sendAndPoll(program.methods.endRound(new anchor.BN(GAME_ID)).accounts({ game, payer: payer.publicKey }), "endRound on ER");
  const g1 = await show(er, "ER REVEALED");
  console.log("\nFinal table (from ER):");
  g1.seats.forEach((s, i) => {
    const isMole = i === g1.moleA || i === g1.moleB;
    console.log(
      `  seat ${i} ${s.isBot ? "BOT  " : "human"} ${isMole ? "MOLE  " : "honest"} balance=${s.balance} shares=${s.shares} payout=${s.payout}`
    );
  });
  console.log(`\nMoles were seats ${g1.moleA} and ${g1.moleB}. Randomness source: ${SOURCE[g1.randomnessSource]}.`);
  console.log(`Pot survived: ${g1.pot} pts. Mole pool paid out: ${g1.molePool} pts.`);

  // 3. Commit + undelegate back to the base layer.
  await sendAndPoll(
    program.methods.undelegateGame().accounts({ payer: payer.publicKey, game, magicProgram: MAGIC_PROGRAM, magicContext: MAGIC_CONTEXT }),
    "undelegateGame on ER"
  );
  console.log("[undelegate] game committed and returned to the base layer");

  // 4. Verify the final state on the base layer.
  const baseInfo = await base.getAccountInfo(game);
  console.log(`Base owner after undelegate: ${baseInfo.owner.toBase58()}`);
  const g2 = await show(base, "BASE final");
  if (g2.status !== 3) throw new Error("base state is not Revealed after undelegate");
  console.log("FINISH OK");
}

main().catch((e) => {
  console.error("FINISH FAILED:", e);
  process.exit(1);
});
