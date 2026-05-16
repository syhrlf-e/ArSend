use crate::security;
use crate::server::{SharedSession, WsMessage};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::io::Error as IoError;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_store::StoreExt;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::{TlsAcceptor, TlsConnector};

lazy_static! {
    static ref CANCELLED_TRANSFERS: tokio::sync::Mutex<HashSet<String>> =
        tokio::sync::Mutex::new(HashSet::new());
    static ref ACCEPTED_NONCES: tokio::sync::Mutex<HashSet<String>> =
        tokio::sync::Mutex::new(HashSet::new());
}

#[tauri::command]
pub async fn cancel_transfer(nonce: String) -> Result<(), String> {
    eprintln!("🛑 Cancelling transfer for nonce: {}", nonce);
    let mut cancelled = CANCELLED_TRANSFERS.lock().await;
    cancelled.insert(nonce);
    Ok(())
}

async fn take_cancel_signal(nonce: &str) -> bool {
    let mut cancelled = CANCELLED_TRANSFERS.lock().await;
    cancelled.remove(nonce)
}

#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static ANDROID_JVM: OnceLock<jni::JavaVM> = OnceLock::new();

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(
    vm: *mut jni::sys::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    if let Ok(java_vm) = jni::JavaVM::from_raw(vm) {
        let _ = ANDROID_JVM.set(java_vm);
    }

    jni::sys::JNI_VERSION_1_6
}

#[cfg(target_os = "android")]
fn open_android_content_uri(uri: &str) -> Result<std::fs::File, String> {
    use jni::objects::{JObject, JValue};
    use std::os::unix::io::FromRawFd;

    let vm = ANDROID_JVM
        .get()
        .ok_or_else(|| "Android JavaVM was not initialized".to_string())?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("JNI Env error: {}", e))?;

    // Parse URI
    let uri_jstring = env.new_string(uri).map_err(|e| e.to_string())?;
    let uri_object = JObject::from(uri_jstring);
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|e| e.to_string())?;
    let parsed_uri = env
        .call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_object)],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    // Get Application context without relying on ndk_context initialization.
    let activity_thread_class = env
        .find_class("android/app/ActivityThread")
        .map_err(|e| e.to_string())?;
    let application = env
        .call_static_method(
            activity_thread_class,
            "currentApplication",
            "()Landroid/app/Application;",
            &[],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    if application.is_null() {
        return Err("Android application context unavailable".to_string());
    }

    // Get ContentResolver
    let content_resolver = env
        .call_method(
            &application,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    // Open FileDescriptor (read-only)
    let mode_jstring = env.new_string("r").map_err(|e| e.to_string())?;
    let mode_object = JObject::from(mode_jstring);
    let pfd = env
        .call_method(
            content_resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(&parsed_uri), JValue::Object(&mode_object)],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    if pfd.is_null() {
        return Err("Android ContentResolver returned a null file descriptor".to_string());
    }

    // Detach the FD so Rust takes ownership
    let fd = env
        .call_method(&pfd, "detachFd", "()I", &[])
        .map_err(|e| e.to_string())?
        .i()
        .map_err(|e| e.to_string())?;

    // Wrap the raw FD in a standard Rust File
    let std_file = unsafe { std::fs::File::from_raw_fd(fd) };
    Ok(std_file)
}

#[cfg(target_os = "android")]
fn get_android_content_uri_size(uri: &str) -> Result<u64, String> {
    use jni::objects::{JObject, JValue};

    let vm = ANDROID_JVM
        .get()
        .ok_or_else(|| "Android JavaVM was not initialized".to_string())?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("JNI Env error: {}", e))?;

    let uri_jstring = env.new_string(uri).map_err(|e| e.to_string())?;
    let uri_object = JObject::from(uri_jstring);
    let uri_class = env
        .find_class("android/net/Uri")
        .map_err(|e| e.to_string())?;
    let parsed_uri = env
        .call_static_method(
            uri_class,
            "parse",
            "(Ljava/lang/String;)Landroid/net/Uri;",
            &[JValue::Object(&uri_object)],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    let activity_thread_class = env
        .find_class("android/app/ActivityThread")
        .map_err(|e| e.to_string())?;
    let application = env
        .call_static_method(
            activity_thread_class,
            "currentApplication",
            "()Landroid/app/Application;",
            &[],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    if application.is_null() {
        return Err("Android application context unavailable".to_string());
    }

    let content_resolver = env
        .call_method(
            &application,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    let mode_jstring = env.new_string("r").map_err(|e| e.to_string())?;
    let mode_object = JObject::from(mode_jstring);
    let pfd = env
        .call_method(
            &content_resolver,
            "openFileDescriptor",
            "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
            &[JValue::Object(&parsed_uri), JValue::Object(&mode_object)],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    let mut stat_size_value: Option<u64> = None;
    if !pfd.is_null() {
        let stat_size = env
            .call_method(&pfd, "getStatSize", "()J", &[])
            .map_err(|e| e.to_string())?
            .j()
            .map_err(|e| e.to_string())?;
        let _ = env.call_method(&pfd, "close", "()V", &[]);

        if stat_size > 0 {
            eprintln!(
                "[ANDROID_SIZE] path={}, reported_size={}, method=ParcelFileDescriptor.getStatSize",
                uri, stat_size
            );
            stat_size_value = Some(stat_size as u64);
        }
    }

    let openable_columns = env
        .find_class("android/provider/OpenableColumns")
        .map_err(|e| e.to_string())?;
    let size_column = env
        .get_static_field(openable_columns, "SIZE", "Ljava/lang/String;")
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;
    let string_class = env
        .find_class("java/lang/String")
        .map_err(|e| e.to_string())?;
    let projection = env
        .new_object_array(1, string_class, JObject::null())
        .map_err(|e| e.to_string())?;
    env.set_object_array_element(&projection, 0, &size_column)
        .map_err(|e| e.to_string())?;

    let projection_obj = JObject::from(projection);
    let null_obj = JObject::null();
    let null_array = JObject::null();
    let cursor = env
        .call_method(
            &content_resolver,
            "query",
            "(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            &[
                JValue::Object(&parsed_uri),
                JValue::Object(&projection_obj),
                JValue::Object(&null_obj),
                JValue::Object(&null_array),
                JValue::Object(&null_obj),
            ],
        )
        .map_err(|e| e.to_string())?
        .l()
        .map_err(|e| e.to_string())?;

    if !cursor.is_null() {
        let moved = env
            .call_method(&cursor, "moveToFirst", "()Z", &[])
            .map_err(|e| e.to_string())?
            .z()
            .map_err(|e| e.to_string())?;

        if moved {
            let column_index = env
                .call_method(
                    &cursor,
                    "getColumnIndex",
                    "(Ljava/lang/String;)I",
                    &[JValue::Object(&size_column)],
                )
                .map_err(|e| e.to_string())?
                .i()
                .map_err(|e| e.to_string())?;

            if column_index >= 0 {
                let query_size = env
                    .call_method(&cursor, "getLong", "(I)J", &[JValue::Int(column_index)])
                    .map_err(|e| e.to_string())?
                    .j()
                    .map_err(|e| e.to_string())?;
                let _ = env.call_method(&cursor, "close", "()V", &[]);

                if query_size > 0 {
                    if let Some(stat_size) = stat_size_value {
                        if stat_size != query_size as u64 {
                            eprintln!(
                                "[ANDROID_SIZE_MISMATCH] path={}, stat_size={}, query_size={}, selected_method=ContentResolver.query(OpenableColumns.SIZE)",
                                uri, stat_size, query_size
                            );
                        }
                    }
                    eprintln!(
                        "[ANDROID_SIZE] path={}, reported_size={}, method=ContentResolver.query(OpenableColumns.SIZE)",
                        uri, query_size
                    );
                    return Ok(query_size as u64);
                }
            }
        }

        let _ = env.call_method(&cursor, "close", "()V", &[]);
    }

    if let Some(stat_size) = stat_size_value {
        eprintln!(
            "[ANDROID_SIZE] path={}, reported_size={}, method=ParcelFileDescriptor.getStatSize_fallback",
            uri, stat_size
        );
        return Ok(stat_size);
    }

    Err("Unable to determine Android content URI size".to_string())
}

fn get_unique_path(base_path: PathBuf) -> PathBuf {
    if !base_path.exists() {
        return base_path;
    }

    let dir = base_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let stem = base_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = base_path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let new_path = dir.join(format!("{} ({}){}", stem, counter, ext));
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

pub async fn open_file_stream(path: &str) -> Result<File, String> {
    #[cfg(target_os = "android")]
    {
        if path.starts_with("content://") {
            let std_file = open_android_content_uri(path)?;
            return Ok(File::from_std(std_file));
        }
    }

    // Fallback for Desktop or standard files
    File::open(path).await.map_err(|e| e.to_string())
}

fn filename_from_path(file_path: &str) -> String {
    if file_path.starts_with("content://") {
        let last_part = file_path
            .rsplit('/')
            .next()
            .filter(|part| !part.is_empty())
            .unwrap_or("unknown_file");

        last_part
            .replace("%20", " ")
            .replace("%2F", "_")
            .replace("%3A", "_")
    } else {
        PathBuf::from(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

async fn file_size(file: &File, context: &str) -> Result<u64, String> {
    file.metadata()
        .await
        .map(|metadata| metadata.len())
        .map_err(|e| {
            eprintln!("❌ {} metadata error: {}", context, e);
            e.to_string()
        })
}

async fn get_file_size_for_path(path: &str, file: &File, context: &str) -> Result<u64, String> {
    #[cfg(target_os = "android")]
    {
        if path.starts_with("content://") {
            return get_android_content_uri_size(path);
        }
    }

    let size = file_size(file, context).await?;
    eprintln!(
        "[FILE_SIZE] path={}, reported_size={}, method=metadata.len, context={}",
        path, size, context
    );
    Ok(size)
}

const CHUNK_SIZE: usize = 2 * 1024 * 1024;
const PORT: u16 = 9528;
const CHUNK_ACK_OK: &[u8; 7] = b"CHNK_OK";
const TRANSFER_ACK_OK: &[u8; 7] = b"DONE_OK";

#[tauri::command]
pub async fn accept_file_offer(app: AppHandle, nonce: String) -> Result<(), String> {
    let tx_option = {
        if let Some(s) = app.try_state::<SharedSession>() {
            let state: tauri::State<'_, SharedSession> = s;
            let sender_guard = state.inner().ws_sender.lock().await;
            sender_guard.clone()
        } else {
            None
        }
    };

    if let Some(tx) = tx_option {
        // Register nonce as authorized for transfer
        {
            let mut registry = ACCEPTED_NONCES.lock().await;
            registry.insert(nonce.clone());
            eprintln!("✅ Nonce registered for transfer: {}", nonce);
        }

        let msg = WsMessage::FileAccept {
            nonce: nonce.clone(),
        };
        if tx.send(msg).await.is_err() {
            // Cleanup on send failure
            let mut registry = ACCEPTED_NONCES.lock().await;
            registry.remove(&nonce);
            return Err("Failed to send accept message".to_string());
        }
        return Ok(());
    }

    Err("Not connected".to_string())
}

#[tauri::command]
pub async fn reject_file_offer(app: AppHandle, nonce: String) -> Result<(), String> {
    let tx_option = {
        if let Some(s) = app.try_state::<SharedSession>() {
            let state: tauri::State<'_, SharedSession> = s;
            let sender_guard = state.inner().ws_sender.lock().await;
            sender_guard.clone()
        } else {
            None
        }
    };

    if let Some(tx) = tx_option {
        let msg = WsMessage::FileReject { nonce };
        let _ = tx.send(msg).await;
        return Ok(());
    }

    Err("Not connected".to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferProgress {
    pub nonce: String,
    pub filename: String,
    pub progress: f64,
    pub speed_mb_s: f64,
    pub sent_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferComplete {
    pub nonce: String,
    pub filename: String,
    pub save_path: String,
    pub is_receive: bool,
    pub total_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferError {
    pub nonce: String,
    pub filename: String,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct OfferResult {
    pub nonce: String,
    pub hash_total: String,
}

#[tauri::command]
pub async fn send_file_offer(app: AppHandle, file_path: String) -> Result<OfferResult, String> {
    eprintln!("🚀 send_file_offer called with file_path: {}", file_path);

    let _peer_ip = {
        if let Some(s) = app.try_state::<SharedSession>() {
            let state: tauri::State<'_, SharedSession> = s;
            let curr = state.inner().active_connection.lock().await;
            curr.ip.clone()
        } else {
            None
        }
    };

    // Menggunakan JNI stream untuk membuka file (Desktop & Android)
    let file = match open_file_stream(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ send_file_offer open error: {}", e);
            return Err(format!("File not found or cannot open: {}", e));
        }
    };

    let filename = filename_from_path(&file_path);
    let total_size = get_file_size_for_path(&file_path, &file, "send_file_offer").await?;
    let num_chunks = (total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;
    let hash_total = String::new();

    eprintln!(
        "📦 send_file_offer: File size: {}, chunks: {}",
        total_size, num_chunks
    );

    let real_nonce = uuid::Uuid::new_v4().to_string();

    if let Some(s) = app.try_state::<SharedSession>() {
        let state: tauri::State<'_, SharedSession> = s;
        let sender_guard = state.inner().ws_sender.lock().await;
        if let Some(tx) = sender_guard.as_ref() {
            let msg = WsMessage::FileOffer {
                name: filename,
                size: total_size,
                hash_total: hash_total.clone(),
                num_chunks,
                nonce: real_nonce.clone(),
            };
            eprintln!("✅ send_file_offer: Sending WsMessage::FileOffer to channel");
            let _ = tx.send(msg).await;
            return Ok(OfferResult {
                nonce: real_nonce,
                hash_total,
            });
        } else {
            eprintln!("❌ send_file_offer: ws_sender is None (not connected)");
        }
    } else {
        eprintln!("❌ send_file_offer: SharedSession state not found");
    }
    Err("Not connected".to_string())
}

#[tauri::command]
pub async fn start_transfer_server(app: AppHandle) -> Result<(), String> {
    let identity = security::get_or_create_identity(&app)?;
    let server_config = security::generate_server_config(&identity)?;
    let acceptor = TlsAcceptor::from(server_config);

    let addr = format!("0.0.0.0:{}", PORT);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!(
                    "✅ Transfer server already listening on {} (HMR active)",
                    addr
                );
                return Ok(());
            }
            return Err(e.to_string());
        }
    };

    tokio::spawn(async move {
        while let Ok((stream, _peer_addr)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let app = app.clone();

            let task_id = uuid::Uuid::new_v4().to_string();
            tokio::spawn(async move {
                eprintln!("[SERVER] [TASK_START] id={}", task_id);
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        eprintln!("[SERVER] [TLS_ACCEPT_SUCCESS] id={}", task_id);
                        let res = handle_receive(tls_stream, app).await;
                        eprintln!("[SERVER] [TASK_FINISHED] id={}, result={:?}", task_id, res);
                    }
                    Err(e) => {
                        eprintln!("[SERVER] [TLS_ACCEPT_ERROR] id={}, error={}", task_id, e);
                    }
                }
            });
        }
    });

    Ok(())
}

async fn handle_receive<S>(mut stream: S, app: AppHandle) -> std::io::Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    eprintln!("[RECEIVER] [STREAM_OPENED]");
    let mut header_len_bytes = [0u8; 4];
    if let Err(e) = stream.read_exact(&mut header_len_bytes).await {
        eprintln!("[RECEIVER] [READ_ERROR] op=header_len, error={}", e);
        return Err(e);
    }
    let header_len = u32::from_be_bytes(header_len_bytes) as usize;

    // 🔒 Phase 3: Limit header length to prevent memory exhaustion
    const MAX_HEADER_LEN: usize = 64 * 1024;
    if header_len > MAX_HEADER_LEN {
        let err_msg = format!(
            "Protocol header too large: {} bytes (max {})",
            header_len, MAX_HEADER_LEN
        );
        eprintln!("[RECEIVER] [SECURITY_VIOLATION] {}", err_msg);
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: "unknown".to_string(),
                filename: "unknown".to_string(),
                error: err_msg,
            },
        );
        return Err(IoError::new(ErrorKind::InvalidData, "Header too large"));
    }

    let mut header_buf = vec![0u8; header_len];
    if let Err(e) = stream.read_exact(&mut header_buf).await {
        eprintln!("[RECEIVER] [READ_ERROR] op=header_content, error={}", e);
        return Err(e);
    }

    #[derive(Deserialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
    }

    let init: TransferInit = serde_json::from_slice(&header_buf)?;
    let transfer_id = init.nonce.clone();

    // 🔒 Phase 3: Comprehensive Metadata Validation
    let safe_filename = std::path::Path::new(&init.filename)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned());

    let validation_error = if init.filename.trim().is_empty() {
        Some("Filename cannot be empty")
    } else if safe_filename.is_none() {
        Some("Invalid filename format")
    } else if init.total_size == 0 {
        Some("File size must be greater than zero")
    } else {
        // Validate num_chunks matches total_size / CHUNK_SIZE
        let expected_chunks = (init.total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;
        if init.num_chunks != expected_chunks {
            Some("Chunk count mismatch with file size")
        } else {
            None
        }
    };

    if let Some(err_msg) = validation_error {
        eprintln!(
            "[RECEIVER] [VALIDATION_ERROR] nonce={}, error={}",
            transfer_id, err_msg
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: transfer_id,
                filename: init.filename,
                error: format!("Invalid metadata: {}", err_msg),
            },
        );
        return Err(IoError::new(ErrorKind::InvalidData, err_msg));
    }

    let safe_filename = safe_filename.unwrap(); // Guaranteed safe now

    // 🔒 HARDENING: Validate nonce authorization
    {
        let mut registry = ACCEPTED_NONCES.lock().await;
        if !registry.remove(&transfer_id) {
            eprintln!(
                "🛑 Unauthorized transfer attempt: nonce={} is not in registry",
                transfer_id
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: transfer_id.clone(),
                    filename: safe_filename.clone(),
                    error: "Unauthorized transfer: invalid or reused nonce".to_string(),
                },
            );
            return Err(IoError::new(
                ErrorKind::PermissionDenied,
                "Unauthorized transfer",
            ));
        }
        eprintln!("🔒 Transfer authorized and nonce consumed: {}", transfer_id);
    }

    let start_total = std::time::Instant::now();

    eprintln!(
        "[RECEIVER] [{}] [{:?}] transfer_init_received: filename={}, size={}, chunks={}",
        transfer_id, start_total, safe_filename, init.total_size, init.num_chunks
    );

    let mut download_dir = app
        .path()
        .document_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("ArSend");

    if let Ok(store) = app.store("arsend_settings.json") {
        let configured_folder = store
            .get("download_folder")
            .or_else(|| store.get("save_folder"));

        if let Some(folder_str) =
            configured_folder.and_then(|folder_val| folder_val.as_str().map(str::to_owned))
        {
            download_dir = PathBuf::from(folder_str);
        }
    }

    #[cfg(target_os = "android")]
    {
        download_dir = app
            .path()
            .cache_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
    }

    tokio::fs::create_dir_all(&download_dir).await?;

    let final_path = get_unique_path(download_dir.join(&safe_filename));
    let temp_path = download_dir.join(format!("{}.part-{}", safe_filename, transfer_id));

    if final_path.is_symlink() || temp_path.is_symlink() {
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: "Symlinks not allowed".to_string(),
            },
        );
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "Symlinks not allowed",
        ));
    }

    let mut file = match File::create(&temp_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "[RECEIVER] [{}] [FILE_CREATE_ERROR] path={}, error={}",
                transfer_id,
                temp_path.display(),
                e
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: init.nonce.clone(),
                    filename: safe_filename.clone(),
                    error: e.to_string(),
                },
            );
            return Err(e);
        }
    };

    let mut hasher_total = Sha256::new();
    let mut received_bytes_val = 0u64;
    let received_bytes_atomic = Arc::new(AtomicU64::new(0));

    let start_time = std::time::Instant::now();

    // 🚀 START DECOUPLED PROGRESS REPORTER TASK
    let reporter_app = app.clone();
    let reporter_nonce = init.nonce.clone();
    let reporter_filename = format!("recv_{}", safe_filename);
    let reporter_total_size = init.total_size;
    let reporter_counter = Arc::clone(&received_bytes_atomic);
    let reporter_start_time = start_time.clone();

    let reporter_handle = tokio::spawn(async move {
        eprintln!("[PROGRESS_TASK] started for receiver: {}", reporter_nonce);
        let interval = Duration::from_millis(150);
        loop {
            tokio::time::sleep(interval).await;
            let current_bytes = reporter_counter.load(Ordering::Relaxed);
            if current_bytes >= reporter_total_size {
                break;
            }

            let elapsed = reporter_start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                (current_bytes as f64 / 1_048_576.0) / elapsed
            } else {
                0.0
            };
            let progress = (current_bytes as f64 / reporter_total_size as f64) * 100.0;

            let _ = reporter_app.emit(
                "transfer-progress-receive",
                TransferProgress {
                    nonce: reporter_nonce.clone(),
                    filename: reporter_filename.clone(),
                    progress,
                    speed_mb_s: speed,
                    sent_bytes: current_bytes,
                    total_bytes: reporter_total_size,
                },
            );
        }
        eprintln!("[PROGRESS_TASK] stopped for receiver: {}", reporter_nonce);
    });

    let mut data_buf = vec![0u8; CHUNK_SIZE];
    let mut chunk_index = 0u32;
    let mut bytes_since_chunk_ack = 0u64;

    // Helper to cleanup temp file on error
    let cleanup_temp = || {
        let p = temp_path.clone();
        tokio::spawn(async move {
            let _ = tokio::fs::remove_file(p).await;
        });
    };

    while received_bytes_val < init.total_size {
        if take_cancel_signal(&init.nonce).await {
            eprintln!(
                "[RECEIVER] [{}] [TRANSFER_CANCELLED] filename={}",
                transfer_id, safe_filename
            );
            reporter_handle.abort();
            cleanup_temp();
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: init.nonce.clone(),
                    filename: safe_filename.clone(),
                    error: "Transfer cancelled by user".to_string(),
                },
            );
            return Err(IoError::new(
                ErrorKind::Interrupted,
                "Transfer cancelled by user",
            ));
        }

        let remaining_bytes = init.total_size - received_bytes_val;
        let read_len = std::cmp::min(data_buf.len() as u64, remaining_bytes) as usize;
        let n = match tokio::time::timeout(
            Duration::from_secs(30),
            stream.read(&mut data_buf[..read_len]),
        )
        .await
        {
            Err(_) => {
                let err_msg = format!(
                    "Receive timeout: received {} of {} bytes",
                    received_bytes_val, init.total_size
                );
                eprintln!("[RECEIVER] [{}] [READ_TIMEOUT] {}", transfer_id, err_msg);
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: init.nonce.clone(),
                        filename: safe_filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(IoError::new(ErrorKind::TimedOut, err_msg));
            }
            Ok(Ok(0)) => {
                let err_msg = format!(
                    "Connection closed before transfer completed: received {} of {} bytes",
                    received_bytes_val, init.total_size
                );
                eprintln!("[RECEIVER] [{}] [UNEXPECTED_EOF] {}", transfer_id, err_msg);
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: init.nonce.clone(),
                        filename: safe_filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(IoError::new(ErrorKind::UnexpectedEof, err_msg));
            }
            Ok(Ok(n)) => n,
            Ok(Err(e)) => {
                let err_msg = format!(
                    "Receive read error after {} of {} bytes: {}",
                    received_bytes_val, init.total_size, e
                );
                eprintln!(
                    "[RECEIVER] [{}] [READ_ERROR] op=stream_data, {}",
                    transfer_id, err_msg
                );
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: init.nonce.clone(),
                        filename: safe_filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(IoError::new(e.kind(), err_msg));
            }
        };

        if let Err(e) = file.write_all(&data_buf[..n]).await {
            eprintln!(
                "[RECEIVER] [{}] [DISK_WRITE_ERROR] bytes={}, error={}",
                transfer_id, received_bytes_val, e
            );
            reporter_handle.abort();
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: init.nonce.clone(),
                    filename: safe_filename.clone(),
                    error: e.to_string(),
                },
            );
            return Err(e);
        }

        hasher_total.update(&data_buf[..n]);
        received_bytes_val += n as u64;
        bytes_since_chunk_ack += n as u64;
        received_bytes_atomic.store(received_bytes_val, Ordering::Relaxed);

        if bytes_since_chunk_ack >= CHUNK_SIZE as u64 || received_bytes_val >= init.total_size {
            if let Err(e) = stream.write_all(CHUNK_ACK_OK).await {
                let err_msg = format!(
                    "Failed to acknowledge chunk {} after {} of {} bytes: {}",
                    chunk_index, received_bytes_val, init.total_size, e
                );
                eprintln!("[RECEIVER] [{}] [CHUNK_ACK_ERROR] {}", transfer_id, err_msg);
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: init.nonce.clone(),
                        filename: safe_filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(IoError::new(e.kind(), err_msg));
            }
            if let Err(e) = stream.flush().await {
                let err_msg = format!(
                    "Failed to flush chunk {} acknowledgment after {} of {} bytes: {}",
                    chunk_index, received_bytes_val, init.total_size, e
                );
                eprintln!(
                    "[RECEIVER] [{}] [CHUNK_ACK_FLUSH_ERROR] {}",
                    transfer_id, err_msg
                );
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: init.nonce.clone(),
                        filename: safe_filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(IoError::new(e.kind(), err_msg));
            }
            eprintln!(
                "[RECEIVER] [{}] [CHUNK_ACK_SENT] index={}, chunk_bytes={}, received_bytes={}",
                transfer_id, chunk_index, bytes_since_chunk_ack, received_bytes_val
            );
            chunk_index = chunk_index.saturating_add(1);
            bytes_since_chunk_ack = 0;
        }
    }

    eprintln!(
        "[RECEIVER] [{}] [RECV_BYTES_DONE] expected_size={}, actual_received={}",
        transfer_id, init.total_size, received_bytes_val
    );

    let mut expected_hash = [0u8; 32];
    if let Err(e) = stream.read_exact(&mut expected_hash).await {
        eprintln!(
            "[RECEIVER] [{}] [READ_ERROR] op=final_hash, error={}",
            transfer_id, e
        );
        reporter_handle.abort();
        cleanup_temp();
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e);
    }

    reporter_handle.abort(); // Pastikan task polling berhenti sebelum emit final 100%
    eprintln!("[PROGRESS_TASK] stopped for receiver: {}", transfer_id);

    let final_hash = hasher_total.finalize();
    if final_hash.as_slice() != expected_hash {
        eprintln!(
            "[RECEIVER] [{}] [TOTAL_HASH_MISMATCH] expected={}, actual={}",
            transfer_id,
            hex::encode(expected_hash),
            hex::encode(final_hash)
        );
        cleanup_temp();
        let err_msg = "Total file hash mismatch".to_string();
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: err_msg.clone(),
            },
        );
        return Err(IoError::new(ErrorKind::InvalidData, err_msg));
    }

    if let Err(e) = file.flush().await {
        eprintln!(
            "[RECEIVER] [{}] [DISK_FLUSH_ERROR] path={}, error={}",
            transfer_id,
            temp_path.display(),
            e
        );
        cleanup_temp();
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e);
    }

    // 🏆 SUCCESS: Rename temporary file to final path
    drop(file); // Close file before renaming
    if let Err(e) = tokio::fs::rename(&temp_path, &final_path).await {
        eprintln!(
            "[RECEIVER] [{}] [RENAME_ERROR] from={}, to={}, error={}",
            transfer_id,
            temp_path.display(),
            final_path.display(),
            e
        );
        cleanup_temp();
        return Err(e);
    }

    if let Err(e) = stream.write_all(TRANSFER_ACK_OK).await {
        eprintln!(
            "[RECEIVER] [{}] [WRITE_ERROR] op=transfer_ack, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: format!("Failed to acknowledge completed transfer: {}", e),
            },
        );
        return Err(e);
    }
    if let Err(e) = stream.flush().await {
        eprintln!(
            "[RECEIVER] [{}] [FLUSH_ERROR] op=transfer_ack, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: format!("Failed to flush completed transfer acknowledgment: {}", e),
            },
        );
        return Err(e);
    }
    eprintln!("[RECEIVER] [{}] [TRANSFER_ACK_SENT]", transfer_id);

    let _ = app.emit(
        "transfer-progress-receive",
        TransferProgress {
            nonce: init.nonce.clone(),
            filename: format!("recv_{}", safe_filename),
            progress: 100.0,
            speed_mb_s: 0.0,
            sent_bytes: init.total_size,
            total_bytes: init.total_size,
        },
    );

    let _ = app.emit(
        "transfer-complete",
        TransferComplete {
            nonce: init.nonce.clone(),
            filename: safe_filename.clone(),
            save_path: final_path.to_string_lossy().into_owned(),
            is_receive: true,
            total_bytes: init.total_size,
        },
    );

    eprintln!(
        "[RECEIVER] [{}] [TRANSFER_COMPLETED] duration={:?}, path={}",
        transfer_id,
        start_total.elapsed(),
        final_path.display()
    );
    eprintln!("[RECEIVER] [STREAM_CLOSED]");
    Ok(())
}

#[tauri::command]
pub async fn send_file(
    app: AppHandle,
    ip: String,
    file_path: String,
    nonce: String,
) -> Result<(), String> {
    eprintln!("🚀 send_file started for IP: {}, file: {}", ip, file_path);

    let file_for_meta = match open_file_stream(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "[SENDER] [{}] [FILE_OPEN_ERROR] path={}, error={}",
                nonce, file_path, e
            );
            return Err(e);
        }
    };

    let filename = filename_from_path(&file_path);
    let total_size = match get_file_size_for_path(&file_path, &file_for_meta, "send_file").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!(
                "[SENDER] [{}] [METADATA_ERROR] filename={}, error={}",
                nonce, filename, e
            );
            return Err(e);
        }
    };
    let num_chunks = (total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;

    let fingerprint = {
        let session = app
            .try_state::<SharedSession>()
            .ok_or("No active session found")?;
        let state: tauri::State<'_, SharedSession> = session;
        let conn = state.inner().active_connection.lock().await;
        conn.public_key
            .clone()
            .ok_or("No public key in active connection")?
    };

    let client_config = match security::generate_client_config(fingerprint) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[SENDER] [{}] [TLS_CONFIG_ERROR] error={}", nonce, e);
            return Err(e);
        }
    };
    let connector = TlsConnector::from(client_config);

    let clean_ip = if ip.starts_with("::ffff:") {
        ip.replace("::ffff:", "")
    } else {
        ip
    };

    let addr = format!("{}:{}", clean_ip, PORT);
    eprintln!("🔌 send_file: connecting to {}", addr);
    let tcp_stream = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            eprintln!(
                "[SENDER] [{}] [TCP_CONNECT_ERROR] addr={}, error={}",
                nonce, addr, e
            );
            return Err(e.to_string());
        }
        Err(_) => {
            eprintln!("[SENDER] [{}] [TCP_CONNECT_TIMEOUT] addr={}", nonce, addr);
            return Err("Connection timed out".to_string());
        }
    };

    let domain = ServerName::try_from("arsend.local").unwrap().to_owned();
    let mut tls_stream = match connector.connect(domain, tcp_stream).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!(
                "[SENDER] [{}] [TLS_CONNECT_ERROR] addr={}, error={}",
                nonce, addr, e
            );
            return Err(e.to_string());
        }
    };

    eprintln!("[SENDER] [STREAM_OPENED] addr={}", addr);
    let transfer_id = nonce.clone();
    let start_total = std::time::Instant::now();

    #[derive(Serialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
    }

    let init = TransferInit {
        nonce: nonce.clone(),
        filename: filename.clone(),
        total_size,
        num_chunks,
    };

    // 🚀 EMIT INITIAL PROGRESS SO UI APPEARS IMMEDIATELY
    let _ = app.emit(
        "transfer-progress-send",
        TransferProgress {
            nonce: nonce.clone(),
            filename: filename.clone(),
            progress: 0.0,
            speed_mb_s: 0.0,
            sent_bytes: 0,
            total_bytes: total_size,
        },
    );

    let init_bytes = match serde_json::to_vec(&init) {
        Ok(b) => b,
        Err(e) => {
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: e.to_string(),
                },
            );
            return Err(e.to_string());
        }
    };

    if let Err(e) = tls_stream
        .write_all(&(init_bytes.len() as u32).to_be_bytes())
        .await
    {
        eprintln!(
            "[SENDER] [{}] [WRITE_ERROR] op=init_len, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: nonce.clone(),
                filename: filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e.to_string());
    }
    if let Err(e) = tls_stream.write_all(&init_bytes).await {
        eprintln!(
            "[SENDER] [{}] [WRITE_ERROR] op=init_content, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: nonce.clone(),
                filename: filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e.to_string());
    }
    if let Err(e) = tls_stream.flush().await {
        eprintln!(
            "[SENDER] [{}] [FLUSH_ERROR] op=init, error={}",
            transfer_id, e
        );
    }

    eprintln!(
        "[SENDER] [{}] [{:?}] transfer_init_sent: filename={}, size={}, chunks={}",
        transfer_id,
        start_total.elapsed(),
        filename,
        total_size,
        num_chunks
    );

    let mut file = match open_file_stream(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "[SENDER] [{}] [FILE_READ_ERROR] path={}, error={}",
                transfer_id, file_path, e
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: e.to_string(),
                },
            );
            return Err(e.to_string());
        }
    };

    let mut sent_bytes_val = 0u64;
    let sent_bytes_atomic = Arc::new(AtomicU64::new(0));
    let start_time = std::time::Instant::now();

    // 🚀 START DECOUPLED PROGRESS REPORTER TASK
    let reporter_app = app.clone();
    let reporter_nonce = nonce.clone();
    let reporter_filename = filename.clone();
    let reporter_total_size = total_size;
    let reporter_counter = Arc::clone(&sent_bytes_atomic);
    let reporter_start_time = start_time.clone();

    let reporter_handle = tokio::spawn(async move {
        eprintln!("[PROGRESS_TASK] started for sender: {}", reporter_nonce);
        let interval = Duration::from_millis(150);
        loop {
            tokio::time::sleep(interval).await;
            let current_bytes = reporter_counter.load(Ordering::Relaxed);
            if current_bytes >= reporter_total_size {
                break;
            }

            let elapsed = reporter_start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                (current_bytes as f64 / 1_048_576.0) / elapsed
            } else {
                0.0
            };
            let progress = (current_bytes as f64 / reporter_total_size as f64) * 100.0;

            let _ = reporter_app.emit(
                "transfer-progress-send",
                TransferProgress {
                    nonce: reporter_nonce.clone(),
                    filename: reporter_filename.clone(),
                    progress,
                    speed_mb_s: speed,
                    sent_bytes: current_bytes,
                    total_bytes: reporter_total_size,
                },
            );
        }
        eprintln!("[PROGRESS_TASK] stopped for sender: {}", reporter_nonce);
    });

    let mut chunk_buf = vec![0u8; CHUNK_SIZE];
    let mut hasher_total = Sha256::new();
    let mut chunk_index = 0u32;
    while sent_bytes_val < total_size {
        if take_cancel_signal(&nonce).await {
            eprintln!(
                "[SENDER] [{}] [TRANSFER_CANCELLED] filename={}",
                transfer_id, filename
            );
            reporter_handle.abort();
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: "Transfer cancelled by user".to_string(),
                },
            );
            return Err("Transfer cancelled by user".to_string());
        }

        let remaining_bytes = total_size - sent_bytes_val;
        let read_len = std::cmp::min(chunk_buf.len() as u64, remaining_bytes) as usize;
        let n = match file.read(&mut chunk_buf[..read_len]).await {
            Ok(0) => {
                reporter_handle.abort();
                let err_msg = format!(
                    "File ended early while sending: expected {} bytes, sent {}",
                    total_size, sent_bytes_val
                );
                eprintln!("[SENDER] [{}] [UNEXPECTED_EOF] {}", transfer_id, err_msg);
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: nonce.clone(),
                        filename: filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(err_msg);
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!(
                    "[SENDER] [{}] [DISK_READ_ERROR] bytes={}, error={}",
                    transfer_id, sent_bytes_val, e
                );
                reporter_handle.abort();
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: nonce.clone(),
                        filename: filename.clone(),
                        error: e.to_string(),
                    },
                );
                return Err(e.to_string());
            }
        };

        hasher_total.update(&chunk_buf[..n]);
        if let Err(e) = tls_stream.write_all(&chunk_buf[..n]).await {
            eprintln!(
                "[SENDER] [{}] [WRITE_ERROR] op=stream_data, bytes={}, error={}",
                transfer_id, sent_bytes_val, e
            );
            reporter_handle.abort();
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: e.to_string(),
                },
            );
            return Err(e.to_string());
        }

        sent_bytes_val += n as u64;
        sent_bytes_atomic.store(sent_bytes_val, Ordering::Relaxed);

        if let Err(e) = tls_stream.flush().await {
            reporter_handle.abort();
            let err_msg = format!("Failed to flush chunk {}: {}", chunk_index, e);
            eprintln!("[SENDER] [{}] [CHUNK_FLUSH_ERROR] {}", transfer_id, err_msg);
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: err_msg.clone(),
                },
            );
            return Err(err_msg);
        }

        let mut chunk_ack = [0u8; 7];
        match tokio::time::timeout(
            Duration::from_secs(30),
            tls_stream.read_exact(&mut chunk_ack),
        )
        .await
        {
            Ok(Ok(_)) if &chunk_ack == CHUNK_ACK_OK => {
                eprintln!(
                    "[SENDER] [{}] [CHUNK_ACK_RECEIVED] index={}, sent_bytes={}",
                    transfer_id, chunk_index, sent_bytes_val
                );
            }
            Ok(Ok(_)) => {
                reporter_handle.abort();
                let err_msg = format!(
                    "Unexpected chunk acknowledgment for chunk {}: {:?}",
                    chunk_index, chunk_ack
                );
                eprintln!("[SENDER] [{}] [CHUNK_ACK_INVALID] {}", transfer_id, err_msg);
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: nonce.clone(),
                        filename: filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(err_msg);
            }
            Ok(Err(e)) => {
                reporter_handle.abort();
                let err_msg = format!("Failed waiting for chunk {} ack: {}", chunk_index, e);
                eprintln!("[SENDER] [{}] [CHUNK_ACK_ERROR] {}", transfer_id, err_msg);
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: nonce.clone(),
                        filename: filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(err_msg);
            }
            Err(_) => {
                reporter_handle.abort();
                let err_msg = format!("Timed out waiting for chunk {} ack", chunk_index);
                eprintln!("[SENDER] [{}] [CHUNK_ACK_TIMEOUT] {}", transfer_id, err_msg);
                let _ = app.emit(
                    "transfer-error",
                    TransferError {
                        nonce: nonce.clone(),
                        filename: filename.clone(),
                        error: err_msg.clone(),
                    },
                );
                return Err(err_msg);
            }
        }

        chunk_index = chunk_index.saturating_add(1);
    }

    eprintln!(
        "[SENDER] [{}] [SEND_BYTES_DONE] declared_size={}, actual_sent={}",
        transfer_id, total_size, sent_bytes_val
    );

    let final_hash = hasher_total.finalize();
    if let Err(e) = tls_stream.write_all(final_hash.as_slice()).await {
        reporter_handle.abort();
        eprintln!(
            "[SENDER] [{}] [WRITE_ERROR] op=final_hash, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: nonce.clone(),
                filename: filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e.to_string());
    }

    if let Err(e) = tls_stream.flush().await {
        reporter_handle.abort();
        eprintln!(
            "[SENDER] [{}] [FLUSH_ERROR] op=final, error={}",
            transfer_id, e
        );
        let _ = app.emit(
            "transfer-error",
            TransferError {
                nonce: nonce.clone(),
                filename: filename.clone(),
                error: e.to_string(),
            },
        );
        return Err(e.to_string());
    }

    let mut transfer_ack = [0u8; 7];
    match tokio::time::timeout(
        Duration::from_secs(30),
        tls_stream.read_exact(&mut transfer_ack),
    )
    .await
    {
        Ok(Ok(_)) if &transfer_ack == TRANSFER_ACK_OK => {
            eprintln!("[SENDER] [{}] [TRANSFER_ACK_RECEIVED]", transfer_id);
        }
        Ok(Ok(_)) => {
            reporter_handle.abort();
            let err_msg = format!("Unexpected transfer acknowledgment: {:?}", transfer_ack);
            eprintln!(
                "[SENDER] [{}] [TRANSFER_ACK_INVALID] {}",
                transfer_id, err_msg
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: err_msg.clone(),
                },
            );
            return Err(err_msg);
        }
        Ok(Err(e)) => {
            reporter_handle.abort();
            let err_msg = format!("Failed waiting for receiver completion ack: {}", e);
            eprintln!(
                "[SENDER] [{}] [TRANSFER_ACK_ERROR] {}",
                transfer_id, err_msg
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: err_msg.clone(),
                },
            );
            return Err(err_msg);
        }
        Err(_) => {
            reporter_handle.abort();
            let err_msg = "Timed out waiting for receiver completion ack".to_string();
            eprintln!(
                "[SENDER] [{}] [TRANSFER_ACK_TIMEOUT] {}",
                transfer_id, err_msg
            );
            let _ = app.emit(
                "transfer-error",
                TransferError {
                    nonce: nonce.clone(),
                    filename: filename.clone(),
                    error: err_msg.clone(),
                },
            );
            return Err(err_msg);
        }
    }

    match tokio::time::timeout(Duration::from_secs(5), tls_stream.shutdown()).await {
        Ok(Ok(())) => {
            eprintln!("[SENDER] [{}] [TLS_SHUTDOWN_SENT]", transfer_id);
        }
        Ok(Err(e)) => {
            eprintln!(
                "[SENDER] [{}] [TLS_SHUTDOWN_ERROR] close_notify failed: {}",
                transfer_id, e
            );
        }
        Err(e) => {
            eprintln!(
                "[SENDER] [{}] [TLS_SHUTDOWN_TIMEOUT] close_notify did not finish: {}",
                transfer_id, e
            );
        }
    }

    reporter_handle.abort(); // Pastikan task polling berhenti sebelum emit final 100%
    eprintln!("[PROGRESS_TASK] stopped for sender: {}", transfer_id);

    let _ = app.emit(
        "transfer-progress-send",
        TransferProgress {
            nonce: nonce.clone(),
            filename: filename.clone(),
            progress: 100.0,
            speed_mb_s: 0.0,
            sent_bytes: total_size,
            total_bytes: total_size,
        },
    );

    let _ = app.emit(
        "transfer-complete",
        TransferComplete {
            nonce: nonce.clone(),
            filename: filename.clone(),
            save_path: file_path.clone(),
            is_receive: false,
            total_bytes: total_size,
        },
    );

    eprintln!(
        "[SENDER] [{}] [TRANSFER_COMPLETED] duration={:?}",
        transfer_id,
        start_total.elapsed()
    );
    eprintln!("[SENDER] [STREAM_CLOSED]");
    Ok(())
}
