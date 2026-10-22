/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Every adversarial script runs in a separate test process with an external
//! watchdog. A regression cannot strand the cargo test process in a VM loop.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use script_engine_api::ScriptEngine;
use script_runtime_api::{Runtime, ScriptResourceLoader, WorkerBackend, WorkerProcessLauncher};

#[cfg(target_os = "linux")]
fn helper_pid() -> u32 {
    let tasks = std::fs::read_dir("/proc/self/task").expect("task list");
    for task in tasks {
        let task = task.expect("task entry");
        let children = std::fs::read_to_string(task.path().join("children")).expect("child pids");
        if let Some(pid) = children.split_whitespace().next() {
            return pid.parse().expect("numeric helper pid");
        }
    }
    panic!("worker helper child pid absent");
}

#[cfg(target_os = "linux")]
fn assert_helper_reaped(pid: u32) {
    assert!(
        !std::path::Path::new(&format!("/proc/{pid}")).exists(),
        "helper {pid} not reaped"
    );
    let tasks = std::fs::read_dir("/proc/self/task").expect("task list");
    for task in tasks {
        let task = task.expect("task entry");
        let name = std::fs::read_to_string(task.path().join("comm")).expect("thread name");
        assert!(
            !name.starts_with("genet-worker-ip"),
            "bridge thread leaked: {name}"
        );
    }
}

#[cfg(target_os = "linux")]
fn assert_no_helper_children() {
    for task in std::fs::read_dir("/proc/self/task").expect("task list") {
        let task = task.expect("task entry");
        let children = std::fs::read_to_string(task.path().join("children")).expect("child pids");
        assert!(
            children.trim().is_empty(),
            "unreaped helper child: {children}"
        );
    }
}

struct Scripts;
impl ScriptResourceLoader for Scripts {
    fn load(&self, url: &str) -> Option<String> {
        let file = url.rsplit('/').next().unwrap_or(url);
        match file {
            "import.js" => Some("globalThis.imported = 'from-page-loader';".into()),
            "child.js" => Some("postMessage('entered'); while (true) {}".into()),
            "root.js" => Some(match std::env::var("GENET_WORKER_CASE").unwrap().as_str() {
                "positive" => "importScripts('import.js'); postMessage(imported); onmessage = e => postMessage('echo:' + e.data);",
                "startup" | "parent_exit" | "document_drop" | "queue_pressure" => "postMessage('entered'); while (true) {}",
                "callback" => "postMessage('ready'); onmessage = () => { postMessage('entered'); while (true) {} };",
                "timer" => "setTimeout(() => { postMessage('entered'); while (true) {} }, 0);",
                "microtasks" => "postMessage('entered'); Promise.resolve().then(function again() { Promise.resolve().then(again); });",
                "nested" => "var child = new Worker('child.js'); child.onmessage = e => postMessage(e.data);",
                other => panic!("unknown worker case {other}"),
            }.into()),
            _ => None,
        }
    }
}

struct DocumentFetcher;
impl genet_host_api::ResourceFetcher for DocumentFetcher {
    fn fetch(&self, _: &str) -> Option<Vec<u8>> {
        None
    }
    fn fetch_request(
        &self,
        _: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        None
    }
}

fn document_fixture<E: ScriptEngine>(backend: WorkerBackend) {
    let mut options = genet_scripted::ScriptedDocumentOptions::default();
    options.worker_process_launcher = Some(WorkerProcessLauncher::new(
        env!("CARGO_BIN_EXE_genet-worker-process").into(),
        backend,
    ));
    let mut doc = genet_scripted::ScriptedDocument::<E>::from_body_with_options(
        "<body></body>",
        &DocumentFetcher,
        "https://worker.test/page",
        None,
        options,
    )
    .expect("scripted document");
    doc.set_script_resource_loader(Box::new(Scripts));
    doc.evaluate(
        "var seen = ''; var w = new Worker('root.js'); w.onmessage = e => { seen = e.data; };",
    )
    .expect("create document worker");
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        doc.pump(0.0);
        if doc
            .evaluate("if (seen !== 'entered') throw Error('pending')")
            .is_ok()
        {
            break;
        }
        if Instant::now() >= deadline {
            panic!("document worker did not enter infinite script");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    #[cfg(target_os = "linux")]
    let pid = helper_pid();
    drop(doc);
    #[cfg(target_os = "linux")]
    assert_helper_reaped(pid);
    println!("PASS {backend:?}/document_drop/drop");
}

type Completion = Box<dyn FnOnce(script_runtime_api::FetchOutcome) + Send>;
struct RetainedLoader(Arc<Mutex<Option<Completion>>>);
impl ScriptResourceLoader for RetainedLoader {
    fn load(&self, _: &str) -> Option<String> {
        None
    }
    fn start(
        &self,
        _: u64,
        _: script_runtime_api::FetchRequest,
        complete: Completion,
    ) -> Option<script_runtime_api::FetchRequest> {
        *self.0.lock().unwrap() = Some(complete);
        None
    }
}

fn retained_callback_fixture<E: ScriptEngine>(backend: WorkerBackend, action: &str) {
    let held = Arc::new(Mutex::new(None));
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://worker.test/page")
        .expect("page URL");
    runtime.set_worker_process_launcher(WorkerProcessLauncher::new(
        env!("CARGO_BIN_EXE_genet-worker-process").into(),
        backend,
    ));
    runtime.set_script_resource_loader(Box::new(RetainedLoader(held.clone())));
    runtime
        .eval("var w = new Worker('root.js')")
        .expect("create worker");
    let deadline = Instant::now() + Duration::from_secs(8);
    while held.lock().unwrap().is_none() {
        runtime.pump_workers();
        if Instant::now() >= deadline {
            panic!("worker request was not retained");
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    #[cfg(target_os = "linux")]
    let pid = helper_pid();
    if action == "terminate" {
        runtime.eval("w.terminate()").expect("terminate worker");
        let report = runtime.shutdown_workers();
        assert_eq!(report.processes_failed, 0, "{report:?}");
    } else {
        drop(runtime);
    }
    assert!(
        held.lock().unwrap().is_some(),
        "callback must remain retained through teardown"
    );
    #[cfg(target_os = "linux")]
    assert_helper_reaped(pid);
    println!("PASS {backend:?}/retained_callback/{action}");
}

#[cfg(unix)]
fn no_read_handshake_fixture<E: ScriptEngine>(backend: WorkerBackend) {
    use std::os::unix::fs::PermissionsExt;
    let path = std::env::temp_dir().join(format!("genet-no-read-{}", std::process::id()));
    std::fs::write(&path, b"#!/bin/sh\nexec sleep 30\n").expect("write no-read helper");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
        .expect("executable helper");
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://worker.test/page")
        .expect("page URL");
    let mut launcher = WorkerProcessLauncher::new(path.clone(), backend);
    launcher.startup_timeout = Duration::from_millis(100);
    runtime.set_worker_process_launcher(launcher);
    let started = Instant::now();
    let result = runtime
        .eval("try { new Worker('root.js', { name: 'x'.repeat(256 * 1024) }); 'unexpected' } catch (e) { e.name }")
        .expect("worker creation result");
    assert_eq!(
        runtime.value_to_string(&result).unwrap(),
        "NotSupportedError"
    );
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "handshake exceeded startup bound"
    );
    std::fs::remove_file(path).expect("remove no-read helper");
    #[cfg(target_os = "linux")]
    assert_no_helper_children();
    println!("PASS {backend:?}/no_read/startup_timeout");
}

fn invalid_queue_fixture<E: ScriptEngine>(backend: WorkerBackend) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://worker.test/page")
        .expect("page URL");
    let mut launcher =
        WorkerProcessLauncher::new(env!("CARGO_BIN_EXE_genet-worker-process").into(), backend);
    launcher.max_queued_messages = usize::MAX;
    runtime.set_worker_process_launcher(launcher);
    let result = runtime
        .eval("try { new Worker('root.js'); 'unexpected' } catch (e) { e.name }")
        .expect("invalid launcher result");
    assert_eq!(
        runtime.value_to_string(&result).unwrap(),
        "NotSupportedError"
    );
    #[cfg(target_os = "linux")]
    assert_no_helper_children();
    println!("PASS {backend:?}/invalid_queue/rejected");
}

fn read<E: ScriptEngine>(runtime: &mut Runtime<E>) -> String {
    let value = runtime.eval("String(seen)").expect("read page state");
    runtime.value_to_string(&value).expect("stringify state")
}

fn until<E: ScriptEngine>(runtime: &mut Runtime<E>, target: &str) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        runtime.pump_workers();
        if read(runtime) == target {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!(
        "worker did not reach {target}; last state {}",
        read(runtime)
    );
}

fn fixture<E: ScriptEngine>(backend: WorkerBackend, case: &str, action: &str) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://worker.test/page")
        .expect("page URL");
    let mut launcher =
        WorkerProcessLauncher::new(env!("CARGO_BIN_EXE_genet-worker-process").into(), backend);
    launcher.shutdown_grace = Duration::from_millis(50);
    if case == "queue_pressure" {
        launcher.max_queued_messages = 1;
    }
    runtime.set_worker_process_launcher(launcher);
    runtime.set_script_resource_loader(Box::new(Scripts));
    runtime
        .eval(
            "var seen = ''; var w = new Worker('root.js'); w.onmessage = e => { seen = e.data; };",
        )
        .expect("create worker");
    if case == "callback" {
        until(&mut runtime, "ready");
        runtime
            .eval("w.postMessage('go')")
            .expect("post callback task");
    }
    until(
        &mut runtime,
        if case == "positive" {
            "from-page-loader"
        } else {
            "entered"
        },
    );
    #[cfg(target_os = "linux")]
    let pid = helper_pid();
    if case == "queue_pressure" {
        runtime.eval(r#"
          w.onerror = () => { seen = 'transport-error'; };
          var payload = 'x'.repeat(16384);
          for (var i = 0; i < 500; i++) w.postMessage(payload);
          var channel = new MessageChannel();
          var syncError = '';
          try { w.postMessage({ port: channel.port1 }, [channel.port1]); } catch (e) { syncError = e.name; }
          var portDetached = false;
          try { channel.port1.postMessage('probe'); } catch (e) { portDetached = e.name === 'InvalidStateError'; }
          var buffer = new ArrayBuffer(8);
          var bufferError = '';
          try { w.postMessage({ buffer: buffer }, [buffer]); } catch (e) { bufferError = e.name; }
          var bufferDetached = buffer.byteLength === 0;
        "#).expect("drive bounded queue to terminal failure");
        let result = runtime
            .eval("syncError + ':' + portDetached + ':' + bufferError + ':' + bufferDetached")
            .expect("transfer result");
        assert_eq!(runtime.value_to_string(&result).unwrap(), ":true::true");
        until(&mut runtime, "transport-error");
        let report = runtime.shutdown_workers();
        assert_eq!(
            report.processes_joined + report.processes_killed + report.processes_crashed,
            1,
            "{report:?}"
        );
        assert_eq!(report.processes_failed, 0, "{report:?}");
        #[cfg(target_os = "linux")]
        assert_helper_reaped(pid);
    } else if case == "positive" {
        runtime
            .eval("w.postMessage('hello')")
            .expect("post to worker");
        until(&mut runtime, "echo:hello");
        let report = runtime.shutdown_workers();
        assert_eq!(report.processes_failed, 0, "{report:?}");
        assert_eq!(
            report.processes_joined + report.processes_killed,
            1,
            "{report:?}"
        );
        assert!(!report.descendants_untracked, "{report:?}");
        #[cfg(target_os = "linux")]
        assert_helper_reaped(pid);
    } else if case == "parent_exit" {
        #[cfg(target_os = "linux")]
        {
            println!("HELPER_PID={pid}");
            std::io::stdout().flush().unwrap();
            std::process::exit(0);
        }
    } else if action == "terminate" {
        runtime.eval("w.terminate()").expect("terminate worker");
        assert_eq!(runtime.shutdown_workers().processes_failed, 0);
        #[cfg(target_os = "linux")]
        assert_helper_reaped(pid);
    } else {
        drop(runtime);
        #[cfg(target_os = "linux")]
        assert_helper_reaped(pid);
    }
    println!("PASS {backend:?}/{case}/{action}");
}

#[test]
fn isolated_worker_case() {
    let Ok(case) = std::env::var("GENET_WORKER_CASE") else {
        return;
    };
    let engine = std::env::var("GENET_WORKER_ENGINE").unwrap();
    let action = std::env::var("GENET_WORKER_ACTION").unwrap();
    match engine.as_str() {
        "boa" if case == "invalid_queue" => {
            invalid_queue_fixture::<script_engine_boa::BoaEngine>(WorkerBackend::Boa)
        },
        #[cfg(unix)]
        "boa" if case == "no_read" => {
            no_read_handshake_fixture::<script_engine_boa::BoaEngine>(WorkerBackend::Boa)
        },
        "boa" if case == "retained_callback" => {
            retained_callback_fixture::<script_engine_boa::BoaEngine>(WorkerBackend::Boa, &action)
        },
        "boa" if case == "document_drop" => {
            document_fixture::<script_engine_boa::BoaEngine>(WorkerBackend::Boa)
        },
        "boa" => fixture::<script_engine_boa::BoaEngine>(WorkerBackend::Boa, &case, &action),
        #[cfg(target_pointer_width = "64")]
        #[cfg(unix)]
        "nova" if case == "no_read" => {
            no_read_handshake_fixture::<script_engine_nova::NovaEngine>(WorkerBackend::Nova)
        },
        #[cfg(target_pointer_width = "64")]
        "nova" if case == "invalid_queue" => {
            invalid_queue_fixture::<script_engine_nova::NovaEngine>(WorkerBackend::Nova)
        },
        #[cfg(target_pointer_width = "64")]
        "nova" if case == "retained_callback" => retained_callback_fixture::<
            script_engine_nova::NovaEngine,
        >(WorkerBackend::Nova, &action),
        #[cfg(target_pointer_width = "64")]
        "nova" if case == "document_drop" => {
            document_fixture::<script_engine_nova::NovaEngine>(WorkerBackend::Nova)
        },
        #[cfg(target_pointer_width = "64")]
        "nova" => fixture::<script_engine_nova::NovaEngine>(WorkerBackend::Nova, &case, &action),
        _ => panic!("unsupported fixture engine {engine}"),
    }
}

#[test]
fn supervised_worker_tree_reclaims_spinning_scripts() {
    let engines = ["boa", "nova"];
    for engine in engines {
        if engine == "nova" && !cfg!(target_pointer_width = "64") {
            continue;
        }
        for case in [
            "positive",
            "startup",
            "callback",
            "timer",
            "microtasks",
            "nested",
            "document_drop",
            "retained_callback",
            "no_read",
            "queue_pressure",
            "invalid_queue",
        ] {
            if case == "no_read" && !cfg!(unix) {
                continue;
            }
            for action in ["terminate", "drop"] {
                if (case == "positive" && action == "drop")
                    || (case == "document_drop" && action == "terminate")
                    || (case == "no_read" && action == "terminate")
                    || (case == "queue_pressure" && action == "drop")
                    || (case == "invalid_queue" && action == "terminate")
                {
                    continue;
                }
                let mut child = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "isolated_worker_case", "--nocapture"])
                    .env("GENET_WORKER_CASE", case)
                    .env("GENET_WORKER_ENGINE", engine)
                    .env("GENET_WORKER_ACTION", action)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("spawn isolated fixture");
                let deadline = Instant::now() + Duration::from_secs(15);
                let status = loop {
                    if let Some(status) = child.try_wait().expect("poll fixture") {
                        break status;
                    }
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        panic!("watchdog killed {engine}/{case}/{action}");
                    }
                    std::thread::sleep(Duration::from_millis(10));
                };
                let output = child.wait_with_output().expect("fixture output");
                assert!(
                    status.success(),
                    "{engine}/{case}/{action}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
fn parent_disappearance_reclaims_spinning_worker() {
    for engine in ["boa", "nova"] {
        if engine == "nova" && !cfg!(target_pointer_width = "64") {
            continue;
        }
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "isolated_worker_case", "--nocapture"])
            .env("GENET_WORKER_CASE", "parent_exit")
            .env("GENET_WORKER_ENGINE", engine)
            .env("GENET_WORKER_ACTION", "drop")
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn parent disappearance fixture");
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("watchdog killed parent disappearance fixture");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pid = stdout
            .lines()
            .find_map(|line| line.split_once("HELPER_PID=").map(|(_, pid)| pid))
            .expect("helper pid receipt");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
            if match &stat {
                Err(_) => true,
                Ok(stat) => stat
                    .split(") ")
                    .nth(1)
                    .is_some_and(|tail| tail.starts_with('Z')),
            } {
                break;
            }
            if Instant::now() >= deadline {
                panic!("helper {pid} survived page process exit");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        println!("PASS {engine}/parent_exit");
    }
}
