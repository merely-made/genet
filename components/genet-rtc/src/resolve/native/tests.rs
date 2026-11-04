use super::*;
use hickory_proto::op::{Message, OpCode, ResponseCode};
use hickory_proto::rr::{Name, RData, Record, RecordType};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr, TcpListener, UdpSocket};

fn server() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    socket
}
fn resolver(servers: Vec<SocketAddr>, limits: DnsLimits) -> DnsResolver {
    DnsResolver::new(DnsConfig { servers, limits }).unwrap()
}
fn receive(socket: &UdpSocket) -> Option<(Message, SocketAddr)> {
    let mut bytes = [0; 4096];
    match socket.recv_from(&mut bytes) {
        Ok((n, address)) => Some((Message::from_vec(&bytes[..n]).unwrap(), address)),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
        Err(e) => panic!("DNS fixture: {e}"),
    }
}
fn response(request: &Message) -> Message {
    let mut message = Message::response(request.metadata.id, OpCode::Query);
    message.queries = request.queries.clone();
    message
}
fn a(name: Name, ip: Ipv4Addr) -> Record {
    Record::from_rdata(name, 60, RData::A(ip.into()))
}
fn wait(mut step: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !step() {
        assert!(Instant::now() < deadline, "bounded fixture deadline");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn dns_address_families_aliases_deduplication_and_socket_release() {
    for family in [Family::V4, Family::V6] {
        let server = server();
        let resolver = resolver(
            vec![server.local_addr().unwrap()],
            DnsLimits {
                max_addresses: 2,
                ..Default::default()
            },
        );
        let now = Instant::now();
        let mut lookup = resolver
            .start(
                "rtc.example.test",
                family,
                now,
                now + Duration::from_secs(1),
            )
            .unwrap();
        let mut client = None;
        let mut result = None;
        wait(|| {
            if let Poll::Ready(value) = lookup.poll(Instant::now()) {
                result = Some(value);
                return true;
            }
            if let Some((request, address)) = receive(&server) {
                client = Some(address);
                assert_eq!(
                    request.queries[0].query_type(),
                    if family == Family::V4 {
                        RecordType::A
                    } else {
                        RecordType::AAAA
                    }
                );
                let name = request.queries[0].name().clone();
                let alias = Name::from_ascii("target.example.test.").unwrap();
                let mut message = response(&request);
                message.answers.push(a(
                    Name::from_ascii("unrelated.example.test.").unwrap(),
                    Ipv4Addr::new(192, 0, 2, 99),
                ));
                message.answers.push(Record::from_rdata(
                    name,
                    60,
                    RData::CNAME(hickory_proto::rr::rdata::CNAME(alias.clone())),
                ));
                for _ in 0..2 {
                    message
                        .answers
                        .push(a(alias.clone(), Ipv4Addr::new(127, 0, 0, 1)));
                    message.answers.push(Record::from_rdata(
                        alias.clone(),
                        60,
                        RData::AAAA(Ipv6Addr::LOCALHOST.into()),
                    ));
                }
                message.answers.push(a(alias, Ipv4Addr::new(127, 0, 0, 2)));
                server.send_to(&message.to_vec().unwrap(), address).unwrap();
            }
            false
        });
        assert_eq!(
            result.unwrap().unwrap(),
            if family == Family::V4 {
                vec![
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)),
                ]
            } else {
                vec![IpAddr::V6(Ipv6Addr::LOCALHOST)]
            }
        );
        assert_eq!(resolver.pending(), 0);
        assert!(
            UdpSocket::bind(client.unwrap()).is_ok(),
            "completed request released UDP ownership before ticket drop"
        );
    }
}

#[test]
fn dns_ignores_wrong_source_transaction_question_and_oversized_counts() {
    let server = server();
    let resolver = resolver(vec![server.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut lookup = resolver
        .start(
            "rtc.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    let mut request = None;
    wait(|| {
        assert!(lookup.poll(Instant::now()).is_pending());
        request = receive(&server);
        request.is_some()
    });
    let (request, address) = request.unwrap();
    let mut message = response(&request);
    message
        .answers
        .push(a(request.queries[0].name().clone(), Ipv4Addr::LOCALHOST));
    let wrong_source = UdpSocket::bind("127.0.0.1:0").unwrap();
    wrong_source
        .send_to(&message.to_vec().unwrap(), address)
        .unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    let mut wrong = message.clone();
    wrong.metadata.id ^= 1;
    server.send_to(&wrong.to_vec().unwrap(), address).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    wrong = message.clone();
    wrong.queries[0] = hickory_proto::op::Query::query(
        Name::from_ascii("other.example.test.").unwrap(),
        RecordType::A,
    );
    server.send_to(&wrong.to_vec().unwrap(), address).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    let mut excessive = message.to_vec().unwrap();
    excessive[6..8].copy_from_slice(&u16::MAX.to_be_bytes());
    server.send_to(&excessive, address).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    server.send_to(&message.to_vec().unwrap(), address).unwrap();
    wait(|| matches!(lookup.poll(Instant::now()), Poll::Ready(Ok(_))));
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn dns_nxdomain_timeout_capacity_and_drop_are_bounded() {
    let server = server();
    let limits = DnsLimits {
        max_queries: 1,
        timeout: Duration::from_millis(30),
        ..Default::default()
    };
    let resolver = resolver(vec![server.local_addr().unwrap()], limits);
    let now = Instant::now();
    let mut lookup = resolver
        .start(
            "missing.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    assert!(matches!(
        resolver.start(
            "extra.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1)
        ),
        Err(ResolveError::Limit)
    ));
    let mut client = None;
    wait(|| {
        if let Poll::Ready(result) = lookup.poll(Instant::now()) {
            assert_eq!(result, Err(ResolveError::NotFound));
            return true;
        }
        if let Some((request, address)) = receive(&server) {
            client = Some(address);
            let mut reply = response(&request);
            reply.metadata.response_code = ResponseCode::NXDomain;
            server.send_to(&reply.to_vec().unwrap(), address).unwrap();
        }
        false
    });
    assert_eq!(resolver.pending(), 0);
    let mut pending = resolver
        .start(
            "silent.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    let _ = pending.poll(now);
    drop(pending);
    assert_eq!(resolver.pending(), 0);
    assert!(UdpSocket::bind(client.unwrap()).is_ok());
    let mut pending = resolver
        .start(
            "silent.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    assert_eq!(
        pending.poll(now + Duration::from_millis(31)),
        Poll::Ready(Err(ResolveError::Timeout))
    );
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn dns_retries_name_servers_and_follows_alias_queries() {
    let silent = server();
    let good = server();
    let resolver = resolver(
        vec![silent.local_addr().unwrap(), good.local_addr().unwrap()],
        DnsLimits {
            retry_interval: Duration::from_millis(10),
            ..Default::default()
        },
    );
    let now = Instant::now();
    let mut lookup = resolver
        .start(
            "rtc.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    let mut seen_silent = false;
    let mut aliases = 0;
    wait(|| {
        if let Poll::Ready(value) = lookup.poll(Instant::now()) {
            assert_eq!(value.unwrap(), vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
            return true;
        }
        seen_silent |= receive(&silent).is_some();
        if let Some((request, address)) = receive(&good) {
            let mut reply = response(&request);
            let name = request.queries[0].name().clone();
            if name.to_ascii().starts_with("rtc.") {
                reply.answers.push(Record::from_rdata(
                    name,
                    60,
                    RData::CNAME(hickory_proto::rr::rdata::CNAME(
                        Name::from_ascii("target.example.test.").unwrap(),
                    )),
                ));
                aliases += 1;
            } else {
                reply.answers.push(a(name, Ipv4Addr::LOCALHOST));
            }
            good.send_to(&reply.to_vec().unwrap(), address).unwrap();
        }
        false
    });
    assert!(seen_silent);
    assert_eq!(aliases, 1);
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn dns_alias_loop_and_name_scope_are_refused() {
    let server = server();
    let resolver = resolver(vec![server.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    assert!(matches!(
        resolver.start("peer.local", Family::V4, now, now + Duration::from_secs(1)),
        Err(ResolveError::Invalid)
    ));
    let mut lookup = resolver
        .start(
            "rtc.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    wait(|| {
        if let Poll::Ready(result) = lookup.poll(Instant::now()) {
            assert_eq!(result, Err(ResolveError::Invalid));
            return true;
        }
        if let Some((request, address)) = receive(&server) {
            let name = request.queries[0].name().clone();
            let mut reply = response(&request);
            reply.answers.push(Record::from_rdata(
                name.clone(),
                60,
                RData::CNAME(hickory_proto::rr::rdata::CNAME(name)),
            ));
            server.send_to(&reply.to_vec().unwrap(), address).unwrap();
        }
        false
    });
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn dns_truncation_uses_owned_nonblocking_tcp_and_fragmented_framing() {
    let udp = server();
    let tcp = TcpListener::bind(udp.local_addr().unwrap()).unwrap();
    tcp.set_nonblocking(true).unwrap();
    let resolver = resolver(
        vec![udp.local_addr().unwrap()],
        DnsLimits {
            retry_interval: Duration::from_secs(1),
            ..Default::default()
        },
    );
    let now = Instant::now();
    let mut lookup = resolver
        .start(
            "rtc.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut stream = None;
    let mut input = Vec::new();
    let mut output = None;
    let mut fragment_sent = false;
    wait(|| {
        if let Poll::Ready(result) = lookup.poll(Instant::now()) {
            assert_eq!(result.unwrap(), vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
            return true;
        }
        if let Some((request, address)) = receive(&udp) {
            let mut reply = response(&request);
            reply.metadata.truncation = true;
            udp.send_to(&reply.to_vec().unwrap(), address).unwrap();
        }
        if stream.is_none()
            && let Ok((socket, _)) = tcp.accept()
        {
            socket.set_nonblocking(true).unwrap();
            stream = Some(socket);
        }
        if let Some(socket) = &mut stream {
            let mut buffer = [0; 4096];
            if let Ok(n) = socket.read(&mut buffer) {
                input.extend_from_slice(&buffer[..n]);
            }
            if input.len() >= 2
                && input.len() >= usize::from(u16::from_be_bytes([input[0], input[1]])) + 2
                && output.is_none()
            {
                let request = Message::from_vec(&input[2..]).unwrap();
                let mut reply = response(&request);
                reply
                    .answers
                    .push(a(request.queries[0].name().clone(), Ipv4Addr::LOCALHOST));
                let bytes = reply.to_vec().unwrap();
                let mut framed = (bytes.len() as u16).to_be_bytes().to_vec();
                framed.extend(bytes);
                output = Some(framed);
            }
            if let Some(bytes) = &mut output {
                if !fragment_sent {
                    socket.write_all(&bytes[..1]).unwrap();
                    bytes.drain(..1);
                    fragment_sent = true;
                } else if !bytes.is_empty() {
                    socket.write_all(bytes).unwrap();
                    bytes.clear();
                }
            }
        }
        false
    });
    assert!(fragment_sent);
    assert_eq!(resolver.pending(), 0);
}
