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

/// An ICE server resolved by the host. Credentials are deliberately not Debug.
pub struct Server {
    /// Parsed URL, including TURN credentials. Its display form omits credentials.
    pub url: Url,
    /// Concrete server endpoint. Resolution is outside this socket driver.
    pub address: SocketAddr,
}

/// Discovery policy supplied by the host, independently of the protocol engine.
pub struct DiscoveryConfig {
    /// At most 32 resolved UDP servers. Duplicate TURN endpoints are refused.
    pub servers: Vec<Server>,
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
    address: SocketAddr,
    relay: Option<SocketAddr>,
    reflexive: Option<SocketAddr>,
    done: bool,
    turn: bool,
}

pub(super) struct Discovery {
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
        if config.servers.len() > 32
            || config.timeout.is_zero()
            || Instant::now().checked_add(config.timeout).is_none()
        {
            return Err(Error::Configuration("invalid discovery budgets"));
        }
        let mut clients: Vec<Managed> = Vec::new();
        for server in config.servers {
            let turn = server.url.scheme == SchemeType::Turn;
            if server.url.proto != ProtoType::Udp
                || !matches!(server.url.scheme, SchemeType::Stun | SchemeType::Turn)
                || server.address.ip().is_unspecified()
                || server.address.port() == 0
                || server.address.is_ipv4() != local.is_ipv4()
            {
                return Err(Error::Configuration("resolved UDP ICE server required"));
            }
            if turn
                && clients
                    .iter()
                    .any(|c| c.turn && c.address == server.address)
            {
                return Err(Error::Configuration("duplicate TURN endpoint"));
            }
            if config.relay_only && !turn {
                continue;
            }
            let client = Client::new(
                ClientConfig {
                    stun_serv_addr: if turn {
                        String::new()
                    } else {
                        server.address.to_string()
                    },
                    turn_serv_addr: if turn {
                        server.address.to_string()
                    } else {
                        String::new()
                    },
                    local_addr: local,
                    transport_protocol: TransportProtocol::UDP,
                    username: server.url.username.clone(),
                    password: server.url.password.clone(),
                    allocation_refresh_interval_cap: config.allocation_refresh_interval_cap,
                    ..Default::default()
                },
                config.crypto_provider.clone(),
            )?;
            clients.push(Managed {
                client: Some(client),
                url: server.url.to_string(),
                address: server.address,
                relay: None,
                reflexive: None,
                done: false,
                turn,
            });
        }
        Ok(Self {
            clients,
            relay_only: config.relay_only,
            timeout: config.timeout,
            deadline: None,
            started: false,
            finished: false,
            errors: VecDeque::new(),
        })
    }

    pub fn relay_only(&self) -> bool {
        self.relay_only
    }

    pub fn start(&mut self, now: Instant) -> Result<(), Error> {
        if self.started {
            return Ok(());
        }
        self.started = true;
        self.deadline = now.checked_add(self.timeout);
        for managed in &mut self.clients {
            if let Some(client) = &mut managed.client {
                if managed.turn {
                    client.allocate(now)?;
                } else {
                    client.send_binding_request(now)?;
                }
            }
        }
        Ok(())
    }

    pub fn next_deadline(&mut self) -> Option<Instant> {
        if !self.started {
            return None;
        }
        self.clients
            .iter_mut()
            .filter_map(|m| m.client.as_mut()?.poll_timeout())
            .chain(self.deadline)
            .min()
    }

    pub fn timeout(&mut self, now: Instant) -> Result<(), Error> {
        if !self.started {
            return Ok(());
        }
        for managed in &mut self.clients {
            if !managed.done && self.deadline.is_some_and(|d| now >= d) {
                managed.done = true;
                managed.client = None;
                self.errors.push_back(managed.url.clone());
            } else if let Some(client) = &mut managed.client
                && client.poll_timeout().is_some_and(|d| now >= d)
            {
                client.handle_timeout(now)?;
            }
        }
        Ok(())
    }

    pub fn advance(
        &mut self,
        core: &mut RTCPeerConnection,
        local: SocketAddr,
    ) -> Result<(), Error> {
        if !self.started {
            return Ok(());
        }
        for managed in &mut self.clients {
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
                        managed.done = true;
                        managed.client = None;
                        self.errors.push_back(managed.url.clone());
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
        if !self.finished && self.clients.iter().all(|m| m.done) {
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
            m.address == packet.transport.peer_addr
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
mod tests {
    use super::*;
    use rtc::stun::message::BINDING_SUCCESS;

    #[test]
    fn shared_server_response_demux_checks_transaction_and_source_but_preserves_ice_requests() {
        let local: SocketAddr = "127.0.0.1:10000".parse().unwrap();
        let server: SocketAddr = "127.0.0.1:3478".parse().unwrap();
        let mut discovery = Discovery::new(
            local,
            DiscoveryConfig {
                servers: vec![Server {
                    url: Url::parse_url("stun:127.0.0.1:3478").unwrap(),
                    address: server,
                }],
                relay_only: false,
                timeout: Duration::from_secs(1),
                allocation_refresh_interval_cap: None,
                crypto_provider: rtc::crypto::default_provider().unwrap(),
            },
        )
        .unwrap();
        discovery.start(Instant::now()).unwrap();
        let request = discovery.poll_write().unwrap();
        let mut message = Message::new();
        message.raw = request.message.to_vec();
        message.decode().unwrap();
        let tid = message.transaction_id;
        let packet = |raw: Vec<u8>, peer_addr| TaggedBytesMut {
            now: Instant::now(),
            message: bytes::BytesMut::from(raw.as_slice()),
            transport: TransportContext {
                local_addr: local,
                peer_addr,
                transport_protocol: TransportProtocol::UDP,
                ecn: None,
            },
        };
        // Even a Binding request with the same ID belongs to ICE, not discovery.
        assert!(
            discovery
                .read(packet(message.raw.clone(), server))
                .unwrap()
                .is_some()
        );
        let mut response = Message::new();
        response
            .build(&[Box::new(tid), Box::new(BINDING_SUCCESS)])
            .unwrap();
        let wrong_source: SocketAddr = "127.0.0.1:3479".parse().unwrap();
        assert!(
            discovery
                .read(packet(response.raw.clone(), wrong_source))
                .unwrap()
                .is_some()
        );
        assert!(
            discovery.clients[0]
                .client
                .as_ref()
                .unwrap()
                .has_transaction(&tid)
        );
        assert!(
            discovery
                .read(packet(response.raw, server))
                .unwrap()
                .is_none()
        );
        assert!(
            !discovery.clients[0]
                .client
                .as_ref()
                .unwrap()
                .has_transaction(&tid)
        );
    }
}
