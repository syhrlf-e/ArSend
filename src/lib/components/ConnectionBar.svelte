<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { Wifi, WifiOff } from 'lucide-svelte';

  export let isConnected = false;
  export let deviceName = '';

  const dispatch = createEventDispatcher();
  let isDisconnecting = false;

  onMount(() => {
    const unlisten = listen('connection-state-changed', (event: any) => {
      isConnected = event.payload.connected;
      deviceName = event.payload.device_name || '';
      if (!isConnected) {
        isDisconnecting = false;
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  });

  const handleDisconnect = () => {
    if (isDisconnecting) return;
    isDisconnecting = true;
    dispatch('disconnect');
  };
</script>

<div
  class="mb-4 flex w-full items-center justify-between rounded-2xl border border-slate-200 bg-white p-4 transition-all duration-300"
>
  <div class="flex items-center gap-3.5">
    <div class="flex items-center justify-center">
      {#if isConnected}
        <Wifi size={24} class="text-success" strokeWidth={2.5} />
      {:else}
        <WifiOff size={24} class="text-slate-300" strokeWidth={2} />
      {/if}
    </div>

    <div class="flex flex-col">
      {#if isConnected}
        <span class="text-[13px] font-medium text-slate-500 leading-tight mb-0.5">Terhubung dengan</span>
        <span class="text-[15px] font-bold text-slate-900 leading-tight truncate max-w-35 sm:max-w-50">
          {deviceName}
        </span>
      {:else}
        <span class="text-[14px] font-bold text-slate-900 leading-tight mb-0.5">Belum terhubung</span>
        <span class="text-[13px] font-medium text-slate-500 leading-tight">Siap transfer file</span>
      {/if}
    </div>
  </div>

  {#if isConnected}
    <button
      on:click={handleDisconnect}
      disabled={isDisconnecting}
      class="rounded-xl bg-slate-50 px-4 py-2.5 text-[13px] font-bold text-error transition-colors hover:bg-error-light active:scale-[0.97] cursor-pointer disabled:opacity-50"
    >
      {isDisconnecting ? 'Memutus...' : 'Putuskan'}
    </button>
  {/if}
</div>
