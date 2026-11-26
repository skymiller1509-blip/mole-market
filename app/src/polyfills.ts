// Node-global polyfills for the browser build. This module MUST be the
// first import in main.tsx: several Solana libraries touch bare `Buffer`
// (and `global`) while they are being loaded, before main.tsx's own code
// runs, so the globals have to exist before any other import evaluates.
import { Buffer } from "buffer";

(globalThis as unknown as { Buffer: typeof Buffer }).Buffer = Buffer;
(globalThis as unknown as { global: typeof globalThis }).global = globalThis;
