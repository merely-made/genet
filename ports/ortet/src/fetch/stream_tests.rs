// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use genet_scripted::LiveryScriptedDocument;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn accept_request(listener: &TcpListener) -> TcpStream {
    let (mut socket, _) = listener.accept().unwrap();
    let deadline = Some(Duration::from_secs(10));
    socket.set_read_timeout(deadline).unwrap();
    socket.set_write_timeout(deadline).unwrap();
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        request.push(byte[0]);
        assert!(request.len() < 8192);
    }
    socket
}

fn wait_for<E: script_engine_api::ScriptEngine>(
    document: &mut LiveryScriptedDocument<E>,
    marker: &str,
) {
    wait_for_with_timeout(document, marker, Duration::from_secs(5));
}

fn wait_for_with_timeout<E: script_engine_api::ScriptEngine>(
    document: &mut LiveryScriptedDocument<E>,
    marker: &str,
    timeout: Duration,
) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        document.pump(start.elapsed().as_secs_f64() * 1000.0);
        let console = document.console();
        assert!(
            !console.iter().any(|line| line.contains("unexpected:")),
            "{console:?}"
        );
        if console.iter().any(|line| line.contains(marker)) {
            return;
        }
        thread::sleep(Duration::from_millis(2));
    }
    panic!("did not see {marker:?}: {:?}", document.console());
}

fn first_chunk_before_eof<E: script_engine_api::ScriptEngine>() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/", listener.local_addr().unwrap());
    let (finish, allow_eof) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut socket = accept_request(&listener);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nA")
            .unwrap();
        if allow_eof.recv_timeout(Duration::from_secs(10)).is_ok() {
            socket.write_all(b"B").unwrap();
        }
    });
    let mut document = LiveryScriptedDocument::<E>::from_body(
        "",
        OrtetFetcher::with_network().unwrap(),
        &base_url,
    )
    .unwrap();
    document
        .evaluate(
            r#"
            var streamReader;
            fetch('/stream').then(response => {
                streamReader = response.body.getReader();
                return streamReader.read();
            }).then(({value}) => console.log('first:' + value[0]),
                    error => console.log('unexpected:' + error));
            "#,
        )
        .unwrap();
    wait_for(&mut document, "first:65");
    finish.send(()).unwrap();
    document
        .evaluate(
            r#"
        (async () => {
            const {value} = await streamReader.read();
            const {done} = await streamReader.read();
            console.log('end:' + value[0] + ':' + done);
        })().catch(error => console.log('unexpected:' + error));
        "#,
        )
        .unwrap();
    wait_for(&mut document, "end:66:true");
    server.join().unwrap();
}

#[test]
fn native_fetch_first_chunk_before_eof_bole() {
    first_chunk_before_eof::<script_engine_boa::BoaEngine>();
}

#[cfg(all(feature = "scripted-nova", target_pointer_width = "64"))]
#[test]
fn native_fetch_first_chunk_before_eof_vano() {
    first_chunk_before_eof::<script_engine_nova::NovaEngine>();
}

#[derive(Default)]
struct DeliveryCounts {
    pulls: usize,
    chunks: usize,
    largest_chunk: usize,
    bytes: usize,
}

struct CountedStream {
    inner: Arc<dyn ResourceFetchStream>,
    counts: Arc<std::sync::Mutex<DeliveryCounts>>,
}

impl ResourceFetchStream for CountedStream {
    fn request_chunk(&self) {
        self.counts.lock().unwrap().pulls += 1;
        self.inner.request_chunk();
    }
}

struct CountedFetcher {
    inner: OrtetFetcher,
    counts: Arc<std::sync::Mutex<DeliveryCounts>>,
}

impl ResourceFetcher for CountedFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.inner.fetch(url)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        self.inner.fetch_request(request)
    }

    fn start_streaming_fetch(
        &self,
        request: &ResourceRequest,
        deliver: ResourceFetchSink,
    ) -> Option<Arc<dyn ResourceFetchStream>> {
        let counts = Arc::clone(&self.counts);
        let observe = Box::new(move |event| {
            if let ResourceFetchEvent::Chunk(bytes) = &event {
                let mut counts = counts.lock().unwrap();
                counts.chunks += 1;
                assert!(counts.chunks <= counts.pulls);
                counts.largest_chunk = counts.largest_chunk.max(bytes.len());
                assert!(
                    bytes
                        .iter()
                        .copied()
                        .enumerate()
                        .all(|(offset, byte)| byte == ((counts.bytes + offset) % 251) as u8)
                );
                counts.bytes += bytes.len();
            }
            deliver(event);
        });
        let inner = self.inner.start_streaming_fetch(request, observe)?;
        Some(Arc::new(CountedStream {
            inner,
            counts: Arc::clone(&self.counts),
        }))
    }
}

fn bounded_delivery<E: script_engine_api::ScriptEngine>() {
    const BODY_LEN: usize = 3 * MAX_STREAM_CHUNK + 1;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/", listener.local_addr().unwrap());
    let (send_tail, tail_allowed) = mpsc::channel();
    let (tail_sent, available) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut socket = accept_request(&listener);
        write!(
            socket,
            "HTTP/1.1 200 OK\r\nContent-Length: {BODY_LEN}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        socket.write_all(&[0]).unwrap();
        if tail_allowed.recv_timeout(Duration::from_secs(10)).is_ok() {
            let tail: Vec<_> = (1..BODY_LEN).map(|offset| (offset % 251) as u8).collect();
            socket.write_all(&tail).unwrap();
            tail_sent.send(()).unwrap();
        }
    });
    let counts = Arc::new(std::sync::Mutex::new(DeliveryCounts::default()));
    let fetcher = CountedFetcher {
        inner: OrtetFetcher::with_network_policy(ResourceFetchPolicy {
            max_response_bytes: BODY_LEN,
            timeout: Duration::from_secs(30),
            ..ResourceFetchPolicy::default()
        })
        .unwrap(),
        counts: Arc::clone(&counts),
    };
    let mut document = LiveryScriptedDocument::<E>::from_body("", fetcher, &base_url).unwrap();
    document
        .evaluate(
            r#"
            var streamReader;
            (async () => {
                const response = await fetch('/bounded');
                console.log('headers:' + response.status);
                streamReader = response.body.getReader();
            })().catch(error => console.log('unexpected:' + error));
            "#,
        )
        .unwrap();
    wait_for(&mut document, "headers:200");
    assert_eq!(counts.lock().unwrap().pulls, 0);
    assert_eq!(counts.lock().unwrap().chunks, 0);
    document
        .evaluate(
            r#"
            streamReader.read().then(({value}) => console.log('first:' + value.length + ':' + value[0]),
                error => console.log('unexpected:' + error));
            "#,
        )
        .unwrap();
    wait_for(&mut document, "first:1:0");
    send_tail.send(()).unwrap();
    available.recv_timeout(Duration::from_secs(5)).unwrap();
    let pause = Instant::now();
    while pause.elapsed() < Duration::from_millis(100) {
        document.pump(0.0);
        thread::sleep(Duration::from_millis(2));
    }
    assert!(!document.has_external_work());
    {
        let counts = counts.lock().unwrap();
        assert_eq!((counts.pulls, counts.chunks), (1, 1));
    }

    document
        .evaluate(
            r#"
            (async () => {
                let length = 1;
                for (;;) {
                    const {value, done} = await streamReader.read();
                    if (done) break;
                    const last = length + value.length - 1;
                    if (value[0] !== length % 251 || value[value.length - 1] !== last % 251) {
                        throw new Error('out of order at ' + length);
                    }
                    length += value.length;
                    console.log('read:' + length);
                }
                console.log('length:' + length);
            })().catch(error => console.log('unexpected:' + error));
            "#,
        )
        .unwrap();
    wait_for_with_timeout(
        &mut document,
        &format!("length:{BODY_LEN}"),
        Duration::from_secs(20),
    );
    assert!(!document.has_external_work());
    let counts = counts.lock().unwrap();
    assert_eq!(counts.bytes, BODY_LEN);
    assert_eq!(counts.pulls, counts.chunks + 1); // Last pull gets EOF.
    assert!(counts.chunks > 3);
    assert!(counts.largest_chunk <= MAX_STREAM_CHUNK);
    eprintln!(
        "{} delivered {BODY_LEN} bytes in {} chunks, largest {} bytes, {} pulls including EOF",
        std::any::type_name::<E>(),
        counts.chunks,
        counts.largest_chunk,
        counts.pulls
    );
    server.join().unwrap();
}

fn abort_pending_read<E: script_engine_api::ScriptEngine>() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}/", listener.local_addr().unwrap());
    let (disconnected, connection_closed) = mpsc::channel();
    let server = thread::spawn(move || {
        let mut socket = accept_request(&listener);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nA")
            .unwrap();
        let result = socket.read(&mut [0]);
        assert!(
            matches!(result, Ok(0))
                || matches!(result, Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset),
            "connection remained open after abort"
        );
        disconnected.send(()).unwrap();
        let mut followup = accept_request(&listener);
        followup
            .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
            .unwrap();
    });
    let fetcher = OrtetFetcher::with_network_policy(ResourceFetchPolicy {
        max_concurrent_fetches: 1,
        ..ResourceFetchPolicy::default()
    })
    .unwrap();
    let mut document = LiveryScriptedDocument::<E>::from_body("", fetcher, &base_url).unwrap();
    document
        .evaluate(
            r#"
            var controller = new AbortController();
            (async () => {
                const response = await fetch('/abort', {signal: controller.signal});
                const reader = response.body.getReader();
                const {value} = await reader.read();
                const blocked = reader.read();
                console.log('pending:' + value[0]);
                try {
                    await blocked;
                    throw new Error('read unexpectedly resolved');
                } catch (error) {
                    if (error.name !== 'AbortError') throw error;
                    console.log('aborted:' + error.name);
                }
            })().catch(error => console.log('unexpected:' + error));
            "#,
        )
        .unwrap();
    wait_for(&mut document, "pending:65");
    assert!(document.has_external_work());
    document.evaluate("controller.abort();").unwrap();
    wait_for(&mut document, "aborted:AbortError");
    connection_closed
        .recv_timeout(Duration::from_secs(5))
        .unwrap();
    assert!(!document.has_external_work());
    document
        .evaluate(
            r#"
            fetch('/after-abort').then(
                response => console.log('recovered:' + response.status + ':' + (response.body === null)),
                error => console.log('unexpected:' + error)
            );
            "#,
        )
        .unwrap();
    wait_for(&mut document, "recovered:204:true");
    assert!(!document.has_external_work());
    server.join().unwrap();
}

#[test]
fn native_fetch_bounded_delivery_bole() {
    bounded_delivery::<script_engine_boa::BoaEngine>();
}

#[cfg(all(feature = "scripted-nova", target_pointer_width = "64"))]
#[test]
fn native_fetch_bounded_delivery_vano() {
    bounded_delivery::<script_engine_nova::NovaEngine>();
}

#[test]
fn native_fetch_abort_pending_read_bole() {
    abort_pending_read::<script_engine_boa::BoaEngine>();
}

#[cfg(all(feature = "scripted-nova", target_pointer_width = "64"))]
#[test]
fn native_fetch_abort_pending_read_vano() {
    abort_pending_read::<script_engine_nova::NovaEngine>();
}
