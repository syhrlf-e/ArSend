<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { initDiscovery } from '$lib/stores/connection';
  import { initTransferEvents, incomingOffers, transferProgress, sendFileOffer } from '$lib/stores/transfer';
  import { initSettings, hasSeenOnboarding, completeOnboarding } from '$lib/stores/settings';
  import { initHistory, transferHistory, addHistoryItem } from '$lib/stores/history';
  import { getDeviceType } from '$lib/utils/platform';
  import DeviceList from '$lib/components/DeviceList.svelte';
  import QRDisplay from '$lib/components/QRDisplay.svelte';
  import QRScanner from '$lib/components/QRScanner.svelte';
  import ConnectionBar from '$lib/components/ConnectionBar.svelte';
  import PillSwitch from '$lib/components/PillSwitch.svelte';
  import FileProgress from '$lib/components/FileProgress.svelte';
  import ReceiveStandby from '$lib/components/ReceiveStandby.svelte';
  import ApprovalModal from '$lib/components/ApprovalModal.svelte';
  import WelcomeCarousel from '$lib/components/WelcomeCarousel.svelte';
  import FileCard from '$lib/components/FileCard.svelte';
  import TofuModal from '$lib/components/TofuModal.svelte';
  import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
  import { isDeviceTrusted } from '$lib/stores/trust';
  import { open } from '@tauri-apps/plugin-dialog';

  let isMobile = false;
  let showScanner = false;
  let showTofuModal = false;
  let pendingDevice: any = null;
  let isConnected = false;
  let connectedDeviceName = '';
  let connectedDeviceIp = '';
  let connectedDevicePublicKey = '';
  let activeTab = 'Transfer';
  let historySearchQuery = '';
  let historyActiveFilter = 'Semua';
  let localDeviceName = '';
  let currentOffer: any = null;
  let intentionalDisconnect = false;
  let unexpectedDisconnect = false;
  let receiveTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let isReceiveTimeout = false;
  let isWaitingForFile = false;

  let unlistenConnection: UnlistenFn;
  let unlistenReceive: UnlistenFn;
  let unlistenSend: UnlistenFn;

  $: recvProgresses = Object.values($transferProgress).filter((p) => p.filename.startsWith('recv_'));
  $: sendProgresses = Object.values($transferProgress).filter((p) => !p.filename.startsWith('recv_'));

  $: if ($incomingOffers.length > 0 || recvProgresses.length > 0) clearReceiveTimeout();
  $: if ($incomingOffers.length > 0 && !currentOffer) currentOffer = $incomingOffers[0];

  $: filteredHistory = $transferHistory.filter((item) => {
    const matchesSearch = item.filename.toLowerCase().includes(historySearchQuery.toLowerCase());
    const matchesFilter =
      historyActiveFilter === 'Semua' ||
      (historyActiveFilter === 'Terkirim' && item.type === 'sent') ||
      (historyActiveFilter === 'Diterima' && item.type === 'received');
    return matchesSearch && matchesFilter;
  });

  const startReceiveTimeout = () => {
    clearReceiveTimeout();
    isReceiveTimeout = false;
    receiveTimeoutId = setTimeout(() => { isReceiveTimeout = true; }, 300000);
  };

  const clearReceiveTimeout = () => {
    if (receiveTimeoutId) { clearTimeout(receiveTimeoutId); receiveTimeoutId = null; }
    isReceiveTimeout = false;
  };

  onMount(async () => {
    await new Promise((resolve) => setTimeout(resolve, 100));
    isMobile = getDeviceType() === 'mobile';
    localDeviceName = await initSettings();

    try {
      let granted = await isPermissionGranted();
      if (!granted) {
        const perm = await requestPermission();
        granted = perm === 'granted';
      }
    } catch (e) {
      console.error('Notification permission error:', e);
    }

    await initDiscovery();
    await initHistory();
    initTransferEvents();

    try {
      const pkInfo: any = await invoke('get_public_key');
      const payload = {
        name: localDeviceName,
        public_key: pkInfo.public_key_hex || '',
        version: '1.0.1',
        port: 9527,
        device_type: isMobile ? 'mobile' : 'desktop'
      };
      await invoke('start_discovery', { payload });
    } catch (e) {
      console.error('❌ Discovery error:', e);
    }

    await listen('server-ready', () => { console.log('✅ Server ready'); });
    invoke('start_server', { deviceName: localDeviceName }).catch(console.error);
    invoke('start_transfer_server').catch(console.error);

    unlistenConnection = await listen('connection-state-changed', (event: any) => {
      const wasConnected = isConnected;
      isConnected = event.payload.connected;
      connectedDeviceName = event.payload.device_name || '';
      connectedDeviceIp = event.payload.ip || '';
      connectedDevicePublicKey = event.payload.public_key || '';

      if (isConnected && !wasConnected) {
        invoke('notify_connection', { deviceName: connectedDeviceName }).catch(console.error);
      }
      if (!isConnected && wasConnected && !intentionalDisconnect) {
        unexpectedDisconnect = true;
      }
      intentionalDisconnect = false;
      if (isConnected && showScanner) showScanner = false;
    });

    unlistenReceive = await listen('transfer-progress-receive', (event: any) => {
      if (event.payload.progress >= 100) {
        invoke('notify_transfer_complete', { filename: event.payload.filename, isReceive: true }).catch(console.error);
        addHistoryItem({
          id: Date.now().toString(),
          filename: event.payload.filename.replace('recv_', ''),
          size: event.payload.total_bytes,
          type: 'received',
          status: 'success',
          timestamp: Date.now()
        });
      }
    });

    unlistenSend = await listen('transfer-progress-send', (event: any) => {
      if (event.payload.progress >= 100) {
        invoke('notify_transfer_complete', { filename: event.payload.filename, isReceive: false }).catch(console.error);
        addHistoryItem({
          id: Date.now().toString(),
          filename: event.payload.filename,
          size: event.payload.total_bytes,
          type: 'sent',
          status: 'success',
          timestamp: Date.now()
        });
      }
    });
  });

  onDestroy(() => {
    unlistenConnection?.();
    unlistenReceive?.();
    unlistenSend?.();
  });

  const handleKirimClick = async () => {
    try {
      const selected = await open({ multiple: true, directory: false });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        for (const file of paths) {
          await sendFileOffer(connectedDeviceIp, file);
        }
      }
    } catch (error) {
      console.error('Failed to open file picker', error);
    }
  };

  const handleApproval = async (event: CustomEvent<{ accept: boolean }>) => {
    if (event.detail.accept && currentOffer) {
      await invoke('accept_file_offer', { nonce: currentOffer.nonce }).catch(console.error);
    } else if (currentOffer) {
      await invoke('reject_file_offer', { nonce: currentOffer.nonce }).catch(console.error);
    }
    $incomingOffers = $incomingOffers.slice(1);
    currentOffer = null;
  };

  const handleQRScanned = async (event: CustomEvent<string>) => {
    showScanner = false;
    try {
      const payload = JSON.parse(event.detail);
      if (!payload.ip) return;
      connectedDevicePublicKey = payload.public_key;
      await invoke('connect_to_device', { ip: payload.ip, deviceName: localDeviceName, fingerprint: payload.public_key });
    } catch (e) {
      console.error('❌ Connection failed:', e);
    }
  };

  const handleTofuResolve = async (e: CustomEvent<{ trusted: boolean; save: boolean }>) => {
    showTofuModal = false;
    if (e.detail.trusted && pendingDevice) {
      connectedDevicePublicKey = pendingDevice.payload.public_key;
      invoke('connect_to_device', { ip: pendingDevice.ip, deviceName: localDeviceName, fingerprint: pendingDevice.payload.public_key })
        .catch((err) => console.error('❌ Connect error:', err));
    }
    pendingDevice = null;
  };
</script>

{#if showScanner}
  <QRScanner on:scanned={handleQRScanned} on:close={() => (showScanner = false)} />
{/if}

{#if currentOffer}
  <ApprovalModal
    show={true}
    deviceName={connectedDeviceName}
    deviceType="unknown"
    fileName={currentOffer.name}
    fileSize={currentOffer.size}
    fileCount={1}
    on:resolve={handleApproval}
  />
{/if}

{#if showTofuModal && pendingDevice}
  <TofuModal
    show={true}
    publicKey={pendingDevice.payload.public_key}
    deviceName={pendingDevice.payload.name}
    on:resolve={handleTofuResolve}
  />
{/if}

{#if isMobile && !$hasSeenOnboarding}
  <WelcomeCarousel on:complete={completeOnboarding} />
{/if}

<main class="flex min-h-screen flex-col items-center bg-slate-50 px-4 py-6">

  <header class="mb-6 flex w-full max-w-md items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="text-[22px] font-bold tracking-tight text-slate-900">ArSend</span>
      {#if localDeviceName}
        <span class="font-mono text-[13px] text-slate-400">· {localDeviceName}</span>
      {/if}
    </div>

    {#if isMobile && !isConnected}
      <button
        id="btn-open-scanner"
        on:click={() => (showScanner = true)}
        class="cursor-pointer rounded-full border border-slate-200 bg-white p-2 text-slate-500 shadow-sm transition-colors hover:border-accent hover:text-accent active:scale-[0.97]"
        title="Scan QR Code"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 7V5a2 2 0 0 1 2-2h2"/>
          <path d="M17 3h2a2 2 0 0 1 2 2v2"/>
          <path d="M21 17v2a2 2 0 0 1-2 2h-2"/>
          <path d="M7 21H5a2 2 0 0 1-2-2v-2"/>
          <rect width="7" height="7" x="3" y="3" rx="1"/>
          <rect width="7" height="7" x="14" y="3" rx="1"/>
          <rect width="7" height="7" x="14" y="14" rx="1"/>
          <rect width="7" height="7" x="3" y="14" rx="1"/>
        </svg>
      </button>
    {/if}
  </header>

  <section class="w-full max-w-md">
    <ConnectionBar
      {isConnected}
      deviceName={connectedDeviceName}
      on:disconnect={async () => {
        intentionalDisconnect = true;
        await invoke('disconnect_device');
      }}
    />
  </section>

  {#if unexpectedDisconnect}
    <section class="mt-10 flex w-full max-w-md flex-col items-center justify-center gap-6">
      <div class="mb-2 flex h-24 w-24 items-center justify-center rounded-full border border-error/20 bg-error-light text-error shadow-sm">
        <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10.71 5.42A2 2 0 0 0 9.27 6H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-5"/>
          <path d="m22 2-6 6"/>
          <path d="m16 2 6 6"/>
        </svg>
      </div>

      <div class="mb-4 flex flex-col gap-2 text-center">
        <h2 class="text-[22px] font-bold text-slate-900">Koneksi Terputus</h2>
        <p class="max-w-70 text-[14px] text-slate-500">
          Sambungan ke <strong class="text-slate-900">{connectedDeviceName}</strong> terputus karena masalah jaringan.
        </p>
      </div>

      <div class="flex w-full flex-col gap-3">
        <button
          on:click={() => {
            unexpectedDisconnect = false;
            invoke('connect_to_device', { ip: connectedDeviceIp, deviceName: localDeviceName, fingerprint: connectedDevicePublicKey })
              .catch((e) => console.error('Reconnect error:', e));
          }}
          class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl bg-accent py-3.5 text-[15px] font-semibold text-white shadow-sm transition-colors hover:bg-accent-hover active:scale-[0.98]"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
            <path d="M3 3v5h5"/>
          </svg>
          Coba Lagi
        </button>
        <button
          on:click={() => {
            unexpectedDisconnect = false;
            connectedDeviceName = '';
            connectedDeviceIp = '';
          }}
          class="w-full cursor-pointer rounded-xl border border-slate-200 bg-white py-3.5 text-[15px] font-semibold text-slate-500 transition-colors hover:border-slate-300 hover:text-slate-900 active:scale-[0.98]"
        >
          Kembali ke Beranda
        </button>
      </div>
    </section>

  {:else if !isConnected}
    <section class="flex w-full max-w-md flex-col items-center gap-6">
      <DeviceList
        on:connect={async (e) => {
          const device = e.detail;
          const trusted = await isDeviceTrusted(device.payload.public_key);
          if (trusted) {
            connectedDevicePublicKey = device.payload.public_key;
            invoke('connect_to_device', { ip: device.ip, deviceName: localDeviceName, fingerprint: device.payload.public_key })
              .catch((err) => console.error('❌ Connect error:', err));
          } else {
            pendingDevice = device;
            showTofuModal = true;
          }
        }}
      />
      {#if !isMobile}
        <QRDisplay isMobile={false} deviceName={localDeviceName} />
      {/if}
    </section>

  {:else}
    <section class="flex w-full max-w-md flex-col gap-4">
      <PillSwitch bind:activeTab />

      {#if activeTab === 'Transfer'}

        {#if isWaitingForFile}
          <ReceiveStandby
            {connectedDeviceName}
            {isReceiveTimeout}
            on:retry={startReceiveTimeout}
            on:cancel={() => { isWaitingForFile = false; clearReceiveTimeout(); }}
          />
        {:else}
          <div class="grid grid-cols-2 gap-4">
            <button
              on:click={handleKirimClick}
              class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm transition-colors hover:border-accent hover:bg-accent-light active:scale-[0.98]"
            >
              <div class="rounded-full bg-accent-light p-3 text-accent">
                <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
              </div>
              <span class="text-[15px] font-semibold text-slate-900">Kirim File</span>
            </button>

            <button
              on:click={() => { isWaitingForFile = true; startReceiveTimeout(); }}
              class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm transition-colors hover:border-emerald-500 hover:bg-emerald-50 active:scale-[0.98]"
            >
              <div class="rounded-full bg-emerald-50 p-3 text-emerald-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
              </div>
              <span class="text-[15px] font-semibold text-slate-900">Terima File</span>
            </button>
          </div>
        {/if}

        {#if sendProgresses.length > 0}
          <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <h2 class="mb-3 text-[15px] font-semibold text-slate-900">Progres Pengiriman</h2>
            <div class="flex flex-col gap-3">
              {#each sendProgresses as p (p.filename)}
                <FileProgress
                  filename={p.filename}
                  progress={p.progress}
                  speedMbS={p.speed_mb_s}
                  sentBytes={p.sent_bytes}
                  totalBytes={p.total_bytes}
                  isReceiving={false}
                  status={p.progress >= 100 ? 'success' : 'sending'}
                />
              {/each}
            </div>
          </div>
        {/if}

        {#if recvProgresses.length > 0}
          <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <h2 class="mb-3 text-[15px] font-semibold text-slate-900">File Masuk</h2>
            <div class="flex flex-col gap-3">
              {#each recvProgresses as p (p.filename)}
                <FileProgress
                  filename={p.filename.replace('recv_', '')}
                  progress={p.progress}
                  speedMbS={p.speed_mb_s}
                  sentBytes={p.sent_bytes}
                  totalBytes={p.total_bytes}
                  isReceiving={true}
                  status={p.progress >= 100 ? 'success' : 'receiving'}
                />
              {/each}
            </div>
          </div>
        {/if}

        {#if sendProgresses.length === 0 && recvProgresses.length === 0}
          <div class="flex flex-col items-center justify-center rounded-2xl border border-slate-200 bg-white p-8 text-center shadow-sm">
            <div class="mb-4 rounded-full bg-accent-light p-4 text-accent">
              <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
                <polyline points="14 2 14 8 20 8"/>
              </svg>
            </div>
            <p class="text-[14px] font-semibold text-slate-900">Belum ada transfer</p>
            <p class="mt-1 text-[13px] text-slate-500">Pilih file di atas untuk mulai mengirim.</p>
          </div>
        {/if}

      {:else if activeTab === 'Riwayat'}
        <div class="flex flex-col gap-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <h2 class="text-[15px] font-semibold text-slate-900">Riwayat Transfer</h2>

          <div class="relative">
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>
            </svg>
            <input
              id="history-search"
              type="text"
              bind:value={historySearchQuery}
              placeholder="Cari file..."
              class="w-full rounded-[10px] border border-slate-200 bg-slate-50 py-2.5 pl-9 pr-4 text-[14px] text-slate-900 placeholder:text-slate-400 focus:border-accent focus:outline-none transition-colors"
            />
          </div>

          <div class="flex flex-wrap gap-2">
            {#each ['Semua', 'Terkirim', 'Diterima'] as filter}
              <button
                on:click={() => (historyActiveFilter = filter)}
                class="cursor-pointer rounded-full border px-3 py-1.5 text-[12px] font-semibold transition-colors
                  {historyActiveFilter === filter
                    ? 'border-accent bg-accent text-white'
                    : 'border-slate-200 bg-slate-50 text-slate-500 hover:border-accent hover:text-accent'}"
              >
                {filter}
              </button>
            {/each}
          </div>

          {#if filteredHistory.length > 0}
            <div class="flex flex-col gap-2">
              {#each filteredHistory as item (item.id)}
                <FileCard
                  filename={item.filename}
                  size={item.size}
                  type={item.type}
                  status={item.status}
                  timestamp={item.timestamp}
                />
              {/each}
            </div>
          {:else}
            <div class="flex flex-col items-center justify-center py-10 text-center">
              <div class="mb-4 rounded-full bg-slate-100 p-4 text-slate-400">
                <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                  <path d="M3 3v5h5"/>
                  <path d="M12 7v5l4 2"/>
                </svg>
              </div>
              <p class="text-[14px] font-semibold text-slate-900">Riwayat kosong</p>
              <p class="mt-1 text-[13px] text-slate-500">Transfer yang sudah selesai akan muncul di sini.</p>
            </div>
          {/if}
        </div>
      {/if}
    </section>
  {/if}
</main>
