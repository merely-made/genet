# Checked raster and shared owner qualification

Final native GPU controls ran against Git-only Netrender c74b7cb28090c28b1c6d80b08dcdea18aff48d3e and provider 2eae6243881d5b24c984d5ee35dde509ca5db782. All three actual gates passed: destruction after admission, destruction after the final precheck through real renderer buffer writes protected by native scopes, and finite huge-curve provider refusal followed by identical valid pixels on the same healthy device. The original failing buffer-upload gate is retained as negative evidence.

The complete default render-host unit suite passed with the actual GPU controls separately ignored for serialized execution. Readback/callback controls use five-second bounds and no physical driver reset. Native checked raster scope coverage and browser/composition/staging limits are specified in [DEVICE_RECOVERY.md](../../DEVICE_RECOVERY.md). Genet Cargo.lock is intentionally untracked; source identities were verified from full Git-only Cargo metadata.
