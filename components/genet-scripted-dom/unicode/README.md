# Unicode 17 form directionality data

The adjacent `DerivedBidiClass-17.0.0.txt` and `LICENSE.txt` are exact retained
response bytes from the official sources:

- https://www.unicode.org/Public/17.0.0/ucd/extracted/DerivedBidiClass.txt
- https://www.unicode.org/license.txt

Source version 17.0.0, source date 2025-07-24. Source SHA256:
`4867b4b7f0731ed1bfcd34cc6251211ff1542541fce0734b6fbda139ee80b3a4`.
Unicode License V3 SHA256:
`e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96`.
The full license/copyright/permission notice accompanies both original and
derived data here; do not remove it when distributing the generated table.

`generate_directionality.py` is an original normative-data transformer,
continuing the original Phase C evidence generator dated 2026-10-09. That
evidence generator's SHA256 was
`0a75fef515d247e4525f3776b376162cf16d4071bb6d098d19aefad59b622990`.
No third-party classifier or HTML directionality implementation was copied.
HTML traversal in `../forms_directionality.rs` is separate handwritten code.

Generation applies all 24 @missing declarations in declaration order and then
all 2323 explicit overrides, excludes surrogate code points, merges R/AL and
all other non-L classes respectively, and collapses adjacent equal intervals.
The full 1124 intervals reproduce original canonical evidence JSON SHA256
`cad2cde85b0f8cc5db7af996c51cba2d0e817d8f732e6bd3571cc85b74682616`.
The native table stores 619 exceptions to default L, including Neutral overrides.
Lookup is binary search; the generator independently expands the normative
declarations pointwise and checks all 1112064 Unicode scalars against lookup.
It rejects source/license hash drift, differing counts, surrogate overlap,
unsorted ranges, or any mismatch before writing output.

From the repository root, regenerate (receipt location is caller-owned):

```sh
python3 components/genet-scripted-dom/unicode/generate_directionality.py \
  --output components/genet-scripted-dom/forms_directionality_data.rs \
  --receipt /absolute/path/to/unicode-generation-receipt.json
```

Changing Unicode versions requires deliberately updating source/license hashes
and the independently retained canonical interval hash, then exhaustive
validation. Other engine text layers may use other Unicode versions; this table
pins the HTML form first-strong classifier explicitly.
