# ArSend 🚀

**ArSend** adalah aplikasi transfer file lintas platform (Desktop & Android) yang mengutamakan kecepatan, keamanan, dan kemudahan penggunaan. Dibangun menggunakan teknologi modern **Tauri v2** dan **Svelte 5**, ArSend memungkinkan pengiriman file berukuran besar secara instan dalam jaringan lokal (LAN/Wi-Fi).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Android-green.svg)
![Build](https://img.shields.io/badge/built%20with-Tauri%20v2-orange.svg)

---

## ✨ Fitur Utama

- **🚀 Transfer Super Cepat:** Optimasi *chunking* 2MB dan pemrosesan stream asinkron untuk kecepatan transfer maksimal.
- **🔒 Keamanan TLS:** Seluruh proses transfer data dilindungi oleh enkripsi TLS end-to-end menggunakan sertifikat yang dihasilkan secara lokal (Self-signed).
- **📱 Lintas Platform:** Mendukung penuh perangkat Windows (Desktop) dan Android dengan antarmuka yang responsif.
- **🔍 Auto-Discovery:** Menemukan perangkat lain di jaringan lokal secara otomatis tanpa perlu input IP manual (via mDNS).
- **🤝 Pairing Aman:** Sistem verifikasi *Public Key Fingerprint* dan QR Code untuk memastikan Anda hanya mengirim file ke perangkat yang dipercaya.
- **📄 Riwayat Transfer:** Mencatat log pengiriman dan penerimaan file secara lokal.
- **🌗 Desain Modern:** Antarmuka bersih, minimalis, dan intuitif dengan dukungan mode terang/gelap (Day/Night).

---

## 🏗️ Arsitektur Teknis

ArSend menggunakan pemisahan tanggung jawab yang ketat antara antarmuka pengguna dan logika sistem:

### 🎨 Frontend (Svelte 5)
- **Framework:** Svelte 5 (Runes) untuk reaktivitas yang efisien.
- **Styling:** Tailwind CSS 4 dengan desain kustom yang elegan.
- **Icons:** Lucide Svelte.
- **State Management:** Svelte Writable Stores untuk manajemen koneksi dan riwayat.

### ⚙️ Backend (Rust)
- **Tauri v2:** Sebagai jembatan antara webview dan sistem operasi.
- **Networking:**
  - `tokio`: Runtime asinkron untuk menangani koneksi konkuren.
  - `tokio-rustls`: Implementasi TLS asinkron untuk transfer data aman.
  - `mdns-sd`: Penemuan layanan (Service Discovery) otomatis di jaringan lokal.
- **Security:** Implementasi `ring` untuk kriptografi dan pembangkitan sertifikat TLS on-the-fly.
- **Storage:** `tauri-plugin-store` untuk persistensi pengaturan dan daftar perangkat terpercaya.

---

## 🛠️ Persyaratan Sistem

### Pengembangan
- **Rust:** Versi terbaru (Stable).
- **Node.js:** Versi 18 atau lebih baru.
- **Android Studio:** Untuk melakukan kompilasi file `.apk` (jika ingin build untuk Android).

### Pengguna
- **Windows:** Windows 10/11 (WebView2 terinstal).
- **Android:** Android 7.0 (Nougat) atau lebih baru.

---

## 🚀 Cara Menjalankan (Development)

1. **Clone Repository:**
   ```bash
   git clone https://github.com/syhrlf-e/ArSend.git
   cd ArSend
   ```

2. **Instal Dependensi:**
   ```bash
   npm install
   ```

3. **Jalankan Mode Desktop:**
   ```bash
   npm run tauri dev
   ```

4. **Jalankan Mode Android:**
   *(Pastikan perangkat Android terhubung via USB Debugging)*
   ```bash
   npm run tauri android dev
   ```

---

## 📝 Roadmap Pengembangan
- [ ] Support transfer Folder secara rekursif.
- [ ] Implementasi sistem *Resume Transfer* jika koneksi terputus.
- [ ] Versi iOS dan macOS.
- [ ] Pengiriman pesan teks singkat antar perangkat.
- [ ] Enkripsi file tambahan sebelum dikirim.

---

## 📄 Lisensi
Proyek ini dilisensikan di bawah **MIT License**. Lihat file [LICENSE](LICENSE) untuk detail lebih lanjut.

---

## 👨‍💻 Kontributor
- **Syahrul Efendi** - *Initial Work & Lead Developer* - [@syhrlf-e](https://github.com/syhrlf-e)

---
*Dibuat dengan ❤️ untuk kemudahan berbagi.*
