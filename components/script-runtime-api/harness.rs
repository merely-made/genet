// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The `testharness.js` results bridge: collect per-subtest results out of a
//! loaded harness into host state.
//!
//! `testharness.js` reports completion through `add_completion_callback(cb)`,
//! where `cb(tests, status)` receives the array of `Test` objects (each with
//! `name` / `status` / `message`). The bridge registers such a callback that
//! forwards each result to the `__reportResult` native sink, which records it in
//! [`HostState::results`]. [`Runtime::run_testharness`] drives the whole flow.
//!
//! Engine-neutral like the rest of the host surface (native sink + JS bootstrap,
//! over `CallCx`); validated on Boa + Nova.

use std::cell::RefCell;

use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::HostState;

/// One subtest's outcome, mirrored out of `testharness.js`. `status` is the
/// harness's numeric code: 0 PASS, 1 FAIL, 2 TIMEOUT, 3 NOTRUN,
/// 4 PRECONDITION_FAILED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResult {
    pub name: String,
    pub status: i64,
    pub message: Option<String>,
}

impl TestResult {
    /// Whether this subtest passed (status PASS).
    pub fn passed(&self) -> bool {
        self.status == 0
    }
}

/// Overall testharness completion, independent of individual assertions.
/// A passing partial result array can accompany a TIMEOUT or ERROR status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessCompletion {
    /// The harness's numeric status: 0 OK, 1 ERROR, 2 TIMEOUT.
    pub status: i64,
    pub message: Option<String>,
}

/// Install the `__reportResult` native sink. The completion-callback JS (installed
/// by [`install_bridge`] after `testharness.js` loads) calls it per subtest.
pub(crate) fn install_report_sink<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<ReportResult>("__reportResult", 3)?;
    engine.set_function::<ReportCompletion>("__reportHarnessCompletion", 2)?;
    engine.set_function::<ClearResults>("__clearHarnessResults", 0)
}

/// Register the completion callback on a loaded `testharness.js`. Must run *after*
/// the harness is evaluated (it defines `add_completion_callback`).
///
/// Installed in the realm the harness itself was evaluated in - the top-level
/// browsing context's, not the agent's bootstrap realm - because
/// `add_completion_callback` is a binding of *that* realm's global.
pub(crate) fn bridge_source() -> &'static str {
    BRIDGE_JS
}

/// `__reportResult(name, status, message)` — record one subtest into host state.
/// `status` and `message` arrive as strings (no number-minting primitive yet);
/// `message` is `"null"`/`"undefined"` when the harness has none.
struct ReportResult;
impl<E: ScriptEngine> NativeFn<E> for ReportResult {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let name_v = cx.arg(0);
        let name = cx.value_to_string(&name_v)?;
        let status_v = cx.arg(1);
        let status = cx.value_to_string(&status_v)?.parse::<i64>().unwrap_or(-1);
        let message_v = cx.arg(2);
        let message = match cx.value_to_string(&message_v)?.as_str() {
            "null" | "undefined" | "" => None,
            other => Some(other.to_string()),
        };
        if let Some(data) = cx.host_data() {
            if let Some(cell) = data.downcast_ref::<RefCell<HostState>>() {
                cell.borrow_mut().results.push(TestResult {
                    name,
                    status,
                    message,
                });
            }
        }
        Ok(cx.undefined())
    }
}

struct ClearResults;
impl<E: ScriptEngine> NativeFn<E> for ClearResults {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        if let Some(data) = cx.host_data() {
            if let Some(cell) = data.downcast_ref::<RefCell<HostState>>() {
                cell.borrow_mut().results.clear();
            }
        }
        Ok(cx.undefined())
    }
}

struct ReportCompletion;
impl<E: ScriptEngine> NativeFn<E> for ReportCompletion {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let status_v = cx.arg(0);
        let status = cx.value_to_string(&status_v)?.parse::<i64>().unwrap_or(-1);
        let message_v = cx.arg(1);
        let message = match cx.value_to_string(&message_v)?.as_str() {
            "null" | "undefined" | "" => None,
            other => Some(other.to_string()),
        };
        if let Some(data) = cx.host_data() {
            if let Some(cell) = data.downcast_ref::<RefCell<HostState>>() {
                cell.borrow_mut().harness_completion = Some(HarnessCompletion { status, message });
            }
        }
        Ok(cx.undefined())
    }
}

/// Disables the harness's HTML output (a headless runner reads results
/// programmatically; the output path renders a results table via DOM APIs we don't
/// implement, e.g. `createElementNS`), then registers a completion callback that
/// forwards each subtest to `__reportResult`.
const BRIDGE_JS: &str = r#"
setup({ output: false });
(function(report, clear, complete, stringify) {
  add_result_callback(function(test) {
    report(stringify(test.name), stringify(test.status), stringify(test.message));
  });
  add_completion_callback(function(tests, status) {
    // Replace interim callback order with WPT's final test-array order. This
    // also includes unfinished tests assigned their final timeout status.
    clear();
    for (var i = 0; i < tests.length; i++) {
      report(stringify(tests[i].name), stringify(tests[i].status), stringify(tests[i].message));
    }
    complete(stringify(status.status), stringify(status.message));
  });
})(__reportResult, __clearHarnessResults, __reportHarnessCompletion, String);
"#;
