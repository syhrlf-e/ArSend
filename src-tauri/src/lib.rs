pub mod network;
pub mod pairing;
pub mod server;
pub mod transfer;
pub mod security;
pub mod notification;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_barcode_scanner::init());
    }

    builder
        .setup(|app| {
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // Ambil identity (public key) dari security module
                let identity = match security::get_or_create_identity(&app_handle) {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("❌ Failed to get identity: {}", e);
                        return;
                    }
                };

                // Ambil device name dari OS
                let device_name = whoami::devicename();

                // Tentukan device type berdasarkan platform
                #[cfg(desktop)]
                let device_type = "desktop".to_string();
                #[cfg(mobile)]
                let device_type = "mobile".to_string();

                let payload = network::DiscoveryPayload {
                    name: device_name,
                    public_key: identity.public_key_hex,
                    version: "1.0.1".to_string(),
                    port: 9527,
                    device_type,
                };

                eprintln!(
                    "🚀 Starting discovery as: {} ({})",
                    payload.name, payload.device_type
                );

                if let Err(e) = network::start_discovery(app_handle, payload).await {
                    eprintln!("❌ Discovery error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            network::get_local_ip,
            network::start_discovery,
            security::get_public_key,
            pairing::get_trusted_devices,
            pairing::trust_device,
            pairing::remove_trusted_device,
            pairing::is_device_trusted,
            pairing::get_qr_payload,
            pairing::generate_qr_svg,
            server::start_server,
            server::connect_to_device,
            transfer::start_transfer_server,
            transfer::send_file,
            notification::notify_transfer_complete,
            notification::notify_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
