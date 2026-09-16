// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `--mutate`: a deterministic per-frame DOM mutation pattern, driven from
//! Rust with no script engine in the session.
//!
//! One stream is `<id-prefix>:<op>[:<per-frame>]`. Each frame the stream takes
//! the next `per-frame` targets from the prefix's id list, wrapping, and emits
//! one [`HostMutation`] per target. Values are functions of the frame index
//! alone, so two runs of the same spec on the same document produce the same
//! batches — which is what makes the timing comparable between runs.

use document_session_api::session_engine::{HostMutation, HostMutationOp};

/// One mutation stream from the command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MutationSpec {
    /// Targets are the elements whose `id` begins with this.
    pub prefix: String,
    pub op: MutationKind,
    /// How many targets this stream touches per frame.
    pub per_frame: u32,
}

/// What a stream does to each of its targets. These are patterns over the
/// session's [`HostMutationOp`] vocabulary, not new capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationKind {
    /// Rewrite the target's `style` attribute: a moving 2D translate plus a
    /// cycling background. Exercises inline-style restyle and repaint.
    Style,
    /// Rewrite the target's `class` attribute between two authored classes.
    /// Exercises selector matching rather than inline declarations.
    Class,
    /// Grow and shrink the subtree: append a child on even frames, remove the
    /// last one on odd frames. Exercises insertion and removal against the
    /// same target, and leaves the document where it started.
    Child,
}

impl MutationKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::Class => "class",
            Self::Child => "child",
        }
    }
}

/// Every stream, with the concrete target ids each resolved once against the
/// loaded document.
#[derive(Clone, Debug, Default)]
pub struct MutationPlan {
    streams: Vec<(MutationSpec, Vec<String>)>,
}

impl MutationPlan {
    /// Resolve each spec's prefix through the session's own id enumeration.
    /// A prefix that matches nothing is kept with an empty target list so the
    /// receipt can say the stream was inert rather than silently dropping it.
    pub fn resolve(specs: &[MutationSpec], ids_with_prefix: impl Fn(&str) -> Vec<String>) -> Self {
        Self {
            streams: specs
                .iter()
                .map(|spec| (spec.clone(), ids_with_prefix(&spec.prefix)))
                .collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.streams.iter().all(|(_, ids)| ids.is_empty())
    }

    /// One line per stream: what it resolved to. A receipt that does not say
    /// how many elements a stream actually hit is not a measurement.
    pub fn describe(&self) -> Vec<String> {
        self.streams
            .iter()
            .map(|(spec, ids)| {
                format!(
                    "{}:{}:{} -> {} target(s)",
                    spec.prefix,
                    spec.op.name(),
                    spec.per_frame,
                    ids.len()
                )
            })
            .collect()
    }

    /// The batch for frame `index` (counting presented frames from zero).
    pub fn batch(&self, index: u32) -> Vec<HostMutation> {
        let mut batch = Vec::new();
        for (spec, ids) in &self.streams {
            if ids.is_empty() {
                continue;
            }
            let take = spec.per_frame.min(ids.len() as u32);
            // `child` appends on even frames and removes on odd ones, so its
            // window advances every SECOND frame: otherwise each removal would
            // land on a target that never received the matching append, and
            // the stream would report half its work as a miss.
            let cycle = if spec.op == MutationKind::Child {
                index / 2
            } else {
                index
            };
            for step in 0..take {
                // A rotating window: over enough frames every target is hit,
                // and no frame's cost depends on which frame it is.
                let cursor =
                    (u64::from(cycle) * u64::from(take) + u64::from(step)) % ids.len() as u64;
                let target = &ids[cursor as usize];
                batch.push(HostMutation::new(target, operation(spec.op, index, step)));
            }
        }
        batch
    }
}

/// The deterministic value for one target on one frame.
fn operation(kind: MutationKind, index: u32, step: u32) -> HostMutationOp {
    match kind {
        MutationKind::Style => {
            // A small cycling offset and hue: enough to change layout position
            // and paint, bounded so the document never leaves the viewport.
            let offset = (index + step) % 16;
            let level = 0x30 + ((index + step) % 8) * 0x10;
            HostMutationOp::SetAttribute {
                name: "style".to_owned(),
                value: format!(
                    "transform: translate({offset}px, {offset}px); background: #{level:02x}5a7f;"
                ),
            }
        },
        MutationKind::Class => HostMutationOp::SetAttribute {
            name: "class".to_owned(),
            value: if (index + step) % 2 == 0 {
                "ctl".to_owned()
            } else {
                "ctl warm".to_owned()
            },
        },
        MutationKind::Child => {
            if index % 2 == 0 {
                HostMutationOp::AppendChild {
                    tag: "span".to_owned(),
                    class: Some("tick".to_owned()),
                    text: None,
                }
            } else {
                HostMutationOp::RemoveLastChild
            }
        },
    }
}

/// Parse a `;`-separated `--mutate` value, the same separator `--actions` uses.
pub fn parse_specs(raw: &str) -> Result<Vec<MutationSpec>, String> {
    raw.split(';')
        .map(str::trim)
        .filter(|stream| !stream.is_empty())
        .map(parse_spec)
        .collect()
}

fn parse_spec(stream: &str) -> Result<MutationSpec, String> {
    let mut fields = stream.split(':');
    let prefix = fields
        .next()
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty())
        .ok_or_else(|| format!("mutation {stream} wants <id-prefix>:<op>[:<per-frame>]"))?;
    let op = match fields.next().map(str::trim) {
        Some("style") => MutationKind::Style,
        Some("class") => MutationKind::Class,
        Some("child") => MutationKind::Child,
        Some(other) => return Err(format!("unknown mutation op {other}")),
        None => return Err(format!("mutation {stream} needs an op after the prefix")),
    };
    let per_frame = match fields.next().map(str::trim) {
        None | Some("") => 1,
        Some(raw) => raw
            .parse::<u32>()
            .map_err(|_| format!("mutation {stream} wants a whole per-frame count, got {raw}"))?,
    };
    if per_frame == 0 {
        return Err(format!("mutation {stream} would mutate nothing"));
    }
    if fields.next().is_some() {
        return Err(format!("mutation {stream} has more than three fields"));
    }
    Ok(MutationSpec {
        prefix: prefix.to_owned(),
        op,
        per_frame,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_parse_in_order_and_reject_nonsense() {
        let specs = parse_specs("ctl:style:4; view:style; ctl:child").expect("parses");
        assert_eq!(
            specs,
            vec![
                MutationSpec {
                    prefix: "ctl".to_owned(),
                    op: MutationKind::Style,
                    per_frame: 4
                },
                MutationSpec {
                    prefix: "view".to_owned(),
                    op: MutationKind::Style,
                    per_frame: 1
                },
                MutationSpec {
                    prefix: "ctl".to_owned(),
                    op: MutationKind::Child,
                    per_frame: 1
                },
            ]
        );
        assert!(parse_specs("ctl:spin").is_err(), "unknown op");
        assert!(parse_specs("ctl").is_err(), "an op is required");
        assert!(parse_specs("ctl:style:0").is_err(), "zero mutates nothing");
        assert!(parse_specs("ctl:style:2:3").is_err(), "three fields only");
    }

    /// The pattern is a pure function of the frame index, which is what makes
    /// two runs of the same spec comparable.
    #[test]
    fn a_batch_is_deterministic_and_rotates_through_its_targets() {
        let ids = |prefix: &str| {
            if prefix == "ctl" {
                vec!["ctl0".to_owned(), "ctl1".to_owned(), "ctl2".to_owned()]
            } else {
                Vec::new()
            }
        };
        let specs = parse_specs("ctl:style:2").expect("parses");
        let plan = MutationPlan::resolve(&specs, ids);

        let first = plan.batch(0);
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].target_id, "ctl0");
        assert_eq!(first[1].target_id, "ctl1");
        // Frame 1 continues the window, wrapping over the three targets.
        let second = plan.batch(1);
        assert_eq!(second[0].target_id, "ctl2");
        assert_eq!(second[1].target_id, "ctl0");
        assert_eq!(plan.batch(0), first, "the same frame is the same batch");
        assert_ne!(first[0].op, second[1].op, "the value moves with the frame");
    }

    /// `child` grows then shrinks, so a long run does not accumulate a subtree
    /// whose size is the real variable.
    #[test]
    fn the_child_stream_appends_and_removes_on_alternating_frames() {
        let specs = parse_specs("ctl:child").expect("parses");
        let ids = vec!["ctl0".to_owned(), "ctl1".to_owned(), "ctl2".to_owned()];
        let plan = MutationPlan::resolve(&specs, |_| ids.clone());
        assert!(matches!(
            plan.batch(0)[0].op,
            HostMutationOp::AppendChild { .. }
        ));
        assert_eq!(plan.batch(1)[0].op, HostMutationOp::RemoveLastChild);
        // The removal must land on the element the append grew, or the stream
        // reports misses instead of work.
        assert_eq!(plan.batch(0)[0].target_id, plan.batch(1)[0].target_id);
        assert!(matches!(
            plan.batch(2)[0].op,
            HostMutationOp::AppendChild { .. }
        ));
        assert_ne!(plan.batch(2)[0].target_id, plan.batch(1)[0].target_id);
    }

    /// A prefix that matches nothing stays in the plan and says so, rather
    /// than disappearing into an empty run that looks like a measurement.
    #[test]
    fn an_unmatched_prefix_is_reported_rather_than_dropped() {
        let specs = parse_specs("absent:style:2").expect("parses");
        let plan = MutationPlan::resolve(&specs, |_| Vec::new());
        assert!(plan.is_empty());
        assert_eq!(plan.describe(), vec!["absent:style:2 -> 0 target(s)"]);
        assert!(plan.batch(3).is_empty());
    }
}
