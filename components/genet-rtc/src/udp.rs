//! A caller-owned UDP socket and peer connection, with no spawned tasks.
//!
//! [`UdpPeer::next`] returns protocol events and messages to the caller and
//! processes one bounded I/O turn. Dropping its future is cancellation-safe:
//! an unsent datagram stays pending. Dropping the peer releases its socket.
//! The host may use `select!` to cancel or dispatch its own commands.

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
use tokio::net::UdpSocket;

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
    socket: UdpSocket,
    local_addr: SocketAddr,
    limits: IoLimits,
    buffer: Vec<u8>,
    pending: Option<TaggedBytesMut>,
    closed: bool,
}

impl UdpPeer {
    /// Bind a concrete local address and register its host candidate.
    ///
    /// Port zero requests an ephemeral port. Wildcard addresses are refused:
    /// this adapter cannot invent the destination interface of received UDP.
    /// This creates no thread/task and performs no STUN, TURN or DNS discovery.
    /// Call [`Self::finish_host_candidates`] when host-only gathering is done.
    pub async fn bind(
        mut core: RTCPeerConnection,
        bind: SocketAddr,
        limits: IoLimits,
    ) -> Result<Self, Error> {
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
        let socket = UdpSocket::bind(bind).await?;
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

    async fn flush(&mut self) -> Result<bool, Error> {
        for _ in 0..self.limits.packets_per_turn {
            if self.pending.is_none() {
                self.pending = self.core.poll_write();
            }
            let Some(packet) = self.pending.as_ref() else {
                return Ok(false);
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
            self.socket
                .send_to(&packet.message, packet.transport.peer_addr)
                .await?;
            self.pending = None;
        }
        self.pending = self.core.poll_write();
        Ok(self.pending.is_some())
    }

    /// Deliver one queued observation or advance I/O until the host deadline.
    ///
    /// The deadline bounds each call, including a busy output queue. Events
    /// and messages stay inside the protocol until the host asks for them,
    /// preserving SCTP's receive-side backpressure. This method does not add
    /// application framing, restrict channel modes or consume protocol events
    /// on the host's behalf.
    pub async fn next(&mut self, deadline: Instant) -> Result<Output, Error> {
        self.next_with_interest(deadline, ReadInterest::All).await
    }

    /// Advance the peer while honoring the host's current receive capacity.
    ///
    /// A full data consumer can select [`ReadInterest::MediaOnly`] and keep
    /// driving ICE, DTLS, SCTP timers and media. It need not stop servicing the
    /// connection or copy reliable messages into another unbounded queue.
    pub async fn next_with_interest(
        &mut self,
        deadline: Instant,
        interest: ReadInterest,
    ) -> Result<Output, Error> {
        if Instant::now() >= deadline {
            return Ok(Output::Deadline);
        }
        let more = tokio::time::timeout_at(deadline.into(), self.flush()).await;
        let more = match more {
            Ok(result) => result?,
            Err(_) => return Ok(Output::Deadline),
        };
        if let Some(event) = self.core.poll_event() {
            if matches!(
                &event,
                RTCPeerConnectionEvent::OnConnectionStateChangeEvent(
                    RTCPeerConnectionState::Closed
                )
            ) {
                self.closed = true;
            }
            return Ok(Output::Event(event));
        }
        let message = match interest {
            ReadInterest::All => self.core.poll_read(),
            ReadInterest::MediaOnly => self.core.poll_media_read(),
            ReadInterest::EventsOnly => None,
        };
        if let Some(message) = message {
            return Ok(Output::Message(message));
        }
        if more {
            tokio::task::yield_now().await;
            return Ok(Output::Progress);
        }
        let remote_closed = self
            .core
            .sctp()
            .is_some_and(|sctp| sctp.transport().state() == RTCDtlsTransportState::Closed);
        if self.closed || remote_closed {
            self.closed = true;
            return Ok(Output::Closed);
        }
        let now = Instant::now();
        let protocol_deadline = self.core.poll_timeout();
        let wake = protocol_deadline
            .map(|time| time.max(now.checked_add(self.limits.timer_floor).unwrap_or(deadline)))
            .unwrap_or(deadline)
            .min(deadline);
        tokio::select! {
            received = self.socket.recv_from(&mut self.buffer) => {
                let (size, peer_addr) = received?;
                if size > self.limits.datagram_bytes {
                    return Err(Error::DatagramLimit { actual: size, limit: self.limits.datagram_bytes });
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
            }
            _ = tokio::time::sleep_until(wake.into()) => {
                let now = Instant::now();
                if now >= deadline {
                    return Ok(Output::Deadline);
                }
                self.core.handle_timeout(now)?;
            }
        }
        Ok(Output::Progress)
    }
}
