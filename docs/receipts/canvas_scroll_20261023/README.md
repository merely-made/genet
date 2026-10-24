# Canvas-colour scroll repair provider evidence

The original pixel reproducer fails with 2 passed / 2 failed. Its lower-viewport sample is transparent instead of opaque beige. The final suite has five CPU pixel cases, including the additional spliced-child guard, and the complete genet-livery run passes 725 tests with eight explicit ignores across 52 cohorts. The scripted lane check passes.

Start with `qualification.json` and `repair.patch`. The source is the recorded Git base plus that patch; the six changed Rust files have exact hashes. The local provider workspace does not track Cargo.lock, so the existing primary lock was copied, allowed to resolve offline, then retained here. Final checks use the frozen lock. `before.log` records the initial no-lock/yanked-cache refusal; it is not the red regression.

`before-with-lock.log` is the actual failing regression run. `after.log` is the first four-case passing raster run. The final suite adds the child-splice case. `full-livery.log` and `full-livery-final.log` preserve two unrelated test/example compiler errors, corrected with explicit byte-map typing and conversion to shared image pixels. `full-livery-repaired.log` is the complete passing run. Build warnings are preserved.

Document scroll leaves the propagated canvas colour outside its transform. Ordinary element paint, child and host slots still move; generic placement and page zoom still move/scale the complete surface. Transparent canvases remain transparent. The serialized paint-list shape and background-image emitter are unchanged. Vulkan receiving and before/after native pixels belong to the Turnstone consumer receipt.
