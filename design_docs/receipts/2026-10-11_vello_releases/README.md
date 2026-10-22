# Genet Vello release adoption — 2026-10-11

Genet's render host, portable composition driver, and standalone embedder smoke
compile against the maintained Classic 0.11 release source. This receipt covers
consumer dependency identity and compilation; no consumer GPU runtime, Wasm
compilation, browser qualification, or native presentation was run.

## Source identity

- Genet base: `0583da30d6e9e36481a0fbe82ac3b5899377681c`.
- Qualified Genet source commit: `cec760f4ae6ce336823bc2be250d9ab4e35a8a47`.
- Netrender family: `46606610cd387e2e6956d8a3ebbcf442eab210d9` for
  `netrender`, `netrender_device`, `paint_list_api`, and `paint_list_render`.
- Maintained Vello provider: `5862bba26c8a9f56f6a5fdc8ea6966b360a5d63a` for
  `netrender-vello`, `vello_encoding`, and `vello_shaders`, all version 0.11.0.
- Resolved Wgpu: 30.0.1. The unused workspace `vello_cpu` declaration now names
  the same maintained provider's 0.3.0 package; no Genet member consumes it.

[identities-and-hashes.json](identities-and-hashes.json) records the scoped
resolved package identities, source/lock hashes, and SHA-256 values for the
complete metadata input and retained check logs. The full metadata JSON remains
in the task's `output/vello-release-adoption-20261011` directory; its 4 MiB
package listing is not duplicated here.

## Validation

| Gate | Result | Evidence |
|---|---|---|
| Root Cargo metadata | Passed after upstream synchronization | Complete-input hash and selected identities in the JSON ledger |
| `cargo check --locked -p genet-render-host -j2` | Passed | [Render-host log](logs/genet-render-host-check.log) |
| `cargo check --locked -p genet-render --no-default-features --features livery,accesskit -j2` | Passed | [Composition log](logs/genet-composition-check.log) |
| Standalone `examples/netrender_smoke` locked check | Compilation passed; runtime not run | [Standalone log](logs/genet-standalone-check.log) |

Checks were serialized with the environment recorded in the JSON ledger.
The standalone smoke now uses `Scene::new`, an opaque red viewport rectangle,
and `Renderer::render_vello`, with an explicit tile cache and Vello enabled.
Its Wgpu validation scope and five-second completion wait are runtime behavior
that this compilation receipt does not claim to have exercised.

## Text and browser boundaries

Genet retains `genet-parley`/`genet-fontique` 0.10. Its text system emits neutral
`FontResource` bytes and glyph instances; `paint_list_render` converts those to
Netrender's scene. No Genet consumer imports `netrender_text` or passes a Parley
layout into it. Netrender's Parley 0.12 adapter therefore does not require a
Genet fork API migration. The two parser generations remain explicit in the
metadata ledger: Skrifa 0.42.1/0.44.0 and read-fonts 0.39.2/0.41.0.

The Genet workspace resolves wasm-bindgen 0.2.127, futures 0.4.77, and
JS/Web bindings 0.3.104 through its worker and Ortet application pins. Manifest
inspection found no dependency from imported Genet render/document/scripted
libraries back to those applications, and no Mere dependency on either one.
Those exact pins therefore do not constrain Mere's separate imported browser
cones, which use the 0.2.129 / 0.4.79 / 0.3.106 family. This is a dependency
scope audit, not a Wasm or browser acceptance receipt.
