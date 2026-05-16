<script lang="ts">
  import { onMount } from 'svelte';
  import { Settings, Save, Smartphone, Monitor, FolderOpen, Shield, Trash2 } from 'lucide-svelte';
  import { getDeviceType } from '$lib/utils/platform';
  import { initSettings, saveDeviceName, saveDownloadFolder, downloadFolder } from '$lib/stores/settings';
  import { getTrustedDevices, removeTrustedDevice } from '$lib/stores/trust';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

  export let isMobile = false;

  let deviceType = 'desktop';
  let isSaving = false;
  let saveSuccess = false;
  let localName = '';
  let localFolder = '';
  let publicKeyHex = '';
  let appVersion = '1.0.1';
  let trustedDevices: { public_key: string; name: string }[] = [];

  onMount(async () => {
    deviceType = getDeviceType();
    localName = await initSettings();
    downloadFolder.subscribe((v) => (localFolder = v));

    try {
      const pkInfo: any = await invoke('get_public_key');
      publicKeyHex = pkInfo.public_key_hex || '';
    } catch (e) {
      console.error(e);
    }

    trustedDevices = await getTrustedDevices();
  });

  const handleSave = async () => {
    isSaving = true;
    await saveDeviceName(localName);
    await saveDownloadFolder(localFolder);

    setTimeout(() => {
      isSaving = false;
      saveSuccess = true;
      setTimeout(() => (saveSuccess = false), 2000);
    }, 500);
  };

  const handlePickFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: localFolder
      });

      if (selected && typeof selected === 'string') {
        localFolder = selected;
        await saveDownloadFolder(localFolder);
      }
    } catch (e) {
      console.error('Failed to pick folder', e);
    }
  };

  const handleRemoveTrust = async (pubKey: string) => {
    await removeTrustedDevice(pubKey);
    trustedDevices = await getTrustedDevices();
  };
</script>

<div class="flex flex-col w-full max-w-2xl mx-auto {isMobile ? '' : 'pt-4'}">
  <div class="flex flex-col gap-6">
    <div class="bg-white p-8 rounded-[20px] border border-slate-200 flex flex-col items-center gap-5 text-center">
      <div class="w-24 h-24 bg-accent-light rounded-full flex items-center justify-center text-accent shrink-0 ring-4 ring-slate-50">
        {#if deviceType === 'mobile'}
          <Smartphone size={40} strokeWidth={1.5} />
        {:else}
          <Monitor size={40} strokeWidth={1.5} />
        {/if}
      </div>

      <div class="w-full max-w-sm">
        <label for="deviceName" class="block text-[12px] font-semibold text-slate-500 mb-2 uppercase tracking-wide">
          Nama Perangkat
        </label>
        <input
          id="deviceName"
          type="text"
          bind:value={localName}
          class="w-full bg-slate-50 border border-slate-200 rounded-xl px-4 py-3 text-[16px] font-bold text-slate-900 text-center focus:outline-none focus:border-accent transition-colors"
          placeholder="Masukkan nama perangkat..."
        />
      </div>
    </div>

    <div class="bg-white p-6 rounded-[20px] border border-slate-200">
      <h2 class="text-[15px] font-semibold text-slate-900 mb-4 flex items-center gap-2">
        <FolderOpen size={18} class="text-accent" />
        Penyimpanan
      </h2>
      <div class="flex flex-col gap-2">
        <label for="downloadFolder" class="block text-[12px] font-semibold text-slate-500 uppercase tracking-wide">
          Folder Unduhan
        </label>
        <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2">
          <input
            id="downloadFolder"
            type="text"
            readonly
            value={localFolder}
            class="w-full bg-slate-50 border border-slate-200 rounded-[10px] px-3 py-2.5 text-[13px] text-slate-900 focus:outline-none"
          />
          <button
            on:click={handlePickFolder}
            class="px-4 py-2.5 bg-slate-50 border border-slate-200 rounded-[10px] text-[13px] font-semibold hover:border-accent hover:text-accent transition-colors cursor-pointer whitespace-nowrap active:scale-95"
          >
            Ubah Folder
          </button>
        </div>
      </div>
    </div>

    <div class="bg-white p-6 rounded-[20px] border border-slate-200">
      <h2 class="text-[15px] font-semibold text-slate-900 mb-4 flex items-center gap-2">
        <Shield size={18} class="text-accent" />
        Perangkat Terpercaya
      </h2>
      {#if trustedDevices.length > 0}
        <div class="flex flex-col gap-3">
          {#each trustedDevices as device (device.public_key)}
            <div class="flex items-center justify-between p-3 bg-slate-50 border border-slate-200 rounded-[10px]">
              <div class="flex flex-col overflow-hidden">
                <span class="text-[14px] font-semibold text-slate-900 truncate">{device.name}</span>
                <span class="text-[11px] font-mono text-slate-400 truncate">
                  {device.public_key.substring(0, 16)}...
                </span>
              </div>
              <button
                on:click={() => handleRemoveTrust(device.public_key)}
                class="p-2 text-slate-400 hover:text-error hover:bg-error-light rounded-full transition-colors cursor-pointer"
                title="Hapus dari daftar"
              >
                <Trash2 size={18} />
              </button>
            </div>
          {/each}
        </div>
      {:else}
        <div class="py-6 flex flex-col items-center text-center">
          <Shield size={32} class="text-slate-300 mb-2" strokeWidth={1} />
          <p class="text-[13px] text-slate-500">Belum ada perangkat yang dipercaya.</p>
        </div>
      {/if}
    </div>

    <div class="flex items-center justify-between pt-2 mb-8">
      <span class="text-[12px] font-medium text-slate-400">ArSend v{appVersion}</span>
      <button
        on:click={handleSave}
        disabled={isSaving}
        class="flex items-center gap-2 px-6 py-2.5 bg-accent text-white text-[14px] font-semibold rounded-xl hover:bg-accent-hover transition-colors active:scale-[0.97] cursor-pointer disabled:opacity-70"
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