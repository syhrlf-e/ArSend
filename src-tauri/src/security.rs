use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use std::sync::Arc;
use tokio_rustls::rustls::{ServerConfig, ClientConfig, RootCertStore};
use tokio_rustls::rustls::client::danger::ServerCertVerifier;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rcgen::generate_simple_self_signed;

const STORE_NAME: &str = "arsend_identity.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identity {
    pub private_key_hex: String,
    pub public_key_hex: String,
}

#[derive(Serialize)]
pub struct IdentityPublic {
    pub public_key_hex: String,
}

pub fn get_or_create_identity(app: &AppHandle) -> Result<Identity, String> {
    let store = app.store(STORE_NAME).map_err(|e| e.to_string())?;

    if let Some(identity_val) = store.get("identity") {
        if let Ok(identity) = serde_json::from_value::<Identity>(identity_val) {
            return Ok(identity);
        }
    }

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let public_key = signing_key.verifying_key();

    let private_key_hex = hex::encode(signing_key.to_bytes());
    let public_key_hex = hex::encode(public_key.to_bytes());

    let identity = Identity {
        private_key_hex,
        public_key_hex,
    };

    let val = serde_json::to_value(&identity).map_err(|e| e.to_string())?;
    store.set("identity", val);
    store.save().map_err(|e| e.to_string())?;

    Ok(identity)
}

#[tauri::command]
pub fn get_public_key(app: AppHandle) -> Result<IdentityPublic, String> {
    let identity = get_or_create_identity(&app)?;
    Ok(IdentityPublic {
        public_key_hex: identity.public_key_hex,
    })
}

pub fn generate_nonce() -> String {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    hex::encode(nonce)
}
pub fn generate_tls_config() -> Result<(Arc<ServerConfig>, Arc<ClientConfig>), String> {
    let subject_alt_names = vec!["arsend.local".to_string()];
    let certified_key = generate_simple_self_signed(subject_alt_names).map_err(|e| e.to_string())?;

    let cert_der = certified_key.cert.into();
    let priv_key_der = certified_key.signing_key.serialize_der();

    let server_cert = vec![cert_der];
    let server_key = PrivateKeyDer::Pkcs8(tokio_rustls::rustls::pki_types::PrivatePkcs8KeyDer::from(priv_key_der));

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(server_cert, server_key)
        .map_err(|e| e.to_string())?;

    let root_store = RootCertStore::empty();
    let mut client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    client_config.dangerous().set_certificate_verifier(Arc::new(NoCertVerifier));

    Ok((Arc::new(server_config), Arc::new(client_config)))
}

#[derive(Debug)]
struct NoCertVerifier;

impl ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &tokio_rustls::rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: tokio_rustls::rustls::pki_types::UnixTime,
    ) -> Result<tokio_rustls::rustls::client::danger::ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
        tokio_rustls::rustls::crypto::ring::default_provider().signature_verification_algorithms.supported_schemes()
    }
}
