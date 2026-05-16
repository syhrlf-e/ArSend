use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_rustls::rustls::pki_types::ServerName;
use sha2::{Sha256, Digest};
use crate::security;
use crate::server::{SharedSession, WsMessage};
use std::io::Error as IoError;
use std::io::ErrorKind;
use tauri_plugin_store::StoreExt;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::time::{Instant, Duration};
use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref CANCELLED_TRANSFERS: tokio::sync::Mutex<HashSet<String>> = tokio::sync::Mutex::new(HashSet::new());
}

#[tauri::command]
pub async fn cancel_transfer(filename: String) -> Result<(), String> {
    eprintln!("🛑 Cancelling transfer for file: {}", filename);
    let mut cancelled = CANCELLED_TRANSFERS.lock().await;
    cancelled.insert(filename);
    Ok(())
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
    let mut env = vm.attach_current_thread().map_err(|e| format!("JNI Env error: {}", e))?;

    // Parse URI
    let uri_jstring = env.new_string(uri).map_err(|e| e.to_string())?;
    let uri_object = JObject::from(uri_jstring);
    let uri_class = env.find_class("android/net/Uri").map_err(|e| e.to_string())?;
    let parsed_uri = env.call_static_method(
        uri_class,
        "parse",
        "(Ljava/lang/String;)Landroid/net/Uri;",
        &[JValue::Object(&uri_object)]
    ).map_err(|e| e.to_string())?.l().map_err(|e| e.to_string())?;

    // Get Application context without relying on ndk_context initialization.
    let activity_thread_class = env.find_class("android/app/ActivityThread").map_err(|e| e.to_string())?;
    let application = env.call_static_method(
        activity_thread_class,
        "currentApplication",
        "()Landroid/app/Application;",
        &[],
    ).map_err(|e| e.to_string())?.l().map_err(|e| e.to_string())?;

    if application.is_null() {
        return Err("Android application context unavailable".to_string());
    }

    // Get ContentResolver
    let content_resolver = env.call_method(
        &application,
        "getContentResolver",
        "()Landroid/content/ContentResolver;",
        &[]
    ).map_err(|e| e.to_string())?.l().map_err(|e| e.to_string())?;

    // Open FileDescriptor (read-only)
    let mode_jstring = env.new_string("r").map_err(|e| e.to_string())?;
    let mode_object = JObject::from(mode_jstring);
    let pfd = env.call_method(
        content_resolver,
        "openFileDescriptor",
        "(Landroid/net/Uri;Ljava/lang/String;)Landroid/os/ParcelFileDescriptor;",
        &[JValue::Object(&parsed_uri), JValue::Object(&mode_object)]
    ).map_err(|e| e.to_string())?.l().map_err(|e| e.to_string())?;

    if pfd.is_null() {
        return Err("Android ContentResolver returned a null file descriptor".to_string());
    }

    // Detach the FD so Rust takes ownership
    let fd = env.call_method(&pfd, "detachFd", "()I", &[]).map_err(|e| e.to_string())?.i().map_err(|e| e.to_string())?;

    // Wrap the raw FD in a standard Rust File
    let std_file = unsafe { std::fs::File::from_raw_fd(fd) };
    Ok(std_file)
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

async fn hash_file_and_size(mut file: File, context: &str) -> Result<(String, u64), String> {
    let mut hasher = Sha256::new();
    let mut total_size = 0u64;
    let mut buf = vec![0u8; 1024 * 1024];

    loop {
        let n = file.read(&mut buf).await.map_err(|e| {
            eprintln!("❌ {} read error: {}", context, e);
            e.to_string()
        })?;
        if n == 0 {
            break;
        }

        total_size += n as u64;
        hasher.update(&buf[..n]);
    }

    Ok((hex::encode(hasher.finalize()), total_size))
}

const CHUNK_SIZE: usize = 2 * 1024 * 1024;
const PORT: u16 = 9528;

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
        let msg = WsMessage::FileAccept { nonce };
        let _ = tx.send(msg).await;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkHeader {
    pub index: u32,
    pub hash: String,
    pub length: u32,
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

lazy_static::lazy_static! {
    static ref RATE_LIMITS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Deserialize)]
pub struct OfferResult {
    pub nonce: String,
    pub hash_total: String,
}

#[tauri::command]
pub async fn send_file_offer(app: AppHandle, file_path: String) -> Result<OfferResult, String> {
    eprintln!("🚀 send_file_offer called with file_path: {}", file_path);

    let peer_ip = {
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
    let (hash_total, total_size) = hash_file_and_size(file, "send_file_offer").await?;
    let num_chunks = (total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;

    eprintln!("📦 send_file_offer: File size: {}, chunks: {}", total_size, num_chunks);

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
                eprintln!("✅ Transfer server already listening on {} (HMR active)", addr);
                return Ok(());
            }
            return Err(e.to_string());
        }
    };

    tokio::spawn(async move {
        while let Ok((stream, _peer_addr)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let app = app.clone();

            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        eprintln!("📥 Transfer server accepted incoming TLS connection");
                        if let Err(e) = handle_receive(tls_stream, app).await {
                            eprintln!("❌ handle_receive error: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Transfer server TLS accept error: {}", e);
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
    let mut header_len_bytes = [0u8; 4];
    stream.read_exact(&mut header_len_bytes).await?;
    let header_len = u32::from_be_bytes(header_len_bytes) as usize;

    let mut header_buf = vec![0u8; header_len];
    stream.read_exact(&mut header_buf).await?;

    #[derive(Deserialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
        hash_total: String,
    }

    let init: TransferInit = serde_json::from_slice(&header_buf)?;

    let mut download_dir = app.path().document_dir().unwrap_or_else(|_| PathBuf::from(".")).join("ArSend");

    if let Ok(store) = app.store("arsend_settings.json") {
        let configured_folder = store
            .get("download_folder")
            .or_else(|| store.get("save_folder"));

        if let Some(folder_str) = configured_folder.and_then(|folder_val| folder_val.as_str().map(str::to_owned)) {
            download_dir = PathBuf::from(folder_str);
        }
    }

    #[cfg(target_os = "android")]
    {
        download_dir = app.path().cache_dir().unwrap_or_else(|_| PathBuf::from("."));
    }

    tokio::fs::create_dir_all(&download_dir).await?;

    let safe_filename = std::path::Path::new(&init.filename).file_name().unwrap_or_default().to_string_lossy().into_owned();
    let file_path = download_dir.join(&safe_filename);

    if file_path.is_symlink() {
        let _ = app.emit("transfer-error", TransferError {
            nonce: init.nonce.clone(),
            filename: safe_filename.clone(),
            error: "Symlinks not allowed".to_string(),
        });
        return Err(IoError::new(ErrorKind::InvalidInput, "Symlinks not allowed"));
    }

    let mut file = match File::create(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            let _ = app.emit("transfer-error", TransferError {
                nonce: init.nonce.clone(),
                filename: safe_filename.clone(),
                error: e.to_string(),
            });
            return Err(e);
        }
    };

    let mut hasher_total = Sha256::new();
    let mut received_bytes = 0u64;

    let start_time = std::time::Instant::now();

    for _i in 0..init.num_chunks {
        let is_cancelled = {
            let mut cancelled = CANCELLED_TRANSFERS.lock().await;
            if cancelled.contains(&safe_filename) {
                cancelled.remove(&safe_filename);
                true
            } else {
                false
            }
        };

        if is_cancelled {
            eprintln!("🛑 handle_receive: Transfer cancelled for {}", safe_filename);
            let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: "Transfer cancelled by user".to_string() });
            return Err(IoError::new(ErrorKind::Interrupted, "Transfer cancelled by user"));
        }

        let mut attempts = 0;
        let max_attempts = 3;
        let mut success = false;

        while attempts < max_attempts {
            let mut cl_bytes = [0u8; 4];
            if let Err(e) = stream.read_exact(&mut cl_bytes).await {
                let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: e.to_string() });
                return Err(e);
            }
            let cl = u32::from_be_bytes(cl_bytes) as usize;

            let mut ch_buf = vec![0u8; cl];
            if let Err(e) = stream.read_exact(&mut ch_buf).await {
                let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: e.to_string() });
                return Err(e);
            }
            let chunk_header: ChunkHeader = serde_json::from_slice(&ch_buf)?;

            let mut data_buf = vec![0u8; chunk_header.length as usize];
            if let Err(e) = stream.read_exact(&mut data_buf).await {
                let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: e.to_string() });
                return Err(e);
            }

            let mut chunk_hasher = Sha256::new();
            chunk_hasher.update(&data_buf);
            let hash = hex::encode(chunk_hasher.finalize());

            if hash != chunk_header.hash {
                attempts += 1;
                let _ = stream.write_all(b"RTRY").await;
                let _ = stream.flush().await;
                if attempts >= max_attempts {
                    let err_msg = format!("Chunk hash mismatch after {} attempts", max_attempts);
                    let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: err_msg.clone() });
                    return Err(IoError::new(ErrorKind::InvalidData, err_msg));
                }
                continue;
            } else {
                if let Err(e) = stream.write_all(b"O_OK").await {
                    let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: e.to_string() });
                    return Err(e);
                }
                if let Err(e) = stream.flush().await {
                    eprintln!("❌ handle_receive flush error: {}", e);
                }

                if let Err(e) = file.write_all(&data_buf).await {
                    let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: e.to_string() });
                    return Err(e);
                }
                hasher_total.update(&data_buf);
                received_bytes += data_buf.len() as u64;
                success = true;
                break;
            }
        }

        if !success {
            return Err(IoError::new(ErrorKind::InvalidData, "Failed to receive chunk"));
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (received_bytes as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };

        let progress = (received_bytes as f64 / init.total_size as f64) * 100.0;

        let _ = app.emit("transfer-progress-receive", TransferProgress {
            nonce: init.nonce.clone(),
            filename: format!("recv_{}", safe_filename),
            progress,
            speed_mb_s: speed,
            sent_bytes: received_bytes,
            total_bytes: init.total_size,
        });
    }

    let final_hash = hex::encode(hasher_total.finalize());
    if final_hash != init.hash_total {
        let err_msg = "Total file hash mismatch".to_string();
        let _ = app.emit("transfer-error", TransferError { nonce: init.nonce.clone(), filename: safe_filename.clone(), error: err_msg.clone() });
        return Err(IoError::new(ErrorKind::InvalidData, err_msg));
    }

    let _ = app.emit("transfer-progress-receive", TransferProgress {
        nonce: init.nonce.clone(),
        filename: format!("recv_{}", safe_filename),
        progress: 100.0,
        speed_mb_s: 0.0,
        sent_bytes: init.total_size,
        total_bytes: init.total_size,
    });

    let _ = app.emit("transfer-complete", TransferComplete {
        nonce: init.nonce.clone(),
        filename: safe_filename.clone(),
        save_path: file_path.to_string_lossy().into_owned(),
        is_receive: true,
        total_bytes: init.total_size,
    });

    Ok(())
}

#[tauri::command]
pub async fn send_file(app: AppHandle, ip: String, file_path: String, nonce: String) -> Result<(), String> {
    eprintln!("🚀 send_file started for IP: {}, file: {}", ip, file_path);

    let file_for_hash = match open_file_stream(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ send_file: failed to open file for hashing: {}", e);
            return Err(e);
        }
    };

    let filename = filename_from_path(&file_path);
    let (hash_total, total_size) = match hash_file_and_size(file_for_hash, "send_file").await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ send_file: failed to hash file: {}", e);
            return Err(e);
        }
    };
    let num_chunks = (total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;

    let fingerprint = {
        let session = app.try_state::<SharedSession>().ok_or("No active session found")?;
        let state: tauri::State<'_, SharedSession> = session;
        let conn = state.inner().active_connection.lock().await;
        conn.public_key.clone().ok_or("No public key in active connection")?
    };

    let client_config = match security::generate_client_config(fingerprint) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ send_file: TLS config error: {}", e);
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
    let tcp_stream = match tokio::time::timeout(std::time::Duration::from_secs(10), TcpStream::connect(&addr)).await {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            eprintln!("❌ send_file: tcp connect error to {}: {}", addr, e);
            return Err(e.to_string());
        }
        Err(_) => {
            eprintln!("❌ send_file: tcp connect timeout to {}", addr);
            return Err("Connection timed out".to_string());
        }
    };

    let domain = ServerName::try_from("arsend.local").unwrap().to_owned();
    let mut tls_stream = match connector.connect(domain, tcp_stream).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("❌ send_file: TLS connect error: {}", e);
            return Err(e.to_string());
        }
    };

    eprintln!("✅ send_file: connected to {}, starting transfer", addr);

    #[derive(Serialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
        hash_total: String,
    }

    let init = TransferInit {
        nonce: nonce.clone(),
        filename: filename.clone(),
        total_size,
        num_chunks,
        hash_total,
    };

    // 🚀 EMIT INITIAL PROGRESS SO UI APPEARS IMMEDIATELY
    let _ = app.emit("transfer-progress-send", TransferProgress {
        nonce: nonce.clone(),
        filename: filename.clone(),
        progress: 0.0,
        speed_mb_s: 0.0,
        sent_bytes: 0,
        total_bytes: total_size,
    });

    let init_bytes = match serde_json::to_vec(&init) {
        Ok(b) => b,
        Err(e) => {
            let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
            return Err(e.to_string());
        }
    };

    if let Err(e) = tls_stream.write_all(&(init_bytes.len() as u32).to_be_bytes()).await {
        let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
        return Err(e.to_string());
    }
    if let Err(e) = tls_stream.write_all(&init_bytes).await {
        let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
        return Err(e.to_string());
    }
    if let Err(e) = tls_stream.flush().await {
        eprintln!("❌ send_file flush error: {}", e);
    }

    let mut file = match open_file_stream(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
            return Err(e.to_string());
        }
    };

    let mut sent_bytes = 0u64;
    let start_time = std::time::Instant::now();

    for i in 0..num_chunks {
        let is_cancelled = {
            let mut cancelled = CANCELLED_TRANSFERS.lock().await;
            if cancelled.contains(&filename) {
                cancelled.remove(&filename);
                true
            } else {
                false
            }
        };

        if is_cancelled {
            eprintln!("🛑 send_file: Transfer cancelled for {}", filename);
            let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: "Transfer cancelled by user".to_string() });
            return Err("Transfer cancelled by user".to_string());
        }

        let remaining_bytes = total_size - sent_bytes;
        let current_chunk_size = std::cmp::min(CHUNK_SIZE as u64, remaining_bytes) as usize;
        
        let mut chunk_buf = vec![0u8; current_chunk_size];
        if let Err(e) = file.read_exact(&mut chunk_buf).await {
            let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
            return Err(e.to_string());
        }
        let n = current_chunk_size;

        let mut hasher = Sha256::new();
        hasher.update(&chunk_buf);
        let hash = hex::encode(hasher.finalize());

        let header = ChunkHeader {
            index: i,
            hash,
            length: n as u32,
        };

        let header_bytes = match serde_json::to_vec(&header) {
            Ok(b) => b,
            Err(e) => {
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
                return Err(e.to_string());
            }
        };

        let mut attempts = 0;
        let max_attempts = 3;
        let mut success = false;

        while attempts < max_attempts {
            if let Err(e) = tls_stream.write_all(&(header_bytes.len() as u32).to_be_bytes()).await {
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
                return Err(e.to_string());
            }
            if let Err(e) = tls_stream.write_all(&header_bytes).await {
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
                return Err(e.to_string());
            }
            if let Err(e) = tls_stream.write_all(&chunk_buf).await {
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
                return Err(e.to_string());
            }
            if let Err(e) = tls_stream.flush().await {
                eprintln!("❌ send_file flush error: {}", e);
            }

            let mut ack = [0u8; 4];
            if let Err(e) = tls_stream.read_exact(&mut ack).await {
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: e.to_string() });
                return Err(e.to_string());
            }

            if &ack == b"O_OK" {
                success = true;
                break;
            } else if &ack == b"RTRY" {
                attempts += 1;
                eprintln!("⚠️ send_file: Chunk {} failed hash verification on receiver, retrying... ({}/{})", i, attempts, max_attempts);
                continue;
            } else {
                let err_msg = "Unknown ACK received".to_string();
                let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: err_msg.clone() });
                return Err(err_msg);
            }
        }

        if !success {
            let err_msg = format!("Chunk {} failed after {} attempts", i, max_attempts);
            let _ = app.emit("transfer-error", TransferError { nonce: nonce.clone(), filename: filename.clone(), error: err_msg.clone() });
            return Err(err_msg);
        }

        sent_bytes += n as u64;
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (sent_bytes as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };

        let progress = (sent_bytes as f64 / total_size as f64) * 100.0;

        let _ = app.emit("transfer-progress-send", TransferProgress {
            nonce: nonce.clone(),
            filename: filename.clone(),
            progress,
            speed_mb_s: speed,
            sent_bytes,
            total_bytes: total_size,
        });
    }

    let _ = app.emit("transfer-progress-send", TransferProgress {
        nonce: nonce.clone(),
        filename: filename.clone(),
        progress: 100.0,
        speed_mb_s: 0.0,
        sent_bytes: total_size,
        total_bytes: total_size,
    });

    let _ = app.emit("transfer-complete", TransferComplete {
        nonce: nonce.clone(),
        filename: filename.clone(),
        save_path: file_path.clone(),
        is_receive: false,
        total_bytes: total_size,
    });

    Ok(())
}
