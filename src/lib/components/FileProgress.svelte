<script lang="ts">
  import { formatBytes, formatTime } from '$lib/utils/format';
  import { XCircle, CheckCircle } from 'lucide-svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { downloadFolder } from '$lib/stores/settings';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';

  export let filename = '';
  export let progress = 0;
  export let speedMbS = 0;
  export let sentBytes = 0;
  export let totalBytes = 0;
  export let isReceiving = false;
  export let status: 'sending' | 'receiving' | 'success' | 'failed' | 'cancelled' = 'sending';

  $: remainingBytes = totalBytes - sentBytes;
  $: estimatedSeconds = speedMbS > 0 ? remainingBytes / (speedMbS * 1024 * 1024) : 0;

  const animatedProgress = tweened(0, { duration: 300, easing: cubicOut });
  $: animatedProgress.set(progress);

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
      await invoke('cancel_transfer', { filename }).catch(() => {});
      status = 'cancelled';
    } catch (e) {
      console.error('Failed to cancel transfer:', e);
    }
  };

  $: barColor =
    status === 'success'
      ? 'bg-success'
      : status === 'failed'
        ? 'bg-error'
        : status === 'cancelled'
          ? 'bg-warning'
          : 'bg-accent';
</script>

<div class="flex flex-col gap-3 rounded-[14px] border border-slate-200 bg-white p-4 shadow-sm">
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

    <!-- Status icon / cancel button -->
    {#if status === 'success'}
      <CheckCircle size={20} class="shrink-0 text-success" />
    {:else if status === 'failed'}
      <XCircle size={20} class="shrink-0 text-error" />
    {:else if status === 'cancelled'}
      <XCircle size={20} class="shrink-0 text-warning" />
    {:else}
      <button
        on:click={handleCancel}
        class="shrink-0 rounded-full p-1.5 text-slate-400 transition-colors hover:bg-error-light hover:text-error active:scale-[0.97] cursor-pointer"
        title="Batalkan Transfer"
      >
        <XCircle size={18} />
      </button>
    {/if}
  </div>

  <!-- Progress bar -->
  <div class="w-full">
    <div class="mb-1.5 flex items-center justify-between text-[11px] font-semibold">
      <span class="text-accent">{$animatedProgress.toFixed(1)}%</span>
      {#if status === 'sending' || status === 'receiving'}
        <span class="text-slate-500">
          {speedMbS.toFixed(1)} MB/s · Sisa {formatTime(estimatedSeconds * 1000)}
        </span>
      {:else if status === 'success'}
        <span class="text-success">{isReceiving ? 'Diterima' : 'Terkirim'}</span>
      {:else if status === 'cancelled'}
        <span class="text-warning">Dibatalkan</span>
      {:else}
        <span class="text-error">Gagal</span>
      {/if}
    </div>

    <!-- Track -->
    <div class="relative h-2 w-full overflow-hidden rounded-full bg-slate-100">
      <div class="h-full rounded-full transition-colors duration-300 {barColor}" style="width: {$animatedProgress}%">
        <!-- Shimmer -->
        {#if status === 'sending' || status === 'receiving'}
          <div
            class="absolute inset-0 -translate-x-full animate-[shimmer_1.5s_infinite] bg-gradient-to-r from-transparent via-white/40 to-transparent"
          ></div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Open folder button on success -->
  {#if status === 'success' && isReceiving}
    <div class="flex justify-end">
      <button
        class="rounded-[8px] border border-slate-200 bg-slate-50 px-3 py-1.5 text-[12px] font-semibold text-slate-600 transition-colors hover:border-accent hover:text-accent cursor-pointer"
        on:click={handleOpenFolder}
      >
        Buka Folder
      </button>
    </div>
  {/if}
</div>

<style>
  @keyframes shimmer {
    100% {
      transform: translateX(200%);
    }
  }
</style>