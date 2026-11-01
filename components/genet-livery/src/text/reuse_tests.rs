use crate::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::ScriptedDom;

#[test]
fn shaping_stats_count_builds_separately_from_formats() {
    let dom = ScriptedDom::from_serialized_document(
        "<html><body><p>Plain text fixture</p></body></html>",
    );
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&["body { width: 30ch; }"]),
        Device::screen(640.0, 480.0),
    );
    document.frame(640, 480).expect("initial frame");
    let stats = document.text_system().shaping_stats();
    assert!(stats.format_requests > 0);
    assert!(stats.metric_layout_builds > 0);
    assert_eq!(
        stats.layout_builds,
        stats.format_requests + stats.metric_layout_builds
    );
    assert!(stats.line_breaks >= stats.format_requests);
    document.frame(640, 480).expect("cached frame");
    assert_eq!(document.text_system().shaping_stats(), stats);
}

#[test]
fn shaping_options_survive_font_resource_rebuild() {
    let dom = ScriptedDom::from_serialized_document("<html><body>test</body></html>");
    let mut document =
        LiveryDocument::new(dom, StyleSet::cambium(&[]), Device::screen(100.0, 100.0));
    let options = crate::TextShapingOptions {
        retained_entries: 7,
        retained_text_bytes: 256,
        parallel_min_groups: 3,
        parallel_min_text_bytes: 100,
        record_timings: true,
    };
    document.set_text_shaping_options(options);
    document.set_font_resource("resource", vec![1]);
    assert_eq!(document.text_system().shaping_options(), options);
}
