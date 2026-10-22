// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ortet's fetch seam: two lanes and a scheme split between them.
//!
//! `data:`, `file:` and bare paths are answered by `genet-documents`'
//! [`LocalFetcher`]. `http(s)` goes to netfetcher's default transport on a
//! background tokio runtime this host owns. Every other scheme is a miss —
//! ortet has no trust store, no protocol registry, and no smolweb lane, and
//! adding one is a product decision that belongs to Mere.

use std::sync::Arc;

use genet_documents::LocalFetcher;
use genet_host_api::{
    ResourceFetchPolicy, ResourceFetchResponse, ResourceFetcher, ResourceRequest, ResourceResponse,
};

/// Which lane serves an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchLane {
    /// `data:`, `file:`, or a bare filesystem path.
    Local,
    /// `http:` or `https:`.
    Remote,
    /// Anything else. Ortet answers `None`; it does not guess a transport.
    Unsupported,
}

/// The scheme split, as a pure function so it is testable without a network.
pub fn lane_for(url: &str) -> FetchLane {
    match crate::args::url_scheme(url) {
        // No scheme is a filesystem path, which is what `LocalFetcher` reads.
        None => FetchLane::Local,
        Some(scheme) => {
            let scheme = scheme.to_ascii_lowercase();
            match scheme.as_str() {
                "data" | "file" => FetchLane::Local,
                "http" | "https" => FetchLane::Remote,
                _ => FetchLane::Unsupported,
            }
        },
    }
}

/// The one fetcher the session engine is constructed with.
#[derive(Clone)]
pub struct OrtetFetcher {
    remote: Option<Arc<RemoteFetcher>>,
}

impl OrtetFetcher {
    /// Local schemes only. The receipts run this way, and so does any run whose
    /// document never names a remote resource.
    pub fn local_only() -> Self {
        Self { remote: None }
    }

    /// Local schemes plus http(s) over netfetcher.
    pub fn with_network() -> Result<Self, String> {
        Self::with_network_policy(ResourceFetchPolicy::default())
    }

    /// The host selects one policy for this fetcher and all its clones.
    pub fn with_network_policy(policy: ResourceFetchPolicy) -> Result<Self, String> {
        Ok(Self {
            remote: Some(Arc::new(RemoteFetcher::with_policy(policy)?)),
        })
    }
}

impl ResourceFetcher for OrtetFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn max_concurrent_fetches(&self) -> usize {
        self.remote.as_ref().map_or_else(
            || ResourceFetchPolicy::default().max_concurrent_fetches,
            |remote| remote.policy.max_concurrent_fetches,
        )
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        match lane_for(&request.url) {
            FetchLane::Local if request.is_plain_get() => {
                LocalFetcher.fetch_request(request)
            },
            FetchLane::Remote => self.remote.as_ref()?.fetch_request(request),
            _ => None,
        }
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        match lane_for(url) {
            FetchLane::Local => LocalFetcher.fetch_response(url),
            FetchLane::Remote => self.remote.as_ref()?.fetch_response(url),
            FetchLane::Unsupported => None,
        }
    }
}

/// http(s) over netfetcher's default transport.
///
/// The document session boundary is synchronous, so the async fetch is run to
/// completion on a runtime this host owns rather than on the UI thread's own
/// executor — the arrangement netfetcher's README describes for its consumers.
pub struct RemoteFetcher {
    runtime: tokio::runtime::Runtime,
    context: netfetcher::FetchContext,
    policy: ResourceFetchPolicy,
    permits: tokio::sync::Semaphore,
}

/// Why a native remote resource could not be delivered. The legacy
/// `ResourceFetcher` trait maps failures to `None`; the native host can use
/// this result directly when it needs a precise report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeFetchFailure {
    UnsupportedScheme,
    InvalidRequest,
    NoAdmissionCapacity,
    Deadline,
    Network,
    Body,
    ResponseTooLarge,
}

impl RemoteFetcher {
    pub fn new() -> Result<Self, String> {
        Self::with_policy(ResourceFetchPolicy::default())
    }

    pub fn with_policy(policy: ResourceFetchPolicy) -> Result<Self, String> {
        if policy.max_concurrent_fetches > tokio::sync::Semaphore::MAX_PERMITS {
            return Err(format!(
                "resource fetch concurrency {} exceeds Tokio's permit limit {}",
                policy.max_concurrent_fetches,
                tokio::sync::Semaphore::MAX_PERMITS
            ));
        }
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("ortet-fetch")
            .enable_all()
            .build()
            .map_err(|error| format!("could not start the ortet fetch runtime: {error}"))?;
        Ok(Self {
            runtime,
            context: netfetcher::FetchContext::permissive()
                .with_redirect_limit(policy.max_redirects)
                .with_response_max_body_bytes(policy.max_response_bytes),
            policy,
            permits: tokio::sync::Semaphore::new(policy.max_concurrent_fetches),
        })
    }
}

impl ResourceFetcher for RemoteFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn max_concurrent_fetches(&self) -> usize {
        self.policy.max_concurrent_fetches
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.fetch_request(&ResourceRequest::plain_get(url))
            .map(ResourceFetchResponse::into_legacy_response)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        self.fetch_request_result(request).ok()
    }
}

impl RemoteFetcher {
    pub fn fetch_request_result(
        &self,
        request: &ResourceRequest,
    ) -> Result<ResourceFetchResponse, NativeFetchFailure> {
        self.fetch_request_with_admission_hook(request, || {})
    }

    fn fetch_request_with_admission_hook(
        &self,
        request: &ResourceRequest,
        on_admission: impl FnOnce(),
    ) -> Result<ResourceFetchResponse, NativeFetchFailure> {
        if lane_for(&request.url) != FetchLane::Remote {
            return Err(NativeFetchFailure::UnsupportedScheme);
        }
        let wire = netfetcher::Request::from_host(request)
            .ok_or(NativeFetchFailure::InvalidRequest)?;
        if self.policy.max_concurrent_fetches == 0 {
            return Err(NativeFetchFailure::NoAdmissionCapacity);
        }
        self.runtime.block_on(async {
            tokio::time::timeout(self.policy.timeout, async {
                on_admission();
                let _permit = self.permits.acquire().await.map_err(|_| NativeFetchFailure::NoAdmissionCapacity)?;
            let response = netfetcher::fetch(wire, &self.context).await;
            if response.is_network_error() {
                return Err(NativeFetchFailure::Network);
            }
            let final_url = response
                .url_list
                .last()
                .map_or_else(|| request.url.clone(), ToString::to_string);
            let content_type = response
                .headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                .map(|(_, value)| value.clone());
            let status = response.status;
            let headers = response.headers.clone();
            let response_type = match response.response_type {
                netfetcher::ResponseType::Basic => "basic",
                netfetcher::ResponseType::Cors => "cors",
                netfetcher::ResponseType::Opaque => "opaque",
                netfetcher::ResponseType::OpaqueRedirect => "opaqueredirect",
                netfetcher::ResponseType::Error => "error",
            }
            .into();
            let redirected = response.url_list.len() > 1;
            let mut body = response.body;
            let mut bytes = Vec::new();
            while let Some(chunk) = body.next_chunk().await {
                let chunk = chunk.map_err(|_| NativeFetchFailure::Body)?;
                if chunk.len() > self.policy.max_response_bytes.saturating_sub(bytes.len()) {
                    return Err(NativeFetchFailure::ResponseTooLarge);
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok(ResourceFetchResponse {
                final_url,
                content_type,
                bytes,
                status,
                // netfetcher exposes the numeric status, not the transport reason phrase.
                status_text: String::new(),
                headers,
                response_type,
                redirected,
            })
            })
            .await
            .map_err(|_| NativeFetchFailure::Deadline)?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scheme_split_sends_local_addresses_to_the_filesystem() {
        assert_eq!(lane_for("a.html"), FetchLane::Local);
        assert_eq!(lane_for("./sub/a.css"), FetchLane::Local);
        assert_eq!(lane_for("C:\\pages\\a.html"), FetchLane::Local);
        assert_eq!(lane_for("file:///x/a.html"), FetchLane::Local);
        assert_eq!(lane_for("FILE:///x/a.html"), FetchLane::Local);
        assert_eq!(lane_for("data:text/html,<p>x</p>"), FetchLane::Local);
    }

    #[test]
    fn only_http_and_https_reach_the_network_lane() {
        assert_eq!(lane_for("http://example.invalid/a"), FetchLane::Remote);
        assert_eq!(lane_for("https://example.invalid/a"), FetchLane::Remote);
        assert_eq!(lane_for("HTTPS://example.invalid/a"), FetchLane::Remote);
    }

    /// Ortet is not a protocol registry. A scheme it does not serve is a clean
    /// miss, never a guess at a transport or a fall-through to the filesystem.
    #[test]
    fn every_other_scheme_is_an_honest_miss() {
        assert_eq!(
            lane_for("gemini://capsule.invalid/"),
            FetchLane::Unsupported
        );
        assert_eq!(lane_for("ws://example.invalid/"), FetchLane::Unsupported);
        assert_eq!(lane_for("about:blank"), FetchLane::Unsupported);

        let fetcher = OrtetFetcher::local_only();
        assert!(fetcher.fetch("gemini://capsule.invalid/").is_none());
        // With no network lane wired, http(s) is a miss rather than a panic.
        assert!(fetcher.fetch("https://example.invalid/a").is_none());
        assert!(fetcher.fetch("/no/such/ortet/file.html").is_none());
    }

    /// The local lane really reads bytes, so a passing "miss" test above is not
    /// merely an instrument that answers `None` to everything.
    #[test]
    fn the_local_lane_reads_a_real_file() {
        let fixture = std::env::temp_dir().join(format!(
            "ortet-fetch-{}-{}.html",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        std::fs::write(&fixture, b"<p>ortet</p>").expect("write fixture");
        let fetcher = OrtetFetcher::local_only();
        let address = crate::args::file_url_from_path(&fixture);
        assert_eq!(
            fetcher.fetch(&address).as_deref(),
            Some(b"<p>ortet</p>".as_slice()),
            "the local lane must read {address}"
        );
        std::fs::remove_file(&fixture).expect("remove fixture");
    }

    // TASK5 DRAFT WIRE FIXTURES BEGIN. Real owned HTTP, never a transport fake.
    #[derive(Debug)]
    struct T5WireRequest { method:String,target:String,headers:Vec<(String,String)>,body:Vec<u8>,raw:Vec<u8> }
    struct T5HttpReply { status:u16,location:Option<&'static str>,body:&'static [u8] }
    struct T5HttpServer {
        origin:String,
        task:std::thread::JoinHandle<Result<Vec<T5WireRequest>,String>>,
    }
    fn t5_read_wire(stream:&std::net::TcpStream)->Result<T5WireRequest,String> {
        use std::io::{BufRead,Read};
        let mut reader=std::io::BufReader::new(stream.try_clone().map_err(|e|e.to_string())?);
        let mut line=String::new();reader.read_line(&mut line).map_err(|e|e.to_string())?;
        let mut raw=line.as_bytes().to_vec();
        let words:Vec<_>=line.split_whitespace().collect();
        if words.len()!=3 {return Err(format!("bad actual request line {line:?}"));}
        let method=words[0].to_owned();let target=words[1].to_owned();
        let mut headers=Vec::new();
        loop {
            line.clear();reader.read_line(&mut line).map_err(|e|e.to_string())?;
            raw.extend_from_slice(line.as_bytes());
            if raw.len()>65536 {return Err("fixture raw header cap".into());}
            if line=="\r\n" {break;}
            if line.is_empty()||headers.len()>128 {return Err("incomplete/oversized actual HTTP headers".into());}
            let (k,v)=line.split_once(':').ok_or("bad actual header")?;
            headers.push((k.to_owned(),v.trim().to_owned()));
        }
        let header=|name:&str|headers.iter().find(|(k,_)|k.eq_ignore_ascii_case(name)).map(|(_,v)|v.as_str());
        let mut body=Vec::new();
        if header("transfer-encoding").is_some_and(|v|v.eq_ignore_ascii_case("chunked")) {
            loop {
                line.clear();let read=reader.read_line(&mut line).map_err(|e|e.to_string())?;
                if read==0{return Err("incomplete actual chunk size".into());}raw.extend_from_slice(line.as_bytes());
                if raw.len()>2*1024*1024{return Err("fixture raw receipt cap".into());}
                let n=usize::from_str_radix(line.trim().split(';').next().unwrap(),16).map_err(|e|e.to_string())?;
                if n==0 {
                    let mut trailer_bytes=0;let mut trailer_count=0;
                    loop {line.clear();let read=reader.read_line(&mut line).map_err(|e|e.to_string())?;
                        if read==0{return Err("incomplete actual chunk trailers".into());}
                        trailer_bytes+=read;trailer_count+=1;
                        if trailer_bytes>65536||trailer_count>128{return Err("fixture trailer cap".into());}
                        raw.extend_from_slice(line.as_bytes());if raw.len()>2*1024*1024{return Err("fixture raw receipt cap".into());}if line=="\r\n"{break;}
                        if !line.contains(':'){return Err("invalid actual trailer".into());}
                    }break;
                }
                if n>1024*1024-body.len() {return Err("fixture body cap".into());}
                let offset=body.len();body.resize(offset+n,0);reader.read_exact(&mut body[offset..]).map_err(|e|e.to_string())?;raw.extend_from_slice(&body[offset..]);
                let mut crlf=[0;2];reader.read_exact(&mut crlf).map_err(|e|e.to_string())?;
                raw.extend_from_slice(&crlf);if crlf!=*b"\r\n" {return Err("bad chunk framing".into());}
            }
        } else if let Some(n)=header("content-length") {
            let n:usize=n.parse().map_err(|e:std::num::ParseIntError|e.to_string())?;
            if n>1024*1024 {return Err("fixture body cap".into());}
            body.resize(n,0);reader.read_exact(&mut body).map_err(|e|e.to_string())?;raw.extend_from_slice(&body);
        }
        if raw.len()>2*1024*1024{return Err("fixture raw receipt cap".into());}
        Ok(T5WireRequest{method,target,headers,body,raw})
    }
    fn t5_http_server(replies:Vec<T5HttpReply>)->T5HttpServer {
        use std::io::Write;
        let listener=std::net::TcpListener::bind(("127.0.0.1",0)).expect("owned ephemeral HTTP bind");
        listener.set_nonblocking(true).unwrap();
        let origin=format!("http://{}",listener.local_addr().unwrap());
        let task=std::thread::spawn(move || {
            let mut log=Vec::new();
            for reply in replies {
                let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
                let mut stream=loop {match listener.accept() {
                    Ok((stream,_))=>break stream,
                    Err(e) if e.kind()==std::io::ErrorKind::WouldBlock=>{
                        if std::time::Instant::now()>=deadline{return Err("fixture accept timeout".into());}
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    },Err(e)=>return Err(e.to_string()),
                }};
                stream.set_read_timeout(Some(std::time::Duration::from_secs(5))).map_err(|e|e.to_string())?;
                stream.set_write_timeout(Some(std::time::Duration::from_secs(5))).map_err(|e|e.to_string())?;
                log.push(t5_read_wire(&stream)?);
                let location=reply.location.map_or(String::new(),|v|format!("Location: {v}\r\n"));
                let header=format!("HTTP/1.1 {} Fixture\r\nContent-Type: text/html\r\nContent-Length: {}\r\n{}Connection: close\r\n\r\n",reply.status,reply.body.len(),location);
                stream.write_all(header.as_bytes()).map_err(|e|e.to_string())?;
                stream.write_all(reply.body).map_err(|e|e.to_string())?;
            }
            Ok(log)
        });
        T5HttpServer{origin,task}
    }
    fn t5_wire_request(origin:&str,path:&str)->ResourceRequest {
        ResourceRequest{url:format!("{origin}{path}"),method:"POST".into(),
            headers:vec![("Content-Type".into(),"text/plain".into())],body:Some(b"q=\0\x80\xff\r\n".to_vec()),
            cache:"default".into(),redirect:"follow".into(),mode:"navigate".into(),
            referrer:format!("{origin}/source#initiator"),referrer_policy:"no-referrer".into(),
            credentials:"include".into(),integrity:String::new(),
            upgrade_insecure_requests:false,block_all_mixed_content:false}
    }
    fn t5_wire_header<'a>(r:&'a T5WireRequest,name:&str)->Option<&'a str> {
        r.headers.iter().find(|(k,_)|k.eq_ignore_ascii_case(name)).map(|(_,v)|v.as_str())
    }
    #[test]
    fn phase_c_t5_ortet_wire_post_policy_and404() {
        let server=t5_http_server(vec![T5HttpReply{status:404,location:None,body:b"<p>HTTP404</p>"}]);
        let request=t5_wire_request(&server.origin,"/same");
        let response=OrtetFetcher::with_network().unwrap().fetch_request(&request).expect("readable404 response");
        let log=server.task.join().unwrap().expect("actual HTTP receipt");
        eprintln!("actual owned HTTP receipt: {log:?}");
        assert!(log[0].raw.starts_with(b"POST /same HTTP/1.1\r\n"));
        assert_eq!(log.len(),1);assert_eq!((&*log[0].method,&*log[0].target),("POST","/same"));
        assert_eq!(log[0].body,b"q=\0\x80\xff\r\n");
        assert_eq!(t5_wire_header(&log[0],"content-type"),Some("text/plain"));
        assert_eq!(t5_wire_header(&log[0],"referer"),None);
        assert_eq!(request.referrer,format!("{}/source#initiator",server.origin));
        assert_eq!(response.status,404);assert_eq!(response.bytes,b"<p>HTTP404</p>");
        assert_eq!(response.final_url,format!("{}/same",server.origin));
    }
    #[test]
    fn phase_c_t5_ortet_wire_redirect303_and307() {
        for status in [303,307] {
            let server=t5_http_server(vec![T5HttpReply{status,location:Some("/final"),body:b""},
                T5HttpReply{status:200,location:None,body:b"<p>FINAL</p>"}]);
            let response=OrtetFetcher::with_network().unwrap().fetch_request(&t5_wire_request(&server.origin,"/old")).unwrap();
            let log=server.task.join().unwrap().expect("two actual HTTP hops");
            eprintln!("actual owned redirect receipts: {log:?}");
            assert!(log[0].raw.starts_with(b"POST /old HTTP/1.1\r\n"));
            assert_eq!(log.len(),2);assert_eq!((&*log[0].method,&*log[0].target),("POST","/old"));
            assert_eq!(log[0].body,b"q=\0\x80\xff\r\n");
            assert_eq!(log[1].target,"/final");
            if status==303 {assert_eq!(log[1].method,"GET");assert!(log[1].body.is_empty());assert_eq!(t5_wire_header(&log[1],"content-type"),None);}
            else {assert_eq!(log[1].method,"POST");assert_eq!(log[1].body,b"q=\0\x80\xff\r\n");assert_eq!(t5_wire_header(&log[1],"content-type"),Some("text/plain"));}
            assert_eq!(response.final_url,format!("{}/final",server.origin));
            assert!(response.redirected);assert_eq!(response.bytes,b"<p>FINAL</p>");
        }
    }
    #[test]
    fn phase_c_t5_ortet_local_url_only_declines_rich() {
        let file=std::env::temp_dir().join(format!("ortet-t5-{}-{}.html",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::write(&file,b"<p>LOCAL</p>").unwrap();
        let url=crate::args::file_url_from_path(&file);
        let fetcher=OrtetFetcher::local_only();
        let mut request=t5_wire_request("https://unused.test","/old");request.url=url.clone();
        assert!(fetcher.fetch_request(&request).is_none());
        request.method="GET".into();request.body=None;request.headers.clear();
        assert!(fetcher.fetch_request(&request).is_none(),"custom navigate policies are not plain local GET");
        assert_eq!(fetcher.fetch(&url).unwrap(),b"<p>LOCAL</p>");
        std::fs::remove_file(file).unwrap();
    }
    #[test]
    fn phase_c_t5_ortet_wire_redirect_fragment_chain() {
        let mut final_urls=Vec::new();
        for (intermediate,want_suffix) in [("/mid","/final#done"),("/mid#other","/final#other"),("/mid#","/final#")] {
            let server=t5_http_server(vec![T5HttpReply{status:307,location:Some(intermediate),body:b""},
                T5HttpReply{status:307,location:Some("/final"),body:b""},
                T5HttpReply{status:200,location:None,body:b"<p id=final>FINAL</p>"}]);
            let response=OrtetFetcher::with_network().unwrap().fetch_request(&t5_wire_request(&server.origin,"/old#done")).unwrap();
            let log=server.task.join().unwrap().expect("three actual307 requests");
            eprintln!("actual owned307 fragment chain receipts: {log:?}");
            assert_eq!(log.len(),3);
            for (request,path) in log.iter().zip(["/old","/mid","/final"]) {
                assert_eq!((&*request.method,&*request.target),("POST",path));
                assert!(request.raw.starts_with(format!("POST {path} HTTP/1.1\r\n").as_bytes()));
                assert!(request.raw.windows(7).any(|bytes|bytes==b"q=\0\x80\xff\r\n"));
                assert_eq!(request.body,b"q=\0\x80\xff\r\n");
                assert_eq!(t5_wire_header(request,"content-type"),Some("text/plain"));
            }
            assert_eq!(response.status,200);assert_eq!(response.response_type,"basic");assert!(response.redirected);
            assert_eq!(response.bytes,b"<p id=final>FINAL</p>");
            final_urls.push((response.final_url,format!("{}{}",server.origin,want_suffix)));
        }
        // All chains execute before the independent fragment metadata comparison.
        for (actual,want) in final_urls {assert_eq!(actual,want,"per-hop fragment inheritance, explicit empty wins");}
    }
    // TASK5 DRAFT WIRE FIXTURES END.

    fn policy_http_server(
        headers: String,
        body: Vec<u8>,
        body_delay: std::time::Duration,
    ) -> (String, std::thread::JoinHandle<Result<T5WireRequest, String>>) {
        use std::io::Write;
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("owned policy server");
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let task = std::thread::spawn(move || {
            let mut stream = policy_accept(&listener)?;
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .map_err(|error| error.to_string())?;
            let request = t5_read_wire(&stream)?;
            stream
                .write_all(headers.as_bytes())
                .map_err(|error| error.to_string())?;
            std::thread::sleep(body_delay);
            // A timed-out client may already have closed the socket.
            let _ = stream.write_all(&body);
            Ok(request)
        });
        (origin, task)
    }

    fn policy_accept(listener: &std::net::TcpListener) -> Result<std::net::TcpStream, String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).map_err(|error| error.to_string())?;
                    return Ok(stream);
                },
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if std::time::Instant::now() >= deadline {
                        return Err("policy fixture accept deadline".into());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                },
                Err(error) => return Err(error.to_string()),
            }
        }
    }

    #[test]
    fn native_policy_counts_chunked_bytes_without_content_length() {
        let (origin, task) = policy_http_server(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".into(),
            b"3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n".to_vec(),
            std::time::Duration::ZERO,
        );
        let fetcher = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_response_bytes: 5,
            ..ResourceFetchPolicy::default()
        })
        .unwrap();
        assert_eq!(
            fetcher.fetch_request_result(&ResourceRequest::plain_get(format!("{origin}/chunked"))),
            Err(NativeFetchFailure::ResponseTooLarge)
        );
        assert_eq!(task.join().unwrap().unwrap().target, "/chunked");
    }

    const POLICY_GZIP: [u8; 47] = [
            31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 75, 73, 77, 206, 79, 73, 77, 81, 40,
            72, 172, 204, 201, 79, 76, 81, 72, 173, 72, 78, 77, 77, 41, 86, 72, 78, 44,
            0, 0, 215, 96, 9, 178, 27, 0, 0, 0,
    ];

    fn policy_gzip_server() -> (String, std::thread::JoinHandle<Result<T5WireRequest, String>>) {
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            POLICY_GZIP.len()
        );
        policy_http_server(headers, POLICY_GZIP.to_vec(), std::time::Duration::ZERO)
    }

    #[test]
    fn native_policy_caps_decoded_gzip_bytes() {
        let (origin, task) = policy_gzip_server();
        let fetcher = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_response_bytes: 20,
            ..ResourceFetchPolicy::default()
        })
        .unwrap();
        assert_eq!(
            fetcher.fetch_request_result(&ResourceRequest::plain_get(format!("{origin}/gzip"))),
            Err(NativeFetchFailure::ResponseTooLarge)
        );
        task.join().unwrap().unwrap();
    }

    #[test]
    fn native_policy_bounds_valid_integrity_buffer_before_host_streaming() {
        let integrity = "sha256-iJN2mv0S9jylmjtI4BPPUOnusDG7lJ38ovUADl2W6fM=";
        let (origin, task) = policy_gzip_server();
        let fetcher = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_response_bytes: 20,
            ..ResourceFetchPolicy::default()
        })
        .unwrap();
        let mut request = ResourceRequest::plain_get(format!("{origin}/integrity"));
        request.integrity = integrity.into();
        assert_eq!(
            fetcher.fetch_request_result(&request),
            Err(NativeFetchFailure::Network),
            "Fetch must reject the over-cap decoded SRI body before returning it"
        );
        task.join().unwrap().unwrap();

        let (origin, task) = policy_gzip_server();
        let fetcher = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_response_bytes: 27,
            ..ResourceFetchPolicy::default()
        })
        .unwrap();
        let mut request = ResourceRequest::plain_get(format!("{origin}/integrity"));
        request.integrity = integrity.into();
        assert_eq!(
            fetcher.fetch_request_result(&request).unwrap().bytes,
            b"decoded payload exceeds cap"
        );
        task.join().unwrap().unwrap();
    }

    #[test]
    fn native_policy_deadline_covers_body_and_permit_wait() {
        let policy = ResourceFetchPolicy {
            timeout: std::time::Duration::from_millis(50),
            max_concurrent_fetches: 1,
            ..ResourceFetchPolicy::default()
        };
        let (origin, task) = policy_http_server(
            "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\n".into(),
            b"slow".to_vec(),
            std::time::Duration::from_millis(150),
        );
        let fetcher = RemoteFetcher::with_policy(policy).unwrap();
        assert_eq!(
            fetcher.fetch_request_result(&ResourceRequest::plain_get(format!("{origin}/slow"))),
            Err(NativeFetchFailure::Deadline)
        );
        task.join().unwrap().unwrap();

        let held = fetcher.permits.try_acquire().unwrap();
        assert_eq!(
            fetcher.fetch_request_result(&ResourceRequest::plain_get(format!("{origin}/waiting"))),
            Err(NativeFetchFailure::Deadline)
        );
        drop(held);

        let no_capacity = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_concurrent_fetches: 0,
            ..policy
        })
        .unwrap();
        assert_eq!(
            no_capacity.fetch_request_result(&ResourceRequest::plain_get(format!("{origin}/closed"))),
            Err(NativeFetchFailure::NoAdmissionCapacity)
        );
    }

    #[test]
    fn native_policy_admits_only_one_parallel_wire_request() {
        use std::io::Write;
        use std::sync::mpsc;

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let (first_seen_tx, first_seen_rx) = mpsc::channel();
        let (second_at_permit_tx, second_at_permit_rx) = mpsc::channel();
        let (checked_tx, checked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let server = std::thread::spawn(move || -> Result<Vec<T5WireRequest>, String> {
            let mut first = policy_accept(&listener)?;
            let first_request = t5_read_wire(&first)?;
            first_seen_tx.send(()).map_err(|error| error.to_string())?;
            second_at_permit_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .map_err(|error| error.to_string())?;
            let held_until = std::time::Instant::now() + std::time::Duration::from_millis(150);
            while std::time::Instant::now() < held_until {
                match listener.accept() {
                    Ok(_) => return Err("second wire request entered while first held permit".into()),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(2));
                    },
                    Err(error) => return Err(error.to_string()),
                }
            }
            checked_tx.send(()).map_err(|error| error.to_string())?;
            release_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .map_err(|error| error.to_string())?;
            first
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\na")
                .map_err(|error| error.to_string())?;
            let mut second = policy_accept(&listener)?;
            let second_request = t5_read_wire(&second)?;
            second
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\nb")
                .map_err(|error| error.to_string())?;
            Ok(vec![first_request, second_request])
        });
        let fetcher = Arc::new(RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_concurrent_fetches: 1,
            timeout: std::time::Duration::from_secs(2),
            ..ResourceFetchPolicy::default()
        }).unwrap());
        let first_fetcher = Arc::clone(&fetcher);
        let first_url = format!("{origin}/first");
        let first = std::thread::spawn(move || {
            first_fetcher.fetch_request_result(&ResourceRequest::plain_get(first_url))
        });
        first_seen_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        let second_fetcher = Arc::clone(&fetcher);
        let second_url = format!("{origin}/second");
        let second = std::thread::spawn(move || {
            second_fetcher.fetch_request_with_admission_hook(
                &ResourceRequest::plain_get(second_url),
                || second_at_permit_tx.send(()).unwrap(),
            )
        });
        checked_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        release_tx.send(()).unwrap();
        assert_eq!(first.join().unwrap().unwrap().bytes, b"a");
        assert_eq!(second.join().unwrap().unwrap().bytes, b"b");
        let requests = server.join().unwrap().unwrap();
        assert_eq!((requests[0].target.as_str(), requests[1].target.as_str()), ("/first", "/second"));
    }

    #[test]
    fn native_policy_redirect_limit_is_applied_to_remote_fetches() {
        let server = t5_http_server(vec![T5HttpReply {
            status: 302,
            location: Some("/next"),
            body: b"",
        }]);
        let fetcher = RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_redirects: 0,
            ..ResourceFetchPolicy::default()
        })
        .unwrap();
        assert_eq!(
            fetcher.fetch_request_result(&ResourceRequest::plain_get(format!("{}/start", server.origin))),
            Err(NativeFetchFailure::Network)
        );
        let requests = server.task.join().unwrap().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].target, "/start");
    }

    #[test]
    fn native_policy_rejects_unrepresentable_concurrency() {
        assert!(RemoteFetcher::with_policy(ResourceFetchPolicy {
            max_concurrent_fetches: tokio::sync::Semaphore::MAX_PERMITS + 1,
            ..ResourceFetchPolicy::default()
        })
        .is_err());
    }
}
