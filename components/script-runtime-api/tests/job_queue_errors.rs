// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! An engine limit raised inside a promise job (runaway recursion, which no
//! script can catch) must not silently take the other queued jobs with it, and
//! must reach the page's `error` event instead of vanishing.

use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

const SCRIPT: &str = r#"
    globalThis.order = [];
    globalThis.errors = [];
    addEventListener('error', function (e) { errors.push(String(e.message)); });
    function recurse() { return recurse() + 1; }
    Promise.resolve().then(function () { order.push('a'); });
    Promise.resolve().then(recurse);
    Promise.resolve().then(function () { order.push('c'); });
    Promise.resolve().then(function () { order.push('d'); });
"#;

/// Run [`SCRIPT`] and return what survived: the jobs that ran and the `error` events seen.
fn run<E: ScriptEngine>() -> (String, String) {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.eval(SCRIPT).expect("script");
    rt.run_microtasks();
    rt.eval("console.log(order.join()); console.log(errors.join(' / '));")
        .expect("report");
    let console = rt.host().borrow().console.clone();
    let tail = console.len().saturating_sub(2);
    (console[tail].clone(), console[tail + 1].clone())
}

#[test]
fn uncatchable_job_error_keeps_the_other_jobs_and_is_reported_on_boa() {
    let (order, errors) = run::<script_engine_boa::BoaEngine>();
    assert_eq!(order, "a,c,d", "jobs queued after the failing one must still run");
    assert!(
        errors.contains("Job queue error"),
        "the lost-to-a-limit job must surface as a window error, got {errors:?}"
    );
}

// Nova turns the same recursion into a catchable RangeError, so the job merely
// rejects: the twin shows the page-visible result agrees and nothing is invented.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn runaway_recursion_job_leaves_the_other_jobs_alone_on_nova() {
    let (order, errors) = run::<script_engine_nova::NovaEngine>();
    assert_eq!(order, "a,c,d");
    assert!(!errors.contains("Job queue error"), "got {errors:?}");
}
