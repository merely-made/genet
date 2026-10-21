//! Element scrolling and the geometry seam, from script down to the retained
//! Livery frame, on both engines.

use genet_livery::Device;
use genet_scripted::LiveryCssom;
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

const CSS: &str = "body { margin: 0; } \
    #box { width: 100px; height: 50px; margin-left: 20px; margin-top: 10px; } \
    #scroller { overflow: hidden; width: 100px; height: 100px; border: 5px solid black; \
                padding: 3px; margin-left: 20px; } \
    #tall { height: 300px; width: 80px; } \
    #gone { display: none; } \
    #filler { height: 1000px; }";

const HTML: &str = "<!DOCTYPE html><html><body>\
    <div id='box'></div>\
    <div id='scroller'><div id='tall'></div></div>\
    <div id='gone'></div>\
    <div id='filler'></div></body></html>";

fn setup<E: ScriptEngine>() -> (Runtime<E>, LiveryCssom) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.load_dom(&StaticDocument::parse(HTML));
    let cssom = LiveryCssom::install(&mut runtime, &[CSS], Device::screen(400.0, 300.0));
    cssom.frame(&mut runtime, 400, 300).expect("first frame");
    runtime.set_viewport_size(400.0, 300.0);
    (runtime, cssom)
}

fn find_by_id(
    dom: &genet_scripted_dom::ScriptedDom,
    from: genet_scripted_dom::NodeId,
    id: &str,
) -> Option<genet_scripted_dom::NodeId> {
    if dom.attribute(from, &Namespace::default(), &LocalName::from("id")) == Some(id) {
        return Some(from);
    }
    dom.dom_children(from)
        .find_map(|child| find_by_id(dom, child, id))
}

fn run<E: ScriptEngine>(runtime: &mut Runtime<E>, source: &str) -> Vec<String> {
    runtime.host().borrow_mut().console.clear();
    runtime.eval(source).expect("script");
    runtime.host().borrow().console.clone()
}

fn boxes_answer_from_the_retained_frame<E: ScriptEngine>() {
    let (mut runtime, _cssom) = setup::<E>();
    let out = run(
        &mut runtime,
        "var r = document.getElementById('box').getBoundingClientRect(); \
         console.log([r.x, r.y, r.width, r.height].join()); \
         var b = document.getElementById('box'); \
         console.log([b.offsetLeft, b.offsetTop, b.offsetWidth, b.offsetHeight, \
                      b.offsetParent === document.body, b.clientWidth].join()); \
         var g = document.getElementById('gone'); \
         console.log([g.getClientRects().length, g.getBoundingClientRect().width, \
                      g.offsetParent, g.offsetWidth].join());",
    );
    assert_eq!(out[0], "20,10,100,50");
    assert_eq!(out[1], "20,10,100,50,true,100");
    assert_eq!(out[2], "0,0,,0", "no box: empty rects, zero width, null offsetParent");
}

fn geometry_follows_script_mutation_without_a_frame<E: ScriptEngine>() {
    let (mut runtime, _cssom) = setup::<E>();
    let out = run(
        &mut runtime,
        "var b = document.getElementById('box'); \
         b.style.width = '160px'; \
         console.log(b.getBoundingClientRect().width + ',' + b.offsetWidth);",
    );
    assert_eq!(out[0], "160,160");
}

fn scroll_containers_report_client_and_scroll_boxes<E: ScriptEngine>() {
    let (mut runtime, _cssom) = setup::<E>();
    let out = run(
        &mut runtime,
        "var s = document.getElementById('scroller'); \
         console.log([s.clientTop, s.clientLeft, s.clientWidth, s.clientHeight, \
                      s.scrollWidth, s.scrollHeight, s.scrollTop, s.scrollLeft].join());",
    );
    // content 100 + padding 3 each side = 106 client box; 3 + 300 + 3 scrollable.
    assert_eq!(out[0], "5,5,106,106,106,306,0,0");
}

fn element_scroll_clamps_moves_boxes_and_reads_back<E: ScriptEngine>() {
    let (mut runtime, _cssom) = setup::<E>();
    let out = run(
        &mut runtime,
        "var s = document.getElementById('scroller'), t = document.getElementById('tall'); \
         var before = t.getBoundingClientRect().y; \
         s.scrollTop = 40; \
         console.log(s.scrollTop + ',' + (before - t.getBoundingClientRect().y)); \
         s.scrollTop = 99999; \
         console.log(s.scrollTop); \
         s.scrollTo(0, 10); s.scrollBy(0, 5); \
         console.log(s.scrollTop); \
         s.scrollTo({ top: 7 }); \
         console.log(s.scrollTop + ',' + s.scrollLeft); \
         t.scrollTop = 5; console.log(t.scrollTop);",
    );
    assert_eq!(out, vec!["40,40", "200", "15", "7,0", "0"]);
}

fn scrolled_container_reaches_paint_and_hit_test<E: ScriptEngine>() {
    let (mut runtime, cssom) = setup::<E>();
    // `tall` starts at viewport y = 10 + 50 + 5 border + 3 padding = 68 and runs
    // 300 down; the probe sits in the scroller's bottom padding at y = 170.
    let hit_id = |runtime: &Runtime<E>| {
        let node = cssom.hit_test(runtime, 30.0, 170.0)?;
        let host = runtime.host().borrow();
        host.dom
            .attribute(node, &Namespace::default(), &LocalName::from("id"))
            .map(str::to_owned)
    };
    assert_eq!(hit_id(&runtime).as_deref(), Some("tall"));
    run(
        &mut runtime,
        "document.getElementById('scroller').scrollTop = 200;",
    );
    cssom.frame(&mut runtime, 400, 300).expect("scrolled frame");
    // The child ends at 368 - 200 = 168, above the probe in the container's padding.
    assert_eq!(hit_id(&runtime).as_deref(), Some("scroller"));
    let tall = {
        let host = runtime.host().borrow();
        find_by_id(&host.dom, host.dom.document(), "tall").expect("tall element")
    };
    let rect = cssom.fragment_rect(tall).expect("fragment");
    assert_eq!(rect[1], 68.0 - 200.0, "queries subtract the scroll above");
}

fn viewport_scrolling_reaches_livery<E: ScriptEngine>() {
    let (mut runtime, cssom) = setup::<E>();
    let out = run(
        &mut runtime,
        "window.scrollTo(0, 200); \
         console.log(window.scrollY + ',' + document.documentElement.scrollTop); \
         window.scrollBy(0, 20); \
         console.log(window.scrollY); \
         window.scrollTo(0, 99999); \
         console.log(window.scrollY); \
         document.documentElement.scrollTop = 0; \
         console.log(window.scrollY); \
         console.log(document.scrollingElement === document.documentElement); \
         console.log(document.documentElement.clientHeight + ',' + document.documentElement.clientWidth);",
    );
    assert_eq!(out[0], "200,200");
    assert_eq!(out[1], "220");
    // 10 + 50 + 116 + 1000 content against a 300 high viewport.
    let max: f32 = out[2].parse().expect("number");
    assert!(max > 800.0 && max < 1200.0, "clamped to the content: {max}");
    assert_eq!(out[3], "0");
    assert_eq!(out[4], "true");
    assert_eq!(out[5], "300,400");
    assert_eq!(cssom.scroll(), (0.0, 0.0));
}

macro_rules! both_engines {
    ($name:ident) => {
        mod $name {
            use super::*;
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

both_engines!(boxes_answer_from_the_retained_frame);
both_engines!(geometry_follows_script_mutation_without_a_frame);
both_engines!(scroll_containers_report_client_and_scroll_boxes);
both_engines!(element_scroll_clamps_moves_boxes_and_reads_back);
both_engines!(scrolled_container_reaches_paint_and_hit_test);
both_engines!(viewport_scrolling_reaches_livery);

fn inline_boxes_follow_their_text_across_lines<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.load_dom(&StaticDocument::parse(
        "<!DOCTYPE html><html><body><div id=c style='width:60px'>\
         <span id=s>aaa bbb ccc ddd</span></div></body></html>",
    ));
    let cssom = LiveryCssom::install(&mut runtime, &["body{margin:0}"], Device::screen(300.0, 300.0));
    cssom.frame(&mut runtime, 300, 300).expect("frame");
    let out = run(
        &mut runtime,
        "var s = document.getElementById('s'); \
         var rects = s.getClientRects(), box = s.getBoundingClientRect(); \
         console.log([rects.length > 1, box.height > rects[0].height, \
                      s.offsetHeight === Math.round(box.height), s.offsetTop === 0].join());",
    );
    assert_eq!(out[0], "true,true,true,true");
}

both_engines!(inline_boxes_follow_their_text_across_lines);
