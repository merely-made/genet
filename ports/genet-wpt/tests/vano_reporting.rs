//! Real CLI reporting regression: a passing JavaScript body must produce
//! subtests through the standard WPT completion callback, on either engine.
use std::{path::PathBuf, process::Command};

#[test]
fn isolated_workers_report_real_harness_results_on_both_engines() {
    let root = std::env::var_os("GENET_WPT_TESTS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wpt/tests"));
    assert!(
        root.join("resources/testharness.js").is_file(),
        "set GENET_WPT_TESTS_ROOT to the vendored WPT corpus"
    );
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reporting.html");
    for engine in ["boa", "nova"] {
        for gc in [true, false] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_genet-wpt"));
            command
                .args(["testharness-one", "reporting.html", "--engine", engine])
                .arg("--test-path")
                .arg(&fixture)
                .arg("--tests-root")
                .arg(&root);
            if !gc {
                command.arg("--no-harness-gc");
            }
            let output = command.output().expect("spawn WPT worker");
            assert!(output.status.success(), "worker failed: {engine}, GC={gc}");
            let stdout = String::from_utf8(output.stdout).expect("worker output UTF-8");
            let result = stdout
                .lines()
                .find_map(|line| line.strip_prefix("__thresult "))
                .unwrap_or_else(|| panic!("missing worker result: {stdout}"));
            let result: serde_json::Value = serde_json::from_str(result).unwrap();
            assert_eq!(result["status"], "pass", "{engine}, GC={gc}: {result}");
            assert_eq!(result["subtests_passed"], 2, "{engine}, GC={gc}: {result}");
            assert_eq!(result["subtests_total"], 2, "{engine}, GC={gc}: {result}");
        }
    }
}
