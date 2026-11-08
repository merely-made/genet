use super::dns_fixture::{DnsFixture, drive, response};
use super::*;
use hickory_proto::op::Message;
use hickory_proto::rr::{
    Name, RData, Record, RecordType,
    rdata::{NAPTR, SRV},
};
use genet_rtc::resolve::DnsLimits;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

fn rr(name: &Name, data: RData) -> Record {
    Record::from_rdata(name.clone(), 0, data)
}
fn srv(name: &Name, priority: u16, port: u16, target: &str) -> Record {
    rr(
        name,
        RData::SRV(SRV::new(
            priority,
            0,
            port,
            Name::from_ascii(target).unwrap(),
        )),
    )
}
fn naptr(name: &Name, flag: &str, target: &str) -> Record {
    rr(
        name,
        RData::NAPTR(NAPTR::new(
            0,
            0,
            flag.as_bytes().into(),
            b"RELAY:turn.udp".as_slice().into(),
            Box::new([]),
            Name::from_ascii(target).unwrap(),
        )),
    )
}
fn runtime<E: ScriptEngine>(dns: &DnsFixture, settings: Settings) -> Runtime<E> {
    let mut rt = Runtime::new().unwrap();
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::with_resolver(settings, dns.resolver(DnsLimits::default())).unwrap(),
    ));
    rt
}
fn service_reply(q: &Message, ip: Ipv4Addr, port: u16) -> Message {
    let mut r = response(q);
    let name = q.queries[0].name();
    match q.queries[0].query_type() {
        RecordType::NAPTR => r
            .answers
            .push(naptr(name, "S", "_turn._udp.hosted.example.test.")),
        RecordType::SRV => r.answers.push(srv(name, 0, port, "server.example.test.")),
        RecordType::A => r.answers.push(rr(name, RData::A(ip.into()))),
        _ => panic!("unexpected fixture query"),
    }
    r
}

fn binding_reply(socket: &UdpSocket) -> bool {
    use genet_rtc::rtc::stun::message::{BINDING_SUCCESS, Message as Stun};
    use genet_rtc::rtc::stun::xoraddr::XorMappedAddress;
    let mut bytes = [0; 4096];
    let Ok((n, from)) = socket.recv_from(&mut bytes) else {
        return false;
    };
    let mut q = Stun::new();
    q.raw = bytes[..n].to_vec();
    q.decode().unwrap();
    let mut r = Stun::new();
    r.build(&[
        Box::new(q.transaction_id),
        Box::new(BINDING_SUCCESS),
        Box::new(XorMappedAddress {
            ip: "198.51.100.20".parse().unwrap(),
            port: from.port(),
        }),
    ])
    .unwrap();
    socket.send_to(&r.raw, from).unwrap();
    true
}

pub(super) fn fallback<E: ScriptEngine>() {
    let mut dns = DnsFixture::new();
    let resolver = dns.resolver(DnsLimits::default());
    let good = UdpSocket::bind("127.0.0.1:0").unwrap();
    good.set_nonblocking(true).unwrap();
    let silent = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::with_resolver(
            Settings {
                gathering_timeout: Duration::from_millis(700),
                ..Settings::default()
            },
            resolver.clone(),
        )
        .unwrap(),
    ));
    run(
        &mut rt,
        r#"
      var problem='',done=false,errors=[],urls=[],nulls=0,ticks=0,url='stun:rtc.example.test';
      var p=new RTCPeerConnection({iceServers:[{urls:[url]}]});p.createDataChannel('service');
      p.onicecandidate=e=>{if(!e.candidate){nulls++;done=true;}else if(e.candidate.type==='srflx'){
        urls.push(e.candidate.url);if('url' in e.candidate.toJSON())problem='signaling metadata';
      }};p.onicecandidateerror=e=>errors.push(e.url);setTimeout(()=>ticks++,0);
      p.setLocalDescription().catch(e=>problem=String(e));
    "#,
    );
    let mut bindings = 0;
    for generation in 0..2 {
        if generation != 0 {
            run(
                &mut rt,
                "done=false;p.createOffer({iceRestart:true}).then(o=>p.setLocalDescription(o)).catch(e=>problem=String(e))",
            );
        }
        drive(&mut rt, "done || problem!==''", |_| {
            dns.poll(|q| {
                let name = q.queries[0].name();
                let mut r = response(q);
                if q.queries[0].query_type() == RecordType::SRV {
                    r.answers.push(srv(
                        name,
                        0,
                        silent.local_addr().unwrap().port(),
                        "bad.example.test.",
                    ));
                    r.answers.push(srv(
                        name,
                        1,
                        good.local_addr().unwrap().port(),
                        "good.example.test.",
                    ));
                } else {
                    assert_eq!(q.queries[0].query_type(), RecordType::A);
                    r.answers
                        .push(rr(name, RData::A(Ipv4Addr::LOCALHOST.into())));
                }
                Some(r)
            });
            bindings += usize::from(binding_reply(&good));
        });
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(
            read(
                &mut rt,
                "errors.length===0&&ticks===1&&urls.length>0&&urls.every(u=>u===url)&&p.getConfiguration().iceServers[0].urls[0]===url"
            ),
            "true"
        );
        assert_eq!(read(&mut rt, "nulls"), (generation + 1).to_string());
        assert_eq!(resolver.pending(), 0);
    }
    assert_eq!(bindings, 2);
    assert_eq!(
        dns.queries
            .iter()
            .filter(|(_, t)| *t == RecordType::SRV)
            .count(),
        2
    );
    run(&mut rt, "p.close()");
    assert_eq!(rt.pending_webrtc(), 0);
}

pub(super) fn cancellation<E: ScriptEngine>() {
    struct Page;
    impl script_runtime_api::ScriptResourceLoader for Page {
        fn load(&self, _: &str) -> Option<String> {
            Some("<body>next</body>".into())
        }
    }
    for held_type in [RecordType::NAPTR, RecordType::SRV, RecordType::A] {
        for navigate in [false, true] {
            let mut dns = DnsFixture::new();
            let resolver = dns.resolver(DnsLimits::default());
            let mut rt = Runtime::<E>::new().unwrap();
            rt.set_webrtc_factory(Rc::new(
                NativeWebRtcFactory::with_resolver(
                    Settings {
                        gathering_timeout: Duration::from_secs(2),
                        ..Settings::default()
                    },
                    resolver.clone(),
                )
                .unwrap(),
            ));
            rt.set_base_url("https://rtc.test/one.html").unwrap();
            rt.set_script_resource_loader(Box::new(Page));
            rt.parse_document_interleaved(
                "<body>first</body>",
                &script_runtime_api::NoScriptLoader,
            );
            run(
                &mut rt,
                r#"
              var problem='',held=false,p=new RTCPeerConnection({iceTransportPolicy:'relay',iceServers:[{urls:['turn:rtc.example.test'],username:'private-user',credential:'private-password'}]});
              p.createDataChannel('pending');p.setLocalDescription().catch(e=>problem=String(e));
            "#,
            );
            let mut pending = None;
            for generation in 0..2 {
                if generation != 0 {
                    run(
                        &mut rt,
                        "held=false;p.createOffer({iceRestart:true}).then(o=>p.setLocalDescription(o)).catch(e=>problem=String(e))",
                    );
                }
                drive(&mut rt, "held || problem!==''", |rt| {
                    let mut reached = false;
                    dns.poll(|q| {
                        if q.queries[0].query_type() == held_type {
                            reached = true;
                            None
                        } else {
                            Some(service_reply(q, Ipv4Addr::LOCALHOST, 3478))
                        }
                    });
                    if reached {
                        run(rt, "held=true");
                    }
                });
                assert_eq!(read(&mut rt, "problem"), "");
                assert_eq!(resolver.pending(), 1);
                let address = *dns.requests.last().unwrap();
                if let Some(old) = pending
                    && old != address
                {
                    UdpSocket::bind(old).expect("restart cancels the prior service query");
                }
                pending = Some(address);
            }
            if navigate {
                run(&mut rt, "location.href='https://rtc.test/two.html'");
                rt.run_event_loop(32).unwrap();
            } else {
                run(&mut rt, "p.close()");
            }
            assert_eq!(resolver.pending(), 0);
            assert_eq!(rt.pending_webrtc(), 0);
            UdpSocket::bind(pending.unwrap()).expect("service socket released by realm cleanup");
        }
    }
}

pub(super) fn errors<E: ScriptEngine>() {
    let mut dns = DnsFixture::new();
    let mut rt = runtime::<E>(
        &dns,
        Settings {
            gathering_timeout: Duration::from_millis(500),
            ..Settings::default()
        },
    );
    run(
        &mut rt,
        r#"
      var problem='',done=false,errors=[],order=[],urls=['turn:unavailable.example.test','turn:missing.example.test?transport=udp','turn:cycle.example.test'];
      var p=new RTCPeerConnection({iceTransportPolicy:'relay',iceServers:[{urls:urls,username:'private-user',credential:'private-password'}]});
      p.createDataChannel('errors');p.onicecandidate=e=>{if(e.candidate)problem='direct leak';else{order.push('null');done=true;}};
      p.onicegatheringstatechange=()=>{if(p.iceGatheringState==='complete')order.push('complete');};
      p.onicecandidateerror=e=>{order.push('error');if(e.errorCode!==701||e.address!==null||e.port!==null||/private/.test(e.url+e.errorText))problem='error privacy';errors.push(e.url);};
      p.setLocalDescription().catch(e=>problem=String(e));
    "#,
    );
    drive(&mut rt, "done || problem!==''", |_| {
        dns.poll(|q| {
            let mut r = response(q);
            let name = q.queries[0].name();
            match name.to_ascii().as_str() {
                "unavailable.example.test." => {
                    r.answers
                        .push(naptr(name, "S", "_turn._udp.unavailable.example.test."))
                },
                "_turn._udp.unavailable.example.test." => r.answers.push(srv(name, 0, 3478, ".")),
                "cycle.example.test." => r.answers.push(naptr(name, "", "cycle.example.test.")),
                "_turn._udp.missing.example.test." | "missing.example.test." => {},
                _ => panic!("invented fallback: {}", name.to_ascii()),
            }
            Some(r)
        })
    });
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(
        read(
            &mut rt,
            "errors.length===3&&new Set(errors).size===3&&errors.every(u=>urls.includes(u))"
        ),
        "true"
    );
    assert_eq!(
        read(&mut rt, "order.join(',')"),
        "error,error,error,complete,null"
    );
    run(&mut rt, "p.close()");
    assert_eq!(rt.pending_webrtc(), 0);
}

pub(super) fn coturn<E: ScriptEngine>() {
    let endpoint: SocketAddr = std::env::var("MURM_RTC_TEST_SERVER")
        .expect("controlled coturn endpoint")
        .parse()
        .unwrap();
    let std::net::IpAddr::V4(ip) = endpoint.ip() else {
        panic!("IPv4 fixture required");
    };
    let probe = UdpSocket::bind("0.0.0.0:0").unwrap();
    probe.connect(endpoint).unwrap();
    for relay in [false, true] {
        let mut dns = DnsFixture::new();
        let mut rt = runtime::<E>(
            &dns,
            Settings {
                bind: probe.local_addr().unwrap().ip(),
                gathering_timeout: Duration::from_secs(3),
                allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
                ..Settings::default()
            },
        );
        let config = serde_json::json!({"iceTransportPolicy":if relay {"relay"}else{"all"},"iceServers":[{"urls":[format!("{}:rtc.example.test",if relay {"turn"}else{"stun"})],"username":"rtc-test","credential":"rtc-test-password"}]});
        run(
            &mut rt,
            &format!("var config={config},problem='',done=false;"),
        );
        run(&mut rt, include_str!("restart.js"));
        drive(&mut rt, "done || problem!==''", |_| {
            dns.poll(|q| Some(service_reply(q, ip, endpoint.port())))
        });
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(rt.pending_webrtc(), 0);
        assert!(dns.queries.iter().any(|(_, t)| *t == RecordType::SRV));
        assert_eq!(
            dns.queries.iter().any(|(_, t)| *t == RecordType::NAPTR),
            relay
        );
    }
}
