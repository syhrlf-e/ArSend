# Updated Product Requirements Document
# ArSend - Local File Transfer App

**Versi PRD:** Update V1.0.1  
**Tanggal Update:** 16 Mei 2026  
**Basis Analisis:** Kondisi repository ArSend saat ini  
**Dokumen Sumber:** `context/prd/PRD_ArSend_V1.0.1.md`  
**Status:** Working PRD - disesuaikan dengan implementasi aktual

---

## 1. Ringkasan Update

Dokumen ini adalah pembaruan PRD ArSend V1.0.1 berdasarkan analisis langsung terhadap struktur proyek, frontend Svelte, backend Rust/Tauri, konfigurasi desktop/mobile, README, dan audit backend transfer yang ada di repository.

PRD awal tetap menjadi dasar visi produk, tetapi implementasi saat ini sudah mendapatkan beberapa penyesuaian penting:

- Discovery jaringan tidak lagi berbasis UDP broadcast manual, melainkan memakai `mDNS` melalui crate `mdns-sd`.
- Identitas perangkat memakai self-signed TLS certificate fingerprint, bukan keypair Ed25519 murni sebagai public identity utama.
- Transfer file memakai pemisahan control plane dan data plane: WebSocket TLS di port `9527`, TCP TLS untuk stream file di port `9528`.
- Chunk transfer sudah memakai ukuran `2MB`, bukan `64KB`.
- Integritas transfer saat ini memakai final SHA-256 hash, bukan hash per chunk + retry per chunk.
- Progress event sudah di-rate-limit secara praktis lewat reporter interval sekitar `150ms`.
- Frontend utama terkonsolidasi di `src/routes/+page.svelte`; route `send`, `receive`, dan sebagian route terpisah masih placeholder.
- Mobile onboarding, QR scan, TOFU modal, history, settings, dan trust store sudah tersedia secara fungsional.
- Beberapa detail UI dan keamanan dari PRD awal belum fully implemented atau perlu dirapikan sebelum production-ready.

---

## 2. Status Produk Saat Ini

ArSend saat ini adalah aplikasi transfer file lokal lintas Windows desktop dan Android berbasis Tauri v2 + Svelte 5. App sudah memiliki fondasi transfer end-to-end lokal:

- perangkat dapat ditemukan melalui mDNS;
- perangkat dapat dipasangkan via list discovery atau QR;
- koneksi signaling memakai WebSocket over TLS;
- transfer data memakai TCP stream over TLS;
- penerima wajib memberi approval sebelum transfer;
- file disimpan ke folder default `Documents/ArSend` atau folder yang dikonfigurasi user;
- riwayat transfer disimpan lokal via `tauri-plugin-store`;
- settings menyimpan nama device, folder unduhan, onboarding state, dan trusted devices.

Target status produk saat ini lebih tepat disebut **functional alpha menuju beta**, bukan lagi "siap development" seperti PRD awal, tetapi juga belum layak disebut final production karena masih ada gap penting pada testing, packaging, reconnect, hardening security, dan polish UI.

---

## 3. Scope Platform

### Platform Aktif

| Platform | Status | Catatan |
|---|---|---|
| Windows Desktop | Aktif | Konfigurasi Tauri window 500x900, non-resizable, transparent |
| Android | Aktif sebagian | Struktur Tauri Android ada; barcode scanner dan JNI content URI handling tersedia |
| macOS | Belum prioritas | Bundle target `all`, tetapi belum ada bukti testing macOS |
| Linux | Belum prioritas | Bundle target `all`, tetapi belum ada bukti testing Linux |
| iOS | Di luar scope V1.0.1 | Roadmap masa depan |

### Minimum OS

| Platform | Requirement |
|---|---|
| Windows | Windows 10/11 dengan WebView2 |
| Android | Android 7.0+ disarankan |

---

## 4. Tech Stack Aktual

### Frontend

| Area | Teknologi |
|---|---|
| Framework | Svelte 5 + SvelteKit |
| Styling | Tailwind CSS v4 |
| Icon | `lucide-svelte` |
| State | Svelte writable stores |
| Storage frontend | `@tauri-apps/plugin-store` |
| File picker | `@tauri-apps/plugin-dialog` |
| QR scanner | `@tauri-apps/plugin-barcode-scanner` untuk mobile |
| Notification | `@tauri-apps/plugin-notification` |
| OS utilities | `@tauri-apps/plugin-os` |

### Backend

| Area | Teknologi |
|---|---|
| App runtime | Tauri v2 |
| Async runtime | Tokio |
| Discovery | `mdns-sd` |
| Control channel | WebSocket over TLS via `tokio-tungstenite` + `tokio-rustls` |
| Data channel | Raw TCP over TLS via `tokio::net::TcpListener/TcpStream` + `tokio-rustls` |
| TLS certificate | Self-signed certificate via `rcgen` |
| Identity fingerprint | SHA-256 hash dari certificate DER |
| File integrity | SHA-256 final hash |
| QR generation | `qrcode` |
| Session token | `uuid` |
| Local IP | `local-ip-address` |
| Android file stream | JNI + ContentResolver untuk `content://` URI |

---

## 5. Arsitektur Aktual

```text
Svelte UI
  |
  | Tauri invoke/event
  v
Rust Backend
  |
  +-- network.rs
  |   +-- get_local_ip
  |   +-- start_discovery via mDNS
  |
  +-- pairing.rs
  |   +-- QR payload
  |   +-- one-time UUID token TTL 3 menit
  |   +-- trust store
  |
  +-- security.rs
  |   +-- local TLS certificate identity
  |   +-- fingerprint verifier
  |
  +-- server.rs
  |   +-- WebSocket TLS server/client port 9527
  |   +-- identity message
  |   +-- heartbeat
  |   +-- FILE_OFFER / ACCEPT / REJECT signaling
  |
  +-- transfer.rs
  |   +-- TCP TLS transfer server port 9528
  |   +-- 2MB chunk stream
  |   +-- final SHA-256 verification
  |   +-- progress event reporter
  |   +-- cancel transfer
  |
  +-- notification.rs
      +-- transfer complete notification
      +-- connection notification
```

### Keputusan Arsitektur Yang Direvisi

| Area | PRD Awal | Implementasi Aktual | Status Update |
|---|---|---|---|
| Discovery | UDP broadcast | mDNS service discovery | PRD direvisi mengikuti mDNS |
| Identity | Ed25519 public key | TLS certificate fingerprint | PRD direvisi mengikuti fingerprint saat ini |
| Chunk size | 64KB | 2MB | PRD direvisi ke 2MB |
| Chunk integrity | SHA-256 per chunk + retry | Final SHA-256 hash | PRD direvisi mengikuti stream sederhana |
| Transfer protocol | Chunk message protocol detail | Header JSON awal + raw bytes + final hash | PRD direvisi |
| Notifikasi | Semua kondisi background | Connection + transfer complete dasar | Checklist disesuaikan |
| Auto reconnect | 3x otomatis | Manual "Coba Lagi" setelah disconnect | Gap |

---

## 6. Fitur Produk Aktual

### 6.1 Discovery Perangkat

Status: **Implemented sebagian besar**

Discovery berjalan melalui service type `_arsend._tcp.local.`. Payload berisi:

- `name`
- `public_key` berupa fingerprint certificate
- `version`
- `port`
- `device_type`

Frontend menerima event:

- `device-discovered`
- `device-removed`

Catatan:

- Filter ghost device sudah ada di frontend.
- Perangkat sendiri diabaikan berdasarkan IP.
- Tidak ada rate limiting eksplisit seperti PRD awal karena mDNS menggantikan UDP broadcast manual.

### 6.2 QR Pairing

Status: **Implemented**

QR payload aktual:

```json
{
  "ip": "local-ip",
  "port": 9527,
  "token": "uuid-once",
  "public_key": "certificate-fingerprint",
  "device_name": "device-name"
}
```

Ketentuan:

- Token UUID berlaku 180 detik.
- Token disimpan di memory dan di-invalidate setelah validasi.
- Desktop menampilkan QR, IP, port, token tersensor, timer, dan refresh manual.
- Mobile dapat scan QR via plugin barcode scanner.

Gap:

- Fallback "Gunakan kode manual" masih tombol UI tanpa flow manual lengkap.
- QR auto-refresh setelah expired belum otomatis generate baru; user menekan refresh/perbarui.

### 6.3 TOFU Trust Store

Status: **Implemented**

Modal TOFU menyediakan:

- Percaya Selalu
- Percaya Sekali
- Tolak

Trusted devices disimpan ke `arsend_trust.json` melalui `tauri-plugin-store`.

Gap:

- Fingerprint ditampilkan pendek di settings, tetapi modal TOFU saat ini lebih menekankan nama perangkat; public key detail belum ditampilkan penuh seperti PRD awal.
- Trust model mengikuti certificate fingerprint, bukan Ed25519 public key.

### 6.4 Koneksi Signaling

Status: **Implemented**

Control channel:

- WebSocket over TLS.
- Server listen di port `9527`.
- Client connect ke IP target.
- TLS diverifikasi dengan fingerprint certificate target.
- Pesan signaling memakai enum `WsMessage`.

Pesan tersedia:

- `Identity`
- `Heartbeat`
- `FileOffer`
- `FileAccept`
- `FileReject`

Gap:

- Heartbeat ada, tetapi auto reconnect 3x interval 5 detik belum implemented.
- Session hanya memegang satu active connection; multi-device simultaneous connection belum tersedia.

### 6.5 Transfer File

Status: **Implemented**

Data channel:

- TCP server port `9528`.
- TLS acceptor/connector.
- Chunk buffer `2MB`.
- File dibaca streaming, tidak diload penuh ke RAM.
- Progress sender/receiver dikirim ke frontend.
- Final SHA-256 hash dikirim dan diverifikasi receiver.
- File name disanitasi via `Path::file_name`.
- Symlink target ditolak pada sisi receiver.
- Transfer dapat dibatalkan.

Gap:

- `send_file_offer` belum menghitung `hash_total`; field dikirim kosong.
- Tidak ada SHA-256 per chunk.
- Tidak ada retry chunk.
- Tidak ada rate limiting `FILE_OFFER`.
- Tidak ada resume transfer.
- Transfer folder rekursif belum support.

### 6.6 Approval Penerima

Status: **Implemented**

Flow:

1. Sender mengirim `FileOffer`.
2. Receiver mendapat `file-offer-received`.
3. UI menampilkan `ApprovalModal`.
4. Receiver accept/reject.
5. Sender mulai `send_file` hanya setelah accept.

Gap:

- Notifikasi OS untuk file masuk menunggu approval belum terlihat di backend.
- Reject menghapus pending transfer, tetapi feedback visual untuk sender masih minimal.

### 6.7 History

Status: **Implemented**

History tersimpan di `arsend_history.json`. Fitur tersedia:

- list riwayat;
- deduplicate saat load;
- tambah item setelah transfer complete;
- hapus item;
- clear semua;
- search nama file;
- filter Semua/Terkirim/Diterima.

Gap:

- History baru dicatat untuk transfer complete success.
- Status failed/cancelled sudah ada di tipe data, tetapi belum dicatat otomatis dari `transfer-error`.

### 6.8 Settings & Profile

Status: **Implemented**

Settings menyimpan:

- nama perangkat;
- folder unduhan;
- onboarding completion;
- trusted devices;
- versi app;
- fingerprint publik.

Gap:

- Ada `settings/+page.svelte` dan `SettingsTab.svelte` dengan sebagian UI mirip/duplikatif.
- Profile di home memakai `SettingsTab`; route settings terpisah juga ada.

### 6.9 Mobile Experience

Status: **Implemented sebagian**

Fitur mobile:

- onboarding carousel first launch;
- bottom navigation;
- QR scanner;
- tombol floating untuk pilih file;
- receive standby;
- Android content URI handling di Rust.

Gap:

- Android build output menunjukkan proses build pernah berjalan, tetapi ada catatan panic di `src-tauri/gen/android/build_output.txt`; perlu verifikasi ulang build Android bersih.
- Parity desktop-mobile perlu diuji end-to-end di device fisik.

### 6.10 Notification

Status: **Implemented dasar**

Backend notification saat ini:

- transfer complete;
- device connected.

Gap:

- File incoming waiting approval belum ada notification.
- File rejected belum ada notification.
- Connection lost belum ada notification.
- Deep link/tap notification ke state relevan belum ada.

---

## 7. Design System Aktual

PRD awal masih relevan untuk arah visual: clean, minimal, premium, light-first, Satoshi, Cobalt Deep `#0045B5`, slate-based neutral palette, dan Lucide icons.

Penyesuaian aktual:

- UI memakai Tailwind v4 token dari `src/app.css`.
- Font Satoshi tersedia sebagai asset lokal.
- DM Mono tersedia dari package `@fontsource/dm-mono`.
- Komponen utama sudah menggunakan rounded besar `16px-28px` di beberapa modal/card.
- Sebagian ikon masih inline SVG manual, belum sepenuhnya Lucide.
- Ada dukungan mode visual day/night disebut README, tetapi implementasi dark mode global belum jelas dari kode utama.

Catatan polish:

- Terdapat teks tidak sengaja `oke` setelah komponen `BottomNav` di `src/routes/+page.svelte`.
- `Dropzone.svelte` sudah ada, tetapi belum digunakan di home transfer flow.
- Route `send/+page.svelte` dan `receive/+page.svelte` masih placeholder.

---

## 8. Batasan V1.0.1 Yang Direvisi

V1.0.1 difokuskan pada transfer file lokal personal antar satu koneksi aktif.

Tidak termasuk scope V1.0.1:

- multi-device simultaneous transfer;
- resume transfer setelah koneksi putus;
- transfer folder rekursif;
- iOS;
- server cloud/relay;
- internet transfer;
- user account/login;
- sync history antar perangkat;
- end-to-end encryption berbasis identity PKI formal selain TLS fingerprint lokal;
- thumbnail/preview file;
- chat/pesan teks.

---

## 9. Risiko & Mitigasi Aktual

| Risiko | Status Saat Ini | Mitigasi Lanjutan |
|---|---|---|
| mDNS tidak stabil di jaringan tertentu | Discovery memakai mDNS | Tambahkan fallback manual code/IP |
| QR expired sebelum dipakai | Token TTL 3 menit + UI timer | Tambahkan auto-regenerate opsional |
| Koneksi putus saat transfer | Disconnect state + Coba Lagi manual | Implement auto reconnect 3x dan resume di roadmap |
| File corrupt | Final SHA-256 verification | Pertahankan final hash; tampilkan error jelas |
| File besar membuat UI lag | Progress reporter 150ms | Pertahankan rate limit, audit IPC load |
| Path traversal | `file_name` sanitization | Tambahkan collision handling dan reserved name handling |
| Symlink attack | Reject symlink di output path | Tambahkan canonical path validation |
| Spam FILE_OFFER | Belum ada rate limit | Tambahkan rate limit per fingerprint/IP |
| Android SAF edge case | JNI content URI handling ada | Test multi-vendor Android fisik |
| Trust salah perangkat | TOFU ada | Tampilkan fingerprint lebih jelas di modal |
| Build Android gagal | Ada build output dengan panic | Jadikan verifikasi Android sebagai checklist blocking |
| CSP null | `csp: null` | Review CSP sebelum production |

---

## 10. Checklist Status Development

Legenda:

- `[x]` selesai/terimplementasi di repository
- `[~]` sebagian selesai atau perlu verifikasi/polish
- `[ ]` belum selesai

### Phase 1 - Project Setup & Foundation

- [x] Init Tauri v2 + Svelte 5
- [x] Setup Tailwind CSS v4
- [x] Setup package scripts dev/build/check/tauri
- [x] Setup struktur `src/` dan `src-tauri/`
- [x] Setup module Rust: `network`, `pairing`, `server`, `transfer`, `security`, `notification`
- [x] Setup capabilities desktop dan mobile
- [x] Setup plugins Tauri store/dialog/fs/notification/process/os/opener
- [x] Setup barcode scanner mobile
- [x] Setup window desktop 500x900 non-resizable
- [~] Verifikasi build desktop lintas OS
- [~] Verifikasi build Android bersih

### Phase 2 - Network & Discovery

- [x] Detect local IP
- [x] Implement mDNS broadcaster
- [x] Implement mDNS browser/listener
- [x] Payload discovery berisi name/fingerprint/version/port/device_type
- [x] Emit `device-discovered` ke Svelte
- [x] Emit `device-removed` ke Svelte
- [x] UI discovery list
- [x] Filter ghost device di frontend
- [ ] Fallback manual connect/code

### Phase 3 - Security Foundation

- [x] Generate local TLS identity sekali dan simpan ke store
- [x] Self-signed certificate via `rcgen`
- [x] Fingerprint SHA-256 certificate
- [x] TLS server config
- [x] TLS client config dengan fingerprint verifier
- [x] TOFU trust store
- [x] UI TOFU modal
- [x] Trust always / trust once / reject
- [~] Nonce utility tersedia, tetapi belum digunakan konsisten di semua message
- [ ] CSP production hardening
- [ ] Sanitized production logging

### Phase 4 - QR Pairing

- [x] Generate QR payload lengkap
- [x] UUID session token
- [x] Token TTL 3 menit
- [x] Token one-time invalidate saat validasi
- [x] Generate QR SVG dari Rust
- [x] UI QR desktop dengan IP/port/token tersensor
- [x] Timer expire QR
- [x] Refresh manual QR
- [x] Mobile QR scanner
- [x] Connect dari QR payload
- [~] Tombol manual code ada, flow belum ada
- [ ] Auto-regenerate QR setelah expired

### Phase 5 - WebSocket & Session Management

- [x] WebSocket server port 9527 over TLS
- [x] WebSocket client connect over TLS
- [x] Identity exchange
- [x] Heartbeat 5 detik
- [x] Active connection state
- [x] Emit `connection-state-changed`
- [x] Disconnect manual
- [x] Halaman/state koneksi terputus
- [~] Tombol Coba Lagi manual
- [ ] Auto reconnect 3x interval 5 detik
- [ ] Multi-device simultaneous connection

### Phase 6 - File Transfer Core

- [x] TCP transfer server port 9528 over TLS
- [x] TCP transfer client over TLS
- [x] Chunk streaming 2MB
- [x] Sender streaming read
- [x] Receiver streaming write
- [x] Final SHA-256 verification
- [x] Progress sender/receiver
- [x] Progress reporter interval sekitar 150ms
- [x] Approval protocol `FILE_OFFER -> ACCEPT/REJECT -> send_file`
- [x] Cancel transfer
- [x] Default save folder `Documents/ArSend`
- [x] Configurable download folder
- [x] Android content URI handling
- [x] Filename sanitization basic
- [x] Reject symlink output path
- [ ] Hash per chunk
- [ ] Chunk retry
- [ ] Rate limiting FILE_OFFER
- [ ] Collision handling untuk nama file sama
- [ ] Resume transfer

### Phase 7 - UI Transfer Desktop

- [x] Card Kirim dan Terima di tab Transfer
- [x] File picker multi-file
- [x] Approval modal
- [x] Receive standby dengan timeout 5 menit
- [x] Progress per file
- [x] Stats progress dasar: speed, sent bytes, total bytes
- [x] Cancel button via `FileProgress`
- [x] History setelah transfer complete
- [~] Dropzone component tersedia, belum terintegrasi
- [~] Notifikasi reject sender masih minimal
- [ ] Banner sukses/gagal khusus
- [ ] Tombol buka folder/file manager setelah selesai

### Phase 8 - UI Transfer Mobile

- [x] Welcome carousel first launch
- [x] Bottom navigation
- [x] QR scanner trigger
- [x] Mobile send flow via floating action button
- [x] Mobile receive standby
- [x] Mobile approval modal
- [x] Platform detection
- [~] End-to-end Android device test perlu diverifikasi
- [ ] Full mobile parity test

### Phase 9 - Riwayat, Settings & Profile

- [x] Store history lokal
- [x] List riwayat
- [x] Search history
- [x] Filter Semua/Terkirim/Diterima
- [x] Clear history
- [x] Delete history item
- [x] Settings nama device
- [x] Auto hostname fallback
- [x] Settings folder unduhan
- [x] Trust Store management
- [x] Public fingerprint display
- [~] Failed/cancelled history status belum otomatis dicatat
- [~] Settings route dan SettingsTab perlu dirapikan agar tidak duplikatif

### Phase 10 - Background Notification

- [x] Plugin notification terpasang
- [x] Request permission di frontend
- [x] Notify connected
- [x] Notify transfer complete
- [ ] Notify incoming file waiting approval
- [ ] Notify file rejected
- [ ] Notify connection lost
- [ ] Tap notification menuju state relevan
- [ ] Test Windows/macOS/Linux/Android

### Phase 11 - Polish, Optimasi & Edge Cases

- [x] HMR guard untuk port already in use
- [x] Buffer transfer reusable 2MB
- [x] Progress emit tidak per byte/chunk terlalu rapat
- [~] Audit backend transfer tersedia
- [~] Animation components tersedia, integrasi perlu diverifikasi
- [ ] Hapus teks/debug UI tidak sengaja
- [ ] Integrasi Dropzone
- [ ] Final accessibility pass
- [ ] Final responsive pass desktop/mobile
- [ ] Test file besar 10GB+
- [ ] Test jaringan tidak stabil
- [ ] Test multi-file besar
- [ ] Test disconnect saat transfer
- [ ] Test trust once/trust always/reject lengkap
- [ ] Review whitelist IPC dan CSP

### Phase 12 - Packaging & Distribution

- [x] Konfigurasi bundle Tauri aktif
- [x] Icon tersedia
- [~] Android generated project tersedia
- [ ] Build Windows installer final
- [ ] Build Android APK final
- [ ] Build macOS/Linux jika tetap ditargetkan
- [ ] Test install fresh
- [ ] Dokumentasi install final

---

## 11. Prioritas Lanjutan

### P0 - Wajib Sebelum Beta

- Hapus teks tidak sengaja `oke` di home mobile.
- Integrasikan `Dropzone.svelte` atau hapus dari scope V1.0.1 agar PRD dan UI tidak bertabrakan.
- Catat `transfer-error` ke history sebagai failed/cancelled.
- Tambahkan notification untuk incoming file offer dan connection lost.
- Verifikasi ulang `npm run check`, `npm run build`, dan build Tauri desktop.
- Verifikasi Android build bersih setelah catatan panic di build output.
- Review CSP `null` dan permissions sebelum release.

### P1 - Production Readiness

- Implement auto reconnect 3x interval 5 detik.
- Tambahkan fallback manual connect/code untuk jaringan yang tidak mendukung mDNS.
- Tambahkan rate limiting `FILE_OFFER`.
- Tambahkan collision handling untuk file dengan nama sama.
- Tampilkan fingerprint lebih jelas di TOFU modal.
- Rapikan duplikasi settings route dan settings tab.
- Tambahkan test end-to-end desktop ke Android.

### P2 - Roadmap Setelah V1.0.1

- Resume transfer.
- Transfer folder rekursif.
- Multi-device simultaneous transfer.
- BLAKE3 sebagai opsi hash performa tinggi.
- Socket buffer tuning.
- Backpressure pipeline network-to-disk.
- macOS/Linux/iOS expansion.

---

## 12. Acceptance Criteria V1.0.1 Revisi

V1.0.1 dianggap memenuhi target jika:

- Dua perangkat ArSend di jaringan lokal dapat saling menemukan melalui mDNS.
- User dapat pairing via discovery list dengan TOFU.
- User dapat pairing via QR dengan token 3 menit.
- File bisa dikirim desktop ke desktop dan desktop ke Android minimal satu arah yang tervalidasi.
- Penerima selalu mendapatkan approval modal sebelum file diterima.
- Progress transfer tampil dengan persentase, byte terkirim, total byte, dan speed.
- File hasil transfer lolos final SHA-256 verification.
- File tersimpan ke folder default atau folder custom yang dipilih user.
- Riwayat sukses/failed/cancelled tercatat lokal.
- Settings nama device, folder download, dan trust store persisten setelah restart.
- Disconnect menampilkan state jelas dan user bisa retry manual.
- Build desktop sukses.
- Build Android sukses atau Android dinyatakan beta-limited secara eksplisit.

---

## 13. Catatan Implementasi Penting

Beberapa poin PRD awal sengaja direvisi karena implementasi aktual lebih masuk akal untuk produk:

- `mDNS` lebih tepat daripada UDP broadcast untuk discovery modern dan cross-platform.
- Final hash verification cukup untuk fase V1.0.1 karena TCP + TLS sudah menyediakan reliability dan integrity di transport layer.
- Chunk `2MB` lebih cocok untuk throughput LAN dibanding `64KB`.
- Control plane dan data plane yang terpisah adalah keputusan arsitektur yang perlu dipertahankan.
- Fingerprint certificate sebagai identity dapat diteruskan untuk V1.0.1, tetapi istilah PRD harus konsisten agar tidak menyebut Ed25519 jika belum menjadi identity utama.

---

## 14. Struktur Folder Aktual

```text
ArSend/
├── context/
│   ├── audit/
│   │   └── audit_v_1_0_0_arsend_backend_transfer_review.md
│   └── prd/
│       ├── PRD_ArSend_V1.0.1.md
│       ├── ArSend_Animation_System_V1.0.0.md
│       └── UPDATE_PRD_V1.0.1.md
├── src/
│   ├── lib/
│   │   ├── assets/
│   │   ├── components/
│   │   ├── stores/
│   │   └── utils/
│   ├── routes/
│   │   ├── +page.svelte
│   │   ├── history/+page.svelte
│   │   ├── receive/+page.svelte
│   │   ├── send/+page.svelte
│   │   └── settings/+page.svelte
│   ├── app.css
│   └── app.html
├── src-tauri/
│   ├── capabilities/
│   ├── gen/android/
│   ├── icons/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── network.rs
│   │   ├── notification.rs
│   │   ├── pairing.rs
│   │   ├── security.rs
│   │   ├── server.rs
│   │   └── transfer.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.js
```

---

## 15. Kesimpulan

ArSend V1.0.1 sudah bergerak cukup jauh dari PRD awal dan kini memiliki fondasi produk nyata: discovery, pairing, TLS, transfer stream, approval, history, settings, notification dasar, dan mobile awareness.

Dokumen PRD update ini menetapkan implementasi aktual sebagai acuan baru. Setelah dokumen ini, pengembangan sebaiknya tidak lagi mengejar PRD lama secara literal, tetapi menyelesaikan gap yang tercatat di checklist dan prioritas lanjutan di atas.

Fokus terdekat adalah membuat V1.0.1 stabil sebagai beta: build bersih, transfer end-to-end tervalidasi, gap UI kecil dibereskan, error state dicatat, dan behavior reconnect/notification dipoles.

---

*UPDATE_PRD_V1.0.1 - Disusun dari analisis repository ArSend per 16 Mei 2026.*
