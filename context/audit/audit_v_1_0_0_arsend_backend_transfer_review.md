# ArSend Backend Transfer Audit v1.0.0

## Project
ArSend — Local Network File Transfer Application

## Audit Scope
Fokus audit ini berada pada:
- Backend transfer architecture
- Network protocol design
- Transfer throughput optimization
- Tokio async behavior
- TLS/TCP implementation
- Chunk streaming pipeline
- Disk I/O behavior
- Android transfer considerations

---

# Executive Summary

Secara keseluruhan, fondasi arsitektur backend ArSend sudah sangat baik dan sudah melewati level “project socket sederhana”.

Implementasi saat ini sudah memiliki:
- Separation of concerns yang benar
- Dedicated signaling channel
- Dedicated transfer channel
- TLS architecture yang baik
- Async Tokio model yang benar
- mDNS discovery modern
- Cross-platform awareness
- Android SAF integration yang matang

Namun bottleneck utama saat ini bukan berada pada:
- Tokio
- TLS
- TCP
- mDNS

Melainkan berada pada:

## Protocol Overhead

Khususnya:
- Per-chunk ACK
- Per-chunk flush
- Per-chunk JSON serialization
- Per-chunk allocation
- Excessive transfer verification
- UI event frequency

Arsitektur saat ini masih menggunakan pola:

```text
SEND -> WAIT -> ACK -> CONTINUE
```

bukan:

```text
CONTINUOUS STREAMING PIPELINE
```

Dan ini menjadi bottleneck terbesar terhadap throughput transfer.

---

# Current Architecture Review

## 1. Signaling Architecture

### Current
- WebSocket over TLS
- Dedicated signaling server
- File offer system
- Accept/reject protocol
- Heartbeat system

### Assessment
Excellent.

Pemisahan signaling dan transfer merupakan keputusan arsitektur yang sangat benar.

### Why It Is Good
Karena:
- Signaling traffic tidak mengganggu transfer throughput
- WebSocket cocok untuk control plane
- Raw TCP cocok untuk data plane
- Session management menjadi lebih clean
- Debugging lebih mudah

### Recommendation
No major change required.

---

# 2. Transfer Transport

## Current
- Raw TCP stream
- TLS encrypted
- Dedicated transfer socket
- Chunk-based transfer

## Assessment
Excellent foundation.

Ini merupakan arsitektur modern yang digunakan oleh banyak aplikasi transfer modern.

### Recommendation
Tetap gunakan:
- TCP
- TLS
- Dedicated transfer stream

Tidak perlu pindah ke QUIC saat ini.

---

# 3. mDNS Discovery

## Current
Using:
- mdns-sd

## Assessment
Good upgrade.

Lebih baik daripada UDP broadcast.

### Benefits
- Cross-platform lebih stabil
- Modern network compatible
- Mengurangi broadcast spam
- Better router compatibility
- Better Android behavior
- Future-proof

### Recommendation
Pertahankan mDNS.

---

# 4. Chunk Size

## Current

```rust
const CHUNK_SIZE: usize = 2 * 1024 * 1024;
```

## Assessment
Good decision.

Untuk workload:
- Local transfer
- Large files
- LAN environment
- Throughput-oriented system

2MB jauh lebih cocok dibanding 64KB.

### Advantages
- Mengurangi syscall
- Mengurangi ACK frequency
- Mengurangi wakeup async task
- Better TLS batching
- Better TCP throughput
- Better disk write batching

### Recommendation
Current recommendation:
- Desktop: 2MB–4MB
- Android: 1MB–2MB

2MB saat ini sudah sangat reasonable.

---

# Major Bottlenecks

# BOTTLENECK #1 — Stop-and-Wait Protocol

## Current Behavior

Current flow:

```text
send chunk
wait ack
send next chunk
wait ack
```

Receiver:

```text
verify hash
send ACK
flush
```

## Problem

Transfer tidak benar-benar streaming.

TCP congestion window tidak pernah berjalan optimal.

Roundtrip latency menjadi faktor throughput.

### Impact
Very high.

Ini kemungkinan bottleneck terbesar dalam keseluruhan sistem.

---

## Recommendation

### Remove
- Per-chunk ACK
- Per-chunk retry
- Per-chunk hash verification

### Keep
- Final SHA verification

---

## Why?

Karena:

TCP sudah memiliki:
- Reliability
- Ordering
- Retransmission
- Checksum

TLS juga memiliki:
- Integrity protection
- Authenticated encryption

Maka:

```text
chunk hash + chunk ACK + chunk retry
```

menjadi redundant untuk local network modern.

---

## Recommended Flow

### Sender

```text
read
send
read
send
read
send
```

### Receiver

```text
read
write
update final hash
continue
```

### Final Step

```text
compare final SHA-256
```

---

# BOTTLENECK #2 — Excessive flush()

## Current

Frequent:

```rust
stream.flush().await
```

inside hot transfer path.

## Problem

flush() terlalu sering dapat:
- Mengurangi batching TCP
- Menambah syscall
- Mengurangi throughput
- Memperburuk TLS packetization

## Recommendation

Remove most flush() calls in hot path.

Flush hanya diperlukan untuk:
- Finalization
- Special protocol boundaries

Continuous transfer sebaiknya membiarkan buffering dilakukan oleh:
- Tokio
- TCP
- TLS layer

---

# BOTTLENECK #3 — Per-Chunk Allocation

## Current

Sender:

```rust
let mut chunk_buf = vec![0u8; current_chunk_size];
```

Receiver:

```rust
let mut data_buf = vec![0u8; chunk_header.length as usize];
```

allocation dilakukan setiap chunk.

## Problem

Untuk chunk 2MB:
- allocator pressure tinggi
- fragmentation meningkat
- unnecessary memory churn
- Android memory pressure meningkat

## Recommendation

Reuse buffer.

### Better Approach

```rust
let mut chunk_buf = vec![0u8; CHUNK_SIZE];
```

sekali saja.

Kemudian:

```rust
file.read(&mut chunk_buf[..size])
```

---

# BOTTLENECK #4 — UI Event Frequency

## Current

Emit progress setiap chunk:

```rust
app.emit(...)
```

## Problem

Tauri IPC:

```text
Rust -> IPC -> JS Runtime -> Svelte Update
```

cukup mahal.

## Recommendation

Rate limit UI update.

### Suggested
- setiap 200ms
atau
- setiap progress berubah 1%

---

# Protocol Design Review

# Current Chunk Header

## Current

JSON serialization per chunk:

```rust
serde_json::to_vec(&header)
```

## Problem

JSON di hot path transfer cukup mahal.

### Cost
- Allocation
- UTF parsing
- Serialization overhead
- Cache miss

---

## Recommendation

Gunakan binary fixed-size protocol.

### Example

```text
[u32 index]
[u32 length]
[32-byte hash]
```

Tanpa JSON.

---

# Hashing System Review

# Current

Current behavior:
- Full file pre-hash
- Per chunk hash
- Final receiver hash

## Problem

CPU work menjadi double/triple.

Untuk file besar:
- Android
- HDD
- thermal condition

akan mulai terasa.

---

# Recommendation

## Remove
- Per chunk hash

## Keep
- Final file SHA verification

---

## Better Hash Flow

### Sender

Hash sambil transfer:

```text
read -> update hash -> send
```

### Receiver

```text
read -> write -> update hash
```

### Final

```text
compare final hash
```

---

# Suggested Hash Upgrade

## Recommendation

Pertimbangkan menggunakan:

- BLAKE3

sebagai alternatif SHA256.

## Why?

BLAKE3:
- Extremely fast
- Streaming friendly
- Modern design
- Excellent for local transfer

---

# Disk I/O Review

# Android Consideration

## Current

Android SAF integration surprisingly mature.

### Good Points
- JNI usage benar
- detachFd ownership benar
- Content URI handling bagus
- Cross-platform awareness baik

## Likely Bottleneck

Kemungkinan bottleneck Android terbesar nanti:

```rust
file.write_all(...)
```

karena:
- Android storage throughput
- Shared storage overhead
- Thermal throttling
- SD card variability

## Recommendation

Jangan terlalu cepat menyalahkan network speed.

Banyak kasus bottleneck utama justru:
- Storage write speed
- Thermal throttling

---

# Backpressure Architecture

# Current

Sender masih assume receiver selalu mampu menerima.

## Problem

Saat transfer menjadi continuous streaming:
- disk bisa lebih lambat dari network
- memory bisa naik
- buffering bisa tidak terkendali

---

# Recommended Modern Architecture

## Suggested Pipeline

```text
Network Task
    ↓
Bounded Channel
    ↓
Disk Writer Task
```

Menggunakan:

```rust
tokio::sync::mpsc
```

---

## Benefits

- Natural backpressure
- Network tetap cepat
- Disk bisa pace sendiri
- Memory lebih stabil
- Scalable

---

# Socket Optimization

# Current

Default socket buffer.

## Recommendation

Pertimbangkan tuning:

```rust
set_send_buffer_size
set_recv_buffer_size
```

Suggested:
- 4MB
- 8MB

Kadang throughput improvement cukup signifikan.

---

# Cancellation System Review

# Current

```rust
Mutex<HashSet<String>>
```

## Assessment

Untuk sekarang aman.

Namun untuk future concurrent transfer:
- bisa terjadi contention

## Recommendation

Pertimbangkan:
- DashMap
atau
- watch channel

---

# Tokio Architecture Review

# Current

```rust
tokio::spawn
```

per connection.

## Assessment

Good.

Simple, scalable, clean.

Tidak ada masalah besar di sini.

---

# Development Workflow Review

# HMR Protection

## Current

```rust
AddrInUse
```

handling.

## Assessment

Good engineering detail.

Small detail but shows maturity.

---

# Recommended Priority Roadmap

# PRIORITY TIER 1 — HIGH IMPACT

## 1. Remove Per-Chunk ACK

### Impact
🔥🔥🔥🔥🔥

### Complexity
Medium

---

## 2. Remove Excessive flush()

### Impact
🔥🔥🔥🔥

### Complexity
Low

---

## 3. Reuse Buffer

### Impact
🔥🔥🔥🔥

### Complexity
Low

---

## 4. Rate Limit Frontend Emit

### Impact
🔥🔥🔥🔥

### Complexity
Low

---

# PRIORITY TIER 2 — IMPORTANT

## 5. Streaming Hash Instead of Prehash

### Impact
🔥🔥🔥

### Complexity
Medium

---

## 6. Binary Protocol Instead of JSON

### Impact
🔥🔥🔥

### Complexity
Medium

---

## 7. Consider BLAKE3

### Impact
🔥🔥

### Complexity
Low

---

# PRIORITY TIER 3 — ADVANCED

## 8. Backpressure Pipeline

### Impact
🔥🔥🔥🔥🔥

### Complexity
High

---

# Estimated Real-World Throughput

## Current Estimated

### Android ↔ PC

Estimated:

```text
8–25 MB/s
```

---

## After Optimization

Possible:

```text
20–50 MB/s
```

tergantung:
- WiFi quality
- Storage speed
- Thermal state
- Frontend load
- Device class

---

# Final Conclusion

ArSend saat ini sudah memiliki:
- Fondasi arsitektur yang benar
- Security model yang baik
- Separation of concerns yang baik
- Async architecture yang benar
- Modern discovery system
- Good transfer foundation

Masalah utama sekarang bukan lagi:

```text
how to make transfer work
```

melainkan:

```text
how to reduce accumulated overhead
```

Dan itu merupakan fase engineering yang memang terjadi pada aplikasi production-grade.

---

# Final Senior Engineering Assessment

## Overall Backend Architecture

### Assessment
Very Good.

### Strong Points
- Correct transport separation
- Correct async model
- Correct TLS placement
- Good platform awareness
- Good Android handling
- Good scalability foundation

### Main Weakness
- Protocol chatter terlalu banyak
- Transfer pipeline belum true streaming
- Hot path masih memiliki unnecessary overhead

### Main Recommendation

Transform transfer architecture dari:

```text
transactional chunk transfer
```

menjadi:

```text
continuous streaming pipeline
```

Karena itu kemungkinan akan memberikan peningkatan performa terbesar untuk ArSend.

