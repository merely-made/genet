//! STUN/TURN routing over the peer's existing physical socket.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rtc::crypto::RTCCryptoProvider;
use rtc::ice::url::{ProtoType, SchemeType, Url};
use rtc::peer_connection::RTCPeerConnection;
use rtc::peer_connection::transport::{
    CandidateConfig, CandidateRelayConfig, CandidateServerReflexiveConfig, RTCIceCandidate,
};
use rtc::sansio::Protocol;
use rtc::shared::{TaggedBytesMut, TransportContext, TransportProtocol};
use rtc::stun::message::{
    CLASS_ERROR_RESPONSE, CLASS_INDICATION, CLASS_SUCCESS_RESPONSE, Message, is_stun_message,
};
use rtc::turn::client::{Client, ClientConfig, Event};
use rtc::turn::proto::chandata::ChannelData;

use super::Error;
use crate::resolve::{Family, Lookup, Resolver};
use std::task::Poll;

#[cfg(test)]
mod dns_tests;
mod managed;

/// An ICE server resolved by the host. Credentials are deliberately not Debug.
#[derive(Clone)]
pub struct Server {
    /// Parsed URL, including TURN credentials. Its display form omits credentials.
    pub url: Url,
    /// Concrete server endpoint. Resolution is outside this socket driver.
    pub address: SocketAddr,
}

/// Discovery policy supplied by the host, independently of the protocol engine.
#[derive(Clone)]
pub struct DiscoveryConfig {
    /// At most 32 resolved UDP servers. Duplicate TURN endpoints are refused.
    pub servers: Vec<Server>,
    /// UDP server URLs whose host names need address resolution. Their ports
    /// are concrete; service-record discovery belongs to the caller.
    pub named_servers: Vec<Url>,
    /// Caller-owned nonblocking resolver, used only when gathering starts.
    pub resolver: Option<Arc<dyn Resolver>>,
    /// Suppress all host/reflexive candidates and all direct peer traffic.
    pub relay_only: bool,
    /// Maximum gathering duration, including authentication and retransmission.
    pub timeout: Duration,
    /// Optional allocation refresh interval cap, principally useful for qualification.
    pub allocation_refresh_interval_cap: Option<Duration>,
    /// The same provider supplied to the peer connection's SettingEngine.
    pub crypto_provider: Arc<dyn RTCCryptoProvider>,
}

struct Managed {
    client: Option<Client>,
    url: String,
    address: Option<SocketAddr>,
    relay: Option<SocketAddr>,
    reflexive: Option<SocketAddr>,
    done: bool,
    turn: bool,
    started: bool,
    spec: Url,
    addresses: VecDeque<SocketAddr>,
    lookup: Option<Box<dyn Lookup>>,
    attempt_deadline: Option<Instant>,
    failed: bool,
}

pub(super) struct Discovery {
    local: SocketAddr,
    config: DiscoveryConfig,
    clients: Vec<Managed>,
    relay_only: bool,
    timeout: Duration,
    deadline: Option<Instant>,
    started: bool,
    finished: bool,
    errors: VecDeque<String>,
}

impl Discovery {
    pub fn new(local: SocketAddr, config: DiscoveryConfig) -> Result<Self, Error> {
        if config.servers.len() + config.named_servers.len() > 32
            || config.timeout.is_zero()
            || Instant::now().checked_add(config.timeout).is_none()
        {
            return Err(Error::Configuration("invalid discovery budgets"));
        }
        let mut clients: Vec<Managed> = Vec::new();
        for (url, address) in config
            .servers
            .iter()
            .map(|s| (s.url.clone(), Some(s.address)))
            .chain(config.named_servers.iter().cloned().map(|u| (u, None)))
        {
            if config.relay_only && url.scheme != SchemeType::Turn {
                continue;
            }
            if address.is_some()
                && url.scheme == SchemeType::Turn
                && clients.iter().any(|c| c.turn && c.address == address)
            {
                return Err(Error::Configuration("duplicate TURN endpoint"));
            }
            clients.push(Managed::new(url, address, local, &config)?);
        }
        Ok(Self {
            local,
            clients,
            relay_only: config.relay_only,
            timeout: config.timeout,
            deadline: None,
            started: false,
            finished: false,
            errors: VecDeque::new(),
            config,
        })
    }

    pub fn relay_only(&self) -> bool {
        self.relay_only
    }

    pub fn start(&mut self, local: SocketAddr, now: Instant) -> Result<(), Error> {
        if self.started {
            return Ok(());
        }
        self.started = true;
        self.deadline = now.checked_add(self.timeout);
        let deadline = self.deadline.expect("validated gathering deadline");
        for managed in &mut self.clients {
            managed.begin(local, &self.config, now, deadline)?;
        }
        Ok(())
    }

    /// Recheck STUN mappings while retaining TURN allocations, their pending
    /// transactions, permissions and refresh deadlines on the same five-tuple.
    pub fn restart(&mut self, local: SocketAddr, now: Instant) -> Result<(), Error> {
        let mut next = Self::new(local, self.config.clone())?;
        for (old, new) in self.clients.iter_mut().zip(&mut next.clients) {
            if old.turn && old.client.is_some() {
                std::mem::swap(old, new);
            }
        }
        self.close();
        *self = next;
        self.start(local, now)
    }

    pub fn next_deadline(&mut self) -> Option<Instant> {
        if !self.started {
            return None;
        }
        let protocol = self
            .clients
            .iter_mut()
            .filter_map(|m| m.client.as_mut()?.poll_timeout())
            .min();
        protocol
            .into_iter()
            .chain(
                self.clients
                    .iter()
                    .filter(|m| !m.done)
                    .filter_map(|m| m.lookup.as_ref().map(|q| q.next_deadline())),
            )
            .chain(
                self.clients
                    .iter()
                    .filter(|m| !m.done)
                    .filter_map(|m| m.attempt_deadline),
            )
            .chain(self.deadline)
            .min()
    }

    pub fn timeout(&mut self, now: Instant) -> Result<(), Error> {
        if !self.started {
            return Ok(());
        }
        self.progress(now);
        for index in 0..self.clients.len() {
            let blocked = self.blocked(index);
            let managed = &mut self.clients[index];
            if let Some(client) = &mut managed.client
                && client.poll_timeout().is_some_and(|d| now >= d)
                && let Err(error) = client.handle_timeout(now)
            {
                if managed.done {
                    return Err(error.into());
                }
                managed.next_address(
                    self.local,
                    &self.config,
                    now,
                    self.deadline.unwrap_or(now),
                    &blocked,
                );
            }
        }
        self.collect_errors();
        Ok(())
    }

    fn blocked(&self, except: usize) -> Vec<SocketAddr> {
        self.clients
            .iter()
            .enumerate()
            .filter(|(i, m)| *i != except && m.turn && m.client.is_some())
            .filter_map(|(_, m)| m.address)
            .collect()
    }
    fn collect_errors(&mut self) {
        for managed in &mut self.clients {
            if std::mem::take(&mut managed.failed) {
                self.errors.push_back(managed.url.clone());
            }
        }
    }
    fn progress(&mut self, now: Instant) {
        if let Some(deadline) = self.deadline {
            for index in 0..self.clients.len() {
                let blocked = self.blocked(index);
                let managed = &mut self.clients[index];
                if managed.done {
                    continue;
                }
                if now >= deadline {
                    managed.fail();
                } else {
                    managed.progress(self.local, &self.config, now, deadline, &blocked);
                }
            }
        }
        self.collect_errors();
    }
    pub fn cancel_resolution(&mut self) {
        for managed in &mut self.clients {
            if managed.lookup.take().is_some() {
                managed.done = true;
            }
        }
    }
    pub fn gathering_pending(&self) -> bool {
        self.started && !self.finished
    }
    pub fn send_failed(&mut self, address: SocketAddr, now: Instant) -> bool {
        let Some(index) = self
            .clients
            .iter()
            .position(|m| !m.done && m.address == Some(address))
        else {
            return false;
        };
        let blocked = self.blocked(index);
        self.clients[index].next_address(
            self.local,
            &self.config,
            now,
            self.deadline.unwrap_or(now),
            &blocked,
        );
        self.collect_errors();
        true
    }

    pub fn advance(
        &mut self,
        core: &mut RTCPeerConnection,
        local: SocketAddr,
    ) -> Result<(), Error> {
        if !self.started {
            return Ok(());
        }
        let now = Instant::now();
        self.progress(now);
        for index in 0..self.clients.len() {
            let blocked = self.blocked(index);
            let managed = &mut self.clients[index];
            while let Some(event) = managed.client.as_mut().and_then(Protocol::poll_event) {
                let base = |addr: SocketAddr| CandidateConfig {
                    network: "udp".into(),
                    address: addr.ip().to_string(),
                    port: addr.port(),
                    component: 1,
                    ..Default::default()
                };
                match event {
                    Event::BindingResponse(_, addr) if !managed.done => {
                        let candidate = CandidateServerReflexiveConfig {
                            base_config: base(addr),
                            rel_addr: local.ip().to_string(),
                            rel_port: local.port(),
                            url: Some(managed.url.clone()),
                        }
                        .new_candidate_server_reflexive()?;
                        let mut init = RTCIceCandidate::from(&candidate).to_json()?;
                        // to_json is deliberately signaling-only. Supplying a
                        // local candidate additionally needs its gatherer URL.
                        init.url = Some(managed.url.clone());
                        core.add_local_candidate(init)?;
                        managed.reflexive = Some(addr);
                        managed.done = true;
                    }
                    Event::AllocateResponse(_, addr) if !managed.done => {
                        let candidate = CandidateRelayConfig {
                            base_config: base(addr),
                            // JSEP relay policy conceals the physical address
                            // in candidate strings, SDP and transport getters.
                            rel_addr: if self.relay_only {
                                if local.is_ipv4() { "0.0.0.0" } else { "::" }.into()
                            } else {
                                local.ip().to_string()
                            },
                            rel_port: if self.relay_only { 0 } else { local.port() },
                            url: Some(managed.url.clone()),
                        }
                        .new_candidate_relay()?;
                        let mut init = RTCIceCandidate::from(&candidate).to_json()?;
                        init.url = Some(managed.url.clone());
                        core.add_local_candidate(init)?;
                        managed.relay = Some(addr);
                        managed.done = true;
                    }
                    Event::BindingError(..)
                    | Event::AllocateError(..)
                    | Event::TransactionTimeout(_)
                        if !managed.done =>
                    {
                        managed.next_address(
                            local,
                            &self.config,
                            now,
                            self.deadline.unwrap_or(now),
                            &blocked,
                        );
                        break;
                    }
                    Event::DataIndicationOrChannelData(_, peer_addr, message) => {
                        if let Some(relay) = managed.relay {
                            core.handle_read(TaggedBytesMut {
                                now: Instant::now(),
                                message,
                                transport: TransportContext {
                                    local_addr: relay,
                                    peer_addr,
                                    transport_protocol: TransportProtocol::UDP,
                                    ecn: None,
                                },
                            })?;
                        }
                    }
                    Event::CreatePermissionError(..) | Event::TransactionTimeout(_) => {
                        return Err(Error::RelayUnavailable);
                    }
                    _ => {}
                }
            }
        }
        self.collect_errors();
        // Expose every terminal server error before completing gathering. In
        // particular, a synchronous resolver refusal must not let a consumer
        // harvest completion from the core before it can drain the 701 queue.
        if !self.finished && self.errors.is_empty() && self.clients.iter().all(|m| m.done) {
            self.finished = true;
            self.deadline = None;
            core.add_local_candidate(Default::default())?;
        }
        Ok(())
    }

    pub fn poll_error(&mut self) -> Option<String> {
        self.errors.pop_front()
    }

    pub fn poll_write(&mut self) -> Option<TaggedBytesMut> {
        self.clients
            .iter_mut()
            .find_map(|m| m.client.as_mut()?.poll_write())
    }

    /// Only exact-server, transaction-matched responses or allocation data belong here.
    pub fn read(&mut self, packet: TaggedBytesMut) -> Result<Option<TaggedBytesMut>, Error> {
        let mut response = None;
        let mut indication = ChannelData::is_channel_data(&packet.message);
        if is_stun_message(&packet.message) {
            let mut message = Message::new();
            message.raw = packet.message.to_vec();
            if message.decode().is_ok() {
                match message.typ.class {
                    CLASS_SUCCESS_RESPONSE | CLASS_ERROR_RESPONSE => {
                        response = Some(message.transaction_id)
                    }
                    CLASS_INDICATION => indication = true,
                    _ => {}
                }
            }
        }
        if let Some(managed) = self.clients.iter_mut().find(|m| {
            m.address == Some(packet.transport.peer_addr)
                && m.client.as_ref().is_some_and(|c| {
                    response.as_ref().is_some_and(|tid| c.has_transaction(tid))
                        || (indication && m.relay.is_some())
                })
        }) {
            managed
                .client
                .as_mut()
                .expect("matched live client")
                .handle_read(packet)?;
            Ok(None)
        } else if self.relay_only {
            Ok(None)
        } else {
            Ok(Some(packet))
        }
    }

    /// Return false while one permission is pending. The caller retains this packet.
    pub fn route(&mut self, packet: &TaggedBytesMut, local: SocketAddr) -> Result<bool, Error> {
        if let Some(managed) = self
            .clients
            .iter_mut()
            .find(|m| m.relay == Some(packet.transport.local_addr))
        {
            let client = managed.client.as_mut().ok_or(Error::RelayUnavailable)?;
            let mut relay = client.relay(packet.transport.local_addr)?;
            match relay.send_to(packet.now, &packet.message, packet.transport.peer_addr) {
                Ok(()) => Ok(true),
                Err(rtc::shared::error::Error::ErrNoPermission) => {
                    relay.create_permission(Instant::now(), packet.transport.peer_addr)?;
                    Ok(false)
                }
                Err(error) => Err(error.into()),
            }
        } else if !self.relay_only
            && (packet.transport.local_addr == local
                || self
                    .clients
                    .iter()
                    .any(|m| m.reflexive == Some(packet.transport.local_addr)))
        {
            // Direct writes are sent by the socket owner.
            Ok(false)
        } else {
            Err(Error::Route {
                transport: packet.transport,
            })
        }
    }

    pub fn is_relay(&self, local: SocketAddr) -> bool {
        self.clients.iter().any(|m| m.relay == Some(local))
    }

    pub fn close(&mut self) {
        self.cancel_resolution();
        self.deadline = None;
        self.started = false;
        for managed in &mut self.clients {
            if let Some(client) = &mut managed.client {
                // Discard queued application traffic before the best-effort release.
                while client.poll_write().is_some() {}
                if let Some(addr) = managed.relay
                    && let Ok(mut relay) = client.relay(addr)
                {
                    let _ = relay.close(Instant::now());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
