<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { Scan, X } from 'lucide-svelte';
  import { scan, cancel, checkPermissions, requestPermissions, Format } from '@tauri-apps/plugin-barcode-scanner';

  const dispatch = createEventDispatcher();
  let hasPermission = false;
  let isScanning = false;

  const handleClose = async () => {
    if (isScanning) {
        try {
            await cancel();
        } catch(e) {
            console.error(e);
        }
    }
    dispatch('close');
  };

  onMount(async () => {
      document.body.classList.add('scanner-active');
      document.documentElement.classList.add('scanner-active');
      try {
        let perm = await checkPermissions();
        if (perm !== 'granted') {
            perm = await requestPermissions();
        }
        
        if (perm === 'granted') {
            hasPermission = true;
            startScanning();
        } else {
            dispatch('close');
        }
      } catch (error) {
          console.error("Camera permission error:", error);
          dispatch('close');
      }
  });

  onDestroy(async () => {
      document.body.classList.remove('scanner-active');
      document.documentElement.classList.remove('scanner-active');
      if (isScanning) {
        try { await cancel(); } catch(e) {}
      }
  });

  const startScanning = async () => {
      if (!hasPermission) return;
      isScanning = true;
      try {
          // windowed: true tells the plugin to render the camera BEHIND the webview
          const result = await scan({ windowed: true, formats: [Format.QRCode] });
          if (result && result.content) {
              dispatch('scanned', result.content);
          }
      } catch (error) {
          console.error("Scanning failed:", error);
      } finally {
          isScanning = false;
      }
  };
</script>

<div class="fixed inset-0 bg-transparent z-50 flex flex-col">
  <!-- Top bar with Close button -->
  <div class="flex justify-between items-center px-4 pb-4 pt-[max(16px,env(safe-area-inset-top))] bg-gradient-to-b from-black/80 to-transparent relative z-20">
    <div class="flex items-center gap-2 text-white font-semibold">
      <Scan size={20} />
      <span>Scan QR Code</span>
    </div>
    <button 
      on:click={handleClose}
      class="p-2 text-white/80 hover:text-white rounded-full bg-white/10 hover:bg-white/20 transition-colors active:scale-[0.97] cursor-pointer pointer-events-auto"
    >
      <X size={24} />
    </button>
  </div>

  <!-- Center scanning area -->
  <div class="flex-1 relative flex items-center justify-center pointer-events-none">
    {#if !hasPermission}
      <div class="absolute inset-0 bg-black flex items-center justify-center -z-10">
          <p class="text-white/50 text-[14px]">Menunggu akses kamera...</p>
      </div>
    {/if}

    <!-- Scanner Overlay / Viewfinder -->
    <div class="w-64 h-64 border-2 border-white/20 rounded-[24px] relative z-10 overflow-hidden">
      <!-- Corner Markers -->
      <div class="absolute top-0 left-0 w-8 h-8 border-t-4 border-l-4 border-accent rounded-tl-[24px]"></div>
      <div class="absolute top-0 right-0 w-8 h-8 border-t-4 border-r-4 border-accent rounded-tr-[24px]"></div>
      <div class="absolute bottom-0 left-0 w-8 h-8 border-b-4 border-l-4 border-accent rounded-bl-[24px]"></div>
      <div class="absolute bottom-0 right-0 w-8 h-8 border-b-4 border-r-4 border-accent rounded-br-[24px]"></div>

      <!-- Scanning Animation Line -->
      <div class="w-full h-0.5 bg-accent shadow-[0_0_8px_2px_rgba(0,69,181,0.5)] animate-[scan_2s_ease-in-out_infinite]"></div>
    </div>
  </div>

  <!-- Bottom text -->
  <div class="p-8 text-center bg-gradient-to-t from-black/90 to-transparent relative z-20">
    <p class="text-white/80 text-[14px] leading-relaxed mb-6">
      Arahkan kamera ke QR Code di perangkat pengirim untuk terhubung secara langsung.
    </p>
  </div>
</div>

<style>
  @keyframes scan {
    0% { transform: translateY(0); opacity: 0; }
    10% { opacity: 1; }
    90% { opacity: 1; }
    100% { transform: translateY(256px); opacity: 0; }
  }
</style>