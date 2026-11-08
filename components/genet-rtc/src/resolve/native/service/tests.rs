use super::super::fixture::*;
use super::*;
use hickory_proto::op::Message;
use hickory_proto::rr::{
    RData,
    rdata::{CNAME, NAPTR, SRV},
};
use std::net::{Ipv4Addr, Ipv6Addr, UdpSocket};

mod wire;

fn request(service: Service, transport_specified: bool, port: Option<u16>) -> ServerRequest {
    ServerRequest {
        host: "rtc.example.test".into(),
        service,
        port,
        transport_specified,
    }
}
fn rr(name: &Name, data: RData) -> Record {
    Record::from_rdata(name.clone(), 0, data)
}
fn naptr(owner: &Name, order: u16, flag: &str, target: &str) -> Record {
    rr(
        owner,
        RData::NAPTR(NAPTR::new(
            order,
            0,
            flag.as_bytes().into(),
            b"RELAY:turn.udp".as_slice().into(),
            Box::new([]),
            Name::from_ascii(target).unwrap(),
        )),
    )
}
fn srv(owner: &Name, priority: u16, port: u16, target: &str) -> Record {
    rr(
        owner,
        RData::SRV(SRV::new(
            priority,
            0,
            port,
            Name::from_ascii(target).unwrap(),
        )),
    )
}
fn next(
    lookup: &mut dyn EndpointLookup,
    socket: &UdpSocket,
    seen: &mut Vec<(String, RecordType, SocketAddr)>,
    mut answer: impl FnMut(&Message) -> Message,
) -> Result<Option<EndpointBatch>, ResolveError> {
    let mut result = None;
    wait(|| {
        if let Poll::Ready(value) = lookup.poll_next(Instant::now()) {
            result = Some(value);
            return true;
        }
        if let Some((query, from)) = receive(socket) {
            seen.push((
                query.queries[0].name().to_ascii(),
                query.queries[0].query_type(),
                from,
            ));
            socket
                .send_to(&answer(&query).to_vec().unwrap(), from)
                .unwrap();
        }
        false
    });
    result.unwrap()
}

#[test]
fn service_discovery_is_lazy_preserves_ports_and_stops_on_drop() {
    let dns = server();
    let resolver = resolver(vec![dns.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut query = resolver
        .start_server(
            &request(Service::Turn, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut seen = Vec::new();
    let answer = |q: &Message| {
        let name = q.queries[0].name();
        let mut r = response(q);
        match q.queries[0].query_type() {
            RecordType::NAPTR => {
                r.answers.push(naptr(name, 20, "A", "last.example.test."));
                r.answers
                    .push(naptr(name, 10, "S", "_turn._udp.hosted.example.test."));
            }
            RecordType::SRV => {
                r.answers.push(srv(name, 10, 39002, "second.example.test."));
                r.answers.push(srv(name, 0, 39001, "first.example.test."));
            }
            RecordType::A => r.answers.push(a(name.clone(), Ipv4Addr::LOCALHOST)),
            _ => panic!("unexpected query"),
        }
        r
    };
    let first = next(query.as_mut(), &dns, &mut seen, answer)
        .unwrap()
        .unwrap();
    assert_eq!(
        first.addresses,
        vec!["127.0.0.1:39001".parse::<SocketAddr>().unwrap()]
    );
    assert_eq!(first.remaining_targets, 2);
    assert_eq!(seen.len(), 3);
    assert_eq!(resolver.pending(), 1);
    assert!(receive(&dns).is_none(), "no eager alternate lookup");
    UdpSocket::bind(seen.last().unwrap().2).expect("batch yields with DNS socket released");
    let second = next(query.as_mut(), &dns, &mut seen, answer)
        .unwrap()
        .unwrap();
    assert_eq!(second.addresses[0].port(), 39002);
    assert_eq!(second.remaining_targets, 1);
    let last = next(query.as_mut(), &dns, &mut seen, answer)
        .unwrap()
        .unwrap();
    assert_eq!(last.addresses[0].port(), 3478);
    assert_eq!(last.remaining_targets, 0);
    assert!(
        next(query.as_mut(), &dns, &mut seen, answer)
            .unwrap()
            .is_none()
    );
    assert_eq!(resolver.pending(), 0);
    assert_eq!(
        seen.iter().map(|s| s.0.as_str()).collect::<Vec<_>>(),
        vec![
            "rtc.example.test.",
            "_turn._udp.hosted.example.test.",
            "first.example.test.",
            "second.example.test.",
            "last.example.test."
        ]
    );
    // A successful protocol attempt drops the cursor instead of polling ahead.
    let mut stopped = resolver
        .start_server(
            &request(Service::Turn, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    next(stopped.as_mut(), &dns, &mut seen, answer)
        .unwrap()
        .unwrap();
    drop(stopped);
    assert_eq!(resolver.pending(), 0);
    assert!(receive(&dns).is_none());
}

#[test]
fn explicit_port_bypasses_service_queries_and_udp_bypasses_naptr() {
    for (service, transport, port, types) in [
        (Service::Stun, false, Some(39999), vec![RecordType::A]),
        (Service::Turn, false, Some(39999), vec![RecordType::A]),
        (
            Service::Stun,
            false,
            None,
            vec![RecordType::SRV, RecordType::A],
        ),
        (
            Service::Turn,
            true,
            None,
            vec![RecordType::SRV, RecordType::A],
        ),
        (
            Service::Turn,
            false,
            None,
            vec![RecordType::NAPTR, RecordType::SRV, RecordType::A],
        ),
    ] {
        let dns = server();
        let resolver = resolver(vec![dns.local_addr().unwrap()], DnsLimits::default());
        let now = Instant::now();
        let mut query = resolver
            .start_server(
                &request(service, transport, port),
                Family::V4,
                now,
                now + Duration::from_secs(1),
            )
            .unwrap();
        let mut seen = Vec::new();
        let batch = next(query.as_mut(), &dns, &mut seen, |q| {
            let mut r = response(q);
            if q.queries[0].query_type() == RecordType::A {
                r.answers
                    .push(a(q.queries[0].name().clone(), Ipv4Addr::LOCALHOST));
            }
            r
        })
        .unwrap()
        .unwrap();
        assert_eq!(batch.addresses[0].port(), port.unwrap_or(3478));
        assert_eq!(seen.iter().map(|s| s.1).collect::<Vec<_>>(), types);
        assert_eq!(resolver.pending(), 0);
    }
}

#[test]
fn explicit_service_unavailability_never_falls_back_to_an_address() {
    let dns = server();
    let resolver = resolver(vec![dns.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut query = resolver
        .start_server(
            &request(Service::Stun, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    let mut seen = Vec::new();
    assert!(
        next(query.as_mut(), &dns, &mut seen, |q| {
            let mut r = response(q);
            r.answers.push(srv(q.queries[0].name(), 0, 3478, "."));
            r
        })
        .unwrap()
        .is_none()
    );
    assert_eq!(seen.len(), 1);
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn naptr_delegation_backtracks_without_inventing_default_routes() {
    let dns = server();
    let resolver = resolver(vec![dns.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut query = resolver
        .start_server(
            &request(Service::Turn, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut seen = Vec::new();
    let batch = next(query.as_mut(), &dns, &mut seen, |q| {
        let mut r = response(q);
        let name = q.queries[0].name();
        match name.to_ascii().as_str() {
            "rtc.example.test." => {
                r.answers.push(naptr(name, 10, "", "child.example.test."));
                r.answers
                    .push(naptr(name, 20, "S", "_turn._udp.empty.example.test."));
                r.answers.push(naptr(name, 30, "A", "final.example.test."));
            }
            "child.example.test." => {
                r.answers.push(naptr(name, 10, "", "rtc.example.test."));
                r.answers
                    .push(naptr(name, 20, "S", "_turn._udp.unavailable.example.test."));
            }
            "_turn._udp.unavailable.example.test." => r.answers.push(srv(name, 0, 3478, ".")),
            "_turn._udp.empty.example.test." => {}
            "final.example.test." => r.answers.push(a(name.clone(), Ipv4Addr::LOCALHOST)),
            _ => panic!("unexpected fallback: {}", name.to_ascii()),
        }
        r
    })
    .unwrap()
    .unwrap();
    assert_eq!(batch.addresses[0].port(), 3478);
    assert_eq!(seen.len(), 5);
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn srv_target_alias_is_refused_but_another_target_can_succeed() {
    let dns = server();
    let resolver = resolver(vec![dns.local_addr().unwrap()], DnsLimits::default());
    let now = Instant::now();
    let mut query = resolver
        .start_server(
            &request(Service::Stun, false, None),
            Family::V6,
            now,
            now + Duration::from_secs(2),
        )
        .unwrap();
    let mut seen = Vec::new();
    let batch = next(query.as_mut(), &dns, &mut seen, |q| {
        let mut r = response(q);
        let name = q.queries[0].name();
        if q.queries[0].query_type() == RecordType::SRV {
            r.answers.push(srv(name, 0, 39001, "alias.example.test."));
            r.answers.push(srv(name, 1, 39002, "good.example.test."));
        } else if name.to_ascii() == "alias.example.test." {
            let target = Name::from_ascii("hidden.example.test.").unwrap();
            r.answers
                .push(rr(name, RData::CNAME(CNAME(target.clone()))));
            r.answers
                .push(rr(&target, RData::AAAA(Ipv6Addr::LOCALHOST.into())));
        } else {
            assert_eq!(name.to_ascii(), "good.example.test.");
            assert_eq!(q.queries[0].query_type(), RecordType::AAAA);
            r.answers
                .push(rr(name, RData::AAAA(Ipv6Addr::LOCALHOST.into())));
        }
        r
    })
    .unwrap()
    .unwrap();
    assert_eq!(
        batch.addresses,
        vec!["[::1]:39002".parse::<SocketAddr>().unwrap()]
    );
    assert_eq!(seen.len(), 3);
    assert_eq!(resolver.pending(), 0);
}

#[test]
fn service_limits_and_drop_bound_queries_and_capacity() {
    for (limits, root) in [
        (
            DnsLimits {
                max_service_queries: 1,
                ..Default::default()
            },
            "_turn._udp.next.example.test.",
        ),
        (
            DnsLimits {
                max_delegations: 1,
                ..Default::default()
            },
            "child.example.test.",
        ),
    ] {
        let dns = server();
        let resolver = resolver(vec![dns.local_addr().unwrap()], limits);
        let now = Instant::now();
        let mut query = resolver
            .start_server(
                &request(Service::Turn, false, None),
                Family::V4,
                now,
                now + Duration::from_secs(1),
            )
            .unwrap();
        let mut seen = Vec::new();
        let result = next(query.as_mut(), &dns, &mut seen, |q| {
            let mut r = response(q);
            r.answers.push(naptr(
                q.queries[0].name(),
                0,
                if root.starts_with('_') { "S" } else { "" },
                root,
            ));
            r
        });
        assert!(matches!(result, Err(ResolveError::Limit) | Ok(None)));
        assert_eq!(seen.len(), 1);
        assert_eq!(resolver.pending(), 0);
    }
    let dns = server();
    let resolver = resolver(
        vec![dns.local_addr().unwrap()],
        DnsLimits {
            max_queries: 1,
            ..Default::default()
        },
    );
    let now = Instant::now();
    let mut query = resolver
        .start_server(
            &request(Service::Turn, false, None),
            Family::V4,
            now,
            now + Duration::from_secs(1),
        )
        .unwrap();
    assert!(matches!(
        resolver.start(
            "other.example.test",
            Family::V4,
            now,
            now + Duration::from_secs(1)
        ),
        Err(ResolveError::Limit)
    ));
    assert!(query.poll_next(now).is_pending());
    let (_, from) = receive(&dns).unwrap();
    drop(query);
    assert_eq!(resolver.pending(), 0);
    UdpSocket::bind(from).expect("cancel releases service DNS I/O");
}

#[test]
fn weighted_order_respects_priority_and_inclusive_zero_weight_selection() {
    let make = |priority, weight, port| {
        SRV::new(
            priority,
            weight,
            port,
            Name::from_ascii("server.example.test.").unwrap(),
        )
    };
    let rows = vec![
        make(10, 65535, 9),
        make(0, 5, 2),
        make(0, 0, 1),
        make(0, 10, 3),
    ];
    let zero = records::weighted(rows.clone(), |_| Ok(0)).unwrap();
    assert_eq!(
        zero.iter().map(|r| r.port).collect::<Vec<_>>(),
        vec![1, 2, 3, 9]
    );
    let max = records::weighted(rows, Ok).unwrap();
    assert_eq!(
        max.iter().map(|r| r.port).collect::<Vec<_>>(),
        vec![3, 2, 1, 9]
    );
}
