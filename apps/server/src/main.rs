use std::{net::SocketAddr, path::PathBuf};
use theia_network::{Result, server, tls};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let bind: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:5000".into())
        .parse()?;
    let cert = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "build/server-cert.der".into()),
    );
    if args.next().is_some() || !bind.ip().is_loopback() {
        return Err("Usage: theia-server [loopback-address:port] [public-certificate.der]; local prototype only".into());
    }
    let (endpoint, certificate) = tls::server(bind)?;
    if let Some(parent) = cert.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&cert, certificate)?;
    println!(
        "listening={} public_certificate={}",
        endpoint.local_addr()?,
        cert.display()
    );
    let (shutdown, stop) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        let _ = shutdown.send(true);
    });
    server::run(endpoint, stop).await
}
