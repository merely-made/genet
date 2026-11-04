# Dependency currency: five crates.io moves

**Date:** 2026-10-21

**Status:** landed, 2026-10-22. The `dep-currency` branch (cut at
`cca45fc7a5c`, tip `0f8a9440225`, 2026-10-09) merged with main
`d00361b7c6f`, gated, and merged into main. Mark ruled on 2026-10-18 "Merge
after a full gate", and on 2026-10-21 "Move murm-wire to 0.30 first". Every
done-condition below is met. The receipt is under Progress.

**Owns:** the workspace pins of the five moves below, the source changes
they forced, and carrying those changes through code main added after the
branch was cut.

**Does not own:** murm-wire's own manifest, which lives in murm (retinue
repo); `examples/netrender_smoke`, a standalone workspace left on pollster
0.4; `windows-core`, held at 0.62 by the genet-fontique fork's DirectWrite
bindings (the Vello lane's).

## Moves

| Crate | Old | New | Forced source change |
|---|---|---|---|
| sha2 (digest 0.10 line retired) | 0.10 | 0.11 | digests lost `LowerHex`; hex written byte by byte |
| cssparser | 0.37 | 0.38 | `ParserInput` gone: `Parser::new(&str)`, one lifetime, unit-valued errors |
| html5ever / markup5ever / xml5ever | 0.39 | 0.40 | `LocalName`/`Namespace` `.as_ref()` ambiguous (E0283/E0282); `.as_str()` |
| accesskit (+ windows / macos / unix adapters) | 0.24 (0.32 / 0.26 / 0.21) | 0.25.1 (0.35.1 / 0.27.1 / 0.24.0) | none beyond main's own AccessKit 0.25 alignment |
| pollster | 0.4 | 1.0 | none (`FutureExt` widened to `IntoFuture`) |
| tokio-tungstenite / tungstenite | 0.29 | 0.30 | none in genet; murm-wire moved first |

Per-move detail and the original per-commit gates are in the five branch
commits (`b30dd56d5fb`, `8785e23f48c`, `6755d0bf5cf`, `6e3cbc87d19`,
`0f8a9440225`).

## Done when

- `git merge main` resolved with main's behaviour kept in every hunk and the
  branch's API moves applied to it.
- `cargo check --workspace --all-targets` clean except reds proven to be
  main's.
- The touched crates' tests green, every red reproduced on main.
- main's TLS advisory floors (rustls 0.23.46, webpki-roots 1.0.10) and its
  dropped `rustls-tls-webpki-roots` feature on netfetcher's tokio-tungstenite
  are preserved.

## Findings

**2026-10-21. Nine conflicted files, all resolved on main's logic.**

| File | Hunks | Resolution |
|---|---|---|
| `Cargo.toml` | 4 | main's patch-exact accesskit pins; html5ever 0.40 plus main's new hkdf/hmac 0.13; main's `sha1 = "0.11"` (main already carried `sha2 = "0.11"`) |
| `components/netfetcher/Cargo.toml` | 1 | branch's 0.30, without the `rustls-tls-webpki-roots` feature main dropped in `99b7c4f4a7b` (RUSTSEC-2026-0381) |
| `ports/genet-wpt/Cargo.toml` | 1 | main's optional `webgl-tests` feature shape, pollster moved to 1.0 inside it |
| `ports/genet-wpt/src/conformance.rs` | 1 | both sides wrote the same byte-wise hex; branch's comment kept |
| `components/genet-scripted-dom/forms.rs` | 2 | main's select/option state, renamed |
| `components/genet-scripted-dom/lib.rs` | 1 | main's form interaction, custom validity and option state, renamed |
| `components/script-runtime-api/dom/xpath_eval.rs` | 1 | main's `html_document` check, renamed |
| `components/script-runtime-api/selector.rs` | 2 | main's validity pseudo-classes and its child-walk test, renamed |
| `components/taproot/lib.rs` | 1 | main's projection-only role matching |

Beyond the hunks: `components/fleece/src/lib.rs` auto-merged with two
`sha256_hex` definitions, identical in output, and the branch's copy was
removed. `components/genet-compositor`, new on main, pinned pollster 0.4 and
moved to 1.0 so the graph keeps one pollster.

**2026-10-21. The rename carried onto main's newer code: 66 call sites in 16
files**, every one named by the compiler (E0283 or E0282 on an atom's
`.as_ref()`), plus two cssparser 0.38 `ParserInput` sites in
`components/livery/src/{cascade,css_comments}.rs`. Against main the whole
html5ever rename now spans 297 sites in 54 files.

**2026-10-21. murm-wire moved first.** Main had routed netfetcher's
WebSocket through `murm_wire::websocket::connect`, and murm-wire (`c646e92`)
pinned tokio-tungstenite 0.29 and re-exported its types. Netfetcher on 0.30
then failed with six E0308 errors in `websocket.rs`. Mark ruled "Move
murm-wire to 0.30 first": murm main `5b1c5cac2f5` moved it, keeping
`rustls-tls-native-roots` and its own webpki-roots 1.x connector, with 7 of 7
murm-wire tests green. Genet's lock then followed (`cargo update -p
murm-wire`), with no manifest change because netfetcher names murm by
`branch = "main"`. The graph now holds one tungstenite (0.30), and the
digest 0.10 line (digest, sha1 0.10, block-buffer 0.10, crypto-common 0.1,
generic-array) left it.

**2026-10-21. Main's advisory floors do not bind a stale lock.** No genet
member takes `rustls` or `webpki-roots` from the workspace table, so the
root floors never force a re-resolve. Both this branch's lock and main's
local lock still held rustls 0.23.45 and webpki-roots 1.0.9. This
worktree's lock was moved by `cargo update -p rustls -p webpki-roots -p
rustls-webpki -p webpki-root-certs` to 0.23.46, 1.0.10, 0.103.16 and 1.0.10.
`Cargo.lock` is untracked, so every other checkout needs the same update.

**2026-10-21. Reds on main, not this merge** (main `d00361b7c6f` in a
detached worktree with main's own lock):

- `genet-livery` example `capture_paint_corpus` (E0308 at line 189) and test
  `host_ch_lengths` (E0790 at line 76) do not compile. `20d78e88bff` made
  `ImagePixels::data` an `Arc<Vec<u8>>`, and `layout_with_text_system` now
  takes `&dyn ImageSourceLookup`, so `&Default::default()` cannot infer. Both
  files are byte-identical to main.
- `script-runtime-api`: `cross_arena_replacement` `{boa,nova}::atomic_refusal`;
  `forms_submission` `phase_c_forms_submission_implicit_{default_button,one_blocker,key_modifiers}::{boa,nova}`;
  `frame_arena_boundary` `associated_subtree_boundary_on_{boa,nova}`. These
  10 fail identically on main.
- `ortet` `fetch::tests::phase_c_t5_ortet_wire_redirect_fragment_chain` and
  `..._redirect303_and307` failed once in the combined gate. They are flaky
  on main too: main failed `redirect_fragment_chain` on one of three full
  `-p ortet --lib` runs, while the merge passed three of three.

## Progress

**2026-10-22. Landed.** Gate in this worktree's own target directory.

- `cargo check --workspace --all-targets --keep-going`: clean except main's
  two genet-livery reds above.
- `cargo test --locked --no-fail-fast` on cadency, fleece, genet-compositor,
  genet-document-resources, genet-documents, genet-render, genet-scripted,
  genet-scripted-dom, genet-static-dom, livery, script-runtime-api,
  layout-dom-api, taproot, servo-webgl-wgpu, genet-wpt, genet-parley,
  netfetcher and ortet: 139 binaries, 2,514 passed, 12 failed, 7 ignored.
  All 12 failures are main's (see Findings).
- genet-livery lib and its 48 test targets other than `host_ch_lengths`
  (`--lib --test …`; examples excluded): 49 binaries, 726 passed, 0 failed,
  8 ignored. Doctests: none.
- In total: 3,240 passed, every red attributed to main.
