// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Observable algorithm controls, shared by both script backends.
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn control<E: ScriptEngine>(body: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let script = format!(
        r#"var streamsResult='pending',streamsPhase='start';
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
    let phase = runtime.eval("streamsPhase").expect("control phase");
    assert_eq!(
        runtime.value_to_string(&result).expect("stringify"),
        "ok",
        "phase: {}",
        runtime.value_to_string(&phase).expect("stringify phase")
    );
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
        else {const r=c.byobRequest;c.close();r.respond(0);}
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

const REACTION_PROPERTIES: &str = r#"
    const originalPromise=Promise;
    for(const name of ['ReadableStream','WritableStream','TransformStream']) {
      for(const property of ['constructor','species']) {
        const target=property==='constructor'?originalPromise.prototype:originalPromise;
        const key=property==='constructor'?'constructor':Symbol.species;
        const descriptor=Object.getOwnPropertyDescriptor(target,key);let accesses=0,thrown;
        try {
          Object.defineProperty(target,key,{get(){++accesses;return originalPromise;},configurable:true});
          try{new globalThis[name]();}catch(error){thrown=error;}
        } finally {Object.defineProperty(target,key,descriptor);}
        check(thrown===undefined,name+' default construction succeeds');
        check(accesses===0,name+' internal setup inspected Promise '+property+' '+accesses+' times');
      }
    }
"#;

const PRIVATE_BRANDS: &str = r#"
    let controller;
    const stream=new ReadableStream({start(c){controller=c;c.enqueue('real');}});
    stream._chunks=['forged'];stream._disturbed=false;stream._reader=null;
    Object.freeze(stream);Object.freeze(controller);
    const reader=stream.getReader();Object.freeze(reader);
    const read=reader.read();controller.close();
    check((await read).value==='real','public legacy fields do not replace private queue');
    check((await reader.read()).done,'frozen controller can close through private state');
    reader.releaseLock();check(!stream.locked,'frozen reader can release its private lock');
    let rejected=false;try{ReadableStream.prototype.getReader.call({_chunks:[]});}catch(e){rejected=e instanceof TypeError;}
    check(rejected,'lookalike object has no stream brand');
    check(typeof globalThis.__streamsFetch==='undefined' && typeof globalThis.__finishStreamsClone==='undefined' &&
        !Object.prototype.hasOwnProperty.call(globalThis,'__streamsPerformPromiseThen'),
        'private installation handoffs are absent before authors');
"#;

const AUTHORED_REACTION_ERRORS: &str = r#"
    function throwingPromise(reason) {
        const promise=Promise.resolve();
        Object.defineProperty(promise,'constructor',{get(){throw reason;}});
        return promise;
    }
    for(const bytes of [false,true]) {
        const reason={pull:bytes},source={pull(){return throwingPromise(reason);}};
        if(bytes)source.type='bytes';
        const reader=new ReadableStream(source,{highWaterMark:0}).getReader();
        const closed=reader.closed.then(()=>false,error=>error===reason);
        let rejected;try{await reader.read();}catch(error){rejected=error;}
        check(rejected===reason && await closed,'pull conversion errors reject read and closed exactly');
    }
    const reason={iterator:true},reader=ReadableStream.from([throwingPromise(reason)]).getReader();
    const closed=reader.closed.then(()=>false,error=>error===reason);
    let rejected;try{await reader.read();}catch(error){rejected=error;}
    check(rejected===reason && await closed,'sync iterator Promise conversion errors finish the read operation');
"#;

const BYTE_TEE_CANCELLATION: &str = r#"
    for(const canceled of [0,1]) {
        let controller;
        const stream=new ReadableStream({type:'bytes',start(c){controller=c;}});
        const branches=stream.tee(),readers=branches.map(branch=>branch.getReader({mode:'byob'}));
        const first=readers[0].read(new Uint8Array([17]));await flush();
        const second=readers[1].read(new Uint8Array([34]));await flush();
        const reads=[first,second],order=[];
        readers[canceled].closed.then(()=>order.push('closed'));
        reads[canceled].then(()=>order.push('read'));
        const cancel=readers[canceled].cancel('unused');
        streamsPhase='cancel read '+canceled;
        const canceledRead=await reads[canceled];
        check(canceledRead.done && canceledRead.value===undefined,'canceled tee branch settles its pending BYOB read without a view');
        check(String(order)==='closed,read','cancel closes the reader before completing pending BYOB reads');
        controller.byobRequest.view[0]=51;controller.byobRequest.respond(1);
        streamsPhase='remaining read '+canceled;
        const item=await reads[1-canceled];
        check(!item.done && item.value[0]===51,'remaining tee branch receives bytes after other branch cancels');
        controller.close();
        streamsPhase='remaining EOF '+canceled;
        check((await readers[1-canceled].read(new Uint8Array(1))).done,'remaining tee branch observes source EOF');
        streamsPhase='cancel completion '+canceled;
        await cancel;
    }
"#;

const READER_RELEASE: &str = r#"
    for(const bytes of [false,true]) for(const closed of [false,true]) {
        const source={start(c){if(closed)c.close();}};
        if(bytes)source.type='bytes';
        const stream=new ReadableStream(source);
        const reader=stream.getReader(bytes?{mode:'byob'}:undefined),before=reader.closed;
        reader.releaseLock();
        const after=reader.closed;
        check(after instanceof Promise && !stream.locked,'released reader keeps its closed Promise and frees the lock');
        check(closed?before!==after:before===after,'closed Promise identity follows the prior stream state');
        let rejected;try{await after;}catch(error){rejected=error;}
        check(rejected instanceof TypeError,'released reader closed Promise rejects with TypeError');
        if(closed)await before;
        rejected=undefined;try{await reader.read(bytes?new Uint8Array(1):undefined);}catch(error){rejected=error;}
        check(rejected instanceof TypeError,'released reader rejects new reads');
    }
"#;

const FETCH_POISONED_PROTOTYPE: &str = r#"
    const oldType=Object.getOwnPropertyDescriptor(Object.prototype,'type');
    const oldThen=Object.getOwnPropertyDescriptor(Object.prototype,'then');
    const stream=new ReadableStream({start(c){c.enqueue(new Uint8Array([65,66]));c.close();}});
    let text;
    try {
        Object.defineProperty(Object.prototype,'type',{configurable:true,get(){throw new Error('type trap');},
            set(){throw new Error('type setter trap');}});
        Object.defineProperty(Object.prototype,'then',{configurable:true,get(){throw new Error('then trap');}});
        text=await new Response(stream).text();
        check(await new Response('AB').text()==='AB','private buffered byte stream avoids inherited source getters');
    } finally {
        if(oldType)Object.defineProperty(Object.prototype,'type',oldType);else delete Object.prototype.type;
        if(oldThen)Object.defineProperty(Object.prototype,'then',oldThen);else delete Object.prototype.then;
    }
    check(text==='AB','Fetch creates and consumes a body without inherited type/then calls');
    const empty=new Response(null);check(await empty.text()==='' && !empty.bodyUsed,'null body remains undisturbed');
    check(await empty.text()==='' && !empty.bodyUsed,'null body can be consumed again');
"#;

const WRITABLE_ABORT_REENTRY: &str = r#"
    const outer={outer:true},inner={inner:true};let received,signal,recursive;
    const writer=new WritableStream({start(c){signal=c.signal;},abort(r){received=r;}}).getWriter();
    signal.addEventListener('abort',()=>{recursive=writer.abort(inner);});
    const abort=writer.abort(outer);
    check(abort===recursive,'reentrant abort reserves the shared abort promise');
    check(signal.aborted && signal.reason===outer,'signal retains the first reason');
    await abort;check(received===inner,'sink uses the inner reserved abort request reason');
"#;

const TRANSFORM_PRESSURE_AND_PIPE_CLOSING: &str = r#"
    const transform=new TransformStream(undefined,undefined,{highWaterMark:0});
    const writer=transform.writable.getWriter();let written=false;
    const write=writer.write('one');write.then(()=>written=true);
    await flush();check(!written,'transform write waits for readable demand');
    const reader=transform.readable.getReader();check((await reader.read()).value==='one','read pulls the pending transform');
    await write;const close=writer.close();check((await reader.read()).done,'flush closes transform readable');await close;
    let sourceCancelled;const closeGate=deferred();
    const source=new ReadableStream({cancel(reason){sourceCancelled=reason;}});
    const dest=new WritableStream({close(){return closeGate.promise;}});
    // Closing before pipeTo must reject/cancel even while its sink close is pending.
    const closing=dest.close();let error;
    try{await source.pipeTo(dest);}catch(e){error=e;}
    check(error instanceof TypeError && sourceCancelled===error,'closing destination cancels source with exact pipe error');
    check(!source.locked && !dest.locked,'closing shutdown releases both locks');
    closeGate.resolve();await closing;
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn deferred_write_backpressure() {
            control::<$backend>(BACKPRESSURE);
        }
        #[test]
        fn cancellation_waits_for_source() {
            control::<$backend>(CANCELLATION);
        }
        #[test]
        fn pipe_error_cancels_source_and_releases_locks() {
            control::<$backend>(PIPE_ERROR);
        }
        #[test]
        fn byob_fill_detaches_and_returns_exact_view() {
            control::<$backend>(BYOB);
        }
        #[test]
        fn internal_reactions_do_not_observe_promise_constructor_or_species() {
            control::<$backend>(REACTION_PROPERTIES);
        }
        #[test]
        fn frozen_objects_and_public_forgeries_preserve_private_brands() {
            control::<$backend>(PRIVATE_BRANDS);
        }
        #[test]
        fn authored_promise_conversion_errors_settle_stream_operations() {
            control::<$backend>(AUTHORED_REACTION_ERRORS);
        }
        #[test]
        fn byte_tee_uses_current_cancellation_state_when_a_read_finishes() {
            control::<$backend>(BYTE_TEE_CANCELLATION);
        }
        #[test]
        fn releasing_readers_preserves_closed_promise_semantics() {
            control::<$backend>(READER_RELEASE);
        }
        #[test]
        fn fetch_body_consumption_survives_inherited_type_and_then_traps() {
            control::<$backend>(FETCH_POISONED_PROTOTYPE);
        }
        #[test]
        fn writable_abort_reentry_preserves_signal_and_inner_request_reasons() {
            control::<$backend>(WRITABLE_ABORT_REENTRY);
        }
        #[test]
        fn transform_backpressure_and_pipe_closing_shutdown() {
            control::<$backend>(TRANSFORM_PRESSURE_AND_PIPE_CLOSING);
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
