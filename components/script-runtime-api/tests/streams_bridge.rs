// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Streams consumers, realm brands, collection, and native body preservation.
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use script_engine_api::ScriptEngine;
use script_runtime_api::{
    FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime, ScriptResourceLoader,
};

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, script: &str) -> String {
    let value = rt.eval(script).expect("eval");
    rt.value_to_string(&value).expect("stringify")
}

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::new().expect("runtime");
    rt.set_base_url("https://streams.test/page.html")
        .expect("base");
    rt.parse_document_interleaved("<html><head></head><body></body></html>", &NoScriptLoader);
    rt
}

struct Uploads {
    bodies: Rc<RefCell<Vec<Option<Vec<u8>>>>>,
    cancelled: Rc<RefCell<Vec<u64>>>,
}
impl FetchHandler for Uploads {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        let bytes = request.body.clone().unwrap_or_default();
        self.bodies.borrow_mut().push(request.body);
        FetchOutcome {
            network_error: false,
            status: 200,
            status_text: "OK".into(),
            response_type: "basic".into(),
            url: request.url,
            redirected: false,
            headers: vec![],
            body: bytes,
        }
    }
    fn cancel(&self, id: u64) {
        self.cancelled.borrow_mut().push(id);
    }
}
fn uploads<E: ScriptEngine>() -> (
    Runtime<E>,
    Rc<RefCell<Vec<Option<Vec<u8>>>>>,
    Rc<RefCell<Vec<u64>>>,
) {
    let bodies = Rc::new(RefCell::new(Vec::new()));
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let mut rt = runtime();
    rt.set_fetch_handler(Box::new(Uploads {
        bodies: bodies.clone(),
        cancelled: cancelled.clone(),
    }));
    (rt, bodies, cancelled)
}

fn request_upload_collects_exact_bytes_and_preserves_empty_presence<E: ScriptEngine>() {
    let (mut rt, bodies, _) = uploads::<E>();
    rt.eval(r#"var result='pending';(async()=>{
      const bytes=new ReadableStream({start(c){c.enqueue(new Uint8Array([0,255]));c.enqueue(new Uint8Array([65]));c.close();}});
      const request=new Request('https://streams.test/upload',{method:'POST',body:bytes,duplex:'half'});
      const response=await fetch(request);const data=await response.bytes();
      if(String(data)!=='0,255,65'||!request.bodyUsed||!bytes.locked)throw new Error('byte upload or proxy disturbance');
      await fetch('https://streams.test/empty',{method:'POST',body:new ReadableStream({start(c){c.close();}}),duplex:'half'});
      await fetch('https://streams.test/absent');result='ok';
    })().catch(e=>result='error:'+e.message);"#).expect("upload");
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "result"), "ok");
    assert_eq!(
        *bodies.borrow(),
        vec![Some(vec![0, 255, 65]), Some(vec![]), None]
    );
}

fn invalid_upload_chunk_never_reaches_native_sink<E: ScriptEngine>() {
    let (mut rt, bodies, _) = uploads::<E>();
    rt.eval(
        r#"var result='pending';fetch('https://streams.test/upload',{method:'POST',duplex:'half',
      body:new ReadableStream({start(c){c.enqueue(new Int16Array([65]));c.close();}})
    }).then(()=>result='wrong',e=>result=e.name);"#,
    )
    .expect("invalid upload");
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "result"), "TypeError");
    assert!(bodies.borrow().is_empty());
    assert_eq!(rt.pending_fetches(), 0);
}

fn abort_during_upload_cancels_reader_without_starting_native_work<E: ScriptEngine>() {
    let (mut rt, bodies, cancelled) = uploads::<E>();
    rt.eval(r#"var result='pending',seenReason;const reason={stop:true};const controller=new AbortController();
      fetch('https://streams.test/upload',{method:'POST',duplex:'half',signal:controller.signal,
        body:new ReadableStream({cancel(r){seenReason=r;}})
      }).then(()=>result='wrong',e=>result=e===reason&&seenReason===reason?'ok':'wrong reason');controller.abort(reason);
    "#).expect("abort upload");
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "result"), "ok");
    assert!(bodies.borrow().is_empty());
    assert!(
        cancelled.borrow().is_empty(),
        "no native request was registered"
    );
    assert_eq!(rt.pending_fetches(), 0);
}

fn live_authored_body_clones_tee_and_preserve_backing_buffer_independence<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(r#"var result='pending',controller;const stream=new ReadableStream({start(c){controller=c;}});
      const response=new Response(stream),copy=response.clone();
      (async()=>{const a=response.body.getReader(),b=copy.body.getReader();
        const first=a.read(),second=b.read();controller.enqueue(new Uint8Array([65,66]));controller.close();
        const x=(await first).value,y=(await second).value;x[0]=90;
        if(y[0]!==65||x.buffer===y.buffer||!(await a.read()).done||!(await b.read()).done)throw new Error('tee clone');
        result='ok';})().catch(e=>result='error:'+e.message);
    "#).expect("authored clone");
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "result"), "ok");
}

fn retained_child_stream_uses_parent_borrowed_methods_after_destruction<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var frame=document.createElement('iframe');frame.srcdoc='<body></body>';document.body.appendChild(frame);").expect("iframe");
    rt.run_event_loop(100).expect("initialize frame");
    let child = rt
        .frame_realms(rt.top_realm())
        .first()
        .map(|(_, realm)| *realm)
        .expect("child");
    rt.eval_in_realm(child, "parent.retainedStream=new ReadableStream({start(c){c.enqueue(new Uint8Array([65]));c.close();}});").expect("foreign stream");
    rt.eval("frame.remove();").expect("remove frame");
    rt.run_event_loop(100).expect("destroy frame");
    rt.collect_garbage();
    rt.eval(r#"var result='pending';const reader=ReadableStream.prototype.getReader.call(retainedStream);
      ReadableStreamDefaultReader.prototype.read.call(reader).then(r=>{
        result=r.value[0]===65&&!r.done?'ok':'wrong';ReadableStreamDefaultReader.prototype.releaseLock.call(reader);
      },e=>result='error:'+e.message);"#).expect("borrowed read");
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "result"), "ok");
    assert_eq!(read(&mut rt, "String(retainedStream.locked)"), "false");
}

fn discarded_stream_cycles_are_collected_by_normal_pump<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        r#"var finalized=[];var registry=new FinalizationRegistry(value=>finalized.push(value));
      (()=>{const readable=new ReadableStream(),byte=new ReadableStream({type:'bytes'});
        const writable=new WritableStream(),transform=new TransformStream();
        registry.register(readable,'readable');registry.register(byte,'byte');
        registry.register(writable,'writable');registry.register(transform,'transform');})();
    "#,
    )
    .expect("register cycles");
    rt.run_microtasks();
    for _ in 0..4 {
        rt.collect_garbage();
        rt.run_event_loop(16).expect("pump finalizers");
        if read(&mut rt, "String(finalized.length===4)") == "true" {
            return;
        }
    }
    assert_eq!(
        read(&mut rt, "finalized.sort().join(',')"),
        "byte,readable,transform,writable"
    );
}

struct WorkerScript;
impl ScriptResourceLoader for WorkerScript {
    fn load(&self, url: &str) -> Option<String> {
        url.ends_with("/streams.js").then(|| r#"(async()=>{
          const transform=new TransformStream();const writer=transform.writable.getWriter();
          const reader=transform.readable.getReader();const write=writer.write('worker');
          const item=await reader.read();await write;const close=writer.close();await reader.read();await close;
          const bytes=new ReadableStream({type:'bytes',start(c){c.enqueue(new Uint8Array([65]));c.close();}});
          const read=await bytes.getReader({mode:'byob'}).read(new Uint8Array(1));
          postMessage(item.value+':'+read.value[0]);
        })().catch(e=>postMessage('error:'+e.message));"#.to_owned())
    }
}
fn worker_installs_transform_and_byob_algorithms<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.set_script_resource_loader(Box::new(WorkerScript));
    rt.eval("var result='pending';var worker=new Worker('https://streams.test/streams.js');worker.onmessage=e=>result=e.data;").expect("worker");
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        rt.run_microtasks();
        rt.run_timers(64, 0.0);
        rt.pump_workers();
        if read(&mut rt, "result") != "pending" {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(read(&mut rt, "result"), "worker:65");
    rt.eval("worker.terminate();").expect("terminate");
}

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn byte_upload_and_empty_presence() {
            request_upload_collects_exact_bytes_and_preserves_empty_presence::<$backend>();
        }
        #[test]
        fn invalid_upload_is_not_started() {
            invalid_upload_chunk_never_reaches_native_sink::<$backend>();
        }
        #[test]
        fn upload_abort_does_not_register_native_work() {
            abort_during_upload_cancels_reader_without_starting_native_work::<$backend>();
        }
        #[test]
        fn authored_clone_preserves_branch_bytes() {
            live_authored_body_clones_tee_and_preserve_backing_buffer_independence::<$backend>();
        }
        #[test]
        fn retained_child_stream_keeps_weak_brand() {
            retained_child_stream_uses_parent_borrowed_methods_after_destruction::<$backend>();
        }
        #[test]
        fn discarded_streams_do_not_leak() {
            discarded_stream_cycles_are_collected_by_normal_pump::<$backend>();
        }
        #[test]
        fn worker_transform_and_byob() {
            worker_installs_transform_and_byob_algorithms::<$backend>();
        }
    };
}
mod boa {
    use super::*;
    cases!(script_engine_boa::BoaEngine);
}
#[cfg(target_pointer_width = "64")]
mod vano {
    use super::*;
    cases!(script_engine_nova::NovaEngine);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn vano_snapshot_preserves_private_stream_maps_without_sharing_state() {
    let mut donor = runtime::<script_engine_nova::NovaEngine>();
    donor.eval("var controller;var stream=new ReadableStream({start(c){controller=c;c.enqueue('initial');}});").expect("stream");
    donor.run_microtasks();
    let mut clone = donor.snapshot_clone().expect("snapshot");
    clone.eval("controller.enqueue('clone-only');controller.close();var result='';(async()=>{for await(const chunk of stream)result+=chunk+'|';})().catch(e=>result='error:'+e.message);").expect("cloned queue");
    clone.run_microtasks();
    assert_eq!(read(&mut clone, "result"), "initial|clone-only|");
    donor.eval("controller.close();var result='';(async()=>{for await(const chunk of stream)result+=chunk+'|';})().catch(e=>result='error:'+e.message);").expect("donor queue");
    donor.run_microtasks();
    assert_eq!(read(&mut donor, "result"), "initial|");
}
