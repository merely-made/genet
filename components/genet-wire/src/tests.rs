use super::*;
use futures_util::{SinkExt, StreamExt};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
fn request(url: &str) -> WireRequest {
    WireRequest {
        url: Url::parse(url).unwrap(),
        method: http::Method::GET,
        headers: vec![],
        body: None,
        prefer_h3: false,
    }
}
#[tokio::test]
async fn raw_http_preserves_redirect_cookie_fields_and_encoded_body_without_policy() {
    let mut server = mockito::Server::new_async().await;
    let raw = [0x1f, 0x8b, 8, 0, 1, 2, 3, 4];
    let redirect = server
        .mock("GET", "/one")
        .match_header("x-explicit", "caller")
        .match_header("cookie", mockito::Matcher::Missing)
        .with_status(302)
        .with_header("location", "/two")
        .with_header("set-cookie", "a=one")
        .with_header("set-cookie", "b=two")
        .with_header("content-encoding", "gzip")
        .with_body(raw)
        .expect(1)
        .create_async()
        .await;
    let forbidden = server.mock("GET", "/two").expect(0).create_async().await;
    let engine = hyper::HttpTransport::webpki(false).unwrap();
    let mut req = request(&format!("{}/one", server.url()));
    req.headers.push(("x-explicit".into(), "caller".into()));
    let mut response = engine.send(req).await.unwrap();
    assert_eq!(response.status, 302);
    assert_eq!(
        response
            .headers
            .iter()
            .filter(|(name, _)| name == "set-cookie")
            .count(),
        2
    );
    let mut body = Vec::new();
    while let Some(chunk) = response.body.next().await {
        body.extend_from_slice(&chunk.unwrap());
    }
    assert_eq!(body, raw);
    redirect.assert_async().await;
    forbidden.assert_async().await;
}

#[tokio::test]
async fn explicit_tls_pools_are_separate_and_http2_exchange_has_no_cookie_authority() {
    use http_body_util::Full;
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = certified.cert.der().clone();
    let _ = rustls::crypto::ring::default_provider().install_default();
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        certified.signing_key.serialize_der(),
    ));
    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], key)
        .unwrap();
    server_config.alpn_protocols = vec![b"h2".to_vec()];
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepted = Arc::new(AtomicUsize::new(0));
    let count = accepted.clone();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let mut connections = tokio::task::JoinSet::new();
        let mut stopped = stopped;
        loop {
            tokio::select! {
                _=&mut stopped=>break,
                connection=listener.accept()=> {
                    let (tcp,_)=connection.unwrap();let acceptor=acceptor.clone();let count=count.clone();
                    connections.spawn(async move {
                        let Ok(tls)=acceptor.accept(tcp).await else{return};
                        assert_eq!(tls.get_ref().1.alpn_protocol(),Some(b"h2".as_slice()));
                        count.fetch_add(1,Ordering::SeqCst);
                        let service=::hyper::service::service_fn(|request: ::hyper::Request<::hyper::body::Incoming>| async move {
                            assert!(!request.headers().contains_key("cookie"));
                            Ok::<_,std::convert::Infallible>(http::Response::builder().status(207)
                                .header("set-cookie","opaque=value").header("x-wire-protocol","h2")
                                .body(Full::new(Bytes::from_static(b"h2 raw"))).unwrap())
                        });
                        let _=::hyper::server::conn::http2::Builder::new(TokioExecutor::new()).serve_connection(TokioIo::new(tls),service).await;
                    });
                }
            }
        }
        connections.abort_all();
        while connections.join_next().await.is_some() {}
    });
    let url = format!("https://localhost:{port}/h2");
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let tls = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let a = Arc::new(hyper::HttpTransport::new(tls.clone(), false));
    let b = hyper::HttpTransport::new(tls, false);
    for engine in [a.as_ref(), a.as_ref(), &b, &b] {
        let mut response = engine.send(request(&url)).await.unwrap();
        assert_eq!(response.status, 207);
        assert!(
            response
                .headers
                .contains(&("x-wire-protocol".into(), "h2".into()))
        );
        while response.body.next().await.is_some() {}
    }
    assert_eq!(
        accepted.load(Ordering::SeqCst),
        2,
        "one HTTP/2 connection per explicit pool"
    );
    let untrusted = hyper::HttpTransport::webpki(false)
        .unwrap()
        .with_response_timeout(Duration::from_secs(2));
    assert!(
        untrusted.send(request(&url)).await.is_none(),
        "self-signed TLS is refused outside the supplied trust partition"
    );
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
}
// Tungstenite fixes the callback error type to its complete HTTP response.
#[allow(clippy::result_large_err)]
#[tokio::test]
async fn raw_websocket_carries_caller_handshake_and_echoes_without_browser_policy() {
    use websocket::tungstenite::{Message, client::IntoClientRequest};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = tokio_tungstenite::accept_hdr_async(
            stream,
            |req: &websocket::tungstenite::handshake::server::Request,
             response: websocket::tungstenite::handshake::server::Response| {
                assert_eq!(req.headers().get("x-caller").unwrap(), "explicit");
                assert!(!req.headers().contains_key("cookie"));
                assert!(!req.headers().contains_key("origin"));
                Ok(response)
            },
        )
        .await
        .unwrap();
        let message = socket.next().await.unwrap().unwrap();
        socket.send(message).await.unwrap();
        socket.close(None).await.unwrap();
    });
    let mut handshake = format!("ws://{address}/raw").into_client_request().unwrap();
    handshake
        .headers_mut()
        .insert("x-caller", http::HeaderValue::from_static("explicit"));
    let (mut socket, response) = websocket::connect(handshake).await.unwrap();
    assert_eq!(response.status(), 101);
    socket
        .send(Message::Binary(vec![0, 1, 255].into()))
        .await
        .unwrap();
    assert_eq!(
        socket.next().await.unwrap().unwrap(),
        Message::Binary(vec![0, 1, 255].into())
    );
    let _ = socket.close(None).await;
    server.await.unwrap();
}
#[tokio::test]
async fn default_wss_connector_runs_webpki_tls_and_rejects_an_unknown_issuer() {
    use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
    use websocket::tungstenite::{Error as WsError, client::IntoClientRequest};
    let certified = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = certified.cert.der().clone();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
        certified.signing_key.serialize_der(),
    ));
    let server_config = rustls::ServerConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.clone()], key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        // Serve two connections: the default one fails its handshake, the trusting one echoes.
        for _ in 0..2 {
            let (tcp, _) = listener.accept().await.unwrap();
            let Ok(tls) = acceptor.accept(tcp).await else {
                continue;
            };
            if let Ok(mut socket) = tokio_tungstenite::accept_async(tls).await {
                let _ = socket.close(None).await;
            }
        }
    });
    let url = format!("wss://localhost:{port}/");
    // The default connector trusts webpki roots only, so a self-signed leaf is refused.
    let refused = websocket::connect(url.as_str().into_client_request().unwrap())
        .await
        .unwrap_err();
    let WsError::Io(io) = refused else {
        panic!("expected a TLS handshake refusal, got {refused:?}");
    };
    let tls = io.get_ref().and_then(|e| e.downcast_ref::<rustls::Error>());
    assert!(
        matches!(
            tls,
            Some(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer
            ))
        ),
        "unexpected refusal: {io:?}"
    );
    // Positive control: the same server accepts a host-selected connector that trusts its cert.
    let mut roots = rustls::RootCertStore::empty();
    roots.add(certificate).unwrap();
    let tls = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let trusted = websocket::connect_with(
        url.as_str().into_client_request().unwrap(),
        Some(websocket::Connector::Rustls(Arc::new(tls))),
    )
    .await;
    assert!(trusted.is_ok(), "trusting connector failed: {:?}", trusted.err());
    server.await.unwrap();
}
