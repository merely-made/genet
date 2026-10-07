// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Regressions for pipeTo brands, pipeThrough lock checks, and pipe scheduling.
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn control<E: ScriptEngine>(body: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let script = format!(
        r#"var streamsResult='pending';
        function check(value, message) {{ if (!value) throw new Error(message); }}
        async function flush() {{ for (let i = 0; i < 16; ++i) await Promise.resolve(); }}
        (async function() {{{body}}})().then(
            () => streamsResult='ok',
            error => streamsResult='error:' + error.name + ':' + error.message);
        "#
    );
    runtime.eval(&script).expect("control evaluates");
    runtime.run_microtasks();
    let result = runtime.eval("streamsResult").expect("control result");
    assert_eq!(runtime.value_to_string(&result).expect("stringify"), "ok");
}

const PIPE_TO_BRANDS_REJECT: &str = r#"
    for (const [source, destination] of [
        [Object.create(ReadableStream.prototype), new WritableStream()],
        [new ReadableStream(), Object.create(WritableStream.prototype)]
    ]) {
        let returned, thrown;
        try { returned = ReadableStream.prototype.pipeTo.call(source, destination); }
        catch (error) { thrown = error; }
        check(thrown === undefined, 'pipeTo brand failures return a promise');
        check(returned instanceof Promise, 'pipeTo returns a promise for brand failures');
        let rejection;
        try { await returned; } catch (error) { rejection = error; }
        check(rejection instanceof TypeError, 'pipeTo rejects with TypeError');
    }
"#;

const PIPE_TO_CONVERSION_REJECTS: &str = r#"
    const reason = { conversion: true };
    let returned, thrown;
    try {
        returned = new ReadableStream().pipeTo(new WritableStream(), {
            get signal() { throw reason; }
        });
    } catch (error) { thrown = error; }
    check(thrown === undefined, 'pipeTo converts binding errors to rejected promises');
    check(returned instanceof Promise, 'pipeTo returns a promise for option conversion errors');
    let rejection;
    try { await returned; } catch (error) { rejection = error; }
    check(rejection === reason, 'pipeTo preserves the option getter error identity');

    let accessed = false;
    let invalidReceiverResult;
    try {
        invalidReceiverResult = ReadableStream.prototype.pipeTo.call(
            {}, new WritableStream(), {
                get preventClose() { accessed = true; return false; }
            });
    } catch (error) { thrown = error; }
    check(thrown === undefined, 'invalid pipeTo receiver still returns a promise');
    check(!accessed, 'pipeTo checks receiver before converting option getters');
    let receiverRejection;
    try { await invalidReceiverResult; } catch (error) { receiverRejection = error; }
    check(receiverRejection instanceof TypeError, 'invalid receiver rejects with TypeError');
"#;

const PIPE_THROUGH_LOCKS_THROW: &str = r#"
    const lockedSource = new ReadableStream();
    lockedSource.getReader();
    let sourceError;
    try {
        lockedSource.pipeThrough({ readable: new ReadableStream(), writable: new WritableStream() });
    } catch (error) { sourceError = error; }
    check(sourceError instanceof TypeError, 'pipeThrough synchronously rejects a locked source');

    const source = new ReadableStream();
    const writable = new WritableStream();
    writable.getWriter();
    let destinationError;
    try { source.pipeThrough({ readable: new ReadableStream(), writable }); }
    catch (error) { destinationError = error; }
    check(destinationError instanceof TypeError, 'pipeThrough synchronously rejects a locked destination');
    check(!source.locked, 'locked destination leaves source unlocked');
"#;

const PIPE_WRITE_IS_DEFERRED: &str = r#"
    let controller, called = false;
    const source = new ReadableStream({ start(value) { controller = value; } }, { highWaterMark: 0 });
    const destination = new WritableStream({ write() { called = true; } });
    source.pipeTo(destination);
    await flush();
    check(!called, 'sink has no write before a source chunk is enqueued');
    controller.enqueue('chunk');
    check(!called, 'enqueue does not synchronously run the sink write algorithm');
    await flush();
    check(called, 'pipe schedules the write after enqueue returns');
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn pipe_to_brand_failures_reject_as_promises() {
            control::<$backend>(PIPE_TO_BRANDS_REJECT);
        }

        #[test]
        fn pipe_to_binding_errors_reject_and_preserve_receiver_order() {
            control::<$backend>(PIPE_TO_CONVERSION_REJECTS);
        }

        #[test]
        fn pipe_through_locked_streams_throw_synchronously() {
            control::<$backend>(PIPE_THROUGH_LOCKS_THROW);
        }

        #[test]
        fn pipe_write_runs_after_enqueue_returns() {
            control::<$backend>(PIPE_WRITE_IS_DEFERRED);
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
