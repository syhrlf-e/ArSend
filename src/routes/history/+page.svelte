<script lang="ts">
  import {
    History,
    ArrowUpRight,
    ArrowDownLeft,
    Clock,
    Trash2,
    CheckCircle,
    XCircle
  } from 'lucide-svelte';
  import { formatBytes } from '$lib/utils/format';
  import FileIcon from '$lib/components/FileIcon.svelte';

  let historyItems = [
    {
      id: '1',
      type: 'send',
      filename: 'Project_Proposal_V2.pdf',
      size: 2450000,
      deviceName: "Rull's Phone",
      timestamp: new Date(Date.now() - 1000 * 60 * 5),
      status: 'success'
    },
    {
      id: '2',
      type: 'receive',
      filename: 'IMG_8942.jpg',
      size: 4800000,
      deviceName: "Budi's Laptop",
      timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
      status: 'success'
    },
    {
      id: '3',
      type: 'send',
      filename: 'video_render_final.mp4',
      size: 156000000,
      deviceName: "Rull's Phone",
      timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24),
      status: 'failed'
    }
  ];

  const clearHistory = () => {
    historyItems = [];
  };

  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('id-ID', {
      day: 'numeric',
      month: 'short',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  };
</script>

<div class="flex flex-col items-center min-h-screen p-6">
  <div class="w-full max-w-4xl bg-white border border-slate-200 rounded-[16px] p-6 mt-8">

    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-2">
        <History size={24} class="text-accent" />
        <h1 class="text-[20px] font-bold text-slate-900">Riwayat Transfer</h1>
      </div>
      {#if historyItems.length > 0}
        <button
          on:click={clearHistory}
          class="flex items-center gap-1.5 px-3 py-1.5 text-[13px] font-semibold text-error hover:bg-error-light rounded-lg transition-colors active:scale-[0.97]"
        >
          <Trash2 size={16} />
          Bersihkan
        </button>
      {/if}
    </div>

    {#if historyItems.length === 0}
      <div class="flex flex-col items-center justify-center py-16 text-center">
        <div class="w-16 h-16 bg-slate-50 rounded-full flex items-center justify-center text-slate-400 mb-4">
          <History size={32} />
        </div>
        <p class="text-[15px] font-semibold text-slate-900 mb-1">
          Belum ada riwayat
        </p>
        <p class="text-[13px] text-slate-500">
          File yang Anda kirim atau terima akan muncul di sini.
        </p>
      </div>
    {:else}
      <div class="flex flex-col gap-3">
        {#each historyItems as item}
          <div class="bg-slate-50 p-4 rounded-xl border border-slate-200 flex items-center justify-between group hover:border-slate-300 transition-colors relative overflow-hidden">

            <div class="flex items-center gap-4 overflow-hidden z-10">
              <FileIcon filename={item.filename} />
              <div class="flex flex-col overflow-hidden">
                <span
                  class="text-[15px] font-semibold text-slate-900 truncate"
                  title={item.filename}
                >
                  {item.filename}
                </span>
                <div class="flex items-center gap-2 mt-0.5">
                  {#if item.type === 'send'}
                    <ArrowUpRight size={14} class="text-accent" />
                  {:else}
                    <ArrowDownLeft size={14} class="text-success" />
                  {/if}
                  <span class="text-[12px] text-slate-500">
                    {formatBytes(item.size)}
                  </span>
                  <span class="text-slate-400 text-[10px]">•</span>
                  <span class="text-[12px] font-medium text-slate-500 truncate">
                    {item.type === 'send' ? 'ke' : 'dari'} {item.deviceName}
                  </span>
                </div>
              </div>
            </div>

            <div class="flex flex-col items-end shrink-0 ml-4 z-10">
              {#if item.status === 'success'}
                <div class="flex items-center gap-1.5 text-success mb-1">
                  <CheckCircle size={14} />
                  <span class="text-[12px] font-semibold uppercase tracking-wider">
                    Selesai
                  </span>
                </div>
              {:else}
                <div class="flex items-center gap-1.5 text-error mb-1">
                  <XCircle size={14} />
                  <span class="text-[12px] font-semibold uppercase tracking-wider">
                    Gagal
                  </span>
                </div>
              {/if}
              <div class="flex items-center gap-1 text-slate-400">
                <Clock size={12} />
                <span class="text-[11px]">{formatDate(item.timestamp)}</span>
              </div>
            </div>

          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
