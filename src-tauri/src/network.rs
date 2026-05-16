use local_ip_address::local_ip;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

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

#[tauri::command]
pub async fn get_local_ip() -> Result<String, String> {
    local_ip()
        .map(|ip| ip.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_discovery(app: AppHandle, payload: DiscoveryPayload) -> Result<(), String> {
    let mdns = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let service_type = "_arsend._tcp.local.";
    let safe_name = payload.name.replace(' ', "-");
    let short_pk: String = payload.public_key.chars().take(8).collect();
    let instance_name = format!("{}-{}", safe_name, short_pk);
    let host_name = format!("{}.local.", instance_name);
    let ip = local_ip().map_err(|e| e.to_string())?;
    let mut properties = HashMap::new();

    properties.insert("name".to_string(), payload.name.clone());
    properties.insert("public_key".to_string(), payload.public_key.clone());
    properties.insert("version".to_string(), payload.version.clone());
    properties.insert("device_type".to_string(), payload.device_type.clone());

    let service_info = ServiceInfo::new(
        service_type,
        &instance_name,
        &host_name,
        ip,
        payload.port,
        Some(properties),
    )
    .map_err(|e| e.to_string())?;

    mdns.register(service_info).map_err(|e| e.to_string())?;
    eprintln!("✅ mDNS Broadcaster started: {}", instance_name);

    let receiver = mdns.browse(service_type).map_err(|e| e.to_string())?;
    eprintln!("🔍 mDNS Browser started, looking for {}", service_type);

    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let mut resolved_ip = String::new();
                    for addr in info.get_addresses() {
                        resolved_ip = addr.to_string();
                        break;
                    }

                    if resolved_ip == ip.to_string() {
                        continue;
                    }

                    let props = info.get_properties();
                    let name = props.get_property_val_str("name").unwrap_or_default().to_string();
                    let public_key = props.get_property_val_str("public_key").unwrap_or_default().to_string();
                    let version = props.get_property_val_str("version").unwrap_or_default().to_string();
                    let device_type = props.get_property_val_str("device_type").unwrap_or_default().to_string();
                    let port = info.get_port();

                    if public_key.is_empty() || name.is_empty() {
                        continue;
                    }

                    let discovered = DiscoveredDevice {
                        payload: DiscoveryPayload {
                            name,
                            public_key: public_key.clone(),
                            version,
                            port,
                            device_type,
                        },
                        ip: resolved_ip,
                    };

                    eprintln!("🎯 mDNS Device Discovered: {} ({})", discovered.payload.name, discovered.ip);
                    let _ = app_handle.emit("device-discovered", discovered);
                }
                ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                    if let Some(instance) = fullname.split('.').next() {
                        if let Some(short_pk) = instance.split('-').last() {
                            eprintln!("❌ mDNS Device Removed: {}", fullname);

                            let _ = app_handle.emit("device-removed", short_pk.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    });

    Ok(())
}
