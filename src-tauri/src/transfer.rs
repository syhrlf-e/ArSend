use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_rustls::rustls::pki_types::ServerName;
use sha2::{Sha256, Digest};
use crate::security;
use std::io::Error as IoError;
use std::io::ErrorKind;

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks
const PORT: u16 = 9528;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkHeader {
    pub index: u32,
    pub hash: String,
    pub length: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransferProgress {
    pub filename: String,
    pub progress: f64,
    pub speed_mb_s: f64,
    pub sent_bytes: u64,
    pub total_bytes: u64,
}

#[tauri::command]
pub async fn start_transfer_server(app: AppHandle) -> Result<(), String> {
    let (server_config, _) = security::generate_tls_config()?;
    let acceptor = TlsAcceptor::from(server_config);
    
    let addr = format!("0.0.0.0:{}", PORT);
    let listener = TcpListener::bind(&addr).await.map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        while let Ok((stream, _peer_addr)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let app = app.clone();
            
            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    let _ = handle_receive(tls_stream, app).await;
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
    // Need to handshake filename, file size, hash total...
    // In our protocol, this is negotiated over WS (FILE_OFFER), 
    // and then the TCP stream is opened for raw chunks.
    // For simplicity here we just read chunks.
    // In a real app we'd need a way to correlate the TCP stream to the accepted offer.
    // We'll read a JSON header first identifying the transfer session (nonce).
    
    let mut header_len_bytes = [0u8; 4];
    stream.read_exact(&mut header_len_bytes).await?;
    let header_len = u32::from_be_bytes(header_len_bytes) as usize;
    
    let mut header_buf = vec![0u8; header_len];
    stream.read_exact(&mut header_buf).await?;
    
    // Assume header contains filename and file size.
    #[derive(Deserialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
    }
    
    let init: TransferInit = serde_json::from_slice(&header_buf)?;
    
    // Create download folder
    let download_dir = dirs::document_dir().unwrap_or_else(|| PathBuf::from(".")).join("ArSend");
    tokio::fs::create_dir_all(&download_dir).await?;
    
    // Basic sanitization
    let safe_filename = std::path::Path::new(&init.filename).file_name().unwrap_or_default();
    let file_path = download_dir.join(safe_filename);
    
    let mut file = File::create(&file_path).await?;
    let mut hasher_total = Sha256::new();
    let mut received_bytes = 0u64;

    let start_time = std::time::Instant::now();

    for i in 0..init.num_chunks {
        // Read chunk header
        let mut cl_bytes = [0u8; 4];
        stream.read_exact(&mut cl_bytes).await?;
        let cl = u32::from_be_bytes(cl_bytes) as usize;
        
        let mut ch_buf = vec![0u8; cl];
        stream.read_exact(&mut ch_buf).await?;
        let chunk_header: ChunkHeader = serde_json::from_slice(&ch_buf)?;
        
        // Read chunk data
        let mut data_buf = vec![0u8; chunk_header.length as usize];
        stream.read_exact(&mut data_buf).await?;
        
        // Verify hash
        let mut chunk_hasher = Sha256::new();
        chunk_hasher.update(&data_buf);
        let hash = hex::encode(chunk_hasher.finalize());
        
        if hash != chunk_header.hash {
            // Write a negative ACK (for retry logic)
            stream.write_all(b"RTRY").await?;
            // A robust implementation would handle retry loops here
            return Err(IoError::new(ErrorKind::InvalidData, "Chunk hash mismatch"));
        } else {
            stream.write_all(b"O_OK").await?;
        }
        
        file.write_all(&data_buf).await?;
        hasher_total.update(&data_buf);
        received_bytes += data_buf.len() as u64;

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed = if elapsed > 0.0 {
            (received_bytes as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };

        let progress = (received_bytes as f64 / init.total_size as f64) * 100.0;
        
        let _ = app.emit("transfer-progress-receive", TransferProgress {
            filename: init.filename.clone(),
            progress,
            speed_mb_s: speed,
            sent_bytes: received_bytes,
            total_bytes: init.total_size,
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn send_file(app: AppHandle, ip: String, file_path: String, nonce: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err("File not found".to_string());
    }
    
    let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let metadata = tokio::fs::metadata(&path).await.map_err(|e| e.to_string())?;
    let total_size = metadata.len();
    let num_chunks = (total_size as f64 / CHUNK_SIZE as f64).ceil() as u32;

    let (_, client_config) = security::generate_tls_config()?;
    let connector = TlsConnector::from(client_config);
    
    let addr = format!("{}:{}", ip, PORT);
    let tcp_stream = TcpStream::connect(&addr).await.map_err(|e| e.to_string())?;
    
    let domain = ServerName::try_from("arsend.local").unwrap().to_owned();
    let mut tls_stream = connector.connect(domain, tcp_stream).await.map_err(|e| e.to_string())?;

    #[derive(Serialize)]
    struct TransferInit {
        nonce: String,
        filename: String,
        total_size: u64,
        num_chunks: u32,
    }

    let init = TransferInit {
        nonce,
        filename: filename.clone(),
        total_size,
        num_chunks,
    };
    
    let init_bytes = serde_json::to_vec(&init).map_err(|e| e.to_string())?;
    tls_stream.write_all(&(init_bytes.len() as u32).to_be_bytes()).await.map_err(|e| e.to_string())?;
    tls_stream.write_all(&init_bytes).await.map_err(|e| e.to_string())?;

    let mut file = File::open(&path).await.map_err(|e| e.to_string())?;
    let mut sent_bytes = 0u64;
    let start_time = std::time::Instant::now();

    for i in 0..num_chunks {
        let mut chunk_buf = vec![0u8; CHUNK_SIZE];
        let n = file.read(&mut chunk_buf).await.map_err(|e| e.to_string())?;
        chunk_buf.truncate(n);

        let mut hasher = Sha256::new();
        hasher.update(&chunk_buf);
        let hash = hex::encode(hasher.finalize());

        let header = ChunkHeader {
            index: i,
            hash,
            length: n as u32,
        };
        
        let header_bytes = serde_json::to_vec(&header).map_err(|e| e.to_string())?;
        tls_stream.write_all(&(header_bytes.len() as u32).to_be_bytes()).await.map_err(|e| e.to_string())?;
        tls_stream.write_all(&header_bytes).await.map_err(|e| e.to_string())?;
        tls_stream.write_all(&chunk_buf).await.map_err(|e| e.to_string())?;

        let mut ack = [0u8; 4];
        tls_stream.read_exact(&mut ack).await.map_err(|e| e.to_string())?;
        if &ack != b"O_OK" {
            return Err("Chunk failed".to_string());
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
            filename: filename.clone(),
            progress,
            speed_mb_s: speed,
            sent_bytes,
            total_bytes: total_size,
        });
    }

    Ok(())
}
