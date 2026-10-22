use genet_scripted::ScriptedDocument;
use script_engine_boa::BoaEngine;
use script_runtime_api::{ScriptResourceLoader, WorkerPolicy};
use std::time::{Duration, Instant};

struct WorkerScript;

impl ScriptResourceLoader for WorkerScript {
    fn load(&self, url: &str) -> Option<String> {
        (url == "https://probe.test/hidden-worker.js")
            .then(|| "onmessage = event => postMessage(event.data);".to_owned())
    }
}

#[test]
fn hidden_worker_reply_wakes_before_an_unrelated_long_timer() {
    let mut doc = ScriptedDocument::<BoaEngine>::parse("<html><body>before</body></html>")
        .expect("headless document");
    doc.set_worker_policy(WorkerPolicy::TrustedCooperative {
        resource_timeout: Duration::from_secs(5),
        shutdown_grace: Duration::from_millis(500),
    });
    doc.set_script_resource_loader(Box::new(WorkerScript));
    doc.evaluate(
        "setTimeout(() => console.log('long timer'), 1000000000);\
         var worker = new Worker('https://probe.test/hidden-worker.js');\
         worker.onmessage = event => document.body.textContent = event.data;\
         worker.postMessage('delivered');",
    )
    .expect("schedule long timer and worker reply");

    doc.set_hidden(true);
    assert_eq!(doc.next_wake_delay(), Some(0.0));
    doc.pump(0.0);
    assert_eq!(
        doc.next_wake_delay(),
        Some(1000.0),
        "pending worker delivery chooses the hidden admission deadline"
    );
    doc.pump(16.0);
    assert!(doc.dom_snapshot().contains("<body>before</body>"));

    let started = Instant::now();
    let mut now_ms = 1000.0;
    while started.elapsed() < Duration::from_secs(5) {
        doc.pump(now_ms);
        if doc.dom_snapshot().contains("<body>delivered</body>") {
            assert!(!doc.console().iter().any(|line| line == "long timer"));
            return;
        }
        now_ms += 1000.0;
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("hidden worker reply was not delivered at admitted turns");
}
