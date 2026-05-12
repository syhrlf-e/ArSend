<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';

  import {
    initDiscovery
  } from '$lib/stores/connection';

  import {
    initTransferEvents,
    incomingOffers,
    transferProgress,
    sendFileOffer
  } from '$lib/stores/transfer';

  import {
    initSettings
  } from '$lib/stores/settings';

  import {
    getDeviceType
  } from '$lib/utils/platform';

  import DeviceList from '$lib/components/DeviceList.svelte';
  import QRDisplay from '$lib/components/QRDisplay.svelte';
  import QRScanner from '$lib/components/QRScanner.svelte';
  import ConnectionBar from '$lib/components/ConnectionBar.svelte';
  import PillSwitch from '$lib/components/PillSwitch.svelte';
  import Dropzone from '$lib/components/Dropzone.svelte';
  import FileProgress from '$lib/components/FileProgress.svelte';
  import ApprovalModal from '$lib/components/ApprovalModal.svelte';

  // ─────────────────────────────────────────
  // STATE
  // ─────────────────────────────────────────

  let isMobile = false;
  let showScanner = false;

  let isConnected = false;
  let connectedDeviceName = '';
  let connectedDeviceIp = '';

  let activeTab = 'Transfer';

  let localDeviceName = '';
  let currentOffer: any = null;

  let unlistenConnection: UnlistenFn;
  let unlistenReceive: UnlistenFn;
  let unlistenSend: UnlistenFn;

  // ─────────────────────────────────────────
  // REACTIVE DATA
  // ─────────────────────────────────────────

  $: recvProgresses = Object.values($transferProgress)
    .filter((p) => p.filename.startsWith('recv_'));

  $: sendProgresses = Object.values($transferProgress)
    .filter((p) => !p.filename.startsWith('recv_'));

  // Auto open approval modal
  $: if ($incomingOffers.length > 0 && !currentOffer) {
    currentOffer = $incomingOffers[0];
  }

  // ─────────────────────────────────────────
  // LIFECYCLE
  // ─────────────────────────────────────────

  onMount(async () => {

    isMobile = getDeviceType() === 'mobile';

    // Init settings & ambil nama device
    localDeviceName = await initSettings();

    // Init discovery dengan nama device langsung
    await initDiscovery(localDeviceName);

    // Init transfer listeners
    initTransferEvents();

    // Start websocket server
    invoke('start_server', {
      deviceName: localDeviceName
    }).catch((e) => {

      console.error('❌ Failed to start server:', e);

      if (!isMobile) {
        alert(
          `Gagal membuka port server di Desktop!\n\n${e}`
        );
      }
    });

    // Start transfer server
    invoke('start_transfer_server')
      .catch(console.error);

    // ───────────────────────────────────────
    // CONNECTION LISTENER
    // ───────────────────────────────────────
    unlistenConnection = await listen(
      'connection-state-changed',
      (event: any) => {

        const wasConnected = isConnected;

        isConnected = event.payload.connected;

        connectedDeviceName =
          event.payload.device_name || '';

        connectedDeviceIp = '127.0.0.1';

        // Notification saat connect
        if (isConnected && !wasConnected) {

          invoke('notify_connection', {
            deviceName: connectedDeviceName
          }).catch(console.error);
        }

        // Auto close scanner
        if (isConnected && showScanner) {
          showScanner = false;
        }
      }
    );

    // ───────────────────────────────────────
    // RECEIVE PROGRESS LISTENER
    // ───────────────────────────────────────
    unlistenReceive = await listen(
      'transfer-progress-receive',
      (event: any) => {

        if (event.payload.progress >= 100) {

          invoke('notify_transfer_complete', {
            filename: event.payload.filename,
            isReceive: true
          }).catch(console.error);
        }
      }
    );

    // ───────────────────────────────────────
    // SEND PROGRESS LISTENER
    // ───────────────────────────────────────
    unlistenSend = await listen(
      'transfer-progress-send',
      (event: any) => {

        if (event.payload.progress >= 100) {

          invoke('notify_transfer_complete', {
            filename: event.payload.filename,
            isReceive: false
          }).catch(console.error);
        }
      }
    );
  });

  onDestroy(() => {

    if (unlistenConnection) {
      unlistenConnection();
    }

    if (unlistenReceive) {
      unlistenReceive();
    }

    if (unlistenSend) {
      unlistenSend();
    }
  });

  // ─────────────────────────────────────────
  // FILE SEND
  // ─────────────────────────────────────────

  const handleFilesSelected = async (
    event: CustomEvent<string[]>
  ) => {

    const files = event.detail;

    for (const file of files) {

      const nonce = 'mock_nonce';

      await sendFileOffer(
        connectedDeviceIp,
        file,
        nonce
      );
    }
  };

  // ─────────────────────────────────────────
  // FILE APPROVAL
  // ─────────────────────────────────────────

  const handleApproval = (
    event: CustomEvent<{ accept: boolean }>
  ) => {

    if (event.detail.accept) {

      // TODO:
      // Send FILE_ACCEPT via WebSocket

    } else {

      // TODO:
      // Send FILE_REJECT via WebSocket
    }

    $incomingOffers =
      $incomingOffers.slice(1);

    currentOffer = null;
  };

  // ─────────────────────────────────────────
  // QR CONNECTION
  // ─────────────────────────────────────────

  const handleQRScanned = async (
    event: CustomEvent<string>
  ) => {

    // Tutup scanner dulu
    showScanner = false;

    let targetIp = 'Unknown';

    try {

      const payload = JSON.parse(event.detail);

      if (!payload.ip) {

        alert(
          'Format QR tidak valid.\nIP tidak ditemukan.'
        );

        return;
      }

      targetIp = payload.ip;

      alert(
        `Mencoba terhubung ke:\n${targetIp}`
      );

      await invoke('connect_to_device', {
        ip: payload.ip,
        deviceName: localDeviceName
      });

    } catch (e) {

      console.error(
        '❌ Connection failed:',
        e
      );

      alert(
        `Gagal terhubung.\n\nTarget IP: ${targetIp}\n\n${e}`
      );
    }
  };
</script>
