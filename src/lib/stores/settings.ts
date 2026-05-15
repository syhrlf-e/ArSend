import { writable } from 'svelte/store';
import { hostname } from '@tauri-apps/plugin-os';
import { Store } from '@tauri-apps/plugin-store';

export const deviceName = writable<string>('');
export const hasSeenOnboarding = writable<boolean>(false);
export const downloadFolder = writable<string>('Documents/ArSend/');

let store: Store;

function isValidName(name: string | null | undefined): name is string {
  if (!name) return false;
  const trimmed = name.trim();
  if (trimmed === '') return false;
  if (trimmed === 'null') return false;
  if (trimmed === 'undefined') return false;
  if (trimmed === 'localhost') return false;
  return true;
}

export async function initSettings(): Promise<string> {
  // ✅ Pakai Store.load() bukan new Store()
  store = await Store.load('arsend_settings.json');

  const savedName = await store.get<string>('device_name');
  const seenOnboarding = await store.get<boolean>('has_seen_onboarding');
  const savedFolder = await store.get<string>('download_folder');
  
  if (seenOnboarding !== undefined && seenOnboarding !== null) {
    hasSeenOnboarding.set(seenOnboarding);
  }

  if (savedFolder) {
    downloadFolder.set(savedFolder);
  }

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

export async function saveDownloadFolder(path: string) {
  downloadFolder.set(path);
  if (store) {
    await store.set('download_folder', path);
    await store.save();
  }
}

export async function completeOnboarding() {
  hasSeenOnboarding.set(true);
  if (store) {
    await store.set('has_seen_onboarding', true);
    await store.save();
  }
}
