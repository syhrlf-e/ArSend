<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { discoveredDevices } from '../stores/connection';
  import { Monitor, Smartphone, Folder } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

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
</script>

<div class="w-full max-w-md mt-6 space-y-3">
  <h2 class="text-[18px] font-semibold text-text-primary mb-4">Perangkat Ditemukan</h2>
  
  {#if $discoveredDevices.length === 0}
    <div class="p-6 text-center text-text-secondary bg-surface rounded-[14px] border border-border">
      <p class="text-[14px]">Mencari perangkat ArSend di jaringan...</p>
      <div class="mt-4 flex justify-center">
        <span class="relative flex h-3 w-3">
          <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-accent opacity-75"></span>
          <span class="relative inline-flex rounded-full h-3 w-3 bg-accent"></span>
        </span>
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      {#each $discoveredDevices as device (device.payload.public_key)}
        {@const Icon = getDeviceIcon(device.payload.device_type)}
        <button 
          on:click={() => dispatch('connect', device.ip)}
          class="w-full flex items-center p-4 bg-surface hover:bg-accent-light transition-colors duration-200 rounded-[14px] border border-border hover:border-accent-mid group text-left cursor-pointer active:scale-[0.97]"
        >
          <div class="p-3 bg-accent-light text-accent rounded-full">
            <Icon size={20} strokeWidth={1.5} />
          </div>
          <div class="ml-4 flex-1">
            <h3 class="text-[15px] font-semibold text-text-primary group-hover:text-accent transition-colors">{device.payload.name}</h3>
            <p class="text-[12px] font-mono text-text-secondary mt-0.5">{device.ip}</p>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>
