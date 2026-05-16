# ArSend

ArSend is a local network file transfer application for desktop and Android. It is built with Tauri v2, Svelte 5, and Rust, with a focus on fast LAN transfer, device-to-device trust, and a simple approval flow before files are received.

The project is currently in active development. Core transfer, discovery, pairing, history, and resumable transfer groundwork are already implemented, while packaging and wider platform validation are still being refined.

## Features

- Local file transfer over LAN/Wi-Fi.
- Desktop and Android-oriented interface.
- Automatic device discovery using mDNS.
- QR pairing and trust-on-first-use device verification.
- TLS-protected signaling and file transfer channels.
- Receiver approval before incoming files are accepted.
- Streaming transfer with 2 MB buffers and final SHA-256 validation.
- Resume support for interrupted transfers using partial files.
- Transfer progress, cancellation, retry/resume UI, and local history.
- Configurable device name and download location.

## Architecture

ArSend separates control traffic from file data:

```text
Svelte UI
  |
  | Tauri commands/events
  v
Rust backend
  |
  +-- mDNS discovery
  +-- TLS identity and fingerprint verification
  +-- WebSocket signaling on port 9527
  +-- TLS file transfer stream on port 9528
  +-- Local stores for settings, trust, and history
```

The signaling channel handles identity exchange, heartbeat, file offers, accept/reject responses, and connection state. File bytes are sent through a separate TCP/TLS stream so the data path can stay simple and fast.

## Technology Stack

Frontend:

- Svelte 5 and SvelteKit
- Tailwind CSS 4
- Lucide Svelte
- Tauri JavaScript APIs and plugins

Backend:

- Tauri v2
- Tokio async runtime
- tokio-rustls for TLS
- tokio-tungstenite for WebSocket signaling
- mdns-sd for local discovery
- rcgen and SHA-256 certificate fingerprints for local identity

## Development Requirements

- Node.js 18 or newer
- Rust stable
- Tauri development prerequisites for your OS
- Android Studio and Android SDK for Android builds

## Getting Started

Install dependencies:

```bash
npm install
```

Run the desktop development app:

```bash
npm run tauri dev
```

Run the Android development app:

```bash
npm run tauri android dev
```

Run frontend checks:

```bash
npm run check
```

Run Rust checks:

```bash
cd src-tauri
cargo check
```

## Project Structure

```text
src/
  lib/
    components/      UI components
    stores/          Svelte stores for connection, transfer, settings, history
    utils/           Formatting and platform helpers
  routes/            SvelteKit routes

src-tauri/
  src/
    network.rs       mDNS discovery and local IP lookup
    pairing.rs       QR payloads, session tokens, trusted devices
    security.rs      TLS identity and fingerprint verification
    server.rs        WebSocket signaling server/client
    transfer.rs      File transfer server/client and resume flow
    notification.rs  System notification helpers
```

## Current Status

Implemented:

- Device discovery
- QR pairing
- TOFU trusted devices
- TLS signaling and transfer
- File offer approval
- Large file streaming
- Transfer cancellation
- Partial-file resume flow
- Transfer progress UI
- Local transfer history
- Basic desktop and mobile layouts

In progress:

- Hardening resume metadata and edge cases
- Android end-to-end validation across devices
- Failed/cancelled history entries
- Production packaging
- Security and permission review for release builds

## Roadmap

- More robust resume validation using transfer metadata.
- Folder transfer support.
- Manual connection fallback for networks where mDNS is unavailable.
- Better reconnect behavior after connection loss.
- Expanded Android testing and final APK packaging.
- Optional performance tuning for socket buffers and hashing.

## License

This project is licensed under the MIT License.

## Author

Syahrul Efendi - [@syhrlf-e](https://github.com/syhrlf-e)
