use crate::network;
use crate::security;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

const TRUST_STORE: &str = "arsend_trust.json";
const QR_TOKEN_TTL: Duration = Duration::from_secs(180);

lazy_static::lazy_static! {
    static ref ACTIVE_QR_TOKENS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrustedDevice {
    pub public_key: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QrPayload {
    pub ip: String,
    pub port: u16,
    pub token: String,
    pub public_key: String,
    pub device_name: String,
}

pub fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

fn remember_session_token(token: &str) -> Result<(), String> {
    let mut tokens = ACTIVE_QR_TOKENS.lock().map_err(|e| e.to_string())?;
    let now = Instant::now();

    tokens.retain(|_, created_at| now.duration_since(*created_at) <= QR_TOKEN_TTL);
    tokens.insert(token.to_string(), now);

    Ok(())
}

pub fn validate_session_token(token: &str) -> bool {
    let Ok(mut tokens) = ACTIVE_QR_TOKENS.lock() else {
        return false;
    };

    let now = Instant::now();
    tokens.retain(|_, created_at| now.duration_since(*created_at) <= QR_TOKEN_TTL);

    tokens
        .remove(token)
        .is_some_and(|created_at| now.duration_since(created_at) <= QR_TOKEN_TTL)
}

#[tauri::command]
pub async fn get_qr_payload(app: AppHandle, device_name: String) -> Result<QrPayload, String> {
    let identity = security::get_public_key(app.clone()).await?;
    let ip = network::get_local_ip().await?;
    let token = generate_session_token();
    remember_session_token(&token)?;

    Ok(QrPayload {
        ip,
        port: 9527,
        token,
        public_key: identity.public_key_hex,
        device_name,
    })
}

#[tauri::command]
pub async fn generate_qr_svg(payload: String) -> Result<String, String> {
    let code = QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?;
    let svg = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#0045B5")) // Cobalt Deep accent
        .light_color(svg::Color("#FFFFFF"))
        .build();
    Ok(svg)
}

#[tauri::command]
pub async fn get_trusted_devices(app: AppHandle) -> Result<Vec<TrustedDevice>, String> {
    let store = app.store(TRUST_STORE).map_err(|e| e.to_string())?;

    if let Some(devices_val) = store.get("trusted_devices") {
        if let Ok(devices) = serde_json::from_value::<Vec<TrustedDevice>>(devices_val) {
            return Ok(devices);
        }
    }

    Ok(vec![])
}

#[tauri::command]
pub async fn trust_device(app: AppHandle, public_key: String, name: String) -> Result<(), String> {
    let store = app.store(TRUST_STORE).map_err(|e| e.to_string())?;

    let mut devices = vec![];
    if let Some(devices_val) = store.get("trusted_devices") {
        if let Ok(existing) = serde_json::from_value::<Vec<TrustedDevice>>(devices_val) {
            devices = existing;
        }
    }

    if !devices.iter().any(|d| d.public_key == public_key) {
        devices.push(TrustedDevice { public_key, name });
        let val = serde_json::to_value(&devices).map_err(|e| e.to_string())?;
        store.set("trusted_devices", val);
        store.save().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn remove_trusted_device(app: AppHandle, public_key: String) -> Result<(), String> {
    let store = app.store(TRUST_STORE).map_err(|e| e.to_string())?;

    let mut devices = vec![];
    if let Some(devices_val) = store.get("trusted_devices") {
        if let Ok(existing) = serde_json::from_value::<Vec<TrustedDevice>>(devices_val) {
            devices = existing;
        }
    }

    devices.retain(|d| d.public_key != public_key);
    let val = serde_json::to_value(&devices).map_err(|e| e.to_string())?;
    store.set("trusted_devices", val);
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn is_device_trusted(app: AppHandle, public_key: String) -> Result<bool, String> {
    let devices = get_trusted_devices(app).await?;
    Ok(devices.iter().any(|d| d.public_key == public_key))
}
