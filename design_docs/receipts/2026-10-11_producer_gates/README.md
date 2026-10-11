# Complete producer-resource source adoption, 2026-10-11

All four workspace Netrender dependency pins now select
`ef5412d760a978584c8c1801e8a765c51aecd403`. The existing immutable Vello
provider `5862bba26c8a9f56f6a5fdc8ea6966b360a5d63a` and stable release
versions remain unchanged.

The workspace Cargo.lock is intentionally ignored; its qualified local
resolution is preserved in [the lock snapshot](Cargo.lock).

The Git-only `cargo check -p genet-render-host -p genet-render -j 2` passed;
see [command and exit](genet-pinned-render-check.json) and
[raw log](genet-pinned-render-check.log). This check initializes no adapter.
Netrender and Mere carry the separate complete-resource, CPU/Sparse GPU,
native presentation, input/accessibility and orderly rehydration receipts.
Classic edge pixel parity remains open, as do browser execution, OS assistive
technology interaction and forced driver-loss recovery. This commit adopts
qualified source contracts without changing shipping host renderer selection.
