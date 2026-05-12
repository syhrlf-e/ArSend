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

  const handleAccept = () => {
    dispatch('resolve', { accept: true });
  };

  const handleReject = () => {
    dispatch('resolve', { accept: false });
  };

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
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div 
    class="fixed inset-0 bg-slate-900/40 flex items-center justify-center z-50 p-4"
    transition:fade={{ duration: 200, easing: cubicOut }}
  >
    <div 
      class="bg-surface border border-border-main rounded-[16px] p-6 shadow-xl max-w-sm w-full"
      transition:scale={{ start: 0.95, duration: 200, easing: cubicOut }}
    >
      <div class="flex flex-col items-center text-center mb-6">
        <div class="w-16 h-16 bg-accent-light rounded-full flex items-center justify-center text-accent mb-4">
          <DeviceIcon size={32} strokeWidth={1.5} />
        </div>
        
        <h3 class="text-[18px] font-semibold text-text-primary mb-1">Terima File?</h3>
        <p class="text-[14px] text-text-secondary leading-relaxed">
          <strong>{deviceName}</strong> ingin mengirim {fileCount > 1 ? `${fileCount} file` : 'sebuah file'}.
        </p>
      </div>
      
      <div class="bg-surface-2 p-4 rounded-[12px] mb-6 border border-border flex items-center gap-3">
        <FileIcon filename={fileName} />
        <div class="flex-col flex overflow-hidden text-left">
          <span class="text-[13px] font-semibold text-text-primary truncate">{fileName}{fileCount > 1 ? ` dan ${fileCount - 1} lainnya` : ''}</span>
          <span class="text-[12px] text-text-secondary mt-0.5">Total ukuran: {formatBytes(fileSize)}</span>
        </div>
      </div>

      <div class="flex justify-end gap-3">
        <button on:click={handleReject} class="flex-1 py-2.5 text-[14px] font-semibold text-text-secondary hover:bg-surface-2 rounded-[12px] transition-colors active:scale-[0.97] cursor-pointer border border-transparent hover:border-border">
          Tolak
        </button>
        <button on:click={handleAccept} class="flex-1 py-2.5 text-[14px] font-semibold bg-accent text-white rounded-[12px] hover:bg-accent-hover transition-colors active:scale-[0.97] cursor-pointer shadow-sm">
          Terima
        </button>
      </div>
    </div>
  </div>
{/if}
