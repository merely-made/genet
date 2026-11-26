// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `design_docs/archive_docs/2026-11-25/2026-11-22_webassembly_object_plan.md` W5: the shared tests in
//! `genet-wasm/conformance`, on Bole.
#![cfg(all(feature = "wasm", not(target_family = "wasm")))]

#[path = "../../genet-wasm/conformance/mod.rs"]
mod conformance;

#[path = "harness/mod.rs"]
mod harness;

use harness::Bole;

conformance::conformance_tests!(Bole);
