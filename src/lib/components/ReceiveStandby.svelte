<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  
  export let connectedDeviceName: string = '';
  export let isReceiveTimeout: boolean = false;
  
  const dispatch = createEventDispatcher();

  $: phrases = [
    "Menunggu kiriman file...",
    `Mendengarkan ${connectedDeviceName}...`,
    "Jalur aman telah terbuka...",
    "Siap menerima transmisi..."
  ];

  let phraseIndex = 0;
  let intervalId: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    intervalId = setInterval(() => {
      phraseIndex = (phraseIndex + 1) % phrases.length;
    }, 3000);
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });
</script>

<div class="flex w-full flex-col items-center justify-center rounded-2xl border border-slate-200 bg-slate-50 px-4 py-8 shadow-sm">
  {#if isReceiveTimeout}
    <div class="mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-slate-200 text-slate-400">
      <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
      </svg>
    </div>
    <h3 class="mb-2 text-[18px] font-bold text-slate-900">Waktu tunggu habis</h3>
    <p class="mb-8 text-center text-[14px] text-slate-500">Tidak ada aktivitas transfer selama 5 menit.</p>
    
    <div class="flex w-full max-w-xs flex-col gap-3">
      <button
        on:click={() => dispatch('retry')}
        class="w-full cursor-pointer rounded-xl bg-accent py-3.5 text-[15px] font-semibold text-white shadow-sm transition-colors hover:bg-accent-hover active:scale-[0.98]"
      >
        Mulai Lagi
      </button>
      <button
        on:click={() => dispatch('cancel')}
        class="w-full cursor-pointer rounded-xl border border-slate-200 bg-white py-3.5 text-[15px] font-semibold text-slate-600 transition-colors hover:bg-slate-50 hover:text-slate-900 active:scale-[0.98]"
      >
        Kembali ke Beranda
      </button>
    </div>
  {:else}
    <!-- Central Animation -->
    <div class="relative mb-8 flex h-24 w-24 items-center justify-center rounded-full bg-accent text-white ring-8 ring-accent/20 animate-pulse shadow-lg">
      <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"></path>
        <path d="M12 12v9"></path>
        <path d="m8 17 4 4 4-4"></path>
      </svg>
    </div>

    <!-- Dynamic Text Area -->
    <div class="relative flex h-8 w-full items-center justify-center">
      {#each phrases as phrase, i}
        <p class="absolute text-center text-[16px] font-semibold text-slate-900 transition-opacity duration-700 ease-in-out"
           style="opacity: {phraseIndex === i ? 1 : 0}">
          {phrase}
        </p>
      {/each}
    </div>

    <!-- Identity Card -->
    <div class="mt-8 w-full max-w-xs rounded-xl border border-slate-200 bg-white px-4 py-3.5 text-center shadow-sm">
      <p class="text-[11px] font-bold text-slate-400 uppercase tracking-widest">Terhubung Dengan</p>
      <p class="mt-1 text-[15px] font-bold text-slate-900">{connectedDeviceName}</p>
    </div>

    <!-- Cancel Button -->
    <button
      on:click={() => dispatch('cancel')}
      class="mt-4 w-full max-w-xs cursor-pointer rounded-xl border border-slate-200 bg-white py-3.5 text-[15px] font-semibold text-slate-600 transition-colors hover:border-error hover:bg-error-light hover:text-error active:scale-[0.98]"
    >
      Batalkan
    </button>
  {/if}
</div>
