// SPDX-License-Identifier: MPL-2.0
//! One wire hop, independent of Fetch, CORS, cookies, redirects and decoding.
//! The host supplies the literal request and selects trust, pools and limits.
//! Imported from Genet's Netfetcher transport with source ancestry preserved.
#![forbid(unsafe_code)]
use std::future::Future;
use std::pin::Pin;

use bytes::Bytes;
use url::Url;

pub use http;
use http::Method;
/// Raw, undecoded response data chunks.
pub type BodyStream =
    std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

#[cfg(all(feature = "h3", not(target_arch = "wasm32")))]
pub mod h3_client;
#[cfg(feature = "http")]
pub mod hyper;
#[cfg(all(feature = "websocket", not(target_arch = "wasm32")))]
pub mod websocket;

/// One hop on the wire, after the Fetch algorithm has decided everything
/// observable about it: the exact URL, method, headers and body to send.
#[derive(Clone, Debug)]
pub struct WireRequest {
    pub url: Url,
    pub method: Method,
    /// Header names are ASCII-lowercase. Protocol encoding retains repeated
    /// values; a caller must not rely on field-name order in HTTP/2 or HTTP/3.
    pub headers: Vec<(String, String)>,
    /// A fully buffered body, empty or absent for GET and HEAD.
    pub body: Option<Bytes>,
    /// The origin advertised HTTP/3 via Alt-Svc. A transport with an h3 lane may
    /// try it first and fall back; one without ignores the hint.
    pub prefer_h3: bool,
}

/// The raw result of one hop: status, headers as received, and the undecoded
/// body stream. Produced identically by every transport so the loop's back half
/// is transport-agnostic.
pub struct RawResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: BodyStream,
}

impl RawResponse {
    /// A response whose whole body is already in hand, as a single-chunk stream.
    pub fn once(status: u16, headers: Vec<(String, String)>, body: Bytes) -> Self {
        Self {
            status,
            headers,
            body: Box::pin(futures_util::stream::once(async move {
                Ok::<_, std::io::Error>(body)
            })),
        }
    }
}

/// The future a [`Transport::send`] returns. Boxed so the trait is object-safe
/// and a host can hold any transport as `Arc<dyn Transport>`.
pub type TransportFuture<'a> = Pin<Box<dyn Future<Output = Option<RawResponse>> + Send + 'a>>;

/// Send one request and return its raw response, or `None` for a network
/// failure (which the Fetch algorithm turns into a network-error response).
pub trait Transport: Send + Sync {
    fn send(&self, request: WireRequest) -> TransportFuture<'_>;
}

/// A transport with no network: every send is a network failure. The default
/// when the crate is built without `hyper-transport`; also useful to a host
/// that wants a fetch context which can serve `data:` URLs and cache hits and
/// nothing else.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoTransport;

impl Transport for NoTransport {
    fn send(&self, _request: WireRequest) -> TransportFuture<'_> {
        Box::pin(async { None })
    }
}

#[cfg(all(
    test,
    feature = "http",
    feature = "websocket",
    not(target_arch = "wasm32")
))]
mod tests;
