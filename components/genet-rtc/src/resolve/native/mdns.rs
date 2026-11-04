// SPDX-License-Identifier: MPL-2.0
//! Bounded one-shot client, including browsers that multicast their replies.
use super::*;
use hickory_proto::op::{Message, MessageType, OpCode, Query as Question, ResponseCode};
use hickory_proto::rr::{DNSClass, Name, RData, RecordType};
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{Ipv4Addr, UdpSocket};

mod link;
#[cfg(test)]
mod tests;

/// Selected local-link placement and bounded one-shot query policy.
#[derive(Clone, Copy, Debug)]
pub struct MdnsConfig {
    /// Concrete IPv4 interface; never a wildcard or automatic route choice.
    pub interface: Ipv4Addr,
    /// Actual on-link prefix length, used to validate unicast response sources.
    pub prefix_len: u8,
    /// Overall lifetime of a request, bounded further by the caller's deadline.
    pub timeout: Duration,
    /// Retry spacing; at least one second to bound multicast traffic.
    pub retry_interval: Duration,
    /// Maximum transmitted queries per request.
    pub max_attempts: u8,
}
impl MdnsConfig {
    /// Build explicit placement; validation occurs when enabling the resolver.
    pub fn new(interface: Ipv4Addr, prefix_len: u8) -> Self {
        Self {
            interface,
            prefix_len,
            timeout: Duration::from_secs(3),
            retry_interval: Duration::from_secs(1),
            max_attempts: 3,
        }
    }
    /// Read the selected interface's on-link prefix, without querying DNS.
    ///
    /// # Errors
    /// Returns `Unavailable` if the interface/prefix cannot be determined.
    pub fn for_interface(interface: Ipv4Addr) -> Result<Self, ResolveError> {
        Ok(Self::new(interface, link::prefix(interface)?))
    }
    pub(super) fn validate(self) -> Result<(), ResolveError> {
        if self.interface.is_unspecified()
            || self.interface.is_multicast()
            || self.interface.is_broadcast()
            || !(1..=32).contains(&self.prefix_len)
            || self.timeout.is_zero()
            || self.retry_interval < Duration::from_secs(1)
            || !(1..=8).contains(&self.max_attempts)
        {
            return Err(ResolveError::Limit);
        }
        Ok(())
    }
    fn on_link(self, address: Ipv4Addr) -> bool {
        let mask = u32::MAX << (32 - self.prefix_len);
        u32::from(address) & mask == u32::from(self.interface) & mask
    }
}

pub(super) struct Query {
    socket: Option<UdpSocket>,
    multicast: Option<UdpSocket>,
    read_multicast: bool,
    config: MdnsConfig,
    limits: DnsLimits,
    request: Message,
    packet: Vec<u8>,
    buffer: Vec<u8>,
    deadline: Instant,
    retry: Instant,
    wake: Instant,
    attempts: u8,
    lease: Option<Lease>,
    resolved: Arc<AtomicUsize>,
    result: Option<Result<Vec<IpAddr>, ResolveError>>,
}
impl Query {
    pub fn new(
        host: &str,
        config: MdnsConfig,
        limits: DnsLimits,
        now: Instant,
        deadline: Instant,
        lease: Lease,
        resolved: Arc<AtomicUsize>,
    ) -> Result<Self, ResolveError> {
        if !is_mdns_candidate_name(host) {
            return Err(ResolveError::Invalid);
        }
        let deadline = deadline.min(now.checked_add(config.timeout).ok_or(ResolveError::Limit)?);
        if deadline <= now {
            return Err(ResolveError::Timeout);
        }
        let name = Name::from_ascii(format!("{host}.")).map_err(|_| ResolveError::Invalid)?;
        let mut nonce = [0; 2];
        getrandom::fill(&mut nonce).map_err(|_| ResolveError::Unavailable)?;
        let mut request =
            Message::new(u16::from_ne_bytes(nonce), MessageType::Query, OpCode::Query);
        request.queries.push(Question::query(name, RecordType::A));
        let packet = request.to_vec().map_err(|_| ResolveError::Invalid)?;
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|_| ResolveError::Io)?;
        socket
            .set_multicast_if_v4(&config.interface)
            .map_err(|_| ResolveError::Io)?;
        socket
            .set_multicast_ttl_v4(255)
            .map_err(|_| ResolveError::Io)?;
        socket
            .set_multicast_loop_v4(true)
            .map_err(|_| ResolveError::Io)?;
        socket
            .bind(&SocketAddr::new(config.interface.into(), 0).into())
            .map_err(|_| ResolveError::Io)?;
        socket.set_nonblocking(true).map_err(|_| ResolveError::Io)?;
        if socket
            .local_addr()
            .map_err(|_| ResolveError::Io)?
            .as_socket()
            .is_none_or(|a| a.port() == 5353)
        {
            return Err(ResolveError::Io);
        }
        // Some browser responders multicast even replies to an ephemeral-port
        // query. This receive-only membership lives exactly as long as this
        // request; it never advertises, sends from 5353 or caches other names.
        let multicast = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|_| ResolveError::Io)?;
        multicast
            .set_reuse_address(true)
            .map_err(|_| ResolveError::Io)?;
        multicast
            .bind(&"0.0.0.0:5353".parse::<SocketAddr>().unwrap().into())
            .map_err(|_| ResolveError::Io)?;
        multicast
            .join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &config.interface)
            .map_err(|_| ResolveError::Io)?;
        multicast
            .set_nonblocking(true)
            .map_err(|_| ResolveError::Io)?;
        Ok(Self {
            socket: Some(socket.into()),
            multicast: Some(multicast.into()),
            read_multicast: false,
            config,
            limits,
            request,
            packet,
            buffer: vec![0; limits.max_response_bytes + 1],
            deadline,
            retry: now,
            wake: now,
            attempts: 0,
            lease: Some(lease),
            resolved,
            result: None,
        })
    }
    fn finish(
        &mut self,
        result: Result<Vec<IpAddr>, ResolveError>,
    ) -> Poll<Result<Vec<IpAddr>, ResolveError>> {
        self.socket = None;
        self.multicast = None;
        self.lease = None;
        if result.is_ok() {
            self.resolved.fetch_add(1, Ordering::Relaxed);
        }
        self.result = Some(result.clone());
        Poll::Ready(result)
    }
    fn answer(&self, bytes: &[u8], multicast: bool) -> Result<Option<IpAddr>, ResolveError> {
        if bytes.len() < 12 || bytes.len() > self.limits.max_response_bytes {
            return Ok(None);
        }
        let count = |n| usize::from(u16::from_be_bytes([bytes[n], bytes[n + 1]]));
        if count(4) > 1 || count(6) + count(8) + count(10) > self.limits.max_records {
            return Ok(None);
        }
        let Ok(message) = Message::from_vec(bytes) else {
            return Ok(None);
        };
        let identity = if multicast && count(4) == 0 {
            // Chromium uses zero; Firefox echoes the request ID even on its
            // multicast, questionless replies. Other transaction IDs fail.
            message.metadata.id == 0 || message.metadata.id == self.request.metadata.id
        } else {
            message.metadata.id == self.request.metadata.id
                && message.queries == self.request.queries
        };
        if !identity
            || message.metadata.message_type != MessageType::Response
            || message.metadata.op_code != OpCode::Query
            || !message.metadata.authoritative
            || message.metadata.truncation
            || message.metadata.response_code != ResponseCode::NoError
        {
            return Ok(None);
        }
        let mut addresses = Vec::new();
        for record in &message.answers {
            if record.name != self.request.queries[0].name().clone()
                || u16::from(record.dns_class) & 0x7fff != u16::from(DNSClass::IN)
            {
                continue;
            }
            if matches!(record.data, RData::CNAME(_)) {
                return Err(ResolveError::Invalid);
            }
            if record.ttl == 0 {
                continue;
            }
            let ip = match &record.data {
                RData::A(a) => IpAddr::V4(a.0),
                RData::AAAA(a) => IpAddr::V6(a.0),
                _ => continue,
            };
            if !addresses.contains(&ip) {
                addresses.push(ip);
            }
        }
        if addresses.len() > 1 {
            return Err(ResolveError::Invalid);
        }
        Ok(addresses.first().copied().filter(|ip| {
            ip.is_ipv4()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !matches!(ip, IpAddr::V4(a) if a.is_broadcast())
        }))
    }
}
impl Lookup for Query {
    fn next_deadline(&self) -> Instant {
        self.wake.min(self.retry).min(self.deadline)
    }
    fn poll(&mut self, now: Instant) -> Poll<Result<Vec<IpAddr>, ResolveError>> {
        if let Some(result) = &self.result {
            return Poll::Ready(result.clone());
        }
        if now >= self.deadline {
            return self.finish(Err(ResolveError::Timeout));
        }
        self.wake = now
            .checked_add(self.limits.poll_interval)
            .unwrap_or(self.deadline)
            .min(self.deadline);
        if now >= self.retry && self.attempts < self.config.max_attempts {
            let sent = self
                .socket
                .as_ref()
                .unwrap()
                .send_to(&self.packet, "224.0.0.251:5353");
            match sent {
                Ok(n) if n == self.packet.len() => {
                    self.attempts += 1;
                    self.retry = now
                        .checked_add(self.config.retry_interval)
                        .unwrap_or(self.deadline)
                        .min(self.deadline);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => self.retry = self.wake,
                _ => return self.finish(Err(ResolveError::Io)),
            }
        } else if self.attempts == self.config.max_attempts {
            self.retry = self.deadline;
        }
        for _ in 0..self.limits.packets_per_poll {
            self.read_multicast = !self.read_multicast;
            let socket = if self.read_multicast {
                self.multicast.as_ref().unwrap()
            } else {
                self.socket.as_ref().unwrap()
            };
            match socket.recv_from(&mut self.buffer) {
                Ok((n, SocketAddr::V4(from)))
                    if from.port() == 5353 && self.config.on_link(*from.ip()) =>
                {
                    match self.answer(&self.buffer[..n], self.read_multicast) {
                        Ok(Some(ip)) => return self.finish(Ok(vec![ip])),
                        Ok(None) => {}
                        Err(e) => return self.finish(Err(e)),
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) if cfg!(windows) && e.raw_os_error() == Some(10040) => {}
                Err(_) => return self.finish(Err(ResolveError::Io)),
            }
        }
        Poll::Pending
    }
}
