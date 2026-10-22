# Current native frame quality completion

All four actual GPU controls passed against published Netrender c74b7cb28090c28b1c6d80b08dcdea18aff48d3e and provider 2eae6243881d5b24c984d5ee35dde509ca5db782. In addition to native loss/error-scope and healthy preflight controls, the finite width-40,000,000 round cap actually submits and reports CurveFlatteningLimit through shader counter readback before the checked call can return success. Its 32x24 target avoids a large render target. The test requires drained counters, the same core/generation, zero replacement calls and subsequent valid qualified nonblank pixels.

The completion fence shares a five-second deadline within admission/error scopes. This is a native shipping acceptance guarantee; browser async counter completion remains open. The provider algorithm and pixel thresholds are unchanged. See [DEVICE_RECOVERY.md](../../DEVICE_RECOVERY.md) for scope and timing implications.

The final deadline gate repeats all four actual controls after enforcing expiry before counter polling, before zero-counter success and after scope collection. Provider, captured API error and device-loss priority are preserved; no successful target may escape an expired request.

The non-test native library check passed after qualifying the deadline Duration through std::time; unit-test-only imports are not required by production.
