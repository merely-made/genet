# Shared Unicode text boundaries

**Status (2026-09-29):** Genet primitive/editor and pinned Mere search/Cambium editor adoption implemented and verified; headed caret and broader consumers remain open.

## Ownership and scope

Genet owns default UAX #29 mechanics through `components/genet-text`. The
implementation wraps the existing unicode-segmentation dependency rather than
reimplementing Unicode rules. Mere retains search normalization, URL connector
splitting, stemming, indexing and persistence policy. Locale dictionaries and
user-specific tailoring require explicit future profiles; default segmentation
must not silently claim dictionary-quality boundaries for Thai or Japanese.

The dependency is pinned to unicode-segmentation 1.13.3, Unicode 17.0.0, matching
the existing Genet lockfile. The moving Unicode specification now describes 18.0;
this slice deliberately makes the implemented version inspectable rather than
implicitly claiming the newest standard. Reference: [UAX #29 revision 47](https://www.unicode.org/reports/tr29/tr29-47.html).

`Segment` exposes half-open start/end positions in UTF-8 bytes, Unicode scalar
values and UTF-16 code units. All refer to the same unmodified string. Callers
must not reuse them against normalized text. Word boundaries include punctuation
and whitespace; the separate `words` filter matches the pre-existing search
consumer. Sentence segments retain trailing whitespace. The profile and Unicode
version are public for future persistent boundary caches; changing a search
profile remains an index compatibility decision owned by Mere.

## Phases and done-conditions

1. Default boundaries: streaming grapheme, word and sentence segments with named
   offsets. Done when the official Unicode 17 break corpora all pass and mixed
   supplementary/combining text proves coordinate conversions.
2. Consumer adoption: Genet editor caret/deletion uses full-string grapheme
   context; Mere lexical search uses the shared word seam while retaining its
   normalization. Done when focused consumer checks pass and source dependency
   publication is resolved. The Genet source is available at revision
   `19c206873ab08ae227217892d9e74d0df18b349a`; Mere adoption now uses that pin.
   The September 26 pending patch is historical; current implementation and
   validation are recorded in the September 29 section below.
3. Deferred consumers: DOM selection movement, find-in-page and Intl.Segmenter
   can adopt the contract after their own acceptance fixtures. This slice changes
   editor movement/deletion, not the DOM Selection API or locale negotiation.

## Findings

- 2026-09-26: `genet-documents/src/engines/livery.rs` previously segmented sliced
  prefixes/suffixes during caret movement and deletion. Segmentation now receives
  the complete value, retaining regional-indicator pairing context. Invalid byte
  coordinates return None; the adapter preserves its current caret on invalid
  input rather than slicing at a non-scalar boundary.
- 2026-09-26: Mere `eidetic-search/src/tokenize.rs` already centralized word
  segmentation, with stemming a caller-supplied hook. Its output policy remains
  separate from the primitive; no new normalization is introduced here.
- 2026-09-26: Fleece already uses unicode-segmentation for quote preservation.
  Migrating that tested consumer is deferred rather than widening this slice.

## Progress

- 2026-09-26: Added primitive, coordinate/caret tests and verbatim official
  Unicode 17 fixtures with their Unicode License v3. `cargo test -p genet-text
  --offline -- --nocapture` passed 2 unit tests plus 3 corpus tests: 766 grapheme,
  1,944 word and 512 sentence cases. Tests use repository `target`.

## Open gates

Headed Cambium caret verification, locale dictionaries,
DOM Selection/find/Intl consumers and persistent profile migration remain open.

## Consumer verification (2026-09-26)

The proposed Mere tokenizer module compiled directly with rustc against the built
`genet-text` rlib and passed its three existing URL/title, stemming and non-ASCII
tests. Full Mere workspace validation is not yet a receipt: the offline resolver
cannot find the cached Genet `origin/main` reference even with a command-local
patch. Both existing workspace locks resolve unicode-segmentation 1.13.3, so this
adapter does not change the Unicode rules used by existing search indices.

The final combined document/render rerun passed after the parallel generated-content
changes settled; the editor receipt is recorded below.

The pending Mere adapter was restored after saving the exact tested diff as a
receipt, because an unpublished crate would otherwise block resolution for the
whole Mere workspace. Mere source remains buildable with its existing dependency.
Strict Clippy for genet-text passed with all targets and warnings denied.

- 2026-09-26: Adjacent caret boundaries use the existing GraphemeCursor over the
  complete string, avoiding a full prefix scan on every ordinary ArrowRight.
  The official grapheme corpus additionally checks previous/next movement from
  every scalar boundary, including positions inside a cluster. All 766 cases
  pass these checks. Strict Clippy rerun passes after this change.

- 2026-09-26: After the parallel style edits settled, cargo test -p
  genet-documents --features livery --lib --offline passed all 52 tests, including
  livery_editor_uses_shared_grapheme_boundaries_for_keys. Existing feature-gated
  unused-import warnings remain. This is an actual editor consumer receipt;
  the DOM Selection API, headed interaction and full Mere integration remain open.

## Mere adoption (2026-09-29)

**Status:** implemented and verified against the published Genet source pin.

The containing Genet revision is now available:
`19c206873ab08ae227217892d9e74d0df18b349a`. Mere's workspace dependency names
`genet-text` at that same revision as its other Genet dependencies. This removes
the earlier unpublished-source blocker; the retained adoption patch is historical.

The bounded consumers are Mere `crates/intel/eidetic-search/src/tokenize.rs` and
Cambium `crates/cambium/cambium/src/controls/text_input/{core,word_motion,multiline}.rs`.
Search uses Genet's word filter before its existing URL connector splitting,
lowercasing and optional caller-supplied stemmer. Cambium uses Genet's segment
positions while keeping its grapheme-index caret model, whitespace-skipping word
commands and hard-newline navigation. Byte-to-grapheme conversion inspects the
complete string instead of slicing arbitrary byte offsets.

Done-conditions: existing Cambium editor/undo/IME tests and lexical-index tests
pass against the pinned source; added fixtures preserve byte snapping inside flag
and combining sequences and the search token stream for apostrophes, decimals,
underscores and non-ASCII words. Headed caret behavior remains a separate gate.

Index compatibility: the pre-adoption Mere lock already resolves
unicode-segmentation 1.13.3 / Unicode 17, exactly the Genet primitive's pinned
engine. `TOKENIZER_NAME`, normalization and stored index schema remain unchanged.
The new public `SEGMENTATION_PROFILE` describes the boundary engine; a future
Unicode or tailoring change still requires an explicit index migration decision,
not merely changing that constant. Locale dictionaries remain deferred.

- 2026-09-29: `cargo +1.98.1 test --manifest-path repos/mere/Cargo.toml -p
  cambium --lib --offline --target-dir C:/t/cargo-targets/mere` passed 232/232
  tests from the Code workspace root. Build output confirms the Genet dependency
  source is the published `19c20687` revision. This includes grapheme byte snapping,
  word motion, multiline movement, IME and exhaustive edit/undo regressions.
  The existing stable Mere target was reused; no isolated home or worktree exists.

- 2026-09-29: `cargo +1.98.1 test --manifest-path repos/mere/Cargo.toml -p
  eidetic-search --lib --offline --target-dir C:/t/cargo-targets/mere` passed
  38 tests, with the existing corpus-scale timing test explicitly ignored.
  Passed coverage includes the new token-stream compatibility fixture, existing
  normalization/stemmer tests, candidate matching, index rebuild/search and
  persisted-spec rejection/recovery. Unrelated dependency warnings remain;
  no extra test target, Cargo home, or worktree was created.
- 2026-09-29: Mere consumer adoption committed and pushed as
  `ada6f265856cbf16b5813452a714e9ea4c55dc56`, retaining the published Genet
  `19c20687` pin used by both test receipts. Only the two consumer lock edges
  were staged; concurrent workspace lock/source changes were preserved.
