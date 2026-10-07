// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Focused observable regressions for ReadableStream byte-controller validation.
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn control<E: ScriptEngine>(body: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let script = format!(
        r#"var streamsReadableResult='pending';
        function check(value,message){{if(!value)throw new Error(message);}}
        (async function(){{{body}}})().then(
            ()=>streamsReadableResult='ok',
            error=>streamsReadableResult='error:'+error.name+':'+error.message);
        "#
    );
    runtime.eval(&script).expect("control evaluates");
    runtime.run_microtasks();
    let result = runtime
        .eval("streamsReadableResult")
        .expect("control result");
    assert_eq!(runtime.value_to_string(&result).expect("stringify"), "ok");
}

const EMPTY_NEW_VIEW_VALIDATION_ORDER: &str = r#"
    let pulls=0;
    const stream=new ReadableStream({type:'bytes',pull(controller){
        if(pulls++)return;
        const request=controller.byobRequest;
        let thrown;
        try{request.respondWithNewView(new Uint8Array());}catch(error){thrown=error;}
        check(thrown instanceof TypeError,
            'readable-state empty new view must throw TypeError before buffer-length RangeError');
        request.view[0]=42;
        request.respond(1);
        controller.close();
    }});
    const reader=stream.getReader({mode:'byob'});
    const result=await reader.read(new Uint8Array(3));
    check(!result.done && result.value.byteLength===1 && result.value[0]===42,
        'BYOB request remains usable after rejected zero-length new view');
    const end=await reader.read(new Uint8Array(1));
    check(end.done && end.value.byteLength===0,'source closes with an empty final view');
    reader.releaseLock();
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn readable_state_empty_new_view_throws_typeerror_before_rangechecks() {
            control::<$backend>(EMPTY_NEW_VIEW_VALIDATION_ORDER);
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
