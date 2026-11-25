// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Bole as the shared WebAssembly tests' engine, for this crate's test and its
//! `wasm_jsapi` example.

use script_engine_api::ScriptEngine;
use script_engine_boa::BoaEngine;

use super::conformance::Harness;

pub struct Bole(pub BoaEngine);

impl Harness for Bole {
    fn new() -> Self {
        let mut engine = BoaEngine::new().unwrap();
        engine.install_webassembly().unwrap();
        Self(engine)
    }

    fn try_eval(&mut self, source: &str) -> Result<String, String> {
        let value = self.0.eval(source).map_err(|e| e.to_string())?;
        self.0.value_to_string(&value).map_err(|e| e.to_string())
    }

    fn pump(&mut self) {
        self.0.pump_microtasks();
    }

    fn set_fuel(&mut self, fuel: Option<u64>) {
        self.0.set_wasm_fuel(fuel).unwrap();
    }
}
