import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getDeviceType } from '../utils/platform';

export interface DiscoveryPayload {
  name: string;
  public_key: string;
  version: string;
  port: number;
  device_type: string;
}

export interface DiscoveredDevice {
  payload: DiscoveryPayload;
  ip: string;
}

export const discoveredDevices = writable<DiscoveredDevice[]>([]);

// ─────────────────────────────────────────────
// Initialize Discovery Service
// ─────────────────────────────────────────────
export async function initDiscovery(name: string) {
  try {

    // Ambil public key dari backend
    const publicKeyData: {
      public_key_hex: string;
    } = await invoke('get_public_key');

    // Payload discovery
    const payload: DiscoveryPayload = {
      name: name || 'ArSend Device',
      public_key: publicKeyData.public_key_hex,
      version: '1.0.1',
      port: 9527,
      device_type: getDeviceType()
    };

    console.log('📡 Starting discovery...', payload);

    // Start discovery service
    await invoke('start_discovery', { payload });

    console.log('✅ Discovery started');

    // Listen device discovery event
    await listen<DiscoveredDevice>(
      'device-discovered',
      (event) => {

        console.log(
          '📥 Device discovered:',
          event.payload
        );

        discoveredDevices.update((devices) => {

          // Cari device berdasarkan public key
          const existingIndex = devices.findIndex(
            (device) =>
              device.payload.public_key ===
              event.payload.payload.public_key
          );

          // Update existing device
          if (existingIndex !== -1) {

            devices[existingIndex] = event.payload;

            return [...devices];
          }

          // Tambahkan device baru
          return [...devices, event.payload];
        });
      }
    );

  } catch (error) {

    console.error(
      '❌ Failed to initialize discovery:',
      error
    );
  }
}
