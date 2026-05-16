<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { trustDevice } from '$lib/stores/trust';
  import { fade, fly } from 'svelte/transition';
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
    class="fixed inset-0 z-60 flex items-end sm:items-center sm:justify-center sm:p-4"
    style="background-color: rgba(15,23,42,0.45);"
    transition:fade={{ duration: 200, easing: cubicOut }}
  >
    <div
      class="w-full bg-white rounded-t-[28px] sm:rounded-[20px] sm:max-w-sm border-t border-slate-200 sm:border pb-safe"
      in:fly={{ y: 400, duration: 300, easing: cubicOut }}
      out:fly={{ y: 400, duration: 250, easing: cubicOut }}
    >

      <div class="flex w-full justify-center pt-4 pb-1 sm:hidden">
        <div class="h-1.5 w-12 rounded-full bg-slate-200"></div>
      </div>

      <div class="p-6 pt-4 sm:pt-6 pb-12 sm:pb-6">
        <div class="mb-8 flex flex-col items-center text-center">
          <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-accent-light text-accent">
            <Shield size={32} strokeWidth={1.5} />
          </div>
          <h3 class="text-[20px] sm:text-[18px] font-semibold text-slate-900">Percayai Perangkat?</h3>
          <p class="mt-1 text-[15px] sm:text-[14px] leading-relaxed text-slate-500">
            <strong class="text-slate-900">{deviceName}</strong>
            ingin terhubung dengan perangkat ini.
          </p>
        </div>

        <div class="flex flex-col gap-3 sm:gap-2">
          <button
            on:click={handleTrustAlways}
            class="w-full rounded-xl bg-accent py-3.5 sm:py-2.5 text-[15px] sm:text-[14px] font-semibold text-white transition-colors hover:bg-accent-hover active:scale-[0.97] cursor-pointer"
          >
            Percaya Selalu
          </button>
          <button
            on:click={handleTrustOnce}
            class="w-full rounded-xl bg-accent-light py-3.5 sm:py-2.5 text-[15px] sm:text-[14px] font-semibold text-accent transition-colors hover:bg-accent-mid active:scale-[0.97] cursor-pointer"
          >
            Percaya Sekali
          </button>
          <button
            on:click={handleReject}
            class="w-full rounded-xl border border-slate-200 py-3.5 sm:py-2.5 text-[15px] sm:text-[14px] font-semibold text-slate-500 transition-colors hover:border-slate-300 hover:bg-slate-50 active:scale-[0.97] cursor-pointer"
          >
            Tolak
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
