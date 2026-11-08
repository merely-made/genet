//! Checked multicast destination/interface through the maintained pktinfo wrapper.
use super::*;
use socket_pktinfo::PktInfoUdpSocket;

pub(super) struct Receiver {
    socket: PktInfoUdpSocket,
    interface: u64,
}
impl Receiver {
    pub fn new(interface: Ipv4Addr) -> io::Result<Self> {
        let index = link::index(interface).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "mDNS interface metadata unavailable",
            )
        })?;
        let socket = PktInfoUdpSocket::new(Domain::IPV4)?;
        socket.set_reuse_address(true)?;
        socket.bind(&"0.0.0.0:5353".parse::<SocketAddr>().unwrap().into())?;
        socket.set_multicast_loop_v4(true)?;
        socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &interface)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            interface: u64::from(index),
        })
    }
    pub fn recv(&self, buffer: &mut [u8]) -> io::Result<Option<(usize, SocketAddr)>> {
        let (n, info) = self.socket.recv(buffer)?;
        Ok((n <= buffer.len()
            && info.if_index == self.interface
            && info.addr_dst == IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)))
        .then_some((n, info.addr_src)))
    }
}
