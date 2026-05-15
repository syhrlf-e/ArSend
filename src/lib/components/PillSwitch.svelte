<script lang="ts">
  export let activeTab = 'Transfer';
</script>

<div class="flex items-center justify-center p-1">
  <div
    class="flex w-full max-w-sm rounded-full border border-slate-200 bg-slate-100 p-1"
  >
    <button
      on:click={() => (activeTab = 'Transfer')}
      class="flex-1 cursor-pointer rounded-full py-2 text-[14px] font-semibold transition-all duration-200
        {activeTab === 'Transfer'
          ? 'bg-white text-slate-900 shadow-sm'
          : 'text-slate-500 hover:text-slate-700'}"
    >
      Transfer
    </button>
    <button
      on:click={() => (activeTab = 'Riwayat')}
      class="flex-1 cursor-pointer rounded-full py-2 text-[14px] font-semibold transition-all duration-200
        {activeTab === 'Riwayat'
          ? 'bg-white text-slate-900 shadow-sm'
          : 'text-slate-500 hover:text-slate-700'}"
    >
      Riwayat
    </button>
  </div>
</div>