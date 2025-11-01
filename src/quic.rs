use anyhow::Result;
use std::{
    io::{Error as IoError, ErrorKind, Result as IoResult},
    net::SocketAddr,
    sync::Arc,
};

use quinn::{
    ClientConfig, Endpoint, RecvStream, SendStream, ServerConfig, TransportConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
};

use quinn::rustls::{
    self, ClientConfig as RustlsClientConfig, DigitallySignedStruct, Error as RustlsError,
    ServerConfig as RustlsServerConfig, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
};

const DATAGRAM_BUFFER_SIZE: usize = 65536;

pub fn generate_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());

    Ok((CertificateDer::from(cert.cert), PrivateKeyDer::Pkcs8(key)))
}

pub fn server_config() -> Result<ServerConfig> {
    let (cert, private_key) = generate_cert()?;
    let mut crypto = RustlsServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], private_key)?;
    crypto.alpn_protocols = vec![b"example".to_vec()];

    let mut config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));

    let transport = Arc::get_mut(&mut config.transport)
        .ok_or_else(|| anyhow::anyhow!("Failed to get mutable reference to transport config"))?;
    transport.max_concurrent_uni_streams(0_u8.into());

    transport.datagram_receive_buffer_size(Some(DATAGRAM_BUFFER_SIZE));
    transport.datagram_send_buffer_size(DATAGRAM_BUFFER_SIZE);

    Ok(config)
}

pub fn client_config() -> Result<ClientConfig> {
    let mut crypto = RustlsClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    crypto.alpn_protocols = vec![b"example".to_vec()];

    let mut transport = TransportConfig::default();
    transport.datagram_receive_buffer_size(Some(DATAGRAM_BUFFER_SIZE));
    transport.datagram_send_buffer_size(DATAGRAM_BUFFER_SIZE);

    let mut config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
    config.transport_config(Arc::new(transport));

    Ok(config)
}

#[derive(Debug)]
struct SkipServerVerification(Arc<CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
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
    ) -> Result<HandshakeSignatureValid, RustlsError> {
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

#[allow(dead_code)]
pub async fn read_stream(recv: &mut RecvStream, buf: &mut [u8]) -> IoResult<usize> {
    match recv.read(buf).await {
        Ok(Some(n)) => Ok(n),
        Ok(None) => Ok(0),
        Err(e) => Err(IoError::new(ErrorKind::Other, e)),
    }
}

#[allow(dead_code)]
pub async fn write_stream(send: &mut SendStream, data: &[u8]) -> IoResult<()> {
    send.write_all(data).await?;
    Ok(())
}

pub fn bind_server(addr: SocketAddr, config: ServerConfig) -> Result<Endpoint> {
    Ok(Endpoint::server(config, addr)?)
}

pub fn bind_client(addr: SocketAddr) -> Result<Endpoint> {
    Ok(Endpoint::client(addr)?)
}
