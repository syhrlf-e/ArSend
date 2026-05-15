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
export async function initDiscovery() {
  try {
    await listen<DiscoveredDevice>(
      'device-discovered',
      (event) => {
        const payload = event.payload.payload;
        
        // Ghost Device Filtering: Ignore devices with name 'Unknown' or empty public keys
        if (!payload || !payload.name || payload.name.trim().toLowerCase() === 'unknown' || !payload.public_key) {
          return;
        }

        discoveredDevices.update((devices) => {
          const existingIndex = devices.findIndex(
            (d) => d.payload.public_key === payload.public_key
          );
          if (existingIndex !== -1) {
            devices[existingIndex] = event.payload;
            return [...devices]; // Svelte 5 assignment reactivity trigger
          }
          return [...devices, event.payload]; // Append new valid device
        });
      }
    );

    await listen<string>(
      'device-removed',
      (event) => {
        const shortPk = event.payload;
        if (!shortPk) return;
        
        discoveredDevices.update((devices) => {
          return devices.filter((d) => !d.payload.public_key.includes(shortPk));
        });
      }
    );
    
    console.log('✅ Discovery listener ready');
  } catch (error) {
    console.error('❌ Discovery listener failed:', error);
  }
}
