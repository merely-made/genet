# Static DOM dependency gate

The current static, script-free package is `genet-static-dom`. Its normal
resolved dependency cone must exclude script engines and the render, host,
media, and WebGL systems. The planned `genet-static-html` package is not a
current workspace member.

CI runs the canonical dependency-cone witness:

```sh
python3 support/ci/check_dependency_cones.py
```

The static DOM assertion shares the full Cargo metadata graph and package-ID
traversal used by the Ortet witness. Resolution failure exits nonzero; it
cannot count as a clean cone. Focused synthetic controls, including failed
Cargo commands, run with:

```sh
python3 -m unittest support/ci/test_check_dependency_cones.py
```

The older shell and PowerShell launchers were retired because they targeted
the absent `genet-static-html` package and could report success after Cargo
failed.
