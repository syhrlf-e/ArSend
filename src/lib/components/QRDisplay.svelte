<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { RefreshCw, QrCode } from 'lucide-svelte';

  export let isMobile = false;
  export let deviceName = '';

  let qrSvg = '';
  let payloadInfo = {
    ip: '',
    port: 0,
    token: ''
  };
  
  let timeLeft = 180; // 3 minutes in seconds
  let timer: ReturnType<typeof setInterval>;
  let isGenerating = false;

  const generateQR = async () => {
    isGenerating = true;
    try {
      const payload: any = await invoke('get_qr_payload', { deviceName });
      payloadInfo = {
        ip: payload.ip,
        port: payload.port,
        token: payload.token.substring(0, 8) + '...'
      };
      
      const payloadStr = JSON.stringify(payload);
      qrSvg = await invoke('generate_qr_svg', { payload: payloadStr });
      
      timeLeft = 180;
      startTimer();
    } catch (error) {
      console.error('Failed to generate QR:', error);
    } finally {
      isGenerating = false;
    }
  };

  const startTimer = () => {
    if (timer) clearInterval(timer);
    timer = setInterval(() => {
      if (timeLeft > 0) {
        timeLeft -= 1;
      } else {
        clearInterval(timer);
        generateQR(); // Auto-refresh when expired
      }
    }, 1000);
  };

  onMount(() => {
    if (!isMobile) {
      generateQR();
    }
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  $: progressPercentage = (timeLeft / 180) * 100;
  $: progressColor = timeLeft < 30 ? 'bg-warning' : 'bg-accent';
</script>

{#if !isMobile}
  <div class="bg-surface border border-border rounded-[16px] p-6 flex flex-col items-center justify-center max-w-sm w-full mx-auto relative overflow-hidden group">
    
    <!-- Header -->
    <div class="w-full flex justify-between items-center mb-6">
      <div class="flex items-center gap-2 text-text-primary font-semibold">
        <QrCode size={20} class="text-accent" />
        <span class="text-[15px]">QR Pairing</span>
      </div>
      <button 
        on:click={generateQR} 
        disabled={isGenerating}
        class="p-2 text-text-secondary hover:text-accent hover:bg-accent-light rounded-full transition-colors active:scale-[0.97] cursor-pointer disabled:opacity-50"
        title="Refresh QR Code"
      >
        <RefreshCw size={18} class={isGenerating ? 'animate-spin' : ''} />
      </button>
    </div>

    <!-- QR Code SVG -->
    <div class="w-48 h-48 bg-surface-2 rounded-xl flex items-center justify-center mb-6 p-2 relative">
      {#if isGenerating}
        <div class="absolute inset-0 bg-surface/80 flex items-center justify-center z-10 rounded-xl backdrop-blur-[2px]">
          <RefreshCw size={24} class="animate-spin text-accent" />
        </div>
      {/if}
      
      {#if qrSvg}
        <div class="w-full h-full flex items-center justify-center [&>svg]:w-full [&>svg]:h-full" style="color: transparent;">
          {@html qrSvg}
        </div>
      {:else if !isGenerating}
        <QrCode size={48} class="text-border-strong" />
      {/if}
      </div>

      {#if payloadInfo.ip}
      <p class="text-[13px] font-mono text-text-secondary bg-surface-2 px-3 py-1 rounded-md mb-6 border border-border">
        {payloadInfo.ip}
      </p>
      {/if}

      <!-- Footer / Info -->
    <div class="w-full bg-slate-100 rounded-xl p-3 mb-6 border border-slate-200">
      <div class="flex justify-between items-center mb-1">
        <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">IP Address</span>
        <span class="text-[12px] font-mono text-slate-900">{payloadInfo.ip}:{payloadInfo.port}</span>
      </div>
      <div class="flex justify-between items-center mb-2">
        <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Token</span>
        <span class="text-[12px] font-mono text-slate-500">{payloadInfo.token}</span>
      </div>
      <div class="text-center pt-2 border-t border-slate-200">
        <button class="text-[12px] text-accent font-medium hover:text-accent-hover transition-colors">
          Gunakan kode manual
        </button>
      </div>
    </div>

    <!-- Timer Progress -->
    <div class="w-full mt-auto">
      <div class="flex justify-between text-[11px] font-semibold text-text-tertiary uppercase tracking-wider mb-2">
        <span>Refresh otomatis</span>
        <span>{Math.floor(timeLeft / 60)}:{(timeLeft % 60).toString().padStart(2, '0')}</span>
      </div>
      <div class="h-1.5 w-full bg-surface-2 rounded-full overflow-hidden">
        <div 
          class="h-full {progressColor} transition-all duration-1000 ease-linear"
          style="width: {progressPercentage}%"
        ></div>
      </div>
    </div>
  </div>
{/if}
