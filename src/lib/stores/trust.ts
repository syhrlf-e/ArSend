import { invoke } from '@tauri-apps/api/core';

export async function isDeviceTrusted(publicKey: string): Promise<boolean> {
  try {
    return await invoke('is_device_trusted', { publicKey });
  } catch (error) {
    console.error('Failed to check if device is trusted:', error);
    return false;
  }
}

export async function trustDevice(publicKey: string, name: string): Promise<void> {
  try {
    await invoke('trust_device', { publicKey, name });
  } catch (error) {
    console.error('Failed to trust device:', error);
  }
}

export async function removeTrustedDevice(publicKey: string): Promise<void> {
  try {
    await invoke('remove_trusted_device', { publicKey });
  } catch (error) {
    console.error('Failed to remove trusted device:', error);
  }
}

export async function getTrustedDevices(): Promise<{public_key: string, name: string}[]> {
  try {
    return await invoke('get_trusted_devices');
  } catch (error) {
    console.error('Failed to get trusted devices:', error);
    return [];
  }
}
