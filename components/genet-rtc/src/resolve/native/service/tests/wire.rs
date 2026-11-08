use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn truncated_srv_uses_owned_tcp_then_resolves_the_discovered_port() {
    let udp = server();
    let tcp = TcpListener::bind(udp.local_addr().unwrap()).unwrap();
    tcp.set_nonblocking(true).unwrap();
    let resolver = resolver(vec![udp.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut lookup = resolver
        .start_server(
            &request(Service::Stun, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut stream = None;
    let mut input = Vec::new();
    let mut output = None;
    let mut fragmented = false;
    let mut client = None;
    wait(|| {
        if let Poll::Ready(result) = lookup.poll_next(Instant::now()) {
            assert_eq!(
                result.unwrap().unwrap().addresses,
                vec!["127.0.0.1:39876".parse::<SocketAddr>().unwrap()]
            );
            return true;
        }
        if let Some((q, from)) = receive(&udp) {
            let mut r = response(&q);
            if q.queries[0].query_type() == RecordType::SRV {
                r.metadata.truncation = true;
            } else {
                assert_eq!(q.queries[0].name().to_ascii(), "server.example.test.");
                r.answers
                    .push(a(q.queries[0].name().clone(), Ipv4Addr::LOCALHOST));
            }
            udp.send_to(&r.to_vec().unwrap(), from).unwrap();
        }
        if stream.is_none()
            && let Ok((socket, from)) = tcp.accept()
        {
            socket.set_nonblocking(true).unwrap();
            client = Some(from);
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
                let q = Message::from_vec(&input[2..]).unwrap();
                assert_eq!(q.queries[0].query_type(), RecordType::SRV);
                let mut r = response(&q);
                r.answers
                    .push(srv(q.queries[0].name(), 0, 39876, "server.example.test."));
                let bytes = r.to_vec().unwrap();
                let mut frame = (bytes.len() as u16).to_be_bytes().to_vec();
                frame.extend(bytes);
                output = Some(frame);
            }
            if let Some(bytes) = &mut output {
                let count = if !fragmented {
                    fragmented = true;
                    1
                } else {
                    bytes.len()
                };
                if count != 0 {
                    socket.write_all(&bytes[..count]).unwrap();
                    bytes.drain(..count);
                }
            }
        }
        false
    });
    assert!(fragmented);
    assert!(client.is_some());
    assert_eq!(resolver.pending(), 0);
    let mut byte = [0];
    assert_eq!(
        stream.as_mut().unwrap().read(&mut byte).unwrap(),
        0,
        "resolver released TCP before cursor drop"
    );
}

#[test]
fn service_record_limits_and_unsupported_transports_are_bounded() {
    // Unsupported advertised protocols count as no supported initial result.
    let dns = server();
    let resolver = resolver(
        vec![dns.local_addr().unwrap()],
        DnsLimits {
            max_targets: 1,
            ..Default::default()
        },
    );
    let now = Instant::now();
    let mut lookup = resolver
        .start_server(
            &request(Service::Turn, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut seen = Vec::new();
    let batch = next(lookup.as_mut(), &dns, &mut seen, |q| {
        let mut r = response(q);
        let name = q.queries[0].name();
        match q.queries[0].query_type() {
            RecordType::NAPTR => r.answers.push(rr(
                name,
                RData::NAPTR(NAPTR::new(
                    0,
                    0,
                    b"S".as_slice().into(),
                    b"RELAY:turn.tcp".as_slice().into(),
                    Box::new([]),
                    Name::from_ascii("_turn._tcp.unsupported.example.test.").unwrap(),
                )),
            )),
            RecordType::SRV => {}
            RecordType::A => {
                r.answers.push(a(name.clone(), Ipv4Addr::LOCALHOST));
                r.answers.push(a(name.clone(), Ipv4Addr::new(127, 0, 0, 2)));
            }
            _ => panic!("unexpected query"),
        }
        r
    })
    .unwrap()
    .unwrap();
    assert_eq!(batch.addresses.len(), 1);
    assert_eq!(resolver.pending(), 0);
    assert!(lookup.poll_next(Instant::now()).is_ready());
    assert_eq!(
        seen.iter().map(|s| s.1).collect::<Vec<_>>(),
        vec![RecordType::NAPTR, RecordType::SRV, RecordType::A]
    );

    let mut limited = resolver
        .start_server(
            &request(Service::Stun, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let result = next(limited.as_mut(), &dns, &mut seen, |q| {
        let mut r = response(q);
        r.answers
            .push(srv(q.queries[0].name(), 0, 39001, "one.example.test."));
        r.answers
            .push(srv(q.queries[0].name(), 1, 39002, "two.example.test."));
        r
    });
    assert!(matches!(result, Err(ResolveError::Limit)));
    assert_eq!(resolver.pending(), 0);
}
