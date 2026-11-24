// Read-only check mirroring the frontend's exact data path (app/src/mole.ts):
// Anchor program fetch of a game account over BASE_RPC, account owner check,
// and router getDelegationStatus. No transactions, no signing, no SOL spent.
import * as anchor from "@anchor-lang/core";
import { Connection, PublicKey } from "@solana/web3.js";
import { readFileSync } from "node:fs";

const RPC = "https://rpc.magicblock.app/devnet";
const ROUTER = "https://devnet-router.magicblock.app";
const PROGRAM_ID = new PublicKey("73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK");
const DEPLOYER = new PublicKey("7F4Ziy4jVQFWLmaTBmxCBQ7DWtWJFeo2Xk9bSQsPECrG");
const STATUS = ["Lobby", "Awaiting randomness", "Trading", "Revealed"];
const SOURCE = ["none", "REAL VRF", "DEMO (not VRF)"];

const idl = JSON.parse(readFileSync("/home/hatch/workspace/hackathons/magicblock/mole-market/app/src/idl/mole_market.json"));
const connection = new Connection(RPC, "confirmed");
const provider = new anchor.AnchorProvider(connection, { publicKey: PublicKey.default }, { commitment: "confirmed" });
const program = new anchor.Program(idl, provider);

function gamePda(gameId) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("game"), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  )[0];
}

async function delegationStatus(account) {
  const res = await fetch(ROUTER, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getDelegationStatus", params: [account.toBase58()] }),
  });
  return res.json();
}

for (const gid of [4245, 4246]) {
  const pda = gamePda(gid);
  console.log(`\n--- game ${gid} PDA ${pda.toBase58()} ---`);
  try {
    const g = await program.account.game.fetch(pda);
    console.log(`status=${g.status} (${STATUS[g.status]}) randomnessSource=${g.randomnessSource} (${SOURCE[g.randomnessSource]})`);
    console.log(`pot=${g.pot.toString()} molePool=${g.molePool.toString()} price=${g.price.toString()} tick=${g.tick.toString()}`);
    console.log(`moles=${g.moleA},${g.moleB} vrfNonce=${g.vrfNonce.toString()} pendingVrf=${g.pendingVrf}`);
    console.log(`seats: ${g.seats.map((s, i) => `${i}:${s.isBot ? "BOT" : "human"} bal=${s.balance.toString()} sh=${s.shares.toString()} payout=${s.payout.toString()}`).join(" | ")}`);
  } catch (e) {
    console.log(`fetch FAILED: ${e.message.slice(0, 200)}`);
  }
  const info = await connection.getAccountInfo(pda);
  console.log(`account owner: ${info ? info.owner.toBase58() : "null (no account)"} lamports=${info ? info.lamports : "-"}`);
  try {
    const st = await delegationStatus(pda);
    console.log(`router getDelegationStatus: ${JSON.stringify(st.result ?? st.error)}`);
  } catch (e) {
    console.log(`router FAILED: ${e.message.slice(0, 200)}`);
  }
}

const bal = await connection.getBalance(DEPLOYER);
console.log(`\ndeployer balance: ${bal} lamports = ${(bal / 1e9).toFixed(9)} SOL`);
const slot = await connection.getSlot();
console.log(`devnet slot: ${slot}`);
console.log("READ-ONLY CHECK DONE — no tx sent, no SOL spent");
