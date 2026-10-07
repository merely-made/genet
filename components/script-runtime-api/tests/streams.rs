// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Observable algorithm controls, shared by both script backends.
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn control<E: ScriptEngine>(body: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let script = format!(
        r#"var streamsResult='pending';
        function check(value, message) {{ if (!value) throw new Error(message); }}
        function deferred() {{ let resolve; const promise=new Promise(r=>resolve=r); return {{promise,resolve}}; }}
        async function flush() {{ for(let i=0;i<16;i++) await Promise.resolve(); }}
        (async function(){{{body}}})().then(
            ()=>streamsResult='ok', e=>streamsResult='error:'+e.name+':'+e.message);
        "#
    );
    runtime.eval(&script).expect("control evaluates");
    runtime.run_microtasks();
    let result = runtime.eval("streamsResult").expect("control result");
    assert_eq!(runtime.value_to_string(&result).expect("stringify"), "ok");
}

const BACKPRESSURE: &str = r#"
    const gate=deferred(); let started=false;
    const writer=new WritableStream({write(){started=true;return gate.promise;}},
        {highWaterMark:1,size(){return 1;}}).getWriter();
    const write=writer.write('chunk'); let ready=false;
    const pendingReady=writer.ready; pendingReady.then(()=>ready=true);
    await flush();
    check(started,'sink write starts');
    check(writer.desiredSize===0,'queued write fills high-water mark');
    check(!ready,'ready stays pending during deferred write');
    gate.resolve(); await write; await pendingReady;
    check(writer.desiredSize===1,'fulfilled write restores desired size');
"#;

const CANCELLATION: &str = r#"
    const gate=deferred(),reason={cancel:true}; let received;
    const reader=new ReadableStream({cancel(r){received=r;return gate.promise;}}).getReader();
    let readState='pending',closed=false,cancelled=false;
    const read=reader.read(); read.then(()=>readState='fulfilled',()=>readState='rejected');
    reader.closed.then(()=>closed=true);
    await flush();
    check(readState==='pending','empty open stream leaves read pending');
    check(!closed,'closed remains pending before cancellation');
    const cancel=reader.cancel(reason); cancel.then(()=>cancelled=true);
    const result=await read; await reader.closed; await flush();
    check(result.done && result.value===undefined,'cancel completes pending read');
    check(closed && received===reason,'cancel closes reader and preserves reason identity');
    check(!cancelled,'cancel waits for underlying source');
    gate.resolve(); await cancel;
"#;

const PIPE_ERROR: &str = r#"
    const failure=new Error('write failure'); let cancelled;
    const source=new ReadableStream({start(c){c.enqueue('one');},cancel(r){cancelled=r;}});
    const sink=new WritableStream({write(){return Promise.reject(failure);}});
    let rejected;
    try { await source.pipeTo(sink); } catch(e) { rejected=e; }
    check(rejected===failure,'pipe rejects with exact sink error');
    check(cancelled===failure,'pipe cancels source with sink error');
    check(!source.locked && !sink.locked,'pipe releases both locks');
"#;

const BYOB: &str = r#"
    let pulls=0;
    const stream=new ReadableStream({type:'bytes',pull(c){
        if(++pulls===1){const r=c.byobRequest;r.view[0]=17;r.view[1]=34;r.view[2]=51;r.respond(3);}
        else c.close();
    }});
    const reader=stream.getReader({mode:'byob'});
    const input=new Uint8Array(5), original=input.buffer;
    const result=await reader.read(input);
    check(original.byteLength===0 && input.byteLength===0,'BYOB detaches supplied buffer');
    check(!result.done && result.value instanceof Uint8Array,'BYOB returns filled view');
    check(result.value.byteLength===3 && String(result.value)==='17,34,51','BYOB returns exact responded bytes');
    const end=await reader.read(new Uint8Array(1));
    check(end.done && end.value.byteLength===0,'BYOB close returns an empty view');
    reader.releaseLock(); check(!stream.locked,'BYOB reader releases its lock');
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        #[ignore = "starting implementation negative control; candidate enables this test"]
        fn deferred_write_backpressure() {
            control::<$backend>(BACKPRESSURE);
        }
        #[test]
        #[ignore = "starting implementation negative control; candidate enables this test"]
        fn cancellation_waits_for_source() {
            control::<$backend>(CANCELLATION);
        }
        #[test]
        #[ignore = "starting implementation negative control; candidate enables this test"]
        fn pipe_error_cancels_source_and_releases_locks() {
            control::<$backend>(PIPE_ERROR);
        }
        #[test]
        #[ignore = "starting implementation negative control; candidate enables this test"]
        fn byob_fill_detaches_and_returns_exact_view() {
            control::<$backend>(BYOB);
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
