import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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
export const transferProgress = writable<Record<string, FileTransferProgress>>({});

export async function initTransferEvents() {
    await listen<FileOffer>('file-offer-received', (event) => {
        incomingOffers.update(offers => [...offers, event.payload]);
    });

    await listen<FileTransferProgress>('transfer-progress-receive', (event) => {
        transferProgress.update(progress => {
            progress[event.payload.filename] = event.payload;
            return { ...progress };
        });
    });

    await listen<FileTransferProgress>('transfer-progress-send', (event) => {
        transferProgress.update(progress => {
            progress[event.payload.filename] = event.payload;
            return { ...progress };
        });
    });
}

export async function sendFileOffer(ip: string, filePath: string, nonce: string) {
    try {
        await invoke('send_file', { ip, filePath, nonce });
    } catch (error) {
        console.error('Failed to send file:', error);
    }
}
