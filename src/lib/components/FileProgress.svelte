<script lang="ts">
  import { formatBytes, formatTime } from '$lib/utils/format';
  import { ArrowUp, RotateCcw, XCircle } from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { downloadFolder } from '$lib/stores/settings';
  import { resumeTransfer } from '$lib/stores/transfer';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';

  export let nonce = '';
  export let filename = '';
  export let progress = 0;
  export let speedMbS = 0;
  export let sentBytes = 0;
  export let totalBytes = 0;
  export let isReceiving = false;
  export let status: 'sending' | 'receiving' | 'success' | 'failed' | 'cancelled' = 'sending';
  export let error = '';
  export let canResume = false;

  let isResuming = false;
  let touchStartX = 0;
  let touchCurrentX = 0;
  let isSwipeOpen = false;

  $: remainingBytes = totalBytes - sentBytes;
  $: estimatedSeconds = speedMbS > 0 ? remainingBytes / (speedMbS * 1024 * 1024) : 0;

  const handleOpenFolder = async () => {
    if (!isReceiving) return;
    try {
      const folder = get(downloadFolder);
      await openPath(folder);
    } catch (e) {
      console.error('Failed to open folder:', e);
    }
  };

  const handleCancel = async () => {
    try {
      await invoke('cancel_transfer', { nonce }).catch(() => {});
      status = 'cancelled';
      isSwipeOpen = false;
    } catch (e) {
      console.error('Failed to cancel transfer:', e);
    }
  };

  const handleResume = async () => {
    if (!nonce || isResuming) return;

    try {
      isResuming = true;
      await resumeTransfer(nonce);
    } catch (e) {
      console.error('Failed to resume transfer:', e);
    } finally {
      isResuming = false;
    }
  };

  $: progressColor =
    status === 'failed'
      ? '#ef4444'
      : status === 'cancelled'
        ? '#f59e0b'
        : progress >= 100
          ? '#22c55e'
          : '#22c55e';
  $: clippedProgress = Math.max(0, Math.min(100, progress));
  $: showResumeAction = canResume && !isReceiving && (status === 'failed' || status === 'cancelled');
  $: canSwipeCancel = status === 'sending' || status === 'receiving';
  $: dragOffset = canSwipeCancel
    ? Math.max(-72, Math.min(0, touchCurrentX - touchStartX))
    : 0;
  $: cardOffset = canSwipeCancel && isSwipeOpen ? -72 : dragOffset;

  const handlePointerDown = (event: PointerEvent) => {
    if (!canSwipeCancel) return;
    touchStartX = event.clientX;
    touchCurrentX = event.clientX;
  };

  const handlePointerMove = (event: PointerEvent) => {
    if (!canSwipeCancel || touchStartX === 0) return;
    const deltaX = event.clientX - touchStartX;
    if (deltaX < 0 || isSwipeOpen) {
      touchCurrentX = event.clientX;
    }
  };

  const handlePointerUp = () => {
    if (!canSwipeCancel || touchStartX === 0) return;
    const deltaX = touchCurrentX - touchStartX;

    if (deltaX < -32) {
      isSwipeOpen = true;
    } else if (deltaX > 24) {
      isSwipeOpen = false;
    }

    touchStartX = 0;
    touchCurrentX = 0;
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (!canSwipeCancel) return;

    if (event.key === 'ArrowLeft') {
      isSwipeOpen = true;
    } else if (event.key === 'ArrowRight' || event.key === 'Escape') {
      isSwipeOpen = false;
    }
  };
</script>

{#if status === 'success'}
  <div class="flex items-center justify-between w-full rounded-[14px] border border-slate-200 bg-white p-3.5 group">
    <div class="flex items-center gap-3 min-w-0">
      <FileIcon {filename} />
      <div class="flex flex-col min-w-0">
        <span class="truncate text-[14px] font-semibold text-slate-900" title={filename}>
          {filename}
        </span>
        <span class="text-[12px] text-slate-500">
          {formatBytes(totalBytes)}
        </span>
      </div>
    </div>
    {#if isReceiving}
      <button
        class="shrink-0 rounded-lg bg-slate-100 px-3 py-1.5 text-[12px] font-semibold text-slate-600 transition-colors hover:bg-slate-200 active:scale-[0.97] cursor-pointer"
        on:click={handleOpenFolder}
      >
        Buka
      </button>
    {/if}
  </div>
{:else}
  <div class="relative overflow-hidden rounded-[14px]">
    {#if canSwipeCancel}
      <button
        on:click={handleCancel}
        class="absolute inset-y-0 right-0 flex w-[72px] cursor-pointer flex-col items-center justify-center gap-1 bg-error text-white transition-colors active:bg-error/90"
        title="Batalkan Transfer"
      >
        <XCircle size={18} />
        <span class="text-[11px] font-semibold">Batal</span>
      </button>
    {/if}

  <div
    class="relative flex flex-col gap-3 rounded-[14px] border border-slate-200 bg-white p-4 transition-transform duration-200"
    style="transform: translateX({cardOffset}px); touch-action: pan-y;"
    on:pointerdown={handlePointerDown}
    on:pointermove={handlePointerMove}
    on:pointerup={handlePointerUp}
    on:pointercancel={handlePointerUp}
    on:keydown={handleKeydown}
    role="button"
    tabindex={canSwipeCancel ? 0 : -1}
    aria-label={`Transfer ${filename}`}
  >
    <!-- Header row -->
    <div class="flex items-start justify-between gap-4">
      <div class="flex min-w-0 items-center gap-3">
        <FileIcon {filename} />
        <div class="flex min-w-0 flex-col">
          <span
            class="truncate text-[14px] font-semibold text-slate-900"
            title={filename}
          >
            {filename}
          </span>
          <span class="text-[12px] text-slate-500">
            {formatBytes(sentBytes)} / {formatBytes(totalBytes)}
          </span>
        </div>
      </div>

      <div class="flex shrink-0 items-center">
        {#if showResumeAction}
          <button
            on:click={handleResume}
            disabled={isResuming}
            class="flex h-8 cursor-pointer items-center gap-1.5 rounded-lg bg-accent px-2.5 text-[12px] font-semibold text-white transition-colors hover:bg-accent-hover active:scale-[0.97] disabled:cursor-not-allowed disabled:opacity-70"
            title="Lanjutkan transfer"
          >
            <span class:animate-spin={isResuming}>
              <RotateCcw size={13} />
            </span>
            {isResuming ? '...' : 'Lanjutkan'}
          </button>
        {:else}
          <div class="flex min-w-[42px] flex-col items-center gap-0.5">
            <div class="relative h-[20px] w-[20px] overflow-hidden text-slate-300">
              <ArrowUp size={20} strokeWidth={2.2} />
              <div
                class="absolute inset-0 overflow-hidden"
                style="clip-path: inset({100 - clippedProgress}% 0 0 0); color: {progressColor};"
              >
                <ArrowUp size={20} strokeWidth={2.2} />
              </div>
            </div>
            <span class="text-[11px] font-bold leading-none text-slate-700">
              {clippedProgress.toFixed(0)}%
            </span>
          </div>
        {/if}
      </div>
    </div>

    <div class="w-full">
      <div class="mb-1.5 flex items-center justify-between text-[11px] font-semibold">
        {#if status === 'sending' || status === 'receiving'}
          <span class="text-slate-500">
            {speedMbS.toFixed(1)} MB/s · Sisa {formatTime(estimatedSeconds * 1000)}
          </span>
        {:else if status === 'cancelled'}
          <span class="text-warning">Dibatalkan</span>
        {:else if status === 'failed'}
          <span class="text-error">Gagal</span>
        {/if}
      </div>

      {#if (status === 'failed' || status === 'cancelled') && error}
        <p class="mt-2 text-[12px] leading-snug text-error">{error}</p>
      {/if}

    </div>
  </div>
  </div>
{/if}
