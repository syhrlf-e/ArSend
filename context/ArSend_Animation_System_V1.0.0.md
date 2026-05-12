# Animation & Motion System
# ArSend — Supplementary Design Document

**Versi:** 1.0.0
**Tanggal:** 10 Mei 2026
**Author:** Syahrul Efendi (Rull)
**Tipe:** Suplemen PRD_ArSend_V1.0.1
**Status:** ✅ Final

---

## Daftar Isi

1. Filosofi Animasi
2. Prinsip Motion
3. Signature Animation — File Transfer Reveal
   - 3.1 Konsep & Story
   - 3.2 Sisi Pengirim
   - 3.3 Sisi Penerima
   - 3.4 Device Icon Dinamis
   - 3.5 Teknik Implementasi
4. File Icon System
   - 4.1 Konsep
   - 4.2 Struktur SVG Generic
   - 4.3 SVG Custom per Ekstensi
   - 4.4 Logic & Folder Structure
5. Animasi Pendukung
   - 5.1 Koneksi & Pairing
   - 5.2 State Transitions
   - 5.3 Feedback & Micro-interaction
6. Timing & Easing Reference
7. Prinsip yang Tidak Boleh Dilanggar

---

## 1. Filosofi Animasi

ArSend memiliki satu prinsip utama dalam animasi:

> *Animasi bukan dekorasi — ia adalah komunikasi.*

Setiap gerakan di ArSend harus:
- **Bermakna** — menceritakan sesuatu yang sedang terjadi
- **Intuitif** — user paham tanpa membaca teks
- **Ringan** — tidak membebani performa app
- **Distinctive** — menjadi identitas visual yang tidak dimiliki app lain

ArSend menolak animasi generik seperti progress bar horizontal biasa. Setiap state harus punya **visual storytelling** yang jelas.

---

## 2. Prinsip Motion

### 2.1 Purposeful
Tidak ada animasi tanpa tujuan. Setiap motion harus menjawab pertanyaan: *"Apa yang sedang terjadi?"*

### 2.2 Responsive
Animasi mengikuti data real — progress transfer dari Rust via Tauri event, bukan timer palsu.

### 2.3 Lightweight
Semua animasi menggunakan **Svelte built-in** (`tweened`, `spring`) dan **CSS** — zero dependency tambahan, zero runtime overhead.

### 2.4 Consistent
Bahasa motion yang sama dipakai di seluruh app — arah, easing, dan timing konsisten.

### 2.5 Interruptible
Semua animasi bisa di-interrupt kapanpun — jika transfer dibatalkan, animasi berhenti dengan clean.

---

## 3. Signature Animation — File Transfer Reveal

Ini adalah animasi paling penting di ArSend — identitas visual utama yang membedakan ArSend dari semua app transfer file lain.

### 3.1 Konsep & Story

Animasi ini menceritakan perpindahan file secara literal:

```
PENGIRIM                        PENERIMA

     📄                              📄
     ↑                               ↓
     ↑    ════════════════►          ↓
     ↑                               ↓
 ┌──╔══╗──┐                    ┌──╔══╗──┐
 │  slot  │                    │  slot  │
 └────────┘                    └────────┘
 [device]                      [device]

File keluar dari device         File masuk ke device
bawah → atas                    atas → bawah
```

**Story yang tersampaikan:**
> *File secara literal keluar dari device pengirim dan masuk ke device penerima — tanpa perlu teks penjelasan apapun.*

---

### 3.2 Sisi Pengirim

File SVG icon **reveal dari bawah ke atas** seiring progress transfer.

```
Progress 0%     Progress 33%    Progress 66%    Progress 100%

                    📄 ▓           📄 ▓▓▓          📄 ▓▓▓▓▓▓
                    │ ▓           │ ▓▓▓          │ ▓▓▓▓▓▓
 ┌────────┐     ┌───│────┐    ┌───│────┐    ┌───│────┐
 │ [slot] │     │ [slot] │    │ [slot] │    │ [slot] │
 └────────┘     └────────┘    └────────┘    └────────┘

▓ = bagian file yang sudah "keluar"
```

**Detail behavior:**
- File icon muncul perlahan dari slot di bagian bawah device icon
- Clip-path `inset(top% 0% 0% 0%)` — top berkurang seiring progress naik
- Kecepatan reveal proporsional dengan kecepatan transfer aktual
- File icon fully revealed = transfer 100%

---

### 3.3 Sisi Penerima

File SVG icon **reveal dari atas ke bawah** seiring progress transfer.

```
Progress 0%     Progress 33%    Progress 66%    Progress 100%

                    📄 ▓▓▓          📄 ▓▓▓          📄 ▓▓▓▓▓▓
                       ▓▓▓             ▓▓▓▓▓▓
 ┌────────┐     ┌────────┐    ┌────────┐    ┌────────┐
 │ [slot] │     │ [slot] │    │ [slot] │    │ [slot] │
 └────────┘     └────────┘    └────────┘    └────────┘

▓ = bagian file yang sudah "masuk"
```

**Detail behavior:**
- File icon muncul dari atas, turun perlahan menuju slot di device icon
- Clip-path `inset(0% 0% bottom% 0%)` — bottom berkurang seiring progress naik
- Saat 100% → file icon "masuk" penuh ke dalam slot device
- Diikuti finish animation (lihat section 4.3)

---

### 3.4 Device Icon Dinamis

Device icon ditentukan berdasarkan tipe device yang terdeteksi saat pairing.

| Device Type | Icon (Lucide) | Fallback |
|---|---|---|
| Laptop | `Laptop` | — |
| Desktop / PC | `Monitor` | — |
| Smartphone / HP | `Smartphone` | — |
| Tidak terdeteksi | `Folder` | ✅ Default fallback |

**Deteksi device type:**
- Info device dikirim dalam payload pairing / UDP broadcast
- Field `device_type`: `"laptop"` \| `"desktop"` \| `"mobile"` \| `"unknown"`
- Frontend map ke Lucide icon yang sesuai

```svelte
const deviceIconMap = {
  laptop:  LaptopIcon,
  desktop: MonitorIcon,
  mobile:  SmartphoneIcon,
  unknown: FolderIcon
}

$: DeviceIcon = deviceIconMap[deviceType] ?? FolderIcon
```

---

### 3.5 Teknik Implementasi

**Stack yang digunakan:**
- Svelte `tweened` — smooth interpolasi nilai progress
- Svelte `spring` — untuk finish animation (bounce)
- CSS `clip-path: inset()` — teknik reveal utama
- SVG custom — file icon dengan kontrol penuh per elemen
- Lucide Icons — device icon dinamis

**Tidak menggunakan:**
- Framer Motion (React only)
- svelte-motion (dependency tidak perlu)
- GSAP (overkill, berbayar untuk commercial)
- Canvas / WebGL (terlalu kompleks untuk kebutuhan ini)

---

**Struktur SVG File Icon:**

File icon dibuat custom — bukan dari Lucide — karena butuh kontrol penuh untuk clip-path animation.

```svg
<svg viewBox="0 0 60 80" fill="none" xmlns="http://www.w3.org/2000/svg">
  <!-- Body dokumen -->
  <rect
    x="4" y="8" width="44" height="64"
    rx="4" ry="4"
    fill="#EEF4FF" stroke="#0045B5" stroke-width="2"
  />
  <!-- Pojok lipatan kanan atas -->
  <path
    d="M36 8 L48 20 L36 20 Z"
    fill="#BFDBFE" stroke="#0045B5" stroke-width="1.5"
  />
  <!-- Garis konten (dekoratif) -->
  <line x1="12" y1="34" x2="40" y2="34" stroke="#94A3B8" stroke-width="2" stroke-linecap="round"/>
  <line x1="12" y1="44" x2="40" y2="44" stroke="#94A3B8" stroke-width="2" stroke-linecap="round"/>
  <line x1="12" y1="54" x2="28" y2="54" stroke="#94A3B8" stroke-width="2" stroke-linecap="round"/>
</svg>
```

---

**Svelte Component — Pengirim:**

```svelte
<script>
  import { tweened } from 'svelte/motion'
  import { cubicOut } from 'svelte/easing'
  import FileIcon from './FileIcon.svelte'

  export let progress = 0        // 0–100, dari Tauri event
  export let deviceType = 'unknown'

  const smoothProgress = tweened(0, {
    duration: 300,
    easing: cubicOut
  })

  $: smoothProgress.set(progress)

  // Reveal bawah ke atas — top clip berkurang seiring progress naik
  $: clipPath = `inset(${100 - $smoothProgress}% 0% 0% 0%)`
</script>

<div class="sender-animation">
  <!-- Device icon -->
  <DeviceIcon size={48} strokeWidth={1.5} />

  <!-- File icon dengan clip-path reveal -->
  <div class="file-reveal" style="clip-path: {clipPath}">
    <FileIcon />
  </div>
</div>
```

---

**Svelte Component — Penerima:**

```svelte
<script>
  import { tweened, spring } from 'svelte/motion'
  import { cubicOut, backOut } from 'svelte/easing'

  export let progress = 0
  export let deviceType = 'unknown'

  const smoothProgress = tweened(0, {
    duration: 300,
    easing: cubicOut
  })

  $: smoothProgress.set(progress)

  // Reveal atas ke bawah — bottom clip berkurang seiring progress naik
  $: clipPath = `inset(0% 0% ${100 - $smoothProgress}% 0%)`

  // Finish animation saat progress = 100
  $: if (progress >= 100) triggerFinish()

  function triggerFinish() {
    // File "masuk" ke device dengan subtle bounce
    // implementasi via spring animation
  }
</script>

<div class="receiver-animation">
  <div class="file-reveal" style="clip-path: {clipPath}">
    <FileIcon />
  </div>

  <!-- Device icon dengan slot di atas -->
  <DeviceIcon size={48} strokeWidth={1.5} />
</div>
```

---

## 4. File Icon System

### 4.1 Konsep

File icon di ArSend bersifat **dinamis berdasarkan ekstensi file** — bukan satu icon generik untuk semua tipe file. Ini memperkuat visual storytelling saat animasi transfer berjalan.

**Dua jenis file icon:**
- **SVG Generic** — putih/clean, label ekstensi di tengah, untuk ekstensi umum
- **SVG Custom** — didesain manual oleh author untuk ekstensi populer (PDF, Excel, Word, dll)

---

### 4.2 Struktur SVG Generic

Untuk ekstensi yang tidak punya SVG custom, ditampilkan file icon putih dengan label ekstensi di bagian tengah.

```
┌──────┐
│      │  ← body putih, border tipis
│ .jpg │  ← label ekstensi (DM Mono)
│      │
└──────┘
```

Struktur SVG:

```svg
<svg viewBox="0 0 60 80" fill="none" xmlns="http://www.w3.org/2000/svg">
  <!-- Body putih -->
  <rect
    x="4" y="8" width="44" height="64" rx="4"
    fill="#FFFFFF"
    stroke="#E2E8F0"
    stroke-width="2"
  />
  <!-- Pojok lipatan kanan atas -->
  <path
    d="M36 8 L48 20 L36 20 Z"
    fill="#F1F5F9"
    stroke="#E2E8F0"
    stroke-width="1.5"
  />
  <!-- Label ekstensi di tengah -->
  <text
    x="26" y="52"
    text-anchor="middle"
    font-family="DM Mono"
    font-size="10"
    font-weight="500"
    fill="#64748B"
  >
    .ext
  </text>
</svg>
```

**Aturan label:**
- Font: **DM Mono** — konsisten dengan design system
- Warna: `#64748B` (Text Secondary) — tidak terlalu mencolok
- Format: lowercase dengan titik (`.jpg`, `.pdf`, `.zip`)
- Jika ekstensi > 4 karakter: potong + ellipsis (`.docx` → `.docx`, `.pages` → `.page`)

---

### 4.3 SVG Custom per Ekstensi

Ekstensi berikut memiliki SVG custom yang didesain manual oleh author — memberikan karakter visual yang lebih kuat dan recognizable.

| Ekstensi | File SVG | Keterangan |
|---|---|---|
| `.pdf` | `pdf.svg` | Desain manual |
| `.xlsx` | `xlsx.svg` | Desain manual |
| `.xls` | `xls.svg` | Desain manual |
| `.docx` | `docx.svg` | Desain manual |
| `.doc` | `doc.svg` | Desain manual |
| `.pptx` | `pptx.svg` | Desain manual |
| `.ppt` | `ppt.svg` | Desain manual |
| *(lainnya)* | `generic.svg` | SVG generic + label ext |

> SVG custom dibuat sepenuhnya oleh author — bukan dari library manapun. Developer cukup menempatkan file SVG di folder yang benar.

---

### 4.4 Logic & Folder Structure

**Folder struktur:**

```
src/
└── lib/
    └── assets/
        └── file-icons/
            ├── pdf.svg       ← custom
            ├── xlsx.svg      ← custom
            ├── xls.svg       ← custom
            ├── docx.svg      ← custom
            ├── doc.svg       ← custom
            ├── pptx.svg      ← custom
            ├── ppt.svg       ← custom
            └── generic.svg   ← fallback semua ekstensi lain
```

**Logic di Svelte:**

```svelte
<script>
  export let filename = ''

  // Daftar ekstensi dengan SVG custom
  const customExtensions = ['pdf', 'xlsx', 'xls', 'docx', 'doc', 'pptx', 'ppt']

  // Baca ekstensi dari nama file
  $: ext = filename.split('.').pop()?.toLowerCase() ?? 'unknown'

  // Tentukan SVG yang dipakai
  $: iconPath = customExtensions.includes(ext)
    ? `/file-icons/${ext}.svg`
    : null  // null = gunakan SVG generic dengan label

  // Label untuk SVG generic
  $: label = `.${ext}`
</script>

{#if iconPath}
  <!-- SVG custom -->
  <img src={iconPath} alt={ext} />
{:else}
  <!-- SVG generic dengan label ekstensi -->
  <GenericFileIcon {label} />
{/if}
```

**Aturan penambahan SVG custom di masa depan:**
1. Buat file SVG dengan viewBox `0 0 60 80`
2. Simpan di `src/lib/assets/file-icons/{ext}.svg`
3. Tambahkan ekstensi ke array `customExtensions`
4. Tidak perlu ubah logic lainnya

---

## 5. Animasi Pendukung

### 5.1 Koneksi & Pairing

**QR Code muncul (desktop):**
- Fade in + scale dari 0.95 → 1.0
- Duration: 200ms, easing: `cubicOut`

**Device terdeteksi di list (discovery):**
- Slide in dari kanan + fade in
- Duration: 250ms, easing: `cubicOut`
- Stagger: 50ms per item jika multiple device muncul bersamaan

**TOFU Modal muncul:**
- Backdrop fade in (opacity 0 → 0.4)
- Modal scale dari 0.95 → 1.0 + fade in
- Duration: 200ms, easing: `cubicOut`

**Terhubung (paired):**
- Connection bar slide down dari atas
- Status dot pulse animation (infinite, subtle)
- Duration: 300ms, easing: `cubicOut`

---

### 5.2 State Transitions

**Belum paired → Sudah paired:**
- Card Kirim & Terima fade in + slide up dari bawah
- Duration: 300ms, easing: `cubicOut`
- Stagger: 80ms antara card Kirim dan Terima

**Tab switch (Transfer ↔ Riwayat):**
- Konten fade out → fade in
- Duration: 150ms, easing: `cubicOut`
- Pill indicator slide ke tab aktif
- Duration: 200ms, easing: `cubicOut`

**Halaman Koneksi Terputus:**
- Fade in seluruh halaman
- Icon `WifiOff` shake animation (sekali, subtle)
- Duration: 300ms

**Reconnect berhasil:**
- Halaman koneksi terputus fade out
- Kembali ke home dengan fade in
- Status dot kembali pulse hijau

---

### 5.3 Feedback & Micro-interaction

**Transfer selesai — Pengirim:**
- Banner sukses slide down dari atas
- Icon `CheckCircle` scale pop: 0 → 1.2 → 1.0
- Duration: 400ms, easing: `backOut`
- Warna accent berubah dari `#0045B5` ke `#10B981` (brief, 500ms)

**Transfer selesai — Penerima:**
- File icon fully revealed → subtle bounce up (spring)
- File icon "masuk" ke slot device dengan scale shrink: 1.0 → 0.0
- Device icon brief glow/pulse: `#10B981`
- Banner sukses muncul dari bawah
- Duration keseluruhan: ~600ms

**File ditolak:**
- File icon shake animation (horizontal, 3×)
- Warna brief flash ke `#F43F5E`
- Banner notifikasi ditolak slide in
- Duration: 400ms

**Tombol diklik (semua tombol):**
- Scale: 1.0 → 0.97 saat pressed, kembali ke 1.0 saat release
- Duration: 100ms, easing: linear
- Implementasi via CSS `active:scale-[0.97]` Tailwind

**QR expire warning (< 30 detik):**
- Progress bar berubah warna dari `#0045B5` ke `#F59E0B`
- Subtle pulse animation pada progress bar
- Duration transisi warna: 500ms

**Device list item hover (desktop):**
- Background fade ke `#EEF4FF`
- Subtle translate X: 0 → 2px
- Duration: 150ms, easing: `cubicOut`

---

## 6. Timing & Easing Reference

### Durasi Standar

| Kategori | Durasi | Penggunaan |
|---|---|---|
| Micro | 100ms | Button press, hover state |
| Fast | 150–200ms | Tab switch, fade ringan |
| Standard | 250–300ms | Modal, card muncul, state transition |
| Expressive | 400–600ms | Signature animation finish, sukses state |

**Aturan:** Tidak ada animasi UI melebihi **600ms** — kecuali signature animation transfer yang mengikuti durasi transfer aktual.

---

### Easing Reference

| Nama | Svelte Import | Penggunaan |
|---|---|---|
| `cubicOut` | `svelte/easing` | Mayoritas animasi — elemen masuk ke layar |
| `cubicIn` | `svelte/easing` | Elemen keluar dari layar |
| `cubicInOut` | `svelte/easing` | Tab switch, transisi simetris |
| `backOut` | `svelte/easing` | Finish animation, success pop |
| `linear` | built-in | Progress reveal (mengikuti data real) |
| `spring` | `svelte/motion` | Bounce, physics-based finish |

**Spring config untuk finish animation:**
```javascript
const finishSpring = spring(0, {
  stiffness: 0.3,
  damping: 0.6
})
```

---

## 7. Prinsip yang Tidak Boleh Dilanggar

- ❌ **Tidak ada animasi looping tanpa makna** — spinner hanya muncul saat benar-benar loading
- ❌ **Tidak ada animasi > 600ms untuk UI** — kecuali signature transfer animation
- ❌ **Tidak ada library animasi eksternal** — hanya Svelte built-in + CSS
- ❌ **Tidak ada animasi yang tidak bisa di-interrupt** — semua bisa dibatalkan
- ❌ **Tidak ada dua animasi besar berjalan bersamaan** — hindari visual noise
- ✅ **Semua animasi mengikuti data real** — tidak ada timer palsu
- ✅ **Animasi harus tetap smooth di device low-end** — test di Android entry-level
- ✅ **Gunakan `prefers-reduced-motion`** — hormati user yang sensitif terhadap animasi

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

*Dokumen ini adalah suplemen dari PRD_ArSend_V1.0.1*
*Setiap perubahan pada sistem animasi harus diperbarui di dokumen ini.*
