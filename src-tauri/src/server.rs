use crate::pairing;
use crate::security;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionState {
    pub connected: bool,
    pub device_name: Option<String>,
    pub public_key: Option<String>,
    pub ip: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WsMessage {
    Identity {
        name: String,
        public_key: String,
        pairing_token: Option<String>,
    },
    Heartbeat,
    FileOffer {
        name: String,
        size: u64,
        hash_total: String,
        num_chunks: u32,
        nonce: String,
    },
    FileAccept {
        nonce: String,
    },
    FileReject {
        nonce: String,
    },
}

pub struct SessionManager {
    pub active_connection: Mutex<ConnectionState>,
    pub ws_sender: Mutex<Option<tokio::sync::mpsc::Sender<WsMessage>>>,
    app_handle: AppHandle,
}

impl SessionManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            active_connection: Mutex::new(ConnectionState {
                connected: false,
                device_name: None,
                public_key: None,
                ip: None,
            }),
            ws_sender: Mutex::new(None),
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
    let identity = security::get_or_create_identity(&app)?;
    let server_config = security::generate_server_config(&identity)?;
    let acceptor = TlsAcceptor::from(server_config);

    let listener = match TcpListener::bind("0.0.0.0:9527").await {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!("✅ Server already listening on 0.0.0.0:9527 (HMR active)");
                let _ = app.emit("server-ready", true);
                return Ok(());
            }
            return Err(e.to_string());
        }
    };

    eprintln!("✅ Server listening on 0.0.0.0:9527");
    let _ = app.emit("server-ready", true);

    let session = Arc::new(SessionManager::new(app.clone()));

    if app.try_state::<SharedSession>().is_none() {
        app.manage(session.clone());
    }

    tokio::spawn(async move {
        while let Ok((stream, peer_addr)) = listener.accept().await {
            eprintln!("📥 Incoming connection from {}", peer_addr);
            let acceptor = acceptor.clone();
            let session = session.clone();
            let app = app.clone();
            let device_name = device_name.clone();

            tokio::spawn(async move {
                if let Ok(tls_stream) = acceptor.accept(stream).await {
                    if let Ok(ws_stream) = tokio_tungstenite::accept_async(tls_stream).await {
                        handle_connection(ws_stream, session, app, peer_addr, device_name, None)
                            .await;
                    }
                }
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn connect_to_device(
    app: AppHandle,
    ip: String,
    device_name: String,
    fingerprint: String,
    token: Option<String>,
) -> Result<(), String> {
    let session = if let Some(s) = app.try_state::<SharedSession>() {
        let state: tauri::State<'_, SharedSession> = s;
        state.inner().clone()
    } else {
        let s = Arc::new(SessionManager::new(app.clone()));
        app.manage(s.clone());
        s
    };

    let client_config = security::generate_client_config(fingerprint)?;
    let connector = TlsConnector::from(client_config);

    let addr = format!("{}:9527", ip);
    let stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr))
        .await
        .map_err(|_| format!("Connection to {} timed out after 5s", addr))?
        .map_err(|e| e.to_string())?;

    let domain = ServerName::try_from("arsend.local").unwrap().to_owned();
    let tls_stream = connector
        .connect(domain, stream)
        .await
        .map_err(|e| e.to_string())?;

    let request = format!("wss://{}:9527/", ip);
    let (ws_stream, _) = tokio_tungstenite::client_async(request, tls_stream)
        .await
        .map_err(|e| e.to_string())?;

    let peer_addr = addr.parse().unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
    tokio::spawn(async move {
        handle_connection(ws_stream, session, app, peer_addr, device_name, token).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn disconnect_device(app: AppHandle) -> Result<(), String> {
    if let Some(session) = app.try_state::<SharedSession>() {
        let mut sender_guard = session.ws_sender.lock().await;
        *sender_guard = None;

        session
            .update_state(ConnectionState {
                connected: false,
                device_name: None,
                public_key: None,
                ip: None,
            })
            .await;
    }
    Ok(())
}

async fn handle_connection<S>(
    mut ws_stream: S,
    session: SharedSession,
    app: AppHandle,
    _peer: SocketAddr,
    device_name: String,
    pairing_token: Option<String>,
) where
    S: SinkExt<Message>
        + StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin
        + Send
        + 'static,
{
    let my_identity = security::get_public_key(app.clone())
        .await
        .unwrap_or_default();
    let id_msg = WsMessage::Identity {
        name: device_name,
        public_key: my_identity.public_key_hex,
        pairing_token,
    };

    if let Ok(msg) = serde_json::to_string(&id_msg) {
        let _ = ws_stream.send(Message::Text(msg.into())).await;
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsMessage>(32);
    {
        let mut sender_guard = session.ws_sender.lock().await;
        *sender_guard = Some(tx);
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
            Some(out_msg) = rx.recv() => {
                if let Ok(msg) = serde_json::to_string(&out_msg) {
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
                                WsMessage::Identity { name, public_key, pairing_token } => {
                                    if let Some(token) = pairing_token {
                                        if !pairing::validate_session_token(&token) {
                                            let _ = app.emit("connection-rejected", "Invalid or expired QR token");
                                            break;
                                        }
                                    }

                                    session.update_state(ConnectionState {
                                        connected: true,
                                        device_name: Some(name),
                                        public_key: Some(public_key),
                                        ip: Some(_peer.ip().to_string()),
                                    }).await;
                                }
                                WsMessage::Heartbeat => {
                                }
                                WsMessage::FileOffer { name, size, hash_total, num_chunks, nonce } => {
                                    #[derive(Serialize, Clone)]
                                    struct OfferPayload {
                                        name: String,
                                        size: u64,
                                        hash_total: String,
                                        num_chunks: u32,
                                        nonce: String,
                                    }
                                    let _ = app.emit("file-offer-received", OfferPayload { name, size, hash_total, num_chunks, nonce });
                                }
                                WsMessage::FileAccept { nonce } => {
                                    let _ = app.emit("file-accept-received", nonce);
                                }
                                WsMessage::FileReject { nonce } => {
                                    let _ = app.emit("file-reject-received", nonce);
                                }
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

    {
        let mut sender_guard = session.ws_sender.lock().await;
        *sender_guard = None;
    }

    session
        .update_state(ConnectionState {
            connected: false,
            device_name: None,
            public_key: None,
            ip: None,
        })
        .await;
}
