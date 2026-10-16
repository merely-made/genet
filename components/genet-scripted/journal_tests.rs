use super::*;
use crate::capture::{DomCaptureRecord, RecordedMutation, read_capture_records};
use script_engine_boa::BoaEngine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_CAPTURE: AtomicU64 = AtomicU64::new(0);

struct CaptureFile(PathBuf);

impl CaptureFile {
    fn new(label: &str) -> Self {
        let unique = NEXT_CAPTURE.fetch_add(1, Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "genet-journal-{label}-{}-{stamp}-{unique}.postcard",
            std::process::id()
        )))
    }
}

impl Drop for CaptureFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[derive(Clone, Default)]
struct CountingFetcher {
    calls: Arc<Mutex<Vec<String>>>,
}

impl CountingFetcher {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().expect("fetch calls").clone()
    }
}

impl ResourceFetcher for CountingFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.calls.lock().expect("fetch calls").push(url.to_owned());
        (url == "https://journal.test/theme.css").then(|| b".card.active { color: red; }".to_vec())
    }
}

struct FrameResult {
    html: String,
    scene: String,
    restyle: genet_livery::RestyleStats,
    sheets: Vec<(Option<String>, Option<String>, String)>,
    fetches: Vec<String>,
    captured_class_edits: usize,
    captured_evaluate_edits: usize,
    captured_media_edits: usize,
}

fn frame_after_edit(capture: bool) -> FrameResult {
    let path = CaptureFile::new("livery");
    let fetcher = CountingFetcher::default();
    let html = "<html><head><link rel=stylesheet href=theme.css></head><body>\
        <div id=card class=card>card</div><div>one</div><div>two</div><div>three</div>\
        </body></html>";
    let mut document = LiveryScriptedDocument::<BoaEngine>::from_body(
        html,
        fetcher.clone(),
        "https://journal.test/index.html",
    )
    .expect("Livery document");
    document.capture = None;

    if capture {
        let before = {
            let host = document.rt.host().borrow();
            let (base, pending) = host.dom.pending_mutations();
            (base, pending.len())
        };
        assert!(
            before.1 > 0,
            "the parser left a baseline journal for Livery"
        );
        let recorder = {
            let mut host = document.rt.host().borrow_mut();
            DomCaptureRecorder::open_at_path(&path.0, &mut host.dom, &[])
                .expect("open capture on the built tree")
        };
        let after = {
            let host = document.rt.host().borrow();
            let (base, pending) = host.dom.pending_mutations();
            (base, pending.len())
        };
        assert_eq!(
            after, before,
            "opening capture must not drain Livery's baseline"
        );
        document.capture = Some(recorder);
    }

    let _initial = document.frame(320, 180);
    let initial_fetches = fetcher.calls();
    assert_eq!(
        initial_fetches,
        vec!["https://journal.test/theme.css"],
        "the linked sheet has one acquisition lifetime"
    );

    document
        .evaluate("document.getElementById('card').setAttribute('data-edit', 'via-evaluate')")
        .expect("ordinary host script turn");
    if capture {
        let host = document.rt.host().borrow();
        assert!(
            !host.dom.pending_mutations().1.is_empty(),
            "capture flush leaves the edit for Livery's next frame"
        );
    }

    // Bypass the document's evaluate/flush wrapper. The next real frame must
    // capture this edit before Livery observes and compacts the same journal.
    document
        .rt
        .eval("document.getElementById('card').setAttribute('class', 'card active')")
        .expect("edit immediately before frame");
    let scene = document.frame(320, 180);
    let restyle = document.cssom.last_restyle_stats();
    assert!(restyle.snapshots > 0, "the edit reached incremental style");
    assert!(!restyle.full_document, "the edit retained scoped restyling");
    let sheets = document
        .resource_set()
        .expect("live resource ledger")
        .stylesheets
        .into_iter()
        .map(|sheet| (sheet.requested_url, sheet.source_url, sheet.text))
        .collect();
    let result = FrameResult {
        html: document.dom_snapshot(),
        scene: scene.dump_ops(),
        restyle,
        sheets,
        fetches: fetcher.calls(),
        captured_class_edits: 0,
        captured_evaluate_edits: 0,
        captured_media_edits: 0,
    };
    assert_eq!(
        result.fetches, initial_fetches,
        "an unrelated class edit must not refetch CSS"
    );

    document
        .evaluate("var mq = matchMedia('(max-width: 400px)'); mq.addEventListener('change', function () { document.getElementById('card').setAttribute('data-media', 'seen'); });")
        .expect("register media change callback");
    let _resized = document.frame(500, 180);
    assert!(document.dom_snapshot().contains("data-media=\"seen\""));
    assert!(
        !document
            .rt
            .host()
            .borrow()
            .dom
            .pending_mutations()
            .1
            .is_empty(),
        "media callback runs after paint and leaves an edit for the next frame"
    );
    if capture {
        let records = read_capture_records(&path.0).expect("capture flushed after callback");
        assert_eq!(count_attribute(&records, "data-media", "seen"), 1);
    }
    let _next = document.frame(500, 180);
    assert!(
        document
            .rt
            .host()
            .borrow()
            .dom
            .pending_mutations()
            .1
            .is_empty(),
        "the next frame consumes the media callback edit"
    );
    document.flush_dom_capture();
    document.flush_dom_capture();
    let _cached = document.frame(500, 180);
    let (captured_class_edits, captured_evaluate_edits, captured_media_edits) = if capture {
        let records = read_capture_records(&path.0).expect("read capture");
        (
            count_attribute(&records, "class", "card active"),
            count_attribute(&records, "data-edit", "via-evaluate"),
            count_attribute(&records, "data-media", "seen"),
        )
    } else {
        (0, 0, 0)
    };
    if capture {
        assert_eq!(
            captured_class_edits, 1,
            "repeated flushes and frames do not duplicate an edit"
        );
        assert_eq!(
            captured_evaluate_edits, 1,
            "host evaluate flush records its edit once"
        );
        assert_eq!(
            captured_media_edits, 1,
            "post-frame callback flush records its edit once"
        );
    }
    FrameResult {
        captured_class_edits,
        captured_evaluate_edits,
        captured_media_edits,
        ..result
    }
}

fn count_attribute(records: &[DomCaptureRecord], local: &str, value: &str) -> usize {
    records
        .iter()
        .flat_map(|record| match record {
            DomCaptureRecord::MutationBatch { mutations, .. } => mutations.as_slice(),
            DomCaptureRecord::SessionStart { .. } => &[],
        })
        .filter(|mutation| {
            matches!(
                mutation,
                RecordedMutation::AttributeChanged { name, new_value, .. }
                    if name.local == local && new_value.as_deref() == Some(value)
            )
        })
        .count()
}

#[test]
fn capture_preserves_livery_incremental_frame_and_resource_decisions() {
    let disabled = frame_after_edit(false);
    let enabled = frame_after_edit(true);
    assert_eq!(enabled.captured_class_edits, 1);
    assert_eq!(enabled.captured_evaluate_edits, 1);
    assert_eq!(enabled.captured_media_edits, 1);
    assert_eq!(enabled.html, disabled.html);
    assert_eq!(enabled.scene, disabled.scene);
    assert_eq!(enabled.restyle, disabled.restyle);
    assert_eq!(enabled.sheets, disabled.sheets);
    assert_eq!(enabled.fetches, disabled.fetches);
}

#[test]
fn headless_script_turns_compact_the_journal_with_or_without_capture() {
    for capture in [false, true] {
        let path = CaptureFile::new("headless");
        let mut document =
            ScriptedDocument::<BoaEngine>::parse("<body><div id=target></div></body>")
                .expect("headless document");
        document.capture = None;
        if capture {
            let recorder = {
                let mut host = document.rt.host().borrow_mut();
                DomCaptureRecorder::open_at_path(&path.0, &mut host.dom, &[])
                    .expect("open headless capture")
            };
            document.capture = Some(recorder);
        }
        let mut last_base = document.rt.host().borrow().dom.pending_mutations().0;
        for value in 0..16 {
            document
                .evaluate(&format!(
                    "document.getElementById('target').setAttribute('data-turn', '{value}')"
                ))
                .expect("script turn");
            let host = document.rt.host().borrow();
            let (base, pending) = host.dom.pending_mutations();
            assert!(
                pending.is_empty(),
                "headless journal is compacted after turn {value}"
            );
            assert!(
                base > last_base,
                "turn {value} advanced the journal sequence"
            );
            last_base = base;
        }
        assert!(
            document
                .evaluate("document.getElementById('target').setAttribute('data-throw', 'kept'); throw new Error('stop')")
                .is_err(),
            "the script throws after changing the DOM"
        );
        assert!(
            document
                .rt
                .host()
                .borrow()
                .dom
                .pending_mutations()
                .1
                .is_empty(),
            "even a failed script turn compacts its observed edit"
        );
        if capture {
            let records = read_capture_records(&path.0).expect("read headless capture");
            let mutations = records
                .iter()
                .flat_map(|record| match record {
                    DomCaptureRecord::MutationBatch { mutations, .. } => mutations.as_slice(),
                    DomCaptureRecord::SessionStart { .. } => &[],
                })
                .collect::<Vec<_>>();
            let captured_turns = mutations
                .iter()
                .filter(|mutation| {
                    matches!(mutation, RecordedMutation::AttributeChanged { name, .. } if name.local == "data-turn")
                })
                .count();
            assert_eq!(captured_turns, 16, "every compacted turn was recorded once");
            assert_eq!(
                mutations
                    .iter()
                    .filter(|mutation| matches!(mutation, RecordedMutation::AttributeChanged { name, .. } if name.local == "data-throw"))
                    .count(),
                1,
                "the edit before a script error is captured"
            );
        }
    }
}

#[test]
fn recorder_rejects_missing_history_and_a_different_arena() {
    let path = CaptureFile::new("stale");
    let mut document = ScriptedDocument::<BoaEngine>::parse("<body><div id=target></div></body>")
        .expect("first document");
    document.capture = None;
    let mut recorder = {
        let host = document.rt.host().borrow();
        DomCaptureRecorder::open_at_path(&path.0, &host.dom, &[]).expect("recorder")
    };
    document
        .rt
        .eval("document.getElementById('target').setAttribute('data-lost', 'yes')")
        .expect("unobserved edit");
    {
        let mut host = document.rt.host().borrow_mut();
        host.dom.drain_mutations(&mut Vec::new());
        assert_eq!(
            recorder.record_pending(&host.dom).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData,
            "capture cannot silently skip a compacted mutation"
        );
    }

    let other = ScriptedDocument::<BoaEngine>::parse("<body><div id=other></div></body>")
        .expect("second document");
    let host = other.rt.host().borrow();
    assert_eq!(
        recorder.record_pending(&host.dom).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData,
        "equal sequence numbers cannot alias another DOM arena"
    );
}
