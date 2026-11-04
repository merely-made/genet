//! A caller-owned UDP socket and peer connection, with no spawned tasks.
//!
//! [`UdpPeer::poll`] processes a bounded nonblocking host turn. The optional
//! `udp` feature adds async `next`/`next_with_interest` on the same driver.
//! Dropping an async wait is cancellation-safe: an unsent datagram stays
//! pending. Dropping the peer releases its socket.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use bytes::BytesMut;
use rtc::peer_connection::RTCPeerConnection;
use rtc::peer_connection::event::RTCPeerConnectionEvent;
use rtc::peer_connection::message::TaggedRTCMessage;
use rtc::peer_connection::state::RTCPeerConnectionState;
use rtc::peer_connection::transport::{
    CandidateConfig, CandidateHostConfig, RTCDtlsTransportState, RTCIceCandidate,
};
use rtc::sansio::Protocol;
use rtc::shared::{TaggedBytesMut, TransportContext, TransportProtocol};
#[cfg(feature = "udp")]
use tokio::net::UdpSocket;

mod discovery;
use discovery::Discovery;
pub use discovery::{DiscoveryConfig, Server};

mod socket;
use socket::Socket;

/// Host-selected I/O budgets. These do not change data-channel message limits.
#[derive(Clone, Copy, Debug)]
pub struct IoLimits {
    /// Largest accepted UDP datagram, between 1 and 65,535 bytes.
    pub datagram_bytes: usize,
    /// Maximum outgoing datagrams processed before yielding to the host.
    pub packets_per_turn: usize,
    /// Floor for repeated protocol deadlines that are already due.
    pub timer_floor: Duration,
}

/// A protocol, socket or explicit host-configuration failure.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The host supplied an invalid budget or a wildcard bind address.
    #[error("invalid RTC UDP host configuration: {0}")]
    Configuration(&'static str),
    /// A datagram exceeded the explicit host budget; it was not truncated.
    #[error("RTC datagram has {actual} bytes, exceeding the {limit}-byte host limit")]
    DatagramLimit {
        /// Observed size, possibly only a lower bound for an incoming datagram.
        actual: usize,
        /// Configured ceiling.
        limit: usize,
    },
    /// A packet requested an unsupported transport or local address.
    #[error("RTC packet does not belong to this UDP socket")]
    Route {
        /// The requested route, without packet contents or credentials.
        transport: TransportContext,
    },
    /// A TURN permission or established relay transaction failed.
    #[error("RTC relay transaction failed")]
    RelayUnavailable,
    /// The underlying socket operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The protocol implementation rejected an operation or input.
    #[error(transparent)]
    Protocol(#[from] rtc::shared::error::Error),
}

/// One observation from a caller-driven host turn.
pub enum Output {
    /// A configured server failed gathering. The URL excludes TURN credentials.
    GatheringError {
        /// Server whose attempt failed.
        url: String,
        /// WebRTC host gathering error (701: server could not be reached/used).
        code: u16,
    },
    /// A peer-connection state or channel event, preserving upstream order.
    Event(RTCPeerConnectionEvent),
    /// An inbound media or data-channel message, preserving message boundaries.
    Message(TaggedRTCMessage),
    /// I/O or a protocol timer advanced; let the host dispatch its other work.
    Progress,
    /// The caller's deadline elapsed. It does not implicitly close the peer.
    Deadline,
    /// Local close or remote DTLS closure has ended this adapter's route.
    /// This does not synthesize a peer-connection state event.
    Closed,
}

/// Which application queues the host has capacity to receive from this turn.
#[derive(Clone, Copy, Debug)]
pub enum ReadInterest {
    /// Deliver media and data-channel messages in upstream arrival order.
    All,
    /// Keep data-channel messages under SCTP backpressure while delivering media.
    MediaOnly,
    /// Service protocol events, I/O and timers without draining either queue.
    EventsOnly,
}

/// A peer and one concrete local UDP route, owned entirely by the caller.
pub struct UdpPeer {
    core: RTCPeerConnection,
    socket: Socket,
    local_addr: SocketAddr,
    limits: IoLimits,
    buffer: Vec<u8>,
    pending: Option<TaggedBytesMut>,
    pending_socket: Option<TaggedBytesMut>,
    discovery: Option<Discovery>,
    closed: bool,
    released: bool,
    last_timeout: Instant,
}

impl UdpPeer {
    /// Bind a concrete local address and register its host candidate.
    ///
    /// Port zero requests an ephemeral port. Wildcard addresses are refused:
    /// this adapter cannot invent the destination interface of received UDP.
    /// This creates no thread/task and performs no STUN, TURN or DNS discovery.
    /// Call [`Self::finish_host_candidates`] when host-only gathering is done.
    ///
    /// # Errors
    /// Returns an error for invalid budgets, a failed bind or invalid candidates.
    #[cfg(feature = "udp")]
    pub async fn bind(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = Socket::Async(UdpSocket::bind(bind).await?);
        Self::from_socket(core, socket, limits, None)
    }

    /// Bind a nonblocking host socket without a Tokio runtime or background task.
    /// Drive it through [`Self::poll`] and schedule its [`Self::next_deadline`].
    ///
    /// # Errors
    /// Returns an error for invalid budgets, a failed bind or invalid candidates.
    pub fn bind_host(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = std::net::UdpSocket::bind(bind)?;
        socket.set_nonblocking(true)?;
        Self::from_socket(core, Socket::Host(socket), limits, None)
    }

    /// Bind with host-resolved UDP discovery. Gathering starts only when
    /// [`Self::finish_host_candidates`] is called. This performs no DNS lookup.
    ///
    /// # Errors
    /// Returns an error for invalid discovery/budgets, socket bind or candidates.
    pub fn bind_host_with_discovery(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
        discovery: DiscoveryConfig,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = std::net::UdpSocket::bind(bind)?;
        socket.set_nonblocking(true)?;
        Self::from_socket(core, Socket::Host(socket), limits, Some(discovery))
    }

    /// Async socket counterpart of [`Self::bind_host_with_discovery`].
    ///
    /// # Errors
    /// Returns an error for invalid discovery/budgets, socket bind or candidates.
    #[cfg(feature = "udp")]
    pub async fn bind_with_discovery(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
        discovery: DiscoveryConfig,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = Socket::Async(UdpSocket::bind(bind).await?);
        Self::from_socket(core, socket, limits, Some(discovery))
    }

    fn validate(bind: SocketAddr, limits: IoLimits) -> Result<(), Error> {
        if bind.ip().is_unspecified() {
            return Err(Error::Configuration(
                "a concrete local IP address is required",
            ));
        }
        if !(1..=65_535).contains(&limits.datagram_bytes)
            || limits.packets_per_turn == 0
            || limits.timer_floor.is_zero()
        {
            return Err(Error::Configuration(
                "nonzero packet, datagram and timer budgets are required",
            ));
        }
        Ok(())
    }

    fn from_socket(
        mut core: RTCPeerConnection,
        socket: Socket,
        limits: IoLimits,
        discovery: Option<DiscoveryConfig>,
    ) -> Result<Self, Error> {
        let local_addr = socket.local_addr()?;
        let discovery = discovery
            .map(|c| Discovery::new(local_addr, c))
            .transpose()?;
        if !discovery.as_ref().is_some_and(Discovery::relay_only) {
            let candidate = CandidateHostConfig {
                base_config: CandidateConfig {
                    network: "udp".into(),
                    address: local_addr.ip().to_string(),
                    port: local_addr.port(),
                    component: 1,
                    ..Default::default()
                },
                ..Default::default()
            }
            .new_candidate_host()?;
            core.add_local_candidate(RTCIceCandidate::from(&candidate).to_json()?)?;
        }
        Ok(Self {
            core,
            socket,
            local_addr,
            limits,
            // Receive a full UDP datagram even under a small acceptance
            // budget. Windows reports WSAEMSGSIZE instead of a byte count
            // for a short receive buffer, so a limit-sized buffer cannot
            // portably distinguish an oversize packet from other I/O errors.
            buffer: vec![0; 65_536],
            pending: None,
            pending_socket: None,
            discovery,
            closed: false,
            released: false,
            last_timeout: Instant::now(),
        })
    }

    /// The bound route, including its actual ephemeral port.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Observe the protocol state without granting it an I/O runtime.
    pub fn core(&self) -> &RTCPeerConnection {
        &self.core
    }

    /// Apply protocol commands, including offers, answers and channel writes.
    pub fn core_mut(&mut self) -> &mut RTCPeerConnection {
        &mut self.core
    }

    /// Finish supplying host candidates and start configured server discovery.
    /// With no servers, gathering completes immediately. Otherwise completion
    /// waits for every server's result or the configured gathering deadline.
    ///
    /// # Errors
    /// Returns a discovery-client or candidate-registration error.
    pub fn finish_host_candidates(&mut self) -> Result<(), Error> {
        if let Some(discovery) = &mut self.discovery {
            discovery.start(self.local_addr, Instant::now())?;
            discovery.advance(&mut self.core, self.local_addr)?;
        } else {
            self.core.add_local_candidate(Default::default())?;
        }
        Ok(())
    }

    /// Finish gathering for a newly applied ICE generation on the same socket.
    ///
    /// Call after applying a restart description in the core. Existing local
    /// candidates and TURN allocations remain usable; STUN gets fresh bounded
    /// transactions. This does not replace servers, rebind interfaces or restart
    /// DTLS/SCTP. The caller publishes retained candidates for the new generation.
    ///
    /// # Errors
    /// Returns a configuration error after close, or a discovery/protocol error.
    pub fn restart_gathering(&mut self) -> Result<(), Error> {
        if self.closed {
            return Err(Error::Configuration("cannot gather on a closed peer"));
        }
        if let Some(discovery) = &mut self.discovery {
            discovery.restart(self.local_addr, Instant::now())?;
            discovery.advance(&mut self.core, self.local_addr)?;
        } else {
            self.core.add_local_candidate(Default::default())?;
        }
        Ok(())
    }

    /// Start graceful protocol close; continue polling to flush close packets.
    /// Dropping the peer instead immediately releases the socket.
    ///
    /// # Errors
    /// Returns the protocol engine's close error.
    pub fn close(&mut self) -> Result<(), Error> {
        self.core.close()?;
        self.closed = true;
        if let Some(discovery) = &mut self.discovery {
            discovery.cancel_resolution();
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<bool, Error> {
        let mut sent = false;
        for _ in 0..self.limits.packets_per_turn {
            if self.pending_socket.is_none() {
                self.pending_socket = self.discovery.as_mut().and_then(Discovery::poll_write);
            }
            if self.pending_socket.is_none() {
                if self.pending.is_none() {
                    self.pending = self.core.poll_write();
                }
                let Some(packet) = self.pending.as_ref() else {
                    if self.closed {
                        self.release_allocations();
                    }
                    return Ok(sent);
                };
                if packet.transport.transport_protocol != TransportProtocol::UDP {
                    return Err(Error::Route {
                        transport: packet.transport,
                    });
                }
                if packet.message.len() > self.limits.datagram_bytes {
                    return Err(Error::DatagramLimit {
                        actual: packet.message.len(),
                        limit: self.limits.datagram_bytes,
                    });
                }
                if let Some(discovery) = &mut self.discovery {
                    let relay = discovery.is_relay(packet.transport.local_addr);
                    let routed = discovery.route(packet, self.local_addr)?;
                    if relay {
                        if !routed {
                            return Ok(sent);
                        }
                        self.pending = None;
                        sent = true;
                        continue;
                    }
                } else if packet.transport.local_addr != self.local_addr {
                    return Err(Error::Route {
                        transport: packet.transport,
                    });
                }
                self.pending_socket = self.pending.take();
            }
            let packet = self.pending_socket.as_ref().expect("pending socket packet");
            if packet.message.len() > self.limits.datagram_bytes {
                return Err(Error::DatagramLimit {
                    actual: packet.message.len(),
                    limit: self.limits.datagram_bytes,
                });
            }
            match self
                .socket
                .send(&packet.message, packet.transport.peer_addr)
            {
                Ok(size) if size == packet.message.len() => {
                    self.pending_socket = None;
                    sent = true;
                }
                Ok(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "short UDP send",
                    )
                    .into());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(sent),
                Err(error) => {
                    if self
                        .discovery
                        .as_mut()
                        .is_some_and(|d| d.send_failed(packet.transport.peer_addr, Instant::now()))
                    {
                        self.pending_socket = None;
                        sent = true;
                        continue;
                    }
                    return Err(error.into());
                }
            }
        }
        Ok(sent)
    }

    /// Next protocol wakeup, including the host's repeated-timeout floor.
    pub fn next_deadline(&mut self) -> Option<Instant> {
        if self.closed {
            return None;
        }
        let floor = self.last_timeout.checked_add(self.limits.timer_floor);
        self.core
            .poll_timeout()
            .into_iter()
            .chain(self.discovery.as_mut().and_then(Discovery::next_deadline))
            .min()
            .map(|time| floor.map_or(time, |floor| time.max(floor)))
    }

    /// Process one bounded, nonblocking host turn. `None` means wait for socket
    /// readiness or [`Self::next_deadline`]. Both native socket adapters use
    /// this same packet, timer, event, message and backpressure machinery.
    ///
    /// # Errors
    /// Returns socket, route, datagram-budget, discovery or protocol errors.
    pub fn poll(
        &mut self,
        deadline: Instant,
        interest: ReadInterest,
    ) -> Result<Option<Output>, Error> {
        let now = Instant::now();
        if now >= deadline {
            return Ok(Some(Output::Deadline));
        }
        let mut progress = self.flush()?;
        // Service timers even while application events/messages are queued.
        if self.next_deadline().is_some_and(|time| time <= now) {
            self.core.handle_timeout(now)?;
            if let Some(discovery) = &mut self.discovery {
                discovery.timeout(now)?;
            }
            self.last_timeout = now;
            progress = true;
        }
        if let Some(discovery) = &mut self.discovery {
            discovery.advance(&mut self.core, self.local_addr)?;
            if let Some(url) = discovery.poll_error() {
                return Ok(Some(Output::GatheringError { url, code: 701 }));
            }
        }
        if let Some(event) = self.core.poll_event() {
            if matches!(
                &event,
                RTCPeerConnectionEvent::OnConnectionStateChangeEvent(
                    RTCPeerConnectionState::Closed
                )
            ) {
                self.closed = true;
            }
            return Ok(Some(Output::Event(event)));
        }
        let message = match interest {
            ReadInterest::All => self.core.poll_read(),
            ReadInterest::MediaOnly => self.core.poll_media_read(),
            ReadInterest::EventsOnly => None,
        };
        if let Some(message) = message {
            return Ok(Some(Output::Message(message)));
        }
        let remote_closed = self
            .core
            .sctp()
            .is_some_and(|sctp| sctp.transport().state() == RTCDtlsTransportState::Closed);
        if (self.closed || remote_closed) && self.pending.is_none() {
            self.release_allocations();
            self.closed = true;
            return Ok(Some(Output::Closed));
        }
        match self.socket.receive(&mut self.buffer) {
            Ok((size, peer_addr)) => {
                if size > self.limits.datagram_bytes {
                    return Err(Error::DatagramLimit {
                        actual: size,
                        limit: self.limits.datagram_bytes,
                    });
                }
                let packet = TaggedBytesMut {
                    now: Instant::now(),
                    transport: TransportContext {
                        local_addr: self.local_addr,
                        peer_addr,
                        transport_protocol: TransportProtocol::UDP,
                        ecn: None,
                    },
                    message: BytesMut::from(&self.buffer[..size]),
                };
                let packet = if let Some(discovery) = &mut self.discovery {
                    discovery.read(packet)?
                } else {
                    Some(packet)
                };
                if let Some(packet) = packet {
                    self.core.handle_read(packet)?;
                }
                progress = true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionRefused
                ) && self
                    .discovery
                    .as_ref()
                    .is_some_and(Discovery::gathering_pending) =>
            {
                // Unconnected UDP supplies no source address for an ICMP
                // refusal. Let per-address deadlines advance the correct URL.
                progress = true;
            }
            Err(error) => return Err(error.into()),
        }
        Ok(progress.then_some(Output::Progress))
    }

    /// Deliver an observation or await native I/O until the host's deadline.
    ///
    /// # Errors
    /// Returns the errors from polling or waiting for the socket.
    #[cfg(feature = "udp")]
    pub async fn next(&mut self, deadline: Instant) -> Result<Output, Error> {
        self.next_with_interest(deadline, ReadInterest::All).await
    }

    /// Await I/O while honoring the host's current receive capacity. A full
    /// data consumer can keep servicing protocol timers, I/O and media.
    ///
    /// # Errors
    /// Returns the errors from polling or waiting for the socket.
    #[cfg(feature = "udp")]
    pub async fn next_with_interest(
        &mut self,
        deadline: Instant,
        interest: ReadInterest,
    ) -> Result<Output, Error> {
        loop {
            if let Some(output) = self.poll(deadline, interest)? {
                if matches!(&output, Output::Progress) {
                    tokio::task::yield_now().await;
                }
                return Ok(output);
            }
            let wake = self.next_deadline().unwrap_or(deadline).min(deadline);
            self.socket
                .wait(wake, self.pending_socket.is_some(), self.limits.timer_floor)
                .await?;
        }
    }

    fn release_allocations(&mut self) {
        if self.released {
            return;
        }
        self.released = true;
        if let Some(discovery) = &mut self.discovery {
            discovery.close();
            // One release per bounded server list, best effort on a nonblocking
            // socket. Dropping never waits for a server or spawns cleanup work.
            while let Some(packet) = discovery.poll_write() {
                let _ = self
                    .socket
                    .send(&packet.message, packet.transport.peer_addr);
            }
        }
    }
}

impl Drop for UdpPeer {
    fn drop(&mut self) {
        self.release_allocations();
    }
}
