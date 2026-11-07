use super::dns_fixture::DnsFixture;
use super::*;
use hickory_proto::op::ResponseCode;
use hickory_proto::rr::{RData, Record};
use murm_rtc::resolve::{DnsConfig, DnsLimits, DnsResolver};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;

struct Dns {
    fixture: DnsFixture,
    addresses: Vec<Ipv4Addr>,
    answer: bool,
}
impl Dns {
    fn new(addresses: Vec<Ipv4Addr>) -> Self {
        Self {
            fixture: DnsFixture::new(),
            addresses,
            answer: true,
        }
    }
    fn resolver(&self) -> Arc<DnsResolver> {
        self.fixture.resolver(DnsLimits {
            timeout: Duration::from_secs(2),
            ..Default::default()
        })
    }
    fn poll(&mut self) {
        let enabled = self.answer;
        let addresses = &self.addresses;
        self.fixture.poll(|request| {
            if !enabled {
                return None;
            }
            let mut response = super::dns_fixture::response(request);
            if addresses.is_empty() {
                response.metadata.response_code = ResponseCode::NXDomain;
            }
            for address in addresses {
                response.answers.push(Record::from_rdata(
                    request.queries[0].name().clone(),
                    0,
                    RData::A((*address).into()),
                ));
            }
            Some(response)
        });
    }
}
fn runtime<E: ScriptEngine>(resolver: Arc<DnsResolver>, bind: std::net::IpAddr) -> Runtime<E> {
    let mut rt = Runtime::new().unwrap();
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::with_resolver(
            Settings {
                bind,
                gathering_timeout: Duration::from_millis(500),
                ..Settings::default()
            },
            resolver,
        )
        .unwrap(),
    ));
    rt
}
fn drive<E: ScriptEngine>(
    rt: &mut Runtime<E>,
    fixture: &mut Dns,
    expression: &str,
    mut io: impl FnMut(),
) {
    super::dns_fixture::drive(rt, expression, |_| {
        fixture.poll();
        io();
    });
}

pub(super) fn fallback<E: ScriptEngine>() {
    use murm_rtc::rtc::stun::message::{BINDING_SUCCESS, Message as Stun};
    use murm_rtc::rtc::stun::xoraddr::XorMappedAddress;
    let mut dns = Dns::new(vec![Ipv4Addr::new(127, 0, 0, 2), Ipv4Addr::LOCALHOST]);
    let resolver = dns.resolver();
    let stun = UdpSocket::bind("127.0.0.1:0").unwrap();
    stun.set_nonblocking(true).unwrap();
    let _blackhole = UdpSocket::bind((
        Ipv4Addr::new(127, 0, 0, 2),
        stun.local_addr().unwrap().port(),
    ))
    .unwrap();
    let mut rt = runtime::<E>(resolver.clone(), Ipv4Addr::LOCALHOST.into());
    run(
        &mut rt,
        &format!(
            "var url='stun:rtc.example.test:{}';",
            stun.local_addr().unwrap().port()
        ),
    );
    run(
        &mut rt,
        r#"
      var problem='',done=false,errors=[],urls=[],nulls=0,ticks=0;
      var p=new RTCPeerConnection({iceServers:[{urls:[url]}]});p.createDataChannel('dns');
      p.onicecandidateerror=e=>errors.push(e.url);
      p.onicecandidate=e=>{if(!e.candidate){nulls++;done=true;}else if(e.candidate.type==='srflx'){
        urls.push(e.candidate.url);if('url' in e.candidate.toJSON())problem='URL leaked into signaling';
      }};
      setTimeout(()=>ticks++,0);p.setLocalDescription().catch(e=>problem=String(e));
    "#,
    );
    let mut bindings = 0;
    let mut io = || {
        let mut bytes = [0; 4096];
        if let Ok((n, from)) = stun.recv_from(&mut bytes) {
            let mut request = Stun::new();
            request.raw = bytes[..n].to_vec();
            request.decode().unwrap();
            let mut response = Stun::new();
            response
                .build(&[
                    Box::new(request.transaction_id),
                    Box::new(BINDING_SUCCESS),
                    Box::new(XorMappedAddress {
                        ip: "192.0.2.10".parse().unwrap(),
                        port: from.port(),
                    }),
                ])
                .unwrap();
            stun.send_to(&response.raw, from).unwrap();
            bindings += 1;
        }
    };
    for attempt in 0..2 {
        if attempt != 0 {
            run(
                &mut rt,
                "done=false;p.createOffer({iceRestart:true}).then(o=>p.setLocalDescription(o)).catch(e=>problem=String(e));",
            );
        }
        drive(&mut rt, &mut dns, "done || problem!==''", &mut io);
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(
            read(
                &mut rt,
                "errors.length===0&&ticks===1&&urls.length>0&&urls.every(u=>u===url)"
            ),
            "true"
        );
        assert_eq!(read(&mut rt, "nulls"), (attempt + 1).to_string());
        assert_eq!(resolver.pending(), 0);
    }
    assert_eq!(bindings, 2);
    assert_eq!(dns.fixture.requests.len(), 2);
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
    for navigate in [false, true] {
        let mut dns = Dns::new(vec![Ipv4Addr::LOCALHOST]);
        dns.answer = false;
        let resolver = dns.resolver();
        let mut rt = runtime::<E>(resolver.clone(), Ipv4Addr::LOCALHOST.into());
        rt.set_base_url("https://rtc.test/one.html").unwrap();
        rt.set_script_resource_loader(Box::new(Page));
        rt.parse_document_interleaved("<body>first</body>", &script_runtime_api::NoScriptLoader);
        run(
            &mut rt,
            r#"
          var problem='',p=new RTCPeerConnection({iceTransportPolicy:'relay',iceServers:[{urls:['turn:rtc.example.test:3478'],username:'unused',credential:'private'}]});
          p.createDataChannel('pending');p.setLocalDescription().catch(e=>problem=String(e));
        "#,
        );
        drive(
            &mut rt,
            &mut dns,
            "p.iceGatheringState==='gathering'",
            || {},
        );
        for _ in 0..8 {
            rt.pump_webrtc(16);
            dns.poll();
        }
        assert_eq!(resolver.pending(), 1);
        assert!(!dns.fixture.requests.is_empty());
        let old = dns.fixture.requests[0];
        run(
            &mut rt,
            "p.createOffer({iceRestart:true}).then(o=>p.setLocalDescription(o)).catch(e=>problem=String(e));",
        );
        rt.pump_webrtc(16);
        dns.poll();
        assert_eq!(resolver.pending(), 1);
        UdpSocket::bind(old).expect("restart released old DNS socket");
        if navigate {
            run(&mut rt, "location.href='https://rtc.test/two.html'");
            rt.run_event_loop(32).unwrap();
        } else {
            run(&mut rt, "p.close()");
        }
        assert_eq!(resolver.pending(), 0);
        assert_eq!(rt.pending_webrtc(), 0);
        // Late answers to the retired source ports cannot recreate a job.
        for address in &dns.fixture.requests {
            dns.fixture.socket.send_to(&[0; 12], address).unwrap();
        }
        rt.pump_webrtc(32);
        assert_eq!(resolver.pending(), 0);
        for address in &dns.fixture.requests {
            UdpSocket::bind(address).expect("DNS socket released after realm teardown");
        }
    }
}

pub(super) fn errors<E: ScriptEngine>() {
    for available in [true, false] {
        let mut dns = Dns::new(Vec::new());
        let resolver = if available {
            dns.resolver()
        } else {
            Arc::new(
                DnsResolver::new(DnsConfig {
                    servers: Vec::new(),
                    limits: DnsLimits::default(),
                })
                .unwrap(),
            )
        };
        let mut rt = runtime::<E>(resolver.clone(), Ipv4Addr::LOCALHOST.into());
        run(
            &mut rt,
            r#"
      var problem='',done=false,errors=[],order=[],p=new RTCPeerConnection({iceTransportPolicy:'relay',iceServers:[{
        urls:['turn:missing.example.test:3478','turns:secure.example.test:5349','turn:tcp.example.test:3478?transport=tcp'],username:'private-user',credential:'private-password'}]});
      p.createDataChannel('errors');p.onicecandidate=e=>{if(e.candidate)problem='direct leak';else{order.push('null');done=true;}};
      p.onicegatheringstatechange=()=>{if(p.iceGatheringState==='complete')order.push('complete');};
      p.onicecandidateerror=e=>{order.push('error');if(e.errorCode!==701||e.address!==null||e.port!==null||/private/.test(e.url+e.errorText))problem='error privacy';errors.push(e.url);};
      p.setLocalDescription().catch(e=>problem=String(e));
    "#,
        );
        drive(&mut rt, &mut dns, "done || problem!==''", || {});
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(
            read(&mut rt, "new Set(errors).size===3&&errors.length===3"),
            "true"
        );
        assert_eq!(
            read(&mut rt, "order.join(',')"),
            "error,error,error,complete,null"
        );
        assert_eq!(
            dns.fixture.requests.len(),
            usize::from(available),
            "unsupported URLs must not start DNS"
        );
        assert_eq!(resolver.pending(), 0);
        run(&mut rt, "p.close()");
        assert_eq!(rt.pending_webrtc(), 0);
    }
}

pub(super) fn coturn<E: ScriptEngine>() {
    let server: SocketAddr = std::env::var("MURM_RTC_TEST_SERVER")
        .expect("controlled coturn endpoint")
        .parse()
        .unwrap();
    let probe = UdpSocket::bind("0.0.0.0:0").unwrap();
    probe.connect(server).unwrap();
    let std::net::IpAddr::V4(ip) = server.ip() else {
        panic!("IPv4 fixture required");
    };
    for relay in [false, true] {
        let mut dns = Dns::new(vec![ip]);
        let resolver = dns.resolver();
        let mut rt = Runtime::<E>::new().unwrap();
        rt.set_webrtc_factory(Rc::new(
            NativeWebRtcFactory::with_resolver(
                Settings {
                    bind: probe.local_addr().unwrap().ip(),
                    gathering_timeout: Duration::from_secs(3),
                    allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
                    ..Settings::default()
                },
                resolver.clone(),
            )
            .unwrap(),
        ));
        let config = serde_json::json!({"iceTransportPolicy":if relay {"relay"} else {"all"},"iceServers":[{
            "urls":[format!("{}:rtc.example.test:{}",if relay {"turn"} else {"stun"},server.port())],
            "username":"rtc-test","credential":"rtc-test-password"}]});
        run(
            &mut rt,
            &format!("var config={config},done=false,problem='';"),
        );
        run(&mut rt, include_str!("restart.js"));
        drive(&mut rt, &mut dns, "done || problem!==''", || {});
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(rt.pending_webrtc(), 0);
        assert_eq!(resolver.pending(), 0);
        assert!(dns.fixture.requests.len() >= 2);
    }
}
