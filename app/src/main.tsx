import React from "react";
import ReactDOM from "react-dom/client";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import { PhantomWalletAdapter } from "@solana/wallet-adapter-wallets";
import App from "./App";
import { BASE_RPC } from "./mole";
import "@solana/wallet-adapter-react-ui/styles.css";

// The production browser build needs these: the app uses Buffer and Anchor
// uses `global`, but Vite does not provide Node globals in the browser, so
// without them the page mounts blank.
import { Buffer } from "buffer";
(window as unknown as { Buffer: typeof Buffer }).Buffer = Buffer;
(window as unknown as { global: typeof globalThis }).global = globalThis;

const wallets = [new PhantomWalletAdapter()];

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ConnectionProvider endpoint={BASE_RPC}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <App />
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  </React.StrictMode>
);
