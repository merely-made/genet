use std::rc::Rc;
use std::time::{Duration, Instant};

use genet_webrtc::{NativeWebRtcFactory, Settings};
use murm_rtc::udp::IoLimits;
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn wired<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::new().expect("runtime");
    rt.set_webrtc_factory(Rc::new(
        NativeWebRtcFactory::new(Settings {
            bind: "127.0.0.1".parse().unwrap(),
            io: IoLimits {
                datagram_bytes: 65_535,
                packets_per_turn: 8,
                timer_floor: Duration::from_millis(1),
            },
            max_peers: 8,
            max_channels: 16,
            max_notifications: 256,
            poll_interval: Duration::from_millis(1),
        })
        .unwrap(),
    ));
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
            "timed out: {}; console: {:?}",
            read(rt, "globalThis.problem || ''"),
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
        if(!a.sctp||a.sctp.state!=='connected'||a.sctp.transport.state!=='connected'||a.sctp.transport.getRemoteCertificates().length===0)problem='native transports';
        if(a.sctp!==a.sctp||a.sctp.transport.iceTransport!==a.sctp.transport.iceTransport)problem='transport identity';
        ch.send('hello λ');
        ch.send(new Uint8Array([1,2,128,255]).buffer);
        ch.send(new Uint8Array([9,3,4,9]).subarray(1,3));
        var blob=new Blob([new Uint8Array([5,0,255])]);blob._b=new Uint8Array([99]);
        Object.defineProperty(blob,'size',{value:999});ch.send(blob);
      };
      ch.onmessage=e=>{if(e.data==='ack'){a.close();b.close();done=true;}};
      (async()=>{await a.setLocalDescription();await b.setRemoteDescription(a.localDescription);await b.setLocalDescription();await a.setRemoteDescription(b.localDescription);})().catch(e=>problem=String(e));
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

macro_rules! both {
    ($module:ident,$engine:ty) => {
        mod $module {
            use super::*;
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
        }
    };
}
both!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
both!(vano, script_engine_nova::NovaEngine);
