// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Run the tree's `wasm/jsapi` web platform tests on Bole with
//! `WebAssembly` installed (Genet's WebAssembly object plan, W3 and W5).
//!
//! ```text
//! cargo run -p genet-bole --features wasm --example wasm_jsapi -- <wpt tests dir>
//! ```

#[path = "../../genet-wasm/conformance/mod.rs"]
mod conformance;
#[path = "../tests/harness/mod.rs"]
mod harness;

fn main() {
    conformance::suite::main::<harness::Bole>();
}
