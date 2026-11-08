// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn control<E: ScriptEngine>(runtime: &mut Runtime<E>, body: &str) {
    let script = format!(
        r#"var streamResult = 'pending';
        function check(value, message) {{ if (!value) throw new Error(message); }}
        (async function () {{{body}}})().then(
          () => streamResult = 'ok',
          error => streamResult = error.name + ': ' + error.message);"#
    );
    runtime.eval(&script).expect("stream control evaluates");
    runtime.run_microtasks();
    let result = runtime.eval("streamResult").expect("stream control result");
    assert_eq!(runtime.value_to_string(&result).expect("stringify"), "ok");
}

fn pending_reads_keep_transform_ready<E: ScriptEngine>(constructor: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .eval(&format!("var stream = new {constructor}()"))
        .expect("construct transform");
    control(
        &mut runtime,
        r#"
        const reader = stream.readable.getReader();
        const writer = stream.writable.getWriter();
        const first = reader.read();
        const second = reader.read();
        for (let i = 0; i < 8; i++) await Promise.resolve();
        await writer.write('A');
        await writer.write('B');
        const chunks = [(await first).value, (await second).value];
        const expected = stream instanceof TextEncoderStream ? '65|66' : 'A|B';
        check(chunks.join('|') === expected, 'both pending reads receive their chunks');
        await writer.close();
        check((await reader.read()).done, 'stream closes after queued reads');
        "#,
    );
}

fn streams_follow_constructor_realm_and_share_brands<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://encoding.test/page.html")
        .expect("base URL");
    runtime.parse_document_interleaved("<html><head></head><body></body></html>", &NoScriptLoader);
    runtime
        .eval("globalThis.frame=document.createElement('iframe');frame.srcdoc='<body></body>';document.body.appendChild(frame);")
        .expect("insert same-origin iframe");
    runtime
        .run_event_loop(100)
        .expect("initialize iframe realm");
    control(
        &mut runtime,
        r#"
        const child = frame.contentWindow;
        const encoder = new child.TextEncoderStream();
        const decoder = new child.TextDecoderStream('latin1', {fatal: true});
        const get = (ctor, key) => Object.getOwnPropertyDescriptor(ctor.prototype, key).get;
        const readable = get(TextEncoderStream, 'readable').call(encoder);
        const writable = get(TextEncoderStream, 'writable').call(encoder);
        check(readable.constructor === child.ReadableStream, 'readable belongs to encoder realm');
        check(writable.constructor === child.WritableStream, 'writable belongs to encoder realm');
        check(get(TextDecoderStream, 'encoding').call(decoder) === 'windows-1252', 'borrowed decoder label getter');
        check(get(TextDecoderStream, 'fatal').call(decoder) === true, 'borrowed decoder option getter');

        let rejected = false;
        try { get(TextEncoderStream, 'readable').call(decoder); }
        catch (error) { rejected = error instanceof TypeError; }
        check(rejected, 'encoder getter rejects decoder brand');

        const reader = readable.getReader();
        const writer = writable.getWriter();
        const read = ReadableStreamDefaultReader.prototype.read.call(reader);
        await WritableStreamDefaultWriter.prototype.write.call(writer, 'A');
        const result = await read;
        check(result.value.constructor === child.Uint8Array && result.value[0] === 65,
              'output bytes belong to encoder realm');
        await writer.close();
        "#,
    );
}

macro_rules! cases {
    ($engine:ty) => {
        #[test]
        fn pending_reads_keep_identity_transform_ready() {
            super::pending_reads_keep_transform_ready::<$engine>("TransformStream");
        }

        #[test]
        fn pending_reads_keep_encoder_transform_ready() {
            super::pending_reads_keep_transform_ready::<$engine>("TextEncoderStream");
        }

        #[test]
        fn streams_follow_constructor_realm_and_share_brands() {
            super::streams_follow_constructor_realm_and_share_brands::<$engine>();
        }
    };
}

mod boa {
    cases!(script_engine_boa::BoaEngine);
}

#[cfg(target_pointer_width = "64")]
mod vano {
    cases!(script_engine_nova::NovaEngine);
}
