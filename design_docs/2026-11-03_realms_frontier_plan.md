# Realms, navigation and document lifetime frontier

**Status: active, adopted 2026-11-03.** Successor to the [historical original](archive_docs/2026-11-03/2026-09-08_realms_plan.md). Review baseline `725e6c4fa30dfb1e7ea39a383ddac912374e27e7`; approved owner carry is included. Historical counts and receipts retain their recorded qualification. Adoption does not implement remaining work or decide open design questions.

## Established contract

One agent hosts a realm per browsing context; context identity is a stable WindowProxy while navigation replaces the associated realm. Same-origin checks distinguish security origin, visible document URL and effective base; opaque and sandboxed origins retain their security boundaries. Ordinary, shadow and template adoption must preserve object identity and creation-realm prototypes across source-realm disposal, including collection after the final reference. Initial blank iframe load ordering, origin and ancestor-origin snapshots, inherited about-document URLs and retained base reads have bounded recorded fixes. `components/script-runtime-api/{frames.rs,frames.js,dom/adoption.rs}`, `components/genet-documents/src/browsing_context.rs`, and `tests/{frame_realms,realm_residuals,retained_base_uri}.rs` are entry points. Historical headed G5 and WPT receipts apply only to their recorded sources, engines and hosts.

### Revised realm ruling and base separation

Mark revised the earlier one-Runtime-per-context proposal: one Runtime per agent and one realm per browsing context, with same-origin frames holding direct references and real object identity, and cross-origin access restricted through WindowProxy with `SecurityError` for forbidden members. Preserve the original ruling and its engine-specific limits.

The first connected `base` selection, parent effective-base snapshots, and inherited about-document fallback are distinct from the visible document URL and security origin. Invalid, data and JavaScript base URLs use the fallback; changing a base does not change document URL or security origin. Verify the current code and the historical boundary cases before extending these contracts.

## Phase R1: synthetic and template metadata

Mark selected this slice for briefing on 2026-10-02 (line-breaking ruling 7). First reproduce `URL`, `documentURI`, and `baseURI` for `createHTMLDocument`, `createDocument`, document cloning and template-content owner documents, including adoption and retained reads after teardown. Do not assume constructor existence proves independent metadata: `dom/bootstrap.js` contains constructors but the old plan names metadata as unresolved. Done when both-engine focused controls cover all four creation paths and lifecycle cases, matched WPT movements are attributed, and base/identity regressions remain green.

## Phase R2: retained documents and inherited bases

Reproduce mutations through retained retired documents separately from already-landed reads, and history restoration of inherited effective bases independently of visible URL/security origin. Done when the selected lifecycle semantics preserve authority and isolation, same-origin/opaque-origin controls and borrowed getters remain correct, and both engines pass focused teardown, navigation and history controls with matched WPT accounting. Preserve borrowed Location entry-realm attribution and lazy-frame activation as separate lifecycle checks.

## Phase R3: remaining security and navigation surfaces

`document.domain` remains a named same-origin-domain/agent-cluster question. Inventory current behavior and decide the security model before changing it. Separately assess `pageswap`, Navigation API, `beforeunload` cancellation, `window.open`, form and link-click navigation, JavaScript/blob/data URL navigation and Location exotic-object behavior. Done for each selected slice means explicit owner and threat/lifecycle boundary, both-engine origin and sandbox refusal/positive controls, and a matched relevant WPT comparison. A completed slice does not close the other navigation surfaces.

## Phase R4: delegated gates and integration

Live canvas context adoption retains its explicit refusal until the canvas/WebGL owner moves producer and storage together and proves identity and pixels. Parser scheduling (`execution-timing/112`, external modules and stream async/defer) belongs in the parser frontier. Scripted accessibility action dispatch and stale-target revalidation belong to the session owner; G5 proves projection only. Capture replay belongs to the capture owner; identity translation is not complete replay. Server-mode inherited fetch counts, range/charset throughput, and glyph-digest instability remain named measurement/host gates: attach route, source, viewport and scale and retain raw controls; do not convert a timeout or digest variance into a semantic pass.

## Questions and qualification

Which retired-document mutations are authorized, and what is their lifetime/authority boundary? What ownership and policy govern `document.domain` and navigation surfaces? These are questions for a scoped future slice, not new rulings. Earlier return-hop wrapper loss, classic external scripts after navigation, discarded Location, retired resource release, initial blank load, ancestorOrigins, window.origin and about URL/base separation were subsequently fixed in the original; do not revive their earlier historical markers. Opaque-root timing has passing isolated guards and a separate aggregate variability qualification, not a fresh failure or fresh pass.

## Owner carry and additional base/security controls, 2026-11-03

The approved working-tree carry in the original assigns historical legacy-charset iframe navigation/parsing, XML navigable parsing and the XML `document.open()` exception to iframe/Realms and parser owners. R1/R3 and parser P2 must first identify the exact cases and current source behavior; do not assume the old gaps remain. The original's final base phase also calls out direct external host DOM mutation followed by journal drain before synchronization, absent `base.href` reflection, incomplete cross-origin History URL-rewrite and Worker constructor security checks, and server-substituted WPT resources. R2/R3 must inventory these separately, with mutation-journal, URL/security, worker and server-mode controls as applicable. These are historical boundaries requiring current verification, not newly reproduced failures.

## Adoption record

Mark approved all eight reviewed cuts on 2026-11-03: "Yep". The original is archived whole; the reviewed technical scope, historical qualifications and open decisions are retained.
