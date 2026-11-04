use super::dns_fixture::drive;
use super::mdns_fixture::Fixture;
use super::*;
use std::net::UdpSocket;

fn runtime<E: ScriptEngine>(
    fixture: &Fixture,
    settings: Settings,
) -> (Runtime<E>, std::sync::Arc<murm_rtc::resolve::DnsResolver>) {
    let resolver = fixture.resolver();
    let mut rt = Runtime::new().unwrap();
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::with_resolver(settings, resolver.clone()).unwrap(),
    ));
    (rt, resolver)
}
fn aliases<E: ScriptEngine>(rt: &mut Runtime<E>, a: &str, b: &str) {
    run(rt, &format!("var aliases=[{a:?},{b:?}];"));
    run(
        rt,
        r#"
      var problem='',done=false,phase='',a,b,c,echo=[],pending=[],collected=new WeakMap(),gatherEnds=new WeakMap();
      function mapped(value,name){return value.replace(/(candidate:\S+\s+\d+\s+udp\s+\d+\s+)\S+/gi,'$1'+name);}
      function watch(p,forward){collected.set(p,[]);let end;gatherEnds.set(p,new Promise(resolve=>end=resolve));p.onicecandidate=e=>{if(e.candidate)collected.get(p).push(e.candidate);else end();if(forward)forward(e);};}
      function gathered(p){return gatherEnds.get(p);}
      function description(d,name,embedded,peer){let sdp=d.sdp.replace(/^a=candidate:.*\r?\n/gm,'');if(embedded)for(const c of collected.get(peer)||[])if(c.type==='host')sdp+='a='+mapped(c.candidate,name)+'\r\n';return {type:d.type,sdp};}
      function trickle(peer,event,name){if(event.candidate&&event.candidate.type==='host'){
        const value={...event.candidate.toJSON(),candidate:mapped(event.candidate.candidate,name)};
        if(peer.remoteDescription)peer.addIceCandidate(value).catch(e=>problem=String(e));else pending.push([peer,value]);}}
      function checkNames(p){const cs=p.sctp.transport.iceTransport.getRemoteCandidates();
        if(cs.length!==1||!cs.every(c=>aliases.includes(c.address)&&c.type==='host'))throw new Error('remote name projection '+JSON.stringify(cs.map(c=>({address:c.address,type:c.type}))));}
      async function privacy(p){const stats=await p.getStats();const host=Array.from(stats.values()).filter(s=>s.type==='remote-candidate'&&s.candidateType==='host');
        if(host.length!==1||host.some(s=>s.address!==null||'url'in s||'relayProtocol'in s))throw new Error('mDNS stats privacy');return host[0].id;}
    "#,
    );
}
pub(super) fn exchange<E: ScriptEngine>() {
    for embedded in [false, true] {
        let mut fixture = Fixture::new();
        let first = fixture.name();
        let second = fixture.name();
        let (mut rt, resolver) = runtime::<E>(&fixture, Settings::default());
        aliases(&mut rt, &first, &second);
        run(&mut rt, &format!("var embedded={embedded};"));
        run(
            &mut rt,
            r#"
          (async()=>{a=new RTCPeerConnection();b=new RTCPeerConnection();c=a.createDataChannel('mdns');
            watch(a,embedded?null:e=>trickle(b,e,aliases[0]));watch(b,embedded?null:e=>trickle(a,e,aliases[1]));
            b.ondatachannel=e=>{e.channel.onmessage=m=>e.channel.send(m.data);};c.onmessage=e=>echo.push(e.data);
            await a.setLocalDescription();await gathered(a);await b.setRemoteDescription(description(a.localDescription,aliases[0],embedded,a));
            await b.setLocalDescription();await gathered(b);await a.setRemoteDescription(description(b.localDescription,aliases[1],embedded,b));
            for(const [p,v]of pending)await p.addIceCandidate(v);
            phase='negotiated';
          })().catch(e=>problem=String(e));
        "#,
        );
        drive(&mut rt, "phase==='negotiated'||problem!==''", |_| {});
        assert_eq!(read(&mut rt, "problem"), "", "embedded {embedded}");
        run(
            &mut rt,
            "(async()=>{globalThis.before=await privacy(a);phase='waiting';})().catch(e=>problem=String(e))",
        );
        drive(&mut rt, "phase==='waiting'||problem!==''", |_| {});
        assert_eq!(read(&mut rt, "problem"), "");
        assert!(resolver.pending() >= 1);
        drive(
            &mut rt,
            "(c.readyState==='open'&&['connected','completed'].includes(a.iceConnectionState)&&['connected','completed'].includes(b.iceConnectionState))||problem!==''",
            |_| fixture.poll(),
        );
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(resolver.mdns_counts(), (2, 2));
        assert_eq!(resolver.pending(), 0);
        run(
            &mut rt,
            r#"c.send('mDNS bytes');(async()=>{checkNames(a);checkNames(b);if(await privacy(a)!==before)throw new Error('unstable observation');done=true;})().catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "(done&&echo.length===1)||problem!==''", |_| {
            fixture.poll()
        });
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(read(&mut rt, "echo[0]"), "mDNS bytes");
        run(&mut rt, "a.close();b.close()");
        assert_eq!(rt.pending_webrtc(), 0);
        for address in &fixture.requests {
            UdpSocket::bind(address).expect("resolved mDNS socket released");
        }
    }
}
pub(super) fn cancellation<E: ScriptEngine>() {
    struct Page;
    impl script_runtime_api::ScriptResourceLoader for Page {
        fn load(&self, _: &str) -> Option<String> {
            Some("<body>next</body>".into())
        }
    }
    for navigate in [false, true] {
        let mut fixture = Fixture::new();
        fixture.hold = true;
        let first = fixture.name();
        let second = fixture.name();
        let (mut rt, resolver) = runtime::<E>(&fixture, Settings::default());
        rt.set_base_url("https://rtc.test/one.html").unwrap();
        rt.set_script_resource_loader(Box::new(Page));
        rt.parse_document_interleaved("<body>first</body>", &script_runtime_api::NoScriptLoader);
        aliases(&mut rt, &first, &second);
        run(
            &mut rt,
            r#"(async()=>{a=new RTCPeerConnection();b=new RTCPeerConnection();b.createDataChannel('held');watch(b);
          await b.setLocalDescription();await gathered(b);await a.setRemoteDescription(description(b.localDescription,aliases[0],true,b));
          await a.setLocalDescription();await gathered(a);await b.setRemoteDescription(description(a.localDescription,aliases[1],false,a));phase='pending';
        })().catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "phase==='pending'||problem!==''", |_| {
            fixture.poll()
        });
        assert_eq!(read(&mut rt, "problem"), "");
        drive(&mut rt, "globalThis.seen===true", |rt| {
            fixture.poll();
            if !fixture.requests.is_empty() {
                run(rt, "seen=true");
            }
        });
        assert_eq!(resolver.pending(), 1);
        let old = *fixture.requests.last().unwrap();
        run(
            &mut rt,
            r#"a.createOffer({iceRestart:true}).then(o=>a.setLocalDescription(o)).then(()=>phase='restarted').catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "phase==='restarted'||problem!==''", |_| {
            fixture.poll()
        });
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(resolver.pending(), 0);
        UdpSocket::bind(old).expect("local restart cancels old query");
        run(
            &mut rt,
            r#"a.setLocalDescription({type:'rollback'}).then(()=>b.createOffer({iceRestart:true})).then(o=>b.setLocalDescription(o)).then(()=>a.setRemoteDescription(description(b.localDescription,aliases[1],true,b))).then(()=>phase='changed').catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "phase==='changed'||problem!==''", |_| {
            fixture.poll()
        });
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(resolver.pending(), 1);
        run(&mut rt, "seen=false");
        let count = fixture.requests.len();
        drive(&mut rt, "seen", |rt| {
            fixture.poll();
            if fixture.requests.len() > count {
                run(rt, "seen=true");
            }
        });
        let last = *fixture.requests.last().unwrap();
        if navigate {
            run(&mut rt, "location.href='https://rtc.test/two.html'");
            rt.run_event_loop(32).unwrap();
        } else {
            run(&mut rt, "a.close();b.close()");
        }
        assert_eq!(resolver.pending(), 0);
        assert_eq!(rt.pending_webrtc(), 0);
        UdpSocket::bind(last).expect("realm cleanup releases mDNS");
    }
}
pub(super) fn ignored_and_quotas<E: ScriptEngine>() {
    for relay in [false, true] {
        let mut fixture = Fixture::new();
        fixture.hold = true;
        let first = fixture.name();
        let second = fixture.name();
        let (mut rt, resolver) = runtime::<E>(
            &fixture,
            Settings {
                max_remote_candidates: 1,
                mdns_timeout: Duration::from_millis(30),
                ..Settings::default()
            },
        );
        aliases(&mut rt, &first, &second);
        run(&mut rt, &format!("var relay={relay};"));
        run(
            &mut rt,
            r#"(async()=>{a=new RTCPeerConnection({iceTransportPolicy:relay?'relay':'all'});b=new RTCPeerConnection();b.createDataChannel('limits');watch(b);
          await b.setLocalDescription();await gathered(b);await a.setRemoteDescription(description(b.localDescription,aliases[0],true,b));
          globalThis.before=await privacy(a);const candidate={candidate:'candidate:extra 1 udp 100 '+aliases[1]+' 50000 typ host',sdpMLineIndex:0};
          try{await a.addIceCandidate(candidate);throw new Error('quota bypass');}catch(e){if(e.name!=='QuotaExceededError')throw e;}
          await a.addIceCandidate({candidate:'candidate:tcp 1 tcp 100 '+aliases[1]+' 50000 typ host tcptype active',sdpMLineIndex:0});
          try{await a.addIceCandidate({...candidate,candidate:candidate.candidate+' x '+ 'x'.repeat(5000)});throw new Error('byte quota bypass');}catch(e){if(e.name!=='QuotaExceededError')throw e;}
          await a.addIceCandidate({candidate:'candidate:ignored 1 udp 100 printer.local 50000 typ host',sdpMLineIndex:0});
          phase='accepted';
        })().catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "phase==='accepted'||problem!==''", |_| {
            fixture.poll()
        });
        assert_eq!(read(&mut rt, "problem"), "");
        std::thread::sleep(Duration::from_millis(40));
        rt.pump_webrtc(32);
        assert_eq!(resolver.pending(), 0);
        assert_eq!(resolver.mdns_counts(), (usize::from(!relay), 0));
        run(
            &mut rt,
            r#"privacy(a).then(id=>{if(id!==before)throw new Error('failure-dependent stats');done=true;}).catch(e=>problem=String(e));"#,
        );
        drive(&mut rt, "done||problem!==''", |_| fixture.poll());
        assert_eq!(read(&mut rt, "problem"), "");
        run(&mut rt, "a.close();b.close()");
        assert_eq!(rt.pending_webrtc(), 0);
    }
}
