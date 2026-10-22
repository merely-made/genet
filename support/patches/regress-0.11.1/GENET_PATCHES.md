# Native form matcher patch

Vendored from the crates.io regress 0.11.1 package. MIT and Apache-2.0 licenses are retained.

Forms Phase B found that nested negated Unicode sets discarded the return value of CodePointSet::inverted(), so `[[a-z]&&[^aeiou]]+` rejected `bcdf`. The local two-line parser fix retains that complement. Version, features and dependency requirements stay unchanged.

The native genet-scripted-dom dependency uses this path directly. Boa and Vano retain their original registry dependencies. The root excludes this directory from workspace membership. This native-only override was explicitly approved by Mark on 2026-10-09.

Before and after evidence, exact upstream inventory and the patch are in testing/genet/forms/thinkpad. Retire this copy when an approved registry release includes the fix and the same native UTF16 / UnicodeSets cases pass.
