// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Vano as the shared WebAssembly tests' engine, for this crate's test and its
//! `wasm_jsapi` example.

use script_engine_api::ScriptEngine;
use script_engine_nova::NovaEngine;

use super::conformance::Harness;

pub struct Vano(pub NovaEngine);

impl Harness for Vano {
    fn new() -> Self {
        let mut engine = NovaEngine::new().unwrap();
        engine.install_webassembly().unwrap();
        Self(engine)
    }

    fn try_eval(&mut self, source: &str) -> Result<String, String> {
        let value = self.0.eval(source)?;
        self.0.value_to_string(&value)
    }

    fn pump(&mut self) {
        self.0.pump_microtasks();
    }

    fn set_fuel(&mut self, fuel: Option<u64>) {
        self.0.set_wasm_fuel(fuel).unwrap();
    }

    fn collect(&mut self) {
        for _ in 0..3 {
            self.0.force_gc();
            self.0.pump_microtasks();
        }
    }

    fn stats(&mut self) -> genet_wasm::Stats {
        self.0.wasm_stats().expect("WebAssembly is installed")
    }
}
