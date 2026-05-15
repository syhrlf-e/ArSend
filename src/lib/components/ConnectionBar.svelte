<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { Wifi, WifiOff } from 'lucide-svelte';

  export let isConnected = false;
  export let deviceName = '';
  export let activeNetwork = 'ArSend Network';

  const dispatch = createEventDispatcher();

  let pulsing = false;
  let hbInterval: ReturnType<typeof setInterval>;
  let isDisconnecting = false;

  onMount(() => {
    const unlisten = listen('connection-state-changed', (event: any) => {
      isConnected = event.payload.connected;
      deviceName = event.payload.device_name || '';
      if (isConnected) {
        startPulse();
      } else {
        stopPulse();
        isDisconnecting = false;
      }
    });
    return () => {
      unlisten.then((f) => f());
      stopPulse();
    };
  });

  const startPulse = () => {
    if (hbInterval) clearInterval(hbInterval);
    hbInterval = setInterval(() => {
      pulsing = true;
      setTimeout(() => (pulsing = false), 500);
    }, 5000);
  };

  const stopPulse = () => {
    if (hbInterval) clearInterval(hbInterval);
    pulsing = false;
  };

  const handleDisconnect = () => {
    if (isDisconnecting) return;
    isDisconnecting = true;
    dispatch('disconnect');
  };
</script>

<div
  class="mb-4 flex w-full items-center justify-between rounded-[14px] border border-slate-200 bg-white p-3 shadow-sm transition-all duration-300"
>
  <div class="flex items-center gap-3">

    <div class="relative ml-1 flex h-3 w-3">
      {#if isConnected}
        <span
          class="absolute inline-flex h-full w-full rounded-full bg-success opacity-75 {pulsing
            ? 'animate-ping'
            : ''}"
        ></span>
        <span class="relative inline-flex h-3 w-3 rounded-full bg-success"></span>
      {:else}
        <span class="relative inline-flex h-3 w-3 rounded-full bg-slate-300"></span>
      {/if}
    </div>

    <div class="flex flex-col">
      <div class="flex items-center gap-1.5">
        {#if isConnected}
          <Wifi size={14} class="text-slate-500" />
          <span class="text-[14px] font-medium leading-tight text-slate-900">{activeNetwork}</span>
        {:else}
          <WifiOff size={14} class="text-slate-400" />
          <span class="text-[14px] leading-tight text-slate-500">Belum terhubung</span>
        {/if}
      </div>
      {#if isConnected}
        <span class="mt-0.5 text-[11px] font-semibold uppercase tracking-wider text-accent">
          Terhubung ke {deviceName}
        </span>
      {/if}
    </div>
  </div>

  {#if isConnected}
    <button
      on:click={handleDisconnect}
      disabled={isDisconnecting}
      class="rounded-lg px-3 py-1.5 text-[12px] font-medium text-error transition-colors hover:bg-error-light active:scale-[0.97] cursor-pointer disabled:opacity-50"
    >
      {isDisconnecting ? 'Memutuskan...' : 'Putuskan'}
    </button>
  {/if}
</div>
