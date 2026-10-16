#!/usr/bin/env python3
"""Focused synthetic controls for the static DOM dependency boundary."""

import importlib.util
import pathlib
import subprocess
import unittest
from contextlib import ExitStack
from unittest.mock import patch


SCRIPT = pathlib.Path(__file__).with_name("check_dependency_cones.py")
SPEC = importlib.util.spec_from_file_location("check_dependency_cones", SCRIPT)
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def graph(*, poison=None, duplicate=False):
    packages = [
        {"id": "static@1", "name": "genet-static-dom"},
        {"id": "html@1", "name": "html5ever"},
        {"id": "relay@1", "name": "relay"},
    ]
    nodes = [
        {"id": "static@1", "deps": [{"pkg": "html@1", "dep_kinds": [{"kind": None}]}]},
        {"id": "html@1", "deps": [{"pkg": "relay@1", "dep_kinds": [{"kind": None}]}]},
        {"id": "relay@1", "deps": []},
    ]
    parent = nodes[-1]
    if duplicate:
        packages.append({"id": "relay@2", "name": "relay"})
        nodes[1]["deps"].append({"pkg": "relay@2", "dep_kinds": [{"kind": None}]})
        parent = {"id": "relay@2", "deps": []}
        nodes.append(parent)
    if poison:
        packages.append({"id": "poison@1", "name": poison})
        parent["deps"].append({"pkg": "poison@1", "dep_kinds": [{"kind": None}]})
        nodes.append({"id": "poison@1", "deps": []})
    return {"packages": packages, "resolve": {"nodes": nodes}}


class StaticDomConeTests(unittest.TestCase):
    def test_clean_transitive_graph_passes(self):
        gate.assert_static_dom_cone(graph())

    def test_forbidden_transitive_edges_fail(self):
        for name in ("script-engine-boa", "genet-scripted-dom", "genet-render-host", "servo-webgl-essl"):
            with self.subTest(name=name), self.assertRaises(SystemExit):
                gate.assert_static_dom_cone(graph(poison=name))

    def test_same_name_package_ids_are_both_traversed(self):
        with self.assertRaises(SystemExit):
            gate.assert_static_dom_cone(graph(poison="boa_engine", duplicate=True))

    def test_resolved_metadata_failure_is_not_clean(self):
        failed = subprocess.CompletedProcess([], 47, "", "deliberate failure")
        with patch.object(gate.subprocess, "run", return_value=failed), self.assertRaises(SystemExit):
            gate.cargo_metadata_resolved()

    def test_tree_failure_is_not_clean(self):
        failed = subprocess.CompletedProcess([], 47, "", "deliberate failure")
        with patch.object(gate.subprocess, "run", return_value=failed), self.assertRaises(SystemExit):
            gate.cargo_tree_resolved_cone("ortet", "wasm32-unknown-unknown")

    def test_main_rejects_forbidden_static_dom_dependency(self):
        # Exercise the actual CI entrypoint. Other witnesses have separate
        # controls and would require Cargo; only the static assertion stays real.
        other_witnesses = (
            "assert_fleece_cone", "self_test_fleece_dependency_shape",
            "assert_fleece_resolved_cones", "self_test_resolved_cone",
            "assert_cargo_metadata_sees_fleece", "assert_ports_depend_inward",
            "assert_host_api_cone", "self_test_mere_witness",
            "assert_no_mere_source", "assert_ortet_cone",
            "assert_netfetcher_semantics_cone", "self_test_ortet_wasm_cone",
            "assert_ortet_wasm_cone",
        )
        with ExitStack() as patches:
            for name in other_witnesses:
                patches.enter_context(patch.object(gate, name))
            patches.enter_context(patch.object(gate, "cargo_metadata", return_value={}))
            patches.enter_context(patch.object(
                gate, "cargo_metadata_resolved",
                return_value=graph(poison="genet-scripted-dom"),
            ))
            with self.assertRaises(SystemExit):
                gate.main()


if __name__ == "__main__":
    unittest.main()
