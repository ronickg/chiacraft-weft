//! TLS for Chia peers. A full node accepts wallets that present any certificate signed by the
//! well-known Chia CA (`chia_ssl::ChiaCertificate::generate` makes one per run: it identifies
//! this connection, not the player, and is never stored), and presents a self-signed node
//! certificate, so the server's certificate isn't checked against roots, only its handshake
//! signature (as every Chia wallet does).
//!
//! A port of chia-sdk-client's `create_rustls_connector` (Apache-2.0, NOTICE) that names its
//! crypto provider instead of using the process default: the game also links rustls with
//! another provider (ureq's), and rustls refuses to guess between two.
use super::failed;
use crate::Result;
use chia_ssl::{CHIA_CA_CRT, ChiaCertificate};
use rustls::{
    ClientConfig, DigitallySignedStruct, RootCertStore, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{
        CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime, pem::PemObject,
    },
};
use std::sync::Arc;

pub use chia_sdk_client::Connector;

#[derive(Debug)]
struct PeerCertificates(CryptoProvider);

impl ServerCertVerifier for PeerCertificates {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

/// A connector with a fresh wallet certificate for this run.
pub fn connector() -> Result<Connector> {
    let cert = ChiaCertificate::generate().map_err(failed("TLS certificate"))?;
    connector_for(&cert)
}

/// A connector presenting `cert`.
pub fn connector_for(cert: &ChiaCertificate) -> Result<Connector> {
    let provider = rustls::crypto::aws_lc_rs::default_provider();
    let mut roots = RootCertStore::empty();
    let ca = CertificateDer::pem_slice_iter(CHIA_CA_CRT.as_bytes())
        .next()
        .ok_or_else(|| crate::Error::new("the Chia CA is missing"))?
        .map_err(failed("the Chia CA"))?;
    roots.add(ca).map_err(failed("the Chia CA"))?;
    let chain: Vec<CertificateDer<'static>> =
        CertificateDer::pem_slice_iter(cert.cert_pem.as_bytes())
            .collect::<std::result::Result<_, _>>()
            .map_err(failed("TLS certificate"))?;
    let key = PrivatePkcs8KeyDer::pem_slice_iter(cert.key_pem.as_bytes())
        .next()
        .ok_or_else(|| crate::Error::new("TLS certificate key missing"))?
        .map_err(failed("TLS certificate key"))?;
    let mut config = ClientConfig::builder_with_provider(Arc::new(provider.clone()))
        .with_safe_default_protocol_versions()
        .map_err(failed("TLS"))?
        .with_root_certificates(roots)
        .with_client_auth_cert(chain, PrivateKeyDer::Pkcs8(key))
        .map_err(failed("TLS"))?;
    config
        .dangerous()
        .set_certificate_verifier(Arc::new(PeerCertificates(provider)));
    Ok(Connector::Rustls(Arc::new(config)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn makes_a_connector() {
        assert!(super::connector().is_ok());
    }
}
