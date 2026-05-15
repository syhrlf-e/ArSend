<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { WifiOff, HardDrive, Zap, ArrowRight, Check } from 'lucide-svelte';
  import { fade, fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';

  const dispatch = createEventDispatcher();

  let currentSlide = 0;

  const slides = [
    {
      title: 'Tanpa Internet',
      description: 'Transfer file tanpa kuota, tanpa cloud. Sepenuhnya lokal dan aman.',
      icon: WifiOff
    },
    {
      title: 'Tanpa Batas Ukuran',
      description: 'Kirim file sebesar apapun, secepat jaringan WiFi atau Hotspot-mu.',
      icon: HardDrive
    },
    {
      title: 'Simpel & Cepat',
      description:
        'Terhubung otomatis di jaringan yang sama, atau scan QR untuk koneksi spesifik.',
      icon: Zap
    }
  ];

  const nextSlide = () => {
    if (currentSlide < slides.length - 1) {
      currentSlide += 1;
    } else {
      dispatch('complete');
    }
  };
</script>

<div
  class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-white p-6 text-center"
  out:fade={{ duration: 300 }}
>

  <div class="relative flex w-full max-w-sm flex-1 flex-col items-center justify-center">
    {#each slides as slide, i (i)}
      {#if currentSlide === i}
        <div
          class="absolute inset-0 flex flex-col items-center justify-center"
          in:fly={{ x: 50, duration: 400, delay: 100, easing: cubicOut }}
          out:fly={{ x: -50, duration: 400, easing: cubicOut }}
        >
          <div
            class="mb-8 flex h-24 w-24 items-center justify-center rounded-full bg-accent-light text-accent shadow-sm"
          >
            <svelte:component this={slide.icon} size={48} strokeWidth={1.5} />
          </div>

          <h2 class="mb-3 text-[28px] font-bold leading-tight tracking-tight text-slate-900">
            {slide.title}
          </h2>
          <p class="max-w-70 text-[15px] leading-relaxed text-slate-500">
            {slide.description}
          </p>
        </div>
      {/if}
    {/each}
  </div>

  <div class="flex w-full max-w-sm flex-col items-center gap-8 pb-10">
    <div class="flex items-center gap-2">
      {#each slides as _, i}
        <div
          class="h-2 rounded-full transition-all duration-300 {i === currentSlide
            ? 'w-6 bg-accent'
            : 'w-2 bg-slate-200'}"
        ></div>
      {/each}
    </div>

    <button
      on:click={nextSlide}
      class="flex w-full items-center justify-center gap-2 rounded-[14px] bg-accent py-4 text-[16px] font-semibold text-white shadow-md shadow-accent/20 transition-colors hover:bg-accent-hover active:scale-[0.98] cursor-pointer"
    >
      {#if currentSlide === slides.length - 1}
        <span>Mulai Sekarang</span>
        <Check size={20} strokeWidth={2} />
      {:else}
        <span>Selanjutnya</span>
        <ArrowRight size={20} strokeWidth={2} />
      {/if}
    </button>
  </div>
</div>
