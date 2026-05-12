use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::net::UdpSocket;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveryPayload {
    pub name: String,
    pub public_key: String,
    pub version: String,
    pub port: u16,
    pub device_type: String,
}

#[derive(Serialize, Clone)]
pub struct DiscoveredDevice {
    pub payload: DiscoveryPayload,
    pub ip: String,
}

fn get_broadcast_addr(local_ip: &str) -> String {
    let parts: Vec<&str> = local_ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.{}.255", parts[0], parts[1], parts[2])
    } else {
        "255.255.255.255".to_string()
    }
}

#[tauri::command]
pub fn get_local_ip() -> Result<String, String> {
    local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_discovery(
    app: AppHandle,
    payload: DiscoveryPayload,
) -> Result<(), String> {

    let local = local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_default();
    let broadcast_ip = get_broadcast_addr(&local);
    let broadcast_addr: std::net::SocketAddr = format!("{}:9526", broadcast_ip)
        .parse()
        .unwrap();

    eprintln!("📡 Local IP: {}", local);
    eprintln!("📡 Broadcast to: {}", broadcast_addr);

    // ── LISTENER SOCKET ──────────────────────────────────────
    let listener_raw = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| e.to_string())?;
    listener_raw.set_reuse_address(true).map_err(|e| e.to_string())?;
    #[cfg(not(windows))]
    listener_raw.set_reuse_port(true).map_err(|e| e.to_string())?;
    listener_raw.set_broadcast(true).map_err(|e| e.to_string())?;
    listener_raw
        .bind(&"0.0.0.0:9526".parse::<std::net::SocketAddr>().unwrap().into())
        .map_err(|e| e.to_string())?;
    let listener_std: std::net::UdpSocket = listener_raw.into();
    listener_std.set_nonblocking(true).map_err(|e| e.to_string())?;
    let listener_udp = UdpSocket::from_std(listener_std).map_err(|e| e.to_string())?;

    // ── SENDER SOCKET ─────────────────────────────────────────
    let sender_raw = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|e| e.to_string())?;
    sender_raw.set_broadcast(true).map_err(|e| e.to_string())?;
    // FIX: bind ke local IP agar OS tahu interface mana yang dipakai untuk broadcast
    let bind_addr = format!("{}:0", local);
    sender_raw
        .bind(&bind_addr.parse::<std::net::SocketAddr>().unwrap().into())
        .map_err(|e| e.to_string())?;
    let sender_std: std::net::UdpSocket = sender_raw.into();
    sender_std.set_nonblocking(true).map_err(|e| e.to_string())?;
    let sender_udp = Arc::new(
        UdpSocket::from_std(sender_std).map_err(|e| e.to_string())?
    );

    let payload_bytes = Arc::new(
        serde_json::to_vec(&payload).map_err(|e| e.to_string())?
    );
    let my_pub_key = payload.public_key.clone();

    // ── BROADCAST TASK ────────────────────────────────────────
    let b_socket = sender_udp.clone();
    let b_payload = payload_bytes.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            interval.tick().await;
            match b_socket.send_to(&b_payload, broadcast_addr).await {
                Ok(bytes) => eprintln!("📤 Broadcast sent ({} bytes) to {}", bytes, broadcast_addr),
                Err(e) => eprintln!("❌ Broadcast error: {}", e),
            }
        }
    });

    // ── LISTENER TASK ─────────────────────────────────────────
    let app_handle = app.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        eprintln!("👂 Listening for devices on 0.0.0.0:9526...");
        loop {
            match listener_udp.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    eprintln!("📥 Received {} bytes from {}", len, addr);
                    match serde_json::from_slice::<DiscoveryPayload>(&buf[..len]) {
                        Ok(discovered) => {
                            if discovered.public_key != my_pub_key {
                                eprintln!("✅ Device found: {} ({})", discovered.name, addr.ip());
                                let _ = app_handle.emit(
                                    "device-discovered",
                                    DiscoveredDevice {
                                        payload: discovered,
                                        ip: addr.ip().to_string(),
                                    },
                                );
                            } else {
                                eprintln!("🔄 Received own broadcast, ignoring");
                            }
                        }
                        Err(e) => eprintln!("⚠️ Failed to parse payload: {}", e),
                    }
                }
                Err(e) => eprintln!("❌ Listener error: {}", e),
            }
        }
    });

    Ok(())
}
