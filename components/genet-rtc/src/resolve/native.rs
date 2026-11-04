// SPDX-License-Identifier: MPL-2.0
use super::*;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

mod address;
#[cfg(test)]
mod fixture;
mod io;
mod query;
mod service;
#[cfg(test)]
mod tests;
mod wire;

/// Caller-configurable lookup and allocation limits.
#[derive(Clone, Copy, Debug)]
pub struct DnsLimits {
    /// Maximum concurrent owned requests across resolver clones.
    pub max_queries: usize,
    /// Maximum addresses returned by a request.
    pub max_addresses: usize,
    /// Maximum DNS records accepted before parsing a response.
    pub max_records: usize,
    /// Maximum DNS response bytes, also enforced on TCP framing.
    pub max_response_bytes: usize,
    /// Maximum alias depth, including aliases in one answer.
    pub max_aliases: usize,
    /// Maximum untried service targets/branches and yielded endpoints per context.
    pub max_targets: usize,
    /// Maximum logical service queries; each keeps its own alias/retry limits.
    pub max_service_queries: usize,
    /// Maximum nonterminal NAPTR path depth, including the original domain.
    pub max_delegations: usize,
    /// Maximum lookup duration, further bounded by the gathering deadline.
    pub timeout: Duration,
    /// Name-server retry interval; configured servers are tried in order.
    pub retry_interval: Duration,
    /// Polling interval for socket readiness; no background task is created.
    pub poll_interval: Duration,
    /// Maximum socket reads per host turn.
    pub packets_per_poll: usize,
}
impl Default for DnsLimits {
    fn default() -> Self {
        Self {
            max_queries: 32,
            max_addresses: 8,
            max_records: 64,
            max_response_bytes: 16384,
            max_aliases: 8,
            max_targets: 32,
            max_service_queries: 32,
            max_delegations: 8,
            timeout: Duration::from_secs(2),
            retry_interval: Duration::from_millis(250),
            poll_interval: Duration::from_millis(5),
            packets_per_poll: 4,
        }
    }
}

/// Explicit name-server placement. No public fallback resolver is invented.
#[derive(Clone, Debug)]
pub struct DnsConfig {
    /// Recursive name servers in caller preference order (up to 32).
    pub servers: Vec<SocketAddr>,
    /// Resource and timing policy.
    pub limits: DnsLimits,
}
impl DnsConfig {
    /// Read local OS name-server configuration. Performs no DNS query or bind.
    /// Missing configuration leaves an empty list; numeric RTC routes still work.
    pub fn system(limits: DnsLimits) -> Self {
        #[cfg(windows)]
        let addresses: Vec<IpAddr> = ipconfig::get_adapters()
            .unwrap_or_default()
            .into_iter()
            .filter(|a| a.oper_status() == ipconfig::OperStatus::IfOperStatusUp)
            .flat_map(|a| a.dns_servers().to_vec())
            .collect();
        #[cfg(not(windows))]
        let addresses: Vec<IpAddr> = std::fs::read_to_string("/etc/resolv.conf")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let mut fields = line.split_whitespace();
                (fields.next() == Some("nameserver"))
                    .then(|| fields.next()?.parse().ok())
                    .flatten()
            })
            .collect();
        let mut servers = Vec::new();
        for ip in addresses.into_iter().filter(|ip| !ip.is_unspecified()) {
            let address = SocketAddr::new(ip, 53);
            if !servers.contains(&address) && servers.len() < 32 {
                servers.push(address);
            }
        }
        Self { servers, limits }
    }
}

/// Native recursive DNS client with owned nonblocking UDP/TCP requests.
/// TCP is used for truncated DNS responses, independently of TURN transport.
#[derive(Clone)]
pub struct DnsResolver {
    config: DnsConfig,
    active: Arc<AtomicUsize>,
}
impl DnsResolver {
    /// Validate caller policy. Creates no thread, task, lookup or socket.
    ///
    /// # Errors
    /// Returns `Limit` for invalid limits or name-server endpoints.
    pub fn new(config: DnsConfig) -> Result<Self, ResolveError> {
        let l = config.limits;
        if config.servers.len() > 32
            || config
                .servers
                .iter()
                .any(|a| a.ip().is_unspecified() || a.port() == 0)
            || l.max_queries == 0
            || !(1..=128).contains(&l.max_addresses)
            || !(1..=512).contains(&l.max_records)
            || !(512..=65535).contains(&l.max_response_bytes)
            || !(1..=32).contains(&l.max_aliases)
            || !(1..=128).contains(&l.max_targets)
            || !(1..=256).contains(&l.max_service_queries)
            || !(1..=32).contains(&l.max_delegations)
            || l.timeout.is_zero()
            || l.retry_interval.is_zero()
            || l.poll_interval.is_zero()
            || l.packets_per_poll == 0
        {
            return Err(ResolveError::Limit);
        }
        Ok(Self {
            config,
            active: Arc::new(AtomicUsize::new(0)),
        })
    }
    /// Number of owned pending lookup contexts, including idle service cursors.
    pub fn pending(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }
}
impl Resolver for DnsResolver {
    fn start(
        &self,
        host: &str,
        family: Family,
        now: Instant,
        deadline: Instant,
    ) -> Result<Box<dyn Lookup>, ResolveError> {
        if self.config.servers.is_empty() {
            return Err(ResolveError::Unavailable);
        }
        let name = wire::name(host)?;
        let deadline = deadline.min(
            now.checked_add(self.config.limits.timeout)
                .ok_or(ResolveError::Limit)?,
        );
        if deadline <= now {
            return Err(ResolveError::Timeout);
        }
        let lease = self.reserve()?;
        let record_type = if family == Family::V4 {
            hickory_proto::rr::RecordType::A
        } else {
            hickory_proto::rr::RecordType::AAAA
        };
        let query = query::Query::new(
            self.config.clone(),
            name,
            record_type,
            true,
            now,
            deadline,
            Some(lease),
        )?;
        Ok(Box::new(address::AddressQuery {
            query,
            family,
            limit: self.config.limits.max_addresses,
        }))
    }
    fn start_server(
        &self,
        request: &ServerRequest,
        family: Family,
        now: Instant,
        deadline: Instant,
    ) -> Result<Box<dyn EndpointLookup>, ResolveError> {
        if let Some(port) = request.port {
            if port == 0 {
                return Err(ResolveError::Invalid);
            }
            return Ok(super::endpoints::address_lookup(
                self.start(&request.host, family, now, deadline)?,
                port,
            ));
        }
        if self.config.servers.is_empty() {
            return Err(ResolveError::Unavailable);
        }
        let name = wire::name(&request.host)?;
        if deadline <= now {
            return Err(ResolveError::Timeout);
        }
        Ok(Box::new(service::Cursor::new(
            self.clone(),
            request,
            name,
            family,
            now,
            deadline,
            self.reserve()?,
        )?))
    }
}
impl DnsResolver {
    fn reserve(&self) -> Result<Lease, ResolveError> {
        let mut count = self.active.load(Ordering::Acquire);
        loop {
            if count >= self.config.limits.max_queries {
                return Err(ResolveError::Limit);
            }
            match self.active.compare_exchange_weak(
                count,
                count + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(Lease(self.active.clone())),
                Err(actual) => count = actual,
            }
        }
    }
}
struct Lease(Arc<AtomicUsize>);
impl Drop for Lease {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}
