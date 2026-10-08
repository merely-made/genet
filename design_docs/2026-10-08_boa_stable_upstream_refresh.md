# Boa stable upstream refresh

**Source, 2026-10-08:** maintained `mark-ik/boa`, branch `genet`, incorporates
upstream stable `v0.22` at `494ae680a10704722df3c98335073245b7088a24`.
The fork retains its `1.0.0-dev` package identity and Genet's exact version
contract. This brings six upstream commits forward: release preparation,
dependency/ICU updates, Rust lint fixes, and the Date intrinsic borrowing repair.
The fork's caller-realm provenance, finalization waiters, narrow standardized
ArrayBuffer transfer feature, and existing GC semantics remain in place.

Engine and GC are pinned to the same immutable revision in
[`script-engine-boa`](../components/script-engine-boa/Cargo.toml).
Browser Wasm targets enable Boa's `js` host glue for randomness and time;
native targets retain their existing feature selection. The broad experimental
feature remains separate from standardized transfer support.

## Qualification

- Provider: 1,112 engine tests and 30 GC tests passed, with the narrow transfer
  feature enabled. The fork's pre-push formatter, all-feature/all-target lint,
  and no-default-feature lint checks also passed with warnings denied.
- Candidate adapter: all 26 Genet Boa adapter tests passed.
- Browser target: `script-engine-boa` checked for `wasm32-unknown-unknown` with
  Boa's `js` feature. The public manifest now supplies that feature automatically
  on browser Wasm targets; public immutable-source checks are recorded separately.
- Clean consumer compilation: Livery, scripted DOM, and scripted worker checked
  against immutable Genet `e84f9c7f9aec23320c539784961d1465f8a53a9b` with the
  candidate. Its code and manifests matched primary `6cb2284a` before this
  dependency change; those revisions differed only in documentation. Concurrent
  DOM/forms/runtime edits were excluded from this compile receipt.

Genet's old generated lock selected arrayvec 0.7.6 and ICU 2.2. Refreshing only
Boa's dependency subtree left the other ICU users locked to 2.2. A full compatible
resolution produced one ICU 2.3 family under existing Genet, Livery, and Vano
requirements. The prior generated lock is archived in the machine-local refresh
evidence, alongside the qualified resolution and input hashes. Re-resolve stale
local overlay locks when adopting the new pin.

The evidence is in `Code/testing/workspace-upstreams`: provider, candidate,
clean consumer, public source, and browser target logs and gate records.
This is Windows native automated and Wasm compile evidence. Aggregate DOM/WPT,
headed browser scripting acceptance, and concurrent forms work remain separate
gates. This refresh does not choose a new GC policy or destruction ordering.
