use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

#[tauri::command]
pub fn notify_transfer_complete(app: AppHandle, filename: String, is_receive: bool) {
    let action = if is_receive { "diterima" } else { "terkirim" };
    let _ = app.notification()
        .builder()
        .title("ArSend Transfer Selesai")
        .body(format!("File {} berhasil {}.", filename, action))
        .show();
}

#[tauri::command]
pub fn notify_connection(app: AppHandle, device_name: String) {
    let _ = app.notification()
        .builder()
        .title("Perangkat Terhubung")
        .body(format!("Terhubung dengan {}.", device_name))
        .show();
}