<script lang="ts">
  import { UploadCloud, DownloadCloud, Clock, User } from 'lucide-svelte';

  export let activeTab: 'Kirim' | 'Terima' | 'Riwayat' | 'Profile' = 'Kirim';

  const tabs = [
    { id: 'Kirim', icon: UploadCloud, label: 'Kirim' },
    { id: 'Terima', icon: DownloadCloud, label: 'Terima' },
    { id: 'Riwayat', icon: Clock, label: 'Riwayat' },
    { id: 'Profile', icon: User, label: 'Profile' }
  ];
</script>

<div class="fixed bottom-16 left-0 right-0 z-50 flex justify-center px-4">
  <div class="flex w-full max-w-md items-center gap-1 rounded-full bg-white/90 px-2 py-2 backdrop-blur-xl border border-slate-200/50">
    {#each tabs as tab}
      <button
        class="relative flex flex-1 min-w-0 cursor-pointer flex-col items-center justify-center rounded-[18px] transition-all duration-300 active:scale-95 pt-3 pb-3
          {activeTab === tab.id ? 'text-accent' : 'text-slate-400 hover:bg-slate-100 hover:text-slate-600'}"
        on:click={() => {
          activeTab = tab.id as any;
        }}
        aria-label={tab.label}
      >
        {#if activeTab === tab.id}
          <div class="absolute inset-0 rounded-[18px] bg-accent-light opacity-50"></div>
        {/if}
        <div class="relative z-10 flex flex-col items-center gap-1.5">
          <svelte:component this={tab.icon} size={22} strokeWidth={activeTab === tab.id ? 2.5 : 2} />
          <span class="text-[10px] tracking-wide {activeTab === tab.id ? 'font-bold opacity-100' : 'font-medium opacity-80'} transition-all duration-300">
            {tab.label}
          </span>
        </div>
      </button>
    {/each}
  </div>
</div>
