use super::*;
use hickory_proto::op::{Message, OpCode};
use hickory_proto::rr::{Name, RData, Record};
use std::net::{Ipv4Addr, UdpSocket};

pub(super) fn server() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    socket
}
pub(super) fn resolver(servers: Vec<SocketAddr>, limits: DnsLimits) -> DnsResolver {
    DnsResolver::new(DnsConfig { servers, limits }).unwrap()
}
pub(super) fn receive(socket: &UdpSocket) -> Option<(Message, SocketAddr)> {
    let mut bytes = [0; 4096];
    match socket.recv_from(&mut bytes) {
        Ok((n, address)) => Some((Message::from_vec(&bytes[..n]).unwrap(), address)),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
        Err(e) => panic!("DNS fixture: {e}"),
    }
}
pub(super) fn response(request: &Message) -> Message {
    let mut message = Message::response(request.metadata.id, OpCode::Query);
    message.queries = request.queries.clone();
    message
}
pub(super) fn a(name: Name, ip: Ipv4Addr) -> Record {
    Record::from_rdata(name, 60, RData::A(ip.into()))
}
pub(super) fn wait(mut step: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !step() {
        assert!(Instant::now() < deadline, "bounded fixture deadline");
        std::thread::sleep(Duration::from_millis(1));
    }
}
