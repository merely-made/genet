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
use genet_host_api::{ResourceFetchResponse, ResourceFetcher, ResourceRequest, ResourceResponse};

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
        Ok(Self {
            remote: Some(Arc::new(RemoteFetcher::new()?)),
        })
    }
}

impl ResourceFetcher for OrtetFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        match lane_for(&request.url) {
            FetchLane::Local if request.is_plain_get() => {
                LocalFetcher.fetch_response(&request.url).map(Into::into)
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
}

impl RemoteFetcher {
    pub fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("ortet-fetch")
            .enable_all()
            .build()
            .map_err(|error| format!("could not start the ortet fetch runtime: {error}"))?;
        Ok(Self {
            runtime,
            context: netfetcher::FetchContext::permissive(),
        })
    }
}

impl ResourceFetcher for RemoteFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.fetch_request(&ResourceRequest {
            url: url.into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
            cache: "default".into(),
            redirect: "follow".into(),
            mode: "cors".into(),
            referrer: String::new(),
            referrer_policy: String::new(),
            credentials: "same-origin".into(),
            integrity: String::new(),
        })
        .map(Into::into)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        if lane_for(&request.url) != FetchLane::Remote {
            return None;
        }
        let parsed = url::Url::parse(&request.url).ok()?;
        let mut wire = netfetcher::Request::get(parsed);
        wire.method = match request.method.as_str() {
            "GET" => netfetcher::Method::Get,
            "HEAD" => netfetcher::Method::Head,
            "POST" => netfetcher::Method::Post,
            "PUT" => netfetcher::Method::Put,
            "DELETE" => netfetcher::Method::Delete,
            "PATCH" => netfetcher::Method::Patch,
            "OPTIONS" => netfetcher::Method::Options,
            other => netfetcher::Method::Other(other.into()),
        };
        wire.headers = request.headers.clone();
        wire.body = request.body.clone().map(Into::into);
        wire.cache = match request.cache.as_str() {
            "default" => netfetcher::CacheMode::Default,
            "no-store" => netfetcher::CacheMode::NoStore,
            "reload" => netfetcher::CacheMode::Reload,
            "no-cache" => netfetcher::CacheMode::NoCache,
            "force-cache" => netfetcher::CacheMode::ForceCache,
            "only-if-cached" => netfetcher::CacheMode::OnlyIfCached,
            _ => return None,
        };
        wire.redirect = match request.redirect.as_str() {
            "follow" => netfetcher::RedirectMode::Follow,
            "error" => netfetcher::RedirectMode::Error,
            "manual" => netfetcher::RedirectMode::Manual,
            _ => return None,
        };
        wire.mode = match request.mode.as_str() {
            "cors" => netfetcher::RequestMode::Cors,
            "no-cors" => netfetcher::RequestMode::NoCors,
            "same-origin" => netfetcher::RequestMode::SameOrigin,
            "navigate" => netfetcher::RequestMode::Navigate,
            _ => return None,
        };
        wire.referrer = if request.referrer.is_empty() {
            None
        } else {
            Some(url::Url::parse(&request.referrer).ok()?)
        };
        wire.origin = wire.referrer.as_ref().map(url::Url::origin);
        wire.referrer_policy = match request.referrer_policy.as_str() {
            "" => netfetcher::ReferrerPolicy::Empty,
            "no-referrer" => netfetcher::ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => netfetcher::ReferrerPolicy::NoReferrerWhenDowngrade,
            "same-origin" => netfetcher::ReferrerPolicy::SameOrigin,
            "origin" => netfetcher::ReferrerPolicy::Origin,
            "strict-origin" => netfetcher::ReferrerPolicy::StrictOrigin,
            "origin-when-cross-origin" => netfetcher::ReferrerPolicy::OriginWhenCrossOrigin,
            "strict-origin-when-cross-origin" => {
                netfetcher::ReferrerPolicy::StrictOriginWhenCrossOrigin
            },
            "unsafe-url" => netfetcher::ReferrerPolicy::UnsafeUrl,
            _ => return None,
        };
        wire.credentials = match request.credentials.as_str() {
            "omit" => netfetcher::Credentials::Omit,
            "same-origin" => netfetcher::Credentials::SameOrigin,
            "include" => netfetcher::Credentials::Include,
            _ => return None,
        };
        wire.integrity = request.integrity.clone();
        if request.mode == "navigate" {
            wire.destination = netfetcher::Destination::Document;
        }
        self.runtime.block_on(async {
            let response = netfetcher::fetch(wire, &self.context).await;
            if response.is_network_error() {
                return None;
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
            let bytes = response.bytes().await.ok()?.to_vec();
            let mut resource: ResourceFetchResponse =
                ResourceResponse::new(final_url, bytes).into();
            resource.content_type = content_type;
            resource.status = status;
            // netfetcher exposes the numeric status, not the transport reason phrase.
            resource.status_text = String::new();
            resource.headers = headers;
            resource.response_type = response_type;
            resource.redirected = redirected;
            Some(resource)
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
            credentials:"include".into(),integrity:String::new()}
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
}
