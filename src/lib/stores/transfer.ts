import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface FileTransferProgress {
  nonce: string;
  filename: string;
  progress: number;
  speed_mb_s: number;
  sent_bytes: number;
  total_bytes: number;
  status?: "sending" | "receiving" | "success" | "failed" | "cancelled";
  error?: string;
}

export interface FileTransferComplete {
  nonce: string;
  filename: string;
  save_path: string;
  is_receive: boolean;
  total_bytes: number;
}

export interface FileOffer {
  name: string;
  size: number;
  hash_total: string;
  num_chunks: number;
  nonce: string;
}

interface FileTransferError {
  nonce: string;
  filename: string;
  error: string;
}

export const incomingOffers = writable<FileOffer[]>([]);
export const transferProgress = writable<Record<string, FileTransferProgress>>(
  {},
);

export const pendingOutboundTransfers = writable<
  Record<string, { ip: string; filePath: string; hash_total: string }>
>({});

let isInitialized = false;

function upsertTransferProgress(
  progress: Record<string, FileTransferProgress>,
  payload: FileTransferProgress,
) {
  const key = payload.nonce || payload.filename;
  const current = progress[key];
  const normalizedProgress =
    payload.total_bytes > 0 && payload.sent_bytes >= payload.total_bytes
      ? 100
      : payload.progress;

  if (key !== payload.filename) {
    delete progress[payload.filename];
  }

  if (current && current.progress >= 100 && normalizedProgress < 100) {
    return progress;
  }

  progress[key] = {
    ...payload,
    progress: Math.max(current?.progress ?? 0, normalizedProgress),
    sent_bytes: Math.max(current?.sent_bytes ?? 0, payload.sent_bytes),
  };

  return progress;
}

export async function initTransferEvents() {
  if (isInitialized) return;
  isInitialized = true;

  await listen<FileTransferError>("transfer-error", (event) => {
    console.error("❌ Transfer Error from Rust:", event.payload);
    const payload = event.payload;

    // Cleanup pending outbound if it exists
    if (payload.nonce) {
      pendingOutboundTransfers.update((p) => {
        const newP = { ...p };
        delete newP[payload.nonce];
        return newP;
      });
    }

    transferProgress.update((progress) => {
      const key =
        payload.nonce ||
        Object.keys(progress).find(
          (candidate) => progress[candidate]?.filename === payload.filename,
        ) ||
        payload.filename;
      const existing = progress[key] ?? progress[payload.filename];

      if (key !== payload.filename) {
        delete progress[payload.filename];
      }

      progress[key] = {
        nonce: payload.nonce || existing?.nonce || "",
        filename: existing?.filename || payload.filename,
        progress: existing?.progress ?? 0,
        speed_mb_s: 0,
        sent_bytes: existing?.sent_bytes ?? 0,
        total_bytes: existing?.total_bytes ?? 0,
        status: payload.error.toLowerCase().includes("cancelled")
          ? "cancelled"
          : "failed",
        error: payload.error,
      };

      return { ...progress };
    });
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
      upsertTransferProgress(progress, event.payload);
      return { ...progress };
    });
  });

  await listen<FileTransferProgress>("transfer-progress-send", (event) => {
    transferProgress.update((progress) => {
      upsertTransferProgress(progress, event.payload);
      return { ...progress };
    });
  });

  await listen<FileTransferComplete>("transfer-complete", (event) => {
    const payload = event.payload;

    // Cleanup pending outbound if it exists
    if (payload.nonce) {
      pendingOutboundTransfers.update((p) => {
        const newP = { ...p };
        delete newP[payload.nonce];
        return newP;
      });
    }

    const filename = payload.is_receive
      ? `recv_${payload.filename}`
      : payload.filename;
    const key = payload.nonce || filename;

    transferProgress.update((progress) => {
      if (key !== filename) {
        delete progress[filename];
      }

      progress[key] = {
        nonce: payload.nonce,
        filename,
        progress: 100,
        speed_mb_s: 0,
        sent_bytes: payload.total_bytes,
        total_bytes: payload.total_bytes,
        status: "success",
      };
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
