//! Adapted from theia-engine's reviewed network fixture; see docs/NETWORKING.md.
//! Exact public-certificate trust, no verification bypass and no stateful 0-RTT.
use crate::Result;
use quinn::{
    ClientConfig, Endpoint, ServerConfig, TransportConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    rustls,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
const ALPN: &[u8] = b"theia-mmo-walk-1";
fn transport() -> Arc<TransportConfig> {
    let mut config = TransportConfig::default();
    config
        .max_concurrent_bidi_streams(0u8.into())
        .max_concurrent_uni_streams(1u8.into())
        .stream_receive_window(32u32.into())
        .receive_window(4096u32.into())
        .send_window(4096)
        .datagram_receive_buffer_size(Some(8192))
        .datagram_send_buffer_size(4096)
        .max_idle_timeout(Some(Duration::from_secs(5).try_into().unwrap()))
        .keep_alive_interval(Some(Duration::from_secs(1)));
    Arc::new(config)
}
pub fn server(bind: SocketAddr) -> Result<(Endpoint, Vec<u8>)> {
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()])?;
    let der = rustls::pki_types::CertificateDer::from(certificate.cert);
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
    let mut crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![der.clone()], key.into())?;
    crypto.alpn_protocols = vec![ALPN.to_vec()];
    crypto.max_early_data_size = 0;
    let mut config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(crypto)?));
    config
        .transport_config(transport())
        .max_incoming(8)
        .incoming_buffer_size(4096)
        .incoming_buffer_size_total(32768);
    Ok((Endpoint::server(config, bind)?, der.to_vec()))
}
pub fn client(address: SocketAddr, certificate: &[u8]) -> Result<Endpoint> {
    if certificate.is_empty() || certificate.len() > 65536 {
        return Err("Invalid certificate size".into());
    }
    let mut roots = rustls::RootCertStore::empty();
    roots.add(rustls::pki_types::CertificateDer::from(
        certificate.to_vec(),
    ))?;
    let mut crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    crypto.alpn_protocols = vec![ALPN.to_vec()];
    crypto.enable_early_data = false;
    let mut config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));
    config.transport_config(transport());
    let bind = if address.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let mut endpoint = Endpoint::client(bind.parse()?)?;
    endpoint.set_default_client_config(config);
    Ok(endpoint)
}
