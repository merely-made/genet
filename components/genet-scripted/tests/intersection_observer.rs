//! `IntersectionObserver` from script through the geometry seam and the
//! rendering-turn update step, on both engines.

use genet_livery::Device;
use genet_scripted::LiveryCssom;
use genet_static_dom::StaticDocument;
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

const CSS: &str = "body { margin: 0; } \
    #above { height: 100px; } \
    #target { width: 50px; height: 40px; } \
    #filler { height: 1000px; }";

const HTML: &str = "<!DOCTYPE html><html><body>\
    <div id='above'></div><div id='target'></div><div id='filler'></div>\
    </body></html>";

struct Page<E: ScriptEngine> {
    runtime: Runtime<E>,
    cssom: LiveryCssom,
    clock: f64,
}

impl<E: ScriptEngine> Page<E> {
    fn new() -> Self {
        let mut runtime = Runtime::<E>::new().expect("runtime");
        runtime.load_dom(&StaticDocument::parse(HTML));
        let cssom = LiveryCssom::install(&mut runtime, &[CSS], Device::screen(200.0, 100.0));
        cssom.frame(&mut runtime, 200, 100).expect("first frame");
        runtime.set_viewport_size(200.0, 100.0);
        Self {
            runtime,
            cssom,
            clock: 0.0,
        }
    }

    fn eval(&mut self, source: &str) -> Vec<String> {
        self.runtime.host().borrow_mut().console.clear();
        self.runtime.eval(source).expect("script");
        self.runtime.host().borrow().console.clone()
    }

    /// One rendering turn: the update step, then the task that delivers.
    fn turn(&mut self) -> usize {
        let queued = self
            .runtime
            .update_intersection_observations()
            .expect("update step");
        self.clock += 16.0;
        self.runtime.run_timers(64, self.clock).expect("timers");
        self.runtime.run_microtasks();
        queued
    }
}

fn init_is_validated_with_the_specified_errors<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    let out = page.eval(
        "function kind(f) { try { f(); return 'none'; } catch (e) { return e.name + (e.code ? ':' + e.code : ''); } } \
         var cb = function() {}; \
         console.log([ \
           kind(function() { new IntersectionObserver(cb, { threshold: [1.1] }); }), \
           kind(function() { new IntersectionObserver(cb, { threshold: ['foo'] }); }), \
           kind(function() { new IntersectionObserver(cb, { rootMargin: '1' }); }), \
           kind(function() { new IntersectionObserver(cb, { rootMargin: '2em' }); }), \
           kind(function() { new IntersectionObserver(cb, { rootMargin: '1px 1px 1px 1px 1px' }); }), \
           kind(function() { new IntersectionObserver(cb, { root: {} }); }), \
           kind(function() { new IntersectionObserver(); }), \
           kind(function() { new IntersectionObserver(1); }), \
           kind(function() { new IntersectionObserver(cb).observe('x'); }), \
           kind(function() { new IntersectionObserver(cb, { trackVisibility: true }); }), \
           kind(function() { IntersectionObserver(cb); }) \
         ].join('|'));",
    );
    assert_eq!(
        out[0],
        "RangeError|TypeError|SyntaxError:12|SyntaxError:12|SyntaxError:12|TypeError|TypeError|TypeError|TypeError|NotSupportedError:9|TypeError"
    );
}

fn attributes_report_the_normalised_init<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    let out = page.eval(
        "var o = new IntersectionObserver(function() {}); \
         console.log([o.root, o.rootMargin, o.scrollMargin, o.thresholds.join(), o.delay, o.trackVisibility].join('|')); \
         var p = new IntersectionObserver(function() {}, { \
           root: document.getElementById('above'), rootMargin: '10% 20px', threshold: [1, 0, 0.5], rootMargin: '10% 20px' }); \
         console.log([p.root.id, p.rootMargin, p.thresholds.join(), Object.isFrozen(p.thresholds), p.thresholds === p.thresholds].join('|')); \
         var q = new IntersectionObserver(function() {}, { rootMargin: ' ', threshold: [] }); \
         console.log(q.rootMargin + '|' + q.thresholds.join());",
    );
    assert_eq!(out[0], "|0px 0px 0px 0px|0px 0px 0px 0px|0|0|false");
    assert_eq!(out[1], "above|10% 20px 10% 20px|0,0.5,1|true|true");
    assert_eq!(out[2], "0px 0px 0px 0px|0");
}

fn the_first_update_reports_every_target_once<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    page.eval(
        "var log = []; \
         globalThis.o = new IntersectionObserver(function(entries, observer) { \
           for (var e of entries) log.push([e.target.id, e.isIntersecting, e.intersectionRatio, \
             e.boundingClientRect.top, e.intersectionRect.height, e.rootBounds.height, e.isVisible, \
             typeof e.time, observer === o, this === o].join()); \
         }); \
         o.observe(document.getElementById('target')); \
         o.observe(document.getElementById('above')); \
         o.observe(document.getElementById('target'));",
    );
    assert_eq!(page.turn(), 2, "one entry per distinct target");
    let out = page.eval("console.log(log.join('|'));");
    // `#target` starts at y = 100, edge-adjacent to the 100px viewport: it
    // intersects with zero area. `#above` fills the viewport.
    assert_eq!(
        out[0],
        "target,true,0,100,0,100,false,number,true,true|above,true,1,0,100,100,false,number,true,true"
    );
    assert_eq!(page.turn(), 0, "nothing moved, nothing reported");
}

fn scrolling_crosses_thresholds_and_stops_when_nothing_changes<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    page.eval(
        "var log = []; \
         var o = new IntersectionObserver(function(entries) { \
           for (var e of entries) log.push(e.isIntersecting + ':' + e.intersectionRatio); \
         }, { threshold: [0, 0.5, 1] }); \
         o.observe(document.getElementById('target'));",
    );
    page.turn();
    page.eval("window.scrollTo(0, 20);");
    assert_eq!(page.turn(), 1, "half the target shows after scrolling 20");
    page.eval("window.scrollTo(0, 40);");
    assert_eq!(page.turn(), 1, "all of it shows");
    page.eval("window.scrollTo(0, 41);");
    assert_eq!(page.turn(), 0, "still wholly visible: same state, no entry");
    page.eval("window.scrollTo(0, 400);");
    assert_eq!(page.turn(), 1, "scrolled out");
    let out = page.eval("console.log(log.join('|'));");
    assert_eq!(out[0], "true:0|true:0.5|true:1|false:0");
}

fn takerecords_and_disconnect_and_unobserve<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    page.eval(
        "var delivered = 0; \
         globalThis.o = new IntersectionObserver(function() { delivered++; }); \
         o.observe(document.getElementById('target')); \
         o.observe(document.getElementById('above'));",
    );
    // The update step queues; takeRecords empties the queue before delivery.
    page.runtime
        .update_intersection_observations()
        .expect("update");
    let out = page.eval(
        "var taken = o.takeRecords(); \
         console.log(taken.length + ',' + o.takeRecords().length + ',' + (taken[0] instanceof IntersectionObserverEntry));",
    );
    assert_eq!(out[0], "2,0,true");
    page.turn();
    assert_eq!(page.eval("console.log(delivered);")[0], "0");
    page.eval("o.unobserve(document.getElementById('target')); window.scrollTo(0, 400);");
    page.turn();
    page.eval("o.disconnect(); window.scrollTo(0, 0);");
    assert_eq!(page.turn(), 0, "a disconnected observer is not updated");
}

fn removed_and_hidden_targets_report_not_intersecting<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    page.eval(
        "var log = []; \
         var o = new IntersectionObserver(function(entries) { \
           for (var e of entries) log.push(e.isIntersecting + ':' + e.boundingClientRect.width); \
         }); \
         globalThis.t = document.getElementById('above'); o.observe(t);",
    );
    page.turn();
    page.eval("t.style.display = 'none';");
    page.turn();
    page.eval("t.style.display = 'block'; document.body.removeChild(t);");
    page.turn();
    let out = page.eval("console.log(log.join('|'));");
    assert_eq!(out[0], "true:200|false:0");
}

fn entries_construct_from_an_init_and_keep_the_idl_shape<E: ScriptEngine>() {
    let mut page = Page::<E>::new();
    let out = page.eval(
        "function kind(f) { try { f(); return 'none'; } catch (e) { return e.name; } } \
         var init = { time: 1, rootBounds: null, boundingClientRect: { width: 2 }, \
           intersectionRect: {}, isIntersecting: true, isVisible: false, \
           intersectionRatio: 0.5, target: document.body }; \
         var e = new IntersectionObserverEntry(init); \
         console.log([e.time, e.rootBounds, e.boundingClientRect.width, e.boundingClientRect instanceof DOMRectReadOnly, \
           e.isIntersecting, e.intersectionRatio, e.target === document.body, String(e)].join('|')); \
         console.log([kind(function() { new IntersectionObserverEntry(); }), \
           kind(function() { new IntersectionObserverEntry({}); }), \
           kind(function() { IntersectionObserverEntry.prototype.time; })].join('|')); \
         console.log([IntersectionObserver.length, IntersectionObserver.prototype.observe.length, \
           Object.prototype.toString.call(new IntersectionObserver(function() {})), \
           typeof Object.getOwnPropertyDescriptor(IntersectionObserver.prototype, 'root').get].join('|'));",
    );
    assert_eq!(out[0], "1||2|true|true|0.5|true|[object IntersectionObserverEntry]");
    assert_eq!(out[1], "TypeError|TypeError|TypeError");
    assert_eq!(out[2], "1|1|[object IntersectionObserver]|function");
}

macro_rules! both_engines {
    ($name:ident) => {
        mod $name {
            #[test]
            fn boa() {
                super::$name::<script_engine_boa::BoaEngine>();
            }
            #[cfg(feature = "scripted-nova")]
            #[test]
            fn nova() {
                super::$name::<script_engine_nova::NovaEngine>();
            }
        }
    };
}

both_engines!(init_is_validated_with_the_specified_errors);
both_engines!(attributes_report_the_normalised_init);
both_engines!(the_first_update_reports_every_target_once);
both_engines!(scrolling_crosses_thresholds_and_stops_when_nothing_changes);
both_engines!(takerecords_and_disconnect_and_unobserve);
both_engines!(removed_and_hidden_targets_report_not_intersecting);
both_engines!(entries_construct_from_an_init_and_keep_the_idl_shape);
