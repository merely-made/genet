/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
//! Netfetcher's adapter to Murm's literal, policy-free wire engine.
use super::{RawResponse, Transport, TransportFuture, WireRequest};
use crate::request::Method;
use std::sync::{Arc, OnceLock};

/// Compatibility knob for the process-wide test transport, called before its
/// first request. Explicit host-owned engines never consult this setting.
pub fn accept_invalid_certs() {
    murm_wire::hyper::accept_invalid_certs();
}

/// Preserve the public process-wide Netfetcher default and its trait identity.
pub struct DefaultTransport {
    engine: WireTransport,
}
impl DefaultTransport {
    pub fn shared() -> Arc<dyn Transport> {
        static SHARED: OnceLock<Arc<DefaultTransport>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                Arc::new(Self {
                    engine: WireTransport::new(murm_wire::hyper::DefaultTransport::shared()),
                })
            })
            .clone()
    }
}
impl Transport for DefaultTransport {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        self.engine.send(request)
    }
}
/// Adapt an explicit host-selected Murm engine without changing Fetch policy.
/// The engine owns its trust, pool partition, connection limits and retry policy.
pub struct WireTransport {
    engine: Arc<dyn murm_wire::Transport>,
}
impl WireTransport {
    pub fn new(engine: Arc<dyn murm_wire::Transport>) -> Self {
        Self { engine }
    }
}
impl Transport for WireTransport {
    fn send(&self, request: WireRequest) -> TransportFuture<'_> {
        let request = murm_wire::WireRequest {
            url: request.url,
            method: http_method(&request.method),
            headers: request.headers,
            body: request.body,
            prefer_h3: request.prefer_h3,
        };
        Box::pin(async move {
            self.engine.send(request).await.map(|response| RawResponse {
                status: response.status,
                headers: response.headers,
                body: response.body,
            })
        })
    }
}
fn http_method(method: &Method) -> murm_wire::http::Method {
    use murm_wire::http;
    match method {
        Method::Get => http::Method::GET,
        Method::Head => http::Method::HEAD,
        Method::Post => http::Method::POST,
        Method::Put => http::Method::PUT,
        Method::Delete => http::Method::DELETE,
        Method::Patch => http::Method::PATCH,
        Method::Options => http::Method::OPTIONS,
        // Preserve the established adapter behavior for a directly constructed
        // invalid token. Fetch validates its own methods before this seam.
        Method::Other(token) => {
            http::Method::from_bytes(token.as_bytes()).unwrap_or(http::Method::GET)
        },
    }
}
