// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use script_engine_api::{ScriptEngine, ScriptEngineSnapshot};
use layout_dom_api::LayoutDom;
use script_runtime_api::{FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime};

fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, source: &str) -> String {
    let value = runtime.eval(source).expect("eval");
    runtime.value_to_string(&value).expect("string")
}

struct Deferred;
impl FetchHandler for Deferred {
    fn fetch(&self, _: FetchRequest) -> FetchOutcome {
        FetchOutcome::network_error()
    }

    fn start(
        &self,
        _: u64,
        _: FetchRequest,
    ) -> Option<FetchOutcome> {
        None
    }
}

fn author_replacement_cannot_steal_host_tasks<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.parse_document_interleaved("<body><button id=target>go</button></body>", &NoScriptLoader);
    let node = {
        let host = runtime.host().borrow();
        let root = host.dom.document();
        let mut pending = vec![root];
        loop {
            let node = pending.pop().expect("button node");
            if host.dom.element_name(node).is_some_and(|name| name.local.as_str() == "button") {
                break node;
            }
            pending.extend(host.dom.dom_children(node));
        }
    };
    runtime.eval("globalThis.events=[]; var target=document.getElementById('target'); target.addEventListener('keydown',function(){events.push('key');});
      target.dispatchEvent=function(){throw new Error('stolen node dispatch');};
      globalThis.Event=function(){throw new Error('stolen event constructor');};
      __dispatchHostEvent=function(){throw new Error('stolen dispatch');};
      __runTimers=function(){throw new Error('stolen timers');};
      __nextTimerDelay=function(){throw new Error('stolen delay');};
      __runOneAnimationFrameCallback=function(){throw new Error('stolen frame');};
      __hasAnimationFrameCallbacks=function(){throw new Error('stolen pending');};
      setTimeout(function(){events.push('timer');},0);
      requestAnimationFrame(function(){events.push('frame');});").unwrap();
    assert!(runtime.dispatch_event(node.raw(), "keydown").unwrap());
    assert_eq!(runtime.run_timers(1, 0.0).unwrap(), 1);
    assert!(runtime.has_animation_frame_callbacks().unwrap());
    assert_eq!(runtime.run_animation_frame_callbacks(16.0).unwrap(), 1);
    assert_eq!(read(&mut runtime, "events.join(',')"), "key,timer,frame");
    assert_eq!(runtime.next_timer_delay().unwrap(), None);
}

fn thrown_timer_is_reported_and_next_task_runs<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.eval("globalThis.done=0; setTimeout(function(){throw new Error('timer boom');},0); setTimeout(function(){done=1;},0);").unwrap();
    assert_eq!(runtime.run_timers(2, 0.0).unwrap(), 2);
    assert_eq!(read(&mut runtime, "String(done)"), "1");
    assert!(runtime.host().borrow().console.iter().any(|line| line.contains("timer boom")));
}

fn fetch_completion_uses_requesting_realm_hook<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.set_fetch_handler(Box::new(Deferred));
    runtime.eval("globalThis.fetchFailure=''; fetch('https://example.test/').catch(function(e){fetchFailure=e.message;}); __fetchFail=function(){throw new Error('stolen fetch');};").unwrap();
    let id: u64 = read(&mut runtime, "Object.keys(__pending)[0]").parse().unwrap();
    runtime.fail_fetch(id, "delivered").unwrap();
    assert_eq!(read(&mut runtime, "fetchFailure"), "delivered");
    assert_eq!(runtime.pending_fetches(), 0);
}

fn snapshot_keeps_private_hooks<E: ScriptEngine + ScriptEngineSnapshot>() {
    let mut donor = Runtime::<E>::new().expect("runtime");
    donor.eval("__runTimers=function(){throw new Error('stolen');};").unwrap();
    let mut clone = donor.snapshot_clone().expect("snapshot");
    clone.eval("globalThis.fired=0; setTimeout(function(){fired++;},0); __runTimers=function(){throw new Error('clone stolen');};").unwrap();
    assert_eq!(clone.run_timers(1, 0.0).unwrap(), 1);
    assert_eq!(read(&mut clone, "String(fired)"), "1");
    donor.eval("globalThis.fired=0; setTimeout(function(){fired++;},0);").unwrap();
    assert_eq!(donor.run_timers(1, 0.0).unwrap(), 1);
    assert_eq!(read(&mut donor, "String(fired)"), "1");
}

fn mutation_checkpoint_uses_retained_hook<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.parse_document_interleaved("<body></body>", &NoScriptLoader);
    runtime.eval("globalThis.observed=0; new MutationObserver(function(records){observed+=records.length;}).observe(document.body,{childList:true}); document.body.appendChild(document.createElement('span')); __moPump=function(){throw new Error('stolen mutation pump');};").unwrap();
    runtime.run_microtasks();
    assert_eq!(read(&mut runtime, "String(observed)"), "1");
}

fn performance_observer_uses_retained_scheduler_and_reporter<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.eval("globalThis.observed=0; new PerformanceObserver(function(list){observed=list.getEntries().length; throw new Error('observer boom');}).observe({entryTypes:['mark']}); setTimeout=function(){throw new Error('stolen scheduler');}; __reportListenerException=function(){throw new Error('stolen reporter');}; performance.mark('entry');").unwrap();
    assert_eq!(runtime.run_timers(1, 0.0).unwrap(), 1);
    assert_eq!(read(&mut runtime, "String(observed)"), "1");
    assert!(runtime.host().borrow().console.iter().any(|line| line.contains("observer boom")));
}

#[test]
fn retained_task_hooks_on_boa() {
    author_replacement_cannot_steal_host_tasks::<script_engine_boa::BoaEngine>();
    thrown_timer_is_reported_and_next_task_runs::<script_engine_boa::BoaEngine>();
    fetch_completion_uses_requesting_realm_hook::<script_engine_boa::BoaEngine>();
    mutation_checkpoint_uses_retained_hook::<script_engine_boa::BoaEngine>();
    performance_observer_uses_retained_scheduler_and_reporter::<script_engine_boa::BoaEngine>();
}

#[cfg(target_pointer_width = "64")]
#[test]
fn retained_task_hooks_on_nova() {
    author_replacement_cannot_steal_host_tasks::<script_engine_nova::NovaEngine>();
    thrown_timer_is_reported_and_next_task_runs::<script_engine_nova::NovaEngine>();
    fetch_completion_uses_requesting_realm_hook::<script_engine_nova::NovaEngine>();
    mutation_checkpoint_uses_retained_hook::<script_engine_nova::NovaEngine>();
    performance_observer_uses_retained_scheduler_and_reporter::<script_engine_nova::NovaEngine>();
    snapshot_keeps_private_hooks::<script_engine_nova::NovaEngine>();
}
