// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exercise overall completion with the real vendored WPT harness.

use std::path::PathBuf;

use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn harness_source() -> String {
    let root = std::env::var_os("GENET_WPT_TESTS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wpt/tests"));
    let path = root.join("resources/testharness.js");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("WPT harness required at {}: {error}", path.display()))
}

fn normal_completion_and_reset<E: ScriptEngine>() {
    let source = harness_source();
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let results = runtime
        .run_testharness(&source, "test(function(){assert_true(true);}, 'complete');")
        .expect("normal test");
    assert_eq!(results.len(), 1);
    assert!(results[0].passed());
    assert_eq!(runtime.harness_completion().expect("completion").status, 0);

    let results = runtime
        .run_testharness(
            &source,
            "setup({explicit_done:true,explicit_timeout:true});test(function(){assert_true(true);},'awaiting done');",
        )
        .expect("uncompleted test");
    assert_eq!(
        results.len(),
        1,
        "retain the observed assertion without completion"
    );
    assert!(results[0].passed());
    assert!(
        runtime.harness_completion().is_none(),
        "old completion must not survive reset"
    );
}

fn timeout_after_passing_assertion<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let results = runtime
        .run_testharness(
            &harness_source(),
            "setup({explicit_done:true,timeout_multiplier:0.001});test(function(){assert_true(true);},'pass before timeout');",
        )
        .expect("timed-out harness");
    assert_eq!(results.len(), 1);
    assert!(results[0].passed());
    assert_eq!(
        runtime
            .harness_completion()
            .expect("timeout completion")
            .status,
        2
    );
}

fn error_after_passing_assertion<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let results = runtime
        .run_testharness(
            &harness_source(),
            "setup({explicit_done:true});test(function(){assert_true(true);return 1;},'pass with invalid returned value');done();",
        )
        .expect("errored harness");
    assert_eq!(results.len(), 1);
    assert!(results[0].passed());
    assert_eq!(
        runtime
            .harness_completion()
            .expect("error completion")
            .status,
        1
    );
}

fn captured_completion_sink<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let results = runtime
        .run_testharness(
            &harness_source(),
            "__reportHarnessCompletion=function(){throw new Error('replaced completion sink');};__clearHarnessResults=function(){throw new Error('replaced clear sink');};__reportResult=function(){throw new Error('replaced result sink');};test(function(){assert_true(true);},'captured sinks');",
        )
        .expect("captured bridge");
    assert_eq!(results.len(), 1);
    assert!(results[0].passed());
    assert_eq!(
        runtime
            .harness_completion()
            .expect("captured completion")
            .status,
        0
    );
}

fn final_result_order<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    let results = runtime
        .run_testharness(
            &harness_source(),
            "var first=async_test('declared first');setTimeout(first.step_func_done(function(){assert_true(true);}),1);test(function(){assert_true(true);},'completed first');",
        )
        .expect("out-of-order assertions");
    assert_eq!(results.len(), 2, "completion must replace interim reports");
    assert_eq!(results[0].name, "declared first");
    assert_eq!(results[1].name, "completed first");
    assert!(results.iter().all(|result| result.passed()));
    assert_eq!(runtime.harness_completion().expect("completion").status, 0);
}

macro_rules! both_engines {
    ($body:ident, $boa:ident, $vano:ident) => {
        #[test]
        fn $boa() {
            $body::<script_engine_boa::BoaEngine>();
        }
        #[test]
        fn $vano() {
            $body::<script_engine_nova::NovaEngine>();
        }
    };
}

both_engines!(
    normal_completion_and_reset,
    normal_and_reset_on_boa,
    normal_and_reset_on_vano
);
both_engines!(
    timeout_after_passing_assertion,
    timeout_on_boa,
    timeout_on_vano
);
both_engines!(error_after_passing_assertion, error_on_boa, error_on_vano);
both_engines!(
    captured_completion_sink,
    captured_sinks_on_boa,
    captured_sinks_on_vano
);
both_engines!(final_result_order, final_order_on_boa, final_order_on_vano);
