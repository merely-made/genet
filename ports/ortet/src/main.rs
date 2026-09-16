// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `ortet` — open one document in one window. See the crate docs in `lib.rs`.

#[cfg(not(target_arch = "wasm32"))]
use ortet::args::{self, Invocation};
#[cfg(not(target_arch = "wasm32"))]
use ortet::fetch::OrtetFetcher;
#[cfg(not(target_arch = "wasm32"))]
use ortet::shell;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ortet: {error}");
            std::process::ExitCode::FAILURE
        },
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run() -> Result<(), String> {
    let config = match args::parse(std::env::args().skip(1))? {
        Invocation::Help => {
            print!("{}", args::USAGE);
            return Ok(());
        },
        Invocation::Run(config) => *config,
    };

    // Both lanes, always: a local page may name a remote stylesheet or image,
    // and the scheme split — not the address ortet was started with — is what
    // decides where each request goes.
    let fetcher = OrtetFetcher::with_network()?;

    println!("ortet: address {}", config.address);
    let outcome = shell::run(config, fetcher)?;
    println!(
        "ortet: engine {} backend {} presented {} frame(s) at {}x{}",
        outcome.engine_id, outcome.backend, outcome.frames, outcome.size.0, outcome.size.1
    );
    println!(
        "ortet: target {} features {} source {}",
        outcome.metadata.target,
        outcome.metadata.features,
        outcome
            .metadata
            .source_revision
            .as_deref()
            .unwrap_or("unknown")
    );
    // The address the run ended on, which differs from the one it started on
    // exactly when a link was followed. That is the whole of what a navigation
    // receipt has to show.
    println!("ortet: settled at {}", outcome.address);
    if let (Some(artifact), Some(digest)) = (&outcome.artifact, outcome.digest) {
        println!(
            "ortet: receipt {} digest 0x{digest:016x}",
            artifact.display()
        );
    }
    if let Some(heading) = outcome.matched_heading.as_deref() {
        println!("ortet: semantic heading {heading:?}");
    }
    // The engine-owned inspection seam a G5 receipt correlates with the
    // captured frame: cumulative reflectors-unpinned/nodes-collected across
    // this session's frame-cadence GC ticks. Zero for non-scripted lanes.
    let (unpinned, collected) = outcome.collection_stats;
    println!("ortet: receipt collection unpinned={unpinned} collected={collected}");
    // The top document's arena census at the first and last presented frames.
    // A sequence that ends where it began — including across a top-level
    // navigation — must not have grown here.
    let census = |count: Option<usize>| {
        count.map_or_else(|| "unavailable".to_owned(), |count| count.to_string())
    };
    println!(
        "ortet: receipt live nodes first={} last={}",
        census(outcome.live_nodes.0),
        census(outcome.live_nodes.1)
    );
    // The T3 mutation instrument. Work and presentation wait are reported
    // separately on purpose: a vsync-bound frame and a layout-bound frame have
    // the same total and nothing else in common.
    for stream in &outcome.mutation_streams {
        println!("ortet: mutation stream {stream}");
    }
    if let Some(reason) = outcome.mutation_unsupported.as_deref() {
        println!("ortet: mutation UNSUPPORTED by this engine: {reason}");
    }
    let (applied, missed) = outcome.host_mutations;
    if applied > 0 || missed > 0 {
        println!("ortet: host mutations applied={applied} missed={missed}");
    }
    let timing = outcome.timing;
    if timing.frames > 0 {
        println!(
            "ortet: timing over {} measured frame(s) — work median {} us p95 {} us; \
             present wait median {} us p95 {} us; total median {} us p95 {} us",
            timing.frames,
            timing.work.median_us,
            timing.work.p95_us,
            timing.wait.median_us,
            timing.wait.p95_us,
            timing.total.median_us,
            timing.total.p95_us
        );
        println!(
            "ortet: timing work split — session.frame median {} us, raster median {} us, \
             mutate median {} us, restyled elements {}",
            timing.frame_phase.median_us,
            timing.raster.median_us,
            timing.mutate.median_us,
            timing.restyled
        );
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
