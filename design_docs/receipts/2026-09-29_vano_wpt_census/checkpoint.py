"""Reject mixed runner, corpus, engine, mode or directory checkpoints."""
def validate_checkpoint(data, scope, provenance):
    expected = {
        "runner_sha256": provenance["runner_sha256"],
        "manifest_sha256": provenance["manifest_sha256"],
        "engine": "nova", "renderer": "livery", "command": "testharness",
        "subset": scope["subset"], "policy": "exact",
    }
    for field, value in expected.items():
        if data.get(field) != value:
            raise ValueError(f"{scope['subset']}: checkpoint {field} mismatch: {data.get(field)!r} != {value!r}")
    if not isinstance(data.get("tests"), dict):
        raise ValueError(f"{scope['subset']}: checkpoint tests must be a map")
