<script lang="ts">
  export let activeTab = 'Transfer';
</script>

<div class="flex items-center justify-center p-2">
  <div class="bg-surface-2 rounded-full p-1 flex w-full max-w-sm border border-border">
    <button 
      on:click={() => activeTab = 'Transfer'}
      class="flex-1 text-center py-2 text-[14px] font-semibold rounded-full transition-all duration-200 cursor-pointer {activeTab === 'Transfer' ? 'bg-surface shadow-sm text-text-primary' : 'text-text-secondary hover:text-text-primary'}"
    >
      Transfer
    </button>
    <button 
      on:click={() => activeTab = 'Riwayat'}
      class="flex-1 text-center py-2 text-[14px] font-semibold rounded-full transition-all duration-200 cursor-pointer {activeTab === 'Riwayat' ? 'bg-surface shadow-sm text-text-primary' : 'text-text-secondary hover:text-text-primary'}"
    >
      Riwayat
    </button>
  </div>
</div>
