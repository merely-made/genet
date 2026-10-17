use genet_scripted::{ScriptedDocument, ScriptedDocumentOptions};
use script_engine_boa::BoaEngine;
use script_runtime_api::{ScriptResourceLoader, WorkerPolicy};
use std::time::{Duration, Instant};

struct Scripts;
impl ScriptResourceLoader for Scripts {
    fn load(&self, url: &str) -> Option<String> {
        eprintln!("worker resource request: {url}");
        (url == "https://probe.test/wake.js")
            .then(|| "onmessage = function(e) { postMessage('wake:' + e.data); };".to_owned())
    }
}

#[test]
fn headless_parser_worker_uses_preparse_policy() {
    let html = "<html><body>before<script>var w = new Worker('https://probe.test/wake.js'); w.onmessage = function(e) { document.body.textContent = e.data; }; w.postMessage('parser');</script></body></html>";
    let mut refused = ScriptedDocument::<BoaEngine>::parse(
        "<body><script>try { new Worker('https://probe.test/wake.js'); } catch (e) { document.body.setAttribute('data-worker-error', e.name); }</script></body>",
    ).unwrap();
    assert!(
        refused
            .dom_snapshot()
            .contains("data-worker-error=\"NotSupportedError\"")
    );
    let mut doc = ScriptedDocument::<BoaEngine>::parse_with_options(
        html,
        ScriptedDocumentOptions {
            worker_policy: WorkerPolicy::TrustedCooperative {
                resource_timeout: Duration::from_secs(30),
                shutdown_grace: Duration::from_millis(500),
            },
            ..ScriptedDocumentOptions::default()
        },
    )
    .unwrap();
    doc.set_script_resource_loader(Box::new(Scripts));
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(5) {
        doc.pump(started.elapsed().as_secs_f64() * 1000.0);
        if doc.dom_snapshot().contains("wake:parser") {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("parser-created trusted worker did not reply");
}

#[test]
fn document_pump_forwards_worker_service_and_waits_for_acknowledged_idle() {
    let mut doc = ScriptedDocument::<BoaEngine>::parse("<html><body>before</body></html>").unwrap();
    doc.set_worker_policy(WorkerPolicy::TrustedCooperative {
        resource_timeout: Duration::from_secs(30),
        shutdown_grace: Duration::from_millis(500),
    });
    doc.set_script_resource_loader(Box::new(Scripts));
    // Evaluated separately, so script marker text cannot contaminate the DOM.
    doc.evaluate("var w = new Worker('https://probe.test/wake.js'); w.onmessage = function(e) { document.body.textContent = e.data; Promise.resolve().then(function(){ document.body.setAttribute('data-reaction','done'); }); }; w.onerror = function(e) { document.body.textContent = 'worker-error:' + e.message; }; w.postMessage('one');").unwrap();
    assert_eq!(
        doc.dom_snapshot(),
        "<html><head></head><body>before</body></html>"
    );
    assert!(
        doc.has_pending_work(),
        "live worker is outstanding without a timer"
    );
    doc.freeze();
    assert!(
        !doc.has_pending_work(),
        "frozen documents advertise no drive"
    );
    doc.pump(0.0);
    assert_eq!(
        doc.dom_snapshot(),
        "<html><head></head><body>before</body></html>"
    );
    doc.resume();
    assert!(
        doc.has_pending_work(),
        "resume restores outstanding worker demand"
    );
    let expected = "<html><head></head><body data-reaction=\"done\">wake:one</body></html>";
    let started = Instant::now();
    let mut pumps = 0;
    while started.elapsed() < Duration::from_secs(30) && doc.dom_snapshot() != expected {
        doc.pump(started.elapsed().as_secs_f64() * 1000.0);
        pumps += 1;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        doc.dom_snapshot(),
        expected,
        "worker reply and its Promise reaction"
    );
    let idle_started = Instant::now();
    while idle_started.elapsed() < Duration::from_secs(30) && doc.has_pending_work() {
        doc.pump(started.elapsed().as_secs_f64() * 1000.0);
        pumps += 1;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!doc.has_pending_work(), "worker idle acknowledgement");
    println!(
        "delivery+idle pumps={pumps}, elapsed_ms={}",
        started.elapsed().as_millis()
    );
}

#[test]
fn missing_worker_route_reports_error_and_eventually_quiesces() {
    let mut doc = ScriptedDocument::<BoaEngine>::parse("<html><body>before</body></html>").unwrap();
    doc.set_worker_policy(WorkerPolicy::TrustedCooperative {
        resource_timeout: Duration::from_secs(30),
        shutdown_grace: Duration::from_millis(500),
    });
    doc.evaluate("var w = new Worker('https://probe.test/unavailable.js'); w.onerror = function() { document.body.textContent = 'route-error'; }; w.postMessage('one');").unwrap();
    assert_eq!(
        doc.dom_snapshot(),
        "<html><head></head><body>before</body></html>"
    );
    let started = Instant::now();
    let expected = "<html><head></head><body>route-error</body></html>";
    while started.elapsed() < Duration::from_secs(30) {
        doc.pump(started.elapsed().as_secs_f64() * 1000.0);
        if doc.dom_snapshot() == expected && !doc.has_pending_work() {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        doc.dom_snapshot(),
        expected,
        "missing route must report failure"
    );
    assert!(!doc.has_pending_work(), "failed worker must retire");
}

#[test]
fn hidden_worker_demand_waits_for_the_clamped_wake() {
    let mut doc = ScriptedDocument::<BoaEngine>::parse("<html><body>before</body></html>").unwrap();
    doc.set_worker_policy(WorkerPolicy::TrustedCooperative {
        resource_timeout: Duration::from_secs(30),
        shutdown_grace: Duration::from_millis(500),
    });
    doc.set_script_resource_loader(Box::new(Scripts));
    doc.evaluate("var w = new Worker('https://probe.test/wake.js'); w.onmessage = function(e) { document.body.textContent = e.data; }; w.postMessage('hidden');").unwrap();
    doc.set_hidden(true);
    assert_eq!(doc.next_wake_delay(), Some(0.0));
    doc.pump(0.0);
    assert_eq!(doc.next_wake_delay(), Some(1000.0));
    doc.pump(16.0);
    assert_eq!(
        doc.dom_snapshot(),
        "<html><head></head><body>before</body></html>"
    );
    let started = Instant::now();
    let mut now = 1000.0;
    while started.elapsed() < Duration::from_secs(5) {
        doc.pump(now);
        if doc.dom_snapshot().contains("wake:hidden") {
            return;
        }
        now += 1000.0;
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("hidden worker reply was not delivered after clamped wakes");
}
