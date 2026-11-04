// SPDX-License-Identifier: MPL-2.0
use super::*;
use std::net::SocketAddr;

/// The UDP service requested by a native host. Secure transports are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    /// STUN Binding discovery.
    Stun,
    /// TURN allocation discovery.
    Turn,
}

/// UDP URL components before parser defaults erase omission.
#[derive(Clone, Debug)]
pub struct ServerRequest {
    /// Original server domain; never credentials or a URI.
    pub host: String,
    /// Service whose DNS discovery rules apply.
    pub service: Service,
    /// Explicit port, or `None` to discover the service.
    pub port: Option<u16>,
    /// Whether the TURN URI explicitly selected UDP. If omitted, use S-NAPTR.
    pub transport_specified: bool,
}

/// One target's route-compatible addresses, ready for a protocol attempt.
pub struct EndpointBatch {
    /// Addresses with the discovered target port.
    pub addresses: Vec<SocketAddr>,
    /// Untried service branches/targets, used to share the gathering deadline.
    pub remaining_targets: usize,
}

/// Owned service-discovery cursor. Drop cancels I/O and untried alternatives.
pub trait EndpointLookup: Send {
    /// Make bounded progress toward the next target. `Ok(None)` means exhausted.
    /// Do not poll ahead while the consumer is trying a previously yielded batch.
    fn poll_next(&mut self, now: Instant) -> Poll<Result<Option<EndpointBatch>, ResolveError>>;
    /// Wakeup for the next needed batch; an idle/finished cursor may return none.
    fn next_deadline(&self) -> Option<Instant>;
}

struct Addresses {
    query: Option<Box<dyn Lookup>>,
    port: u16,
}
pub(super) fn address_lookup(query: Box<dyn Lookup>, port: u16) -> Box<dyn EndpointLookup> {
    Box::new(Addresses {
        query: Some(query),
        port,
    })
}
impl EndpointLookup for Addresses {
    fn next_deadline(&self) -> Option<Instant> {
        self.query.as_ref().map(|q| q.next_deadline())
    }
    fn poll_next(&mut self, now: Instant) -> Poll<Result<Option<EndpointBatch>, ResolveError>> {
        let Some(query) = &mut self.query else {
            return Poll::Ready(Ok(None));
        };
        match query.poll(now) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                self.query = None;
                Poll::Ready(result.map(|addresses| {
                    Some(EndpointBatch {
                        addresses: addresses
                            .into_iter()
                            .map(|ip| SocketAddr::new(ip, self.port))
                            .collect(),
                        remaining_targets: 0,
                    })
                }))
            }
        }
    }
}
