// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Deterministic document-frame shaping control. Run with `--help` for options.
//! Pass the full source commit used to build the executable as `--source-revision`.
//! This caller-supplied label is not verified; hash the executable externally.
//! Example: `measure_text_shaping --source-revision <full-commit> --font <font-file>`.
use genet_livery::{Device, LiveryDocument, StyleSet, TextShapingOptions};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use paint_list_api::{PaintEnvelope, PaintList};
use std::{
    env, fs,
    io::{self, Write},
    time::Instant,
};

fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn find(dom: &ScriptedDom, node: NodeId) -> Option<NodeId> {
    if dom.attribute(node, &Namespace::default(), &LocalName::from("id")) == Some("edit") {
        return Some(node);
    }
    dom.dom_children(node).find_map(|child| find(dom, child))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut mode = "uncached-serial".to_owned();
    let mut case = "article".to_owned();
    let mut paragraphs = 32usize;
    let mut iterations = 30usize;
    let mut warmups = 5usize;
    let mut protocol = "cold".to_owned();
    let mut step = false;
    let mut workers = 1usize;
    let mut source_revision = String::new();
    let mut font = String::new();
    let mut font_dump = String::new();
    let mut family = "Fixture".to_owned();
    let mut output = String::new();
    let mut options = TextShapingOptions {
        record_timings: true,
        ..Default::default()
    };
    while let Some(key) = args.next() {
        if key == "--help" {
            println!(
                "--source-revision FULL_GIT_COMMIT --mode uncached-serial|cached-serial|cached-parallel --case small|article|mixed|single-long|long-short|atomic|churn --paragraphs N --workers N --iterations N --warmups N --protocol cold|unchanged|width|edit|churn --step 0|1 --record-timings 0|1 --font PATH --font-dump DIRECTORY --font-family NAME --cache-entries N --cache-text-bytes N --cache-layout-bytes N --parallel-min-groups N --parallel-min-text-bytes N --output PATH"
            );
            return Ok(());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        match key.as_str() {
            "--source-revision" => source_revision = value,
            "--mode" => mode = value,
            "--case" => case = value,
            "--paragraphs" => paragraphs = value.parse()?,
            "--workers" => workers = value.parse()?,
            "--iterations" => iterations = value.parse()?,
            "--warmups" => warmups = value.parse()?,
            "--protocol" => protocol = value,
            "--record-timings" => {
                options.record_timings = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("--record-timings must be 0 or 1".into()),
                }
            },
            "--step" => {
                step = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("--step must be 0 or 1".into()),
                }
            },
            "--font" => font = value,
            "--font-dump" => font_dump = value,
            "--font-family" => family = value,
            "--cache-entries" => options.retained_entries = value.parse()?,
            "--cache-text-bytes" => options.retained_text_bytes = value.parse()?,
            "--cache-layout-bytes" => options.retained_layout_bytes = value.parse()?,
            "--parallel-min-groups" => options.parallel_min_groups = value.parse()?,
            "--parallel-min-text-bytes" => options.parallel_min_text_bytes = value.parse()?,
            "--output" => output = value,
            _ => return Err(format!("unknown option {key}").into()),
        }
    }
    if mode != "uncached-serial" && mode != "cached-serial" && mode != "cached-parallel" {
        return Err(format!("mode {mode} is not implemented").into());
    }
    if mode == "uncached-serial" {
        options.retained_entries = 0;
        options.retained_text_bytes = 0;
    }
    if ![
        "small",
        "article",
        "mixed",
        "single-long",
        "long-short",
        "atomic",
        "churn",
    ]
    .contains(&case.as_str())
    {
        return Err(format!("unknown case {case}").into());
    }
    if !["cold", "unchanged", "width", "edit", "churn"].contains(&protocol.as_str()) {
        return Err(format!("unknown protocol {protocol}").into());
    }
    if paragraphs == 0
        || iterations == 0
        || workers == 0
        || (mode != "cached-parallel" && workers != 1)
    {
        return Err(
            "paragraphs and iterations must be positive; serial modes require one worker".into(),
        );
    }
    if source_revision.len() != 40 || !source_revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("--source-revision requires the full 40-digit build source commit".into());
    }
    if font.is_empty() {
        return Err("--font is required for reproducible font identity".into());
    }
    if family.contains(['\'', '"', '\\']) {
        return Err("font family contains unsupported CSS quoting characters".into());
    }
    let font_bytes = fs::read(&font)?;
    let font_hash = hash(&font_bytes);
    let features = match (
        cfg!(feature = "image-decode"),
        cfg!(all(feature = "parallel-text", not(target_arch = "wasm32"))),
    ) {
        (true, true) => "image-decode,parallel-text",
        (true, false) => "image-decode",
        (false, true) => "parallel-text",
        (false, false) => "none",
    };
    #[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
    let pool = if mode == "cached-parallel" {
        Some(std::sync::Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()?,
        ))
    } else {
        None
    };
    #[cfg(not(all(feature = "parallel-text", not(target_arch = "wasm32"))))]
    if mode == "cached-parallel" {
        return Err("cached-parallel requires native parallel-text".into());
    }
    let mut out: Box<dyn Write> = if output.is_empty() {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create(output)?)
    };
    let para = match case.as_str() {
        "small" => "A small line.",
        "mixed" => "English العربية e\u{301} ligature office, punctuation and tabs.",
        "single-long" | "long-short" => {
            "A long paragraph tests wrapping and shaping across many words. "
        },
        "atomic" => {
            "Text with <span style='float:left;width:12px;height:12px'></span><span style='display:inline-block;width:24px;height:12px'>box</span> inline."
        },
        "churn" => "Editing one paragraph among many retained paragraphs.",
        _ => {
            "A paragraph contains several words, punctuation, and enough text to wrap across lines."
        },
    };
    let count = if case == "single-long" {
        1
    } else if case == "long-short" {
        9
    } else {
        paragraphs
    };
    let mut html = String::from("<!doctype html><html><body>");
    for i in 0..count {
        let content = if case == "single-long" || (case == "long-short" && i == 0) {
            para.repeat(paragraphs)
        } else {
            para.to_owned()
        };
        html.push_str(&format!(
            "<p {}>{content} unique-{i}</p>",
            if i == 0 { "id='edit'" } else { "" }
        ));
    }
    html.push_str("</body></html>");
    let css = format!(
        "@font-face {{ font-family: '{family}'; src: url('fixture.ttf'); }} body {{ font-family: '{family}'; margin: 0; }} p {{ margin: 0 0 8px; }}"
    );
    genet_livery::phase::enable();
    // Cold samples create a fresh document. Warm samples keep document and pool
    // scratch alive for all warmups and measurements of a single protocol.
    let create = || {
        let dom = ScriptedDom::from_serialized_document(&html);
        let edit = find(&dom, dom.document()).expect("fixture edit node");
        let mut doc = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(800.0, 600.0),
        );
        doc.set_text_shaping_options(options);
        #[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
        doc.set_text_shaping_pool(pool.clone());
        doc.set_font_resource("fixture.ttf", font_bytes.clone());
        (doc, edit)
    };
    let mut retained = if protocol == "cold" || protocol == "churn" {
        None
    } else {
        Some(create())
    };
    if step {
        writeln!(out, "ready")?;
        out.flush()?;
    }
    for iteration in 0..warmups + iterations {
        if step {
            let mut command = String::new();
            if io::stdin().read_line(&mut command)? == 0 || command.trim() != "step" {
                return Err("expected step on stdin".into());
            }
        }
        let cycle_start = Instant::now();
        if protocol == "cold" || protocol == "churn" {
            retained = Some(create());
        }
        let (doc, edit) = retained.as_mut().expect("document exists");
        if protocol == "edit" || protocol == "churn" {
            // A new key each time prevents two alternating edits becoming cache hits.
            let text = if case == "single-long" || case == "long-short" {
                // Preserve the entire long paragraph and insert a short unique
                // change near its middle instead of replacing it with a label.
                format!(
                    "{} revision-{iteration} {} unique-0",
                    para.repeat(paragraphs / 2),
                    para.repeat(paragraphs - paragraphs / 2)
                )
            } else {
                format!("Edited paragraph revision-{iteration} unique-0")
            };
            if protocol == "churn" {
                let _ = doc.frame(800, 600)?;
            }
            doc.mutate_dom(|dom| dom.set_text_content(*edit, &text));
        }
        let width = if protocol == "width" && iteration % 2 == 1 {
            640
        } else {
            800
        };
        let before = doc.text_system().shaping_stats();
        let _ = genet_livery::phase::take();
        let start = Instant::now();
        let list = doc.frame(width, 600)?;
        let elapsed = start.elapsed().as_nanos();
        let phase_us = genet_livery::phase::take();
        let after = doc.text_system().shaping_stats();
        if protocol == "cold" || protocol == "churn" {
            retained = None;
        }
        // Churn covers creation, initial frame, edit frame, and document drop.
        let cycle_ns = cycle_start.elapsed().as_nanos();
        let selected_fonts: Vec<_> = list
            .fonts()
            .iter()
            .map(|font| {
                let bytes = font.data.as_slice();
                (hash(bytes), bytes)
            })
            .collect();
        if !font_dump.is_empty() {
            fs::create_dir_all(&font_dump)?;
            for (id, bytes) in &selected_fonts {
                let path = std::path::Path::new(&font_dump)
                    .join(format!("{id:016x}-{}.font", bytes.len()));
                if path.exists() {
                    if fs::read(&path)? != *bytes {
                        return Err("font fingerprint collision".into());
                    }
                } else {
                    fs::write(path, bytes)?;
                }
            }
        }
        let selected_font_ids: Vec<_> = selected_fonts
            .iter()
            .map(|(id, bytes)| format!("{id:016x}:{}", bytes.len()))
            .collect();
        let bytes = postcard::to_allocvec(&PaintEnvelope::from_list(&list))?;
        writeln!(
            out,
            "source_revision={source_revision} features={features} mode={mode} case={case} paragraphs={paragraphs} protocol={protocol} sample={} measured={} width={width} font_fnv64={font_hash:016x} family={family:?} workers={workers} options={options:?} before={before:?} after={after:?} frame_ns={elapsed} cycle_ns={cycle_ns} phase_us={phase_us:?} selected_fonts={selected_font_ids:?} digest_fnv64={:016x}",
            iteration,
            iteration >= warmups,
            hash(&bytes)
        )?;
        out.flush()?;
    }
    Ok(())
}
