<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { trustDevice } from '$lib/stores/trust';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { Shield, Fingerprint, Info } from 'lucide-svelte';

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

  // Format fingerprint as AAAA-BBBB-CCCC-DDDD
  $: formattedFingerprint = publicKey
    .match(/.{1,4}/g)
    ?.join('-')
    .toUpperCase() || publicKey;
</script>

{#if show}
  <div
    class="fixed inset-0 z-[100] flex items-end sm:items-center sm:justify-center sm:p-4"
    style="background-color: rgba(15,23,42,0.45);"
    transition:fade={{ duration: 200, easing: cubicOut }}
  >
    <div
      class="w-full bg-white rounded-t-[28px] sm:rounded-[20px] sm:max-w-md border-t border-slate-200 sm:border pb-safe shadow-2xl"
      in:fly={{ y: 400, duration: 300, easing: cubicOut }}
      out:fly={{ y: 400, duration: 250, easing: cubicOut }}
    >

      <div class="flex w-full justify-center pt-4 pb-1 sm:hidden">
        <div class="h-1.5 w-12 rounded-full bg-slate-200"></div>
      </div>

      <div class="p-6 pt-4 sm:pt-8 pb-12 sm:pb-8">
        <div class="mb-6 flex flex-col items-center text-center">
          <div class="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-accent/10 text-accent">
            <Shield size={32} strokeWidth={1.5} />
          </div>
          <h3 class="text-[20px] sm:text-[22px] font-bold text-slate-900">Verifikasi Perangkat</h3>
          <p class="mt-2 text-[15px] sm:text-[14px] leading-relaxed text-slate-500">
            Perangkat <strong class="text-slate-900 font-semibold">{deviceName}</strong> mencoba terhubung. 
            Pastikan kode di bawah ini cocok dengan yang tampil di perangkat tersebut.
          </p>
        </div>

        <!-- Fingerprint Box -->
        <div class="mb-6 rounded-2xl bg-slate-50 border border-slate-200 p-4">
          <div class="flex items-center gap-2 mb-2 text-slate-400">
            <Fingerprint size={14} />
            <span class="text-[11px] font-bold uppercase tracking-wider">Fingerprint Keamanan</span>
          </div>
          <div class="font-mono text-[16px] sm:text-[18px] font-bold tracking-widest text-slate-800 break-all text-center">
            {formattedFingerprint}
          </div>
        </div>

        <!-- Warning/Info Box -->
        <div class="mb-8 flex gap-3 rounded-xl bg-amber-50 p-4 border border-amber-100">
          <Info size={18} class="text-amber-600 shrink-0 mt-0.5" />
          <p class="text-[13px] text-amber-800 leading-snug">
            <strong>Percaya Selalu</strong> akan menyimpan perangkat ini sehingga Anda tidak perlu memverifikasi lagi di masa depan.
          </p>
        </div>

        <div class="flex flex-col gap-3">
          <button
            on:click={handleTrustAlways}
            class="w-full rounded-xl bg-accent py-4 sm:py-3 text-[15px] font-bold text-white shadow-lg shadow-accent/20 transition-all hover:bg-accent-hover hover:scale-[1.01] active:scale-[0.98] cursor-pointer"
          >
            Percaya Selalu
          </button>
          <div class="grid grid-cols-2 gap-3">
            <button
              on:click={handleTrustOnce}
              class="rounded-xl bg-slate-100 py-3.5 sm:py-3 text-[14px] font-semibold text-slate-700 transition-all hover:bg-slate-200 active:scale-[0.97] cursor-pointer"
            >
              Percaya Sekali
            </button>
            <button
              on:click={handleReject}
              class="rounded-xl border-2 border-slate-100 py-3.5 sm:py-3 text-[14px] font-semibold text-slate-500 transition-all hover:border-slate-200 hover:bg-slate-50 active:scale-[0.97] cursor-pointer"
            >
              Tolak
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
