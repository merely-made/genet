# The component model: custom elements, ElementInternals and shadow trees

**Status:** In progress, 2026-10-21. P1 (reaction stack and timing) is
implemented on branch `component-model` and is **not merged**. Matched maps
lose no previously passing subtest on either engine. `custom-elements` rises
2,251 → 2,654 on Boa and 2,255 → 2,658 on Vano. The numbers and their
attribution are under Progress. P2 (definition and upgrade) is next by the
ruled order. Mark's answers from three card rounds are recorded below.

Authority: [standards ledger](2026-09-07_standards_to_features_ledger.md) row 14
(Custom Elements and Shadow DOM, so that Cambium widgets can be hosted inside web
documents), picked as a conformance target by Mark on 2026-10-20 ("Fetch and
request security cluster (Recommended),WebCrypto,Component model"). It
continues the landed [Shadow DOM plan](2026-09-07_shadow_dom_plan.md) and
[parser/script interleaving plan](2026-09-08_parser_script_interleaving_plan.md),
and is bound by the arena semantic contract
[G5](../docs/2026-06-11_gc_arena_dom_plan.md#g5-arena-semantic-contract-partially-implemented-2026-09-08).
Form-associated custom elements touch the
[Forms value model](2026-10-07_forms_value_validation_submission_plan.md)
(Checkpoint C accepted 2026-10-20). That model is shared with this plan, not
copied into it.

Behaviour comes from HTML §4.13 (custom elements) and the DOM standard, with
member shapes taken from the vendored WebIDL
`tests/wpt/tests/interfaces/{html,dom,wai-aria,cssom}.idl`. The vendored WPT
directories `dom`, `html/dom`, `custom-elements` and `shadow-dom` measure the
work. No source is copied from another engine.

## Rulings (Mark's words; unnumbered until Mark numbers them)

Put to Mark as one card round on 2026-10-20, with the seven completed baseline
maps as evidence (`dom` was still running). Each card carried a recommendation.

- **Phase order.** Mark: **"Reactions first (Recommended)"**. P1 reaction
  stack and timing, with custom-element state in the arena; P2 definition and
  upgrade including `:defined`; P3 ARIA reflection, then ElementInternals and
  form-associated elements; P4 the four shadow defects and focus delegation; P5
  adoption and moveBefore; P6 remaining `dom` interfaces and `html/dom`
  reflection.
- **ElementInternals and the Forms model.** Mark: **"Native listed-control
  kind (Recommended)"**. An arena flag set at upgrade with a `formAssociated`
  definition; the listed, submittable, resettable and labelable predicates read
  it; `setFormValue`, `setValidity`, `validity`, `form` and `labels` go through
  native sinks into the existing Forms state; the Forms plan carries a
  cross-reference.
- **G5.** Mark: **"Record at the arena (Recommended)"**. Reaction candidates
  are a third typed record stream at the arena's one mutation point, recorded
  only while a definition exists. Custom-element state moves into the arena
  node. A `[CEReactions]` wrapper generated from the vendored IDL drains the
  stream; the parser and native edits use the backup element queue. Queued
  reactions hold wrappers, so their nodes stay alive, with a regression for an
  `innerHTML` replacement that leaves a disconnectedCallback pending.
- **Adopted reactions (second round, F4).** Mark: **"JS adoption step records
  it (Recommended)"**. The bootstrap's adoption step, which already computes the
  old and new documents, calls a native sink. The sink walks the moved subtree
  shadow-including and appends `adopted` records to the same arena stream, and
  they are delivered through the same reaction stack. The plan records this as
  G5's explicit lower-level contract until ownership moves into the arena in the
  adoption phase.
- **Reactions inside `document.write` (third round, F8).** Mark: **"Fix it in
  P1 (Recommended)"**. While a definition exists, written source is fed to the
  tokenizer a tag at a time. The write call hands control back to script at
  each element creation, where the bootstrap upgrades the element and runs that
  insertion's reactions before the write continues.
- **Scope.** Mark: **"Scoped later, tentative out (Recommended)"**. Scoped
  registries become a phase after definition and upgrade, and from P1 the
  definition lookup is shaped so it can take a registry. Reference target,
  declarative adopted stylesheets, `dom/observable` and
  `HTMLSubmitButtonBehavior` stay out of this plan as residuals.

## Phases

Each phase is done when its focused regressions pass on both engines, the
crate gates it touches are green, and matched before/after maps for all four
directories on both engines lose no previously passing subtest, with every
movement attributed.

**P1. Reaction stack and timing.**
- The arena records reaction candidates (connected, disconnected, adopted,
  attribute changed) at its mutation point for elements whose custom-element
  state is `custom`. Recording starts with the first definition.
- Custom-element state lives in the arena node.
- The bootstrap keeps HTML's custom element reactions stack: element queues,
  per-element reaction queues and the backup element queue.
- Every `[CEReactions]` member in the vendored IDL is wrapped, from a table
  the interface-table generator emits, and the wrapper drains the arena
  stream.
- The JS-side enqueue sites are removed, so nothing is delivered twice.
- `attributeChangedCallback` receives its namespace argument.
- A throwing callback is reported and the remaining reactions still run.
- Done-conditions:
  - `attribute-changed-callback.html`, `connected-callbacks.html` and
    `disconnected-callbacks.html` move on both engines.
  - A regression shows a parser-inserted custom element receives
    `connectedCallback` through the backup queue.
  - A regression shows the `innerHTML`-replacement node stays intact for its
    pending `disconnectedCallback`.

**P2. Definition and upgrade.** The upgrade reaction in its element queue,
the failed state, `:defined`, customized built-ins on `HTMLElement`-interface
tags, clone and import upgrades, and the order in which `define` reads the
constructor's properties.

**P3. ARIA reflection, then ElementInternals.** ARIA reflection from
`wai-aria.idl` through the interface table, then form-associated custom
elements as a native listed-control kind, then CustomStateSet with `:state()`.

**P4. Shadow defects and focus delegation.**
- Declarative attachment checks the shadow-host element names.
- `slotchange` is queued as a signal-slot task.
- Event paths for targets with a `relatedTarget` climb through the shadow
  roots.
- `elementFromPoint` on Document and ShadowRoot.
- `delegatesFocus`.

**P5. Scoped registries.** Per Mark's scope ruling.

**P6. Adoption and moveBefore.**

**P7. Remaining `dom` interfaces and `html/dom` reflection.**

## Findings

**F1, 2026-10-20: measurement.** The frozen "before" runner was built from main
`16b971188f3` with a clean tree. The lockfile digest is `2cb3d086…`, the same one
the WebCrypto lane froze. The runner itself is `genet-wpt-before.exe`, sha256
`40d61bb3…`. Settings: `--jobs 1 --timeout 120 --drive-deadline 15 --verbose`,
disk mode, Livery. The 2026-09-29 census used a 30 s timeout with four workers,
so its counts are not comparable. `custom-elements` on Vano reads 2,255 of 4,034
here, against 265 of 1,456 in the census. Evidence:
`Code/testing/genet/component-model/`.

**F2, 2026-10-20: where each piece lives.**

| Piece | Lives in | Shape today |
|---|---|---|
| Registry, definitions, upgrade, reactions | `script-runtime-api/dom/bootstrap.js` (about 2470–2710 and 6209–6314) | JavaScript only. Custom-element state is a JS `WeakMap`; reactions are flushed in a microtask; connect and disconnect walk `childNodes`, not shadow-including descendants |
| Parser upgrade hooks | `bootstrap.js` `__ceUpgradeParsed`, `__ceDefinedCount`; `genet-scripted-dom/parser.rs` | Upgrade at element creation once a definition exists |
| Arena, mutation point | `genet-scripted-dom/lib.rs` (`Node`, `ScriptedDom`, `ObservedMutation`) | One mutation point feeds both layout records and observer records; no custom-element state |
| Shadow roots, slots | `genet-scripted-dom/shadow.rs`, `script-runtime-api/dom/shadow.rs` | Native, landed 2026-09-07/08 |
| Event path, retargeting | `genet-events/path.rs`, `script-runtime-api/events.rs`, bootstrap | Native path, JS dispatch |
| Adoption | `genet-scripted-dom/adoption.rs`, `script-runtime-api/dom/adoption.rs` | Native |
| Interface table | `support/idl-interface-table` → `script-runtime-api/dom/html_interfaces_generated.rs` | Generated from vendored IDL with a drift test; does not yet carry `[CEReactions]` |
| Forms value model | `genet-scripted-dom/forms*.rs` | Native; whether an element is a listed control comes from a tag-name table (`LISTED`, `forms_validation.rs:113`, `is_listed_control` :572) |
| ElementInternals, ARIA reflection | nowhere | Absent |

**F3, 2026-10-20: causes, `custom-elements` on Vano.** Failing subtests, with
files in brackets: reaction queue and timing 688 (70); definition and upgrade
477 (29); scoped registries 353 (31); form-associated and ElementInternals
253 (24). A small remainder is harness-dependent.

- **Reactions.** The reaction tests check the callback log straight after the
  DOM call returns, for example in `attribute-changed-callback.html` and
  `reactions/resources/reactions.js`. Genet runs callbacks in a later microtask,
  so they have not run yet. `attributeChangedCallback` also lacks its
  `namespace` argument.
- **ARIA reflection is missing.** About 104 reaction subtests (`AriaMixin-*`)
  depend on it, and so do `ElementInternals-accessibility` and
  `ElementInternals-role`.
- **Definition and upgrade.** `builtin-coverage.html` accounts for 333 failing
  subtests. `customElementConstructibleWithHtmlInterface` refuses any
  customized built-in whose interface is `HTMLElement` (`b`, `cite`, `article`
  and so on), and a cloned customized built-in loses its custom constructor.
  `:defined` is not implemented, so both `pseudo-class-defined*` files time out.

The vendored IDL marks 541 members `[CEReactions]`: 49 in DOM, 436 in HTML, 52
in ARIA and 4 in CSSOM.

**F3b, 2026-10-20: the other baselines.** Passing subtests in the "before"
maps (Boa / Vano):

| Directory | Boa | Vano |
|---|---|---|
| `custom-elements` | 2,251 / 4,026 | 2,255 / 4,034 |
| `shadow-dom` | 1,501 / 2,237 | 1,501 / 2,237 |
| `html/dom` | 18,077 / 26,325 | 5,743 / 10,486 |
| `dom` | 15,100 / 19,151 | 12,074 / 15,576 |

The `html/dom` gap between the engines is timing, not behaviour: Boa finishes
three more large reflection files inside the 120 s budget. Each engine is only
ever compared with its own "before" map.

Causes in `shadow-dom`, the same on both engines:
- **Declarative attachment.** Declarative roots are attached to elements that
  may not host a shadow root (96 subtests).
- **Early `slotchange`.** `flushSlotChanges` fires `slotchange` synchronously
  (0 of 32).
- **Truncated event paths.** Paths for a target with a `relatedTarget` stop at
  the target (0 of 18).
- **Missing `elementFromPoint`.** It is absent on ShadowRoot and Document
  (0 of 41).

In `html/dom`, ARIA reflection accounts for about 1,520 failing subtests.

**F4, 2026-10-20: the arena does not know a node's document.** The owning
document lives in the bootstrap's `ownerDocuments` map, as G5's 2026-09-07
review recorded, so the arena cannot produce `adopted` by itself. Mark's second
ruling (above) has the adoption step record it. HTML's adopt step removes the
node before enqueueing `adopted`. The bootstrap therefore removes the node from
its old parent explicitly before recording, so each element sees disconnected,
then adopted, then connected.

**F5, 2026-10-20: G5 lifetime.** `release_subtree` freed a removed subtree
whenever no observer was active, and `collect` knew nothing of queued
reactions. While reactions are recorded, `release_subtree` now leaves the
subtree for collection, and `collect` seeds every pending candidate's node.
`a_pending_candidate_keeps_its_removed_element_alive` fails with both guards
reverted ("removal freed a node a reaction still names") and passes with them.

**F6, 2026-10-20: one reactions stack per realm.** HTML keeps one reactions
stack per similar-origin window agent. Genet keeps one per realm. Candidates
are drained agent-wide (`ceFlush`), and each realm enqueues only for the
elements its own registry upgraded. So when one realm's member mutates an
element another realm defined, that element's reactions arrive through the
other realm's backup queue instead of at the end of the member. Recorded as a
residual for the adoption phase.

**F8, 2026-10-21: the first "after" maps.** `custom-elements` rose 2,251 →
2,643 on Boa and 2,255 → 2,647 on Vano, with 12 lost rows on each engine.
- **Ten were a bug of mine.** A failed upgrade dropped the element's
  definition, so each later `define` reran the throwing constructor ("name
  already defined") and the rest of `CustomElementRegistry.html` cascaded. An
  upgrade now proceeds only from `undefined` or `uncustomized`, read from the
  arena. `a_failed_upgrade_is_not_retried` fails without the guard and passes
  with it.
- **The other two rows are one file.**
  `parser/parser-constructs-custom-element-in-document-write.html` passed
  vacuously: `connectedCallback` never ran for a parser-inserted element, so the
  count it checks kept its initial 0. Once the callback did run, it ran after
  the write had parsed the children (2), because the written string was
  tokenized whole inside one native call. Fixed under Mark's third ruling;
  `a_written_custom_element_is_connected_before_its_children` covers it.

`shadow-dom` gained one subtest and lost none on either engine.

`html/dom` on Vano jumped 5,743 → 34,519 with no losses. Six reflection files
that the "before" run killed at 120 s now complete. A control reran the
"before" runner on those six files at the same hour. Five complete with exactly
the "after" counts; `reflection-embedded.html` still hits the limit. So the
movement is machine load at the budget edge, not this change.

**F9, 2026-10-21: removing the JS walks makes DOM mutation cheaper.** The old
bootstrap walked the whole affected subtree in JavaScript, through the agent
dispatch, on every insertion, removal and `innerHTML` replacement. It did so
whether or not anything was defined. P1 deletes those walks, and the arena
records the facts at the mutation. Standalone on Vano under the same load:
- `dom/nodes/ParentNode-querySelector-All.html` is killed at 120 s by the
  "before" runner and finishes in 96 s on `after-r1` (1,829 / 1,975).
- `dom/ranges/Range-isPointInRange.html` is at the limit on both runners
  (128 s and 129 s). In the matched window it completed on `after-r1`
  (5,460 / 5,733).

The `dom` gains on Vano are mostly these two files reaching completion: 7,289
of 7,292.

**F7, 2026-10-20: wrapping cost.** A wrapped member costs one property check
until the first definition anywhere in the agent (`ceShared.active`). After
that it costs one native drain per call. The fast path matters because the
large reflection files already sit near the 120 s budget on Vano.

## Progress

- 2026-10-20: worktree `Code/worktrees/genet-component-model` on branch
  `component-model` at `16b971188f3`; "before" runner frozen; baseline maps
  started on both engines.
- 2026-10-21: **P1 implemented and measured.** It touches six areas:
  - Arena (`genet-scripted-dom/custom_elements.rs`): custom-element state on
    the node, the reaction-candidate stream at every tree and attribute
    mutator, the shadow-including walk, and G5 retention in `release_subtree`
    and `collect`.
  - Runtime sinks (`script-runtime-api/dom/custom_elements.rs`), the agent
    operations `ceActive` and `ceFlush`, and `__cePump` at the microtask
    checkpoint.
  - The generator (`support/idl-interface-table`), which now emits
    `CE_REACTIONS`: 601 pairs.
  - The bootstrap's reactions stack and the `[CEReactions]` wrappers. The old
    JS connect, disconnect, adopted and attribute call sites are removed.
  - `adopted` recorded by the adoption step (F4).
  - Written source fed a tag at a time while a definition exists (F8).

  **Regressions:** 7 arena tests and 10 runtime tests, each on Boa and Vano.
  Controls: the G5 retention test and the failed-upgrade test both fail with
  their fix removed.

  One existing unit test, `custom_elements_customized_builtins`, pinned the old
  microtask timing; its expectations now follow HTML. The event order and the
  final state are unchanged.

  **Crate gates (Rust 1.99.1):**

  | Crate | Result |
  |---|---|
  | `genet-scripted-dom` | 210 lib + 27 integration, all pass |
  | `genet-idl-interface-table` | 14 pass |
  | `genet-scripted` | 133 pass |
  | `script-runtime-api` | 1,142 pass, 10 fail |

  The 10 `script-runtime-api` failures are the set the WebCrypto lane recorded
  as failing on main's own source at this base commit: `atomic_refusal` ×2,
  `associated_subtree_boundary` ×2, Forms implicit submission ×6. This lane did
  not rebuild main to recheck them.

  **Runners:**
  - "before" `40d61bb3…` built from main `16b971188f3`.
  - Candidate `after-r1` `6d8e05c8…`; source diff
    `bin/after-r1.source.diff`.
  - Both use lockfile `2cb3d086…`.
  - `html/dom` and `dom` were re-measured with the unchanged runner (tag
    `beforeB`) in the same window as `after-r1` (F8).

  **Matched accounting** (`account_p1.py`; passing subtests before → after,
  lost rows):

  | Directory | Boa | Vano | Lost |
  |---|---|---|---|
  | `custom-elements` | 2,251 → 2,654 | 2,255 → 2,658 | 0 / 0 |
  | `shadow-dom` | 1,501 → 1,502 | 1,501 → 1,502 | 0 / 0 |
  | `html/dom` | 42,639 → 42,639 | 22,681 → 22,681 | 0 / 0 |
  | `dom` | 32,131 → 32,134 | 12,913 → 20,205 | 0 / 0 |

  **Attribution:**
  - **`custom-elements`:** 439 gained rows per engine. The done-condition
    files on Vano:

    | File | Before | After |
    |---|---|---|
    | `attribute-changed-callback.html` | 0 / 13 | 13 / 13 |
    | `connected-callbacks.html` | 7 / 40 | 31 / 40 |
    | `disconnected-callbacks.html` | 7 / 40 | 31 / 40 |
    | `adopted-callback.html` | 0 / 71 | 42 / 71 |
    | `reactions/Element.html` | 18 / 47 | 41 / 47 |
    | `parser/…-in-document-write.html` | 1 / 2 | 2 / 2 (now for the real reason) |

    The 20 or 21 changed rows per engine:
    - 18 come from four `state/` files whose `connectedCallback` calls
      `attachInternals`. That exception is now reported, as HTML specifies,
      instead of being lost in a microtask, and testharness turns it into a
      harness error. These files had 0 passes on both sides and wait on P3.
    - 2 are rows that did not run (`not-run`) on the "before" side.
    - Boa has one more, `upgrade-custom-element-error-event`, which now times
      out.
  - **`shadow-dom`:** one gained subtest per engine.
  - **`html/dom`:** no movement in the matched window. The earlier unmatched
    jump was load (F8).
  - **`dom` on Vano:**
    - 7,289 of 7,292 gains are `Range-isPointInRange` and
      `ParentNode-querySelector-All` completing inside the 120 s budget (F9).
    - Three are reaction gains: `Node-appendChild-cereactions-vs-script`,
      `EventTarget-add-listener-platform-object` and one
      `moveBefore/custom-element-move-reactions` subtest.
    - The 421 changed rows are the subtests those two files now report.
  - **`dom` on Boa:** the same three reaction gains (+3 subtests, two more
    files pass), and no file crossed the budget.

