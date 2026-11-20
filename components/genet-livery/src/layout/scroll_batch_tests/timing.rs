// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The release phase breakdown of one admitted scroll batch on a document the
//! size of Graphshell's table, against a fresh complete layout of the same
//! final DOM. Run with
//! `cargo test --release -p genet-livery --lib scroll_batch_release_breakdown -- --ignored --nocapture`.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDomMut, QualName};

use super::super::scroll_batch::take_phases;
use super::super::*;
use super::harness::{HEIGHT, WIDTH, ahem_text_system, attr, by_id};
use crate::{IncrementalStyle, InteractionStates, StyleSet, resolve_styles};

const WINDOW: usize = 30;
const CELLS: usize = 6;
const ROW: usize = 24;
const CHROME_ITEMS: usize = 110;

const CSS: &str = "html, body { margin: 0; padding: 0; font: 10px monospace; } \
    #side { width: 380px; } .item { height: 12px; } \
    #body { position: relative; overflow: hidden; width: 380px; height: 240px; } \
    #rows { position: absolute; left: 0; width: 380px; } \
    .row { position: absolute; left: 0; width: 360px; height: 24px; } \
    .cell { position: absolute; top: 0; width: 60px; height: 24px; }";

fn row_html(index: usize) -> String {
    let cells = (0..CELLS)
        .map(|cell| {
            format!(
                "<div class=cell style=\"left: {}px\">r{index} c{cell}</div>",
                cell * 60
            )
        })
        .collect::<String>();
    format!(
        "<div class=row id=r{index} style=\"top: {}px\">{cells}</div>",
        index * ROW
    )
}

fn document_html(first: usize) -> String {
    let chrome = (0..CHROME_ITEMS)
        .map(|item| format!("<div class=item><span>item</span><span>{item}</span></div>"))
        .collect::<String>();
    let rows = (first..first + WINDOW).map(row_html).collect::<String>();
    format!(
        "<html><body><div id=side>{chrome}</div><div id=body>\
         <div id=rows style=\"top: -{}px\">{rows}</div></div></body></html>",
        first * ROW
    )
}

fn div(dom: &mut ScriptedDom) -> NodeId {
    dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from("div"),
    ))
}

/// One scroll frame as the table sends it.
fn scroll_frame(dom: &mut ScriptedDom, first: usize) {
    let strip = by_id(dom, "rows");
    dom.set_attribute(
        strip,
        attr("style"),
        &format!("top: -{}px", (first + 1) * ROW),
    );
    let leaving = by_id(dom, &format!("r{first}"));
    dom.remove(leaving);
    let index = first + WINDOW;
    let row = div(dom);
    dom.set_attribute(row, attr("class"), "row");
    dom.set_attribute(row, attr("id"), &format!("r{index}"));
    dom.set_attribute(row, attr("style"), &format!("top: {}px", index * ROW));
    for cell_index in 0..CELLS {
        let cell = div(dom);
        dom.set_attribute(cell, attr("class"), "cell");
        dom.set_attribute(cell, attr("style"), &format!("left: {}px", cell_index * 60));
        let text = dom.create_text(&format!("r{index} c{cell_index}"));
        dom.append_child(cell, text);
        dom.append_child(row, cell);
    }
    dom.append_child(strip, row);
    for kept in first + 1..first + 1 + WINDOW {
        let row = by_id(dom, &format!("r{kept}"));
        dom.set_attribute(row, attr("aria-rowindex"), &kept.to_string());
        dom.set_attribute(row, attr("data-row"), &kept.to_string());
        dom.set_attribute(row, attr("tabindex"), "-1");
    }
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

#[test]
#[ignore = "release timing receipt; run by hand with --release"]
fn scroll_batch_release_breakdown() {
    const WARMUP: usize = 5;
    const FRAMES: usize = 101;
    let mut dom = ScriptedDom::from_serialized_document(&document_html(0));
    dom.drain_mutations(&mut Vec::new());
    let elements = {
        let mut count = 0;
        let mut pending = vec![dom.document()];
        while let Some(node) = pending.pop() {
            count += usize::from(dom.kind(node) == NodeKind::Element);
            pending.extend(dom.dom_children(node));
        }
        count
    };
    let styles_set = StyleSet::cambium(&[CSS]);
    let device = Device::screen(WIDTH, HEIGHT);
    let viewport = ViewportSizes::uniform(WIDTH, HEIGHT);
    let images = HashMap::<String, Vec<u8>>::new();
    let mut text = ahem_text_system();
    let states = InteractionStates::default();
    let mut session = IncrementalStyle::new();
    session.update(&dom, &styles_set, &device, &states, &[]);
    let cascade = resolve_styles(&dom, &styles_set, &device, &states);
    let (mut styles, mut layout) =
        layout_with_text_system(&dom, &cascade, WIDTH, HEIGHT, viewport, &mut text, &images)
            .unwrap();

    let mut retained = Vec::new();
    let mut control = Vec::new();
    let mut phases: BTreeMap<&'static str, Vec<Duration>> = BTreeMap::new();
    let mut order = Vec::new();
    let mut mutation_count = 0;
    for frame in 0..WARMUP + FRAMES {
        scroll_frame(&mut dom, frame);
        let mut mutations: Vec<DomMutation<NodeId>> = Vec::new();
        dom.drain_mutations(&mut mutations);
        session.update(&dom, &styles_set, &device, &states, &mutations);
        let cascade = session.styles();
        take_phases();
        let started = Instant::now();
        let admitted = layout
            .relayout_borrowed_dom(
                &dom,
                &styles_set,
                cascade,
                &mut styles,
                session.restyled_elements(),
                viewport,
                &mutations,
                &mut text,
                &images,
            )
            .unwrap();
        let elapsed = started.elapsed();
        assert!(admitted, "frame {frame} takes the route");
        let started = Instant::now();
        let fresh =
            layout_with_text_system(&dom, cascade, WIDTH, HEIGHT, viewport, &mut text, &images)
                .unwrap();
        let fresh_elapsed = started.elapsed();
        drop(fresh);
        let mut frame_phases: BTreeMap<&'static str, Duration> = BTreeMap::new();
        for (name, duration) in take_phases() {
            if !order.contains(&name) {
                order.push(name);
            }
            *frame_phases.entry(name).or_default() += duration;
        }
        if frame >= WARMUP {
            mutation_count = mutations.len();
            retained.push(elapsed);
            control.push(fresh_elapsed);
            for (name, duration) in frame_phases {
                phases.entry(name).or_default().push(duration);
            }
        }
    }
    // What the whole-plane style resolve is made of, on the final DOM.
    let cascade = resolve_styles(&dom, &styles_set, &device, &InteractionStates::default());
    let (mut clone, mut ch, mut relative) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..FRAMES {
        let started = Instant::now();
        let mut plane = cascade.clone();
        clone.push(started.elapsed());
        let started = Instant::now();
        plane.resolve_ch_lengths(&mut text, viewport);
        ch.push(started.elapsed());
        let started = Instant::now();
        resolve_relative_subtree(
            &dom,
            dom.document(),
            &mut plane,
            livery::values::RelativeLengthEnvironment::container_fallback(viewport),
        );
        relative.push(started.elapsed());
    }
    for (name, samples) in [
        ("  plane clone", clone),
        ("  ch lengths", ch),
        ("  relative walk", relative),
    ] {
        eprintln!("{name:>16} {:>9.1} us", median(samples).as_secs_f64() * 1e6);
    }
    let total = median(retained);
    eprintln!("{elements} elements, {mutation_count} mutations a frame, median of {FRAMES} frames");
    for name in order {
        let value = median(phases[name].clone());
        eprintln!(
            "{name:>16} {:>9.1} us {:>5.1}%",
            value.as_secs_f64() * 1e6,
            100.0 * value.as_secs_f64() / total.as_secs_f64()
        );
    }
    eprintln!(
        "{:>16} {:>9.1} us",
        "retained total",
        total.as_secs_f64() * 1e6
    );
    eprintln!(
        "{:>16} {:>9.1} us",
        "fresh layout",
        median(control).as_secs_f64() * 1e6
    );
}
