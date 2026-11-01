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

enum Socket {
    Host(std::net::UdpSocket),
    #[cfg(feature = "udp")]
    Async(UdpSocket),
}

impl Socket {
    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        match self {
            Self::Host(socket) => socket.local_addr(),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.local_addr(),
        }
    }
    fn send(&self, bytes: &[u8], peer: SocketAddr) -> std::io::Result<usize> {
        match self {
            Self::Host(socket) => socket.send_to(bytes, peer),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.try_send_to(bytes, peer),
        }
    }
    fn receive(&self, bytes: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        match self {
            Self::Host(socket) => socket.recv_from(bytes),
            #[cfg(feature = "udp")]
            Self::Async(socket) => socket.try_recv_from(bytes),
        }
    }
    #[cfg(feature = "udp")]
    async fn wait(&self, wake: Instant, pending: bool, floor: Duration) -> std::io::Result<()> {
        match self {
            Self::Async(socket) => tokio::select! {
                ready = socket.readable() => ready,
                ready = socket.writable(), if pending => ready,
                _ = tokio::time::sleep_until(wake.into()) => Ok(()),
            },
            Self::Host(_) => {
                let wake = wake.min(Instant::now().checked_add(floor).unwrap_or(wake));
                tokio::time::sleep_until(wake.into()).await;
                Ok(())
            }
        }
    }
}

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
    Route,
    /// The underlying socket operation failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The protocol implementation rejected an operation or input.
    #[error(transparent)]
    Protocol(#[from] rtc::shared::error::Error),
}

/// One observation from a caller-driven host turn.
pub enum Output {
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
    closed: bool,
    last_timeout: Instant,
}

impl UdpPeer {
    /// Bind a concrete local address and register its host candidate.
    ///
    /// Port zero requests an ephemeral port. Wildcard addresses are refused:
    /// this adapter cannot invent the destination interface of received UDP.
    /// This creates no thread/task and performs no STUN, TURN or DNS discovery.
    /// Call [`Self::finish_host_candidates`] when host-only gathering is done.
    #[cfg(feature = "udp")]
    pub async fn bind(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = Socket::Async(UdpSocket::bind(bind).await?);
        Self::from_socket(core, socket, limits)
    }

    /// Bind a nonblocking host socket without a Tokio runtime or background task.
    /// Drive it through [`Self::poll`] and schedule its [`Self::next_deadline`].
    pub fn bind_host(
        core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
    ) -> Result<Self, Error> {
        Self::validate(bind, limits)?;
        let socket = std::net::UdpSocket::bind(bind)?;
        socket.set_nonblocking(true)?;
        Self::from_socket(core, Socket::Host(socket), limits)
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
    ) -> Result<Self, Error> {
        let local_addr = socket.local_addr()?;
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
            closed: false,
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

    /// Mark host-only candidate gathering complete, emitting its real event.
    pub fn finish_host_candidates(&mut self) -> Result<(), Error> {
        self.core.add_local_candidate(Default::default())?;
        Ok(())
    }

    /// Start graceful protocol close; continue polling to flush close packets.
    /// Dropping the peer instead immediately releases the socket.
    pub fn close(&mut self) -> Result<(), Error> {
        self.core.close()?;
        self.closed = true;
        Ok(())
    }

    fn flush(&mut self) -> Result<bool, Error> {
        let mut sent = false;
        for _ in 0..self.limits.packets_per_turn {
            if self.pending.is_none() {
                self.pending = self.core.poll_write();
            }
            let Some(packet) = self.pending.as_ref() else {
                return Ok(sent);
            };
            if packet.transport.transport_protocol != TransportProtocol::UDP
                || packet.transport.local_addr != self.local_addr
            {
                return Err(Error::Route);
            }
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
                    self.pending = None;
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
                Err(error) => return Err(error.into()),
            }
        }
        Ok(sent)
    }

    /// Next protocol wakeup, including the host's repeated-timeout floor.
    pub fn next_deadline(&mut self) -> Option<Instant> {
        let floor = self.last_timeout.checked_add(self.limits.timer_floor);
        self.core
            .poll_timeout()
            .map(|time| floor.map_or(time, |floor| time.max(floor)))
    }

    /// Process one bounded, nonblocking host turn. `None` means wait for socket
    /// readiness or [`Self::next_deadline`]. Both native socket adapters use
    /// this same packet, timer, event, message and backpressure machinery.
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
            self.last_timeout = now;
            progress = true;
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
                self.core.handle_read(TaggedBytesMut {
                    now: Instant::now(),
                    transport: TransportContext {
                        local_addr: self.local_addr,
                        peer_addr,
                        transport_protocol: TransportProtocol::UDP,
                        ecn: None,
                    },
                    message: BytesMut::from(&self.buffer[..size]),
                })?;
                progress = true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
        Ok(progress.then_some(Output::Progress))
    }

    /// Deliver an observation or await native I/O until the host's deadline.
    #[cfg(feature = "udp")]
    pub async fn next(&mut self, deadline: Instant) -> Result<Output, Error> {
        self.next_with_interest(deadline, ReadInterest::All).await
    }

    /// Await I/O while honoring the host's current receive capacity. A full
    /// data consumer can keep servicing protocol timers, I/O and media.
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
                .wait(wake, self.pending.is_some(), self.limits.timer_floor)
                .await?;
        }
    }
}
