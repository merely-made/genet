// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Run the tree's `wasm/jsapi` web platform tests on a backend with
//! `WebAssembly` installed: a measure of the distance left, not a target.
//!
//! The tests run in `testharness.js`'s shell environment (their `jsshell`
//! global) on a bare engine: no DOM, no timers, no fetch. Each file runs in a
//! child process with a three-minute limit, since no backend can bound a
//! runaway script; the slowest file takes about 45 seconds in a debug build.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::Harness;

const LIMIT: Duration = Duration::from_secs(180);

const PRELUDE: &str = "globalThis.self = globalThis;
globalThis.GLOBAL = { isWindow: () => false, isWorker: () => false, isShadowRealm: () => false };
if (typeof console === 'undefined') {
  const quiet = () => {};
  globalThis.console = { log: quiet, debug: quiet, info: quiet, warn: quiet, error: quiet };
}";

const COLLECTOR: &str = "globalThis.__results = [];
globalThis.__harness = null;
add_result_callback((t) => __results.push([t.status, t.name, String(t.message ?? '')]));
add_completion_callback((_, s) => { __harness = [s.status, String(s.message ?? '')]; });";

const REPORT: &str = "(() => {
  const line = (s) => s.replace(/[\\t\\n\\r]/g, ' ');
  const out = __results.map(([status, name, message]) =>
    ['RESULT', status, line(name), line(message)].join('\\t'));
  out.push(['HARNESS', __harness ? __harness[0] : 'none', line(__harness ? __harness[1] : '')].join('\\t'));
  return out.join('\\n');
})()";

fn metas(source: &str) -> Vec<(String, String)> {
    source
        .lines()
        .take_while(|l| l.starts_with("//") || l.trim().is_empty())
        .filter_map(|l| l.strip_prefix("// META:"))
        .filter_map(|m| m.trim().split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn resolve(root: &Path, test: &Path, src: &str) -> PathBuf {
    match src.strip_prefix('/') {
        Some(rooted) => root.join(rooted),
        None => test.parent().unwrap().join(src),
    }
}

fn eval<H: Harness>(engine: &mut H, what: &str, source: &str) -> Result<(), String> {
    engine
        .try_eval(source)
        .map(drop)
        .map_err(|e| format!("{what}: {e}"))
}

/// Run one file in this process and print its result lines.
fn run_one<H: Harness>(root: &Path, test: &Path) -> Result<(), String> {
    let source = std::fs::read_to_string(test).map_err(|e| e.to_string())?;
    let mut engine = H::new();
    engine.set_fuel(Some(50_000_000_000));
    eval(&mut engine, "prelude", PRELUDE)?;
    let harness = std::fs::read_to_string(root.join("resources/testharness.js"))
        .map_err(|e| e.to_string())?;
    eval(&mut engine, "testharness.js", &harness)?;
    eval(&mut engine, "collector", COLLECTOR)?;
    for (key, value) in metas(&source) {
        if key == "script" {
            let path = resolve(root, test, &value);
            let script = std::fs::read_to_string(&path).map_err(|e| format!("{value}: {e}"))?;
            eval(&mut engine, &value, &script)?;
        }
    }
    let outcome = eval(&mut engine, "test", &source);
    for _ in 0..8 {
        engine.pump();
    }
    eval(&mut engine, "timeout", "if (!__harness) timeout();")?;
    engine.pump();
    println!("{}", engine.try_eval(REPORT)?);
    if let Err(e) = outcome {
        println!("THREW\t{}", e.replace(['\t', '\n'], " "));
    }
    Ok(())
}

#[derive(Default)]
struct Tally {
    pass: usize,
    total: usize,
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, out);
        } else if path.to_string_lossy().ends_with(".any.js") {
            out.push(path);
        }
    }
}

fn run_child(test: &Path, root: &Path) -> (String, bool) {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--one")
        .arg(root)
        .arg(test)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            let out = child.wait_with_output().unwrap();
            return (
                String::from_utf8_lossy(&out.stdout).into_owned(),
                out.status.success(),
            );
        }
        if start.elapsed() > LIMIT {
            let _ = child.kill();
            let _ = child.wait();
            return ("HARNESS\tkilled\ttime limit".to_string(), false);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The runner's entry point: with `--one`, one file in this process;
/// otherwise every file, each in a child process of this binary.
pub fn main<H: Harness>() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--one") {
        if let Err(e) = run_one::<H>(Path::new(&args[2]), Path::new(&args[3])) {
            println!("HARNESS\terror\t{}", e.replace(['\t', '\n'], " "));
        }
        return;
    }
    let root = PathBuf::from(args.get(1).expect("usage: wasm_jsapi <wpt tests dir>"));
    let mut tests = Vec::new();
    files(&root.join("wasm/jsapi"), &mut tests);
    let mut all = Tally::default();
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    let mut skipped = Vec::new();
    for test in &tests {
        let name = test.strip_prefix(&root).unwrap().display().to_string();
        let source = std::fs::read_to_string(test).unwrap();
        let global = metas(&source)
            .into_iter()
            .find(|(k, _)| k == "global")
            .map(|(_, v)| v);
        if !global.is_some_and(|g| g.split(',').any(|g| g.trim() == "jsshell")) {
            skipped.push(name);
            continue;
        }
        let (out, exited) = run_child(test, &root);
        let mut file = Tally::default();
        let mut harness = String::from("none");
        for line in out.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            match fields.as_slice() {
                ["RESULT", status, _, message] => {
                    file.total += 1;
                    if *status == "0" {
                        file.pass += 1;
                    } else {
                        let reason: String = message.chars().take(90).collect();
                        *reasons.entry(reason).or_default() += 1;
                    }
                },
                ["HARNESS", status, message] => {
                    harness = format!("{status} {}", message.chars().take(100).collect::<String>());
                },
                ["THREW", message] => harness.push_str(&format!(" | threw {message:.100}")),
                _ => {},
            }
        }
        if !exited {
            harness.push_str(" (child failed)");
        }
        println!(
            "{:4}/{:<4} {name}  [{}]",
            file.pass,
            file.total,
            harness.trim()
        );
        all.pass += file.pass;
        all.total += file.total;
    }
    println!(
        "\n{} of {} subtests passed in {} files",
        all.pass,
        all.total,
        tests.len() - skipped.len()
    );
    println!("not run (no jsshell global): {}", skipped.join(", "));
    let mut top: Vec<_> = reasons.into_iter().collect();
    top.sort_by_key(|entry| std::cmp::Reverse(entry.1));
    println!("\nmost common failure messages:");
    for (reason, count) in top.into_iter().take(25) {
        println!("{count:5}  {reason}");
    }
}
