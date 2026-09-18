# cadency: the owned selector engine, and the last of stylo

**Date:** 2026-09-18

**Status:** S0 and C0-C3 landed in the working tree 2026-09-18, uncommitted. C4
(publication) is Mark's and not executed.

**Parent:** [The stylo fork deletion lane](../docs/2026-08-16_stylo_fork_deletion_lane_plan.md),
complete 2026-08-21. That lane removed the fork. This one removes what the
servo/stylo repository still supplied from crates.io.

## The ruling this serves

Mark, 2026-09-18: remove stylo entirely from genet; nothing of it remains after
this pass. Asked whether that reaches the `selectors` and `servo_arc` crates,
which publish from the servo/stylo repository without being the style engine,
he ruled that it does. `cssparser` publishes from servo/rust-cssparser and is
not in scope.

Rulings taken the same day, each asked with the measured options:

- The engine is **owned and spec-shaped**, written against Selectors Level 4 for
  the slice Livery enables. Upstream is reference material. A harvest of the
  7.5k-line upstream was declined: it would carry `SelectorImpl`'s generic
  machinery and `servo_arc::ThinArc` in with it.
- It is a **standalone crate**, so other DOM walkers can consume it.
- It is named **cadency**: heraldry's marks telling which child of a house
  someone is, by birth order and descent. `cognizance` was picked first and
  withdrawn on length. Both exact-name free on crates.io, checked 2026-09-18;
  no other registry checked.
- Dated docs, receipts, `license.html` and MPL provenance headers keep their
  stylo mentions as historical record and attribution.
- `import-stylo-db` and the four `genet-stylo*` name-claim directories are
  deleted in-repo. crates.io is not touched by this plan.
- genet-wpt's geometry label is corrected rather than left.

## What cadency has to be

Livery enables a slice of upstream, read off `components/livery/src/selector.rs`
at `bc3a3c71084`:

| Enabled | Not enabled |
|---|---|
| type, universal, `#id`, `.class`, `ns` forms `*\|E` and `\|E` | named namespace prefixes, default namespace |
| attribute selectors, all six operators, `i` / `s` flags | `:has()`, relative selectors |
| descendant, child, next-sibling, later-sibling | pseudo-elements other than the two below |
| `:not()`, `:is()`, `:where()` | quirks-mode matching |
| `:root`, `:empty`, `:scope`, the `first/last/only` child and of-type forms | ancestor bloom filter, selector flags |
| `:nth-child`, `:nth-last-child` with `of S`; `:nth-of-type`, `:nth-last-of-type` | invalidation matching, `:visited` handling |
| `:host`, `:host()`, `::slotted()`, `::part()` | `:host-context()` |
| six states: hover, active, focus, focus-within, disabled, checked | |

A selector list is unforgiving: one bad selector fails the list. `:is()` and
`:where()` arguments are forgiving, as upstream parses them. The accepted set
must not widen in this pass, because a newly accepted selector turns a dropped
rule into a live one and moves WPT for a reason that is not the engine swap.
Widening is later work with its own measurement.

Specificity keeps upstream's packing, `id << 20 | class << 10 | element`, each
capped at 10 bits, because `livery::cascade::Specificity(u32)` is ordered on it.

### Shape

- `cadency::Element`: the owned trait, holding only what matching calls.
  `opaque`, `apply_selector_flags`, `add_element_unique_hashes`,
  `is_pseudo_element`, `match_pseudo_element`, `has_custom_state` and `is_link`
  do not survive.
- `cadency::PseudoClass`: the consumer's state vocabulary, parsed by name. The
  six Livery states stay Livery's; cadency hardcodes none.
- `cadency::SelectorList<P>`: parse, `matches`, `specificity`, and the three
  facts a cascade index needs, read off the parsed form rather than the source
  text: reach across a tree scope, the rightmost key, and sibling/structural
  dependency.

Livery keeps `livery::selector` as a thin module over cadency so its public
names (`SelectorList`, `SelectorKey`, `SelectorReach`, `StatePseudoClass`)
hold for mere and woodshed, neither of which implements the trait. `Atom` and
`AttributeValue` existed to satisfy upstream's `SelectorImpl` and go with it.

## Phases

### S0. Mechanical removals

Done-conditions:

- `stylo_malloc_size_of` and direct `servo_arc` leave every manifest;
  `servo-malloc-size-of` owns `MallocSizeOfOps`.
- The tool and the four name-claim directories are gone.
- Comments describing a Stylo that is not there are corrected.
- genet-wpt labels geometry by what drives it, with tests green.

### C0. The crate: parse

Done-conditions:

- `components/cadency`, workspace member, depends on `cssparser` only.
- Every row of the enabled column parses; every row of the other column is a
  parse error, asserted per construct.
- Specificity asserted against hand-computed values, including `:is()` taking
  its most specific argument, `:where()` zero, and `:nth-child(... of S)`.

### C1. The crate: match

Done-conditions:

- Matching runs right to left over an `Element`, with the tree-scope rules for
  `:host`, `::slotted()` and `::part()` including `exportparts` forwarding.
- A fixture DOM in the crate's own tests covers each combinator, each
  structural form, and `an+b` edge cases (negative `a`, zero `a`, `of S`).

### C2. Livery and genet-livery move

Done-conditions:

- `selectors` leaves both manifests and the workspace table, and
  `precomputed-hash` leaves Livery's.
- `components/livery/tests/selectors.rs` passes with its assertions unchanged;
  only its fixture's trait impl changes.
- livery, genet-livery, genet-documents, genet-wpt and ortet suites green.

### C3. Proof

Done-conditions:

- `Cargo.lock` has no `selectors`, `servo_arc`, `stylo*` or `to_shmem*`
  package, shown by `cargo tree -i` failing to find each. `precomputed-hash`
  stays: it publishes from its own repository and `string_cache` needs it.
- `git grep -i stylo` over `*.rs`, `*.toml` and scripts, outside dated docs and
  receipts, returns only provenance headers and dated history comments, listed
  in Findings.
- WPT before and after over `css/selectors`, `css/css-cascade`,
  `css/CSS2/selectors`, `dom/nodes` (testharness) and `css/selectors`,
  `shadow-dom` (reftest), the set lane F measured: every transition attributed.
  Zero is the expectation; a transition is a finding, not a failure to hide.

### C4. Publication

Not executed by this plan. `cadency` needs a real publish to hold the name, and
livery and genet-livery need breaking bumps before they can publish against it.
Both are outward actions and Mark's.

## Findings

- 2026-09-18. The fork left `Cargo.lock` on 2026-08-21. What remained at
  `bc3a3c71084` was 775 mentions in 107 files, about 700 of them in dated docs
  and receipts.
- 2026-09-18. `stylo_malloc_size_of` had one consumer, a single
  `pub use ...::MallocSizeOfOps` at `components/malloc_size_of/lib.rs:59`.
  The type is 3 fields and 5 methods and is now owned there. This changes the
  type's identity for anything that mixed it with upstream's, so
  `servo-malloc-size-of` takes a breaking bump when it next publishes.
- 2026-09-18. `servo_arc` had no consumer but four `MallocSizeOf` impls and
  their compile-fail doctests.
- 2026-09-18. `Element` has two implementors in the whole `Code` tree:
  `components/genet-livery/src/dom.rs` and Livery's test fixture. mere and
  woodshed depend on Livery and never name `selectors::`.
- 2026-09-18. genet-wpt hardcoded `testharness_geometry: "stylo"` under a
  comment saying Stylo drove geometry. It has not since 2026-08-21: the route
  is Livery's cascade into Buckram. The label is now `"buckram"`. No committed
  file records the field; baselines under `Code/testing/genet/` that recorded
  `"stylo"` will report a route change on their next comparison, which is the
  instrument working.
- 2026-09-18. `components/script-runtime-api` carries its own
  `crate::selector` for `querySelector`. It is a second run at the same
  problem and a cadency consumer in waiting. Not touched here.

- 2026-09-18. **Spec-shaped departure from upstream, deliberate.** Once a
  selector has matched a featureless host from inside its shadow tree, cadency
  refuses any compound further left (`body :host > div` does not match), per
  CSS Scoping 1: nothing outside a shadow tree is visible from inside it.
  Upstream keeps walking. `components/cadency/src/matching.rs`, `matches_from`.
- 2026-09-18. **Dependency facts are read off the parsed form now**, not
  scanned from source text. Two consequences, both narrowing a false positive
  or closing a hole: `li:nth-child(2n+1)` no longer reports a sibling
  dependency (the old scan saw the `+`), and `:nth-child(2 of .a)` now does
  (the old scan saw neither `+` nor `~`, though a class change on one sibling
  renumbers the rest). Structural dependency is unchanged for both.
- 2026-09-18. **Pre-existing bug, preserved on purpose.**
  `components/genet-livery/src/dom.rs` `imported_part` maps an inner part name
  to its outer name. The contract, upstream's and cadency's, runs the other
  way. Identity forwarding (`exportparts="a"`) is unaffected; a renaming
  `exportparts="a: b"` forwards wrongly. No test covers it. Left as it was so
  the WPT before/after measures the engine swap alone. cadency's own fixture
  exercises the renaming case against the correct contract.
- 2026-09-18. `SelectorTree::identities` in the same file existed to mint stable
  addresses for `selectors::OpaqueElement`. cadency compares node ids, so the
  map now only backs `len()`. Building it per restyle is removable cost; not
  removed here.
- 2026-09-18. `cargo check --workspace --all-targets` fails in one place,
  `support/patches/parley`'s test target (`parley_dev` unresolved). Untouched
  by this pass and present at `bc3a3c71084`.
- 2026-09-18. Remaining `stylo` mentions outside dated docs and receipts, all
  provenance or dated history: three root `Cargo.toml` comments recording the
  2026-05-20 and 2026-08-21 deletions; harvest headers in
  `components/livery/src/values/{calc,logical,transform_matrix}.rs` and
  `values/color/{mod,mix,parse,space/mod,space/rgb}.rs`,
  `components/genet-livery/src/legacy_color.rs`, and `livery/build.rs:73,710`;
  the audit provenance in `livery/consumed_longhands.toml` and the generated
  block header in `livery/properties.toml`; `genet-livery/src/lib.rs:11,123`;
  one test name each in `livery/src/values/logical.rs` and
  `genet-livery/tests/cambium_lane.rs`; and the fork-fix note at
  `ports/genet-wpt/src/harness.rs:1591`.

## Progress

- 2026-09-18. S0 landed in the working tree. `cargo check` on
  `servo-malloc-size-of` and `malloc_size_of_tests` clean; genet-wpt 68 passed,
  3 ignored.
- 2026-09-18. C0 and C1: `components/cadency`, four modules on `cssparser`
  alone. Its suite is 10 tests over a fixture tree with two nested shadow
  trees: a 59-row accept table, a 62-row reject table, 15 specificity rows,
  and matching for every combinator, structural form and boundary crossing.
  clippy clean.
- 2026-09-18. C2: Livery's `selector.rs` went from 514 lines to 179 and lost
  `Atom`, `AttributeValue`, `LiverySelectorImpl`, `NoPseudoElement` and the
  upstream re-exports. `tests/selectors.rs` assertions unchanged; its fixture
  impl shrank by 60 lines. Suites: cadency 10, livery 206, genet-livery 556,
  genet-documents 9 (53 with `scripted`), ortet 32, genet-wpt 68, all green and
  equal to the counts recorded on 2026-09-16.
- 2026-09-18. C3, first two conditions: `cargo tree -i` finds none of
  `selectors`, `servo_arc`, `stylo_malloc_size_of`, `to_shmem`, `stylo`.
- 2026-09-18. C3 closed. WPT before (`bc3a3c71084`) and after over lane F's ten
  runs plus `shadow-dom` testharness (1523/8825): zero transitions of any kind
  in all eleven. The before binary reproduces lane F's recorded numbers
  exactly. `css/css-scoping` and `css/css-shadow-parts` are absent from the
  checkout and unclaimed. Receipt
  `Code/testing/genet/wpt-ledger/2026-09-18_cadency_selector_engine/results.md`.
- 2026-09-18. The preserved `imported_part` bug is fixed in a commit of its own,
  after the measurement: `components/genet-livery/src/dom.rs` now reads
  `exportparts` outer to inner. `exportparts_renames_a_part_for_the_outer_scope`
  in `tests/shadow_flat_tree.rs` failed before the fix and passes after;
  genet-livery 557 green. No WPT directory in the checkout covers renaming.
