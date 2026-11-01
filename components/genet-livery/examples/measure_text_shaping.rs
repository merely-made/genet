// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Deterministic document-frame shaping control. Run with `--help` for options.
//! Pass the full source commit used to build the executable as `--source-revision`.
//! This caller-supplied label is not verified; the output fingerprints the executable.
//! Example: `measure_text_shaping --source-revision <full-commit> --font <font-file>`.
use genet_livery::{Device, LiveryDocument, StyleSet, TextShapingOptions};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use paint_list_api::PaintEnvelope;
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
    let mut iterations = 1usize;
    let mut workers = 1usize;
    let mut source_revision = String::new();
    let mut font = String::new();
    let mut family = "Fixture".to_owned();
    let mut output = String::new();
    let mut options = TextShapingOptions {
        record_timings: true,
        ..Default::default()
    };
    while let Some(key) = args.next() {
        if key == "--help" {
            println!(
                "--source-revision FULL_GIT_COMMIT --mode uncached-serial|cached-serial|cached-parallel --case small|article|mixed|single-long|atomic|churn --paragraphs N --workers N --iterations N --font PATH --font-family NAME --cache-entries N --cache-text-bytes N --parallel-min-groups N --parallel-min-text-bytes N --output PATH"
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
            "--font" => font = value,
            "--font-family" => family = value,
            "--cache-entries" => options.retained_entries = value.parse()?,
            "--cache-text-bytes" => options.retained_text_bytes = value.parse()?,
            "--parallel-min-groups" => options.parallel_min_groups = value.parse()?,
            "--parallel-min-text-bytes" => options.parallel_min_text_bytes = value.parse()?,
            "--output" => output = value,
            _ => return Err(format!("unknown option {key}").into()),
        }
    }
    if mode != "uncached-serial" {
        return Err(format!("mode {mode} is not implemented").into());
    }
    if ![
        "small",
        "article",
        "mixed",
        "single-long",
        "atomic",
        "churn",
    ]
    .contains(&case.as_str())
    {
        return Err(format!("unknown case {case}").into());
    }
    if paragraphs == 0 || iterations == 0 || workers != 1 {
        return Err(
            "paragraphs and iterations must be positive; uncached-serial requires one worker"
                .into(),
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
    // The source label is supplied by the builder; the binary hash identifies
    // the executable even when it is invoked from another checkout.
    let executable_hash = hash(&fs::read(env::current_exe()?)?);
    let features = if cfg!(feature = "image-decode") {
        "image-decode"
    } else {
        "none"
    };
    let mut out: Box<dyn Write> = if output.is_empty() {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create(output)?)
    };
    let para = match case.as_str() {
        "small" => "A small line.",
        "mixed" => "English العربية e\u{301} ligature office, punctuation and tabs.",
        "single-long" => "A long paragraph tests wrapping and shaping across many words. ",
        "atomic" => {
            "Text with <span style='display:inline-block;width:24px;height:12px'>box</span> inline."
        },
        "churn" => "Editing one paragraph among many retained paragraphs.",
        _ => {
            "A paragraph contains several words, punctuation, and enough text to wrap across lines."
        },
    };
    let count = if case == "single-long" { 1 } else { paragraphs };
    let mut html = String::from("<!doctype html><html><body>");
    for i in 0..count {
        let content = if case == "single-long" {
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
    for iteration in 0..iterations {
        let dom = ScriptedDom::from_serialized_document(&html);
        let edit = find(&dom, dom.document()).ok_or("fixture edit node missing")?;
        let mut doc = LiveryDocument::new(
            dom,
            StyleSet::cambium(&[&css]),
            Device::screen(800.0, 600.0),
        );
        doc.set_text_shaping_options(options);
        doc.set_font_resource("fixture.ttf", font_bytes.clone());
        for (stage, width) in [
            ("initial", 800),
            ("unchanged", 800),
            ("width", 640),
            ("edit", 640),
        ] {
            if stage == "edit" {
                doc.mutate_dom(|dom| dom.set_text_content(edit, "Edited paragraph unique-0"));
            }
            let start = Instant::now();
            let list = doc.frame(width, 600)?;
            let elapsed = start.elapsed().as_nanos();
            let bytes = postcard::to_allocvec(&PaintEnvelope::from_list(&list))?;
            writeln!(
                out,
                "source_revision={source_revision} features={features} executable_fnv64={executable_hash:016x} mode={mode} case={case} paragraphs={paragraphs} iteration={iteration} stage={stage} width={width} font_fnv64={font_hash:016x} family={family:?} workers={workers} options={options:?} stats={:?} frame_ns={elapsed} digest_fnv64={:016x}",
                doc.text_system().shaping_stats(),
                hash(&bytes)
            )?;
        }
    }
    Ok(())
}
