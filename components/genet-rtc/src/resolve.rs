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
pub use native::{DnsConfig, DnsLimits, DnsResolver};
