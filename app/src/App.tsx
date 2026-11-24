import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import {
  BASE_RPC,
  ER_RPC_EU,
  SOURCE,
  STATUS,
  VRF_DEFAULT_QUEUE,
  botPda,
  fetchGame,
  fmtPrice,
  gamePda,
  getDelegationStatus,
  getProgram,
  isDelegatedOnBase,
  shortKey,
  type GameState,
} from "./mole";

const MAGIC_PROGRAM_ID = new PublicKey("Magic11111111111111111111111111111111111111");
const MAGIC_CONTEXT_ID = new PublicKey("MagicContext1111111111111111111111111111111");

type Cluster = "devnet" | "localnet";

function clusterRpc(cluster: Cluster): string {
  return cluster === "devnet" ? BASE_RPC : "http://127.0.0.1:8899";
}

export default function App() {
  const walletAdapter = useWallet();
  const [cluster, setCluster] = useState<Cluster>("devnet");
  const [demoKey, setDemoKey] = useState<Keypair | null>(null);
  const [gameId, setGameId] = useState<number>(4245); // open on the verified live game
  const [game, setGame] = useState<GameState | null>(null);
  const [log, setLog] = useState<string[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [delegated, setDelegated] = useState<boolean | null>(null);
  const [amount, setAmount] = useState<number>(100);
  const [mySeat, setMySeat] = useState<number>(0);
  const sessionKeyRef = useRef<Keypair | null>(null);
  const [sessionOn, setSessionOn] = useState(false);

  const connection = useMemo(() => new Connection(clusterRpc(cluster), "confirmed"), [cluster]);
  const erConnection = useMemo(() => new Connection(ER_RPC_EU, "confirmed"), []);

  // The signing wallet: adapter wallet if connected, else the demo keypair.
  const signer: { publicKey: PublicKey; signTransaction: (tx: anchor.web3.Transaction) => Promise<anchor.web3.Transaction>; signAllTransactions: (txs: anchor.web3.Transaction[]) => Promise<anchor.web3.Transaction[]> } | null =
    useMemo(() => {
      if (walletAdapter.publicKey && walletAdapter.signTransaction) {
        return {
          publicKey: walletAdapter.publicKey,
          signTransaction: walletAdapter.signTransaction,
          signAllTransactions: walletAdapter.signAllTransactions!,
        };
      }
      if (demoKey) {
        const kp = demoKey;
        return {
          publicKey: kp.publicKey,
          signTransaction: async (tx) => {
            tx.partialSign(kp);
            return tx;
          },
          signAllTransactions: async (txs) => {
            txs.forEach((tx) => tx.partialSign(kp));
            return txs;
          },
        };
      }
      return null;
    }, [walletAdapter.publicKey, walletAdapter.signTransaction, demoKey]);

  const program = useMemo(() => {
    if (!signer) return null;
    return getProgram(connection, signer as unknown as anchor.Wallet);
  }, [connection, signer]);

  const erProgram = useMemo(() => {
    if (!signer) return null;
    return getProgram(erConnection, signer as unknown as anchor.Wallet);
  }, [erConnection, signer]);

  // Read-only program so the live game loads on first open, before any wallet
  // is connected. It can only fetch state; every transaction still needs a signer.
  const readOnlyProgram = useMemo(() => {
    return getProgram(connection, { publicKey: PublicKey.default } as unknown as anchor.Wallet);
  }, [connection]);

  const [gameAddress] = useMemo(() => gamePda(gameId), [gameId]);

  const pushLog = useCallback((line: string) => {
    setLog((prev) => [`${new Date().toLocaleTimeString()} ${line}`, ...prev].slice(0, 30));
  }, []);

  const refresh = useCallback(async () => {
    const p = program ?? readOnlyProgram;
    const g = await fetchGame(p, gameAddress);
    setGame(g);
    if (g) {
      try {
        setDelegated(await isDelegatedOnBase(connection, gameAddress));
      } catch {
        setDelegated(null);
      }
    }
  }, [program, readOnlyProgram, gameAddress, connection]);

  useEffect(() => {
    refresh();
    const t = setInterval(refresh, 3000);
    return () => clearInterval(t);
  }, [refresh]);

  async function run(label: string, fn: () => Promise<string>) {
    if (!program || !signer) {
      pushLog("Connect a wallet (or make a demo key) first.");
      return;
    }
    setBusy(label);
    try {
      const sig = await fn();
      pushLog(`${label}: ok ${sig.slice(0, 12)}…`);
      await refresh();
    } catch (e) {
      pushLog(`${label}: FAILED — ${(e as Error).message.slice(0, 200)}`);
    } finally {
      setBusy(null);
    }
  }

  // Pick the right program: ER when the game is delegated, base otherwise.
  const liveProgram = delegated ? erProgram ?? program : program;

  const doCreate = () =>
    run("Create game", () =>
      program!.methods
        .createGame(new anchor.BN(gameId))
        .accounts({ game: gameAddress, creator: signer!.publicKey })
        .rpc()
    );

  const doJoin = () =>
    run(`Join seat ${mySeat}`, () =>
      program!.methods
        .joinGame(new anchor.BN(gameId), mySeat)
        .accounts({ game: gameAddress, player: signer!.publicKey })
        .rpc()
    );

  const doFillBots = () =>
    run("Fill bots", () =>
      program!.methods
        .fillBots(new anchor.BN(gameId))
        .accounts({ game: gameAddress, payer: signer!.publicKey })
        .rpc()
    );

  const doStartRound = () =>
    run("Start round (REAL VRF)", () =>
      program!.methods
        .startRound(new anchor.BN(gameId))
        .accounts({
          payer: signer!.publicKey,
          game: gameAddress,
          oracleQueue: VRF_DEFAULT_QUEUE,
        })
        .rpc()
    );

  const doDemoMoles = () =>
    run("Pick moles (DEMO randomness)", () =>
      program!.methods
        .pickMolesDemo(new anchor.BN(gameId), new anchor.BN(Math.floor(Math.random() * 1e9)))
        .accounts({ game: gameAddress, creator: signer!.publicKey })
        .rpc()
    );

  const doTrade = (isBuy: boolean) =>
    run(`${isBuy ? "Buy" : "Sell"} ${amount}`, () =>
      liveProgram!.methods
        .trade(new anchor.BN(gameId), mySeat, isBuy, new anchor.BN(amount))
        .accounts({ game: gameAddress, signer: sessionOn && sessionKeyRef.current ? sessionKeyRef.current.publicKey : signer!.publicKey })
        .signers(sessionOn && sessionKeyRef.current ? [sessionKeyRef.current] : [])
        .rpc()
    );

  const doBotRound = async () => {
    if (!game) return;
    for (let seat = 0; seat < 8; seat++) {
      if (!game.seats[seat].isBot) continue;
      await run(`Bot seat ${seat}`, () =>
        liveProgram!.methods
          .botTrade(new anchor.BN(gameId), seat)
          .accounts({ game: gameAddress, payer: signer!.publicKey })
          .rpc()
      );
    }
  };

  const doEndRound = () =>
    run("End round + reveal", () =>
      program!.methods
        .endRound(new anchor.BN(gameId))
        .accounts({ game: gameAddress, payer: signer!.publicKey })
        .rpc()
    );

  const doDelegate = () =>
    run("Delegate to rollup", async () => {
      const sig = await program!.methods
        .delegateGame(new anchor.BN(gameId))
        .accounts({ payer: signer!.publicKey, game: gameAddress })
        .rpc();
      // Wait until the router sees it delegated.
      for (let i = 0; i < 20; i++) {
        await new Promise((r) => setTimeout(r, 1500));
        const st = await getDelegationStatus(gameAddress);
        if (st.isDelegated) break;
      }
      return sig;
    });

  const doCommit = () =>
    run("Commit to base layer", () =>
      erProgram!.methods
        .commitGame()
        .accounts({
          payer: signer!.publicKey,
          game: gameAddress,
          magicProgram: MAGIC_PROGRAM_ID,
          magicContext: MAGIC_CONTEXT_ID,
        })
        .rpc()
    );

  const doUndelegate = () =>
    run("Commit + undelegate", () =>
      erProgram!.methods
        .undelegateGame()
        .accounts({
          payer: signer!.publicKey,
          game: gameAddress,
          magicProgram: MAGIC_PROGRAM_ID,
          magicContext: MAGIC_CONTEXT_ID,
        })
        .rpc()
    );

  const doSessionKey = async () => {
    if (!program || !signer) return;
    const kp = Keypair.generate();
    sessionKeyRef.current = kp;
    const slot = await connection.getSlot();
    await run("Create session key", () =>
      program.methods
        .createSession(new anchor.BN(gameId), mySeat, kp.publicKey, new anchor.BN(slot + 5000))
        .accounts({ game: gameAddress, owner: signer.publicKey })
        .rpc()
    );
    setSessionOn(true);
    pushLog(`Session key ${shortKey(kp.publicKey)} active for ~5000 slots.`);
  };

  const leakPct = game ? Math.round(((8000 - game.pot.toNumber()) / 8000) * 100) : 0;
  const isRevealed = game?.status === 3;

  return (
    <div style={{ fontFamily: "system-ui, sans-serif", maxWidth: 860, margin: "0 auto", padding: 16 }}>
      <h1>Mole Market</h1>
      <p>Among Us meets day trading. Eight players, one shared pot, two secret moles skimming it.</p>

      <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <label>
          Network{" "}
          <select value={cluster} onChange={(e) => setCluster(e.target.value as Cluster)}>
            <option value="devnet">Devnet</option>
            <option value="localnet">Localnet</option>
          </select>
        </label>
        <WalletMultiButton />
        <button
          onClick={() => {
            const kp = Keypair.generate();
            setDemoKey(kp);
            pushLog(`Demo key made: ${kp.publicKey.toBase58()} (devnet only, no real funds)`);
          }}
        >
          Make demo key
        </button>
        <span>
          {signer ? `Signed in as ${shortKey(signer.publicKey)}` : "Not signed in"}
          {demoKey ? " (demo key)" : ""}
        </span>
      </div>

      <hr />
      <div style={{ display: "flex", gap: 8, alignItems: "center", flexWrap: "wrap" }}>
        <label>
          Game ID{" "}
          <input type="number" value={gameId} onChange={(e) => setGameId(Number(e.target.value))} style={{ width: 100 }} />
        </label>
        <span>Game PDA: {shortKey(gameAddress)}</span>
        <button onClick={doCreate} disabled={!!busy || !signer}>Create game</button>
        <button onClick={refresh}>Refresh</button>
      </div>

      {game ? (
        <>
          <h2>
            Game {game.gameId.toString()} — {STATUS[game.status]}
            {game.randomnessSource > 0 && (
              <span style={{ marginLeft: 8, fontSize: 14 }}>Randomness: {SOURCE[game.randomnessSource]}</span>
            )}
          </h2>
          <div style={{ display: "flex", gap: 24, flexWrap: "wrap" }}>
            <div>
              <strong>Shared pot: {game.pot.toString()} pts</strong> (started 8,000 — leaked {leakPct}%)
              <div style={{ width: 220, height: 12, background: "#ddd" }}>
                <div style={{ width: `${(game.pot.toNumber() / 8000) * 100}%`, height: "100%", background: "#c0392b" }} />
              </div>
              <div>Skimmed into the mole pool: {game.molePool.toString()} pts (who got it? nobody knows — yet)</div>
              <div>Index price: ${fmtPrice(game.price)}</div>
              <div>Trades so far: {game.tick.toString()}</div>
              <div>Delegated to rollup: {delegated === null ? "?" : delegated ? "YES — trades are gasless" : "no — trades settle on base"}</div>
            </div>
          </div>

          <h3>Seats</h3>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 8 }}>
            {game.seats.map((seat, i) => {
              const isMole = isRevealed && (i === game.moleA || i === game.moleB);
              const isMe = signer && seat.owner.equals(signer.publicKey);
              return (
                <div key={i} style={{ border: "1px solid #999", padding: 8, background: isMole ? "#f9e79f" : isMe ? "#d5f5e3" : "#fff" }}>
                  <div>
                    <strong>Seat {i}</strong> {seat.isBot ? "BOT" : "human"} {isMe ? "(YOU)" : ""}
                  </div>
                  <div>{seat.owner.equals(PublicKey.default) ? "empty" : shortKey(seat.owner)}</div>
                  <div>Balance: {seat.balance.toString()} pts</div>
                  <div>Shares: {seat.shares.toString()}</div>
                  {isRevealed ? (
                    <div>{isMole ? "MOLE" : "honest"} — payout {seat.payout.toString()} pts</div>
                  ) : (
                    <div>Role: ? (hidden until reveal)</div>
                  )}
                  {i === mySeat && <div>← your seat</div>}
                </div>
              );
            })}
          </div>

          <h3>Play</h3>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center" }}>
            <label>
              My seat{" "}
              <select value={mySeat} onChange={(e) => setMySeat(Number(e.target.value))}>
                {Array.from({ length: 8 }, (_, i) => (
                  <option key={i} value={i}>{i}</option>
                ))}
              </select>
            </label>
            <button onClick={doJoin} disabled={!!busy}>Join seat</button>
            <button onClick={doFillBots} disabled={!!busy}>Fill empty seats with bots</button>
            <button onClick={doStartRound} disabled={!!busy}>Start round — REAL VRF</button>
            <button onClick={doDemoMoles} disabled={!!busy}>Pick moles — DEMO randomness</button>
          </div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", marginTop: 8 }}>
            <label>
              Amount{" "}
              <select value={amount} onChange={(e) => setAmount(Number(e.target.value))}>
                <option value={50}>50</option>
                <option value={100}>100</option>
                <option value={250}>250</option>
                <option value={500}>500</option>
              </select>
            </label>
            <button onClick={() => doTrade(true)} disabled={!!busy}>Buy</button>
            <button onClick={() => doTrade(false)} disabled={!!busy}>Sell / short</button>
            <button onClick={doBotRound} disabled={!!busy}>Run bot round</button>
            <button onClick={doEndRound} disabled={!!busy}>End round + reveal moles</button>
          </div>
          <div style={{ display: "flex", gap: 8, flexWrap: "wrap", alignItems: "center", marginTop: 8 }}>
            <button onClick={doSessionKey} disabled={!!busy || sessionOn}>
              {sessionOn ? "Session key active" : "Enable gasless session key"}
            </button>
            <button onClick={doDelegate} disabled={!!busy}>Delegate to rollup</button>
            <button onClick={doCommit} disabled={!!busy}>Commit to base</button>
            <button onClick={doUndelegate} disabled={!!busy}>Commit + undelegate</button>
          </div>
          <p style={{ fontSize: 13, color: "#555" }}>
            Bot seats: {game.seats.map((s, i) => (s.isBot ? botPda(gameId, i).toBase58().slice(0, 4) : null)).filter(Boolean).join(", ") || "none yet"}
          </p>
        </>
      ) : (
        <p>No game loaded at this ID yet. Create one, or pick an existing game ID.</p>
      )}

      <h3>Log</h3>
      <pre style={{ background: "#111", color: "#0f0", padding: 8, maxHeight: 220, overflow: "auto", fontSize: 12 }}>
        {busy ? `Working: ${busy}…\n` : ""}
        {log.join("\n") || "—"}
      </pre>

      <hr />
      <p style={{ fontSize: 13, color: "#555" }}>
        Honest labels: points are virtual, not money. Mole picks use real MagicBlock VRF when you press
        “REAL VRF”; the demo button uses slot-hash randomness and the game records it on chain as DEMO.
        Mole identities are hidden by this UI until reveal. On chain they sit in the delegated game
        account, so true secrecy needs a Private Ephemeral Rollup (TEE) — see README. Trades run on the
        Ephemeral Rollup only after you delegate; before that they settle on devnet base layer.
      </p>
    </div>
  );
}
