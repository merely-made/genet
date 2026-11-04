// SPDX-License-Identifier: MPL-2.0
//! Polling address-resolution contract, independent of an executor or RTC peer.
use std::net::IpAddr;
use std::task::Poll;
use std::time::Instant;

mod endpoints;
pub use endpoints::{EndpointBatch, EndpointLookup, ServerRequest, Service};

/// Address family selected by the host's concrete RTC route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    /// DNS A records.
    V4,
    /// DNS AAAA records.
    V6,
}
impl Family {
    /// Whether an address belongs to this family.
    pub fn accepts(self, ip: IpAddr) -> bool {
        ip.is_ipv4() == (self == Self::V4)
    }
}

/// Sanitized resolution failure; never carries credentials or packet text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// No usable answer exists.
    NotFound,
    /// The caller's lookup deadline elapsed.
    Timeout,
    /// The caller's request/response budget was exhausted.
    Limit,
    /// No configured name server is available.
    Unavailable,
    /// Invalid name, response, alias loop or unsupported name scope.
    Invalid,
    /// All configured name-server routes failed.
    Io,
}

/// One owned lookup. Dropping it cancels its I/O and any later result.
pub trait Lookup: Send {
    /// Make bounded, nonblocking progress using the caller's clock.
    fn poll(&mut self, now: Instant) -> Poll<Result<Vec<IpAddr>, ResolveError>>;
    /// Latest time at which the caller should poll again, including I/O sampling.
    fn next_deadline(&self) -> Instant;
}

/// Host-supplied address resolver. Implementations must not block `start`/`poll`.
pub trait Resolver: Send + Sync {
    /// Start a request limited by the supplied deadline and address family.
    ///
    /// # Errors
    /// Returns a sanitized validation, capacity or configuration failure.
    fn start(
        &self,
        host: &str,
        family: Family,
        now: Instant,
        deadline: Instant,
    ) -> Result<Box<dyn Lookup>, ResolveError>;

    /// Resolve a remote UUID-v4 `.local` candidate on the selected interface.
    /// This must never use recursive DNS. Hosts without mDNS may refuse it.
    ///
    /// # Errors
    /// Returns a sanitized validation, capacity or unavailable-capability error.
    fn start_mdns(
        &self,
        _host: &str,
        _interface: IpAddr,
        _now: Instant,
        _deadline: Instant,
    ) -> Result<Box<dyn Lookup>, ResolveError> {
        Err(ResolveError::Unavailable)
    }

    /// Resolve a UDP server, retaining whether port/transport were omitted.
    /// Poll the returned cursor only when another endpoint batch is needed.
    ///
    /// # Errors
    /// The default supports explicit ports through `start`. Portless service
    /// discovery requires an implementation that overrides this method.
    fn start_server(
        &self,
        request: &ServerRequest,
        family: Family,
        now: Instant,
        deadline: Instant,
    ) -> Result<Box<dyn EndpointLookup>, ResolveError> {
        let port = request
            .port
            .filter(|p| *p != 0)
            .ok_or(ResolveError::Unavailable)?;
        Ok(endpoints::address_lookup(
            self.start(&request.host, family, now, deadline)?,
            port,
        ))
    }
}

#[cfg(feature = "dns")]
mod native;
#[cfg(feature = "dns")]
pub use native::mdns::MdnsConfig;
#[cfg(feature = "dns")]
pub use native::{DnsConfig, DnsLimits, DnsResolver};

/// Restrict peer discovery to the ephemeral name format used by WebRTC agents.
pub fn is_mdns_candidate_name(host: &str) -> bool {
    let bytes = host.as_bytes();
    bytes.len() == 42
        && bytes[36..].eq_ignore_ascii_case(b".local")
        && bytes[..36].iter().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                *b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
        && bytes[14] == b'4'
        && matches!(bytes[19].to_ascii_lowercase(), b'8' | b'9' | b'a' | b'b')
}
