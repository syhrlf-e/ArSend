<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { trustDevice } from '$lib/stores/trust';
  import { fade, scale } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { Shield } from 'lucide-svelte';

  export let show = false;
  export let publicKey = '';
  export let deviceName = '';

  const dispatch = createEventDispatcher();

  const handleTrustOnce = () => dispatch('resolve', { trusted: true, save: false });

  const handleTrustAlways = async () => {
    await trustDevice(publicKey, deviceName);
    dispatch('resolve', { trusted: true, save: true });
  };

  const handleReject = () => dispatch('resolve', { trusted: false, save: false });

  $: displayKey =
    publicKey.length > 16
      ? `${publicKey.slice(0, 6)}...${publicKey.slice(-4)}`
      : publicKey;
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
          <Shield size={32} strokeWidth={1.5} />
        </div>
        <h3 class="text-[18px] font-semibold text-slate-900">Percayai Perangkat?</h3>
        <p class="mt-1 text-[14px] leading-relaxed text-slate-500">
          <strong class="text-slate-900">{deviceName}</strong>
          ingin terhubung dengan perangkat ini.
        </p>
      </div>

      <div class="mb-5 rounded-xl border border-slate-200 bg-slate-50 p-4">
        <span class="mb-1 block text-[11px] font-semibold uppercase tracking-widest text-slate-400">
          Public Key Fingerprint
        </span>
        <span class="font-mono text-[13px] text-slate-900">{displayKey}</span>
      </div>

      <div class="flex flex-col gap-2">
        <button
          on:click={handleTrustAlways}
          class="w-full rounded-xl bg-accent py-2.5 text-[14px] font-semibold text-white transition-colors hover:bg-accent-hover active:scale-[0.97] cursor-pointer"
        >
          Percaya Selalu
        </button>
        <button
          on:click={handleTrustOnce}
          class="w-full rounded-[12px] bg-accent-light py-2.5 text-[14px] font-semibold text-accent transition-colors hover:bg-accent-mid active:scale-[0.97] cursor-pointer"
        >
          Percaya Sekali
        </button>
        <button
          on:click={handleReject}
          class="w-full rounded-[12px] border border-slate-200 py-2.5 text-[14px] font-semibold text-slate-500 transition-colors hover:border-slate-300 hover:bg-slate-50 active:scale-[0.97] cursor-pointer"
        >
          Tolak
        </button>
      </div>
    </div>
  </div>
{/if}
