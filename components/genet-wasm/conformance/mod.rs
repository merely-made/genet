// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The WebAssembly object's tests, shared by every backend (Genet's
//! WebAssembly object plan, W5): W1 and W2 in `calls.rs`, W4 in `entities.rs`,
//! SIMD and 64-bit addresses in `wide.rs`.
//! A backend includes this module in a test crate, implements [`Harness`] for
//! its engine and invokes `conformance_tests!` with it; `suite` runs the
//! tree's web platform tests the same way.

#![allow(dead_code)]

pub mod calls;
pub mod entities;
pub mod freeing;
pub mod suite;
pub mod wide;

/// What the tests need from an engine with `WebAssembly` installed.
pub trait Harness: Sized {
    /// An engine with the namespace installed.
    fn new() -> Self;
    /// Evaluate `source`; its completion value as a string, or the error.
    fn try_eval(&mut self, source: &str) -> Result<String, String>;
    /// Run pending microtasks.
    fn pump(&mut self);
    /// Set the wasm fuel budget.
    fn set_fuel(&mut self, fuel: Option<u64>);
    /// Collect garbage, and run the jobs it queues (`FinalizationRegistry`
    /// cleanups), until nothing more is freed.
    fn collect(&mut self);
    /// What the wasm store holds.
    fn stats(&mut self) -> genet_wasm::Stats;

    /// An engine with `bytes` bound to `wat`'s module as a `Uint8Array`.
    fn with_module(wat: &str) -> Self {
        let mut engine = Self::new();
        engine.eval(&bytes_script(wat));
        engine
    }

    /// Evaluate `body` as a function body; what it returns, or what it threw.
    fn run(&mut self, body: &str) -> String {
        self.try_eval(&run_script(body))
            .expect("the wrapper catches")
    }

    /// Evaluate a script for its effects.
    fn eval(&mut self, source: &str) {
        self.try_eval(source).expect("script evaluates");
    }
}

/// The source `with_module` binds `bytes` with.
pub fn bytes_script(wat: &str) -> String {
    let bytes = wat::parse_str(wat).expect("test module parses");
    let list: Vec<String> = bytes.iter().map(u8::to_string).collect();
    format!("globalThis.bytes = new Uint8Array([{}]);", list.join(","))
}

/// The wrapper `run` evaluates: `body`'s return value as a string, or what
/// it threw.
pub fn run_script(body: &str) -> String {
    format!(
        "(() => {{ try {{ return String((() => {{ {body} }})()); }} \
         catch (e) {{ return 'threw ' + (e && e.name) + ': ' + (e && e.message); }} }})()"
    )
}

/// One `#[test]` per shared test, for the harness type given.
#[allow(unused_macros)]
macro_rules! conformance_tests {
    ($harness:ty) => {
        #[test]
        fn an_export_calls_an_imported_script_function() {
            conformance::calls::an_export_calls_an_imported_script_function::<$harness>();
        }
        #[test]
        fn i64_crosses_as_bigint_both_ways() {
            conformance::calls::i64_crosses_as_bigint_both_ways::<$harness>();
        }
        #[test]
        fn compile_link_and_runtime_errors_have_their_classes() {
            conformance::calls::compile_link_and_runtime_errors_have_their_classes::<$harness>();
        }
        #[test]
        fn a_runaway_module_is_cut_off_and_the_engine_survives() {
            conformance::calls::a_runaway_module_is_cut_off_and_the_engine_survives::<$harness>();
        }
        #[test]
        fn script_called_from_wasm_can_call_back_into_the_same_instance() {
            conformance::calls::script_called_from_wasm_can_call_back_into_the_same_instance::<$harness>();
        }
        #[test]
        fn an_import_exception_passes_through_and_the_store_survives() {
            conformance::calls::an_import_exception_passes_through_and_the_store_survives::<$harness>();
        }
        #[test]
        fn a_start_function_calls_script_during_instantiation() {
            conformance::calls::a_start_function_calls_script_during_instantiation::<$harness>();
        }
        #[test]
        fn an_import_reached_without_a_wasm_frame_returns_its_result() {
            conformance::calls::an_import_reached_without_a_wasm_frame_returns_its_result::<$harness>();
        }
        #[test]
        fn the_namespace_and_its_promises() {
            conformance::calls::the_namespace_and_its_promises::<$harness>();
        }
        #[test]
        fn an_exported_memory_buffer_is_the_linear_memory() {
            conformance::calls::an_exported_memory_buffer_is_the_linear_memory::<$harness>();
        }
        #[test]
        fn growth_from_wasm_detaches_the_old_buffer() {
            conformance::calls::growth_from_wasm_detaches_the_old_buffer::<$harness>();
        }
        #[test]
        fn growth_from_script_detaches_even_by_zero_pages() {
            conformance::calls::growth_from_script_detaches_even_by_zero_pages::<$harness>();
        }
        #[test]
        fn a_memory_made_by_script_is_imported_and_exported_as_itself() {
            conformance::calls::a_memory_made_by_script_is_imported_and_exported_as_itself::<$harness>();
        }
        #[test]
        fn memory_moved_under_a_suspended_call_is_seen_by_both_sides() {
            conformance::calls::memory_moved_under_a_suspended_call_is_seen_by_both_sides::<$harness>();
        }
        #[test]
        fn an_empty_memory_has_an_empty_buffer_until_it_grows() {
            conformance::calls::an_empty_memory_has_an_empty_buffer_until_it_grows::<$harness>();
        }
        #[test]
        fn globals_cross_both_ways() {
            conformance::entities::globals_cross_both_ways::<$harness>();
        }
        #[test]
        fn a_table_holds_functions_with_their_identity() {
            conformance::entities::a_table_holds_functions_with_their_identity::<$harness>();
        }
        #[test]
        fn externref_keeps_the_script_value() {
            conformance::entities::externref_keeps_the_script_value::<$harness>();
        }
        #[test]
        fn multi_value_crosses_both_ways() {
            conformance::entities::multi_value_crosses_both_ways::<$harness>();
        }
        #[test]
        fn a_wasm_function_imported_elsewhere_is_linked_and_checked() {
            conformance::entities::a_wasm_function_imported_elsewhere_is_linked_and_checked::<$harness>();
        }
        #[test]
        fn one_function_exported_twice_is_one_script_function() {
            conformance::entities::one_function_exported_twice_is_one_script_function::<$harness>();
        }
        #[test]
        fn imports_and_exports_keep_the_module_order() {
            conformance::entities::imports_and_exports_keep_the_module_order::<$harness>();
        }
        #[test]
        fn instantiating_a_module_reads_imports_at_once() {
            conformance::entities::instantiating_a_module_reads_imports_at_once::<$harness>();
        }
        #[test]
        fn a_resizable_buffer_follows_its_memory() {
            conformance::entities::a_resizable_buffer_follows_its_memory::<$harness>();
        }
        #[test]
        fn a_memory_without_a_maximum_has_no_resizable_buffer() {
            conformance::entities::a_memory_without_a_maximum_has_no_resizable_buffer::<$harness>();
        }
        #[test]
        fn compiled_and_dropped_modules_stay_bounded() {
            conformance::freeing::compiled_and_dropped_modules_stay_bounded::<$harness>();
        }
        #[test]
        fn externrefs_in_a_loop_stay_bounded() {
            conformance::freeing::externrefs_in_a_loop_stay_bounded::<$harness>();
        }
        #[test]
        fn an_object_crossing_again_reuses_its_slot() {
            conformance::freeing::an_object_crossing_again_reuses_its_slot::<$harness>();
        }
        #[test]
        fn wasm_keeps_an_object_script_dropped() {
            conformance::freeing::wasm_keeps_an_object_script_dropped::<$harness>();
        }
        #[test]
        fn an_import_closing_over_its_own_instance_is_collected() {
            conformance::freeing::an_import_closing_over_its_own_instance_is_collected::<$harness>();
        }
        #[test]
        fn the_store_is_rebuilt_once_nothing_is_live() {
            conformance::freeing::the_store_is_rebuilt_once_nothing_is_live::<$harness>();
        }
        #[test]
        fn a_buffer_holds_the_store_and_its_memory() {
            conformance::freeing::a_buffer_holds_the_store_and_its_memory::<$harness>();
        }
        #[test]
        fn v128_throws_a_type_error_at_the_boundary() {
            conformance::wide::v128_throws_a_type_error_at_the_boundary::<$harness>();
        }
        #[test]
        fn simd_runs_inside_a_module() {
            conformance::wide::simd_runs_inside_a_module::<$harness>();
        }
        #[test]
        fn an_i64_memory_uses_bigints() {
            conformance::wide::an_i64_memory_uses_bigints::<$harness>();
        }
        #[test]
        fn an_i64_table_uses_bigints() {
            conformance::wide::an_i64_table_uses_bigints::<$harness>();
        }
    };
}
#[allow(unused_imports)]
pub(crate) use conformance_tests;
