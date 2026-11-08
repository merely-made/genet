use hickory_proto::op::{Message, OpCode};
use hickory_proto::rr::{RData, Record, RecordType};
use genet_rtc::resolve::{DnsConfig, DnsLimits, DnsResolver, MdnsConfig};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

static SERIAL: AtomicUsize = AtomicUsize::new(1);
pub(super) struct Fixture {
    socket: UdpSocket,
    names: BTreeMap<String, Ipv4Addr>,
    pub requests: Vec<SocketAddr>,
    pub hold: bool,
}
impl Fixture {
    pub fn new() -> Self {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket
            .bind(&"0.0.0.0:5353".parse::<SocketAddr>().unwrap().into())
            .unwrap();
        socket
            .join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &Ipv4Addr::LOCALHOST)
            .unwrap();
        socket.set_nonblocking(true).unwrap();
        Self {
            socket: socket.into(),
            names: BTreeMap::new(),
            requests: Vec::new(),
            hold: false,
        }
    }
    pub fn name(&mut self) -> String {
        let name = format!(
            "12345678-1234-4234-8234-{:012x}.local",
            SERIAL.fetch_add(1, Ordering::Relaxed)
        );
        self.names.insert(format!("{name}."), Ipv4Addr::LOCALHOST);
        name
    }
    pub fn resolver(&self) -> Arc<DnsResolver> {
        Arc::new(
            DnsResolver::new(DnsConfig {
                servers: Vec::new(),
                limits: DnsLimits::default(),
            })
            .unwrap()
            .with_mdns(MdnsConfig::new(Ipv4Addr::LOCALHOST, 8))
            .unwrap(),
        )
    }
    pub fn poll(&mut self) {
        let mut bytes = [0; 4096];
        for _ in 0..16 {
            let Ok((n, from)) = self.socket.recv_from(&mut bytes) else {
                break;
            };
            let Ok(request) = Message::from_vec(&bytes[..n]) else {
                continue;
            };
            let Some(q) = request.queries.first() else {
                continue;
            };
            let Some(ip) = self.names.get(&q.name().to_ascii()) else {
                continue;
            };
            assert_eq!(q.query_type(), RecordType::A);
            assert!(self.requests.len() < 1024);
            self.requests.push(from);
            if self.hold {
                continue;
            }
            let mut reply = Message::response(request.metadata.id, OpCode::Query);
            reply.metadata.authoritative = true;
            reply.queries = request.queries.clone();
            reply.answers.push(Record::from_rdata(
                q.name().clone(),
                120,
                RData::A((*ip).into()),
            ));
            self.socket.send_to(&reply.to_vec().unwrap(), from).unwrap();
        }
    }
}
