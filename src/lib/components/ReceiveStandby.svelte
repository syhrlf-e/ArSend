<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  
  export let connectedDeviceName: string = '';
  export let isReceiveTimeout: boolean = false;
  
  const dispatch = createEventDispatcher();

  // Acak urutan array saat mount agar tidak selalu mulai dari kata yang sama
  const shuffleArray = (array: string[]) => {
    for (let i = array.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [array[i], array[j]] = [array[j], array[i]];
    }
    return array;
  };

  $: basePhrases = [
    "Menunggu transmisi data...",
    `Mendengarkan sinyal dari ${connectedDeviceName}...`,
    "Jalur aman telah terbuka...",
    "Siap menerima gelombang file...",
    "Mengkalibrasi kecepatan transfer...",
    "Memeriksa integritas jaringan...",
    "Menjaga saluran tetap terbuka...",
    "Mode siaga diaktifkan...",
    `Memindai paket dari ${connectedDeviceName}...`,
    "Sistem penerimaan online...",
    "Menyesuaikan bandwidth...",
    "Tidak ada anomali terdeteksi...",
    "Menunggu otorisasi pengirim...",
    "Protokol hand-shake siap...",
    "Menganalisis spektrum koneksi...",
    "Menunggu instruksi selanjutnya...",
    `Terkunci pada ${connectedDeviceName}...`,
    "Menyiapkan ruang penyimpanan...",
    "Menstabilkan frekuensi radio...",
    "Memantau lalu lintas data...",
    "Enkripsi ujung-ke-ujung aktif...",
    "Menunggu byte pertama...",
    "Koneksi dalam kondisi optimal...",
    "Menyaring gangguan eksternal...",
    "Saluran komunikasi jernih...",
    "Mengaktifkan mode penerima cepat...",
    "Menunggu payload data...",
    `Mengarahkan antena ke ${connectedDeviceName}...`,
    "Menyelaraskan clock jaringan...",
    "Berada dalam mode mendengarkan pasif...",
    "Siaga penuh..."
  ];

  let phrases: string[] = [];
  let phraseIndex = 0;
  let intervalId: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    phrases = shuffleArray([...basePhrases]);
    intervalId = setInterval(() => {
      phraseIndex = (phraseIndex + 1) % phrases.length;
    }, 3500); // Sedikit dilambatkan jadi 3.5 detik agar sempat dibaca
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });
</script>

<div class="flex flex-1 w-full flex-col items-center justify-center px-4 py-8 mt-4 sm:mt-0 sm:rounded-2xl sm:border sm:border-slate-200 sm:bg-slate-50">
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
        class="w-full cursor-pointer rounded-xl bg-accent py-3.5 text-[15px] font-semibold text-white transition-colors hover:bg-accent-hover active:scale-[0.98]"
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
    <div class="relative mb-8 flex h-24 w-24 items-center justify-center rounded-full bg-accent text-white ring-8 ring-accent/20 animate-pulse">
      <svg xmlns="http://www.w3.org/2000/svg" width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"></path>
        <path d="M12 12v9"></path>
        <path d="m8 17 4 4 4-4"></path>
      </svg>
    </div>

    <!-- Dynamic Text Area -->
    <div class="relative flex h-8 w-full items-center justify-center">
      {#if phrases.length > 0}
        {#each phrases as phrase, i}
          <p class="absolute px-6 text-center text-[16px] font-medium text-slate-600 transition-opacity duration-700 ease-in-out"
             style="opacity: {phraseIndex === i ? 1 : 0}">
            {phrase}
          </p>
        {/each}
      {/if}
    </div>
  {/if}
</div>
