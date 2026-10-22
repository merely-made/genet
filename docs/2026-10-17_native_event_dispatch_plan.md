# Native event dispatch, focus and activation (plan, 2026-10-17)

Status: **approved 2026-10-17 as proposed (rulings in §6); Phases 0 and 1 in progress.** Supersedes the
two-dispatcher "shared scenario table" approach of the
[2026-06-01 convergence plan](./2026-06-01_event_model_convergence_plan.md): that
plan kept two algorithms and tested them for agreement. This one removes the
second algorithm.

## Problem

WPT tests Genet, not Cambium. Cambium inherits conformance only where it goes
through Genet. Today events, focus and activation exist twice, so WPT progress on
`dom/events` or `html/interaction/focus` never reaches the GUI, and the GUI's
focus/activation behaviour is not under WPT at all.

Cambium's ownership rule says otherwise
([architecture, "Ownership"](../../mere/design_docs/cambium_docs/technical_architecture/2026-09-03_cambium_architecture.md)):
"Genet owns DOM, style, layout, paint, input, accessibility, and browser
behavior." Two precedents already follow it: accessible names (Genet computes,
Cambium's AccessKit tree and Mesquite selectors consume) and forms field
ownership (moved to Genet 2026-10-07).

## Verified current state (2026-10-16)

Cambium (`mere/crates/cambium/cambium/src/`):

- `context.rs` keeps eight listener registries on `GenetCtx`:
  `register_click` (:339), `register_key` (:378), `register_focusable` (:422),
  `register_focus` (:447), `register_pointer` (:464), `register_value` (:481),
  `register_file` (:491), `register_hover` (:511), `register_wheel` (:528).
  Click and key handlers carry a per-listener `capture` bit.
- `runner.rs` owns: the focus pointer (`RunnerTree.focus`), click-to-focus and
  focus/blur routing (`dispatch_click` :426, `route_focus_transition` :372), the
  capture-then-bubble walk (`phase_ordered_paths` :345, `Propagation` flags
  checked between paths), Tab traversal (`focus_traverse` :591,
  `collect_focusables` :1506 pre-order, optional wrap or exit), Enter/Space
  activation (`semantic_activation_matches` :1519, synthesised click in
  `dispatch_key` :641), default-prevented tracking (`last_default_prevented`),
  pointer capture (:814-:917), hover (:944), wheel (:1104), dead-node scrubbing
  (:292).
- Not a bubbling walk: pointer, hover, wheel and value route to the *nearest
  ancestor with a handler*, not through a path.

Genet (`genet/components/`):

- `script-runtime-api/lib.rs:1236` `dispatch_event` evals `__dispatchSynthetic`;
  `"click"` diverts to `dispatch_form_activation`
  (`form_submission.rs:325`, `formActivateRaw`); keydown to `dispatch_form_keydown`
  (:335, implicit submission only). Nothing runs without a script engine.
- The real algorithm is JS: `dom/bootstrap.js` `addEventListener` :994, `fire`
  :1013 (once, passive, stopImmediate, exception reporting), `dispatchNodeEvent`
  :1042 (path through shadow host when `composed`, retargeting, window appended,
  click activation target via `formOperations`). The path walk uses
  `parentNode` plus shadow host; I found no assigned-slot step in it (verify in
  Phase 1; `assignedSlot` itself exists, :6565).
- Focus: only a host-projected `hostFocusedRef` set by `formFocusRaw`
  (`form_submission.rs:316`) feeding `document.activeElement` (:2767). A grep of
  the DOM bootstrap found no `focus()`/`blur()`, sequential navigation, focus
  events or tabindex processing (only the reflected `tabIndex` IDL). So the
  Genet side of `html/interaction/focus` is largely greenfield; Cambium's runner
  is currently the only working focus model.
- Shadow state needed by the path already lives in Rust: `shadow_root_of`,
  `shadow_host_of`, `assigned_slot_of` (`genet-scripted-dom/shadow.rs`).

## 1. Inventory: spec behaviour Cambium implements itself

| Behaviour (Cambium today) | Spec area | WPT dir | Disposition |
|---|---|---|---|
| Listener lists per node, registration order, stacked listeners | DOM EventTarget | `dom/events` | Move |
| `capture()` per-listener phase; capture/bubble walk | DOM dispatch algorithm | `dom/events` | Move |
| `stop_propagation` / stop-immediate / `prevent_default` via shared cell | DOM Event flags | `dom/events` | Move |
| Passive listeners (none natively) | DOM, UI Events wheel/touch | `dom/events` | Add via layer |
| Shadow retargeting, `composedPath` (none natively) | DOM, Shadow DOM | `dom/events`, `shadow-dom` | Add via layer |
| Click target resolution, click event | UI Events, Pointer Events | `uievents/mouse`, `pointerevents` | Move |
| Click-to-focus (nearest focusable ancestor, clear on miss) | HTML focus update steps, mousedown default | `html/interaction/focus`, `uievents/focus` | Move |
| focus/blur routing, lost-then-gained order | HTML focus events, UI Events | `uievents/focus`, `html/interaction/focus` | Move (adds focusin/out, relatedTarget) |
| Focus pointer, liveness scrub on removal | HTML focus fixup rule | `html/interaction/focus/focus-management` | Move |
| Focusable set = key handler or `focusable()` marker | HTML focusable areas, tabindex | `.../sequential-focus-navigation-and-the-tabindex-attribute` | Move; Cambium supplies a native flag |
| Tab / Shift+Tab, DOM pre-order, wrap | HTML sequential focus navigation | `html/interaction/focus` | Move (spec order: tabindex>0 first, shadow-flat) |
| Focus exits at set ends (`FocusExit`) | HTML "move focus to browser UI" | same | Move as outcome; wrap-vs-exit stays host policy |
| Enter/Space synthesises click on native button/other | UI Events keyboard, HTML activation behaviour | `uievents/keyboard`, `html/semantics/forms`, `html/semantics/interactive-elements` | Move (native elements) |
| Enter/Space on `role=button/checkbox/switch` | ARIA APG, not core spec | none | Stays Cambium policy, registered as a native activation hook |
| Default-prevented gates host default (Tab, activation, Space scroll) | DOM canceled flag, "default action" | `dom/events` | Move: `DispatchOutcome` |
| Pointer-down capture, move/up routing, owner pointer id | Pointer Events capture | `pointerevents` | Move |
| Hover enter/leave routed to nearest handler | Pointer/Mouse enter-leave-over-out | `pointerevents`, `uievents/mouse` | Move (semantic change, see risks) |
| Wheel to nearest scroll handler | UI Events WheelEvent, passive defaults | `uievents/mouse`, `dom/events` | Move |
| `OpenFile` request/response | HTML file input activation | `html/semantics/forms/the-input-element` | Request seam stays; activation moves |
| Value events (AccessKit set-value) | none (assistive tech action) | none | Stays Cambium; delivered as a native synthetic event |
| Roving tabs (`take_tab_focus`, tablist arrows), radio groups, command-menu F10/Alt | APG widget patterns | none | Stays Cambium policy |

## 2. Target design: a Rust EventTarget layer

One registry and one dispatch algorithm in Rust, with no script-engine types.

```text
Registry (per document)         EventState (Rc, shared)         Invoker (per owner)
  node -> [Listener]              type, bubbles, cancelable,      fn invoke(&mut self,
  Listener {                      composed, trusted, data,          listener_token,
    owner: OwnerId,               target, current_target,           &mut EventState)
    token: u64,                   phase, path, flags:
    type, capture,                stop, stop_immediate,
    once, passive }               canceled, in_passive
```

- **Listener = (owner, token).** Owners are `Script` (JS function, token is the
  engine's rooted callback handle) or a native owner such as a Cambium runner
  (token indexes the runner's view-path table). The layer never calls a closure
  it cannot name; it asks the owner's `Invoker`. This keeps engine types out and
  gives Cambium its `View::message` call with `&mut State`.
- **Dispatch** implements the DOM algorithm: get-the-parent with shadow-including
  path and assigned slots, `composed`, target retargeting per path item, window
  at the root, `composedPath` visibility for closed roots, capture / at-target /
  bubble, per-node listener snapshot, `once`, `passive` (preventDefault ignored
  while in a passive listener), `stopPropagation`, `stopImmediatePropagation`,
  canceled flag, exception reporting delegated to the owner, legacy pre-activation
  and activation behaviour (below). Returns
  `DispatchOutcome { canceled, default_actions_run }`.
- **Mixed documents interleave in spec order** because both owners' listeners sit
  in the same per-node list, ordered by registration. One `dispatch` call drives
  all owners.
- **Activation behaviour** (`activation_behavior(node, event)`): the element
  table currently in `genet-scripted-dom` forms code plus `<a>`, `<label>`,
  `<details>/<summary>`, checkbox/radio legacy-pre-activation. Runs after the
  path completes if not canceled. Keyboard Enter/Space synthesise a click through
  the same `dispatch`, so native and script clicks share one path.
- **Focus module** (`focus.rs` in the same crate): focus update steps with
  focus/blur/focusin/focusout and `relatedTarget`, `activeElement` including
  shadow retargeting, `focus()`/`blur()`, `autofocus`, fixup on removal or
  `inert`/`hidden`, tabindex parse, focusable-area test, sequential navigation
  (tabindex>0 ascending, then 0 in shadow-flat tree order, `delegatesFocus`,
  sequential starting point), `Navigation::{Moved, Exited(dir)}`. Wrapping is a
  parameter.
- **`FocusEnv` trait** supplied by the host for what the DOM cannot know: "being
  rendered" (display:none, visibility), inert, and a native-focusable flag for
  toolkit nodes. A script-only document gets a conservative default.
- **Event data.** `EventData::{Plain, Pointer, Key, Wheel, Focus, Custom}` carries
  typed payloads. The JS `Event` object becomes a projection of `EventState`
  (flags `__stop`, `__canceled` etc. disappear from JS); Cambium's `PointerClick`
  and `KeyEvent` wrap the same state.
- **Fast path.** A per-document count of listeners by type lets `pointermove` and
  `wheel` skip path construction when nobody listens, so Cambium's hot path does
  not regress.

## 3. Crate ownership and seam effects

- **New crate `genet-events`** (`components/genet-events`), depends on
  `genet-scripted-dom` (NodeId, tree, shadow accessors) and `layout-dom-api`;
  **no** dependency on `script-engine-api`, Boa or Nova. `publish = false` until
  the API settles.
- `script-runtime-api` depends on it and keeps only a thin binding:
  `addEventListener` registers `Listener{owner: Script, token}`; its `Invoker`
  calls the engine through `script-engine-api`; `dispatchEvent` calls the layer;
  `dispatch_event` and `dispatch_form_*` become wrappers over host-facing
  `genet-events` calls. `bootstrap.js` loses `fire`, `dispatchNodeEvent`, the
  listener maps and `formOperations` wiring (~150 lines), keeping the wrapper
  classes. The crate stays engine-neutral.
- `genet-scripted-dom` (published, 0.1.2): **additive only**: an
  `event_parent(node, event)` helper (slot/shadow aware) if the accessors are not
  enough, a live-node hook so the registry can drop listeners on node removal, and
  possibly tabindex/disabled helpers. Additive pub methods mean a minor bump; no
  existing signature changes. The form activation code currently here stays here
  and is called by `activation_behavior`.
- `layout-dom-api` (published): **no change** in the plan. The layer does not
  need to be generic over `LayoutDom`; Cambium already depends on `ScriptedDom`.
- Non-Node EventTargets (`AbortSignal`, `MessagePort`, `WebSocket`, `Worker`,
  `XMLHttpRequest`) stay on the JS side through Phase 5. The registry's key is an
  `EventTargetId` so they can move later.

## 4. Cambium migration

The runner shrinks to translating "listener fired → meristem message path".

- `GenetCtx::register_click/key/...` stay as `pub` methods with unchanged
  signatures. They now call `Registry::add_listener(node, Listener{owner:
  cambium, token})` and record `token → view path` locally. Unregister removes
  by token. The eight maps and `Handler` bookkeeping disappear.
- Cambium implements `Invoker`: `invoke(token, state)` builds the typed
  `DynMessage` from `EventState` and calls `view.message(...)`. The rebuild
  happens after `dispatch` returns, exactly as today (no rebuild between
  listeners now).
- Public API stays stable: `OnClick`, `OnKey`, `.capture()`, `Propagation`.
  `Propagation` becomes a handle onto the shared `EventState`; its methods keep
  their names. The Cambium-only `key_target`/`tab_focus` fields move to a
  Cambium-side post-dispatch hook.
- `focus`, `focus_traverse`, `focusables`, `set_focus`, `set_focus_exits` become
  calls on the layer's focus module; `FocusExit` maps to `Navigation::Exited`.
  `focusable()` and key-handling nodes set the native-focusable flag.
- Pointer, hover, wheel: native listeners on pointerdown/move/up, pointerover/out
  and wheel. "Nearest handler" is reproduced by the handler calling
  `stop_propagation()` after it handles the event.
- Stays in Cambium: roving tabs, radio groups, command-menu keys, ARIA-role
  activation (registered as a native activation hook), value events, focus-exit
  vs wrap choice.
- Hosts (`cambium-genet-winit-host`, `genet-winit-host`) call one entry point
  (`dispatch_input`) for hit-tested pointer and focused keyboard input, instead
  of choosing between runner methods and `ScriptRuntime::dispatch_event`.

## 5. Phasing and gates

Each phase lands on main on its own and keeps both products green. "Cambium
green" means `cargo test -p cambium` (`tests.rs`, 146 test functions plus the
runner and per-control tests) and the Mesquite scenario suites in
`crates/cambium/mesquite`. "WPT gate" means the pass count per file does not drop
against the recorded baseline in `ports/genet-wpt/expectations/testharness` and the
named subset runs **through the Rust layer** (the script binding delegates; no JS
fallback path).

| Phase | Work | Gate |
|---|---|---|
| 0 | Record baselines: `dom/events`, `uievents`, `pointerevents`, `html/interaction/focus`, selected `html/semantics` subsets, per-file pass counts on Boa and Nova. Keep the 2026-06-01 scenario table as the cross-check. | Baselines committed to the expectations tree. |
| 1 | `genet-events` core: registry, path (shadow, slots, window), phases, flags, once/passive, `composedPath`. JS binding delegates; JS dispatcher deleted. Cambium untouched. | WPT `dom/events` subset, `shadow-dom` event tests: no per-file regression, assigned-slot path cases improve. `script-runtime-api` event tests pass on Boa and Nova. Cambium green. |
| 2 | Cambium click/key/focus listeners register on the layer; runner routes through `Invoker`; eight registries removed. Mixed-document test (JS and Cambium listeners on one node interleave by registration). | Cambium green, including capture/stop/prevent tests. Shared scenario table passes against the single implementation. `dom/events` unchanged. |
| 3 | Activation behaviour and keyboard activation into Rust; `formOperations` JS wiring removed. | WPT `html/semantics/forms` (checkbox, radio, button, label), `interactive-elements`, `uievents/keyboard` subsets: no regression. Cambium activation tests and Mesquite click/keyboard scenarios green. |
| 4 | Focus model: focus update steps and events, `focus()`/`blur()`, `activeElement`, fixup, sequential navigation, `delegatesFocus`, autofocus. Runner focus code deleted. | WPT `html/interaction/focus` (`focus-management`, `sequential-focus-navigation-and-the-tabindex-attribute`, `document-level-focus-apis`) and `uievents/focus` improve over baseline. Cambium Tab/Shift+Tab, focus-exit and click-to-focus tests green. |
| 5 | Pointer capture, hover/enter/leave, wheel through the layer; runner reduced to the translator; update the architecture "Ownership" section to name this. | WPT `pointerevents` and `uievents/mouse` subsets: no regression. Cambium pointer, hover, wheel and slider tests and Mesquite hover/drag/wheel scenarios green. Hot-path benchmark (pointermove, wheel) within noise of Phase 0. |

If a gate fails, the phase reverts rather than carrying a fallback dispatcher.

## 6. Risks and open questions for the owner

1. **Crate placement.** Recommended: new `genet-events`. Alternative: inside
   `genet-scripted-dom`, which avoids a registry-cleanup hook but grows a
   published crate and ties events to its release cadence. Which?
   **Ruling (ruled 2026-10-17: as proposed):** a new `genet-events` crate.
2. **Re-entrancy.** A script listener can call `dispatchEvent` that reaches a
   Cambium listener while the runner is borrowed (native dispatch → JS listener →
   nested dispatch). Proposed rule: the runner's state sits behind a try-borrow
   and a nested native invoke is queued and run when the outer dispatch unwinds,
   with a flagged ordering deviation. Acceptable, or must nested dispatch be
   synchronous?
   **Ruling (ruled 2026-10-17: as proposed):** a nested native invoke is queued and
   runs when the outer dispatch unwinds; the ordering deviation stays flagged.
3. **Focusable marking.** Cambium nodes become focusable through a native flag in
   the layer (no DOM change, proposed) or by emitting `tabindex="0"` attributes
   (visible to selectors and accessibility, but mutates the DOM and the AccessKit
   projection).
   **Ruling (ruled 2026-10-17: as proposed):** a native focusable flag, no
   `tabindex` mutation.
4. **Focus scope with several windows in one document.** Cambium's forest-DOM
   projection mounts N windows as sibling subtrees of one document, but a spec
   document has one focused area. Proposal: a focus scope per mount root. Needs a
   ruling because it departs from the spec's single focused area.
   **Ruling (ruled 2026-10-17: as proposed):** one focus scope per mount root.
5. **Behaviour changes users will see.** Tab order moves from pre-order to spec
   order (positive tabindex first, shadow-flat). Hover moves from "nearest
   handler" to enter/leave semantics; mitigated by stop_propagation in the
   handler, covered by Mesquite hover scenarios. Accept both?
   **Ruling (ruled 2026-10-17: as proposed):** accept the Tab-order and hover
   behaviour changes.
6. **JS callback lifetime.** Registry-held script tokens must keep callbacks
   rooted and release them when the node dies or the realm tears down. The shape
   of that handle differs between Boa and Nova; Phase 1 needs a short spike on
   both before the design is final.
   **Ruling (ruled 2026-10-17: as proposed):** use the callback-lifetime shape the
   plan proposes.
7. **Performance.** Per-dispatch `EventState` and cross-owner invocation on
   pointermove and wheel. Fast path in §2 plus the Phase 5 benchmark gate; if it
   fails the gate, the fallback is a per-type native listener index, not a second
   dispatcher.
   **Ruling (ruled 2026-10-17: as proposed):** the plan's fast path and its Phase 5
   gate.
8. **ARIA-role activation.** Spec is silent on Enter/Space for `role=button`. Keep
   as Cambium policy (proposed) or drop it and require authors to use native
   elements.
   **Ruling (ruled 2026-10-17: as proposed):** `role=button` Enter/Space stays
   Cambium policy.
9. **Superseded plan.** Mark the 2026-06-01 convergence plan as superseded once
   Phase 2 lands, keeping its scenario table as a permanent test.
   **Ruling (ruled 2026-10-17: as proposed):** mark the 2026-06-01 convergence plan
   superseded after Phase 2.

## 7. Phase 0 and 1 results (2026-10-17)

Baselines are `ports/genet-wpt/expectations/testharness/<subset>_<engine>.json`,
recorded with the release runner (`--jobs 6`) on the rebased main before any
Phase 1 code. Subtests passed / total, Boa and Nova identical except where shown.

| Subset | Phase 0 | Phase 1 |
|---|---|---|
| `dom/events` | 308/721 Boa, 309/723 Nova | 314/721 Boa, 315/723 Nova |
| `shadow-dom` | 1462/2237 | 1501/2237 |
| `uievents` | 20/116 | 20/116 |
| `pointerevents` | 145/638 | 145/638 |
| `html/interaction/focus` | 117/639 | 117/639 |
| `html/semantics/forms/the-input-element` | 904/1457 | 904/1457 |
| `.../the-button-element` | 42/63 | 42/63 |
| `.../the-label-element` | 83/159 | 83/159 |
| `html/semantics/interactive-elements` | 24/512 | 24/512 |

No file regressed in any subset on either engine. `dom/events` and `shadow-dom`
files that gained passes are the expectation files now committed at the Phase 1
values; the other subsets keep their Phase 0 files.

What Phase 1 built, and where it differs from the sketch in §2:

- `genet-events` holds the registry, the shadow-including path (slots, closed
  trees, window), phases, stop/once/passive flags and `composedPath`. The
  window is keyed per realm, not per object: the WindowProxy outlives a
  navigation but its listeners must not.
- A JS callback stays on its target's wrapper (agent-wide `WeakMap`), so it is
  collected with the wrapper; the registry holds only `(owner, token)`. Rust
  runs a listener through the dispatching realm's own `evHook`, handed to every
  native dispatch, so a wrapper whose realm was discarded still works. This is
  the callback-lifetime shape ruled in §6 item 6; no per-engine rooted handle was
  needed. Dead nodes' listeners are dropped at the GC tick.
- Deleted: the JS `fire`, the path walk in `dispatchNodeEvent`, `EventTarget.__fire`,
  `__composedPathVisible`, the `__listeners` maps and `__dispatchSynthetic`. Host
  input dispatch now builds its event in a three-line `__dispatchHostEvent` and
  hands it to the native walk; `dispatchNodeEvent` survives only as the call into it.
- Activation behaviour is still the JS `formOperations` code, reached from the
  Rust walk through the same hook (Phase 3 moves it).
- Not done yet: `relatedTarget` retargeting (the field is not projected), `signal`
  in `addEventListener` options, and `window.event`.
- Environment notes: dedicated-worker WPT variants error on every engine at
  current main ("Worker cancellation is unsupported by this script engine"), so
  they are baselined as errors. `genet-compositor` does not compile on macOS at
  current main (`compositor_calayer/errors.rs` and `iosurface.rs` are missing),
  which blocks `cargo test -p script-runtime-api` through its `webgl_wgpu`
  dev-dependency; the suite was run with that dev-dependency removed.
  `frame_arena_boundary` and `cross_arena_replacement` need a WebGL context and
  fail the same way without Phase 1.
