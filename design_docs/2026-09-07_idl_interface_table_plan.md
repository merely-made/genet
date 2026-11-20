# Generating the scripted tier's interface table from WPT's WebIDL

**Date:** 2026-09-07

**Status:** landed 2026-09-07. `components/script-runtime-api` (the DOM table
and its bootstrap glue) plus a new offline generator at
`support/idl-interface-table`. No renderer, layout or network source touched.

**Parent:** [`2026-09-06_web_platform_wpt_census.md`](2026-09-06_web_platform_wpt_census.md);
measured against the [`2026-09-07_wpt_harness_repair_plan.md`](2026-09-07_wpt_harness_repair_plan.md)
ledger, which is this lane's baseline.

**Receipts:** `Code/testing/genet/wpt-ledger/2026-09-07_idl_interface_table/`.

## Purpose

`components/script-runtime-api/dom/html_interfaces.rs` carried a hand-written
declarative table: 65 HTML interfaces, their parents, their tag names and 277
reflected IDL attributes, all typed out by hand from the HTML standard. Every
row was a transcription that could be wrong, could drift as the standard moved,
and could only grow by more transcription. WPT already ships the machine
extraction of the same standard — `tests/wpt/tests/interfaces/*.idl`, produced
by Reffy from the spec — and it is vendored in this checkout. This lane makes
the table a build product of that vendored IDL, keeps the bootstrap's algorithm
glue where it is, and widens coverage as far as the generated data allows
without implementing new algorithms.

## Findings

Dated 2026-09-07, verified in this checkout at `7f0a3c80d29`.

### F1 — WebIDL alone cannot produce the table; two inputs are needed

WebIDL carries no tag names. `interface HTMLAnchorElement : HTMLElement` does
not say that `<a>` selects it. WPT ships the missing half as data too:
`html/semantics/interfaces.js` is a 151-row `[tag, interface]` array, and it is
the very file `html/semantics/interfaces.html` tests against. The generator
reads both, so the tag map is vendored rather than transcribed, and the file
that asserts the mapping and the file that produces it are the same data.

Three of that file's rows are deliberately not tag mappings: `""` means
`HTMLElement`, `"Unknown"` means `HTMLUnknownElement`, and the non-lowercase
rows (`foo-BAR`, `å-bar`) exercise the unknown-name fallback rather than a tag.

### F2 — the IDL marks reflection explicitly, and richly

The HTML IDL carries `[Reflect]` (254 uses), `[ReflectURL]` (19),
`[ReflectSetter]` (27), `[ReflectDefault=]` (10), `[ReflectRange=]` (4),
`[ReflectNonNegative]`, `[ReflectPositive]` and `[ReflectPositiveWithFallback]`,
plus `Reflect="content-attribute-name"` for the 13 cases where the IDL name and
the content attribute name differ (`relList` → `rel`, `httpEquiv` →
`http-equiv`, `acceptCharset` → `accept-charset`, and kin). Together with the
IDL type that is enough to derive the kind, the content-attribute name, the
missing-value default and the read-only flag mechanically.

`[ReflectSetter]` marks an attribute whose *setter* is the reflection setter
while the getter is prose. For the `USVString` cases (`href` on `<base>`, and
on `<a>`/`<area>` through the `HTMLHyperlinkElementUtils` mixin) that prose is
"resolve against the base URL", which is exactly what the runtime's `u` kind
already does, so those are generated. `[ReflectSetter]` on other types
(`tabIndex`, `autocomplete`, `<img>`'s rendered `width`/`height`) is not.

### F3 — what the IDL does not carry

HTML's **enumerated** attributes are plain `DOMString` in WebIDL with no
`[Reflect]` at all: the keyword set, the missing-value default and the
invalid-value default live in prose. `dir`, `crossOrigin`, `referrerPolicy`,
`decoding`, `loading`, `kind`, `scope`, `autocomplete`, `enctype`, `method`,
`formEnctype` and `<button>`'s `type` are all in this class. So are a handful
of prose-defined getters the runtime approximates by reflection (`text` on
`<title>`/`<a>`/`<option>`/`<script>`, `value` on `<input>`/`<textarea>`/
`<option>`, `<canvas>`'s `width`/`height` defaults). These are the override
list, and they are the reason it is 41 rows rather than zero.

### F4 — mixins carry real reflected attributes

`HTMLElement includes HTMLOrSVGElement` is where `autofocus` lives;
`HTMLAnchorElement includes HTMLHyperlinkElementUtils` is where `href` lives.
A generator that ignores `includes` statements silently loses them, so the
parser expands mixins into every including interface before classification.

### F5 — a getter where there was no property changes assignment semantics

`<script blocking="render">` and kin: `[SameObject, PutForwards=value,
Reflect] readonly attribute DOMTokenList blocking`. Installing that as a
getter-only accessor made `element.blocking = 'render'` throw
`TypeError: cannot set non-writable property`, where previously — with no
property at all — the assignment created an own data property and the test
passed. This turned two `html/dom/render-blocking` files into
`evaluation-threw` errors in the first candidate run. `[PutForwards=value]` is
precisely the fix: assigning to the IDL attribute forwards to the token list's
`value`, i.e. writes the content attribute. Every read-only reflected attribute
the generator produces (10 of them) is a `PutForwards` `DOMTokenList`, so the
`t` kind gained that setter and the regression closed.

Recorded because it generalises: widening an interface's shape is not
automatically safe. A property the page could previously create by assignment
becomes a guarded accessor the moment the table names it.

### F6 — the unknown-element fallback was HTMLElement

`wrapNode` fell back to `HTMLElement.prototype` for any HTML-namespaced tag the
table did not list. Per the standard the fallback is `HTMLUnknownElement`,
except for valid custom element names. Since `HTMLUnknownElement` inherits from
`HTMLElement`, correcting this changes only `instanceof` and the class string,
never a method lookup.

The runtime's `__tagName` uppercases, so a case-preserving
`createElementNS(…, 'foo-BAR')` cannot be told from `foo-bar` at this seam.
Validity is therefore checked on the folded name, which keeps custom elements
working and leaves three `interfaces.html` subtests failing (`foo-BAR: useNS`,
`å-bar: useNS`, `å-BAR: createElement`). That is a `genet-scripted-dom` name
question, not a table question, and it is not opened here.

## The generator

`support/idl-interface-table` is a dependency-free workspace member with a
library and a binary. Pass `run -p genet-idl --bin genet-idl-interface-table`
to the workspace build launcher to write
`components/script-runtime-api/dom/html_interfaces_generated.rs`. Add
`-- --check` to verify the table.

**Parser.** `src/idl.rs` is a hand-written reader for the WebIDL subset the
table needs — about 495 lines including the tokenizer. It handles `interface`,
`partial interface`, `interface mixin`, `X includes Y;`, extended-attribute
lists, and `attribute` members with their types; every other definition and
member is skipped to its terminating `;` by a brace-aware scan. A crate
dependency (weedle2 or similar) was rejected because this repository's
dependency resolution is deliberately conservative — see the census plan's
notes and `Code/CLAUDE.md` — and because the subset is small enough that a
parser is cheaper than a new registry edge. Partials merge onto the base
interface as they are read; mixins are expanded into their including interfaces
in a second pass, so an attribute reaches the classifier exactly once,
in declaration order, base first.

**Classifier.** One function maps an IDL attribute to a reflected attribute or
to nothing: `[ReflectURL]` → the `u` kind; `[Reflect]` by IDL type
(`DOMString`/`USVString` → `s`, `boolean` → `b`, `long` → `l`,
`unsigned long` → `ul`, `DOMTokenList` → `t`); `[ReflectSetter]` on `USVString`
→ `u`; everything else, including nullable and union types, produces nothing.
The content-attribute name is `Reflect="…"` when given and the ASCII-lowercased
IDL name otherwise. `[ReflectDefault=]` supplies the missing-value default, and
the numeric kinds always emit one explicitly so the bootstrap's implicit
fallback can never apply.

**Overrides.** `OVERRIDES` in `src/lib.rs` is a flat table of
`(interface, IDL name, operation, reason)`. An entry replaces the generated row
or inserts one. The initial table used four reasons: `enum` (27 rows — the
keyword set is prose, per F3), `prose`
(12 rows — the getter is prose but the runtime's approximation is a reflection),
`union` (1 row — `hidden`'s `(boolean or unrestricted double or DOMString)?`,
of which the runtime models the boolean form), and `default` (1 row —
`<input>`'s `size`, `[Reflect]` with no `[ReflectDefault]` where HTML's prose
still gives 20). Nothing else is hand-written: the generated file carries no
hand edits, and the drift test proves it.

**Emission and drift.** The output is a plain Rust data file with a
`@generated` header, deterministic for fixed inputs. Both consts carry
`#[rustfmt::skip]` so that `cargo fmt` cannot rewrite the long tag arrays out
from under the byte-comparison. `dom/tests.rs::generated_table_is_current`
regenerates in-process from the vendored WPT tree and asserts byte equality,
plus floors on the three table sizes, so a stale checked-in file fails the
crate's own test run rather than drifting silently.

**Ordering.** Interfaces are emitted in IDL declaration order, then stably
re-ordered so a parent always precedes its children — the bootstrap builds
prototype chains in table order.

## Before and after

| | before (hand table) | after (generated) |
|---|---|---|
| HTML element interfaces | 65 | 72 |
| Reflected IDL attributes | 277 | 338 |
| Tag names mapped | 74 | 148 |
| DOM/CSSOM shape-only interfaces | 0 | 41 |
| Hand-written rows to maintain | 342 | 41 overrides |
| `html_interfaces.rs` | 1,004 lines of data + glue | 166 lines of glue |

New interfaces: `HTMLUnknownElement`, `HTMLPictureElement`,
`HTMLDirectoryElement`, `HTMLFontElement`, `HTMLFrameElement`,
`HTMLFrameSetElement`, `HTMLSelectedContentElement`.

The 41 shape-only records come from `dom.idl` and `cssom.idl`: the interface
object, its prototype chain and its class string, with no members, and only
where the bootstrap has not already defined the name. Where it has — `Node`,
`Element`, `Document`, `Text`, `Comment`, `DocumentFragment`, `DOMTokenList`,
the `CSS*` family — the pass only adds the class string and leaves the working
implementation alone. Five names are on an explicit deny list
(`MutationObserver`, `MutationRecord`, `AbortController`, `AbortSignal`,
`XSLTProcessor`) because a page that feature-detects them and finds a bare
shape takes a worse path than one that finds nothing.

Three `[LegacyFactoryFunction]` names now exist — `Image`, `Audio`, `Option` —
built from the table's own tag name with the documented argument mapping.

## Phases

### I1 — the generator and the checked-in table

**Done-conditions**

- The `genet-idl-interface-table` binary writes the table from the vendored
  WPT inputs with no network access and no dependencies. **Met.**
- Output is byte-stable across runs and across `cargo fmt`. **Met**, proved by
  `-- --check` after a formatting pass.
- Drift fails loudly: `generated_table_is_current` regenerates and compares.
  **Met.**

### I2 — the runtime consumes the generated table

**Done-conditions**

- `html_interfaces.rs` holds record shapes and JS serialization only; no
  interface data. **Met** (1,004 → 166 lines).
- The bootstrap's algorithm glue is unchanged in kind: `installHtmlInterfaceTable`,
  `installReflectedAttributes` and `makeHtmlInterfaceConstructor` keep their
  shape and gain only what the new columns require — `[Exposed]` filtering,
  `[HTMLConstructor]` (an interface without it always throws), the class
  string, read-only handling, and the named-constructor factories. **Met.**
- `cargo test -p script-runtime-api` green. **Met**: 125 + 16 + 7 + 13 tests,
  0 failed, including the pre-existing `html_interface_table_works` reflection
  test unchanged.

### I3 — widened coverage, measured

**Done-conditions**

- `html/dom` and the idlharness files hold or gain. **Met**: `html/dom` gains
  one file (25 → 26 all-pass) and +1,542 subtest passes; all six idlharness
  files under `html` and `dom` hold at their prior status.
- `dom` and `custom-elements` hold. **Met**: `dom` moves one file
  `error -> fail` and gains +108 subtests; `custom-elements` moves one file
  `error -> fail`, one `fail -> pass`, and gains +50 subtests.
- No unexplained pass-to-fail movement. **Met**: zero `pass -> fail`, zero
  `pass -> error` and zero `fail -> error` across all 16 directories.

## The census diff

Sixteen directories, disk mode, Boa/Livery, `--jobs 8 --timeout 90`, run twice
from the same runner build lineage: `pre/` at `7f0a3c80d29` unmodified, `post/`
with the lane landed. Every `pre/` tally is identical to the corresponding
2026-09-07 harness-repair map, which is the positive control that the two runs
differ only in the engine source.

| Directory | status movements | subtest passes |
|---|---|---|
| `html/dom` | 1 `fail -> pass` | +1,542 |
| `html/semantics/interfaces.html` | — | +298 |
| `dom` | 1 `error -> fail` | +108 |
| `custom-elements` | 1 `error -> fail`, 1 `fail -> pass` | +50 |
| `html/semantics/embedded-content` | 3 `error -> fail`, 3 `fail -> pass` | +23 |
| `html/semantics/forms` | — | +25 |
| `shadow-dom` | — | +3 |
| nine others | — | 0 |
| **total** | **5 `error -> fail`, 5 `fail -> pass`** | **+2,049** |

Named movements:

- `html/dom/elements/name-content-attribute-and-property.html` `fail -> pass`:
  the obsolete-members partial in `html.idl` supplies `name` on interfaces the
  hand table omitted it from.
- `custom-elements/parser/parser-constructs-custom-elements.html` `fail -> pass`
  and `custom-elements/reactions/CSSStyleDeclaration.html` `error -> fail`: the
  `HTMLUnknownElement` fallback and the widened constructor set.
- `dom/events/Body-FrameSet-Event-Handlers.html` `error -> fail`:
  `HTMLFrameSetElement` now exists, so the file evaluates instead of throwing.
- `html/semantics/embedded-content`: two `the-iframe-element` parentage files
  and `delay-load-event-detached.html` move `error -> fail`;
  `resource-selection-invoke-audio-constructor-no-src.html` and two `sandbox_*`
  files move `fail -> pass` on the `Audio` factory function and the `sandbox`
  token list.
- `html/dom/render-blocking/{remove-attr,remove-element}-unblocks-rendering.optional.html`:
  regressed to `evaluation-threw` in the first candidate and are green in the
  landed run — see F5. They are the only pass-to-fail movement this lane
  produced, and it was fixed rather than explained away.

`html/semantics/interfaces.html` moves 0/438 → 298/438 subtests. It stays
`fail` at the file level. Of the 140 residual subtests, 137 are the `useParser`
variant, which needs `DOMParser` — a missing global, not a table gap — and
three are the case-preservation limit recorded in F6.

## Known gaps carried forward

- `input.type` still uses Unicode lowercasing for its keyword comparison.
  This makes `checKbox` match `checkbox` under reflection.
- The three case-sensitivity subtests in F6.
- The generator reads `html.idl`, `dom.idl` and `cssom.idl`. `SVG.idl`,
  `uievents.idl` and the rest of the 338 vendored IDL files are untouched;
  extending the shape pass to them is the obvious next slice and is not
  scheduled.

## `[ReflectRange]` follow-through (2026-11-17)

The generator passes the IDL bounds through the Rust table to the JavaScript
property descriptor. `headingOffset` uses 0–8, `span` and `colSpan` use 1–1000,
and `rowSpan` uses 0–65534. The getter clamps valid nonnegative integers and
uses the reflected default or lower bound for missing or invalid text. The
setter stores the value produced by the ordinary unsigned-long reflection
rules. [HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

The focused runtime control passed on Bole and Vano after failing on both in
the pre-fix run. It covers the bounds, defaults, retained attribute text,
unsigned conversion, and integers larger than the engine's integer parser
could handle. The checked-in table passes the generator's `--check`.

The `reflection-tabular.html` WPT passed 6100/6116 assertions on Bole and
6110/6116 on Vano, up from 6022/6116 and 6032/6116 on the prior frozen runner.
Each engine gained all 78 previously failing range assertions and preserved
every prior pass. The 16 remaining Bole failures and six remaining Vano
failures concern `tabIndex` negative zero and legacy string null handling.
The raw maps and assertion comparison are retained in
`Code/output/genet-reflect-range/`.

## Tabular reflection closeout (2026-11-18)

The generated string bindings preserve `[LegacyNullToEmptyString]` from
the IDL type. An annotated setter writes an empty content attribute for
JavaScript `null`. Ordinary string setters still stringify their input. The
signed integer getter normalizes a parsed `-0` to positive zero while the
content attribute retains its text. [Web IDL string conversion](https://webidl.spec.whatwg.org/#es-DOMString)

Four focused runtime tests passed across Bole and Vano, covering the prior
range cases, annotated and ordinary string conversion, and signed zero.
The generated interface table passed `--check`.

Both engines passed all 6116 assertions in `reflection-tabular.html`. Bole
gained 16 passes and Vano gained six over the range-only runner. Comparison
by assertion name and occurrence preserved every prior pass. The runner,
logs and comparison are retained in `Code/output/genet-tabular-reflection/`.

## Textarea positive reflection with fallback (2026-11-19)

The generated table installs `HTMLTextAreaElement.cols` and `.rows` with
defaults of 20 and 2. The getter accepts parsed values from 1 through
2147483647 and returns the default for missing, invalid and out-of-range
text. A property assignment converts to Web IDL `unsigned long` first, then
writes the default when the converted value is outside that interval. The
bounded integer parser also serves the existing `[ReflectRange]` getters.
[HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

Six focused runtime tests passed across Bole and Vano. They cover the new
textarea boundaries and the existing range, legacy string and signed-zero
cases. The generated-table drift test passed.

`reflection-forms.html` improved from 7016/8271 to 7156/8271 on each engine.
All 70 `cols` assertions and all 70 `rows` assertions moved from fail to pass.
Every other assertion retained its status, including the 1115 existing
failures. The frozen runner, raw maps and assertion comparison are retained
in `Code/output/genet-textarea-reflection/`.

## Nonnegative length reflection (2026-11-19)

The generated table now installs `maxLength` and `minLength` on `input` and
`textarea`. Reads accept parsed integers from 0 through 2147483647 and use
`-1` for missing, invalid, negative or oversized content. A setter converts to
Web IDL `long`, rejects a negative result with `IndexSizeError`, and writes
other values through the element's attribute mutation path. [HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

All eight focused reflection tests passed across Bole and Vano, including
the new length boundary cases and earlier reflection variants. The generated
interface table passed `--check`.

`reflection-forms.html` improved from 7156/8271 to 7444/8271 on each engine.
All 72 assertions for each of the four element and property pairs moved
from fail to pass. The other assertions retained their status, including the
827 existing failures. The frozen runner, raw maps and comparison are in
`Code/output/genet-nonnegative-reflection/`.

## Positive progress maximum reflection (2026-11-19)

The generated table installs `HTMLProgressElement.max` as a positive `double`
with a default of `1.0`. The getter parses the content attribute using HTML's
floating-point rules and falls back to `1.0` when the parsed value is missing,
invalid, or nonpositive. The setter first converts to Web IDL `double`, which
rejects nonfinite values with `TypeError`. It writes the numeric string for a
positive value and skips the attribute write for zero or a negative value.
[HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)
[Web IDL `double` conversion](https://webidl.spec.whatwg.org/#es-double)

The focused test failed before the binding was installed and passed on both
Bole and Vano after the fix. It exercises ASCII whitespace, numeric prefixes,
incomplete exponents, underflow, overflow, and finite double limits. It also
checks attribute preservation after ignored assignments and conversion errors.
The generated interface table passed `--check`.

`reflection-forms.html` improved from 7444/8271 to 7534/8271 assertions on
each engine. All 90 `progress.max` assertions moved from fail to pass, with
zero regressions and 737 failures remaining in the fixture. The frozen
runner, raw maps, and assertion comparison are retained in
`Code/output/genet-progress-reflection/`.

## Meter setter reflection and computed getters (2026-11-19)

The generated bindings install six `HTMLMeterElement` setters that apply Web
IDL `double` conversion and serialize the resulting finite number to the
corresponding content attribute.
The getters compute their values from the current attributes. `min` defaults
to 0 and `max` defaults to 1, with `max` raised to `min` when needed. `value`,
`low`, and `optimum` clamp to the effective `min` and `max`. `high` clamps to
the effective `low` and `max`. Their defaults are 0, `min`, the midpoint of
`min` and `max`, and `max`, respectively, before clamping.
[HTML meter element](https://html.spec.whatwg.org/multipage/form-elements.html#the-meter-element)
[Web IDL `double` conversion](https://webidl.spec.whatwg.org/#es-double)

The focused regression failed before the fix and passed afterward on Bole
and Vano. It covers defaults, recalculation after mutations, out-of-range
attribute preservation, rejected conversions, and finite and subnormal
midpoint cases. The generated table passed `--check`.

Both engines passed all 50 assertions in `meter.html` and all 12 in
`meter-leading-plus-sign.html`, up from 1/50 and 0/12, respectively.
`reflection-forms.html` improved from 7534/8271 to 7600/8271 on each engine.
All 11 assertions for each of the six meter properties moved from fail to
pass, with zero regressions and 671 failures remaining in the fixture.
The frozen runner, raw maps, and assertion comparison are retained in
`Code/output/genet-meter-reflection/`.

## Form method enumeration defaults (2026-11-19)

`form.method` returns `"get"` for missing or invalid content. The `formMethod`
getter on `input` and `button` returns `""` when its attribute is missing and
`"get"` when it is present and invalid, including when it is empty. All three
getters match `get`, `post`, and `dialog` using ASCII case-insensitive matching
and return the canonical lowercase keyword. Assignment reflects the Web IDL
`DOMString` value into the content attribute, preserving spelling and whitespace.
[HTML form submission attributes](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-attributes)
[HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

The focused regression failed before the change and passed afterward on Bole
and Vano. It covers missing, empty, and invalid content, attribute removal,
`dialog` on all three elements, mixed-case keywords, and DOMString coercion.
The generated table passed `--check`.

`reflection-forms.html` improved from 7600/8271 to 7787/8271 on each engine.
Every failing method assertion moved to pass, with 57 gains for `form.method`,
60 for `input.formMethod`, and 70 for `button.formMethod`. The fixture has
484 remaining failures and zero regressions. The frozen runner, raw maps,
and assertion comparison are retained in `Code/output/genet-method-reflection/`.

## Form encoding enumeration defaults (2026-11-19)

`form.enctype` and its `encoding` alias read and write the same `enctype`
content attribute. Both getters return `application/x-www-form-urlencoded`
when the attribute is missing or invalid. The `formEnctype` getter on `input`
and `button` returns `""` when `formenctype` is missing and
`application/x-www-form-urlencoded` when it is present and invalid, including
when it is empty. The getters match the three MIME keywords using ASCII
case-insensitive comparison. The setters store the converted DOMString with
its original case and whitespace. The generated table carries separate
missing and invalid defaults for the encoding and method bindings.
[HTML form submission attributes](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-attributes)
[HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

The focused encoding regression failed before the fix on both engines.
Afterward, both the encoding and method tests passed on Bole and Vano. They
cover missing, empty, and invalid content, bidirectional alias writes,
attribute removal, mixed-case keywords, and DOMString conversion. The
generated table passed `--check`.

`reflection-forms.html` improved from 7787/8271 to 8005/8271 on each engine.
All failing encoding assertions moved to pass. `form.enctype` and
`form.encoding` gained 55 each, and `input.formEnctype` and
`button.formEnctype` gained 54 each. The fixture has 266 remaining failures
and zero regressions. The frozen runner, raw maps, and assertion comparison
are retained in `Code/output/genet-encoding-reflection/`.

## Form action URL reflection (2026-11-19)

`form.action`, `input.formAction`, and `button.formAction` return the owner
document's URL when the corresponding attribute is missing or empty. A
nonempty value resolves against that document's effective base URL. A value
that fails URL parsing is returned as a scalar-value string. The setters
store the Web IDL `USVString` value without resolving it.
[HTML form submission attributes](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#form-submission-attributes)
[Web IDL `USVString` conversion](https://webidl.spec.whatwg.org/#es-USVString)

The focused test failed before the fix and passed afterward on Bole and Vano.
It covers missing and empty values with a live `<base>`, relative resolution
after base changes, adoption into auxiliary and frame documents, raw attribute
writes, invalid URL fallback, scalar conversion, and legacy query encoding.
The freshly built generator passed `--check`.

The unmodified `reflection-forms.html` improved from 8005/8271 to 8008/8271
on both engines, gaining the three missing-attribute defaults with zero
regressions. The fixture builds its expected URLs from anchor component
getters that Genet has yet to implement, which affects the 120 remaining
action assertions. An isolated copy restricted to the three action properties
used URL expectations precomputed with Node's WHATWG URL parser and kept all
132 named action assertions. It moved from 129/132 to 132/132 on both engines.
The frozen runner, raw maps, diagnostic fixture, and assertion comparison are
retained in `Code/output/genet-action-reflection/`.

## Hyperlink URL components (2026-11-19)

Anchors and areas now read `href`, `origin`, and the individual URL components
from the current `href` content attribute and its owner document's base and
query encoding. Setting a component serializes the adjusted URL back into
`href`. Setting `href` itself stores the converted Web IDL string, and the
element stringifier reads the same value as the `href` getter. [HTML hyperlink URL APIs](https://html.spec.whatwg.org/multipage/links.html#api-for-hyperlink-elements)

Ten focused runtime tests passed on Bole and Vano. They cover hyperlink
component mutations, empty query and fragment components, invalid URLs,
adoption, and the existing form-action and location URL behavior. The
generated table passed `--check`.

The five URL getter and setter fixtures under
`html/semantics/links/links-created-by-a-and-area-elements` improved from
10/139 to 139/139 assertions on both engines. The other eight fixtures in
that directory retain their prior statuses.

The unmodified `reflection-forms.html` improved from 8008/8271 to 8170/8271
on both engines. All 120 action assertions that depended on the anchor
getters now pass, as do 42 additional `input.src` assertions. The fixture has
101 remaining failures and zero regressions. The frozen runner, raw maps,
and assertion comparison are retained in `Code/output/genet-hyperlink-reflection/`.

## Form autocomplete enumeration defaults (2026-11-19)

The generated `HTMLFormElement.autocomplete` binding uses `on` for both its
missing and invalid defaults. The getter recognizes `on` and `off` with
ASCII case-insensitive matching and returns the canonical lowercase keyword.
Empty and whitespace-padded values use the invalid default. The setter
reflects the converted DOMString with its case and whitespace preserved.
[HTML form autocomplete attribute](https://html.spec.whatwg.org/multipage/forms.html#attr-form-autocomplete)

The focused regression failed before the fix and passed afterward on Bole
and Vano. It covers missing, empty, and invalid values, mixed-case keywords,
attribute removal, and DOMString conversion. The generated table passed
`--check`.

`reflection-forms.html` improved from 8170/8271 to 8219/8271 on each engine.
All 49 failing `form.autocomplete` assertions moved to pass. The fixture has
52 remaining failures and zero regressions. The frozen runner, raw maps, and
assertion comparison are retained in `Code/output/genet-autocomplete-reflection/`.

## Option label reflection (2026-11-19)

`option.label` returns the value of a present null-namespace `label` attribute,
including an explicit empty string. When the attribute is missing, the getter
uses the same HTML-aware text collection as `option.text`. The setter writes
the converted DOMString to the null-namespace attribute. The specification's
display label algorithm uses the option text for both missing and empty
labels. [HTML option element](https://html.spec.whatwg.org/multipage/form-elements.html#the-option-element)

The focused regression failed before the fix and passed afterward on Bole
and Vano. It covers text mutations, whitespace, explicit empty labels,
null-namespace writes alongside a namespaced label, and DOMString conversion.
The generated table passed `--check`.

On both engines, `option-label.html` improved from 0/12 to 12/12 and
`reflection-forms.html` improved from 8219/8271 to 8238/8271. All 19 gains
in the forms fixture concern `option.label`. The fixture has 33 remaining
failures and zero regressions. The frozen runner, raw maps, and assertion
comparison are retained in `Code/output/genet-option-label/`.

## Input size positive reflection (2026-11-19)

The generated record for `input.size` carries the positive unsigned-long
reflection rule and default of 20 from the HTML prose. The getter accepts
parsed integers from 1 through 2147483647 and returns 20 for missing, invalid,
zero, or out-of-range content attributes. The setter applies Web IDL
unsigned-long conversion before checking the range. A converted zero throws
`IndexSizeError` before changing the attribute. A converted value above
2147483647 writes 20. [HTML input size](https://html.spec.whatwg.org/multipage/input.html#attr-input-size)

The focused regression failed before the fix and passed afterward on Bole
and Vano. It checks the default, raw attribute parsing, getter and setter
bounds, zero assignments preserving the attribute, and Web IDL conversion
including wraparound and BigInt rejection. The generated table passed
`--check`.

On both engines, `reflection-forms.html` improved from 8238/8271 to
8247/8271. All nine gains concern `input.size`, with 24 remaining failures
and zero regressions. Vano completed with a 600-second timeout after its
initial candidate run reached the 300-second limit. The frozen runner, raw
maps, and assertion comparison are retained in `Code/output/genet-input-size/`.

## Select size reflection (2026-11-19)

The `select.size` getter accepts parsed non-negative integers from 0 through
2147483647. A missing, invalid, or out-of-range content attribute returns 0
for both single and multiple selects. The setter applies Web IDL unsigned-long
conversion first. Zero writes `"0"`, and a converted value above 2147483647
also writes `"0"`. [HTML select element](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)
[HTML reflection rules](https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes)

The generated record uses the vendored IDL default and the shared bounded
unsigned-long reflection path. The focused regression failed before the fix
and passed afterward on Bole and Vano. It covers both missing defaults, raw
attribute parsing and preservation, zero writes, the range boundaries, and
Web IDL conversion including wraparound and BigInt rejection. The generated
table passed `--check`.

On both engines, `reflection-forms.html` improved from 8247/8271 to
8253/8271. All six gains concern `select.size`, with 18 remaining failures
and zero regressions. The frozen runner, raw maps, and assertion comparison
are retained in `Code/output/genet-select-size/`.

## Progress

- **2026-09-07** — Generator written (`support/idl-interface-table`, 1,323
  lines, no dependencies), table generated, `html_interfaces.rs` reduced to
  glue, bootstrap extended for `[Exposed]`, `[HTMLConstructor]`, class strings,
  read-only/`PutForwards` and named constructors, `wrapNode` corrected to fall
  back to `HTMLUnknownElement`. Drift test and shape test added.
- **2026-09-07** — First candidate run found the `PutForwards` regression (F5);
  fixed, re-run, and the landed run has zero pass-to-fail movement.
- **2026-09-07** — Gates: `cargo test -p script-runtime-api` 161 tests green
  across four targets; `cargo test -p genet-wpt --features netfetch` 63 green,
  3 ignored; clippy clean in every touched file; rustfmt applied. Release
  runner SHA-256
  `20130f96b9495762a0eb66afbc7ad88a078fab20fe3ba87d1db56dbcbf6a1cb2`, genet
  `7f0a3c80d29`, manifest SHA-256 prefix `d5ec5be9bf1a75ed`. Maps under
  `Code/testing/genet/wpt-ledger/2026-09-07_idl_interface_table/`.
