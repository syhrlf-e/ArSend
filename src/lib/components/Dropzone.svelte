<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { UploadCloud } from 'lucide-svelte';

  const dispatch = createEventDispatcher();

  let isDragging = false;

  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    isDragging = true;
  };
  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    isDragging = false;
  };
  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
  };
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    isDragging = false;
    if (e.dataTransfer?.files) {
      const files = Array.from(e.dataTransfer.files).map((f) => f.name);
      dispatch('filesSelected', files);
    }
  };

  const openFilePicker = async () => {
    try {
      const selected = await open({ multiple: true, directory: false });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        dispatch('filesSelected', paths);
      }
    } catch (error) {
      console.error('Failed to open file picker', error);
    }
  };
</script>

<div
  role="button"
  tabindex="0"
  aria-label="Pilih atau drag file ke sini"
  class="flex min-h-[180px] cursor-pointer flex-col items-center justify-center rounded-[16px] border-2 border-dashed p-8 text-center transition-all duration-200
    {isDragging
    ? 'border-accent bg-accent-light'
    : 'border-slate-300 bg-white hover:border-accent hover:bg-accent-light'}"
  on:dragenter={handleDragEnter}
  on:dragleave={handleDragLeave}
  on:dragover={handleDragOver}
  on:drop={handleDrop}
  on:click={openFilePicker}
  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && openFilePicker()}
>
  <div
    class="mb-4 rounded-full bg-accent-light p-4 text-accent transition-transform duration-200 {isDragging
      ? 'scale-110'
      : ''}"
  >
    <UploadCloud size={28} strokeWidth={1.5} />
  </div>

  <h3 class="mb-1 text-[15px] font-semibold text-slate-900">
    {isDragging ? 'Lepaskan untuk mengirim' : 'Pilih atau drag file ke sini'}
  </h3>
  <p class="text-[13px] text-slate-500">Tidak ada batas ukuran file</p>
</div>
