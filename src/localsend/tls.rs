//! TLS self-signed certificate generation using `rcgen` and `rustls`.

use rcgen::{CertificateParams, KeyPair};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Container holding rustls server configuration and calculated SHA-256 fingerprint.
pub struct TlsConfig {
    pub server_config: Arc<rustls::ServerConfig>,
    pub fingerprint: String,
}

/// Generates an in-memory self-signed certificate for LocalSend HTTPS communication.
pub fn generate_self_signed_cert(
    alias: &str,
) -> Result<TlsConfig, Box<dyn std::error::Error + Send + Sync>> {
    let subject_alt_names = vec![alias.to_string(), "localhost".to_string()];
    let params = CertificateParams::new(subject_alt_names)?;
    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    let cert_der = cert.der();
    let mut hasher = Sha256::new();
    hasher.update(cert_der.as_ref());
    let fingerprint = hex::encode(hasher.finalize());

    let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert_der.to_vec())];
    let key_der = rustls::pki_types::PrivateKeyDer::Pkcs8(
        rustls::pki_types::PrivatePkcs8KeyDer::from(key_pair.serialize_der()),
    );

    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?;

    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(TlsConfig {
        server_config: Arc::new(server_config),
        fingerprint,
    })
}
