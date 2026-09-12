// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The engine boundary that decides where the top-level browsing context's
//! realm may live.
//!
//! Separating the agent's bootstrap realm from the top document's realm means
//! `Runtime::new` creates a realm before the embedder has run a line of script.
//! `snapshot_clone` is how `NovaHarnessTemplate` gives every WPT test a fresh
//! heap without re-evaluating `testharness.js`, and the Nova adapter used to
//! refuse to clone an agent that had ever created a realm - so that refusal,
//! not any question of HTML, is what this design had to answer first. It was
//! lifted (see `script-engine-nova`'s `snapshot_carries_every_realm`); this
//! measures the consequence at the `Runtime` level, which is the level the
//! harness template actually uses.
//!
//! Measured with its positive control in the same run: a runtime whose top
//! document is a realm of its own clones, and the clone's realms are its own.

#![cfg(target_pointer_width = "64")]

use script_engine_api::{MAIN_REALM, ScriptEngine as _};
use script_engine_nova::NovaEngine;
use script_runtime_api::Runtime;

#[test]
fn a_runtime_with_a_separate_top_realm_still_snapshot_clones() {
    let mut runtime: Runtime<NovaEngine> = Runtime::new().expect("nova runtime");

    // The premise: the top document is not the bootstrap realm.
    let top = runtime.top_realm();
    assert_ne!(
        top, MAIN_REALM,
        "the top document's realm is created through the realm API"
    );

    // Positive control: the clone the WPT harness template depends on.
    let mut clone = runtime
        .snapshot_clone()
        .expect("two-realm agent must clone");

    // The clone shows the same top realm, and it is a live realm in the clone's
    // own heap - not a dangling index into the donor's.
    assert_eq!(clone.top_realm(), top);
    clone.eval("globalThis.__fromClone = 7").expect("eval");
    let value = clone.eval("globalThis.__fromClone").expect("read back");
    assert_eq!(clone.engine_mut().value_to_string(&value).unwrap(), "7");

    // And the two agents are separate: what the clone wrote is not in the donor.
    let value = runtime
        .eval("typeof globalThis.__fromClone")
        .expect("donor");
    assert_eq!(
        runtime.engine_mut().value_to_string(&value).unwrap(),
        "undefined",
        "the clone's top realm must not be the donor's"
    );
}
