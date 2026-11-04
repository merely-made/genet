use super::*;
use genet_webrtc::LocalAddressPolicy;

pub(super) fn exchange<E: ScriptEngine>() {
    struct Page;
    impl script_runtime_api::ScriptResourceLoader for Page {
        fn load(&self, _: &str) -> Option<String> {
            Some("<body>next</body>".into())
        }
    }
    for embedded in [false, true] {
        let mut rt = wired_with::<E>(Settings {
            gathering_timeout: Duration::from_millis(250),
            local_address_policy: LocalAddressPolicy::Mdns,
            ..Settings::default()
        });
        rt.set_base_url("https://rtc.test/one.html").unwrap();
        rt.set_script_resource_loader(Box::new(Page));
        rt.parse_document_interleaved("<body>first</body>", &script_runtime_api::NoScriptLoader);
        run(&mut rt, &format!("var embedded={embedded};"));
        run(
            &mut rt,
            r#"
      var restartStage='initial',problem='',done=false,seen=[],a=new RTCPeerConnection(),b=new RTCPeerConnection();
      var pending=[],echo=0,hosts=new WeakMap(),completions=new WeakMap(),finish=new WeakMap();
      function reset(p){hosts.set(p,[]);completions.set(p,new Promise(r=>finish.set(p,r)));}
      async function gathered(p){await completions.get(p);}
      function description(p){return {type:p.localDescription.type,sdp:p.localDescription.sdp.replace(/^a=candidate:.*\r?\n/gm,'')+hosts.get(p).map(c=>'a='+c+'\r\n').join('')};}
      function check(c){if(c.type==='host'&&!/\.local$/.test(c.address))throw Error('numeric host');seen.push(c.address);}
      function forward(p,from,e){if(!e.candidate){finish.get(from)();return;}try{check(e.candidate);}catch(x){problem=String(x);}
        hosts.get(from).push(e.candidate.candidate);if(embedded)return;
        if(p.remoteDescription)p.addIceCandidate(e.candidate).catch(e=>problem=String(e));else pending.push([p,e.candidate]);}
      a.onicecandidate=e=>forward(b,a,e);b.onicecandidate=e=>forward(a,b,e);
      b.ondatachannel=e=>e.channel.onmessage=m=>e.channel.send(m.data);
      var ch=a.createDataChannel('private');ch.onmessage=e=>echo++;
      async function negotiate(){restartStage='offer';reset(a);reset(b);var offer=await a.createOffer();
        if(offer.sdp.includes('127.0.0.1'))throw Error('offer leak');
        await a.setLocalDescription(offer);if(embedded){restartStage='gather-a';await gathered(a);}await b.setRemoteDescription(embedded?description(a):a.localDescription);
        var answer=await b.createAnswer();if(answer.sdp.includes('127.0.0.1'))throw Error('answer leak');
        await b.setLocalDescription(answer);if(embedded){restartStage='gather-b';await gathered(b);}await a.setRemoteDescription(embedded?description(b):b.localDescription);
        restartStage='negotiated '+embedded+' '+a.iceConnectionState+' '+b.iceConnectionState+' '+hosts.get(a).length+' '+hosts.get(b).length;
        for(const [p,c]of pending.splice(0))await p.addIceCandidate(c);restartStage='negotiated';}
      negotiate().catch(e=>problem=String(e));
    "#,
        );
        until(&mut rt, "ch.readyState==='open'||problem!==''");
        assert_eq!(read(&mut rt, "problem"), "");
        run(&mut rt, "ch.send('private exchange');");
        until(&mut rt, "echo===1||problem!==''");
        run(
            &mut rt,
            r#"
      (async()=>{for(const p of [a,b]){
        if(p.localDescription.sdp.includes('127.0.0.1'))throw Error('local SDP leak');
        var local=p.sctp.transport.iceTransport.getLocalCandidates();
        if(!local.length||local.some(c=>!c.address.endsWith('.local')))throw Error('transport leak');
        var stats=Array.from((await p.getStats()).values()).filter(s=>s.type==='local-candidate');
        if(!stats.length||stats.some(s=>typeof s.address!=='string'||!s.address.endsWith('.local')))throw Error('stats leak');
      } globalThis.before=seen.slice();a.restartIce();await negotiate();done=true;
      })().catch(e=>problem=String(e));
    "#,
        );
        until(&mut rt, "done||problem!==''");
        assert_eq!(read(&mut rt, "problem"), "");
        assert_eq!(read(&mut rt, "seen.every(n=>before.includes(n))"), "true");
        assert_eq!(read(&mut rt, "new Set(before).size===2"), "true");
        if embedded {
            run(&mut rt, "location.href='https://rtc.test/private-next'");
            rt.run_event_loop(32).unwrap();
        } else {
            run(&mut rt, "a.close();b.close();");
        }
        assert_eq!(rt.pending_webrtc(), 0);
    }
}
