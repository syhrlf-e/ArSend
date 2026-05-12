<script lang="ts">
  import { onMount } from 'svelte';
  import { Settings, Save, Smartphone, Monitor } from 'lucide-svelte';
  import { getDeviceType } from '$lib/utils/platform';
  import { initSettings, saveDeviceName, deviceName } from '$lib/stores/settings';

  let deviceType = 'desktop';
  let isSaving = false;
  let saveSuccess = false;
  let localName = '';

  onMount(async () => {
      deviceType = getDeviceType();
      localName = await initSettings();
  });

  const handleSave = async () => {
      isSaving = true;
      await saveDeviceName(localName);
      setTimeout(() => {
          isSaving = false;
          saveSuccess = true;
          setTimeout(() => saveSuccess = false, 2000);
      }, 500);
  };
</script>

<div class="flex flex-col items-center min-h-screen p-6">
  <div class="w-full max-w-2xl bg-surface border border-border rounded-[16px] p-6 shadow-sm mt-8">
      <div class="flex items-center gap-2 mb-6">
          <Settings size={24} class="text-accent" />
          <h1 class="text-[20px] font-bold text-text-primary">Pengaturan Profil</h1>
      </div>

      <div class="flex flex-col gap-6">
          <!-- Profil Section -->
          <div class="bg-surface-2 p-5 rounded-[12px] border border-border flex flex-col md:flex-row gap-6 items-center md:items-start">
              <div class="w-20 h-20 bg-accent-light rounded-full flex items-center justify-center text-accent shrink-0 border-2 border-surface">
                  {#if deviceType === 'mobile'}
                      <Smartphone size={36} strokeWidth={1.5} />
                  {:else}
                      <Monitor size={36} strokeWidth={1.5} />
                  {/if}
              </div>

              <div class="flex-1 w-full">
                  <label for="deviceName" class="block text-[13px] font-semibold text-text-secondary mb-1.5 uppercase tracking-wide">Nama Perangkat</label>
                  <input 
                      id="deviceName"
                      type="text" 
                      bind:value={localName}
                      class="w-full bg-surface border border-border-main rounded-[12px] px-4 py-2.5 text-[15px] font-medium text-text-primary focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/30 transition-all"
                      placeholder="Masukkan nama perangkat..."
                  />
                  <p class="text-[12px] text-text-tertiary mt-2">
                      Nama ini akan terlihat oleh perangkat lain saat proses pairing.
                  </p>
              </div>
          </div>

          <!-- Network Settings (Placeholder) -->
          <div class="bg-surface-2 p-5 rounded-[12px] border border-border">
              <h2 class="text-[15px] font-semibold text-text-primary mb-1">Pengaturan Jaringan</h2>
              <p class="text-[13px] text-text-secondary mb-4">Konfigurasi port dan preferensi koneksi (Auto-discovery).</p>
              
              <div class="flex items-center justify-between py-2 border-t border-border/50">
                  <span class="text-[14px] font-medium text-text-primary">Auto-discovery (UDP Broadcast)</span>
                  <div class="w-11 h-6 bg-accent rounded-full relative cursor-pointer">
                      <div class="absolute right-1 top-1 w-4 h-4 bg-white rounded-full"></div>
                  </div>
              </div>
          </div>

          <div class="flex justify-end pt-4">
              <button 
                  on:click={handleSave}
                  disabled={isSaving}
                  class="flex items-center gap-2 px-6 py-2.5 bg-accent text-white text-[14px] font-semibold rounded-[12px] shadow-sm hover:bg-accent-hover transition-colors active:scale-[0.97] cursor-pointer disabled:opacity-70"
              >
                  {#if saveSuccess}
                      Tersimpan!
                  {:else}
                      <Save size={18} />
                      {isSaving ? 'Menyimpan...' : 'Simpan Perubahan'}
                  {/if}
              </button>
          </div>
      </div>
  </div>
</div>
