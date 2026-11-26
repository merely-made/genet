use genet_scripted::ScriptedDocument;
use script_engine_boa::BoaEngine;
use script_runtime_api::{ScriptResourceLoader, WorkerPolicy};
use std::time::{Duration, Instant};

struct WorkerScript;

// (module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))
impl ScriptResourceLoader for WorkerScript {
    fn load(&self, url: &str) -> Option<String> {
        (url == "https://probe.test/wasm-worker.js").then(|| {
            "var m = new WebAssembly.Module(new Uint8Array([0,97,115,109,1,0,0,0,1,7,1,96,2,\
             127,127,1,127,3,2,1,0,7,7,1,3,97,100,100,0,0,10,9,1,7,0,32,0,32,1,106,11]));\
             postMessage(String(new WebAssembly.Instance(m).exports.add(40, 2)));"
                .to_owned()
        })
    }
}

#[test]
fn dedicated_worker_calls_webassembly() {
    let mut doc = ScriptedDocument::<BoaEngine>::parse("<html><body>before</body></html>")
        .expect("headless document");
    doc.set_worker_policy(WorkerPolicy::TrustedCooperative {
        resource_timeout: Duration::from_secs(5),
        shutdown_grace: Duration::from_millis(500),
    });
    doc.set_script_resource_loader(Box::new(WorkerScript));
    doc.evaluate(
        "var worker = new Worker('https://probe.test/wasm-worker.js');\
         worker.onmessage = event => document.body.textContent = event.data;",
    )
    .expect("start worker");

    let started = Instant::now();
    let mut now_ms = 0.0;
    while started.elapsed() < Duration::from_secs(5) {
        doc.pump(now_ms);
        if doc.dom_snapshot().contains("<body>42</body>") {
            return;
        }
        now_ms += 16.0;
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!(
        "worker never reported the wasm result: {}",
        doc.dom_snapshot()
    );
}
