use super::super::fixture::wait;
use super::*;
use hickory_proto::rr::Record;
use std::sync::Mutex;

static FIXTURE: Mutex<()> = Mutex::new(());
const HOST: &str = "12345678-1234-4234-8234-123456789abc.local";

fn responder() -> UdpSocket {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_address(true).unwrap();
    socket
        .bind(&"0.0.0.0:5353".parse::<SocketAddr>().unwrap().into())
        .unwrap();
    socket
        .join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 251), &Ipv4Addr::LOCALHOST)
        .unwrap();
    socket.set_multicast_if_v4(&Ipv4Addr::LOCALHOST).unwrap();
    socket.set_multicast_loop_v4(true).unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.into()
}
fn resolver() -> DnsResolver {
    DnsResolver::new(DnsConfig {
        servers: Vec::new(),
        limits: DnsLimits {
            max_queries: 1,
            ..Default::default()
        },
    })
    .unwrap()
    .with_mdns(MdnsConfig::new(Ipv4Addr::LOCALHOST, 8))
    .unwrap()
}
fn response(request: &Message, addresses: &[Ipv4Addr]) -> Message {
    let mut reply = Message::response(request.metadata.id, OpCode::Query);
    reply.metadata.authoritative = true;
    reply.queries = request.queries.clone();
    for ip in addresses {
        reply.answers.push(Record::from_rdata(
            request.queries[0].name().clone(),
            120,
            RData::A((*ip).into()),
        ));
    }
    reply
}
fn receive(socket: &UdpSocket) -> (Message, SocketAddr) {
    let mut result = None;
    wait(|| {
        let mut bytes = [0; 4096];
        if let Ok((n, from)) = socket.recv_from(&mut bytes) {
            let q = Message::from_vec(&bytes[..n]).unwrap();
            if q.queries.first().is_some_and(|q| {
                q.name()
                    .to_ascii()
                    .eq_ignore_ascii_case(&format!("{HOST}."))
            }) {
                result = Some((q, from));
            }
        }
        result.is_some()
    });
    result.unwrap()
}
#[test]
fn multicast_query_and_validated_unicast_reply_release_socket_and_capacity() {
    let _guard = FIXTURE.lock().unwrap();
    let socket = responder();
    let resolver = resolver();
    let now = Instant::now();
    let mut lookup = resolver
        .start_mdns(
            HOST,
            Ipv4Addr::LOCALHOST.into(),
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    assert!(lookup.poll(now).is_pending());
    let (q, from) = receive(&socket);
    assert_eq!(from.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_ne!(from.port(), 5353);
    assert_eq!(q.queries[0].query_type(), RecordType::A);
    assert!(!q.metadata.recursion_desired);
    assert_eq!(resolver.pending(), 1);
    assert_eq!(resolver.mdns_counts(), (1, 0));
    assert!(matches!(
        resolver.start_mdns(
            HOST,
            Ipv4Addr::LOCALHOST.into(),
            now,
            now + Duration::from_secs(2)
        ),
        Err(ResolveError::Limit)
    ));
    let reply = response(&q, &[Ipv4Addr::LOCALHOST, Ipv4Addr::LOCALHOST]);
    let wrong = UdpSocket::bind("127.0.0.1:0").unwrap();
    wrong.send_to(&reply.to_vec().unwrap(), from).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    let mut bad = reply.clone();
    bad.metadata.id ^= 1;
    socket.send_to(&bad.to_vec().unwrap(), from).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    bad = reply.clone();
    bad.queries[0] = Question::query(Name::from_ascii("other.local.").unwrap(), RecordType::A);
    socket.send_to(&bad.to_vec().unwrap(), from).unwrap();
    assert!(lookup.poll(Instant::now()).is_pending());
    socket.send_to(&reply.to_vec().unwrap(), from).unwrap();
    wait(|| {
        if let Poll::Ready(result) = lookup.poll(Instant::now()) {
            assert_eq!(result.unwrap(), vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
            true
        } else {
            false
        }
    });
    assert_eq!(resolver.pending(), 0);
    assert_eq!(resolver.mdns_counts(), (1, 1));
    UdpSocket::bind(from).unwrap();
}
#[test]
fn invalid_names_timeout_and_drop_do_not_use_recursive_dns() {
    let _guard = FIXTURE.lock().unwrap();
    let socket = responder();
    let resolver = resolver();
    let now = Instant::now();
    for host in [
        "printer.local",
        "12345678-1234-1234-8234-123456789abc.local",
        "12345678-1234-4234-1234-123456789abc.local",
        "rtc.example.test",
    ] {
        assert!(matches!(
            resolver.start_mdns(
                host,
                Ipv4Addr::LOCALHOST.into(),
                now,
                now + Duration::from_secs(1)
            ),
            Err(ResolveError::Invalid)
        ));
    }
    assert!(is_mdns_candidate_name(&HOST.to_uppercase()));
    assert!(matches!(
        resolver.start(HOST, Family::V4, now, now + Duration::from_secs(1)),
        Err(ResolveError::Unavailable | ResolveError::Invalid)
    ));
    for cancel in [true, false] {
        let mut lookup = resolver
            .start_mdns(
                HOST,
                Ipv4Addr::LOCALHOST.into(),
                now,
                now + Duration::from_millis(30),
            )
            .unwrap();
        assert!(lookup.poll(now).is_pending());
        let (_, from) = receive(&socket);
        if !cancel {
            assert!(matches!(
                lookup.poll(now + Duration::from_millis(31)),
                Poll::Ready(Err(ResolveError::Timeout))
            ));
        }
        drop(lookup);
        assert_eq!(resolver.pending(), 0);
        UdpSocket::bind(from).unwrap();
    }
}
#[test]
fn malformed_ambiguous_alias_and_off_link_answers_are_not_accepted() {
    let _guard = FIXTURE.lock().unwrap();
    let socket = responder();
    let resolver = resolver();
    let now = Instant::now();
    let mut lookup = resolver
        .start_mdns(
            HOST,
            Ipv4Addr::LOCALHOST.into(),
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    assert!(lookup.poll(now).is_pending());
    let (q, from) = receive(&socket);
    let reply = response(&q, &[Ipv4Addr::LOCALHOST]);
    // Parser checks are exercised on the exact request, separately from placement.
    let query = Query::new(
        HOST,
        MdnsConfig::new(Ipv4Addr::LOCALHOST, 8),
        DnsLimits::default(),
        now,
        now + Duration::from_secs(1),
        Lease(Arc::new(AtomicUsize::new(1))),
        Arc::default(),
    )
    .unwrap();
    assert!(!query.config.on_link(Ipv4Addr::new(192, 0, 2, 2)));
    let mut own = response(&query.request, &[Ipv4Addr::LOCALHOST]);
    own.answers[0].ttl = 0;
    assert_eq!(query.answer(&own.to_vec().unwrap(), false).unwrap(), None);
    own = response(
        &query.request,
        &[Ipv4Addr::LOCALHOST, Ipv4Addr::new(127, 0, 0, 2)],
    );
    assert_eq!(
        query.answer(&own.to_vec().unwrap(), false),
        Err(ResolveError::Invalid)
    );
    own = response(&query.request, &[]);
    own.answers.push(Record::from_rdata(
        query.request.queries[0].name().clone(),
        120,
        RData::CNAME(hickory_proto::rr::rdata::CNAME(
            Name::from_ascii("alias.local.").unwrap(),
        )),
    ));
    assert_eq!(
        query.answer(&own.to_vec().unwrap(), false),
        Err(ResolveError::Invalid)
    );
    own = response(&query.request, &[]);
    own.answers.push(Record::from_rdata(
        query.request.queries[0].name().clone(),
        120,
        RData::AAAA(std::net::Ipv6Addr::LOCALHOST.into()),
    ));
    assert_eq!(query.answer(&own.to_vec().unwrap(), false).unwrap(), None);
    let mut too_many = reply.to_vec().unwrap();
    too_many[6..8].copy_from_slice(&u16::MAX.to_be_bytes());
    socket.send_to(&too_many, from).unwrap();
    socket.send_to(&[0; 17000], from).unwrap();
    socket.send_to(&reply.to_vec().unwrap(), from).unwrap();
    wait(|| matches!(lookup.poll(Instant::now()), Poll::Ready(Ok(_))));
}

#[test]
fn multicast_zero_id_questionless_cache_flush_answer_is_accepted_only_on_the_joined_socket() {
    let _guard = FIXTURE.lock().unwrap();
    let socket = responder();
    let resolver = resolver();
    let now = Instant::now();
    let mut lookup = resolver
        .start_mdns(
            HOST,
            Ipv4Addr::LOCALHOST.into(),
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    assert!(lookup.poll(now).is_pending());
    let (q, from) = receive(&socket);
    let mut reply = response(&q, &[Ipv4Addr::LOCALHOST]);
    reply.metadata.id = 0;
    reply.queries.clear();
    reply.answers[0].dns_class = DNSClass::Unknown(0x8001);
    let mut echo_id = response(&q, &[Ipv4Addr::LOCALHOST]);
    echo_id.queries.clear();
    echo_id.answers[0].dns_class = DNSClass::Unknown(0x8001);
    socket.send_to(&reply.to_vec().unwrap(), from).unwrap();
    assert!(
        lookup.poll(Instant::now()).is_pending(),
        "legacy unicast still requires its question/ID"
    );
    socket
        .send_to(&reply.to_vec().unwrap(), "224.0.0.251:5353")
        .unwrap();
    wait(|| {
        if let Poll::Ready(result) = lookup.poll(Instant::now()) {
            assert_eq!(result.unwrap(), vec![IpAddr::V4(Ipv4Addr::LOCALHOST)]);
            true
        } else {
            false
        }
    });
    assert_eq!(resolver.pending(), 0);
    UdpSocket::bind(from).unwrap();
    let now = Instant::now();
    let mut query = Query::new(
        HOST,
        MdnsConfig::new(Ipv4Addr::LOCALHOST, 8),
        DnsLimits::default(),
        now,
        now + Duration::from_secs(1),
        Lease(Arc::new(AtomicUsize::new(1))),
        Arc::default(),
    )
    .unwrap();
    echo_id.metadata.id = query.request.metadata.id;
    assert_eq!(
        query.answer(&echo_id.to_vec().unwrap(), true).unwrap(),
        Some(Ipv4Addr::LOCALHOST.into())
    );
    assert_eq!(
        query.answer(&echo_id.to_vec().unwrap(), false).unwrap(),
        None
    );
    echo_id.metadata.id = if query.request.metadata.id == 1 { 2 } else { 1 };
    assert_eq!(
        query.answer(&echo_id.to_vec().unwrap(), true).unwrap(),
        None
    );
    assert!(matches!(
        query.poll(now + Duration::from_secs(2)),
        Poll::Ready(Err(ResolveError::Timeout))
    ));
    assert!(query.socket.is_none() && query.multicast.is_none() && query.lease.is_none());
}
