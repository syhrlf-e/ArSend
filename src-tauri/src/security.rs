use rand::{rngs::OsRng, RngCore};
use rcgen::generate_simple_self_signed;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tokio_rustls::rustls::client::danger::ServerCertVerifier;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerConfig};
use tokio_rustls::rustls::crypto::CryptoProvider;

const STORE_NAME: &str = "arsend_identity.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Identity {
    pub cert_der: Vec<u8>,
    pub private_key_der: Vec<u8>,
    pub public_key_hex: String, // Fingerprint
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

    let subject_alt_names = vec!["arsend.local".to_string()];
    let certified_key = generate_simple_self_signed(subject_alt_names).map_err(|e| e.to_string())?;

    let cert_der = certified_key.cert.der().to_vec();
    let private_key_der = certified_key.signing_key.serialize_der();

    let mut hasher = Sha256::new();
    hasher.update(&cert_der);
    let fingerprint = hex::encode(hasher.finalize());

    let identity = Identity {
        cert_der,
        private_key_der,
        public_key_hex: fingerprint,
    };

    let val = serde_json::to_value(&identity).map_err(|e| e.to_string())?;
    store.set("identity", val);
    store.save().map_err(|e| e.to_string())?;

    Ok(identity)
}

#[tauri::command]
pub async fn get_public_key(app: AppHandle) -> Result<IdentityPublic, String> {
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

pub fn generate_server_config(identity: &Identity) -> Result<Arc<ServerConfig>, String> {
    let cert_der = CertificateDer::from(identity.cert_der.clone());
    let priv_key_der = PrivateKeyDer::Pkcs8(tokio_rustls::rustls::pki_types::PrivatePkcs8KeyDer::from(identity.private_key_der.clone()));

    let server_cert = vec![cert_der];
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(server_cert, priv_key_der)
        .map_err(|e| e.to_string())?;

    Ok(Arc::new(server_config))
}

pub fn generate_client_config(expected_fingerprint: String) -> Result<Arc<ClientConfig>, String> {
    let root_store = RootCertStore::empty();
    let mut client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    client_config.dangerous().set_certificate_verifier(Arc::new(FingerprintVerifier {
        expected_fingerprint,
        crypto_provider: Arc::new(tokio_rustls::rustls::crypto::ring::default_provider()),
    }));

    Ok(Arc::new(client_config))
}

#[derive(Debug)]
struct FingerprintVerifier {
    expected_fingerprint: String,
    crypto_provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for FingerprintVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &tokio_rustls::rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: tokio_rustls::rustls::pki_types::UnixTime,
    ) -> Result<tokio_rustls::rustls::client::danger::ServerCertVerified, tokio_rustls::rustls::Error> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(end_entity.as_ref());
        let hash = hex::encode(hasher.finalize());

        if hash == self.expected_fingerprint {
            Ok(tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
        } else {
            Err(tokio_rustls::rustls::Error::General(format!(
                "Fingerprint mismatch: expected {}, got {}",
                self.expected_fingerprint, hash
            )))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        tokio_rustls::rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<tokio_rustls::rustls::client::danger::HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        tokio_rustls::rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.crypto_provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
        self.crypto_provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

impl Default for IdentityPublic {
    fn default() -> Self {
        Self { public_key_hex: String::new() }
    }
}
