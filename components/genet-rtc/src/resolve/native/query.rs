// SPDX-License-Identifier: MPL-2.0
use super::io::Io;
use super::*;
use hickory_proto::op::Message;
use hickory_proto::rr::{Name, Record, RecordType};

pub(super) struct Query {
    config: DnsConfig,
    current: Name,
    record_type: RecordType,
    allow_alias: bool,
    request: Message,
    packet: Vec<u8>,
    io: Option<Io>,
    server: usize,
    deadline: Instant,
    retry: Instant,
    wake: Instant,
    aliases: Vec<Name>,
    result: Option<Result<Vec<Record>, ResolveError>>,
    lease: Option<Lease>,
}
impl Query {
    pub fn new(
        config: DnsConfig,
        name: Name,
        record_type: RecordType,
        allow_alias: bool,
        now: Instant,
        deadline: Instant,
        lease: Option<Lease>,
    ) -> Result<Self, ResolveError> {
        let (request, packet) = wire::request(&name, record_type)?;
        let mut query = Self {
            config,
            current: name.clone(),
            record_type,
            allow_alias,
            request,
            packet,
            io: None,
            server: 0,
            deadline,
            retry: now,
            wake: now,
            aliases: vec![name],
            result: None,
            lease,
        };
        query.open(now)?;
        Ok(query)
    }
    fn open(&mut self, now: Instant) -> Result<(), ResolveError> {
        for _ in 0..self.config.servers.len() {
            let (request, packet) = wire::request(&self.current, self.record_type)?;
            self.request = request;
            self.packet = packet;
            match Io::udp(self.config.servers[self.server]) {
                Ok(io) => {
                    self.io = Some(io);
                    self.retry = now
                        .checked_add(self.config.limits.retry_interval)
                        .ok_or(ResolveError::Limit)?
                        .min(self.deadline);
                    return Ok(());
                }
                Err(_) => self.server = (self.server + 1) % self.config.servers.len(),
            }
        }
        Err(ResolveError::Io)
    }
    fn finish(
        &mut self,
        result: Result<Vec<Record>, ResolveError>,
    ) -> Poll<Result<Vec<Record>, ResolveError>> {
        self.io = None;
        self.lease = None;
        self.result = Some(result.clone());
        Poll::Ready(result)
    }
    fn retry(&mut self, now: Instant) -> Result<(), ResolveError> {
        self.io = None;
        self.server = (self.server + 1) % self.config.servers.len();
        self.open(now)
    }
}
impl Query {
    pub fn next_deadline(&self) -> Instant {
        self.wake.min(self.retry).min(self.deadline)
    }
    pub fn poll(&mut self, now: Instant) -> Poll<Result<Vec<Record>, ResolveError>> {
        if let Some(result) = &self.result {
            return Poll::Ready(result.clone());
        }
        if now >= self.deadline {
            return self.finish(Err(ResolveError::Timeout));
        }
        if now >= self.retry
            && let Err(e) = self.retry(now)
        {
            return self.finish(Err(e));
        }
        self.wake = now
            .checked_add(self.config.limits.poll_interval)
            .unwrap_or(self.deadline)
            .min(self.deadline);
        for _ in 0..self.config.limits.packets_per_poll {
            let packet = match self
                .io
                .as_mut()
                .expect("pending query owns I/O")
                .poll(&self.packet)
            {
                Ok(Some(packet)) => packet,
                Ok(None) => break,
                Err(_) => {
                    if let Err(e) = self.retry(now) {
                        return self.finish(Err(e));
                    }
                    break;
                }
            };
            match wire::answer(&packet, &self.request, self.config.limits) {
                Ok(wire::Answer::Ignore) => {}
                Ok(wire::Answer::Truncated) => {
                    if matches!(&self.io, Some(Io::Tcp { .. })) {
                        return self.finish(Err(ResolveError::Invalid));
                    }
                    match Io::tcp(
                        self.config.servers[self.server],
                        &self.packet,
                        self.config.limits.max_response_bytes,
                    ) {
                        Ok(io) => self.io = Some(io),
                        Err(_) => {
                            if let Err(e) = self.retry(now) {
                                return self.finish(Err(e));
                            }
                        }
                    }
                }
                Ok(wire::Answer::Records(records, path)) => {
                    if let Err(e) = self.remember(&path) {
                        return self.finish(Err(e));
                    }
                    return self.finish(Ok(records));
                }
                Ok(wire::Answer::Alias(path)) => {
                    if let Err(e) = self.remember(&path) {
                        return self.finish(Err(e));
                    }
                    self.current = path.last().expect("alias has a target").clone();
                    if wire::name(&self.current.to_ascii()).is_err() {
                        return self.finish(Err(ResolveError::Invalid));
                    }
                    if let Err(e) = self.open(now) {
                        return self.finish(Err(e));
                    }
                }
                Err(ResolveError::Io) => {
                    if let Err(e) = self.retry(now) {
                        return self.finish(Err(e));
                    }
                }
                Err(e) => return self.finish(Err(e)),
            }
        }
        Poll::Pending
    }
}
impl Query {
    fn remember(&mut self, path: &[Name]) -> Result<(), ResolveError> {
        if !self.allow_alias && path.len() > 1 {
            return Err(ResolveError::Invalid);
        }
        for name in path.iter().skip(1) {
            if self.aliases.contains(name) {
                return Err(ResolveError::Invalid);
            }
            if self.aliases.len() > self.config.limits.max_aliases {
                return Err(ResolveError::Limit);
            }
            self.aliases.push(name.clone());
        }
        Ok(())
    }
}
