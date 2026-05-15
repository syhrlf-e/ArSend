<script lang="ts">
  import { onMount } from 'svelte';
  import { Settings, Save, Smartphone, Monitor, ArrowLeft, FolderOpen, Shield, Trash2, Key } from 'lucide-svelte';
  import { getDeviceType } from '$lib/utils/platform';
  import { initSettings, saveDeviceName, saveDownloadFolder, downloadFolder } from '$lib/stores/settings';
  import { getTrustedDevices, removeTrustedDevice } from '$lib/stores/trust';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';

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

<div class="flex flex-col items-center min-h-screen p-6 pb-safe">
  <div class="w-full max-w-2xl">
    <div class="flex items-center justify-between mb-6">
      <a
        href="/"
        class="p-2 -ml-2 rounded-full hover:bg-slate-50 dark:bg-slate-800 transition-colors cursor-pointer text-slate-500 dark:text-slate-400 hover:text-slate-900 dark:text-white"
      >
        <ArrowLeft size={24} />
      </a>
      <h1 class="text-[20px] font-bold text-slate-900 dark:text-white flex items-center gap-2">
        <Settings size={20} class="text-accent" />
        Pengaturan
      </h1>
      <div class="w-10"></div>
      </div>

    <div class="flex flex-col gap-6">
      <div class="bg-white dark:bg-slate-800 p-5 rounded-2xl border border-slate-200 dark:border-slate-700 flex flex-col md:flex-row gap-6 items-center md:items-start shadow-sm">
        <div class="w-20 h-20 bg-accent-light rounded-full flex items-center justify-center text-accent shrink-0 border border-slate-200 dark:border-slate-700">
          {#if deviceType === 'mobile'}
            <Smartphone size={36} strokeWidth={1.5} />
          {:else}
            <Monitor size={36} strokeWidth={1.5} />
          {/if}
        </div>

        <div class="flex-1 w-full flex flex-col gap-4">
          <div>
            <label for="deviceName" class="block text-[12px] font-semibold text-slate-500 dark:text-slate-400 mb-1.5 uppercase tracking-wide">
              Nama Perangkat
            </label>
            <input
              id="deviceName"
              type="text"
              bind:value={localName}
              class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-[10px] px-3 py-2 text-[14px] font-medium text-slate-900 dark:text-white focus:outline-none focus:border-accent transition-colors"
              placeholder="Masukkan nama perangkat..."
            />
          </div>
          <div>
            <label class="block text-[12px] font-semibold text-slate-500 dark:text-slate-400 mb-1.5 uppercase tracking-wide">
              Public Key Fingerprint
            </label>
            <div class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-[10px] px-3 py-2 text-[12px] font-mono text-slate-500 dark:text-slate-400 flex items-center gap-2 break-all overflow-hidden">
              <Key size={14} class="shrink-0" />
              <span class="truncate">{publicKeyHex || 'Loading...'}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-slate-800 p-5 rounded-[16px] border border-slate-200 dark:border-slate-700 shadow-sm">
        <h2 class="text-[15px] font-semibold text-slate-900 dark:text-white mb-4 flex items-center gap-2">
          <FolderOpen size={18} class="text-accent" />
          Penyimpanan
        </h2>
        <div class="flex flex-col gap-2">
          <label class="block text-[12px] font-semibold text-slate-500 dark:text-slate-400 uppercase tracking-wide">
            Folder Unduhan
          </label>
          <div class="flex items-center gap-2">
            <input
              type="text"
              readonly
              value={localFolder}
              class="w-full bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-[10px] px-3 py-2 text-[13px] text-slate-900 dark:text-white focus:outline-none"
            />
            <button
              on:click={handlePickFolder}
              class="px-4 py-2 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-[10px] text-[13px] font-medium hover:border-accent hover:text-accent transition-colors cursor-pointer whitespace-nowrap"
            >
              Ubah Folder
            </button>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-slate-800 p-5 rounded-2 border border-slate-200 dark:border-slate-700 shadow-sm">
        <h2 class="text-[15px] font-semibold text-slate-900 dark:text-white mb-4 flex items-center gap-2">
          <Shield size={18} class="text-accent" />
          Perangkat Terpercaya
        </h2>
        {#if trustedDevices.length > 0}
          <div class="flex flex-col gap-3">
            {#each trustedDevices as device (device.public_key)}
              <div class="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-[10px]">
                <div class="flex flex-col overflow-hidden">
                  <span class="text-[14px] font-semibold text-slate-900 dark:text-white truncate">{device.name}</span>
                  <span class="text-[11px] font-mono text-slate-400 dark:text-slate-500 truncate">
                    {device.public_key.substring(0, 16)}...
                  </span>
                </div>
                <button
                  on:click={() => handleRemoveTrust(device.public_key)}
                  class="p-2 text-slate-400 dark:text-slate-500 hover:text-error hover:bg-error-light rounded-full transition-colors cursor-pointer"
                  title="Hapus dari daftar"
                >
                  <Trash2 size={18} />
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="py-6 flex flex-col items-center text-center">
            <Shield size={32} class="text-border-strong mb-2" strokeWidth={1} />
            <p class="text-[13px] text-slate-500 dark:text-slate-400">Belum ada perangkat yang dipercaya.</p>
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between pt-2 mb-8">
        <span class="text-[12px] font-medium text-slate-400 dark:text-slate-500">ArSend v{appVersion}</span>
        <button
          on:click={handleSave}
          disabled={isSaving}
          class="flex items-center gap-2 px-6 py-2.5 bg-accent text-white text-[14px] font-semibold rounded-xl shadow-sm hover:bg-accent-hover transition-colors active:scale-[0.97] cursor-pointer disabled:opacity-70"
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
