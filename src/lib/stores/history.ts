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
    // Deduplicate array based on ID to fix existing corrupted data
    const uniqueSaved = saved.filter((item, index, self) =>
      index === self.findIndex((t) => t.id === item.id)
    );
    transferHistory.set(uniqueSaved);
    
    // Automatically repair the store file if we found duplicates
    if (uniqueSaved.length !== saved.length) {
      store.set('history_list', uniqueSaved).then(() => store.save());
    }
  }
}

export async function addHistoryItem(item: HistoryItem) {
  transferHistory.update(history => {
    // Prevent duplicates by filtering out any existing item with the same id
    const filteredHistory = history.filter(h => h.id !== item.id);
    const newHistory = [item, ...filteredHistory];
    
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

export async function removeHistoryItem(id: string) {
  transferHistory.update(history => {
    const newHistory = history.filter(item => item.id !== id);
    if (store) {
      store.set('history_list', newHistory).then(() => store.save());
    }
    return newHistory;
  });
}