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
      const files = Array.from(e.dataTransfer.files).map(f => f.name); // Using names as placeholders, Tauri handles actual paths differently
      // In a real Tauri app, file drops might need to be handled via Tauri events or by extracting paths if possible.
      // For now, we'll emit what we can.
      dispatch('filesSelected', files);
    }
  };

  const openFilePicker = async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      });
      if (selected) {
        // If multiple is true, it returns an array of paths
        const paths = Array.isArray(selected) ? selected : [selected];
        dispatch('filesSelected', paths);
      }
    } catch (error) {
      console.error('Failed to open file picker', error);
    }
  };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
  class="border-2 border-dashed rounded-[16px] p-8 flex flex-col items-center justify-center text-center transition-all duration-200 cursor-pointer min-h-[200px] {isDragging ? 'border-accent bg-accent-light' : 'border-border-strong hover:border-accent hover:bg-surface-2'}"
  on:dragenter={handleDragEnter}
  on:dragleave={handleDragLeave}
  on:dragover={handleDragOver}
  on:drop={handleDrop}
  on:click={openFilePicker}
  on:keydown={(e) => e.key === 'Enter' && openFilePicker()}
>
  <div class="p-4 bg-accent-light rounded-full text-accent mb-4 transition-transform duration-200 {isDragging ? 'scale-110' : ''}">
    <UploadCloud size={32} strokeWidth={1.5} />
  </div>
  
  <h3 class="text-[15px] font-semibold text-text-primary mb-1">
    {isDragging ? 'Lepaskan untuk mengirim' : 'Pilih atau drag file ke sini'}
  </h3>
  <p class="text-[13px] text-text-secondary">
    Tidak ada batas ukuran file
  </p>
</div>
