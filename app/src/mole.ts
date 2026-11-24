import * as anchor from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  DELEGATION_PROGRAM_ID,
  GetCommitmentSignature,
} from "@magicblock-labs/ephemeral-rollups-sdk";
import idl from "./idl/mole_market.json";

export const PROGRAM_ID = new PublicKey("73qWAngM3BnmDeGghKwXEqk4VcmMcWZmCZwRT7SCZAuK");

// MagicBlock VRF base-layer queue on devnet (verified in the VRF docs).
export const VRF_DEFAULT_QUEUE = new PublicKey(
  "Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh"
);
// Verified from ephemeral-vrf-sdk 0.18.0 src/consts.rs (installed crate).
export const VRF_PROGRAM_ID = new PublicKey(
  "Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz"
);

export const BASE_RPC = "https://rpc.magicblock.app/devnet";
export const ER_RPC_EU = "https://devnet-eu.magicblock.app";
export const ROUTER_RPC = "https://devnet-router.magicblock.app";

export const STARTING_POT = 8000;
export const STATUS = ["Lobby", "Awaiting randomness", "Trading", "Revealed"] as const;
export const SOURCE = ["—", "REAL VRF", "DEMO (not VRF)"] as const;

export type Seat = {
  owner: PublicKey;
  isBot: boolean;
  balance: anchor.BN;
  shares: anchor.BN;
  entryPrice: anchor.BN;
  payout: anchor.BN;
  sessionKey: PublicKey;
  sessionExpiry: anchor.BN;
};

export type GameState = {
  address: PublicKey;
  creator: PublicKey;
  gameId: anchor.BN;
  status: number;
  randomnessSource: number;
  pot: anchor.BN;
  molePool: anchor.BN;
  price: anchor.BN;
  tick: anchor.BN;
  moleA: number;
  moleB: number;
  vrfNonce: anchor.BN;
  pendingVrf: boolean;
  seats: Seat[];
};

export function gamePda(gameId: number | anchor.BN): [PublicKey, number] {
  const id = typeof gameId === "number" ? new anchor.BN(gameId) : gameId;
  return PublicKey.findProgramAddressSync(
    [Buffer.from("game"), id.toArrayLike(Buffer, "le", 8)],
    PROGRAM_ID
  );
}

export function botPda(gameId: number, seat: number): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("bot"), new anchor.BN(gameId).toArrayLike(Buffer, "le", 8), Buffer.from([seat])],
    PROGRAM_ID
  );
  return pda;
}

export function getProgram(
  connection: Connection,
  wallet: anchor.Wallet
): anchor.Program {
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
  return new anchor.Program(idl as unknown as anchor.Idl, provider);
}

export async function fetchGame(
  program: anchor.Program,
  address: PublicKey
): Promise<GameState | null> {
  try {
    const raw = (await (
      program.account as unknown as {
        game: {
          fetch: (address: PublicKey) => Promise<{
            creator: PublicKey;
            gameId: anchor.BN;
            status: number;
            randomnessSource: number;
            pot: anchor.BN;
            molePool: anchor.BN;
            price: anchor.BN;
            tick: anchor.BN;
            moleA: number;
            moleB: number;
            vrfNonce: anchor.BN;
            pendingVrf: boolean;
            seats: {
              owner: PublicKey;
              isBot: boolean;
              balance: anchor.BN;
              shares: anchor.BN;
              entryPrice: anchor.BN;
              payout: anchor.BN;
              sessionKey: PublicKey;
              sessionExpiry: anchor.BN;
            }[];
          }>;
        };
      }
    ).game.fetch(address)) as {
      creator: PublicKey;
      gameId: anchor.BN;
      status: number;
      randomnessSource: number;
      pot: anchor.BN;
      molePool: anchor.BN;
      price: anchor.BN;
      tick: anchor.BN;
      moleA: number;
      moleB: number;
      vrfNonce: anchor.BN;
      pendingVrf: boolean;
      seats: {
        owner: PublicKey;
        isBot: boolean;
        balance: anchor.BN;
        shares: anchor.BN;
        entryPrice: anchor.BN;
        payout: anchor.BN;
        sessionKey: PublicKey;
        sessionExpiry: anchor.BN;
      }[];
    };
    return { address, ...raw };
  } catch {
    return null;
  }
}

/** Router delegation status for one account. */
export async function getDelegationStatus(account: PublicKey): Promise<{
  isDelegated: boolean;
  fqdn?: string;
}> {
  const res = await fetch(ROUTER_RPC, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getDelegationStatus",
      params: [account.toBase58()],
    }),
  });
  const body = (await res.json()) as {
    result?: { isDelegated?: boolean; fqdn?: string };
    error?: { message: string };
  };
  if (body.error) throw new Error(body.error.message);
  return {
    isDelegated: body.result?.isDelegated ?? false,
    fqdn: body.result?.fqdn,
  };
}

export async function isDelegatedOnBase(
  connection: Connection,
  address: PublicKey
): Promise<boolean> {
  const info = await connection.getAccountInfo(address);
  return info?.owner.equals(DELEGATION_PROGRAM_ID) ?? false;
}

export function shortKey(key: PublicKey): string {
  const s = key.toBase58();
  return `${s.slice(0, 4)}…${s.slice(-4)}`;
}

export function fmtPrice(price: anchor.BN): string {
  return (price.toNumber() / 100).toFixed(2);
}

export { Keypair, SystemProgram, GetCommitmentSignature };
