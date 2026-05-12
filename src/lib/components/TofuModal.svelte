<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { trustDevice } from '$lib/stores/trust';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  export let show = false;
  export let publicKey = '';
  export let deviceName = '';

  const dispatch = createEventDispatcher();

  const handleTrustOnce = () => {
    dispatch('resolve', { trusted: true, save: false });
  };

  const handleTrustAlways = async () => {
    await trustDevice(publicKey, deviceName);
    dispatch('resolve', { trusted: true, save: true });
  };

  const handleReject = () => {
    dispatch('resolve', { trusted: false, save: false });
  };

  $: displayKey = publicKey.length > 12 
    ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}` 
    : publicKey;
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
      <h3 class="text-[18px] font-semibold text-text-primary mb-2">Percayai Perangkat?</h3>
      <p class="text-[14px] text-text-secondary mb-4 leading-relaxed">
        <strong>{deviceName}</strong> ingin terhubung dengan perangkat ini.
      </p>
      
      <div class="bg-surface-2 p-3 rounded-[12px] mb-6 border border-border">
        <span class="text-[11px] font-semibold text-text-tertiary uppercase tracking-wider block mb-1">Public Key Fingerprint</span>
        <span class="text-[12px] font-mono text-text-primary">{displayKey}</span>
      </div>

      <div class="flex flex-col gap-2">
        <button on:click={handleTrustAlways} class="w-full py-2.5 text-[14px] font-semibold bg-accent text-white rounded-[12px] hover:bg-accent-hover transition-colors active:scale-[0.97] cursor-pointer">
          Percaya Selalu
        </button>
        <button on:click={handleTrustOnce} class="w-full py-2.5 text-[14px] font-semibold text-accent bg-accent-light rounded-[12px] hover:bg-accent-mid transition-colors active:scale-[0.97] cursor-pointer">
          Percaya Sekali
        </button>
        <button on:click={handleReject} class="w-full py-2.5 text-[14px] font-semibold text-text-secondary hover:bg-surface-2 rounded-[12px] transition-colors active:scale-[0.97] cursor-pointer">
          Tolak
        </button>
      </div>
    </div>
  </div>
{/if}
