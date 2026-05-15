pub mod network;
pub mod pairing;
pub mod server;
pub mod transfer;
pub mod security;
pub mod notification;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider()
        .install_default();

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
        .setup(|_app| {
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
            server::disconnect_device,
            transfer::start_transfer_server,
            transfer::accept_file_offer,
            transfer::reject_file_offer,
            transfer::send_file_offer,
            transfer::send_file,
            notification::notify_transfer_complete,
            notification::notify_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
