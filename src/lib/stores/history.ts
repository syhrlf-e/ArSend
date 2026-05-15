import { writable } from 'svelte/store';
import { Store } from '@tauri-apps/plugin-store';

export interface HistoryItem {
  id: string;
  filename: string;
  size: number;
  type: 'sent' | 'received';
  status: 'success' | 'failed' | 'cancelled';
  timestamp: number;
}

export const transferHistory = writable<HistoryItem[]>([]);

let store: Store;

export async function initHistory() {
  store = await Store.load('arsend_history.json');
  const saved = await store.get<HistoryItem[]>('history_list');
  if (saved) {
    transferHistory.set(saved);
  }
}

export async function addHistoryItem(item: HistoryItem) {
  transferHistory.update(history => {
    const newHistory = [item, ...history];
    if (store) {
      store.set('history_list', newHistory).then(() => store.save());
    }
    return newHistory;
  });
}

export async function clearHistory() {
  transferHistory.set([]);
  if (store) {
    await store.set('history_list', []);
    await store.save();
  }
}