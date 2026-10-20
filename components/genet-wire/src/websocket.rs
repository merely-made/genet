// SPDX-License-Identifier: MPL-2.0
//! Raw RFC 6455 connector. The caller assembles its handshake, including any
//! credentials, origin and subprotocols. No browser policy or cookie store is
//! consulted here; a browser must refuse forbidden connections before calling.
use std::sync::Arc;

pub use tokio_tungstenite::tungstenite;
pub use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
pub type Connection = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
pub type HandshakeResponse = http::Response<Option<Vec<u8>>>;

/// Mozilla roots (webpki-roots 1.x) and an explicit ring provider, independent
/// of global trust. tokio-tungstenite's own `rustls-tls-webpki-roots` feature is
/// not used: it pins webpki-roots 0.26, which lacks the emSign removal
/// (RUSTSEC-2026-0381). The feature that is enabled only makes `Connector::Rustls`
/// available; its OS-roots default is never reached.
fn default_connector() -> Result<Connector, tungstenite::Error> {
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|e| tungstenite::Error::Tls(tungstenite::error::TlsError::Rustls(Box::new(e))))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    Ok(Connector::Rustls(Arc::new(tls)))
}

pub async fn connect(
    request: http::Request<()>,
) -> Result<(Connection, HandshakeResponse), tungstenite::Error> {
    connect_with(request, None).await
}
/// An explicit TLS connector is host selected; `None` uses the webpki 1.x roots
/// above. No redirect is followed.
pub async fn connect_with(
    request: http::Request<()>,
    connector: Option<Connector>,
) -> Result<(Connection, HandshakeResponse), tungstenite::Error> {
    let connector = match connector {
        Some(c) => c,
        None => default_connector()?,
    };
    tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector)).await
}
