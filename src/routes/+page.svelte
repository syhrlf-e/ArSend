<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import type { UnlistenFn } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { initDiscovery } from '$lib/stores/connection';
  import { initTransferEvents, incomingOffers, transferProgress, sendFileOffer } from '$lib/stores/transfer';
  import { initSettings, hasSeenOnboarding, completeOnboarding } from '$lib/stores/settings';
  import { initHistory, transferHistory, addHistoryItem, removeHistoryItem, clearHistory } from '$lib/stores/history';
  import { getDeviceType } from '$lib/utils/platform';
  import DeviceList from '$lib/components/DeviceList.svelte';
  import QRDisplay from '$lib/components/QRDisplay.svelte';
  import QRScanner from '$lib/components/QRScanner.svelte';
  import ConnectionBar from '$lib/components/ConnectionBar.svelte';
  import PillSwitch from '$lib/components/PillSwitch.svelte';
  import BottomNav from '$lib/components/BottomNav.svelte';
  import FileProgress from '$lib/components/FileProgress.svelte';
  import ReceiveStandby from '$lib/components/ReceiveStandby.svelte';
  import ApprovalModal from '$lib/components/ApprovalModal.svelte';
  import WelcomeCarousel from '$lib/components/WelcomeCarousel.svelte';
  import FileCard from '$lib/components/FileCard.svelte';
  import TofuModal from '$lib/components/TofuModal.svelte';
  import SettingsTab from '$lib/components/SettingsTab.svelte';
  import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
  import { isDeviceTrusted } from '$lib/stores/trust';
  import { open } from '@tauri-apps/plugin-dialog';
  import { Wifi } from 'lucide-svelte';

  let isMobile = false;
  let showScanner = false;
  let showTofuModal = false;
  let pendingDevice: any = null;
  let isConnected = false;
  let connectedDeviceName = '';
  let connectedDeviceIp = '';
  let connectedDevicePublicKey = '';
  let activeTab = 'Transfer';
  let mobileActiveTab: 'Kirim' | 'Terima' | 'Riwayat' | 'Profile' = 'Kirim';
  let historySearchQuery = '';
  let historyActiveFilter = 'Semua';
  let localDeviceName = '';
  let currentOffer: any = null;
  let intentionalDisconnect = false;
  let unexpectedDisconnect = false;
  let receiveTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let isReceiveTimeout = false;
  let isWaitingForFile = false;
  let showDisconnectConfirm = false;
  let isPreparingFile = false;
  let preparingFileName = '';

  let unlistenConnection: UnlistenFn;
  let unlistenComplete: UnlistenFn;

  $: recvProgresses = Object.values($transferProgress).filter((p) => p.filename.startsWith('recv_'));
  $: sendProgresses = Object.values($transferProgress).filter((p) => !p.filename.startsWith('recv_'));

  $: if ($incomingOffers.length > 0 || recvProgresses.length > 0) clearReceiveTimeout();
  $: if ($incomingOffers.length > 0 && !currentOffer) currentOffer = $incomingOffers[0];

  $: if (isMobile) {
    if (mobileActiveTab === 'Terima' && !receiveTimeoutId && !isReceiveTimeout && !recvProgresses.length && $incomingOffers.length === 0) {
      startReceiveTimeout();
    } else if (mobileActiveTab !== 'Terima') {
      clearReceiveTimeout();
    }
  }

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
      showDisconnectConfirm = false;
      if (isConnected && showScanner) showScanner = false;
    });

    unlistenComplete = await listen('transfer-complete', (event: any) => {
      const payload = event.payload;
      invoke('notify_transfer_complete', {
        filename: payload.filename,
        isReceive: payload.is_receive
      }).catch(console.error);
      addHistoryItem({
        id: payload.nonce || Date.now().toString(),
        filename: payload.filename,
        size: payload.total_bytes || 0,
        type: payload.is_receive ? 'received' : 'sent',
        status: 'success',
        timestamp: Date.now()
      });
    });
  });

  onDestroy(() => {
    unlistenConnection?.();
    unlistenComplete?.();
  });

  const handleKirimClick = async () => {
    try {
      const selected = await open({ multiple: true, directory: false });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        for (const file of paths) {
          isPreparingFile = true;
          preparingFileName = file.split(/[\\/]/).pop() || 'File';
          await sendFileOffer(connectedDeviceIp, file);
        }
        isPreparingFile = false;
      }
    } catch (error) {
      console.error('Failed to open file picker', error);
      isPreparingFile = false;
    }
  };

  const handleApproval = (event: CustomEvent<{ accept: boolean }>) => {
    const offerToProcess = currentOffer;

    // Langsung tutup modal secara sinkron agar UI merespon instan
    $incomingOffers = $incomingOffers.slice(1);
    currentOffer = null;

    if (event.detail.accept && offerToProcess) {
      invoke('accept_file_offer', { nonce: offerToProcess.nonce }).catch(console.error);
    } else if (offerToProcess) {
      invoke('reject_file_offer', { nonce: offerToProcess.nonce }).catch(console.error);
    }
  };

  const handleQRScanned = async (event: CustomEvent<string>) => {
    showScanner = false;
    try {
      const payload = JSON.parse(event.detail);
      
      // 🔒 Phase 6: Strict QR Validation
      if (!payload.ip || !payload.public_key || !payload.token || !payload.device_name) {
        console.error('❌ Invalid QR Payload: missing required fields', payload);
        // You might want to show a toast/notification here in the future
        return;
      }

      console.log(`📡 QR Scanned: connecting to ${payload.device_name} (${payload.ip})`);
      connectedDevicePublicKey = payload.public_key;
      
      await invoke('connect_to_device', {
        ip: payload.ip,
        deviceName: localDeviceName,
        fingerprint: payload.public_key,
        token: payload.token
      });
    } catch (e) {
      console.error('❌ Connection failed or invalid QR data:', e);
    }
  };

  const handleTofuResolve = async (e: CustomEvent<{ trusted: boolean; save: boolean }>) => {
    showTofuModal = false;
    if (e.detail.trusted && pendingDevice) {
      console.log('🤝 Device trusted (TOFU), connecting...');
      connectedDevicePublicKey = pendingDevice.payload.public_key;
      
      // Note: for TOFU (discovery), token is usually null as it's not a pre-shared secret like QR
      invoke('connect_to_device', { 
        ip: pendingDevice.ip, 
        deviceName: localDeviceName, 
        fingerprint: pendingDevice.payload.public_key, 
        token: null 
      }).catch((err) => console.error('❌ Connect error:', err));
    }
    pendingDevice = null;
  };
</script>

{#if showScanner}
  <QRScanner on:scanned={handleQRScanned} on:close={() => (showScanner = false)} />
{/if}

<ApprovalModal
  show={!!currentOffer}
  deviceName={connectedDeviceName}
  deviceType="unknown"
  fileName={currentOffer ? currentOffer.name : ''}
  fileSize={currentOffer ? currentOffer.size : 0}
  fileCount={1}
  on:resolve={handleApproval}
/>

{#if showTofuModal && pendingDevice}
  <TofuModal
    show={true}
    publicKey={pendingDevice.payload.public_key}
    deviceName={pendingDevice.payload.name}
    on:resolve={handleTofuResolve}
  />
{/if}

{#if isPreparingFile}
  <div
    class="fixed inset-0 z-[70] flex items-center justify-center p-4"
    style="background-color: rgba(15,23,42,0.45);"
  >
    <div class="flex flex-col items-center gap-4 rounded-2xl bg-white p-6 shadow-xl max-w-[280px] w-full text-center">
      <div class="h-10 w-10 animate-spin rounded-full border-4 border-accent border-r-transparent"></div>
      <div class="flex flex-col gap-1">
        <span class="text-[15px] font-bold text-slate-900">Menyiapkan File...</span>
        <span class="text-[13px] text-slate-500 line-clamp-2">Memindai <strong class="text-slate-700">{preparingFileName}</strong></span>
      </div>
    </div>
  </div>
{/if}

{#if isMobile && !$hasSeenOnboarding}
  <WelcomeCarousel on:complete={completeOnboarding} />
{/if}

<main class="flex min-h-screen flex-col items-center bg-slate-100 px-4 pt-[108px] pb-6">

  <header class="fixed top-0 left-0 right-0 z-40 flex justify-center bg-white/80 backdrop-blur-md border-b border-slate-200/60 px-4 pt-[54px] pb-3.5">
    <div class="flex w-full max-w-md items-center justify-between">
      <span class="text-[22px] font-bold tracking-tight text-slate-900">ArSend</span>

      <div class="flex items-center gap-3">
        {#if isConnected}
          {#if showDisconnectConfirm}
            <button
              on:click={async () => {
                showDisconnectConfirm = false;
                intentionalDisconnect = true;
                await invoke('disconnect_device');
              }}
              class="flex items-center gap-2 rounded-full bg-error px-4 py-1.5 transition-colors active:scale-95 cursor-pointer"
            >
              <Wifi size={14} class="text-white" strokeWidth={2.5} />
              <span class="text-[13px] font-bold text-white">Putuskan</span>
            </button>
          {:else}
            <button
              on:click={() => (showDisconnectConfirm = true)}
              class="flex items-center gap-2 px-2 py-1.5 transition-colors active:scale-95 cursor-pointer"
            >
              <Wifi size={16} class="text-success" strokeWidth={2.5} />
              <span class="text-[13px] font-semibold text-slate-800 truncate max-w-[120px] sm:max-w-[200px]">
                {connectedDeviceName}
              </span>
            </button>
          {/if}
        {:else if isMobile}
          <button
            id="btn-open-scanner"
            on:click={() => (showScanner = true)}
            class="cursor-pointer rounded-full border border-slate-200 bg-white p-2 text-slate-500 transition-colors hover:border-accent hover:text-accent active:scale-[0.97]"
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
      </div>
    </div>
  </header>

  {#if unexpectedDisconnect}
    <section class="mt-10 flex w-full max-w-md flex-col items-center justify-center gap-6">
      <div class="mb-2 flex h-24 w-24 items-center justify-center rounded-full border border-error/20 bg-error-light text-error">
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
            invoke('connect_to_device', { ip: connectedDeviceIp, deviceName: localDeviceName, fingerprint: connectedDevicePublicKey, token: null })
              .catch((e) => console.error('Reconnect error:', e));
          }}
          class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl bg-accent py-3.5 text-[15px] font-semibold text-white transition-colors hover:bg-accent-hover active:scale-[0.98]"
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
            invoke('connect_to_device', { ip: device.ip, deviceName: localDeviceName, fingerprint: device.payload.public_key, token: null })
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
    <section class="flex w-full max-w-md flex-col gap-4 {isMobile ? 'pb-[180px]' : ''}">
      {#if isMobile}
        <BottomNav bind:activeTab={mobileActiveTab} />
      {:else}
        <PillSwitch bind:activeTab />
      {/if}

      {#snippet sendProgressBlock()}
        {#if sendProgresses.length > 0}
          <div class="flex flex-col gap-1">
            <h2 class="ml-1 mb-2 text-[15px] font-semibold text-slate-900">Pengiriman Berhasil</h2>
            {#each sendProgresses as p (p.nonce || p.filename)}
              <FileProgress
                nonce={p.nonce}
                filename={p.filename}
                progress={p.progress}
                speedMbS={p.speed_mb_s}
                sentBytes={p.sent_bytes}
                totalBytes={p.total_bytes}
                isReceiving={false}
                status={p.status ?? (p.progress >= 100 ? 'success' : 'sending')}
                error={p.error ?? ''}
                canResume={p.can_resume ?? false}
              />
            {/each}
          </div>
        {/if}
      {/snippet}

      {#snippet recvProgressBlock()}
        {#if recvProgresses.length > 0}
          <div class="flex flex-col gap-1">
            <h2 class="ml-1 mb-2 text-[15px] font-semibold text-slate-900">File Masuk</h2>
            {#each recvProgresses as p (p.nonce || p.filename)}
              <FileProgress
                nonce={p.nonce}
                filename={p.filename.replace('recv_', '')}
                progress={p.progress}
                speedMbS={p.speed_mb_s}
                sentBytes={p.sent_bytes}
                totalBytes={p.total_bytes}
                isReceiving={true}
                status={p.status ?? (p.progress >= 100 ? 'success' : 'receiving')}
                error={p.error ?? ''}
                canResume={p.can_resume ?? false}
              />
            {/each}
          </div>
        {/if}
      {/snippet}

      {#if !isMobile && activeTab === 'Transfer'}
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
              class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-slate-200 bg-white p-6 transition-colors hover:border-accent hover:bg-accent-light active:scale-[0.98]"
            >
              <div class="rounded-full bg-accent-light p-3 text-accent">
                <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
              </div>
              <span class="text-[15px] font-semibold text-slate-900">Kirim File</span>
            </button>

            <button
              on:click={() => { isWaitingForFile = true; startReceiveTimeout(); }}
              class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-slate-200 bg-white p-6 transition-colors hover:border-emerald-500 hover:bg-emerald-50 active:scale-[0.98]"
            >
              <div class="rounded-full bg-emerald-50 p-3 text-emerald-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
              </div>
              <span class="text-[15px] font-semibold text-slate-900">Terima File</span>
            </button>
          </div>
        {/if}

        {@render sendProgressBlock()}
        {@render recvProgressBlock()}

        {#if sendProgresses.length === 0 && recvProgresses.length === 0}
          <div class="flex flex-col items-center justify-center rounded-2xl border border-slate-200 bg-white p-8 text-center">
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

      {:else if isMobile && mobileActiveTab === 'Kirim'}
        {@render sendProgressBlock()}

        {#if sendProgresses.length === 0}
          <div class="mt-16 flex flex-col items-center justify-center text-center">
            <div class="mb-5 flex h-20 w-20 items-center justify-center rounded-full bg-accent-light text-accent">
              <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
            </div>
            <h3 class="text-[18px] font-bold text-slate-900">Mulai Mengirim File</h3>
            <p class="mt-2 max-w-[260px] text-[14px] leading-relaxed text-slate-500">
              Tekan tombol <strong class="text-accent">+</strong> di pojok kanan bawah untuk memilih file yang akan dikirim ke <strong class="text-slate-700">{connectedDeviceName}</strong>.
            </p>
          </div>
        {/if}

        <button
          on:click={handleKirimClick}
          class="fixed bottom-[164px] right-6 z-40 flex h-[60px] w-[60px] cursor-pointer items-center justify-center rounded-full bg-accent text-white transition-transform active:scale-90"
          title="Pilih File"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>

      {:else if isMobile && mobileActiveTab === 'Terima'}
        {#if recvProgresses.length === 0}
          <ReceiveStandby
            {connectedDeviceName}
            {isReceiveTimeout}
            on:retry={startReceiveTimeout}
            on:cancel={() => { mobileActiveTab = 'Kirim'; clearReceiveTimeout(); }}
          />
        {/if}
        {@render recvProgressBlock()}

      {:else if (!isMobile && activeTab === 'Riwayat') || (isMobile && mobileActiveTab === 'Riwayat')}
        <div class="flex flex-col gap-4 rounded-2xl border border-slate-200 bg-white p-5">
          <div class="flex items-center justify-between">
            <h2 class="text-[15px] font-semibold text-slate-900">Riwayat Transfer</h2>
            {#if $transferHistory.length > 0}
              <button
                on:click={async () => await clearHistory()}
                class="text-[12px] font-semibold text-error hover:text-error/80 transition-colors cursor-pointer active:scale-95"
              >
                Bersihkan Semua
              </button>
            {/if}
          </div>

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
                  on:delete={async () => await removeHistoryItem(item.id)}
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
      {:else if (!isMobile && activeTab === 'Profile') || (isMobile && mobileActiveTab === 'Profile')}
        <SettingsTab {isMobile} />
      {/if}
    </section>
  {/if}
</main>
