# Fetch and request-security conformance

**Status (2026-10-21): in progress. P0 and P1 landed on branch `fetch-security` (not
merged) with matched Vano and Boa accounting, zero lost passes; the Murm fix sits on
Murm branch `fetch-close-notify` (not merged). P2 (element loads) is next and needs no
further ruling.** Rulings are recorded unnumbered below until Mark numbers them.

Mark's brief, 2026-10-20 (ledger, "Conformance targets, 2026-10-20"): asked
which conformance targets get briefed next against the 2026-09-29 Vano census
(33,824 of 144,238 subtests passing), he answered **"Fetch and request security
cluster (Recommended),WebCrypto,Component model"**. This is the first of those
lanes. Scope: `fetch`, `referrer-policy`, `mixed-content`,
`content-security-policy`, `upgrade-insecure-requests`, on the qualified
netfetcher wire adoption
([plan](2026-10-14_netfetcher_wire_adoption_plan.md)) and the Streams
reader-backed Fetch seam ([plan](2026-10-06_whatwg_streams_plan.md)). Engines:
Boa and Vano (Nova), both gated; zero lost passes on either.

Evidence lives in `Code/testing/genet/fetch-security/` (frozen runner, maps,
logs). Census source:
`receipts/2026-09-29_vano_wpt_census/outcomes.json.gz`.

## Findings (2026-10-20)

### F1. The census measured these directories in disk mode, which cannot pass most of them

The census ran `disk_mode: true` (`provenance.json`). Every generated
`security-features` file needs a server-side Python handler to echo what the
server saw, so disk mode fails them by construction.

| Directory | Census files (all-pass / total) | Census subtests | Files needing a handler or https |
|---|---:|---:|---|
| `fetch` | 86 / 646 | 2,016 / 7,439 | 61 non-pass files name a `.py`/`.headers` handler; 176 are `.https.`; 133 are worker variants |
| `referrer-policy` | 1 / 1,390 | 2 / 8,448 | 1,378 of 1,389 non-pass files use `security-features` handlers |
| `mixed-content` | 0 / 388 | 2 / 2,281 | 387 of 388 are `.https.` |
| `content-security-policy` | 64 / 846 | 175 / 3,748 | 330 use handlers, 218 `.https.`, 259 observe `securitypolicyviolation` |
| `upgrade-insecure-requests` | 0 / 197 | 0 / 1,000 | 197 of 197 are `.https.` |

The frozen runner's server-mode "before" maps, same box, jobs 1, subtests passed
over total (Vano / Boa), against the census:

| Directory | Census (disk, Vano) | Server mode, Vano | Server mode, Boa |
|---|---:|---:|---:|
| `referrer-policy` | 2 / 8,448 | 389 / 8,456 | 391 / 8,466 |
| `mixed-content` | 2 / 2,281 | 545 / 2,281 | 545 / 2,281 |
| `upgrade-insecure-requests` | 0 / 1,000 | 154 / 1,000 | 154 / 1,000 |
| `fetch` | 2,016 / 7,439 | 2,292 / 5,436 | 2,405 / (see map) |
| `content-security-policy` | 175 / 3,748 | 1,359 / 3,643 | 1,369 / 3,655 |

Totals differ because a file that now runs reports its subtests and a file that
times out reports fewer; compare maps, not these sums, across modes.

So the census is a floor, not a gap. A same-mode sample says the same for
`fetch`: `fetch/api/basic` scores 175/709 in the census and **309/408** in server
mode on the frozen runner (log
`fetch-security/logs/sample_fetch_api_basic_nova.log`; 36 of its 69 files error, 32 of them
`.worker.` variants that stop on "Worker cancellation is unsupported", which
is the Workers lane's residual, not this lane's). Every claim of progress here
therefore compares server-mode maps from a frozen runner and a candidate
runner, never the census.

### F2. What the runner's server lacks

The server is not the gap. This box already has everything `wpt serve` needs:
the WPT hosts entries (`www1`, `www2`, the punycode IDN host), Python 3.11,
the certificate store under `tests/wpt/tests/tools/certs`, and `wpt serve`
answers on http 8000, https 8443/8444, ws, wss and h2 9000 (probed with `curl`
against `www1.web-platform.test` on both schemes, 2026-10-20).

What the **runner** lacks, with the evidence:

1. **No https document origin.** `ServerCtx` carries one plain-http origin
   (`ports/genet-wpt/src/net.rs`, `ServerCtx::origin`); `.https.` and `.h2.`
   files are loaded from `http://web-platform.test:8000`. A probe from that
   page reports `window.isSecureContext` as `undefined`. `PAGE_ORIGIN` is a
   process-wide `OnceLock` (`net.rs:167`), so cross-origin detection, the
   `Origin` header, `secure_context` for mixed-content and WebSocket all key on
   one origin for the whole run. Mixed-content (387/388 files), upgrade-insecure
   (197/197), 218 CSP files and 176 fetch files are `.https.`.
2. **No initial-document response headers.** `do_get_document` keeps the body,
   content type and final URL, and drops the rest (`net.rs:215`). Header-delivered
   policy (`*.http-rp` referrer-policy, mixed-content and UIR files, CSP
   `.headers` files) has no way in.
3. **No load or error events for script-created `img`, `script` or `link`
   elements.** Probes in server mode: `iframe` fires `load`; `img` (same-origin
   and cross-origin, http and https), `script` (same- and cross-origin), a
   stylesheet `link`, and an `img` that 404s fire **nothing** and the file hits
   `harness-timeout`. The runtime has `__dispatchFrameLoad` and a
   parser-script `__dispatchScriptLoadError` and no other element load path
   (`script-runtime-api/dom/bootstrap.js:5950-5958`). This is why the
   `img-tag`, `script-tag`, `script-tag-dynamic-import`, `a-tag`, `svg-a-tag`
   cases of the generated files time out (sample: `referrer-policy/gen/top.meta/always`,
   6 of 12 files `harness-timeout`).
4. `window.isSecureContext` is not defined.

### F3. A real transport defect, root-caused: https bodies without `Content-Length`

In server mode a `cors` `fetch()` or XHR to an https URL rejects with `TypeError:
Failed to read response body` / `Network Error`, while the same request over
http succeeds (sample: `referrer-policy/gen/top.meta/always/fetch.http.html` 6/12,
`xhr.http.html` 6/12; the six failures are exactly the `*-https` cases).
`wptserve` answers close-delimited HTTP/1.1 over TLS and closes the socket without
a TLS `close_notify` (`curl -v`: "server closed abruptly (missing close_notify)",
"no chunk, no close, no size"). A controlled probe with the same URL:
`?pipe=header(Content-Length,3)` over https reads fine; without it, the body read
fails. The strict check is in the wire engine (`murm-wire`'s rustls/hyper client at
the pinned `f9fe850`); netfetcher only receives the failed body stream. Browsers
and `curl` treat a close without `close_notify` on a close-delimited body as end
of body. This is a wire-adoption residual, found because the old `?wss` and plain
http paths never exercised it. The fix is in Murm, outside this repository.

### F4. Referrer policy: the engine is half there

`netfetcher::referrer` already computes the `Referer` per redirect hop for all
eight policies and honours a `Referrer-Policy` on redirect responses
(`referrer.rs`; `fetch/mod.rs`). The runtime's `fetch()` and XHR pass
`referrer` and `referrer_policy` strings from the JS `Request` through
`FetchRequest` (`ports/genet-wpt/src/net.rs:382-397`). Missing is the
**document's policy container**: the referrer policy delivered by `<meta
name=referrer>` and the `Referrer-Policy` header, inherited by `srcdoc`/`about:blank`/blob
children and workers, and the `referrerpolicy` attributes. Today `frames.rs`
tracks only the iframe attribute and `ancestor_origins` masking
(`frames.rs:994-1004`, `:216`). The runner's `ResourceRequest` for host loads
carries `referrer`/`referrer_policy` fields (`genet-host-api/lib.rs:139`) but no
production caller computes them from the document.

### F5. Mixed content and upgrade-insecure-requests

netfetcher has one blocking-split function (`fetch/mixed_content.rs`): in a
secure context an `http` target is upgraded for image/audio/video and blocked for
the rest, and HSTS upgrades first. It reads `request.origin` for "secure", so it
inherits F2.1. It has no notion of the document's CSP-delivered
`upgrade-insecure-requests` and `block-all-mixed-content`, of the `Upgrade-Insecure-Requests`
request header, or of the Mixed Content Level 2 split between "upgradable" and
"blockable". The fetch()/XHR rows are reachable with a two-directive
policy parse; the element rows are F2.3.

### F6. Content Security Policy does not exist

`grep` for `content-security-policy` or `securitypolicyviolation` over all
`.rs` and `.js` under `components/` and `ports/`: zero hits. The only hook is
`CspChecker::allows_connect` in netfetcher (`context.rs:190`), defaulting to
`AllowAllCsp`; WebSocket and fetch call it. The census's 175 passing CSP
subtests are incidental (absence-of-feature passes, per the `Node-baseURI` rule
in `DOC_README.md`). 3,748 subtests across 846 files need a parser, a
policy container, directive checks for every fetch destination, nonce/hash
matching, inline-script and `eval` checks, `securitypolicyviolation` events and
reporting. This is a lane of its own (decision D4).

### F7. Fetch Metadata and the fetch API

No `Sec-Fetch-*` header is produced anywhere (zero hits). `fetch/metadata/generated`
is 66 files and 1,613 subtests (census 0), plus 79 more in `fetch/metadata`.
The bulk of those are element loads (F2.3); `fetch()`, XHR, `navigate` and
worker cases are reachable earlier. The rest of the `fetch` directory in server
mode fails on small, separable behaviours seen in the `fetch/api/basic` sample:
`statusText` for HTTP/2 responses, `header-value-combining`, non-ASCII request
headers, URL parsing under a document charset, `keepalive`, conditional GET,
`Float16Array` bodies, plus the Workers residuals above.

### F8. Spec text

The brief says WHATWG/W3C spec text is vendored. This checkout holds no copy of
the Fetch, Referrer Policy, Mixed Content, CSP or UIR specifications outside WPT's
own tests and the `LICENSE_WHATWG_SPECS` notice (searched `docs/`, `design_docs/`,
`resources/`, `Code/src`). This plan reads behaviour from the WPT expectations and
the algorithms the existing netfetcher modules cite. Where a phase needs the
exact algorithm text, that phase's first step is to ask Mark where the text
lives (the family spec registry at `merelyllc.com/content/specs.toml` is named by
the ledger) rather than download or paste it.

### F9. P0 implementation facts (2026-10-20)

- `ServerCtx` now carries an https and an h2 origin derived from the server host
  (ports 8443 and 9000, overridable with `--server-https-base` and
  `--server-h2-base`); `origin_for` picks one from the dotted flags in the file
  name, as `wpt serve` does (`ports/genet-wpt/src/net.rs`, `serve_flag`). The page
  origin is an `RwLock` set per test before its document loads (it was a
  once-set `OnceLock`); `testharness-one` workers are one process per test, so
  the value never leaks between tests.
- `window.isSecureContext` is a global accessor beside `origin`
  (`script-runtime-api/platform.rs`): https/wss, or http/ws on a loopback host;
  an opaque origin is not. The ancestor condition for nested contexts is not
  applied (residual, named in the code comment).
- `.h2.` files stay on the plain origin by default (Mark's round-2 ruling): serving
  them from port 9000 lost `request-upload.h2.any.html` 8 passes to 1 because
  Murm's request body is buffered. `--server-h2-base` keeps the h2 origin opt-in. The
  matched capture for these five `fetch` files and one `xhr` file used the first
  candidate (`p0`), which routed `.h2.` to 9000; the final runner (`p0b`) matches the
  frozen runner on all six when run alone (8/12, 1/7, 0/5, 1/7).
- Initial-document response headers are **not** handed to the runtime in P0: the
  runtime has no consumer or API for them yet (`grep` for any header-fed document
  state: none), and a hand-over with no reader would be dead plumbing. They land
  in P1 with the policy container that reads `Referrer-Policy` and the CSP
  directives. This narrows the second half of the runner ruling to P1; the
  document fetch already has `response.headers` in hand at `do_get_document`.
- Murm: the `LenientEof` wrapper makes `murm-wire`'s own regression pass and fails
  on the unpatched source with `peer closed connection without sending TLS
  close_notify`. It adds `tower-service` (already in the lockfile at 0.3.3, pulled by
  `hyper-util`) as a direct dependency of `murm-wire`. Mark approved it; it is commit
  `a897a97e824` on Murm branch `fetch-close-notify`, based on the pinned
  `f9fe8507cf1`, in worktree `Code/worktrees/murm-close-notify`; `cargo test -p
  murm-wire --locked --offline` passes 6 of 6 with default features. Not pushed or
  merged. The candidate runner uses a local `--config patch` override that is not
  committed, so this branch builds against Murm `main` only after Mark merges it.
- Gates, all with the override and an own target directory: `genet-wpt`
  (`--features netfetch`) 94 + 1 + 1 tests pass, 2 existing ignores;
  `script-runtime-api` 1,134 pass and 10 fail across three targets
  (`cross_arena_replacement::atomic_refusal` x2,
  `forms_submission::phase_c_forms_submission_implicit_*` x6,
  `frame_arena_boundary::associated_subtree_boundary_on_*` x2). The same 10 fail
  with `platform.rs` restored to `HEAD` in the same target directory, so they
  predate this work (the first needs a minted WebGL context, which this box
  lacks); logs in `fetch-security/logs/`. The new `global_origin::secure_context`
  test passes on both engines.

## Phases

Ordered by subtests unlocked per unit of work, from the generated cases counted
in the vendored tests (referrer-policy 15,332 cases of which fetch+xhr 4,920 and
iframe/img/script 6,672; mixed-content 4,528 of which fetch+xhr 928; UIR 1,984 of which
fetch+xhr 528; CSP 2,480 generated). Every done-condition is in terms of
named directories and retained passes, measured in server mode on both engines by a
frozen runner built before the first edit and a candidate built from the lane
branch, rows compared one by one, every old pass retained, every movement
attributed.

**P0. Instrument.** (a) Capture the frozen server-mode "before" maps for the five
directories on both engines. (b) Runner: serve `.https.` and `.h2.` variants from
the https origin with a per-test page origin and `window.isSecureContext`; hand
the initial document's response headers to the runtime. (c) Murm: tolerate a
close without `close_notify` on a close-delimited body, then repin. Done when
the `fetch.http.html`/`xhr.http.html` `*-https` cases in
`referrer-policy/gen/top.meta/always` read their bodies, a probe page served over
https reports `isSecureContext` true, and the five directories' maps show zero
lost passes against the frozen runner on both engines.

**P1. Request policy for `fetch()` and XHR.** The document policy container in
the runtime: `<meta name=referrer>`, `Referrer-Policy` header, inheritance to
`srcdoc`/`about:blank`/blob children and workers, `referrerpolicy` attributes,
the `Upgrade-Insecure-Requests` request header, and the two CSP-delivered
mixed-content directives (`upgrade-insecure-requests`, `block-all-mixed-content`),
all handed to netfetcher as request policy; netfetcher's mixed-content split
brought to the Mixed Content Level 2 rules. Done when the `fetch`, `xhr` and
`websocket` rows of `referrer-policy/gen`, `mixed-content/gen` and
`upgrade-insecure-requests/gen` pass in the cases the specification defines, with
the iframe/worker rows reported separately, and the Boa/Vano maps move together.

**P2. Element subresource loads.** `load`/`error` events and requests for
script-created and parser-created `img`, `script` (classic and dynamic import),
`link rel=stylesheet`, `a`/`area`/`svg a` navigation, `iframe` documents, with the
document's request policy, destination and referrer. Done when
`referrer-policy/gen/*/img-tag`, `script-tag`, `iframe-tag`, `a-tag`,
`mixed-content/gen/*` and `upgrade-insecure-requests/gen/*` element rows pass
and no `harness-timeout` remains for a supported element, both engines. Touches
`script-runtime-api` DOM files that other lanes also edit; it lands behind the
P1 policy container.

**P3. Fetch Metadata.** `Sec-Fetch-Site`, `-Mode`, `-Dest`, `-User` on fetch,
XHR and the P2 loaders. Done when `fetch/metadata` and
`fetch/metadata/generated` rows for supported initiators pass.

**P4. Fetch API residuals.** The small behaviours of F7, one per finding with a
regression at the layer that owns it (netfetcher or the runtime binding). Done
when the named `fetch/api/*` files move and none loses a pass.

**P5. Content Security Policy** (own lane, D4). Parser and policy container,
then `connect-src`, `script-src` (nonce, hash, `strict-dynamic`, inline, `eval`),
`style-src`, `img-src`, `frame-ancestors`, `sandbox`, `securitypolicyviolation`,
reporting. Its own dated plan and implementing agent after Mark's ruling.

**Out of this lane**, named so nobody absorbs them silently: `SharedWorker`,
module workers and worker cancellation (the Workers lane's recorded residuals;
133 fetch files and 192 referrer-policy files are worker variants), WebCrypto and
the component model (the other two briefs), `cookies`.

## Decisions for Mark (card round 1)

D1 phase order and first landed phase; D2 runner https origin and initial
headers; D3 Murm `close_notify` tolerance; D4 CSP as its own lane (round 1). D5
home of the document policy container; D6 the Murm diff; D7 `.h2.` routing
(round 2).

P0 result against its done-conditions (server mode, matched maps, both engines):

- The `*-https` cases of `referrer-policy/gen/top.meta/always/fetch.http.html` and
  `xhr.http.html` read their bodies: 6/12 to 12/12 each on Vano, and the same across
  the generated tree (below).
- A probe page served over https reports `isSecureContext` true (unit tests
  `global_origin::secure_context_{boa,nova}` plus the 19 `.https.` mixed-content
  files that moved).
- Zero lost passes on either engine over 47,625 unchanged rows, 868 gains and 185
  non-pass changes, every row attributed; 75 raw lost rows on Vano settled by
  isolated reruns (see Progress).

### F10. P1 implementation facts (2026-10-21)

- The container is `script-runtime-api/policy.rs`. `read_request` (the one native entry
  of `fetch()` and synchronous XHR) asks it for the running document's policy, so no
  JavaScript changed. The referrer policy comes from the `Referrer-Policy` header, then
  every `<meta name=referrer>` in tree order, the last valid one winning; the two
  mixed-content directives come from `Content-Security-Policy` headers and `<meta
  http-equiv>` (a report-only header enforces nothing). The tree is read when a request
  starts, so a `<meta>` removed after it applied no longer applies (residual, in the
  module comment).
- Hand-over of the initial headers is `Runtime::set_document_response_headers`; the
  runner keeps the final response's headers in `LoadedHtmlDocument` and passes them
  through `ScriptSrcLoader::document_response_headers`. Header-delivered UIR works
  (`upgrade-insecure-requests/gen/top.http-rp` fetch and XHR pass).
- `block-all-mixed-content` is carried to netfetcher but changes no outcome: Mixed
  Content Level 2 already blocks or upgrades every `http` subresource of a secure
  context, and WPT expects optionally-blockable loads to be upgraded even under the
  directive. It is there for the CSP lane. UIR leaves a loopback host alone; the spec
  text that settles that case is not vendored (F8), so this is a stated assumption.
- netfetcher had no 4096-byte referrer cap (Fetch cuts a longer referrer URL to its
  origin). With a real policy in place 80 `referrer-policy/4K+1` cases lost passes that
  the old default policy had given by accident; the cap is now in `referrer.rs` with a
  test at the limit, and those cases pass.
- `ResourceRequest` gained two `bool` fields, `#[serde(default)]`. Every literal in this
  workspace is updated (genet-host-api, genet-documents, genet-scripted,
  script-runtime-api, ortet). **Mere and any other out-of-tree host that builds a
  `ResourceRequest` literal must add the two fields when it takes this change.**
- Gates (own target directory, local Murm override): netfetcher 98 pass; genet-host-api
  5; genet-wpt `--features netfetch` 95 + 1 + 1 (2 existing ignores); genet-scripted
  all pass; `script-runtime-api` 1,143 pass and the same 10 predating failures
  (`cross_arena_replacement`, `forms_submission::phase_c_forms_submission_implicit_*`,
  `frame_arena_boundary`); `check --tests` clean for genet-documents (all features)
  and ortet; default-features `check -p genet-wpt` clean. New tests:
  `tests/document_referrer_policy.rs` (meta spellings, header, precedence, request
  override, CSP directives, sync XHR, on both engines), `policy::tests`, netfetcher
  `upgrade_insecure_requests_upgrades_every_http_url_but_loopback` and
  `a_referrer_over_4096_bytes_is_cut_to_its_origin`, genet-wpt
  `the_initial_document_keeps_the_final_responses_headers`.

## Rulings (Mark's answers, 2026-10-20; unnumbered until Mark numbers them)

Asked the phase order, with the census-versus-server-mode evidence, Mark:
**"P0 instrument, P1 fetch/XHR policy first (Recommended)"**.

Asked whether the runner serves `.https.`/`.h2.` tests from the https origin with a
per-test page origin and passes initial-document headers to the runtime, Mark:
**"Yes, both changes (Recommended)"**.

Asked who fixes the missing-`close_notify` body failure in `murm-wire`, Mark:
**"Authorise a small Murm change (Recommended)"**. The option as offered reads:
treat a close without `close_notify` as a clean EOF only for HTTP/1 close-delimited
bodies, with a `murm-wire` test, then Genet repins; the exact diff is named and
Mark's OK taken before Murm is touched. This lane holds to that.

Asked whether CSP is its own lane, Mark: **"Own lane, own plan (Recommended)"**.
This lane implements only `upgrade-insecure-requests` and `block-all-mixed-content`
behind a small policy-container seam.

Round 2, same day. Asked where the document policy container lives (referrer policy,
the two mixed-content directives, and later CSP), Mark: **"Runtime owns it, host
feeds headers (Recommended)"**. The container is per environment settings object in
`script-runtime-api`, inherited by frames and workers, filled from `<meta>` by the
runtime and from the initial response headers by a new host-called Runtime setter;
netfetcher stays a policy-taking engine. P1 builds exactly this.

Asked about the exact Murm diff (a `LenientEof` wrapper, `tower-service` as a direct
dependency of `murm-wire`, a regression that fails on the unpatched source), Mark:
**"OK, apply to Murm and repin (Recommended)"**. Applied as one commit on branch
`fetch-close-notify` in a worktree of the Murm repository (`repos/retinue`'s remote
`merely-made/murm`), based on the pinned `f9fe8507cf1` that Genet resolves today. Not
pushed or merged; Mark merges as usual. Genet follows Murm by `branch = "main"` with
its lock ignored, so the repin is the supplier resolving the merged commit; no Genet
manifest changes.

Round 3, 2026-10-21. Asked how `upgrade-insecure-requests` and `block-all-mixed-content`
reach the host that sends a request, Mark: **"Two bool fields on ResourceRequest
(Recommended)"**. Implemented as `upgrade_insecure_requests` and `block_all_mixed_content`
on `genet_host_api::ResourceRequest` and `netfetcher::Request`.

Asked to confirm that `.h2.` tests stay on the plain origin (serving them from the h2
origin lost `request-upload.h2.any.html` 8 passes to 1, found by the matched
accounting, because Murm's request body is buffered), Mark: **"Keep .h2. on plain
origin for now (Recommended)"**. `--server-h2-base` stays as an opt-in. Follow-up for
Murm: stream request bodies so full-duplex h2 upload can run, then the h2 origin can
become the default.

## Progress

- **2026-10-20** Worktree `fetch-security` from `main` at `16b971188f3`, own
  target directory, main's `Cargo.lock` copied in (the lock is ignored; the fresh
  re-resolve is kept as `Cargo.lock.fresh-resolve`). Baseline runner built before any
  edit (dev profile, debug info off, `--features netfetch`) and frozen: SHA-256
  `073c9cae5ade1099c752283532e97901a46cd8295aee8caf3ff34d250b3a1b9c`.
  Server-mode "before" maps in progress for Vano; Boa to follow.
- **2026-10-20** P0 landed on branch `fetch-security`. Runner changes
  (`ports/genet-wpt`): `--server-https-base`, `--server-h2-base`, https origin chosen
  per test from the dotted file-name flag, per-test page origin; `window.isSecureContext`
  in `script-runtime-api/platform.rs`. Tests: `net.rs` `serve_flag`/`origin_for`
  (2 new), `global_origin::secure_context` on Boa and Vano. Gates (own target
  directory, local Murm override): `genet-wpt --features netfetch` 94 + 1 + 1 pass,
  2 existing ignores; `genet-wpt` default-features `check` clean for the touched
  code; `script-runtime-api` 1,134 pass, 10 fail in three targets that fail identically
  with `platform.rs` restored to `HEAD` (no WebGL context here, forms implicit
  submission, frame arena boundary: pre-existing, not touched); `murm-wire` 6 of 6.
- **2026-10-20** Matched accounting (`Code/testing/genet/fetch-security/`,
  `account_pre_post.py`, `accounting_post.json`): frozen runner `genet-wpt-pre`
  SHA-256 `073c9cae5ade1099c752283532e97901a46cd8295aee8caf3ff34d250b3a1b9c`
  (built from `16b971188f3` plus main's `Cargo.lock`, SHA-256 `2cb3d086e2a90bea68c2416e03b9669c14e4bf95b5ea9151b2e6449b8c6f267d`,
  before any edit); candidate `genet-wpt-p0b`
  `e1bef41b750adc27d4628070a6f2da959fb7fa1bb2eb5ac2d570e7ae649889ec` (same tree plus
  this change and the local Murm patch; lock after resolution
  `6b84c30a7030721f6ffd8c380a997f814c145c331a633028824fda44884ccfbb`). The first
  candidate `p0` (`0b99a586...`) differs only in routing `.h2.`; its Vano maps for
  directories without `.h2.` files stand for `p0b`, and the six `.h2.` files were
  rerun alone. Subtest passes, before to after:

| Directory | Vano | Boa | Files all-pass (Vano) |
|---|---:|---:|---:|
| `referrer-policy` | 389 to 720 | 391 to 739 | 34 to 73 |
| `mixed-content` | 545 to 558 | 545 to 558 | 0 to 6 |
| `upgrade-insecure-requests` | 154 to 154 | 154 to 154 | 31 to 31 |
| `fetch` | 2,292 to 2,259 (raw, see below) | 2,405 to 2,434 | 107 to 105 (raw) |
| `content-security-policy` | 1,359 to 1,359 | 1,369 to 1,369 | 52 to 52 |

  Gains: 376 + 390 referrer-policy subtest and file rows are `fetch`/`xhr` cases that
  read an https body (the Murm fix); 19 + 19 `mixed-content` rows are six
  `fetch.https.html` and `xhr.https.html` files (`top.http-rp` and `top.meta`, `opt-in`
  and `unset`) now passing from the https origin, plus 13 subtests; `fetch` gains 4 + 4
  `.https.` rows and moves 17 + 17 rows of the four
  `fetch/local-network-access/navigate.tentative.window.html` variants from harness
  error to subtest failure because `isSecureContext` is now defined (no pass involved). `upgrade-insecure-requests` and `content-security-policy` do
  not move: they need P1 and the CSP lane.

  Lost-pass settlement. The raw Vano capture holds 75 lost rows (`referrer-policy` 8,
  `fetch` 67) and the Boa capture none. Every file holding a lost or unexplained
  row was rerun alone on both frozen runners (`recheck_isolated.py`): the 8
  `referrer-policy` rows (three `4K` files) pass on the candidate and fail on the
  pre runner in isolation (the pre runner fails their https cases), so they were
  capture-time load artefacts; 46 + 10 + 2 + 2 of the `fetch` rows are in
  `redirect-referrer-override`, `redirect-mode`, `redirect-location` and
  `redirect-origin`, which end `error` (drive deadline) on both runners alone, so
  their pass counts follow wall-clock load (in an earlier, calmer isolation pass
  `redirect-mode` and `redirect-location` pass 61/61 and 55/55 on both runners); 7 are `request-upload.h2.any.html`,
  fixed by `p0b` as above. Three chains ran at once during capture, which is what
  moved those deadline-bound counts. Two Boa `request-keepalive-quota` rows are
  variance on the pre runner (three alternating repeats, `keepalive_quota_repeat.txt`).
  Nothing remains unattributed. **Limit:** `fetch` on Vano reads 2,292 to 2,259 raw;
  a rerun of this directory on a quiet machine, one chain at a time, is the clean
  figure and is worth taking before P1 measures against it.
- **2026-10-20** Open items carried: P1 (policy container, header hand-over, mixed-content
  split, Sec-Fetch seam) waits on this branch; Murm branch awaits Mark's merge, after
  which the local override is dropped and `fetch-security` is rebuilt against `main`;
  `isSecureContext` ancestor condition; Murm request-body streaming for full-duplex h2
  upload; worker variants (Workers lane) remain the largest `fetch` error source.
- **2026-10-21** Quiet-machine `fetch` figure on Vano, one runner at a time, nothing else
  running: frozen runner 2,357 of 5,459 subtests, candidate `p0b` 2,360 of 5,499 (files
  all-pass 111 to 111), one row lost that sits in `fetch/metadata/generated/element-iframe.sub.html`,
  which ends `harness-timeout` on both runners alone (twice each). So the loaded-machine
  2,292 to 2,259 was noise, and P0's `fetch` movement is **+3, no confirmed loss**.
  (Maps `quietpre_fetch_nova.json`, `quietpost_fetch_nova.json`.)
- **2026-10-21** P1 landed on branch `fetch-security`. Frozen candidate `genet-wpt-p1b`
  SHA-256 `cb79fe1b7d39bb2926d37628810d444f3edaee87418ec53a4d3e0678ec2cd908`
  (the first P1 build `p1`, `37a1f39a...`, lacks the 4096-byte cap and was used only for
  directories it cannot affect: `4K+1` exists only in `referrer-policy`). Before-maps are
  the P0 after-maps (the quiet pair for `fetch` on Vano). Subtest passes, before to after:

| Directory | Vano | Boa | Files all-pass (Vano / Boa) |
|---|---:|---:|---:|
| `referrer-policy` | 720 to 908 | 739 to 908 | 73 to 119 / 76 to 119 |
| `upgrade-insecure-requests` | 154 to 178 | 154 to 178 | 31 to 35 / 31 to 35 |
| `mixed-content` | 558 to 558 | 558 to 558 | 6 to 6 |
| `fetch` | 2,360 to 2,352 (raw) | 2,434 to 2,437 | not tabulated |
| `content-security-policy` | 1,359 to 1,359 | 1,369 to 1,369 | 52 to 52 / 60 to 60 |

  Over 48,201 unchanged rows, 507 gained and 51 non-pass changes, **zero lost passes on
  either engine and nothing unattributed**. Gains are the `fetch` and `xhr` rows under
  `referrer-policy/gen` and `generic` (document policy), the `fetch.https.html` and
  `xhr.https.html` rows of `upgrade-insecure-requests/gen/top.meta/upgrade` and
  `top.http-rp/upgrade` (24 subtests, four files per engine), and
  `fetch/api/policies/referrer-origin.html`. Boa and Vano now report the same 908 of
  8,466 for `referrer-policy`. Eleven raw Vano `fetch` rows were lost in capture and
  settled by isolated reruns: nine in `request-forbidden-headers.any.html` and two in
  `redirect-referrer-override.any.html`, both of which end in the 15 s drive deadline on
  both runners alone (`redirect-referrer-override` flipped between 128/128 and the
  deadline on each runner, `redirect_referrer_override_repeat.txt`); one Boa `request-keepalive-quota` row is the same kind of
  variance as in P0. The `fetch` rows after P1 were captured with two runner chains and a
  build active, so they are not a quiet-machine figure; a quiet rerun of
  `fetch` for both runners is the clean comparison.
- **2026-10-21** Open items carried: P2 element loads (`img`, `script`, `link`,
  `iframe`, `a` load and error events with the policy container, the next-largest block:
  the `img-tag`, `script-tag`, `iframe-tag`, `a-tag` files still end `harness-timeout`);
  workers and nested contexts inherit nothing yet (`srcdoc`/`about:blank` children and
  `Worker`/`SharedWorker` start with the default policy); WebSocket has the mixed-content
  check but not UIR; Mere's `ResourceRequest` literals.
