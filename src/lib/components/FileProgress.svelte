<script lang="ts">
  import { formatBytes, formatTime } from '$lib/utils/format';
  import { XCircle, CheckCircle } from 'lucide-svelte';
  import { tweened } from 'svelte/motion';
  import { cubicOut } from 'svelte/easing';
  import FileIcon from '$lib/components/FileIcon.svelte';

  export let filename = '';
  export let progress = 0; // 0 to 100
  export let speedMbS = 0;
  export let sentBytes = 0;
  export let totalBytes = 0;
  export let isReceiving = false;
  export let status: 'sending' | 'receiving' | 'success' | 'failed' = 'sending';

  $: remainingBytes = totalBytes - sentBytes;
  $: estimatedSeconds = speedMbS > 0 ? (remainingBytes / (speedMbS * 1024 * 1024)) : 0;

  // Smooth progress animation
  const animatedProgress = tweened(0, {
    duration: 300,
    easing: cubicOut
  });

  $: animatedProgress.set(progress);
</script>

<div class="bg-surface p-4 rounded-[14px] border border-border flex flex-col gap-3">
  <!-- Header -->
  <div class="flex justify-between items-start gap-4">
    <div class="flex items-center gap-3 overflow-hidden">
      <FileIcon {filename} />
      
      <div class="flex flex-col overflow-hidden">
        <span class="text-[14px] font-semibold text-text-primary truncate" title={filename}>{filename}</span>
        <span class="text-[12px] text-text-secondary">
          {formatBytes(sentBytes)} / {formatBytes(totalBytes)}
        </span>
      </div>
    </div>

    <!-- Status Icon or Cancel Button -->
    {#if status === 'success'}
      <CheckCircle size={20} class="text-success shrink-0" />
    {:else if status === 'failed'}
      <XCircle size={20} class="text-error shrink-0" />
    {:else}
      <button class="p-1.5 text-text-tertiary hover:text-error hover:bg-error-light rounded-full transition-colors active:scale-[0.97] cursor-pointer" title="Batalkan">
        <XCircle size={18} />
      </button>
    {/if}
  </div>

  <!-- Progress Bar -->
  <div class="w-full relative">
    <div class="flex justify-between text-[11px] font-semibold mb-1">
      <span class="text-accent">{$animatedProgress.toFixed(1)}%</span>
      {#if status === 'sending' || status === 'receiving'}
        <span class="text-text-secondary">{speedMbS.toFixed(1)} MB/s • Sisa {formatTime(estimatedSeconds * 1000)}</span>
      {:else if status === 'success'}
        <span class="text-success">{isReceiving ? 'Diterima' : 'Terkirim'}</span>
      {:else}
        <span class="text-error">Gagal</span>
      {/if}
    </div>
    
    <div class="h-2 w-full bg-surface-2 rounded-full overflow-hidden relative">
      <div 
        class="h-full rounded-full transition-colors duration-300 {status === 'success' ? 'bg-success' : status === 'failed' ? 'bg-error' : 'bg-accent'}"
        style="width: {$animatedProgress}%"
      >
        <!-- Shimmer Effect when active -->
        {#if status === 'sending' || status === 'receiving'}
          <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent -translate-x-full animate-[shimmer_1.5s_infinite]"></div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  @keyframes shimmer {
    100% { transform: translateX(100%); }
  }
</style>
