/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Where the full projection's time goes on the table fixture. Run with
//! `cargo test --release -p genet-render --lib a11y_stage_breakdown -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom, NodeKind};

use super::table_fixture::*;
use super::*;
use crate::fragments_from_scripted_dom;

const RUNS: u32 = 200;

fn timed<T>(mut f: impl FnMut() -> T) -> Duration {
    let start = Instant::now();
    for _ in 0..RUNS {
        std::hint::black_box(f());
    }
    start.elapsed() / RUNS
}

/// Full projection against the session's patch, over table scroll frames.
#[test]
#[ignore = "release timing; run by hand"]
fn a11y_session_scroll_batch_timing() {
    const FRAMES: usize = 100;
    let mut host = super::session_tests::Host::new(table_dom(500));
    let (mut patched, mut full, mut restyled) = (Duration::ZERO, Duration::ZERO, 0);
    for frame in 0..FRAMES {
        let (p, f, r) = host.timed(|dom| scroll_down(dom, 500 + frame));
        patched += p;
        full += f;
        restyled = r;
    }
    println!(
        "table scroll batch over {FRAMES} frames ({} nodes, restyled set {restyled}):\n\
         full projection {:?}/frame, session patch {:?}/frame",
        host.session.len(),
        full / FRAMES as u32,
        patched / FRAMES as u32,
    );
}

#[test]
#[ignore = "release timing; run by hand"]
fn a11y_stage_breakdown() {
    let dom = table_dom(500);
    let layout = fragments_from_scripted_dom(&dom, CSS, WIDTH, HEIGHT).expect("layout");
    let generated = |_: NodeId| (String::new(), String::new());
    let rendered = |_: NodeId| true;
    let style = A11yStyleQueries {
        generated: &generated,
        rendered: &rendered,
    };
    let names = || {
        name::Names::new(&dom)
            .with_generated(&generated)
            .with_rendered(&rendered)
    };
    let mut projected = Vec::new();
    let mut pending = vec![dom.document()];
    while let Some(node) = pending.pop() {
        if dom.kind(node) == NodeKind::Element && layout.get(node).is_some() {
            projected.push(node);
        }
        pending.extend(dom.flat_children(node));
    }
    let built = names();
    let full = timed(|| document_a11y_projection_with_style(&dom, &layout, None, 0, None, &style));
    let index = timed(names);
    let walk = timed(|| projection_with_names(&dom, &layout, None, 0, None, &built));
    let name = timed(|| projected.iter().map(|n| built.name(*n)).collect::<Vec<_>>());
    let description = timed(|| {
        projected
            .iter()
            .map(|n| built.description(*n))
            .collect::<Vec<_>>()
    });
    let semantics = timed(|| {
        projected
            .iter()
            .map(|n| {
                let role = document_role(&dom, *n);
                let state = neutral_state(&dom, *n, None);
                let numbers = aria_number(&dom, *n, "aria-valuenow");
                (role, neutral_actions(&dom, *n, role, state, false), numbers)
            })
            .collect::<Vec<_>>()
    });
    let bounds = timed(|| projected.iter().filter_map(|n| layout.get(*n)).count());
    // Rows take their names from their cells; the strip, a generic
    // container, takes none (D65).
    let strip = by_id(&dom, "strip");
    let rows: Vec<_> = dom.dom_children(strip).collect();
    let row_names = timed(|| rows.iter().map(|n| built.name(*n)).collect::<Vec<_>>());
    let strip_name = timed(|| built.name(strip));
    let nodes = projection_with_names(&dom, &layout, None, 0, None, &built);
    println!(
        "projected {} nodes ({} element boxes)\n\
         full with_style  {full:?}\n  index (Names::new) {index:?}\n  walk with index {walk:?}\n\
         per stage over projected nodes:\n  name {name:?}\n  description {description:?}\n\
         \x20 role+state+actions+numbers {semantics:?}\n  bounds lookup {bounds:?}\n\
         name of the 30 rows alone {row_names:?}; name of the strip alone {strip_name:?}\n\
         strip name length {} chars",
        nodes.nodes().len(),
        projected.len(),
        built.name(strip).map_or(0, |s| s.len()),
    );
}
