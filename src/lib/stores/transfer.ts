import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface FileTransferProgress {
  filename: string;
  progress: number;
  speed_mb_s: number;
  sent_bytes: number;
  total_bytes: number;
}

export interface FileOffer {
  name: string;
  size: number;
  hash_total: string;
  num_chunks: number;
  nonce: string;
}

export const incomingOffers = writable<FileOffer[]>([]);
export const transferProgress = writable<Record<string, FileTransferProgress>>(
  {},
);

export const pendingOutboundTransfers = writable<
  Record<string, { ip: string; filePath: string; hash_total: string }>
>({});

let isInitialized = false;

export async function initTransferEvents() {
  if (isInitialized) return;
  isInitialized = true;

  await listen<any>("transfer-error", (event) => {
    console.error("❌ Transfer Error from Rust:", event.payload);
  });

  await listen<FileOffer>("file-offer-received", (event) => {
    incomingOffers.update((offers) => {
      // Mencegah duplikasi offer (terutama karena efek HMR di Svelte saat development)
      if (offers.some((o) => o.nonce === event.payload.nonce)) return offers;
      return [...offers, event.payload];
    });
  });

  await listen<string>("file-accept-received", async (event) => {
    const nonce = event.payload;
    console.log(`✅ file-accept-received for nonce:`, nonce);
    const pending = get(pendingOutboundTransfers)[nonce];
    if (pending) {
      try {
        console.log(`🚀 Invoking send_file to ${pending.ip}`);
        await invoke("send_file", {
          ip: pending.ip,
          filePath: pending.filePath,
          nonce,
        });
        console.log(`✅ send_file finished successfully`);
      } catch (error) {
        console.error("❌ Failed to send file after accept:", error);
      }
      pendingOutboundTransfers.update((p) => {
        const newP = { ...p };
        delete newP[nonce];
        return newP;
      });
    } else {
      console.warn(
        "⚠️ file-accept-received: no pending transfer for nonce",
        nonce,
      );
    }
  });

  await listen<string>("file-reject-received", (event) => {
    const nonce = event.payload;
    console.log(`🚫 File offer rejected for nonce: ${nonce}`);
    pendingOutboundTransfers.update((p) => {
      const newP = { ...p };
      delete newP[nonce];
      return newP;
    });
  });

  await listen<FileTransferProgress>("transfer-progress-receive", (event) => {
    transferProgress.update((progress) => {
      progress[event.payload.filename] = event.payload;
      return { ...progress };
    });
  });

  await listen<FileTransferProgress>("transfer-progress-send", (event) => {
    transferProgress.update((progress) => {
      progress[event.payload.filename] = event.payload;
      return { ...progress };
    });
  });
}

export async function sendFileOffer(ip: string, filePath: string) {
  try {
    const result: { nonce: string; hash_total: string } = await invoke(
      "send_file_offer",
      { filePath },
    );
    const { nonce, hash_total } = result;

    pendingOutboundTransfers.update((p) => {
      p[nonce] = { ip, filePath, hash_total };
      return p;
    });

    console.log(`📤 File offer sent | nonce: ${nonce} | ip: ${ip}`);
  } catch (error) {
    console.error("❌ Failed to send file offer:", error);
  }
}
