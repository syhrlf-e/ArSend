import { writable } from 'svelte/store';
import { hostname } from '@tauri-apps/plugin-os';
import { Store } from '@tauri-apps/plugin-store';

export const deviceName = writable<string>('');

let store: Store;

// Helper: validasi nama device benar-benar usable
function isValidName(name: string | null | undefined): name is string {
  if (!name) return false;
  const trimmed = name.trim();
  if (trimmed === '') return false;
  if (trimmed === 'null') return false;      // ← tangkap string "null" dari plugin
  if (trimmed === 'undefined') return false; // ← jaga-jaga
  if (trimmed === 'localhost') return false; // ← hostname Android kadang ini
  return true;
}

export async function initSettings(): Promise<string> {
  store = new Store('arsend_settings.json');
  const savedName = await store.get<string>('device_name');

  // Kalau sudah ada nama valid di store, langsung pakai
  if (isValidName(savedName)) {
    deviceName.set(savedName);
    return savedName;
  }

  // Belum ada — coba ambil dari hostname OS
  let resolvedName = 'ArSend Device'; // fallback default
  try {
    const host = await hostname();
    if (isValidName(host)) {
      resolvedName = host;
    }
  } catch (e) {
    console.error('Failed to get hostname:', e);
  }

  // Simpan ke store
  deviceName.set(resolvedName);
  await store.set('device_name', resolvedName);
  await store.save();

  return resolvedName;
}

export async function saveDeviceName(name: string) {
  deviceName.set(name);
  if (store) {
    await store.set('device_name', name);
    await store.save();
  }
}
