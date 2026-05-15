<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { formatBytes } from '$lib/utils/format';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { Monitor, Smartphone, Folder } from 'lucide-svelte';
  import FileIcon from '$lib/components/FileIcon.svelte';

  export let show = false;
  export let deviceName = '';
  export let deviceType = 'unknown';
  export let fileName = '';
  export let fileSize = 0;
  export let fileCount = 1;

  const dispatch = createEventDispatcher();

  const handleAccept = () => dispatch('resolve', { accept: true });
  const handleReject = () => dispatch('resolve', { accept: false });

  const getDeviceIcon = (type: string) => {
    switch (type) {
      case 'desktop':
      case 'laptop':
        return Monitor;
      case 'mobile':
        return Smartphone;
      default:
        return Folder;
    }
  };

  $: DeviceIcon = getDeviceIcon(deviceType);
</script>

{#if show}
  <div
    class="fixed inset-0 z-50 flex items-end justify-center p-4 sm:items-center"
    style="background-color: rgba(15,23,42,0.45);"
    transition:fade={{ duration: 200, easing: cubicOut }}
  >
    <div
      class="w-full max-w-sm rounded-[20px] border border-slate-200 bg-white p-6 shadow-2xl"
      transition:scale={{ start: 0.95, duration: 220, easing: cubicOut }}
    >
      <div class="mb-5 flex flex-col items-center text-center">
        <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-accent-light text-accent">
          <DeviceIcon size={32} strokeWidth={1.5} />
        </div>
        <h3 class="text-[18px] font-semibold text-slate-900">Terima File?</h3>
        <p class="mt-1 text-[14px] leading-relaxed text-slate-500">
          <strong class="text-slate-900">{deviceName}</strong>
          ingin mengirim {fileCount > 1 ? `${fileCount} file` : 'sebuah file'}.
        </p>
      </div>

      <div class="mb-5 flex items-center gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4">
        <FileIcon filename={fileName} />
        <div class="flex flex-col overflow-hidden text-left">
          <span class="truncate text-[13px] font-semibold text-slate-900">
            {fileName}{fileCount > 1 ? ` dan ${fileCount - 1} lainnya` : ''}
          </span>
          <span class="mt-0.5 text-[12px] text-slate-500">
            Total ukuran: {formatBytes(fileSize)}
          </span>
        </div>
      </div>

      <div class="flex gap-3">
        <button
          on:click={handleReject}
          class="rounded-xl border border-slate-200 px-5 py-2.5 text-[14px] font-semibold text-slate-500 transition-colors hover:border-slate-300 hover:bg-slate-50 active:scale-[0.97] cursor-pointer"
        >
          Tolak
        </button>
        <button
          on:click={handleAccept}
          class="flex-1 rounded-xl bg-accent py-2.5 text-[14px] font-semibold text-white shadow-sm transition-colors hover:bg-accent-hover active:scale-[0.97] cursor-pointer"
        >
          Terima
        </button>
      </div>
    </div>
  </div>
{/if}
