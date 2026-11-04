// SPDX-License-Identifier: MPL-2.0
use super::*;

/// A UDP server name with original URI omission preserved before defaults.
#[derive(Clone)]
pub struct NamedServer {
    /// Parsed URI and TURN credentials; deliberately not Debug.
    pub url: Url,
    /// Explicit URI port, or `None` to discover the service port.
    pub port: Option<u16>,
    /// Whether the TURN URI explicitly supplied `transport=udp`.
    pub transport_specified: bool,
}
impl From<Url> for NamedServer {
    /// Keep the existing address-only interpretation of an already parsed URL.
    fn from(url: Url) -> Self {
        Self {
            port: Some(url.port),
            transport_specified: url.scheme == SchemeType::Turn,
            url,
        }
    }
}
impl NamedServer {
    pub(super) fn metadata(url: &Url, port: Option<u16>, transport_specified: bool) -> String {
        if port.is_some() {
            return url.to_string();
        }
        let query = if url.scheme == SchemeType::Turn && transport_specified {
            "?transport=udp"
        } else {
            ""
        };
        format!("{}:{}{}", url.scheme, url.host, query)
    }
}
