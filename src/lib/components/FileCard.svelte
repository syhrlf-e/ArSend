<script lang="ts">
  import { formatBytes } from '$lib/utils/format';
  import { ArrowDownLeft, ArrowUpRight, CheckCircle, XCircle, Trash2 } from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';
  import { createEventDispatcher } from 'svelte';

  export let filename: string;
  export let size: number;
  export let type: 'sent' | 'received';
  export let status: 'success' | 'failed' | 'cancelled';
  export let timestamp: number;

  const dispatch = createEventDispatcher();

  $: date = new Date(timestamp);
  $: timeString = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  $: dateString = date.toLocaleDateString([], {
    day: 'numeric',
    month: 'short',
    year: 'numeric'
  });
</script>

<div
  class="group flex items-center justify-between gap-3 rounded-[14px] border border-slate-200 bg-white p-4 transition-all hover:border-slate-300"
>
  <div class="relative shrink-0">
    <FileIcon {filename} />
    <div
      class="absolute -bottom-1 -right-1 rounded-full border border-slate-200 bg-white p-0.5"
    >
      {#if type === 'received'}
        <ArrowDownLeft size={11} class="text-success" strokeWidth={2.5} />
      {:else}
        <ArrowUpRight size={11} class="text-accent" strokeWidth={2.5} />
      {/if}
    </div>
  </div>

  <!-- Info -->
  <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <span class="truncate text-[14px] font-semibold text-slate-900" title={filename}>
      {filename}
    </span>
    <div class="mt-0.5 flex items-center gap-2 text-[12px] text-slate-500">
      <span>{formatBytes(size)}</span>
      <span>·</span>
      <span>{dateString}, {timeString}</span>
    </div>
  </div>

  <!-- Status badge & Delete -->
  <div class="flex shrink-0 items-center gap-2">
    {#if status === 'success'}
      <span
        class="inline-flex items-center gap-1 rounded-md border border-success/20 bg-success-light px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-success"
      >
        Selesai
      </span>
    {:else if status === 'failed'}
      <span
        class="inline-flex items-center gap-1 rounded-md border border-error/20 bg-error-light px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-error"
      >
        Gagal
      </span>
    {:else}
      <span
        class="inline-flex items-center gap-1 rounded-md border border-warning/20 bg-warning-light px-2 py-1 text-[11px] font-semibold uppercase tracking-wide text-warning"
      >
        Batal
      </span>
    {/if}
    <button
      on:click={() => dispatch('delete')}
      class="rounded-full p-2 text-slate-400 opacity-0 transition-all hover:bg-error-light hover:text-error group-hover:opacity-100 sm:opacity-100 cursor-pointer active:scale-95"
      title="Hapus riwayat"
    >
      <Trash2 size={16} />
    </button>
  </div>
</div>
