// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Helpers the WebAssembly tests share.

use script_engine_api::ScriptEngine;
use script_engine_nova::NovaEngine;

/// An engine with `WebAssembly` installed and `bytes` bound to the module.
pub fn engine_with(wat: &str) -> NovaEngine {
    let mut engine = NovaEngine::new().unwrap();
    engine.install_webassembly().unwrap();
    let bytes = wat::parse_str(wat).unwrap();
    let list: Vec<String> = bytes.iter().map(u8::to_string).collect();
    engine
        .eval(&format!(
            "globalThis.bytes = new Uint8Array([{}]);",
            list.join(",")
        ))
        .unwrap();
    engine
}

/// Evaluate `body` as a function body; what it returns, or what it threw.
pub fn run(engine: &mut NovaEngine, body: &str) -> String {
    let source = format!(
        "(() => {{ try {{ return String((() => {{ {body} }})()); }} \
         catch (e) {{ return 'threw ' + (e && e.name) + ': ' + (e && e.message); }} }})()"
    );
    let value = engine.eval(&source).unwrap();
    engine.value_to_string(&value).unwrap()
}
