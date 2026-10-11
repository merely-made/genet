#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Original Unicode17 normative-data transformer for native HTML directionality.

This maintained generator continues the original Phase C evidence derivation.
It contains no copied third-party classifier implementation. The source file
and license are retained byte-for-byte beside it; see README.md for provenance.
"""

import argparse
import bisect
import hashlib
import json
import re
from pathlib import Path

SOURCE_HASH = "4867b4b7f0731ed1bfcd34cc6251211ff1542541fce0734b6fbda139ee80b3a4"
LICENSE_HASH = "e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96"
FULL_INTERVAL_HASH = "cad2cde85b0f8cc5db7af996c51cba2d0e817d8f732e6bd3571cc85b74682616"


def digest(data):
    return hashlib.sha256(data).hexdigest()


def declarations(source):
    missing, explicit = [], []
    pattern = re.compile(r"([0-9A-F]+)(?:\.\.([0-9A-F]+))?\s*;\s*(\w+)")
    for line in source.splitlines():
        if line.startswith("# @missing:"):
            target, record = missing, line.split(":", 1)[1].strip()
        else:
            target, record = explicit, line.split("#", 1)[0].strip()
        match = pattern.fullmatch(record)
        if match:
            low, high, category = match.groups()
            target.append((int(low, 16), int(high or low, 16), category))
    assert len(missing) == 24 and len(explicit) == 2323
    return missing, explicit


def direction(category):
    if category in ("L", "Left_To_Right"):
        return "L"
    if category in ("R", "AL", "Right_To_Left", "Arabic_Letter"):
        return "Rtl"
    return "Neutral"


def intervals(missing, explicit):
    boundaries = {0, 0xD800, 0xE000, 0x110000}
    for low, high, _ in missing + explicit:
        boundaries.update((low, high + 1))
    points = sorted(boundaries)
    result = []
    for low, stop in zip(points, points[1:]):
        if 0xD800 <= low < 0xE000:
            continue
        category = "Left_To_Right"
        for records in (missing, explicit):
            for start, end, override in records:
                if start <= low <= end:
                    category = override
        value = direction(category)
        if result and result[-1][1] + 1 == low and result[-1][2] == value:
            result[-1][1] = stop - 1
        else:
            result.append([low, stop - 1, value])
    return result


def verify(source_text, exceptions):
    # Independent pointwise expansion of the original declarations. Compare
    # exception-table binary lookup, including default-L gaps, for every scalar.
    codes = {"L": 0, "Rtl": 1, "Neutral": 2}
    expected = bytearray(0x110000)
    for records in declarations(source_text):
        for start, end, category in records:
            expected[start:end + 1] = bytes([codes[direction(category)]]) * (end - start + 1)
    starts = [row[0] for row in exceptions]
    assert all(a <= b and not (a <= 0xDFFF and b >= 0xD800) for a, b, _ in exceptions)
    assert all(exceptions[i - 1][1] < exceptions[i][0] for i in range(1, len(exceptions)))
    count = 0
    for scalar in range(0x110000):
        if 0xD800 <= scalar <= 0xDFFF:
            continue
        index = bisect.bisect_right(starts, scalar) - 1
        actual = 0
        if index >= 0 and scalar <= exceptions[index][1]:
            actual = codes[exceptions[index][2]]
        assert actual == expected[scalar], f"scalar U+{scalar:04X}"
        count += 1
    assert count == 1112064
    return count


def rust_table(exceptions):
    lines = [
        "// Generated from Unicode 17.0.0 DerivedBidiClass.txt. Do not edit.",
        "// Source SHA256: " + SOURCE_HASH,
        "// Generator: unicode/generate_directionality.py; source and full license in unicode/.",
        "// Unicode License V3 applies to the derived classification data.",
        "",
        "use crate::FormDirection;",
        "",
        "// Sorted default-L exceptions: true means R/AL, false means Neutral.",
        "const EXCEPTIONS: &[(u32, u32, bool)] = &[",
    ]
    for start, end, category in exceptions:
        lines.append(f"    (0x{start:06X}, 0x{end:06X}, {'true' if category == 'Rtl' else 'false'}),")
    lines += [
        "];", "",
        "pub(super) fn strong_direction(character: char) -> Option<FormDirection> {",
        "    let scalar = character as u32;",
        "    let index = EXCEPTIONS.partition_point(|&(start, _, _)| start <= scalar);",
        "    if index != 0 {",
        "        let (_, end, rtl) = EXCEPTIONS[index - 1];",
        "        if scalar <= end {",
        "            return if rtl { Some(FormDirection::Rtl) } else { None };",
        "        }",
        "    }",
        "    Some(FormDirection::Ltr)",
        "}", "",
    ]
    return "\n".join(lines).encode("utf-8")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--receipt", type=Path, required=True)
    args = parser.parse_args()
    own_path = Path(__file__).resolve()
    source = (own_path.parent / "DerivedBidiClass-17.0.0.txt").read_bytes()
    license_data = (own_path.parent / "LICENSE.txt").read_bytes()
    assert digest(source) == SOURCE_HASH and digest(license_data) == LICENSE_HASH
    text = source.decode("utf-8")
    assert "DerivedBidiClass-17.0.0.txt" in text
    full = intervals(*declarations(text))
    canonical = (json.dumps({"unicode_version": "17.0.0", "classes": ["L", "Rtl", "Neutral"], "intervals": full}, separators=(",", ":")) + "\n").encode()
    assert digest(canonical) == FULL_INTERVAL_HASH
    exceptions = [row for row in full if row[2] != "L"]
    assert len(full) == 1124 and len(exceptions) == 619
    scalars_checked = verify(text, exceptions)
    output = rust_table(exceptions)
    args.output.write_bytes(output)
    receipt = {
        "unicode_version": "17.0.0", "source_sha256": digest(source), "source_bytes": len(source),
        "license_sha256": digest(license_data), "license_bytes": len(license_data),
        "generator_sha256": digest(own_path.read_bytes()), "output_sha256": digest(output),
        "output_bytes": len(output), "full_interval_sha256": digest(canonical),
        "interval_count": len(full), "exception_count": len(exceptions),
        "scalars_checked": scalars_checked, "mismatches": 0, "surrogates_excluded": 2048,
        "validation": "pointwise normative declaration expansion versus default-L exception binary lookup",
    }
    args.receipt.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(receipt, sort_keys=True))


if __name__ == "__main__":
    main()
