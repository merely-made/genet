//! Owned, nonblocking IPv4 mDNS registration. No worker or shared cache.
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rtc::mdns::responder::{RegistrationState, Responder, ResponderConfig};
use socket_pktinfo::PktInfoUdpSocket;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

/// Generate one unpredictable UUID-v4 candidate name, without opening sockets.
/// # Errors
/// Returns an I/O error when system entropy is unavailable.
pub fn candidate_name() -> io::Result<String> {
    let mut b = [0; 16];
    getrandom::fill(&mut b).map_err(io::Error::other)?;
    b[6] = (b[6] & 15) | 64;
    b[8] = (b[8] & 63) | 128;
    let h: String = b.iter().map(|v| format!("{v:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}.local",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    ))
}

/// One owned advertisement. Dropping withdraws once and releases both sockets.
/// The caller bounds concurrent registrations (for example with its peer quota).
/// Incoming queries use multicast; direct-unicast querying is not supported.
pub struct Registration {
    receiver: PktInfoUdpSocket,
    sender: UdpSocket,
    responder: Responder,
    interface: Ipv4Addr,
    index: u64,
    prefix: u8,
}
impl Registration {
    /// Bind a registration to one selected interface and on-link prefix.
    /// # Errors
    /// Returns a sanitized validation or socket error; never falls back to DNS.
    pub fn new(name: &str, interface: Ipv4Addr, prefix: u8, now: Instant) -> io::Result<Self> {
        if !(1..=32).contains(&prefix) {
            return Err(io::Error::other("Invalid mDNS prefix"));
        }
        let responder = Responder::new(now, name, interface, ResponderConfig::default())
            .map_err(|_| io::Error::other("Invalid mDNS registration"))?;
        let index = NetworkInterface::show()
            .map_err(io::Error::other)?
            .into_iter()
            .find(|i| i.addr.iter().any(|a| a.ip() == IpAddr::V4(interface)))
            .ok_or_else(|| io::Error::other("Missing mDNS interface"))?
            .index as u64;
        let receiver = PktInfoUdpSocket::new(Domain::IPV4)?;
        receiver.set_reuse_address(true)?;
        receiver.bind(&SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5353).into())?;
        receiver.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &interface)?;
        receiver.set_nonblocking(true)?;
        let sender = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        sender.set_reuse_address(true)?;
        sender.bind(&SocketAddr::new(interface.into(), 5353).into())?;
        sender.set_multicast_if_v4(&interface)?;
        sender.set_multicast_ttl_v4(255)?;
        sender.set_ttl_v4(255)?;
        sender.set_multicast_loop_v4(true)?;
        sender.set_nonblocking(true)?;
        Ok(Self {
            receiver,
            sender: sender.into(),
            responder,
            interface,
            index,
            prefix,
        })
    }
    /// Make bounded progress. The caller supplies its clock and input budget.
    /// # Errors
    /// Returns socket or entropy failure. The caller must drop the registration.
    pub fn poll(&mut self, now: Instant, budget: usize) -> io::Result<()> {
        let mut buffer = [0; 1501];
        for _ in 0..budget.min(8) {
            match self.receiver.recv(&mut buffer) {
                Ok((n, info)) => {
                    let SocketAddr::V4(source) = info.addr_src else {
                        continue;
                    };
                    let mask = u32::MAX << (32 - self.prefix);
                    if n > 1500
                        || info.if_index != self.index
                        || info.addr_dst != IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251))
                        || u32::from(*source.ip()) & mask != u32::from(self.interface) & mask
                    {
                        continue;
                    }
                    let mut random = [0];
                    // Rejection sampling gives uniform 20..=120 ms jitter.
                    for _ in 0..8 {
                        getrandom::fill(&mut random).map_err(io::Error::other)?;
                        if random[0] < 202 {
                            break;
                        }
                    }
                    if random[0] >= 202 {
                        return Err(io::Error::other("mDNS jitter entropy budget"));
                    }
                    self.responder.handle_packet(
                        now,
                        source,
                        Ipv4Addr::new(224, 0, 0, 251),
                        &buffer[..n],
                        Duration::from_millis(20 + u64::from(random[0] % 101)),
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        if let Some(tx) = self.responder.poll_transmit(now) {
            match self.sender.send_to(&tx.payload, tx.destination) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    /// Whether the unique name has been withdrawn after a conflict.
    pub fn conflicted(&self) -> bool {
        self.responder.state() == RegistrationState::Conflict
    }
}
impl Drop for Registration {
    fn drop(&mut self) {
        self.responder.close();
        if let Some(tx) = self.responder.poll_transmit(Instant::now()) {
            let _ = self.sender.send_to(&tx.payload, tx.destination);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_names_are_distinct_uuid_v4_names() {
        let a = candidate_name().unwrap();
        let b = candidate_name().unwrap();
        assert_ne!(a, b);
        assert!(crate::resolve::is_mdns_candidate_name(&a));
        assert!(crate::resolve::is_mdns_candidate_name(&b));
    }
    #[test]
    fn invalid_registration_never_binds() {
        assert!(
            Registration::new("invalid.local", Ipv4Addr::LOCALHOST, 8, Instant::now()).is_err()
        );
        assert!(
            Registration::new(
                &candidate_name().unwrap(),
                Ipv4Addr::UNSPECIFIED,
                0,
                Instant::now()
            )
            .is_err()
        );
    }
}
