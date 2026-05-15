<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { RefreshCw, QrCode } from 'lucide-svelte';

  export let isMobile = false;
  export let deviceName = '';

  let qrSvg = '';
  let payloadInfo = { ip: '', port: 0, token: '' };
  let timeLeft = 180;
  let timer: ReturnType<typeof setInterval>;
  let isGenerating = false;
  let isExpired = false;

  const generateQR = async () => {
    isGenerating = true;
    isExpired = false;
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
        isExpired = true;
      }
    }, 1000);
  };

  onMount(() => {
    if (!isMobile) generateQR();
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });

  $: progressPercentage = (timeLeft / 180) * 100;
  $: timerColor = isExpired ? 'text-error' : timeLeft < 30 ? 'text-warning' : 'text-slate-500';
  $: barColor = timeLeft < 30 ? 'bg-warning' : 'bg-accent';
</script>

{#if !isMobile}
  <div
    class="relative w-full max-w-sm overflow-hidden rounded-[16px] border border-slate-200 bg-white p-6 shadow-sm"
  >
    <!-- Header -->
    <div class="mb-5 flex w-full items-center justify-between">
      <div class="flex items-center gap-2 text-slate-900">
        <QrCode size={18} class="text-accent" strokeWidth={1.5} />
        <span class="text-[15px] font-semibold">QR Pairing</span>
      </div>
      <button
        on:click={generateQR}
        disabled={isGenerating}
        class="rounded-full p-2 text-slate-400 transition-colors hover:bg-slate-100 hover:text-accent active:scale-[0.97] cursor-pointer disabled:opacity-40"
        title="Refresh QR Code"
      >
        <RefreshCw size={16} class={isGenerating ? 'animate-spin' : ''} />
      </button>
    </div>

    <!-- QR Code area -->
    <div
      class="relative mx-auto mb-5 flex h-48 w-48 items-center justify-center overflow-hidden rounded-[12px] border border-slate-100 bg-slate-50 p-2"
    >
      <!-- Loading overlay -->
      {#if isGenerating}
        <div
          class="absolute inset-0 z-10 flex items-center justify-center rounded-[12px] bg-white"
        >
          <RefreshCw size={24} class="animate-spin text-accent" />
        </div>
      {/if}

      <!-- Expired overlay -->
      {#if isExpired}
        <div
          class="absolute inset-0 z-20 flex flex-col items-center justify-center rounded-[12px] bg-white/95"
        >
          <p class="mb-3 text-[13px] font-semibold text-slate-900">Kode QR Kedaluwarsa</p>
          <button
            on:click={generateQR}
            class="rounded-[10px] bg-accent px-4 py-2 text-[13px] font-semibold text-white shadow-sm transition-colors hover:bg-accent-hover active:scale-[0.97] cursor-pointer"
          >
            Perbarui QR
          </button>
        </div>
      {/if}

      <!-- QR SVG -->
      {#if qrSvg}
        <div
          class="flex h-full w-full items-center justify-center transition-opacity duration-300 [&>svg]:h-full [&>svg]:w-full {isExpired
            ? 'opacity-20'
            : 'opacity-100'}"
        >
          {@html qrSvg}
        </div>
      {:else if !isGenerating}
        <QrCode size={48} class="text-slate-300" />
      {/if}
    </div>

    <!-- IP info pill -->
    {#if payloadInfo.ip}
      <p
        class="mx-auto mb-4 w-fit rounded-full border border-slate-200 bg-slate-50 px-3 py-1 font-mono text-[12px] text-slate-500 transition-opacity duration-300 {isExpired
          ? 'opacity-40'
          : 'opacity-100'}"
      >
        {payloadInfo.ip}:{payloadInfo.port}
      </p>
    {/if}

    <!-- Token info -->
    <div
      class="mb-4 w-full rounded-[12px] border border-slate-200 bg-slate-50 p-3 transition-opacity duration-300 {isExpired
        ? 'opacity-40'
        : 'opacity-100'}"
    >
      <div class="mb-1.5 flex items-center justify-between">
        <span class="text-[11px] font-semibold uppercase tracking-widest text-slate-400">Token</span>
        <span class="font-mono text-[12px] text-slate-500">{payloadInfo.token}</span>
      </div>
      <div class="border-t border-slate-200 pt-2 text-center">
        <button
          class="w-full py-0.5 text-center text-[12px] font-medium text-slate-400 transition-colors hover:text-slate-900 cursor-pointer"
        >
          Gunakan kode manual
        </button>
      </div>
    </div>

    <!-- Timer bar -->
    <div class="w-full">
      <div class="mb-1.5 flex items-center justify-between">
        <span class="text-[11px] font-semibold uppercase tracking-widest text-slate-400">
          Kadaluwarsa dalam
        </span>
        <span class="text-[11px] font-semibold {timerColor}">
          {Math.floor(timeLeft / 60)}:{(timeLeft % 60).toString().padStart(2, '0')}
        </span>
      </div>
      <div class="h-1.5 w-full overflow-hidden rounded-full bg-slate-100">
        <div
          class="h-full rounded-full transition-all duration-1000 ease-linear {barColor}"
          style="width: {progressPercentage}%"
        ></div>
      </div>
    </div>
  </div>
{/if}