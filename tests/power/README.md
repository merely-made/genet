`PowerMeasure.py` was a historical Servo PowerMetrics runner. It launched
`servoshell` with retired command-line arguments and stopped PowerMetrics by
process name, so the runner and its invocation instructions have been retired.
Its source remains available in Git history.

Benchmark and expected-result fixtures are retained as data; this runner did
not establish a current Genet benchmark gate. In particular,
`tests/jquery/expected_selector.txt` remains alongside the retired jQuery
launcher. Imported Blink benchmark assets and their licenses are also retained.
