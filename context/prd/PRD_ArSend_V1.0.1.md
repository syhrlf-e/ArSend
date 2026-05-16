# Product Requirements Document
# ArSend — Local File Transfer App

**Versi:** 1.0.1
**Tanggal:** 10 Mei 2026
**Author:** Syahrul Efendi (Rull)
**Status:** ✅ Final — Siap Development

---

## Daftar Isi

**BAGIAN I — PRODUK & IDENTITAS**
1. Ringkasan Produk
2. Asal Nama
3. Visi & Misi
4. Keunggulan vs Kompetitor
5. Target Pengguna

**BAGIAN II — DESAIN & PENGALAMAN PENGGUNA**
6. Design System
   - 6.1 Identitas Visual
   - 6.2 Typography Scale
   - 6.3 Color Palette
   - 6.4 Spacing System
   - 6.5 Icon System
   - 6.6 Prinsip & Hukum UI/UX
   - 6.7 Layout Desktop
   - 6.8 Layout Mobile
7. Alur Pengguna (User Flow)
   - 7.1 Onboarding
   - 7.2 Koneksi & Pairing
   - 7.3 Kirim File
   - 7.4 Terima File
   - 7.5 Disconnect

**BAGIAN III — FITUR & SPESIFIKASI**
8. Fitur Lengkap
   - 8.1 Home — Belum Terhubung
   - 8.2 TOFU — Pertama Kali Terhubung
   - 8.3 Home — Sudah Terhubung
   - 8.4 Kirim File
   - 8.5 Terima File
   - 8.6 Riwayat Transfer
   - 8.7 Notifikasi Background
   - 8.8 Profile & Settings
9. Struktur Layar
   - 9.1 Mobile
   - 9.2 Desktop
10. Batasan v1.0

**BAGIAN IV — TEKNIS & ARSITEKTUR**
11. Tech Stack
12. Platform & Minimum OS
13. Arsitektur Sistem
    - 13.1 Big Picture
    - 13.2 Layer Komunikasi
    - 13.3 Peran Setiap Module Rust
14. Mekanisme Koneksi
    - 14.1 Mode 1 — Auto Discovery (UDP)
    - 14.2 Mode 2 — QR Pairing
    - 14.3 Multi-Device Support
    - 14.4 Protokol Transfer File
    - 14.5 Disconnect Handling
15. Keamanan & Enkripsi
    - 15.1 Threat Model
    - 15.2 Security Stack (6 Layer)
    - 15.3 Impact Keamanan terhadap Performa
    - 15.4 Crate Security
16. Estimasi Performa Transfer
17. Struktur Folder Project

**BAGIAN V — PERENCANAAN**
18. Risiko & Mitigasi
19. Milestone & Phase Development

---

# BAGIAN I — PRODUK & IDENTITAS

---

## 1. Ringkasan Produk

ArSend adalah aplikasi transfer file lokal lintas platform (desktop & Android) yang memungkinkan pengguna mengirim dan menerima file antar perangkat tanpa memerlukan internet, kabel USB, atau layanan cloud.

Koneksi dilakukan melalui jaringan lokal (WiFi yang sama atau hotspot HP) dengan dua mekanisme:
- **Auto Discovery** — semua device ArSend di jaringan terdeteksi otomatis
- **QR Pairing** — koneksi langsung ke device spesifik

> *"Kirim file, secepat lokalmu."*

---

## 2. Asal Nama

**ArSend** lahir dari dua makna personal yang menyatu dalam satu kata:

- **Ar** — penyebutan huruf R dalam bahasa Inggris, inisial dari nama panggilan **Rull**
- **Send** — diambil dari **S**yahrul Efe**nd**i, sekaligus merepresentasikan fungsi utama aplikasi yaitu mengirim file

Nama ini bukan sekadar branding — ia adalah identitas personal yang tertanam langsung dalam produk.

---

## 3. Visi & Misi

### Visi
> Mengurangi latensi dan menghilangkan pengaruh lambatnya jaringan internet dalam proses transfer file antar perangkat pribadi — dengan keamanan end-to-end, tanpa server, tanpa cloud, tanpa kompromi.

### Misi
- Membuat transfer file antar device semudah dan secepat mungkin
- Menjamin keamanan file pengguna tanpa bergantung pihak ketiga
- Memberikan pengalaman yang ringan, bersih, dan intuitif di semua platform

---

## 4. Keunggulan vs Kompetitor

| | **ArSend** | WhatsApp | Google Drive | AirDrop | LocalSend |
|---|---|---|---|---|---|
| Internet | ❌ Tidak perlu | ✅ Wajib | ✅ Wajib | ❌ Tidak perlu | ❌ Tidak perlu |
| Enkripsi E2E | ✅ | ✅ | ⚠️ Partial | ✅ | ⚠️ Partial |
| Batas ukuran file | ❌ Tidak ada | ✅ Ada | ✅ Ada (free) | ❌ Tidak ada | ❌ Tidak ada |
| Cross platform | ✅ | ✅ | ✅ | ❌ Apple only | ✅ |
| Multi device | ✅ | ✅ | ✅ | ⚠️ Terbatas | ✅ |
| Kecepatan | 🔥 Lokal penuh | 🐢 Internet | 🐢 Internet | 🔥 Lokal | 🔥 Lokal |
| Approval penerima | ✅ | ❌ | ❌ | ✅ | ✅ |
| Kompresi file | ❌ Tidak ada | ✅ Ada | ❌ | ❌ | ❌ |

---

## 5. Target Pengguna

Individu yang sering berpindah file antara laptop dan smartphone dalam ekosistem pribadi — terutama:

- **Mahasiswa** — transfer materi kuliah, tugas, presentasi
- **Desainer & kreator konten** — file besar seperti video, PSD, Figma
- **Developer** — build artifact, project file, dataset
- **Pekerja kreatif** — foto, audio, dokumen kerja

Yang menginginkan solusi transfer **simpel, cepat, aman, dan bebas internet.**

---

# BAGIAN II — DESAIN & PENGALAMAN PENGGUNA

---

## 6. Design System

### 6.1 Identitas Visual

| Atribut | Nilai |
|---|---|
| Aesthetic | Clean, Minimal, Premium, Serius |
| Primary Mode | Light |
| Font Utama | **Satoshi** |
| Font Monospace | **DM Mono** |
| Icon Library | **Lucide Icons** |
| Border Radius Card | 14–16px |
| Border Radius Button | 10–12px |
| Border Radius Pill | 99px |

---

### 6.2 Typography Scale

Font utama **Satoshi** — modern, clean, premium, sangat readable di semua ukuran.
Font monospace **DM Mono** — untuk nilai teknis seperti IP, token, dan fingerprint.

| Peran | Size | Weight | Penggunaan |
|---|---|---|---|
| Display | 28px | 700 Bold | Judul onboarding carousel |
| Heading 1 | 22px | 700 Bold | Judul halaman utama |
| Heading 2 | 18px | 600 SemiBold | Sub judul, nama section |
| Heading 3 | 15px | 600 SemiBold | Card title, modal title |
| Body Large | 14px | 400 Regular | Konten utama, deskripsi |
| Body | 13px | 400 Regular | Teks umum, list item |
| Body Small | 12px | 400 Regular | Metadata, hint, caption |
| Label | 11px | 600 SemiBold | Badge, tag, status label uppercase |
| Monospace | 12px | 500 Medium | IP address, token, fingerprint |

**Aturan Typography:**
- Line height body: **1.5×**, heading: **1.2×**
- Letter spacing heading: **-0.3px** (tight, premium)
- Letter spacing label uppercase: **+0.06em**
- Maksimal **2 font weight** dalam satu screen
- Italic hanya untuk tagline/quote, tidak untuk UI element

---

### 6.3 Color Palette

| Peran | Nama | Hex |
|---|---|---|
| **Accent Utama** | **Cobalt Deep** | **`#0045B5`** |
| Accent Hover | — | `#003A99` |
| Accent Light | — | `#EEF4FF` |
| Accent Mid | — | `#BFDBFE` |
| Background | Slate 50 | `#F8FAFC` |
| Surface | White | `#FFFFFF` |
| Surface 2 | Slate 100 | `#F1F5F9` |
| Border | Slate 200 | `#E2E8F0` |
| Border Strong | Slate 300 | `#CBD5E1` |
| Text Primary | Slate 900 | `#0F172A` |
| Text Secondary | Slate 500 | `#64748B` |
| Text Tertiary | Slate 400 | `#94A3B8` |
| Success | Emerald 500 | `#10B981` |
| Success Light | — | `#F0FDF4` |
| Error | Rose 500 | `#F43F5E` |
| Error Light | — | `#FFF1F2` |
| Warning | Amber 500 | `#F59E0B` |
| Warning Light | — | `#FFFBEB` |

**Aturan Warna:**
- Accent `#0045B5` **hanya** untuk aksi utama/CTA, bukan dekoratif
- Tombol destruktif (Tolak, Putuskan) tidak pernah berwarna accent
- Warna sukses hanya untuk state berhasil, bukan estetika
- Contrast ratio minimum **4.5:1** untuk semua teks (WCAG AA)

---

### 6.4 Spacing System

Basis **4px grid** — semua nilai spacing adalah kelipatan 4.

| Token | Value | Penggunaan Umum |
|---|---|---|
| space-1 | 4px | Micro gap, icon margin inline |
| space-2 | 8px | Gap dalam komponen kecil |
| space-3 | 12px | Padding komponen kecil, badge |
| space-4 | 16px | Padding standar, gap antar elemen |
| space-5 | 20px | Padding card, gap section |
| space-6 | 24px | Padding konten utama |
| space-8 | 32px | Gap antar section besar |
| space-10 | 40px | Padding halaman / top bar |

---

### 6.5 Icon System — Lucide Icons

Seluruh icon menggunakan **Lucide Icons** — satu library, konsisten, stroke-based, open source.

| Konteks | Lucide Name |
|---|---|
| Kirim file | `Upload` |
| Terima file | `Download` |
| Hubungkan perangkat | `Link` |
| Putuskan koneksi | `Unlink` |
| WiFi aktif | `Wifi` |
| WiFi tidak aktif | `WifiOff` |
| Perangkat HP | `Smartphone` |
| Perangkat Desktop | `Monitor` |
| QR Code | `QrCode` |
| Scan / Kamera | `Scan` |
| File | `File` |
| Folder | `Folder` |
| Riwayat | `Clock` |
| Settings | `Settings` |
| Profile / User | `User` |
| Sukses | `CheckCircle` |
| Error | `XCircle` |
| Peringatan | `AlertCircle` |
| Keamanan | `Shield` |
| Kecepatan | `Zap` |
| Tutup / Close | `X` |
| Kembali | `ArrowLeft` |
| Refresh | `RefreshCw` |
| Cari | `Search` |
| Filter | `Sliders` |

**Aturan Icon:**
- Stroke width: **1.5px** — tidak terlalu tebal, tidak terlalu tipis
- Size inline: **16px**, navigasi: **20px**, hero/featured: **24px**
- Icon selalu disertai label teks kecuali ikon universal (X, ArrowLeft)
- Warna icon mengikuti warna teks konteksnya
- Tidak mencampur library icon lain

---

### 6.6 Prinsip & Hukum UI/UX

Semua keputusan desain ArSend mengacu pada prinsip dan hukum UI/UX berikut. Setiap prinsip diikat langsung ke implementasi nyata, bukan sekadar teori.

---

#### Hukum Fitts
> *Semakin besar dan dekat sebuah target, semakin mudah diklik/ditap.*

Implementasi:
- Tombol Kirim, Terima, Hubungkan selalu **full-width dan berukuran besar**
- Tombol destruktif (Tolak, Putuskan) lebih kecil dan tidak menonjol
- Approval modal: tombol **"Terima" di kanan** (zona ibu jari kanan), "Tolak" di kiri
- Touch target minimum **44×44px** di semua elemen interaktif mobile

---

#### Hukum Hick
> *Semakin banyak pilihan, semakin lama user memutuskan.*

Implementasi:
- Maksimal **2 aksi utama** per screen (Kirim / Terima)
- Pill switch hanya **2 tab** (Transfer / Riwayat)
- Home tidak menampilkan semua fitur sekaligus
- Filter riwayat hanya **3 opsi** (Semua / Terkirim / Diterima)

---

#### Hukum Miller (7 ± 2)
> *Manusia hanya bisa memproses 7±2 item sekaligus dalam working memory.*

Implementasi:
- Recent Activity maksimal **6–7 item** di home
- Informasi progress dibatasi **3 stats** (kecepatan, terkirim, sisa waktu)
- Device discovery dikelompokkan: trusted / baru

---

#### Jakob's Law
> *User menghabiskan lebih banyak waktu di app lain — mereka prefer app yang bekerja seperti yang sudah mereka kenal.*

Implementasi:
- QR pairing flow mirip **WhatsApp Web** — sudah familiar
- Approval file mirip **AirDrop** — sudah familiar
- Bottom navigation mobile — **standar Android**
- Drag & drop desktop — **standar semua OS**

---

#### Prinsip Proximity
> *Elemen yang berhubungan harus didekatkan, yang tidak berhubungan dijauhkan.*

Implementasi:
- Info WiFi & status koneksi selalu berdekatan di connection bar
- Card Kirim & Terima berdampingan dalam satu grup
- Metadata file (nama, ukuran, waktu) dalam satu card tanpa pemisah berlebihan

---

#### Prinsip Visual Hierarchy
> *Mata user harus diarahkan secara natural dari informasi terpenting ke informasi pendukung.*

```
Level 1 — Paling menonjol (accent #0045B5, ukuran besar):
  → Tombol aksi utama: Kirim, Terima, Hubungkan
  → Status koneksi: connected / disconnected

Level 2 — Penting (Text Primary #0F172A, SemiBold):
  → Nama device terhubung
  → Progress transfer & persentase
  → Nama file

Level 3 — Pendukung (Text Secondary #64748B, Regular):
  → Nama jaringan WiFi
  → Ukuran file, timestamp
  → Kecepatan transfer

Level 4 — Paling samar (Text Tertiary #94A3B8, kecil):
  → Hint text, placeholder
  → Versi app
  → Public key fingerprint
```

---

#### Prinsip Feedback & Visibility
> *User harus selalu tau apa yang sedang terjadi.*

Implementasi:
- Progress bar + persentase + stats real-time saat transfer berlangsung
- Spinner + teks dinamis saat mode menunggu
- Badge status koneksi selalu visible di connection bar
- Notifikasi OS saat app di background
- Banner sukses/gagal setelah setiap aksi selesai

---

#### Prinsip Error Prevention
> *Lebih baik mencegah error daripada memulihkannya.*

Implementasi:
- Approval wajib sebelum terima file — tidak bisa di-bypass
- Konfirmasi TOFU sebelum percaya device baru
- QR memiliki timer expire + auto-refresh — tidak bisa pakai QR kedaluwarsa
- Tombol destruktif tidak pernah berwarna accent

---

#### Prinsip Konsistensi
> *Elemen yang serupa harus berperilaku dan terlihat serupa di seluruh app.*

Implementasi:
- Satu icon library (Lucide) — tidak campur
- Satu font utama (Satoshi) — konsisten semua screen
- Spacing berbasis 4px grid — tidak ada angka ajaib
- Semua card: border radius, padding, shadow yang identik
- Accent `#0045B5` hanya untuk CTA — tidak dekoratif

---

#### Prinsip Accessibility
> *Desain harus bisa digunakan oleh semua orang.*

Implementasi:
- Contrast ratio minimum **4.5:1** untuk semua teks (WCAG AA)
- Icon selalu disertai label teks
- Touch target minimum **44×44px** di mobile
- Tidak mengandalkan warna sebagai satu-satunya indikator status
- Focus state visible untuk keyboard navigation di desktop

---

### 6.7 Layout Desktop

- Window size: **40% lebar layar × 80% tinggi layar** — portrait compact
- **Tanpa sidebar** — navigasi via pill switch
- **Top bar:** logo ArSend (Satoshi Bold 18px) + icon `User` (Lucide, 20px)
- **Connection bar:** dot status + nama jaringan WiFi + nama device terhubung
- **Pill switch:** Transfer / Riwayat
- Padding konten: **24px horizontal, 20px vertical**
- Window tidak resizable (fixed proportion)

---

### 6.8 Layout Mobile

- Full screen
- **Bottom navigation:** icon Lucide + label Satoshi
- Welcome carousel saat first launch (3 slide)
- Padding konten: **20px horizontal, 16px vertical**
- Vibes, komponen, dan warna konsisten dengan desktop

---

## 7. Alur Pengguna (User Flow)

### 7.1 Onboarding

**Desktop:**
```
Buka app pertama kali
      ↓
Langsung masuk Home
QR otomatis ter-generate & tampil
List device discovery aktif
```

**Mobile:**
```
Install & buka pertama kali
      ↓
Slide 1: "Tanpa Internet"
Transfer file tanpa kuota, tanpa cloud
      ↓
Slide 2: "Tanpa Batas Ukuran"
Kirim file sebesar apapun, secepat WiFi-mu
      ↓
Slide 3: "Simpel & Cepat"
Terhubung otomatis, scan QR untuk spesifik
      ↓
Tap "Mulai" → masuk Home
```

---

### 7.2 Koneksi & Pairing

**Mode Auto Discovery:**
```
Buka ArSend di semua device
      ↓
UDP Broadcast otomatis berjalan
      ↓
Device dengan ArSend saling terdeteksi
      ↓
Pilih device dari list
      ↓
TOFU check (pertama kali → konfirmasi)
      ↓
TLS Handshake
      ↓
✅ Terhubung
```

**Mode QR Pairing:**
```
Device A → "Tampilkan QR"
      ↓
QR tampil dengan timer 3 menit
      ↓
Device B scan QR via kamera
      ↓
Token diverifikasi → TLS Handshake
      ↓
✅ Terhubung langsung (spesifik)
```

---

### 7.3 Kirim File

```
Tap/klik "Kirim File"
      ↓
Pilih file (file picker / drag & drop)
Bisa multi-file sekaligus
      ↓
FILE_OFFER dikirim ke penerima
      ↓
Menunggu approval penerima
      ↓
Jika diterima → transfer dimulai
Progress bar + stats real-time
      ↓
✅ Selesai → banner sukses + "Buka Folder"

Jika ditolak → notifikasi "File ditolak oleh [device]"
```

---

### 7.4 Terima File

```
Tap/klik "Terima File"
      ↓
Mode waiting aktif
Teks menunggu dinamis
Auto-timeout 5 menit (bisa dibatalkan)
      ↓
FILE_OFFER masuk → approval popup:
"[Device] ingin mengirim [N] file ([ukuran])"
[Terima] [Tolak]
      ↓
Jika terima → progress transfer
      ↓
✅ Selesai → info file + "Buka di File Manager"

Jika tolak → pengirim dapat notifikasi
```

---

### 7.5 Disconnect

```
Koneksi terputus (heartbeat gagal)
      ↓
Auto reconnect × 3 (interval 5 detik)
      ↓
Berhasil → lanjut seperti biasa
      ↓
Gagal → halaman "Koneksi Terputus"
[Coba Lagi] [Kembali ke Beranda]
```

---

# BAGIAN III — FITUR & SPESIFIKASI

---

## 8. Fitur Lengkap

### 8.1 Home — Belum Terhubung

**Informasi yang tampil:**
- Nama jaringan WiFi aktif
- Nama perangkat ini
- Status: Belum Terhubung

**Desktop:**
- QR Code otomatis ter-generate dan tampil
- Info payload: IP Address, Port, Token (tersensor sebagian)
- Progress bar expire QR (3 menit) + auto-refresh
- List device discovery (kosong jika belum ada ArSend lain)
- Fallback: "Gunakan kode manual"
- Link: "Lihat cara penggunaan"

**Mobile:**
- List device yang terdeteksi di jaringan (auto-discovery)
- Tombol "Scan QR" → kamera aktif
- Fallback: "Gunakan kode manual"
- Link: "Lihat cara penggunaan"

Tombol **Kirim & Terima disembunyikan** sampai pairing berhasil.

---

### 8.2 TOFU — Pertama Kali Terhubung ke Device Baru

Saat pertama kali terhubung ke device yang belum pernah dipercaya:

```
Modal konfirmasi muncul:

"Rull's MacBook Pro ingin terhubung
 Public key: abc1...xyz9
 Percayai perangkat ini?"

[Percaya Sekali]  [Percaya Selalu]  [Tolak]
```

- **Percaya Sekali** — session ini saja, lain kali konfirmasi lagi
- **Percaya Selalu** — simpan ke trust store, tidak muncul lagi
- **Tolak** — koneksi tidak dibentuk

Device yang dipercaya dikelola di Settings → Trust Store.

---

### 8.3 Home — Sudah Terhubung

**Informasi yang tampil:**
- Badge hijau: "Terhubung ke [nama device]"
- Nama jaringan WiFi
- IP Address + indikator kekuatan sinyal

**Tab Transfer:**
- Card **Kirim File**
- Card **Terima File**
- Dropzone drag & drop (desktop)
- Recent Activity (6–7 file terakhir)

**Tab Riwayat:**
- Search bar
- Filter pill: Semua / Terkirim / Diterima
- List lengkap riwayat transfer

**Aksi lain:**
- Tombol "Putuskan Koneksi"

---

### 8.4 Kirim File

- Multi-file — pilih beberapa sekaligus
- Semua format file didukung
- Tidak ada batas ukuran file
- Drag & drop (desktop) + file picker native
- Progress per file + total keseluruhan
- Stats real-time: kecepatan (MB/s), terkirim, sisa waktu
- Tombol batalkan transfer kapanpun
- Selesai → banner sukses + tombol "Buka Folder"
- File ditolak → notifikasi "File ditolak oleh [device]"

---

### 8.5 Terima File

- Mode waiting dengan teks dinamis (tidak statis)
- Auto-timeout 5 menit, bisa dibatalkan manual
- Approval popup wajib saat file masuk
- File otomatis disimpan ke `Documents/ArSend/` (default)
- Folder penyimpanan bisa diubah di Settings
- Selesai → info file (nama, ukuran) + tombol "Buka di File Manager"

---

### 8.6 Riwayat Transfer

- List semua file yang pernah dikirim dan diterima
- Search bar — cari berdasarkan nama file
- Filter: **Semua / Terkirim / Diterima**
- Setiap item menampilkan: nama file, ukuran, arah transfer, waktu, status
- Data riwayat disimpan lokal via `tauri-plugin-store`

---

### 8.7 Notifikasi Background

Ketika app berjalan di background, notifikasi OS native muncul:

| Kondisi | Isi Notifikasi |
|---|---|
| File masuk, menunggu approval | "[Device] ingin mengirim [N] file ([ukuran])" |
| File selesai diterima | "[Nama file] berhasil diterima" |
| File selesai terkirim | "File berhasil terkirim ke [Device]" |
| Koneksi terputus | "Koneksi dengan [Device] terputus" |
| File ditolak penerima | "File ditolak oleh [Device]" |

---

### 8.8 Profile & Settings

**Profile (via avatar di top bar):**
- Nama perangkat — auto-detect dari OS hostname, bisa diedit manual
- Public key fingerprint (untuk verifikasi identitas ke device lain)

**Settings:**
- Folder penyimpanan file yang diterima (default: `Documents/ArSend/`)
- Nama perangkat
- Trust Store — lihat & hapus device yang sudah dipercaya
- Informasi versi aplikasi

---

## 9. Struktur Layar

### 9.1 Mobile

```
App (Mobile)
│
├── Welcome Carousel (3 slide)         — first launch only
│
├── Home
│   ├── Belum Terhubung
│   │   ├── List device discovery
│   │   ├── Tombol "Scan QR"
│   │   ├── Fallback kode manual
│   │   └── Link cara penggunaan
│   │
│   └── Sudah Terhubung
│       ├── Connection bar
│       ├── Tab Transfer
│       │   ├── Card Kirim
│       │   ├── Card Terima
│       │   └── Recent Activity (6–7 item)
│       └── Tab Riwayat
│           ├── Search bar
│           ├── Filter pill
│           └── List riwayat
│
├── TOFU Modal                         — saat connect device baru
├── QR Scanner (fullscreen kamera)
│
├── Kirim
│   ├── File picker
│   ├── Progress transfer (per file + total)
│   └── Selesai (banner + buka file manager)
│
├── Terima
│   ├── Waiting state (teks dinamis)
│   ├── Approval popup
│   ├── Progress transfer
│   └── Selesai (info file + buka file manager)
│
├── Koneksi Terputus
│   └── [Coba Lagi] [Kembali ke Beranda]
│
└── Settings
    ├── Folder penyimpanan
    ├── Nama perangkat
    ├── Trust Store
    └── Versi app
```

---

### 9.2 Desktop

```
App (Desktop) — 40% lebar × 80% tinggi layar
│
├── Home
│   ├── Belum Terhubung
│   │   ├── QR Code (auto-generate, timer 3 menit)
│   │   ├── Info: IP, Port, Token (tersensor)
│   │   ├── List device discovery
│   │   ├── Fallback kode manual
│   │   └── Link cara penggunaan
│   │
│   └── Sudah Terhubung
│       ├── Connection bar
│       ├── Tab Transfer
│       │   ├── Card Kirim
│       │   ├── Card Terima
│       │   ├── Dropzone drag & drop
│       │   └── Recent Activity (6–7 item)
│       └── Tab Riwayat
│           ├── Search bar
│           ├── Filter pill
│           └── List riwayat
│
├── TOFU Modal                         — saat connect device baru
│
├── Kirim
│   ├── Dropzone + file picker (multi-file)
│   ├── Progress transfer (per file + total + stats)
│   └── Selesai (banner + buka folder)
│
├── Terima
│   ├── Waiting state (teks dinamis)
│   ├── Approval popup
│   ├── Progress transfer
│   └── Selesai (info file + buka folder)
│
├── Koneksi Terputus
│   └── [Coba Lagi] [Kembali ke Beranda]
│
└── Profile / Settings
    ├── Nama perangkat
    ├── Public key fingerprint
    ├── Folder penyimpanan
    ├── Trust Store
    └── Versi app
```

---

## 10. Batasan v1.0 (Non-Goals)

| Fitur | Keterangan |
|---|---|
| Dark mode | 🔜 Planned v1.1 |
| Resume transfer setelah restart | 🔜 Planned v1.1 |
| iOS support | 🔜 Planned v2.0 |
| Clipboard sync | 🔜 Planned v2.0 |
| Sinkronisasi folder otomatis | 🔜 Planned v2.0 |
| Transfer via internet / relay server | ❌ Bukan visi produk |

---

# BAGIAN IV — TEKNIS & ARSITEKTUR

---

## 11. Tech Stack

| Layer | Teknologi | Alasan |
|---|---|---|
| Desktop & Mobile Framework | **Tauri V2** | Ringan, native, satu codebase untuk semua platform |
| Frontend / UI | **Svelte 5** | Compile ke vanilla JS, zero runtime overhead, paling ringan |
| Styling | **Tailwind CSS v4** | Utility-first, CSS di-purge, output minimal |
| Backend / Core Logic | **Rust** | Memory safe, blazing fast, ideal untuk I/O intensif |
| Async Runtime | `tokio` | Async I/O foundation seluruh Rust backend |
| Komunikasi Kontrol | `tokio-tungstenite` | WebSocket async |
| Transfer File | `tokio` TCP Stream | Raw bytes chunk-based, no overhead |
| Discovery | UDP Broadcast | Auto-detect device di jaringan lokal |
| TLS / Enkripsi Transport | `rustls` | Zero-cost TLS, tanpa OpenSSL |
| Keypair / Identity | `ed25519-dalek` | Ed25519 keypair unik per device |
| Hashing Integrity | `sha2` | SHA-256 per chunk + per file |
| Nonce / Anti-replay | `rand` | Random nonce setiap pesan |
| QR Generation | `qrcode` | Generate QR dari Rust |
| QR Scanner Mobile | `tauri-plugin-barcode-scanner` | Kamera scanner native Android |
| Network Detection | `local-ip-address` | Detect IP aktif di jaringan |
| Session Token | `uuid` | UUID sekali pakai untuk pairing |
| Serialisasi | `serde_json` | JSON encode/decode |
| Penyimpanan Lokal | `tauri-plugin-store` | Persistent storage lintas platform |
| Notifikasi | `tauri-plugin-notification` | Local notification background |

---

## 12. Platform & Minimum OS

| Platform | Status | Minimum |
|---|---|---|
| Desktop — Windows | ✅ Didukung | Windows 7+ |
| Desktop — macOS | ✅ Didukung | macOS 10.13+ |
| Desktop — Linux | ✅ Didukung | Distro modern |
| Mobile — Android | ✅ Didukung | Android 8.0 (API 26)+ |
| Mobile — iOS | 🔜 v2.0 | — |

**Estimasi ukuran aplikasi:**

| Platform | Estimasi |
|---|---|
| Desktop (.exe Windows) | ~5–9 MB |
| Mobile (.apk Android) | ~5–9 MB |

---

## 13. Arsitektur Sistem

### 13.1 Big Picture

```
┌──────────────────────────────────────────────────────────┐
│                    JARINGAN LOKAL                         │
│            (WiFi yang sama / Hotspot HP)                  │
│                                                           │
│   ┌─────────────┐   UDP Broadcast   ┌─────────────┐     │
│   │   Desktop   │◄─────────────────►│   Mobile    │     │
│   │  Tauri V2   │                   │  Tauri V2   │     │
│   │  Svelte 5   │◄═════════════════►│  Svelte 5   │     │
│   │  Rust Core  │  WebSocket + TCP  │  Rust Core  │     │
│   └─────────────┘       TLS         └─────────────┘     │
│          ▲                                  ▲            │
│          │          UDP Broadcast           │            │
│          └─────────────────────────────────┘            │
│                          ▲                               │
│                          │                               │
│                 ┌─────────────────┐                      │
│                 │  Device Lain N  │                      │
│                 │ (HP/Laptop lain)│                      │
│                 └─────────────────┘                      │
└──────────────────────────────────────────────────────────┘

Multi-device support:
✅ Desktop ↔ Mobile
✅ Desktop ↔ Desktop
✅ Mobile ↔ Mobile
✅ Satu device → Banyak device
```

---

### 13.2 Layer Komunikasi

```
┌──────────────────────────────────────┐
│          SVELTE 5 FRONTEND           │
│   UI · State · User Interaction      │
└────────────────┬─────────────────────┘
                 │ Tauri IPC
                 │ (whitelisted commands only)
                 ▼
┌──────────────────────────────────────┐
│            RUST CORE                 │
│                                      │
│  network.rs      pairing.rs          │
│  server.rs       transfer.rs         │
│  security.rs     notification.rs     │
└──────────┬───────────────────────────┘
           │
    ┌──────┴───────┐
    ▼              ▼
┌──────────┐  ┌──────────┐
│WebSocket │  │   TCP    │
│Port 9527 │  │Port 9528 │
│Signaling │  │File bytes│
│  + TLS   │  │  + TLS   │
└──────────┘  └──────────┘
```

---

### 13.3 Peran Setiap Module Rust

| Module | Tanggung Jawab |
|---|---|
| `network.rs` | Detect IP aktif, kelola UDP broadcast discovery |
| `pairing.rs` | Generate QR payload, session token, TOFU trust store |
| `server.rs` | WebSocket server, TCP server, session & koneksi management |
| `transfer.rs` | Chunk-based file streaming, hash per chunk, hash total, retry |
| `security.rs` | TLS setup via rustls, keypair Ed25519, nonce, sanitasi input |
| `notification.rs` | Trigger local notification OS saat app di background |

---

## 14. Mekanisme Koneksi

### 14.1 Mode 1 — Auto Discovery (UDP Broadcast)

Device secara aktif mem-broadcast kehadirannya di jaringan:

```
Payload UDP Broadcast:
{
  "name": "Rull's Phone",
  "public_key": "abc123...",   ← Ed25519 public key
  "version": "1.0.1",
  "port": 9527
}
```

Flow:
```
ArSend dibuka
      ↓
UDP Broadcast aktif berjalan
      ↓
Device lain reply dengan payload mereka
      ↓
List device muncul di UI (real-time)
      ↓
User pilih device target
      ↓
TOFU check
      ↓
TLS Handshake
      ↓
✅ Terhubung
```

---

### 14.2 Mode 2 — QR Pairing (Spesifik)

Untuk koneksi langsung ke device tertentu:

```
QR Payload:
{
  "ip": "192.168.43.105",
  "port": 9527,
  "token": "a3f9x...",         ← UUID sekali pakai
  "public_key": "abc123...",   ← Ed25519 public key
  "device": "Rull's MacBook Pro"
}
```

Flow:
```
Device A tap "Tampilkan QR"
      ↓
QR ter-generate + timer 3 menit berjalan
      ↓
Device B scan QR via kamera
      ↓
Device B parse payload → kirim token ke Device A
      ↓
Device A validasi token → cocok → token di-invalidate
      ↓
TLS Handshake
      ↓
✅ Terhubung langsung & spesifik
```

---

### 14.3 Multi-Device Support

| Kombinasi | Status |
|---|---|
| Desktop → Mobile | ✅ |
| Mobile → Desktop | ✅ |
| Desktop → Desktop | ✅ |
| Mobile → Mobile | ✅ |
| Satu → Banyak device | ✅ |

---

### 14.4 Protokol Transfer File

```
Pengirim                                Penerima
   │                                        │
   │── FILE_OFFER ──────────────────────►  │
   │   { nama, ukuran, hash_total,          │
   │     jumlah_chunk, nonce }              │
   │                                        │
   │              (approval popup muncul)   │
   │                                        │
   │◄── FILE_ACCEPT / FILE_REJECT ─────────│
   │                                        │
   │   [loop setiap chunk]                  │
   │══ CHUNK { index, data, hash } ════════►│
   │◄── CHUNK_ACK (ok) / CHUNK_RETRY ──────│
   │                                        │
   │◄── FILE_DONE { hash_verify } ─────────│
   │                                        │
   │── TRANSFER_COMPLETE ──────────────────►│
```

---

### 14.5 Disconnect Handling

```
Koneksi terputus
(WebSocket heartbeat tidak ada respons)
      ↓
Auto reconnect
Percobaan 1 → tunggu 5 detik
Percobaan 2 → tunggu 5 detik
Percobaan 3 → tunggu 5 detik
      ↓
Berhasil reconnect → lanjut seperti biasa
      ↓
Semua gagal → Halaman "Koneksi Terputus"
              [Coba Lagi]  [Kembali ke Beranda]
```

---

## 15. Keamanan & Enkripsi

### 15.1 Threat Model

| Skenario Penggunaan | Level Risiko |
|---|---|
| Hotspot pribadi (hanya device sendiri) | 🟢 Rendah |
| WiFi rumah / kantor terpercaya | 🟡 Sedang |
| WiFi publik (kafe, kampus, mall) | 🔴 Tinggi |

ArSend dirancang aman di semua skenario di atas.

---

### 15.2 Security Stack (6 Layer)

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 1 — DISCOVERY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• UDP Broadcast menyertakan Ed25519 public key
• Device tanpa public key valid tidak tampil di list
• Rate limiting broadcast per device
• Mencegah: device asing muncul di list tanpa identitas

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 2 — TRUST & IDENTITY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• Setiap device punya Ed25519 keypair unik
  (generate sekali saat install, disimpan lokal)
• TOFU (Trust on First Use) — konfirmasi manual
  pertama kali connect ke device baru
• QR Pairing menyertakan public key device
• Session token UUID: sekali pakai, TTL 3 menit
• Nonce unik per pesan (mencegah replay attack)
• Mencegah: MITM, identitas palsu, session hijack

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 3 — TRANSPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• Semua komunikasi WebSocket & TCP dibungkus TLS
  menggunakan rustls (zero-cost, no OpenSSL)
• Metadata file (nama, ukuran) ikut terenkripsi
• Mencegah: packet sniffing, man-in-the-middle

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 4 — TRANSFER INTEGRITY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• SHA-256 hash per chunk — chunk corrupt di-retry
• SHA-256 hash keseluruhan file — verifikasi akhir
• Chunk-based streaming — file tidak pernah
  di-load penuh ke RAM (aman untuk file 10GB+)
• Approval wajib di penerima sebelum transfer mulai
• Rate limiting FILE_OFFER per device
• Mencegah: file corrupt, data manipulation, DoS

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 5 — STORAGE SAFETY
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• Sanitasi nama file di Rust (path traversal prevention)
• Reject symlink pada file yang diterima
• Tidak auto-extract archive (ZIP bomb prevention)
• File hanya disimpan ke folder yang diizinkan user
• Mencegah: path traversal, symlink attack, ZIP bomb

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
LAYER 6 — APP LEVEL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• Max concurrent connections per IP
• Whitelist Tauri IPC commands (anti XSS injection)
• Validasi semua input di Rust side
• Log production di-sanitasi (nama file & IP di-mask)
• Mencegah: DoS, XSS ke IPC, data leak via log
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

### 15.3 Impact Keamanan terhadap Performa

| Security Feature | Impact Performa |
|---|---|
| TLS via rustls | ~1–3% overhead — tidak terasa |
| Ed25519 keypair | Generate sekali saat install, ~0ms saat transfer |
| SHA-256 per chunk | ~0.1% overhead — sangat cepat di Rust |
| Nonce per pesan | Bytes tambahan minimal |
| Chunk streaming | **Justru meningkatkan performa** |
| Input validation | Microseconds — tidak terasa |

**Kesimpulan: seluruh security stack ini practically zero-cost terhadap kecepatan transfer.**

---

### 15.4 Crate Security (Rust)

| Kebutuhan | Crate |
|---|---|
| TLS | `rustls` |
| Keypair Ed25519 | `ed25519-dalek` |
| Hashing SHA-256 | `sha2` |
| Nonce & Random | `rand` |
| Session Token | `uuid` |

---

## 16. Estimasi Performa Transfer

Menggunakan **hotspot HP biasa (WiFi 5, ~200 Mbps real)**:

| Ukuran File | Estimasi Waktu |
|---|---|
| 100 MB | ~4 detik |
| 1 GB | ~40 detik |
| 5 GB | ~3 menit |
| 10 GB | ~7 menit |

Kecepatan transfer = kecepatan WiFi lokal.
Tidak ada bottleneck internet, tidak ada kompresi, tidak ada relay server.

---

## 17. Struktur Folder Project

```
arsend/
│
├── src/                                  # Svelte 5 Frontend
│   ├── lib/
│   │   ├── components/
│   │   │   ├── QRDisplay.svelte          # Tampil QR Code + timer
│   │   │   ├── QRScanner.svelte          # Scanner kamera (mobile)
│   │   │   ├── DeviceList.svelte         # List device discovery
│   │   │   ├── ConnectionBar.svelte      # Status koneksi top bar
│   │   │   ├── PillSwitch.svelte         # Tab Transfer / Riwayat
│   │   │   ├── Dropzone.svelte           # Drag & drop area (desktop)
│   │   │   ├── FileProgress.svelte       # Progress bar transfer
│   │   │   ├── FileCard.svelte           # Item di list riwayat
│   │   │   ├── ApprovalModal.svelte      # Popup approval terima file
│   │   │   └── TofuModal.svelte          # Popup trust device baru
│   │   │
│   │   ├── stores/
│   │   │   ├── connection.ts             # State koneksi & discovery
│   │   │   ├── transfer.ts               # State transfer file
│   │   │   └── trust.ts                  # State TOFU trust store
│   │   │
│   │   └── utils/
│   │       ├── format.ts                 # Format ukuran file, waktu
│   │       └── platform.ts               # Detect desktop vs mobile
│   │
│   ├── routes/
│   │   ├── +page.svelte                  # Home
│   │   ├── send/+page.svelte             # Kirim file
│   │   ├── receive/+page.svelte          # Terima file
│   │   ├── history/+page.svelte          # Riwayat
│   │   └── settings/+page.svelte         # Settings & Profile
│   │
│   └── app.css                           # Tailwind base + font import
│
├── src-tauri/                            # Rust Backend
│   ├── src/
│   │   ├── main.rs                       # Entry point
│   │   ├── lib.rs                        # Module declarations
│   │   ├── network.rs                    # IP detect, UDP broadcast
│   │   ├── pairing.rs                    # QR gen, token, TOFU store
│   │   ├── server.rs                     # WebSocket + TCP server
│   │   ├── transfer.rs                   # Chunk stream, hash verify
│   │   ├── security.rs                   # TLS, keypair, nonce, sanitasi
│   │   └── notification.rs               # Background notification
│   │
│   ├── capabilities/
│   │   ├── default.json                  # Desktop capabilities
│   │   └── mobile.json                   # Android capabilities
│   │
│   └── Cargo.toml
│
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tailwind.config.ts
└── tauri.conf.json
```

---

# BAGIAN V — PERENCANAAN

---

## 18. Risiko & Mitigasi

| Risiko | Mitigasi |
|---|---|
| QR kedaluwarsa sebelum di-scan | Auto-refresh QR + progress bar timer visual |
| Koneksi putus di tengah transfer | WebSocket heartbeat + auto reconnect 3× (interval 5 detik) |
| File besar menyebabkan OOM | Chunk-based TCP streaming — file tidak pernah di-load penuh ke RAM |
| Tauri Mobile belum feature parity | Mulai dari fitur core, hindari API experimental |
| Nama device tidak terdeteksi | Fallback ke "Perangkat ArSend" + bisa diedit manual |
| File ditolak tanpa notifikasi | WebSocket event `FILE_REJECT` dikirim ke pengirim |
| MITM saat QR pairing | QR include public key, verifikasi via TLS handshake |
| Replay attack | Nonce unik setiap pesan |
| Path traversal via nama file | Sanitasi nama file di Rust sebelum disimpan |
| ZIP bomb / archive berbahaya | ArSend tidak auto-extract — file simpan as-is |
| DoS spam FILE_OFFER | Rate limiting per IP di Rust server |
| XSS inject ke Tauri IPC | Whitelist command + validasi semua input di Rust |
| Log expose data sensitif | Mask nama file & IP address di production build |
| File corrupt saat transfer | SHA-256 per chunk + retry otomatis + hash final |

---

## 19. Milestone & Phase Development

Setiap phase menghasilkan output yang **bisa langsung ditest** — dari skeleton hingga production-ready.

---

### PHASE 1 — Project Setup & Architecture Foundation
**Goal:** Struktur project berdiri, semua module terhubung, build berjalan di semua platform.

**Tasks:**
- [ ] Init project Tauri V2 + Svelte 5 + Tailwind v4
- [ ] Setup struktur folder sesuai spec (src/ + src-tauri/)
- [ ] Konfigurasi `tauri.conf.json` — window size 40%×80%, window non-resizable
- [ ] Setup `Cargo.toml` dengan semua dependencies Rust
- [ ] Setup Tailwind v4 + import font Satoshi + DM Mono
- [ ] Setup Lucide Icons di Svelte
- [ ] Buat whitelist Tauri IPC command
- [ ] Buat skeleton semua Rust module (main.rs, lib.rs, + 6 module)
- [ ] Buat skeleton semua Svelte route & komponen utama
- [ ] Verifikasi build desktop berjalan (Windows/macOS/Linux)
- [ ] Verifikasi build Android berjalan

**Output yang bisa ditest:** App terbuka di desktop dan Android — UI skeleton tampil, tidak crash.

---

### PHASE 2 — Network & Discovery
**Goal:** Device saling menemukan di jaringan yang sama secara otomatis.

**Tasks:**
- [ ] Implementasi `network.rs` — detect IP aktif di jaringan
- [ ] Generate Ed25519 keypair saat pertama install, simpan ke store
- [ ] Implementasi UDP Broadcast — kirim payload `{ name, public_key, version, port }`
- [ ] Implementasi UDP Listener — terima & parse broadcast dari device lain
- [ ] Filter device: hanya tampilkan yang punya public key valid
- [ ] Svelte store `connection.ts` — state list device terdeteksi
- [ ] UI: list device discovery di home (real-time update)

**Output yang bisa ditest:** Dua device di jaringan yang sama → keduanya muncul di list masing-masing secara otomatis.

---

### PHASE 3 — Security Foundation
**Goal:** Semua komunikasi terenkripsi, identitas device terverifikasi.

**Tasks:**
- [ ] Implementasi `security.rs` — TLS setup via `rustls`
- [ ] Implementasi TOFU trust store — simpan device yang dipercaya ke `tauri-plugin-store`
- [ ] UI: Modal konfirmasi TOFU (Percaya Sekali / Percaya Selalu / Tolak)
- [ ] Implementasi nonce generation — unik per pesan
- [ ] Implementasi session token UUID — generate, validasi, invalidate
- [ ] Test TLS handshake berhasil antar dua device

**Output yang bisa ditest:** Connect dua device → TOFU modal muncul → approve → koneksi TLS terbentuk → device sama tidak muncul modal lagi.

---

### PHASE 4 — QR Pairing
**Goal:** User bisa pair ke device spesifik via QR Code.

**Tasks:**
- [ ] Implementasi `pairing.rs` — generate QR payload lengkap
- [ ] QR payload: `{ ip, port, token, public_key, device_name }`
- [ ] UI Desktop: tampil QR Code + info IP/port/token (sebagian tersensor)
- [ ] UI Desktop: progress bar expire QR (3 menit) + tombol refresh manual
- [ ] UI Mobile: QR Scanner fullscreen via `tauri-plugin-barcode-scanner`
- [ ] Logic: scan QR → parse → kirim token → verifikasi → TLS connect
- [ ] UI: fallback "Gunakan kode manual"
- [ ] UI: link "Lihat cara penggunaan"

**Output yang bisa ditest:** Desktop tampil QR → HP scan → keduanya terhubung langsung ke satu sama lain.

---

### PHASE 5 — WebSocket Server & Session Management
**Goal:** Koneksi persistent antara device, signaling & heartbeat berjalan.

**Tasks:**
- [ ] Implementasi `server.rs` — WebSocket server port 9527 + TLS
- [ ] Implementasi WebSocket client di sisi yang connect
- [ ] Implementasi heartbeat — ping/pong setiap N detik
- [ ] Implementasi session management — track semua device terhubung
- [ ] Implementasi auto reconnect — 3× percobaan, interval 5 detik
- [ ] UI: connection bar (nama device, status dot, sinyal)
- [ ] UI: halaman "Koneksi Terputus" + [Coba Lagi] [Beranda]
- [ ] Emit Tauri event ke Svelte saat status koneksi berubah

**Output yang bisa ditest:** Dua device terhubung → matikan WiFi salah satu → auto reconnect berjalan → jika gagal → halaman koneksi terputus muncul.

---

### PHASE 6 — File Transfer Core
**Goal:** File bisa dikirim dan diterima dengan utuh, aman, dan terverifikasi.

**Tasks:**
- [ ] Implementasi TCP server port 9528 + TLS
- [ ] Implementasi `transfer.rs` — chunk-based file streaming (chunk size: 64KB)
- [ ] Implementasi SHA-256 hash per chunk
- [ ] Implementasi SHA-256 hash keseluruhan file
- [ ] Implementasi `CHUNK_RETRY` jika hash chunk tidak cocok
- [ ] Implementasi protokol lengkap: `FILE_OFFER → FILE_ACCEPT/REJECT → CHUNK → FILE_DONE → TRANSFER_COMPLETE`
- [ ] Implementasi rate limiting `FILE_OFFER` per device
- [ ] Sanitasi nama file di Rust (path traversal prevention)
- [ ] Reject symlink pada file yang diterima
- [ ] Simpan file ke folder yang dikonfigurasi user

**Output yang bisa ditest:** Kirim file dari desktop ke HP → file diterima utuh → hash total cocok → file tersimpan di `Documents/ArSend/`.

---

### PHASE 7 — UI Transfer Desktop
**Goal:** UI desktop untuk kirim dan terima file fully functional.

**Tasks:**
- [ ] UI: Card Kirim + Card Terima di tab Transfer
- [ ] UI: Dropzone drag & drop + tombol "Pilih File" (multi-file)
- [ ] UI: Progress bar per file + total + stats real-time (kecepatan, terkirim, sisa waktu)
- [ ] UI: Shimmer animation pada progress bar
- [ ] UI: Approval modal saat ada `FILE_OFFER` masuk
- [ ] UI: Waiting state "Terima File" dengan teks dinamis
- [ ] UI: Banner sukses setelah transfer + tombol "Buka Folder"
- [ ] UI: Notifikasi "File ditolak oleh [device]" di sisi pengirim
- [ ] UI: Tombol batalkan transfer

**Output yang bisa ditest:** Transfer file end-to-end desktop ↔ desktop dengan UI lengkap dan semua state berjalan.

---

### PHASE 8 — UI Transfer Mobile
**Goal:** UI mobile untuk kirim dan terima file fully functional.

**Tasks:**
- [ ] UI: Welcome carousel 3 slide (first launch only)
- [ ] UI: Home mobile — discovery list + tombol "Scan QR"
- [ ] UI: Card Kirim + Card Terima
- [ ] UI: File picker native Android (multi-file)
- [ ] UI: Progress transfer per file + total
- [ ] UI: Approval modal
- [ ] UI: Waiting state dengan teks dinamis
- [ ] UI: Selesai — info file + tombol "Buka di File Manager"
- [ ] Platform detection — beda behavior & layout desktop vs mobile

**Output yang bisa ditest:** Transfer file end-to-end mobile ↔ desktop dan mobile ↔ mobile dengan UI lengkap.

---

### PHASE 9 — Riwayat, Settings & Profile
**Goal:** Fitur pendukung lengkap dan data persisten.

**Tasks:**
- [ ] UI: Tab Riwayat — list semua riwayat transfer
- [ ] UI: Search bar — cari berdasarkan nama file
- [ ] UI: Filter pill — Semua / Terkirim / Diterima
- [ ] Simpan & baca riwayat dari `tauri-plugin-store`
- [ ] UI: Settings — folder penyimpanan, nama device, versi app
- [ ] UI: Profile — nama device, public key fingerprint
- [ ] UI: Trust Store management — lihat & hapus device dipercaya
- [ ] Auto-detect nama device dari OS hostname
- [ ] Edit nama device manual

**Output yang bisa ditest:** Beberapa file ditransfer → cek riwayat → filter → search → ubah settings → restart app → data tetap ada.

---

### PHASE 10 — Background Notification
**Goal:** Notifikasi OS muncul saat app di background, semua platform.

**Tasks:**
- [ ] Setup `tauri-plugin-notification` di desktop & Android
- [ ] Trigger notifikasi: file masuk (approval needed)
- [ ] Trigger notifikasi: file selesai diterima
- [ ] Trigger notifikasi: file selesai terkirim
- [ ] Trigger notifikasi: koneksi terputus
- [ ] Trigger notifikasi: file ditolak penerima
- [ ] Test di Windows, macOS, Linux
- [ ] Test di Android

**Output yang bisa ditest:** App diminimalkan → kirim file dari device lain → notifikasi muncul di OS → tap notifikasi → app terbuka ke state yang relevan.

---

### PHASE 11 — Polish, Optimasi & Edge Cases
**Goal:** App production-ready, tidak ada edge case yang tidak di-handle.

**Tasks:**
- [ ] Sanitasi log production — mask nama file & IP address
- [ ] Implementasi max concurrent connections per IP
- [ ] Test file sangat besar (10GB+)
- [ ] Test koneksi tidak stabil / sinyal lemah
- [ ] Test kirim multi-file besar sekaligus
- [ ] Test semua kombinasi: desktop↔desktop, mobile↔mobile, desktop↔mobile
- [ ] Test TOFU flow lengkap (percaya sekali / percaya selalu / tolak)
- [ ] Test QR expire & refresh
- [ ] Test disconnect & reconnect di tengah transfer aktif
- [ ] Test file ditolak di tengah antrian multi-file
- [ ] Optimasi bundle size Svelte
- [ ] Optimasi Rust binary (strip debug symbols di production)
- [ ] Review semua whitelist Tauri IPC command
- [ ] Final security review semua 6 layer

**Output yang bisa ditest:** Semua skenario ekstrim berjalan tanpa crash, UI tetap responsif.

---

### PHASE 12 — Packaging & Distribution
**Goal:** App siap diinstal dan didistribusikan di semua platform.

**Tasks:**
- [ ] Build & packaging Windows (.exe / .msi)
- [ ] Build & packaging macOS (.dmg)
- [ ] Build & packaging Linux (.AppImage / .deb)
- [ ] Build & packaging Android (.apk)
- [ ] Test install fresh di masing-masing platform
- [ ] Verifikasi first launch experience (onboarding berjalan benar)
- [ ] Verifikasi ukuran binary final sesuai estimasi (~5–9 MB)
- [ ] Dokumentasi cara install & penggunaan dasar

**Output yang bisa ditest:** Install dari nol di setiap platform → app berjalan sempurna dari first launch.

---

### Ringkasan Phase

| Phase | Fokus | Output yang Bisa Ditest |
|---|---|---|
| 1 | Setup & Architecture | App terbuka di semua platform |
| 2 | Network & Discovery | Device saling ditemukan otomatis |
| 3 | Security Foundation | Koneksi TLS + TOFU berjalan |
| 4 | QR Pairing | Pair via QR berhasil |
| 5 | WebSocket & Session | Koneksi persistent + disconnect handled |
| 6 | File Transfer Core | File terkirim utuh & terverifikasi hash |
| 7 | UI Desktop | Full transfer UI desktop |
| 8 | UI Mobile | Full transfer UI mobile |
| 9 | Riwayat & Settings | Data persisten, fitur pendukung lengkap |
| 10 | Notifikasi Background | Notifikasi OS di semua platform |
| 11 | Polish & Edge Cases | App production-ready |
| 12 | Packaging | App siap distribusi |

---

*PRD_ArSend_V1.0.1 — Dokumen ini adalah acuan utama pengembangan ArSend.*
*Setiap perubahan signifikan harus didokumentasikan sebagai revisi PRD baru.*
