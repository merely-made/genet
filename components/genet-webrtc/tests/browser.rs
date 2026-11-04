use std::rc::Rc;
use std::time::{Duration, Instant};

use genet_webrtc::{NativeWebRtcFactory, Settings};
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

#[path = "browser/dns_fixture.rs"]
mod dns_fixture;
#[path = "browser/local_mdns.rs"]
mod local_mdns;
#[path = "browser/mdns.rs"]
mod mdns;
#[path = "browser/mdns_fixture.rs"]
mod mdns_fixture;
#[path = "browser/service.rs"]
mod service;

#[path = "browser/dns.rs"]
mod dns;

#[path = "browser/restart.rs"]
mod restart;

fn wired<E: ScriptEngine>() -> Runtime<E> {
    wired_with(Settings {
        max_peers: 8,
        max_channels: 16,
        ..Settings::default()
    })
}

fn wired_with<E: ScriptEngine>(settings: Settings) -> Runtime<E> {
    let mut rt = Runtime::new().expect("runtime");
    rt.set_webrtc_factory(Rc::new(NativeWebRtcFactory::new(settings).unwrap()));
    rt
}

fn run<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) {
    if let Err(error) = rt.eval(source) {
        panic!("{}", rt.describe_error(&error));
    }
    rt.run_microtasks();
}

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) -> String {
    let value = rt.eval(source).unwrap();
    rt.value_to_string(&value).unwrap()
}

fn until<E: ScriptEngine>(rt: &mut Runtime<E>, expression: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while read(rt, expression) != "true" {
        assert!(
            Instant::now() < deadline,
            "timed out: {}; stage: {}; console: {:?}",
            read(rt, "globalThis.problem || ''"),
            read(rt, "globalThis.restartStage || ''"),
            rt.host().borrow().console
        );
        rt.pump_webrtc(128);
        let now = rt.clock_elapsed_ms();
        rt.run_timers(64, now).unwrap();
        rt.run_microtasks();
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn constructors_and_close<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    run(
        &mut rt,
        r#"
      function check(v) { if(!v)throw new Error('RTC assertion'); }
      check(typeof __rtc_new==='undefined' && typeof __rtc_command==='undefined' && typeof __rtcDiscard==='undefined');
      var p = new RTCPeerConnection();
      check(p.signalingState==='stable' && p.connectionState==='new');
      check(p.localDescription===null && p.remoteDescription===null);
      var ch=p.createDataChannel('zero',{ordered:false,maxRetransmits:0});
      check(ch.ordered===false && ch.maxRetransmits===0 && ch.maxPacketLifeTime===null);
      check(ch.readyState==='connecting');
      check(p.createDataChannel('ignored',{id:65535}).id===null);
      p.createDataChannel('negotiated',{negotiated:true,id:42});
      try {p.createDataChannel('duplicate',{negotiated:true,id:42});throw new Error('duplicate accepted');}catch(e){check(e.name==='OperationError');}
      try { ch.send('early');throw new Error('send accepted'); } catch(e) { check(e.name==='InvalidStateError'); }
      try { p.createDataChannel('bad',{maxRetransmits:0,maxPacketLifeTime:0});throw new Error('conflict accepted'); } catch(e) {check(e instanceof TypeError);}
      p.close(); p.close();
      check(p.signalingState==='closed' && ch.readyState==='closed');
      var rejected=false;p.createOffer().catch(e=>{check(e.name==='InvalidStateError');rejected=true;});
    "#,
    );
    assert_eq!(read(&mut rt, "rejected"), "true");
    assert_eq!(rt.pending_webrtc(), 0);
    let mut bare = Runtime::<E>::new().unwrap();
    assert_eq!(
        read(
            &mut bare,
            "(()=>{try{new RTCPeerConnection();return 'accepted';}catch(e){return e.name;}})()"
        ),
        "NotSupportedError"
    );
}

fn encrypted_browser_exchange<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    run(
        &mut rt,
        r#"
      var problem='',done=false,events=[],received=[];
      var a=new RTCPeerConnection(),b=new RTCPeerConnection();
      var ch=a.createDataChannel('bytes',{ordered:false,maxRetransmits:0});
      a.onicecandidate=e=>{if(e.candidate)b.addIceCandidate(e.candidate).catch(e=>problem=String(e));};
      b.onicecandidate=e=>{if(e.candidate)a.addIceCandidate(e.candidate).catch(e=>problem=String(e));};
      b.ondatachannel=e=>{
        var c=e.channel;events.push('datachannel');Promise.resolve().then(()=>events.push('microtask'));
        if(c.ordered!==false||c.maxRetransmits!==0)problem='incoming reliability';
        c.onopen=()=>events.push('open');
        c.onmessage=e=>{
          received.push(typeof e.data==='string'?e.data:Array.from(new Uint8Array(e.data)).join(','));
          if(received.length===4)c.send('ack');
        };
      };
      ch.onopen=()=>{
        if(ch.id===null)problem='missing assigned stream ID';
        try{a.createDataChannel('duplicate',{negotiated:true,id:ch.id});problem='duplicate stream ID';}catch(e){if(e.name!=='OperationError')problem=String(e);}
        if(!a.sctp||a.sctp.state!=='connected'||a.sctp.transport.state!=='connected'||a.sctp.transport.getRemoteCertificates().length===0)problem='native transports';
        if(a.sctp!==a.sctp||a.sctp.transport.iceTransport!==a.sctp.transport.iceTransport)problem='transport identity';
        ch.send('hello λ');
        ch.send(new Uint8Array([1,2,128,255]).buffer);
        ch.send(new Uint8Array([9,3,4,9]).subarray(1,3));
        var blob=new Blob([new Uint8Array([5,0,255])]);blob._b=new Uint8Array([99]);
        Object.defineProperty(blob,'size',{value:999});ch.send(blob);
      };
      ch.onmessage=e=>{if(e.data==='ack'){Promise.all([a.getStats(),b.getStats()]).then(reports=>{
        for(const report of reports)for(const row of report.values())if(row.type==='candidate-pair'&&(!report.has(row.localCandidateId)||!report.has(row.remoteCandidateId)))problem='dangling candidate stats reference';
        a.close();b.close();done=true;
      }).catch(e=>problem=String(e));}};
      (async()=>{await a.setLocalDescription();await b.setRemoteDescription(a.localDescription);
        await b.addIceCandidate({candidate:'candidate:mdns 1 udp 123456 unresolved.local 50000 typ host',sdpMLineIndex:0});
        await b.setLocalDescription();await a.setRemoteDescription(b.localDescription);
        var checkId=()=>{if(a.sctp.state==='connected'&&ch.id===null)problem='SCTP connected before stream ID assignment';};a.sctp.onstatechange=checkId;checkId();
      })().catch(e=>problem=String(e));
      globalThis.__rtcDeliver=()=>{problem='authored task hook called';};
    "#,
    );
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(
        read(&mut rt, "JSON.stringify(received)"),
        r#"["hello λ","1,2,128,255","3,4","5,0,255"]"#
    );
    assert_eq!(
        read(&mut rt, "events.join(',')"),
        "datachannel,microtask,open"
    );
    assert_eq!(rt.pending_webrtc(), 0);
}

fn descriptions_and_rollback<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    run(
        &mut rt,
        r#"
      var problem='',done=false,signals=[],p=new RTCPeerConnection();p.createDataChannel('rollback');
      p.onsignalingstatechange=()=>signals.push(p.signalingState);
      (async()=>{
        var offer=await p.createOffer();
        await p.setLocalDescription(offer);
        if(signals.join(',')!=='have-local-offer')throw new Error('signaling event after promise');
        if(p.signalingState!=='have-local-offer'||!p.pendingLocalDescription||p.currentLocalDescription!==null)throw new Error('pending SDP');
        var first=p.localDescription;if(first!==p.localDescription||first!==p.pendingLocalDescription)throw new Error('unstable wrapper');
        await p.setLocalDescription({type:'rollback'});
        if(signals.join(',')!=='have-local-offer,stable')throw new Error('rollback event after promise');
        if(p.signalingState!=='stable'||p.pendingLocalDescription!==null)throw new Error('rollback');
        p.close();done=true;
      })().catch(e=>problem=String(e));
    "#,
    );
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(rt.pending_webrtc(), 0);
}

fn child_lifetime<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    rt.parse_document_interleaved(
        "<html><body></body></html>",
        &script_runtime_api::NoScriptLoader,
    );
    run(
        &mut rt,
        r#"
      var f=document.createElement('iframe');document.body.appendChild(f);
      var oldWindow=f.contentWindow;
      oldWindow.eval('var peer=new RTCPeerConnection();');
    "#,
    );
    let stale = rt.eval(r#"oldWindow.eval('(function(){try{new RTCPeerConnection();return "accepted";}catch(e){return e.name;}})')"#).unwrap();
    assert_eq!(rt.pending_webrtc(), 1);
    run(&mut rt, "f.remove();");
    rt.run_event_loop(16).unwrap();
    assert_eq!(
        rt.pending_webrtc(),
        0,
        "discarded document retained a native peer"
    );
    let value = rt.engine_mut().call_function(&stale, &stale, &[]).unwrap();
    assert_eq!(rt.value_to_string(&value).unwrap(), "InvalidStateError");
}

fn relay_and_limits<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    run(
        &mut rt,
        r#"
      var problem='',done=false,p=new RTCPeerConnection({iceTransportPolicy:'relay'});
      p.createDataChannel('private');
      (async()=>{await p.setLocalDescription();if(p.localDescription.sdp.includes('a=candidate:'))throw new Error('direct candidate in relay policy');
      try{p.setConfiguration({iceTransportPolicy:'all'});throw new Error('route change accepted');}catch(e){if(e.name!=='NotSupportedError')throw e;}
      p.close();done=true;})().catch(e=>problem=String(e));
    "#,
    );
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    run(
        &mut rt,
        r#"
      var peers=Array.from({length:8},()=>new RTCPeerConnection());
      try{new RTCPeerConnection();throw new Error('peer cap bypassed');}catch(e){if(e.name!=='QuotaExceededError')throw e;}
      peers.pop().close();var replacement=new RTCPeerConnection();replacement.close();peers.forEach(p=>p.close());
    "#,
    );
    assert_eq!(rt.pending_webrtc(), 0);
}

fn top_navigation<E: ScriptEngine>() {
    struct Page;
    impl script_runtime_api::ScriptResourceLoader for Page {
        fn load(&self, _: &str) -> Option<String> {
            Some("<html><body>next</body></html>".into())
        }
    }
    let mut rt = wired::<E>();
    rt.set_base_url("https://rtc.test/one.html").unwrap();
    rt.set_script_resource_loader(Box::new(Page));
    rt.parse_document_interleaved(
        "<html><body>first</body></html>",
        &script_runtime_api::NoScriptLoader,
    );
    run(&mut rt, "var p=new RTCPeerConnection();");
    let stale=rt.eval(r#"(function(Ctor,peer){return function(){var result=[];for(const f of [()=>new Ctor(),()=>peer.createDataChannel('stale')]){try{f();result.push('accepted');}catch(e){result.push(e.name);}}return result.join(',');};})(RTCPeerConnection,p)"#).unwrap();
    run(&mut rt, "location.href='https://rtc.test/two.html';");
    rt.run_event_loop(32).unwrap();
    assert_eq!(read(&mut rt, "location.href"), "https://rtc.test/two.html");
    assert_eq!(rt.pending_webrtc(), 0);
    run(&mut rt, "var fresh=new RTCPeerConnection();");
    let value = rt.engine_mut().call_function(&stale, &stale, &[]).unwrap();
    assert_eq!(
        rt.value_to_string(&value).unwrap(),
        "InvalidStateError,InvalidStateError"
    );
    assert_eq!(rt.pending_webrtc(), 1);
    run(&mut rt, "fresh.close();");
    assert_eq!(rt.pending_webrtc(), 0);
}

fn channel_string_limits<E: ScriptEngine>() {
    let mut rt = wired::<E>();
    run(
        &mut rt,
        r#"
      var p=new RTCPeerConnection();
      function tooLong(label,options){try{p.createDataChannel(label,options);throw new Error('oversize accepted');}catch(e){if(!(e instanceof TypeError))throw e;}}
      tooLong('l'.repeat(65536),{});tooLong('µ'.repeat(32768),{});
      tooLong('',{protocol:'p'.repeat(65536)});tooLong('',{protocol:'¶'.repeat(32768)});
      var label='l'.repeat(65535),protocol='p'.repeat(65535),ch=p.createDataChannel(label,{protocol});
      if(ch.label!==label||ch.protocol!==protocol)throw new Error('maximum label altered');
      var repaired=p.createDataChannel('\ud800A\udc00🙂');
      if(repaired.label!=='�A�🙂')throw new Error('USV string conversion');p.close();
    "#,
    );
    assert_eq!(rt.pending_webrtc(), 0);
}

fn gathering_errors<E: ScriptEngine>() {
    let blackhole = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let config = serde_json::json!({"iceTransportPolicy":"relay", "iceServers":[
        {"urls":[format!("turn:{}",blackhole.local_addr().unwrap()),"turn:unresolved.invalid:3478","turn:127.0.0.1:3478?transport=tcp"],
         "username":"unused", "credential":"private-test-password"}]});
    let mut rt = wired_with::<E>(Settings {
        gathering_timeout: Duration::from_millis(40),
        ..Settings::default()
    });
    run(&mut rt, &format!("var config={config};"));
    run(
        &mut rt,
        r#"
      var p=new RTCPeerConnection(config),problem='',done=false,errors=[],order=[];
      p.createDataChannel('refused');
      p.onicecandidate=e=>{if(e.candidate)problem='direct candidate leaked';else{order.push('null');done=true;}};
      p.onicegatheringstatechange=()=>{if(p.iceGatheringState==='complete')order.push('complete');};
      p.onicecandidateerror=e=>{
        if(!(e instanceof RTCPeerConnectionIceErrorEvent)||e.errorCode!==701||e.address!==null||e.port!==null||e.url.indexOf('password')>=0||e.errorText==='')problem='ICE error metadata';
        errors.push(e.url);
      };
      try{p.setConfiguration({iceTransportPolicy:'relay'});problem='server replacement accepted';}catch(e){if(e.name!=='NotSupportedError')problem=String(e);}
      p.restartIce(); // Before initial negotiation this is a harmless request.
      p.setLocalDescription().catch(e=>problem=String(e));
    "#,
    );
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(read(&mut rt, "errors.length"), "3");
    assert_eq!(read(&mut rt, "order.join(',')"), "complete,null");
    run(&mut rt, "p.close();");
    assert_eq!(rt.pending_webrtc(), 0);
}

fn coturn_relay<E: ScriptEngine>() {
    let server: std::net::SocketAddr = std::env::var("MURM_RTC_TEST_SERVER")
        .expect("controlled coturn endpoint")
        .parse()
        .unwrap();
    let probe = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    probe.connect(server).unwrap();
    let mut rt = wired_with::<E>(Settings {
        bind: probe.local_addr().unwrap().ip(),
        gathering_timeout: Duration::from_secs(3),
        allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
        ..Settings::default()
    });
    let config = serde_json::json!({"iceTransportPolicy":"relay","iceServers":[
        {"urls":[format!("turn:{server}?transport=udp")],"username":"rtc-test","credential":"rtc-test-password"}]});
    run(&mut rt, &format!("var config={config};"));
    run(
        &mut rt,
        r#"
      var a=new RTCPeerConnection(config),b=new RTCPeerConnection(config),problem='',done=false,localCandidates=0,nullCandidates=0;
      function ice(other,e){
        if(e.candidate){
          localCandidates++;
          if(e.candidate.type!=='relay'||e.candidate.relayProtocol!=='udp'||e.candidate.url!==config.iceServers[0].urls[0]||e.candidate.relatedAddress!=='0.0.0.0'||e.candidate.relatedPort!==0)problem='relay metadata:'+JSON.stringify([e.candidate.type,e.candidate.relayProtocol,e.candidate.url,e.candidate.relatedAddress,e.candidate.relatedPort]);
          if('url' in e.candidate.toJSON()||'relayProtocol' in e.candidate.toJSON())problem='local metadata in signaling';
          other.addIceCandidate(e.candidate).catch(e=>problem=String(e));
        }else nullCandidates++;
      }
      a.onicecandidate=e=>ice(b,e);b.onicecandidate=e=>ice(a,e);
      a.onicecandidateerror=b.onicecandidateerror=e=>problem='unexpected gathering error';
      var ch=a.createDataChannel('relay'),bytes=new Uint8Array(48000);bytes.fill(90);
      b.ondatachannel=e=>{e.channel.onmessage=m=>{
        var received=new Uint8Array(m.data);
        if(received.length!==48000||received[0]!==90||received[47999]!==90)problem='relay payload';
        e.channel.send('relay acknowledged');
      };};
      ch.onopen=()=>setTimeout(()=>ch.send(bytes),2200);
      ch.onmessage=e=>{if(e.data==='relay acknowledged')done=true;else problem='reply';};
      (async()=>{await a.setLocalDescription();await b.setRemoteDescription(a.localDescription);await b.setLocalDescription();await a.setRemoteDescription(b.localDescription);})().catch(e=>problem=String(e));
    "#,
    );
    until(&mut rt, "done || problem!==''");
    assert_eq!(read(&mut rt, "problem"), "");
    assert_eq!(
        read(&mut rt, "localCandidates===2&&nullCandidates===2"),
        "true"
    );
    assert_eq!(
        read(
            &mut rt,
            "a.sctp.transport.getRemoteCertificates().length>0&&b.sctp.transport.getRemoteCertificates().length>0"
        ),
        "true"
    );
    assert_eq!(
        read(
            &mut rt,
            "a.sctp.transport.iceTransport.getLocalCandidates().every(c=>c.url===config.iceServers[0].urls[0]&&c.relayProtocol==='udp')"
        ),
        "true"
    );
    run(&mut rt, "a.close();b.close();");
    assert_eq!(rt.pending_webrtc(), 0);
}

macro_rules! both {
    ($module:ident,$engine:ty) => {
        mod $module {
            use super::*;
            #[test]
            fn local_mdns_registration() {
                super::local_mdns::exchange::<$engine>();
            }
            #[test]
            fn constructors() {
                constructors_and_close::<$engine>();
            }
            #[test]
            fn transfer() {
                encrypted_browser_exchange::<$engine>();
            }
            #[test]
            fn rollback() {
                descriptions_and_rollback::<$engine>();
            }
            #[test]
            fn document_teardown() {
                child_lifetime::<$engine>();
            }
            #[test]
            fn policy_and_budgets() {
                relay_and_limits::<$engine>();
            }
            #[test]
            fn navigation_revokes_old_handles() {
                top_navigation::<$engine>();
            }
            #[test]
            fn strings_and_utf8_limits() {
                channel_string_limits::<$engine>();
            }
            #[test]
            fn gathering_errors() {
                super::gathering_errors::<$engine>();
            }
            #[test]
            fn dns_fallback() {
                super::dns::fallback::<$engine>();
            }
            #[test]
            fn dns_cancellation() {
                super::dns::cancellation::<$engine>();
            }
            #[test]
            fn dns_errors() {
                super::dns::errors::<$engine>();
            }
            #[test]
            fn service_fallback() {
                super::service::fallback::<$engine>();
            }
            #[test]
            fn service_cancellation() {
                super::service::cancellation::<$engine>();
            }
            #[test]
            fn service_errors() {
                super::service::errors::<$engine>();
            }
            #[test]
            fn mdns_exchange() {
                super::mdns::exchange::<$engine>();
            }
            #[test]
            fn mdns_cancellation() {
                super::mdns::cancellation::<$engine>();
            }
            #[test]
            fn mdns_ignored_and_quotas() {
                super::mdns::ignored_and_quotas::<$engine>();
            }
            #[test]
            fn restart() {
                super::restart::ordinary::<$engine>();
            }
            #[test]
            #[ignore = "requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint"]
            fn coturn_dns_restart() {
                super::dns::coturn::<$engine>();
            }
            #[test]
            #[ignore = "requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint"]
            fn coturn_service_restart() {
                super::service::coturn::<$engine>();
            }
            #[test]
            #[ignore = "requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint"]
            fn coturn_restart() {
                super::restart::coturn::<$engine>();
            }
            #[test]
            #[ignore = "requires controlled coturn; MURM_RTC_TEST_SERVER numeric endpoint"]
            fn coturn_relay() {
                super::coturn_relay::<$engine>();
            }
        }
    };
}
both!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
both!(vano, script_engine_nova::NovaEngine);
