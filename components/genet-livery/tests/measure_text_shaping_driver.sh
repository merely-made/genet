#!/bin/sh
# Validate a compiled driver's declared identity from any working directory.
set -eu
binary=$1
expected_features=$2
font=$3
label=0123456789abcdef0123456789abcdef01234567
result=$(mktemp)
trap 'rm -f "$result"' EXIT
if "$binary" --font "$font" --paragraphs 1 > "$result" 2>&1; then
    echo 'missing source revision accepted' >&2
    exit 1
fi
if "$binary" --source-revision invalid --font "$font" > "$result" 2>&1; then
    echo 'invalid source revision accepted' >&2
    exit 1
fi
"$binary" --source-revision "$label" --font "$font" --paragraphs 1 --output "$result"
test "$(wc -l < "$result")" -eq 4
if grep -v "source_revision=$label features=$expected_features " "$result" | grep .; then
    echo 'incorrect source or feature label' >&2
    exit 1
fi
fingerprint=$(python3 - "$binary" <<'PYHASH'
import sys
h = 0xcbf29ce484222325
for byte in open(sys.argv[1], 'rb').read():
    h = ((h ^ byte) * 0x100000001b3) & 0xffffffffffffffff
print(f'{h:016x}')
PYHASH
)
if grep -v "executable_fnv64=$fingerprint " "$result" | grep .; then
    echo 'missing executable fingerprint' >&2
    exit 1
fi
