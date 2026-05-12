use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_tungstenite::tungstenite::Message;
use crate::security;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio::time::{interval, Duration};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionState {
    pub connected: bool,
    pub device_name: Option<String>,
    pub public_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WsMessage {
    Identity { name: String, public_key: String },
    Heartbeat,
    FileOffer { name: String, size: u64, hash_total: String, num_chunks: u32, nonce: String },
    FileAccept { nonce: String },
    FileReject { nonce: String },
}

pub struct SessionManager {
    // In a real app we'd map public keys or IPs to the specific connection senders.
    // For v1, we focus on a single active P2P connection for simplicity, or manage multiple.
    // Here we'll just track if we are connected.
    pub active_connection: Mutex<ConnectionState>,
    app_handle: AppHandle,
}

impl SessionManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            active_connection: Mutex::new(ConnectionState {
                connected: false,
                device_name: None,
                public_key: None,
            }),
            app_handle,
        }
    }

    pub async fn update_state(&self, state: ConnectionState) {
        let mut curr = self.active_connection.lock().await;
        *curr = state.clone();
        let _ = self.app_handle.emit("connection-state-changed", state);
    }
}

pub type SharedSession = Arc<SessionManager>;

#[tauri::command]
pub async fn start_server(app: AppHandle, device_name: String) -> Result<(), String> {
    let (server_config, _) = security::generate_tls_config()?;
    let acceptor = TlsAcceptor::from(server_config);
    
    let listener = TcpListener::bind("0.0.0.0:9527").await.map_err(|e| e.to_string())?;
    
    let session = Arc::new(SessionManager::new(app.clone()));
    app.manage(session.clone());

    tokio::spawn(async move {
        while let Ok((stream, peer_addr)) = listener.accept().await {
            let acceptor = acceptor.clone();
            let session = session.clone();
            let app = app.clone();
            let device_name = device_name.clone();
            
            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    if let Ok(ws_stream) = tokio_tungstenite::accept_async(tls_stream).await {
                        handle_connection(ws_stream, session, app, peer_addr, device_name).await;
                    }
                }
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn connect_to_device(app: AppHandle, ip: String, device_name: String) -> Result<(), String> {
    let session = if let Some(s) = app.try_state::<SharedSession>() {
        let state: tauri::State<'_, SharedSession> = s;
        state.inner().clone()
    } else {
        let s = Arc::new(SessionManager::new(app.clone()));
        app.manage(s.clone());
        s
    };

    let (_, client_config) = security::generate_tls_config()?;
    let connector = TlsConnector::from(client_config);
    
    let addr = format!("{}:9527", ip);
    let stream = tokio::time::timeout(
        Duration::from_secs(5),
        TcpStream::connect(&addr)
    ).await
    .map_err(|_| format!("Connection to {} timed out after 5s", addr))?
    .map_err(|e| e.to_string())?;
    
    let domain = ServerName::try_from("arsend.local").unwrap().to_owned();
    let tls_stream = connector.connect(domain, stream).await.map_err(|e| e.to_string())?;
    
    let request = format!("wss://{}:9527/", ip);
    let (ws_stream, _) = tokio_tungstenite::client_async(request, tls_stream).await.map_err(|e| e.to_string())?;

    let peer_addr = addr.parse().unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    tokio::spawn(async move {
        handle_connection(ws_stream, session, app, peer_addr, device_name).await;
    });

    Ok(())
}

async fn handle_connection<S>(mut ws_stream: S, session: SharedSession, app: AppHandle, _peer: SocketAddr, device_name: String) 
where
    S: SinkExt<Message> + StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'static,
{
    // Send our identity
    let my_identity = security::get_public_key(app.clone()).unwrap_or_default();
    let id_msg = WsMessage::Identity {
        name: device_name,
        public_key: my_identity.public_key_hex,
    };
    
    if let Ok(msg) = serde_json::to_string(&id_msg) {
        let _ = ws_stream.send(Message::Text(msg.into())).await;
    }

    let mut heartbeat = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let hb_msg = WsMessage::Heartbeat;
                if let Ok(msg) = serde_json::to_string(&hb_msg) {
                    if ws_stream.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(Message::Text(txt))) => {
                        if let Ok(parsed) = serde_json::from_str::<WsMessage>(&txt) {
                            match parsed {
                                WsMessage::Identity { name, public_key } => {
                                    session.update_state(ConnectionState {
                                        connected: true,
                                        device_name: Some(name),
                                        public_key: Some(public_key),
                                    }).await;
                                }
                                WsMessage::Heartbeat => {
                                    // Received heartbeat, connection is alive
                                }
                                WsMessage::FileOffer { name, size, hash_total, num_chunks, nonce } => {
                                    let _ = app.emit("file-offer-received", WsMessage::FileOffer { name, size, hash_total, num_chunks, nonce });
                                }
                                _ => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // Disconnected
    session.update_state(ConnectionState {
        connected: false,
        device_name: None,
        public_key: None,
    }).await;
}

// Implement Default for IdentityPublic to help with the unwrap above
impl Default for security::IdentityPublic {
    fn default() -> Self {
        Self { public_key_hex: String::new() }
    }
}