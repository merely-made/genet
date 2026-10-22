// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The test262 command lane: script, module and async harness modes
//! over the runner core in [`crate::test262`], plus the command that
//! walks the corpus.

use super::*;

/// One test262 outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum T262 {
    Pass,
    Fail,
    Skip,
    Unsupported,
    Error,
}

/// Run every required variant through the shared routing and outcome policy.
pub(crate) fn run_262<E: ScriptEngine>(
    hns: &test262::Harness,
    test_src: &str,
    meta: &test262::Test262Meta,
    path: &Path,
) -> T262 {
    let Ok(preamble) = hns.preamble(meta) else {
        return T262::Error; // missing harness include is infrastructure failure
    };
    for route in test262::body_routes(&meta.flags) {
        let observation = std::panic::catch_unwind(AssertUnwindSafe(|| {
            observe_262::<E>(&preamble, test_src, route, path,
                meta.negative.as_ref().is_some_and(|n| n.error_type == "Test262Error"))
        }));
        let observation = match observation {
            Ok(value) => value,
            Err(_) => return T262::Error,
        };
        if std::env::var_os("GENET_TEST262_DIAGNOSTICS").is_some() {
            eprintln!("test262 {} {:?}: {:?}", path.display(), route, observation);
        }
        let result = match test262::classify(meta.negative.as_ref(), &observation) {
            test262::Outcome::Pass => T262::Pass,
            test262::Outcome::Fail => T262::Fail,
            test262::Outcome::Unsupported => T262::Unsupported,
            test262::Outcome::Error => T262::Error,
        };
        if result != T262::Pass {
            return result;
        }
    }
    T262::Pass
}

/// Set up the harness separately from the body so setup failures cannot satisfy
/// a negative test. The body uses the engines' phase and native-error evidence.
fn observe_262<E: ScriptEngine>(
    preamble: &str,
    test_src: &str,
    route: test262::BodyRoute,
    path: &Path,
    track_harness_error: bool,
) -> test262::Observation {
    use test262::{BodyRoute, Observation};
    let Ok(mut rt) = script_runtime_api::Runtime::<E>::new() else {
        return Observation::SetupError;
    };
    let completion = match route {
        BodyRoute::Script { completion, .. } | BodyRoute::Module { completion } => completion,
    };
    if completion {
        // The harness may print a diagnostic, but print is not completion authority.
        if rt.eval("globalThis.print=function(_s){};").is_err() {
            return Observation::SetupError;
        }
    }
    if !preamble.is_empty() && rt.eval(preamble).is_err() {
        return Observation::SetupError;
    }
    let identity_check = if track_harness_error {
        match rt.eval(include_str!("test262_error_identity.js")) {
            Ok(check) => Some(check),
            Err(_) => return Observation::SetupError,
        }
    } else {
        None
    };
    // ScriptEngine::Value is a rooted handle across engine calls. This callable
    // lives only in Rust; no author-visible global can change its closure state.
    let completion_poll = if completion {
        match rt.eval(include_str!("test262_completion.js")) {
            Ok(poll) => Some(poll),
            Err(_) => return Observation::SetupError,
        }
    } else {
        None
    };
    let resolver_io_error = std::rc::Rc::new(std::cell::Cell::new(false));
    let pending_module = match route {
        BodyRoute::Script { strict, .. } => {
            let source = if strict {
                format!("\"use strict\";\n{test_src}")
            } else {
                test_src.to_string()
            };
            let realm = rt.top_realm();
            let observed = rt.engine_mut().observe_script(realm, &source);
            if observed.observation != script_engine_api::ExecutionObservation::Completed {
                return identify_262_error(&mut rt, observed, identity_check.as_ref());
            }
            None
        },
        BodyRoute::Module { .. } => {
            let Ok(base_path) = path.canonicalize() else {
                return Observation::SetupError;
            };
            let base = base_path.to_string_lossy();
            let delayed_error = std::rc::Rc::clone(&resolver_io_error);
            if !rt.engine_mut().set_owned_module_resolver(Some(Box::new(move |specifier, referrer| {
                resolve_test_module(specifier, referrer, &delayed_error)
            }))) {
                return Observation::Execution(script_engine_api::ExecutionObservation::Unsupported);
            }
            let mut resolve = |specifier: &str, referrer: &str| {
                resolve_test_module(specifier, referrer, &resolver_io_error)
            };
            let realm = rt.top_realm();
            let observed = rt.engine_mut().observe_module(realm, test_src, &base, &mut resolve);
            if resolver_io_error.get() {
                return Observation::SetupError;
            }
            match observed {
                script_engine_api::ModuleObservation::Final(result)
                    if result.observation == script_engine_api::ExecutionObservation::Completed => None,
                script_engine_api::ModuleObservation::Final(other) =>
                    return identify_262_error(&mut rt, other, identity_check.as_ref()),
                script_engine_api::ModuleObservation::Pending(handle) => Some(handle),
            }
        },
    };
    if completion || pending_module.is_some() {
        if rt.run_event_loop(1024).is_err() {
            return Observation::SetupError;
        }
    }
    if resolver_io_error.get() {
        return Observation::SetupError;
    }
    if let Some(pending) = pending_module.as_ref() {
        let observed = rt.engine_mut().poll_module(pending);
        if observed.observation != script_engine_api::ExecutionObservation::Completed {
            return identify_262_error(&mut rt, observed, identity_check.as_ref());
        }
    }
    if completion {
        let poll = completion_poll.as_ref().expect("async route has poll");
        let Ok(value) = rt.engine_mut().call_function(poll, poll, &[]) else {
            return Observation::SetupError;
        };
        let Ok(status) = rt.value_to_string(&value) else {
            return Observation::SetupError;
        };
        return match status.as_str() {
            "1" => Observation::Completed,
            "2" => Observation::CompletionFailure,
            "0" => Observation::Incomplete,
            _ => Observation::SetupError,
        };
    }
    Observation::Completed
}

fn identify_262_error<E: ScriptEngine>(
    rt: &mut script_runtime_api::Runtime<E>,
    result: script_engine_api::ObservedExecution<E::Value>,
    identity_check: Option<&E::Value>,
) -> test262::Observation {
    use script_engine_api::ExecutionObservation;
    if let (Some(check), Some(thrown), ExecutionObservation::Failed { phase, .. }) =
        (identity_check, result.thrown.as_ref(), result.observation)
    {
        let Ok(value) = rt.engine_mut().call_function(check, check, std::slice::from_ref(thrown)) else {
            return test262::Observation::SetupError;
        };
        let Ok(marked) = rt.value_to_string(&value) else {
            return test262::Observation::SetupError;
        };
        if std::env::var_os("GENET_TEST262_DIAGNOSTICS").is_some() {
            eprintln!("test262 private harness identity: {marked}, outcome: {:?}", result.observation);
        }
        if marked == "1" {
            return test262::Observation::HarnessError { phase };
        }
        if marked != "0" {
            return test262::Observation::SetupError;
        }
    }
    test262::Observation::Execution(result.observation)
}

fn resolve_test_module(
    specifier: &str,
    referrer: &str,
    io_error: &std::cell::Cell<bool>,
) -> Option<(String, String)> {
    let Some(parent) = Path::new(referrer).parent() else {
        io_error.set(true);
        return None;
    };
    let target = parent.join(specifier);
    match std::fs::read_to_string(&target) {
        Ok(source) => match target.canonicalize() {
            Ok(identity) => Some((identity.to_string_lossy().into_owned(), source)),
            Err(_) => {
                io_error.set(true);
                None
            },
        },
        // A missing import is an authored module-resolution failure. Let the
        // engine report its phase and native error; missing harness includes
        // are checked separately before this point.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => {
            io_error.set(true);
            None
        },
    }
}

/// Dispatch [`run_262`] to the concrete engine, mirroring `harness::run_test`.
pub(crate) fn run_262_on(
    engine: harness::Engine,
    hns: &test262::Harness,
    test_src: &str,
    meta: &test262::Test262Meta,
    path: &Path,
) -> T262 {
    match engine {
        harness::Engine::Boa => run_262::<script_engine_boa::BoaEngine>(hns, test_src, meta, path),
        harness::Engine::Nova => {
            run_262::<script_engine_nova::NovaEngine>(hns, test_src, meta, path)
        },
    }
}

/// Worker mode: run ONE test262 test (both engines) and print per-engine results,
/// each line flushed, so the parent ([`test262_cmd`]) can attribute a hang to the
/// engine that never reported. The parent spawns this as a subprocess per test, so a
/// hanging test (the engines cannot be step-metered) kills only this process.
pub(crate) fn test262_one(args: &Args) {
    use std::io::Write;
    // A panicking test is caught by run_262's catch_unwind (→ Error); silence the hook.
    panic::set_hook(Box::new(|_| {}));

    let t262_root = Path::new(&args.tests_root).join("third_party/test262");
    let hns = match test262::Harness::load(&t262_root.join("harness")) {
        Ok(h) => h,
        Err(_) => std::process::exit(2), // parent sees no output → counts as error
    };
    let path = t262_root.join("test").join(&args.subset);
    let Ok(src) = fs::read_to_string(&path) else {
        std::process::exit(2);
    };
    let meta = test262::parse_meta(&src);

    let mut so = std::io::stdout();
    let boa = run_262_on(harness::Engine::Boa, &hns, &src, &meta, &path);
    let _ = writeln!(so, "boa {}", t262_word(&boa));
    let _ = so.flush();
    let nova = run_262_on(harness::Engine::Nova, &hns, &src, &meta, &path);
    let _ = writeln!(so, "nova {}", t262_word(&nova));
    let _ = so.flush();
}

/// The wire word for one engine's outcome (the `test262-one` worker protocol).
pub(crate) fn t262_word(t: &T262) -> &'static str {
    match t {
        T262::Pass => "pass",
        T262::Fail => "fail",
        T262::Skip => "skip",
        T262::Unsupported => "unsupported",
        T262::Error => "error",
    }
}

/// Parse a worker result line. `None` if the
/// engine never reported (it hung, or the worker died before reaching it).
pub(crate) fn parse_engine_result(out: &str, engine: &str) -> Option<T262> {
    for line in out.lines() {
        if let Some(rest) = line.strip_prefix(engine).and_then(|s| s.strip_prefix(' ')) {
            return Some(match rest.trim() {
                "pass" => T262::Pass,
                "fail" => T262::Fail,
                "skip" => T262::Skip,
                "unsupported" => T262::Unsupported,
                "error" => T262::Error,
                _ => return None,
            });
        }
    }
    None
}

fn parse_worker_results(out: &str) -> Option<(T262, T262)> {
    let lines: Vec<_> = out.lines().collect();
    if lines.len() != 2 || !lines[0].starts_with("boa ") || !lines[1].starts_with("nova ") {
        return None;
    }
    Some((parse_engine_result(lines[0], "boa")?, parse_engine_result(lines[1], "nova")?))
}

pub(crate) fn is_262_test(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "js")
        && !p
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("_FIXTURE.js"))
}

pub(crate) fn collect_262(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.is_file() {
        if is_262_test(dir) {
            out.push(dir.to_path_buf());
        }
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.is_dir() {
            collect_262(&p, out);
        } else if is_262_test(&p) {
            out.push(p);
        }
    }
}

/// `test262 <subset>`: run each test262 test (under `third_party/test262/test/<subset>`)
/// on **both** engines and diff. Boa-pass / Nova-fail is a **Nova JS-engine gap** —
/// the actual Nova worklist, since WPT showed Boa/Nova at parity. Disk only; run in
/// **release** (debug frames overflow on bounded-deep recursion).
pub(crate) fn test262_cmd(args: &Args) {
    let t262_root = Path::new(&args.tests_root).join("third_party/test262");
    // Preflight: fail fast with a clear message if the harness is missing. The actual
    // runs happen in `test262-one` worker subprocesses, which load it themselves.
    if let Err(e) = test262::Harness::load(&t262_root.join("harness")) {
        eprintln!("test262 harness load failed ({}): {e}", t262_root.display());
        std::process::exit(2);
    }
    let subset_dir = t262_root.join("test").join(&args.subset);
    if !subset_dir.exists() {
        eprintln!(
            "test262 subset path does not exist: {}",
            subset_dir.display()
        );
        std::process::exit(2);
    }
    let mut files = Vec::new();
    collect_262(&subset_dir, &mut files);
    if files.is_empty() {
        eprintln!("test262 subset contains no runnable tests: {}", subset_dir.display());
        std::process::exit(2);
    }
    let test_root = t262_root.join("test");

    // Boa and Nova cannot be step-metered (eval_bounded is unbounded for both), so a
    // pathological test (e.g. a Promise.race iterator-close infinite loop) would hang
    // the whole run. We isolate each test in a worker subprocess (`test262-one`) with a
    // wall-clock timeout: a hang kills only that process, is recorded as a timeout
    // (attributed to whichever engine never reported), and the run continues. A shared
    // work index keeps the worker pool balanced across the sorted corpus; jemalloc is
    // already linked, so per-test cost is engine-bound, not allocator-bound. Process
    // startup (~0.1s) is modest against per-test engine work, the price of hang-safety.
    let test_timeout = std::time::Duration::from_secs(args.timeout_secs);

    #[derive(Default)]
    struct Tally {
        both_pass: u64,
        both_fail: u64,
        boa_only: u64,
        nova_only: u64,
        skipped: u64,
        unsupported: u64,
        error: u64,
        timeout: u64,
        worklist: Vec<String>,
        timeouts: Vec<String>,
        excluded: Vec<(String, T262, T262)>,
    }

    let jobs = args.jobs;
    let verbose = args.verbose;
    let test_root = test_root.as_path();
    let files = &files;
    let next = std::sync::atomic::AtomicUsize::new(0);
    let next = &next;
    let tests_root = args.tests_root.as_str();
    let exe = std::env::current_exe().ok();
    let exe = exe.as_deref();
    let subset_label = if args.subset.is_empty() {
        "<all>"
    } else {
        &args.subset
    };
    println!(
        "test262 [{subset_label}]: {} tests x 2 engines on {jobs} worker procs (timeout {}s)…",
        files.len(),
        test_timeout.as_secs(),
    );

    let tally = std::thread::scope(|scope| {
        // A shared work index: workers pull the next test as they finish, so the
        // heterogeneous corpus stays balanced (contiguous chunks imbalance when the
        // slow both-pass tests cluster, as they do in the sorted corpus).
        let handles: Vec<_> = (0..jobs)
            .map(|_| {
                scope.spawn(move || {
                    let mut t = Tally::default();
                    let Some(exe) = exe else {
                        return t; // cannot locate our own binary to spawn workers
                    };
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if i >= files.len() {
                            break;
                        }
                        let path = &files[i];
                        let rel = path.strip_prefix(test_root).unwrap_or(path);
                        let name = rel.to_string_lossy().replace('\\', "/");

                        let spawned = std::process::Command::new(exe)
                            .arg("test262-one")
                            .arg(rel.as_os_str())
                            .arg("--tests-root")
                            .arg(tests_root)
                            .stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::null())
                            .spawn();
                        let Ok(mut child) = spawned else {
                            t.error += 1;
                            t.excluded.push((name, T262::Error, T262::Error));
                            continue;
                        };

                        let start = std::time::Instant::now();
                        let (timed_out, status) = loop {
                            match child.try_wait() {
                                Ok(Some(status)) => break (false, Some(status)),
                                Ok(None) => {},
                                Err(_) => break (false, None),
                            }
                            if start.elapsed() >= test_timeout {
                                let _ = child.kill();
                                let _ = child.wait();
                                break (true, None);
                            }
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        };
                        let mut out = String::new();
                        if let Some(mut so) = child.stdout.take() {
                            use std::io::Read;
                            let _ = so.read_to_string(&mut out);
                        }
                        let boa = parse_engine_result(&out, "boa");
                        let nova = parse_engine_result(&out, "nova");

                        if timed_out {
                            // Whichever engine never reported is the one still spinning.
                            let eng = if boa.is_none() { "boa" } else { "nova" };
                            if verbose {
                                println!("TIMEOUT[{eng}]  {name}");
                            }
                            t.timeout += 1;
                            t.timeouts.push(format!("{name} ({eng})"));
                            continue;
                        }
                        if !status.is_some_and(|status| status.success()) {
                            t.error += 1;
                            t.excluded.push((name, boa.unwrap_or(T262::Error), nova.unwrap_or(T262::Error)));
                            continue;
                        }
                        let (b, n) = match parse_worker_results(&out) {
                            Some(pair) => pair,
                            None => {
                                t.error += 1;
                                t.excluded.push((name, boa.unwrap_or(T262::Error), nova.unwrap_or(T262::Error)));
                                continue;
                            },
                        };
                        match (b, n) {
                            (T262::Error, _) | (_, T262::Error) => {
                                t.error += 1;
                                t.excluded.push((name, b, n));
                            },
                            (T262::Unsupported, _) | (_, T262::Unsupported) => {
                                t.unsupported += 1;
                                t.excluded.push((name, b, n));
                            },
                            (T262::Skip, _) | (_, T262::Skip) => t.skipped += 1,
                            (T262::Pass, T262::Pass) => t.both_pass += 1,
                            (T262::Fail, T262::Fail) => t.both_fail += 1,
                            (T262::Pass, T262::Fail) => {
                                if verbose {
                                    println!("NOVA-GAP  {name}");
                                }
                                t.boa_only += 1;
                                t.worklist.push(name);
                            },
                            (T262::Fail, T262::Pass) => t.nova_only += 1,
                        }
                    }
                    t
                })
            })
            .collect();
        let mut total = Tally::default();
        for h in handles {
            let t = h.join().unwrap_or_default();
            total.both_pass += t.both_pass;
            total.both_fail += t.both_fail;
            total.boa_only += t.boa_only;
            total.nova_only += t.nova_only;
            total.skipped += t.skipped;
            total.unsupported += t.unsupported;
            total.error += t.error;
            total.timeout += t.timeout;
            total.worklist.extend(t.worklist);
            total.timeouts.extend(t.timeouts);
            total.excluded.extend(t.excluded);
        }
        total
    });

    let mut nova_worklist = tally.worklist;
    nova_worklist.sort();
    let mut timeouts = tally.timeouts;
    timeouts.sort();
    let mut excluded = tally.excluded;
    excluded.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        tally.both_pass + tally.both_fail + tally.boa_only + tally.nova_only +
        tally.timeout + tally.skipped + tally.unsupported + tally.error,
        files.len() as u64,
        "every discovered test262 file must have exactly one accounted outcome",
    );
    println!(
        "\ntest262 compare [{subset_label}]: both-pass={} both-fail={} boa-only={} (Nova gap) \
         nova-only={} timeout={} skipped={} unsupported-excluded={} error-excluded={}",
        tally.both_pass,
        tally.both_fail,
        tally.boa_only,
        tally.nova_only,
        tally.timeout,
        tally.skipped,
        tally.unsupported,
        tally.error,
    );
    if !timeouts.is_empty() {
        println!(
            "\nExceeded {}s — infinite hang or pathological slowness (the engine that \
             never reported) — {} test(s):",
            test_timeout.as_secs(),
            timeouts.len()
        );
        for name in timeouts.iter().take(40) {
            println!("  {name}");
        }
        if timeouts.len() > 40 {
            println!("  … and {} more", timeouts.len() - 40);
        }
    }
    if !excluded.is_empty() {
        println!(
            "\nExcluded outcomes (test-level; boa and nova preserved) — {} test(s):",
            excluded.len()
        );
        for (name, boa, nova) in excluded.iter().take(40) {
            println!("  {name} boa={} nova={}", t262_word(boa), t262_word(nova));
        }
        if excluded.len() > 40 {
            println!(
                "  … and {} more in the worklist artifact",
                excluded.len() - 40
            );
        }
    }
    if !nova_worklist.is_empty() {
        println!(
            "\nNova worklist (pass on Boa, fail on Nova) — {} test(s):",
            nova_worklist.len()
        );
        for name in nova_worklist.iter().take(40) {
            println!("  {name}");
        }
        if nova_worklist.len() > 40 {
            println!("  … and {} more", nova_worklist.len() - 40);
        }
    }

    if let Some(out_path) = &args.worklist_out {
        use std::io::Write;
        let mut buf = format!(
            "# test262 worklist [{subset_label}]\n\
             # both-pass={} both-fail={} boa-only={} nova-only={} timeout={} skipped={} unsupported-excluded={} error-excluded={}\n",
            tally.both_pass,
            tally.both_fail,
            tally.boa_only,
            tally.nova_only,
            tally.timeout,
            tally.skipped,
            tally.unsupported,
            tally.error,
        );
        buf.push_str(&format!(
            "\n## Timeouts (hang or pathological slowness; engine) — {}\n",
            timeouts.len()
        ));
        for t in &timeouts {
            buf.push_str(t);
            buf.push('\n');
        }
        buf.push_str(&format!(
            "\n## Excluded outcomes (test-level; boa and nova preserved) — {}\n",
            excluded.len()
        ));
        for (name, boa, nova) in &excluded {
            buf.push_str(&format!(
                "{name} boa={} nova={}\n",
                t262_word(boa),
                t262_word(nova)
            ));
        }
        buf.push_str(&format!(
            "\n## Nova gaps (pass on Boa, fail on Nova) — {}\n",
            nova_worklist.len()
        ));
        for n in &nova_worklist {
            buf.push_str(n);
            buf.push('\n');
        }
        match std::fs::File::create(out_path).and_then(|mut f| f.write_all(buf.as_bytes())) {
            Ok(()) => println!(
                "\nworklist written to {out_path} ({} Nova gaps, {} timeouts)",
                nova_worklist.len(),
                timeouts.len()
            ),
            Err(e) => {
                eprintln!("failed to write worklist to {out_path}: {e}");
                std::process::exit(2);
            },
        }
    }
    if tally.error != 0 {
        eprintln!("test262 encountered {} infrastructure error(s)", tally.error);
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> (PathBuf, test262::Harness) {
        let dir = std::env::temp_dir().join(format!("genet-t262-{name}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("assert.js"), "").unwrap();
        fs::write(dir.join("sta.js"), "").unwrap();
        fs::write(dir.join("doneprintHandle.js"), include_str!("../../../tests/wpt/tests/third_party/test262/harness/doneprintHandle.js")).unwrap();
        let harness = test262::Harness::load(&dir).unwrap();
        (dir, harness)
    }

    fn check_both(harness: &test262::Harness, path: &Path, source: &str, want: T262) {
        if !path.exists() {
            fs::write(path, source).unwrap();
        }
        let meta = test262::parse_meta(source);
        assert_eq!(run_262::<script_engine_boa::BoaEngine>(harness, source, &meta, path), want, "Boa: {source}");
        assert_eq!(run_262::<script_engine_nova::NovaEngine>(harness, source, &meta, path), want, "Nova: {source}");
    }

    #[test]
    fn negative_requires_phase_and_native_error_identity() {
        let (dir, harness) = fixture("negative");
        let path = dir.join("entry.js");
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\nlet = ;", T262::Pass);
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nthrow new TypeError('expected');", T262::Pass);
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nlet e = new TypeError(); Object.defineProperty(e, 'message', {get() {throw new Error('getter')}}); e.name='SyntaxError'; e.constructor=SyntaxError; throw e;", T262::Pass);
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nthrow new SyntaxError('wrong');", T262::Fail);
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: SyntaxError\n---*/\nlet = ;", T262::Fail);
        check_both(&harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nthrow {name:'TypeError', constructor:TypeError};", T262::Fail);
        fs::write(dir.join("sta.js"), include_str!("../../../tests/wpt/tests/third_party/test262/harness/sta.js")).unwrap();
        let error_harness = test262::Harness::load(&dir).unwrap();
        check_both(&error_harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: Test262Error\n---*/\nthrow new Test262Error();", T262::Pass);
        check_both(&error_harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: Test262Error\n---*/\nif (!Object.prototype.hasOwnProperty.call(Test262Error, 'thrower') || Object.getPrototypeOf(Test262Error) !== Function.prototype) throw new TypeError('constructor shape'); Test262Error.thrower();", T262::Pass);
        check_both(&error_harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: Test262Error\n---*/\nthrow {__proto__: Test262Error.prototype, constructor: Test262Error, name: 'Test262Error'};", T262::Fail);
        fs::write(dir.join("assert.js"), include_str!("../../../tests/wpt/tests/third_party/test262/harness/assert.js")).unwrap();
        let assert_harness = test262::Harness::load(&dir).unwrap();
        check_both(&assert_harness, &path,
            "/*---\nnegative:\n  phase: runtime\n  type: Test262Error\n---*/\ntry { assert.throws(Test262Error, () => { throw new Test262Error(); }); } catch (_) { throw new TypeError('assert.throws changed'); } throw new Test262Error.prototype.constructor();", T262::Pass);
    }

    #[test]
    fn harness_and_completion_failures_do_not_pass() {
        let (dir, harness) = fixture("completion");
        let path = dir.join("entry.js");
        check_both(&harness, &path,
            "/*---\nincludes: [missing.js]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nthrow new TypeError();", T262::Error);
        fs::write(dir.join("bad.js"), "throw new TypeError('setup');").unwrap();
        check_both(&harness, &path,
            "/*---\nincludes: [bad.js]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\n0;", T262::Error);
        check_both(&harness, &path, "/*---\nflags: [async]\n---*/\n$DONE();", T262::Pass);
        check_both(&harness, &path, "/*---\nflags: [async]\n---*/\n0;", T262::Fail);
        check_both(&harness, &path, "/*---\nflags: [async]\n---*/\n$DONE(new Error('bad'));", T262::Fail);
    }

    #[test]
    fn module_linking_and_async_imports_use_module_route() {
        let (dir, harness) = fixture("module");
        let path = dir.join("entry.js");
        fs::write(dir.join("dep.js"), "export const answer = 42;").unwrap();
        check_both(&harness, &path,
            "/*---\nflags: [module]\nnegative:\n  phase: resolution\n  type: SyntaxError\n---*/\nimport {missing} from './dep.js';", T262::Pass);
        check_both(&harness, &path,
            "/*---\nflags: [module, async]\n---*/\nimport {answer} from './dep.js'; if (answer === 42) $DONE();", T262::Pass);
        check_both(&harness, &path,
            "/*---\nflags: [module, async]\n---*/\nimport {answer} from './dep.js';", T262::Fail);
        check_both(&harness, &path,
            "/*---\nflags: [module, async]\n---*/\nawait new Promise(resolve => setTimeout(resolve, 0)); $DONE();", T262::Pass);
        check_both(&harness, &path,
            "/*---\nflags: [module, async]\n---*/\nPromise.resolve().then(() => import('./dep.js')).then(ns => { if (ns.answer === 42) $DONE(); else $DONE(new Error('bad import')); }, $DONE);", T262::Pass);
        fs::write(dir.join("realm.js"), "globalThis.realmMarker = 42;").unwrap();
        fs::write(dir.join("realm-dep.js"), "export const marker = globalThis.realmMarker;").unwrap();
        let realm_harness = test262::Harness::load(&dir).unwrap();
        check_both(&realm_harness, &path,
            "/*---\nflags: [module, async]\nincludes: [realm.js]\n---*/\nPromise.resolve().then(() => import('./realm-dep.js')).then(ns => { if (ns.marker === 42) $DONE(); else $DONE(new Error('wrong realm')); }, $DONE);", T262::Pass);
        check_both(&harness, &path,
            "/*---\nflags: [module]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nawait new Promise(resolve => setTimeout(resolve, 0)); throw new TypeError('late');", T262::Pass);
        check_both(&harness, &path,
            "/*---\nflags: [module]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nawait new Promise(resolve => setTimeout(resolve, 0)); throw new SyntaxError('wrong');", T262::Fail);
    }

    #[test]
    fn circular_dynamic_import_uses_entry_module_identity() {
        let (dir, _) = fixture("circular-import");
        fs::write(dir.join("assert.js"), include_str!("../../../tests/wpt/tests/third_party/test262/harness/assert.js")).unwrap();
        fs::write(dir.join("verify-dfs-a_FIXTURE.js"), include_str!("../../../tests/wpt/tests/third_party/test262/test/language/module-code/verify-dfs-a_FIXTURE.js")).unwrap();
        fs::write(dir.join("verify-dfs-b_FIXTURE.js"), include_str!("../../../tests/wpt/tests/third_party/test262/test/language/module-code/verify-dfs-b_FIXTURE.js")).unwrap();
        let harness = test262::Harness::load(&dir).unwrap();
        let source = include_str!("../../../tests/wpt/tests/third_party/test262/test/language/module-code/verify-dfs.js");
        check_both(&harness, &dir.join("verify-dfs.js"), source, T262::Pass);
    }

    fn check_replaced_resolver<E: ScriptEngine>() {
        use script_engine_api::{ExecutionObservation, ModuleObservation};
        let mut runtime = script_runtime_api::Runtime::<E>::new().unwrap();
        let realm = runtime.top_realm();
        let entry = "/test262/entry.js";
        for answer in [1, 2] {
            assert!(runtime.engine_mut().set_owned_module_resolver(Some(Box::new(
                move |_, _| Some(("/test262/dep.js".into(), format!("export const answer = {answer};"))),
            ))));
            let mut resolve = |_: &str, _: &str| {
                Some(("/test262/dep.js".into(), format!("export const answer = {answer};")))
            };
            let source = format!("import {{answer}} from './dep.js'; if (answer !== {answer}) throw new Error('stale module');");
            assert!(matches!(
                runtime.engine_mut().observe_module(realm, &source, entry, &mut resolve),
                ModuleObservation::Final(script_engine_api::ObservedExecution {
                    observation: ExecutionObservation::Completed, ..
                }),
            ), "replacement must load dependency version {answer}");
        }
    }

    #[test]
    fn replacing_owned_resolver_starts_fresh_graph() {
        check_replaced_resolver::<script_engine_boa::BoaEngine>();
        check_replaced_resolver::<script_engine_nova::NovaEngine>();
    }

    #[test]
    fn malformed_worker_words_have_no_success_path() {
        assert_eq!(parse_engine_result("boa pass\nnova banana\n", "nova"), None);
        assert_eq!(parse_engine_result("boa pass\n", "nova"), None);
        assert_eq!(parse_worker_results("boa pass\nnova banana\n"), None);
        assert_eq!(parse_worker_results("boa pass\nboa fail\nnova pass\n"), None);
    }
}
