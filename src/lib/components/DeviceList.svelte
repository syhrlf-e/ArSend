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

<div class="w-full max-w-md space-y-3">
  <h2 class="mb-3 text-[15px] font-semibold text-slate-900">Perangkat Ditemukan</h2>

  {#if $discoveredDevices.length === 0}

    <div class="rounded-[14px] border border-slate-200 bg-white p-6 text-center">
      <p class="text-[14px] text-slate-500">Mencari perangkat ArSend di jaringan...</p>
      <div class="mt-4 flex justify-center">
        <span class="relative flex h-3 w-3">
          <span
            class="absolute inline-flex h-full w-full animate-ping rounded-full bg-accent opacity-75"
          ></span>
          <span class="relative inline-flex h-3 w-3 rounded-full bg-accent"></span>
        </span>
      </div>
    </div>
  {:else}
    <div class="space-y-2">
      {#each $discoveredDevices as device (device.payload.public_key)}
        {@const Icon = getDeviceIcon(device.payload.device_type)}
        <button
          on:click={() => dispatch('connect', device)}
          class="group flex w-full cursor-pointer items-center rounded-[14px] border border-slate-200 bg-white p-4 text-left transition-all duration-200 hover:border-accent-mid hover:bg-accent-light active:scale-[0.97]"
        >
          <div class="rounded-full bg-accent-light p-3 text-accent">
            <Icon size={20} strokeWidth={1.5} />
          </div>
          <div class="ml-4 flex-1 overflow-hidden">
            <h3
              class="text-[15px] font-semibold text-slate-900 transition-colors group-hover:text-accent"
            >
              {device.payload.name}
            </h3>
            <p class="mt-0.5 font-mono text-[12px] text-slate-500">{device.ip}</p>
          </div>

          <svg
            class="ml-2 h-4 w-4 shrink-0 text-slate-300 transition-colors group-hover:text-accent"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 18l6-6-6-6" />
          </svg>
        </button>
      {/each}
    </div>
  {/if}
</div>
