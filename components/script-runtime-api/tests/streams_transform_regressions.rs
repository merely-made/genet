// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Transform cancellation regressions shared by both script backends.
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn control<E: ScriptEngine>(body: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let script = format!(
        r#"var streamsResult='pending';
        function check(value, message) {{ if (!value) throw new Error(message); }}
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

const READABLE_CANCEL_ERRORS_WRITABLE: &str = r#"
    for (const mode of ['throw', 'reject']) {
        const failure={mode}, reason={cancel:true};
        const ts=new TransformStream({cancel(){
            if(mode==='throw') throw failure;
            return Promise.reject(failure);
        }});
        const writer=ts.writable.getWriter();
        const closed=writer.closed.then(()=>false,error=>error===failure);
        let cancelError;
        try { await ts.readable.cancel(reason); } catch(error) { cancelError=error; }
        check(cancelError===failure,mode+' readable.cancel rejects with exact transformer error');
        check(await closed,mode+' cancellation errors writer.closed with exact transformer error');
    }
"#;

const CANCEL_DURING_FLUSH_REUSES_FINISH: &str = r#"
    let ts, cancelPromise, flushCalled=false, cancelCalled=false;
    ts=new TransformStream({
        flush(){flushCalled=true;cancelPromise=ts.readable.cancel('cancel during flush');},
        cancel(){cancelCalled=true;}
    });
    const closing=ts.writable.close();
    await closing;
    await cancelPromise;
    check(flushCalled,'writable close runs flush');
    check(!cancelCalled,'readable cancellation during flush reuses finish promise');
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn readable_cancel_throw_and_rejection_error_writable() {
            control::<$backend>(READABLE_CANCEL_ERRORS_WRITABLE);
        }

        #[test]
        fn readable_cancel_during_flush_reuses_existing_finish_operation() {
            control::<$backend>(CANCEL_DURING_FLUSH_REUSES_FINISH);
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
