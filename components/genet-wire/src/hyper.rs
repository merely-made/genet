/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The default transport: hyper 1 (h1/h2) over rustls with one process-wide
//! connection pool, plus the HTTP/3 lane (feature `h3`) tried first when the
//! origin advertised it and falling back to h1/h2 on any failure.
//!
//! One pool per process, lazily built. Per-context TLS configuration is a later
//! refinement; a host that needs per-session trust or partitioned pools supplies
//! its own [`Transport`].

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};
use std::sync::{Arc, OnceLock};

use bytes::Bytes;
use futures_util::TryStreamExt;
use http_body_util::{BodyExt, Full};
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::{Connected, Connection, HttpConnector};
use hyper_util::rt::TokioExecutor;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};

use super::{RawResponse, Transport, TransportFuture, WireRequest};

/// Request body type for the client: a fully-buffered `Bytes` (empty for GET).
/// Streaming request bodies are a later increment.
type ReqBody = Full<Bytes>;

/// The connection-pooled, TLS-capable client type.
type HttpClient = Client<LenientConnector<HttpsConnector<HttpConnector>>, ReqBody>;

/// Reads that end in `UnexpectedEof` (a TLS peer that closed without a
/// `close_notify`) are reported as a clean end of stream. hyper still fails a
/// body whose declared `Content-Length` or chunked framing is not complete, so
/// only a close-delimited HTTP/1 body is affected, which is what browsers and
/// curl accept from servers that close this way.
struct LenientEof<T>(T);

impl<T: hyper::rt::Read + Unpin> hyper::rt::Read for LenientEof<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        match Pin::new(&mut self.0).poll_read(cx, buf) {
            Poll::Ready(Err(e)) if e.kind() == io::ErrorKind::UnexpectedEof => Poll::Ready(Ok(())),
            other => other,
        }
    }
}

impl<T: hyper::rt::Write + Unpin> hyper::rt::Write for LenientEof<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }
    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
    }
}

impl<T: Connection> Connection for LenientEof<T> {
    fn connected(&self) -> Connected {
        self.0.connected()
    }
}

/// Wraps a connector so every connection it makes reads leniently.
#[derive(Clone)]
struct LenientConnector<C>(C);

impl<C> tower_service::Service<http::Uri> for LenientConnector<C>
where
    C: tower_service::Service<http::Uri>,
    C::Response: hyper::rt::Read + hyper::rt::Write + Connection + Unpin + Send + 'static,
    C::Future: Send + 'static,
{
    type Response = LenientEof<C::Response>;
    type Error = C::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, C::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }
    fn call(&mut self, dst: http::Uri) -> Self::Future {
        let connecting = self.0.call(dst);
        Box::pin(async move { connecting.await.map(LenientEof) })
    }
}

static CLIENT: OnceLock<HttpClient> = OnceLock::new();
static SHARED: OnceLock<Arc<DefaultTransport>> = OnceLock::new();

/// Whether the shared client should trust any server certificate. Set before the
/// first fetch (the client is built once, lazily).
static ACCEPT_INVALID_CERTS: AtomicBool = AtomicBool::new(false);

/// Make the default transport trust any server certificate, for a local test
/// harness driving a self-signed server (WPT). Must be called before the first
/// request; production never calls it. This is a knob on the default
/// transport only: a host-supplied [`Transport`] owns its own trust.
pub fn accept_invalid_certs() {
    ACCEPT_INVALID_CERTS.store(true, Ordering::Relaxed);
}

/// The default [`Transport`]. Obtain it through [`DefaultTransport::shared`];
/// hosts may choose this compatibility default or an isolated `HttpTransport`.
pub struct DefaultTransport {
    _private: (),
}

impl DefaultTransport {
    /// The process-wide default transport.
    pub fn shared() -> Arc<dyn Transport> {
        let shared: Arc<DefaultTransport> = SHARED
            .get_or_init(|| Arc::new(DefaultTransport { _private: () }))
            .clone();
        shared
    }
}

impl Transport for DefaultTransport {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        Box::pin(send(request))
    }
}

/// A host-owned connection pool with explicit trust and retry policy. It never
/// reads the compatibility default's process-wide insecure-test switch.
/// Create one per host-selected partition; cloning the host's Arc shares only
/// that partition. Response/body policy remains with the caller.
pub struct HttpTransport {
    client: HttpClient,
    response_timeout: Option<std::time::Duration>,
    #[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
    h3: Option<quinn::ClientConfig>,
}
impl HttpTransport {
    pub fn new(tls: rustls::ClientConfig, retry_canceled_requests: bool) -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        Self::from_connector(connector, retry_canceled_requests)
    }
    /// Mozilla roots and an explicit ring provider, independent of global trust.
    pub fn webpki(retry_canceled_requests: bool) -> Result<Self, rustls::Error> {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_provider_and_webpki_roots(rustls::crypto::ring::default_provider())?
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        Ok(Self::from_connector(connector, retry_canceled_requests))
    }
    fn from_connector(connector: HttpsConnector<HttpConnector>, retry: bool) -> Self {
        Self {
            client: Client::builder(TokioExecutor::new())
                .retry_canceled_requests(retry)
                .build(LenientConnector(connector)),
            response_timeout: None,
            #[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
            h3: None,
        }
    }
    /// Bounds connection and response-header wait; the host bounds body reading.
    pub fn with_response_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.response_timeout = Some(timeout);
        self
    }
    /// HTTP/3 trust is explicit too; a custom HTTP pool never falls back to the
    /// process default's QUIC trust configuration.
    #[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
    pub fn with_h3(mut self, config: quinn::ClientConfig) -> Self {
        self.h3 = Some(config);
        self
    }
    async fn exchange(&self, request: WireRequest) -> Option<RawResponse> {
        #[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
        if request.prefer_h3
            && let Some(config) = &self.h3
            && let Some(response) = crate::h3_client::fetch_h3(
                config.clone(),
                &request.url,
                request.method.clone(),
                &request.headers,
                request.body.clone(),
            )
            .await
        {
            return Some(RawResponse::once(
                response.status,
                response.headers,
                response.body,
            ));
        }
        send_h1(request, &self.client).await
    }
}
impl Transport for HttpTransport {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        Box::pin(async move {
            match self.response_timeout {
                Some(timeout) => tokio::time::timeout(timeout, self.exchange(request))
                    .await
                    .ok()
                    .flatten(),
                None => self.exchange(request).await,
            }
        })
    }
}

/// Send one request over h3 (if preferred and available) or h1/h2.
async fn send(request: WireRequest) -> Option<RawResponse> {
    #[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
    if request.prefer_h3
        && let Some(h3) = crate::h3_client::fetch_h3_default(
            &request.url,
            request.method.clone(),
            &request.headers,
            request.body.clone(),
        )
        .await
    {
        return Some(RawResponse::once(h3.status, h3.headers, h3.body));
    }
    // A failed preferred h3 attempt falls back to the same configured h1/h2 pool.

    send_h1(request, shared_client()).await
}

async fn send_h1(request: WireRequest, client: &HttpClient) -> Option<RawResponse> {
    let uri = http::Uri::try_from(request.url.as_str()).ok()?;
    let mut builder = http::Request::builder()
        .method(request.method.clone())
        .uri(uri);
    for (name, value) in &request.headers {
        builder = builder.header(name.as_str(), value.as_str());
    }
    let req = builder
        .body(Full::new(request.body.unwrap_or_default()))
        .ok()?;
    let resp = client.request(req).await.ok()?;
    let status = resp.status().as_u16();
    let headers = collect_headers(resp.headers());
    let data = resp
        .into_body()
        .into_data_stream()
        .map_err(io::Error::other);
    Some(RawResponse {
        status,
        headers,
        body: Box::pin(data),
    })
}

/// The process-wide client, built on first use.
fn shared_client() -> &'static HttpClient {
    CLIENT.get_or_init(build_client)
}

fn build_client() -> HttpClient {
    // rustls 0.23 needs a process-default CryptoProvider for the high-level
    // config builders hyper-rustls uses. Installing is idempotent-ish; it errors
    // if one is already installed, which we ignore.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let builder = hyper_rustls::HttpsConnectorBuilder::new();
    // Escape hatch for local test harnesses (WPT's self-signed server): when
    // [`accept_invalid_certs`] was called, trust any server certificate. Off by
    // default; production uses the webpki trust anchors.
    let connector = if ACCEPT_INVALID_CERTS.load(Ordering::Relaxed) {
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        // ALPN is set by enable_http1()/enable_http2() below; don't pre-define it.
        builder.with_tls_config(config)
    } else {
        builder.with_webpki_roots()
    };

    let https = connector
        .https_or_http() // allow plaintext http:// too (local/dev, smolweb)
        .enable_http1()
        .enable_http2()
        .build();

    Client::builder(TokioExecutor::new()).build(LenientConnector(https))
}

fn collect_headers(map: &http::HeaderMap) -> Vec<(String, String)> {
    map.iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|s| (k.as_str().to_owned(), s.to_owned()))
        })
        .collect()
}

/// Accept any server certificate; armed by [`accept_invalid_certs`] for test
/// harnesses driving a self-signed local server only.
#[derive(Debug)]
struct NoVerify;

impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _: &CertificateDer,
        _: &[CertificateDer],
        _: &ServerName,
        _: &[u8],
        _: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
