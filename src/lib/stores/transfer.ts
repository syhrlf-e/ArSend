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
  can_resume?: boolean;
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

const outboundTransferSources = writable<Record<string, { ip: string; filePath: string }>>({});

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

function findResumableReceiveKey(offer: FileOffer) {
  const receiveFilename = `recv_${offer.name}`;
  const progress = get(transferProgress);

  return Object.keys(progress).find((key) => {
    const item = progress[key];
    const isSameFile =
      item.filename === receiveFilename &&
      (item.total_bytes === 0 || item.total_bytes === offer.size);
    const canContinue =
      item.status === "failed" || item.status === "cancelled";

    return isSameFile && canContinue && item.sent_bytes < offer.size;
  });
}

function prepareReceiveResume(oldKey: string, offer: FileOffer) {
  transferProgress.update((progress) => {
    const existing = progress[oldKey];
    if (!existing) return progress;

    progress[offer.nonce] = {
      ...existing,
      nonce: offer.nonce,
      status: "receiving",
      error: "",
      speed_mb_s: 0,
      can_resume: false,
    };
    delete progress[oldKey];

    return { ...progress };
  });
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
      const retrySource = payload.nonce
        ? get(outboundTransferSources)[payload.nonce]
        : undefined;

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
        can_resume: !!retrySource,
      };

      return { ...progress };
    });
  });

  await listen<FileOffer>("file-offer-received", (event) => {
    const offer = event.payload;
    const resumableKey = findResumableReceiveKey(offer);

    if (resumableKey) {
      console.log("▶️ Auto-accepting resumable receive offer:", offer.nonce);
      prepareReceiveResume(resumableKey, offer);
      invoke("accept_file_offer", { nonce: offer.nonce }).catch((error) => {
        console.error("❌ Failed to accept resumable offer:", error);
        transferProgress.update((progress) => {
          if (progress[offer.nonce]) {
            progress[offer.nonce] = {
              ...progress[offer.nonce],
              status: "failed",
              error: "Gagal melanjutkan penerimaan file.",
            };
          }
          return { ...progress };
        });
      });
      return;
    }

    incomingOffers.update((offers) => {
      // Mencegah duplikasi offer (terutama karena efek HMR di Svelte saat development)
      if (offers.some((o) => o.nonce === offer.nonce)) return offers;
      return [...offers, offer];
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
      outboundTransferSources.update((sources) => {
        const newSources = { ...sources };
        delete newSources[payload.nonce];
        return newSources;
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
        can_resume: false,
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
      return { ...p };
    });
    outboundTransferSources.update((sources) => {
      sources[nonce] = { ip, filePath };
      return { ...sources };
    });

    console.log(`📤 File offer sent | nonce: ${nonce} | ip: ${ip}`);
    return result;
  } catch (error) {
    console.error("❌ Failed to send file offer:", error);
    return null;
  }
}

export async function resumeTransfer(nonce: string) {
  const source = get(outboundTransferSources)[nonce];
  if (!source) {
    console.warn("⚠️ resumeTransfer: no retry source for nonce", nonce);
    return;
  }

  transferProgress.update((progress) => {
    if (progress[nonce]) {
      progress[nonce] = {
        ...progress[nonce],
        status: "sending",
        error: "",
        speed_mb_s: 0,
        can_resume: false,
      };
    }
    return { ...progress };
  });

  const result = await sendFileOffer(source.ip, source.filePath);
  if (!result) {
    transferProgress.update((progress) => {
      if (progress[nonce]) {
        progress[nonce] = {
          ...progress[nonce],
          status: "failed",
          error: "Gagal mengirim ulang penawaran file.",
          can_resume: true,
        };
      }
      return { ...progress };
    });
    return;
  }

  if (result.nonce === nonce) return;

  transferProgress.update((progress) => {
    const existing = progress[nonce];
    if (existing) {
      progress[result.nonce] = {
        ...existing,
        nonce: result.nonce,
        status: "sending",
        error: "",
        speed_mb_s: 0,
        can_resume: false,
      };
      delete progress[nonce];
    }
    return { ...progress };
  });
}
