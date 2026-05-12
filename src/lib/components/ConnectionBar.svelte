<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { Wifi, WifiOff } from 'lucide-svelte';

  export let isConnected = false;
  export let deviceName = '';
  export let activeNetwork = 'ArSend Network'; // To be dynamically updated

  // Pulse animation trigger
  let pulsing = false;
  let hbInterval: ReturnType<typeof setInterval>;

  onMount(() => {
    // Listen to connection state changes
    const unlisten = listen('connection-state-changed', (event: any) => {
      isConnected = event.payload.connected;
      deviceName = event.payload.device_name || '';
      
      if (isConnected) {
        startPulse();
      } else {
        stopPulse();
      }
    });

    return () => {
      unlisten.then(f => f());
      stopPulse();
    };
  });

  const startPulse = () => {
    if (hbInterval) clearInterval(hbInterval);
    hbInterval = setInterval(() => {
      pulsing = true;
      setTimeout(() => pulsing = false, 500);
    }, 5000); // Pulse every 5 seconds (heartbeat)
  };

  const stopPulse = () => {
    if (hbInterval) clearInterval(hbInterval);
    pulsing = false;
  };
</script>

<div class="bg-surface p-3 rounded-[14px] shadow-sm border border-border flex items-center justify-between w-full max-w-4xl mb-6 transition-all duration-300">
  <div class="flex items-center gap-3">
    <!-- Status Indicator -->
    <div class="relative flex h-3 w-3 ml-1">
      {#if isConnected}
        <span class={`absolute inline-flex h-full w-full rounded-full bg-success opacity-75 ${pulsing ? 'animate-ping' : ''}`}></span>
        <span class="relative inline-flex rounded-full h-3 w-3 bg-success"></span>
      {:else}
        <span class="relative inline-flex rounded-full h-3 w-3 bg-slate-300"></span>
      {/if}
    </div>

    <!-- Network Info -->
    <div class="flex flex-col">
      <div class="flex items-center gap-1.5">
        {#if isConnected}
          <Wifi size={14} class="text-text-secondary" />
          <span class="text-[14px] font-medium text-text-primary leading-tight">{activeNetwork}</span>
        {:else}
          <WifiOff size={14} class="text-text-tertiary" />
          <span class="text-[14px] text-text-secondary leading-tight">Belum terhubung</span>
        {/if}
      </div>
      {#if isConnected}
        <span class="text-[11px] font-semibold text-accent uppercase tracking-wider mt-0.5">Terhubung ke {deviceName}</span>
      {/if}
    </div>
  </div>

  {#if isConnected}
    <button class="px-3 py-1.5 text-[12px] font-medium text-error hover:bg-error-light rounded-lg transition-colors active:scale-[0.97] cursor-pointer">
      Putuskan
    </button>
  {/if}
</div>
