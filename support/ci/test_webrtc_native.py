"""Exercise the unattended runner's artifact selection and execution boundary."""
import contextlib
import io
from pathlib import Path
import shutil
import types
import unittest
from unittest.mock import patch

from support.ci import webrtc_native as runner


@unittest.skipUnless(runner.os.name == "nt", "Windows RTC receiving runner")
class NativeRunnerTests(unittest.TestCase):
    def check_firewall_case(self, case):
        legacy = Path(runner.os.environ.get("SystemRoot", "C:/Windows")) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        shells = [str(legacy)] if legacy.is_file() else []
        if modern := shutil.which("pwsh"):
            shells.append(modern)
        self.assertTrue(shells, "A PowerShell runtime is required on Windows")
        for shell in shells:
            with self.subTest(shell=shell, case=case):
                result = runner.subprocess.run([
                    shell, "-NoLogo", "-NoProfile", "-NonInteractive", "-File",
                    str(runner.ROOT / "support/ci/test_webrtc_permissions.ps1"), "-Case", case
                ], capture_output=True, text=True, encoding="utf-8", timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                report = runner.json.loads(result.stdout)
                self.assertTrue(report["valid" if case == "Scope" else "stopped"])

    def test_firewall_scope_passes_windows_validation_without_installing(self):
        self.check_firewall_case("Scope")

    def test_firewall_rejection_stops_the_installation_pipeline(self):
        self.check_firewall_case("Failure")

    def test_windows_powershell_51_accepts_the_setup_preview(self):
        legacy = Path(runner.os.environ.get("SystemRoot", "C:/Windows")) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        if not legacy.is_file():
            self.skipTest("Windows PowerShell 5.1 is not installed")
        result = runner.subprocess.run([
            str(legacy), "-NoLogo", "-NoProfile", "-NonInteractive", "-File",
            str(runner.ROOT / "support/ci/webrtc_permissions.ps1"), "-Kind", "Native", "-Install", "-WhatIf"
        ], capture_output=True, text=True, encoding="utf-8", timeout=60)
        self.assertEqual(result.returncode, 0, result.stderr)
        reports = [runner.json.loads(line) for line in result.stdout.splitlines() if line.startswith("{")]
        self.assertEqual(len(reports), 1)
        self.assertEqual(reports[0]["kind"], "Native")

    def test_only_a_runnable_browser_test_artifact_is_selected(self):
        good = {"reason": "compiler-artifact", "target": {"name": "browser", "kind": ["test"]},
                "profile": {"test": True}, "executable": "C:/artifact/browser.exe"}
        self.assertEqual(runner.artifact(good), Path(good["executable"]))
        for change in [{"executable": None}, {"reason": "compiler-message"},
                       {"profile": {"test": False}},
                       {"target": {"name": "browser", "kind": ["example"]}},
                       {"target": {"name": "other", "kind": ["test"]}}]:
            with self.subTest(change=change):
                self.assertIsNone(runner.artifact(good | change))

    def enter_runner(self, stack, argv):
        stack.enter_context(patch.object(runner.sys, "argv", ["runner"] + argv))
        stack.enter_context(patch.object(runner, "ownership_lock", contextlib.nullcontext))
        stack.enter_context(patch.object(runner, "prepare", return_value={"sha256": "fixture"}))
        stack.enter_context(patch.object(runner, "digest", return_value="fixture"))
        stack.enter_context(contextlib.redirect_stdout(io.StringIO()))

    def test_missing_preflight_starts_no_test_process(self):
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, [])
            stack.enter_context(patch.object(runner, "preflight", side_effect=RuntimeError("missing rule")))
            execute = stack.enter_context(patch.object(runner.subprocess, "run"))
            with self.assertRaisesRegex(RuntimeError, "missing rule"):
                runner.main()
            execute.assert_not_called()

    def test_enumeration_only_bypasses_the_listener_preflight(self):
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, ["--list"])
            preflight = stack.enter_context(patch.object(runner, "preflight"))
            execute = stack.enter_context(patch.object(runner.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)))
            self.assertEqual(runner.main(), 0)
            preflight.assert_not_called()
            self.assertEqual(execute.call_args.args[0], [str(runner.EXECUTABLE), "--list"])

    def test_network_run_checks_permissions_first_and_clears_ambient_server(self):
        events = []
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, ["--filter", "boa::constructors"])
            stack.enter_context(patch.dict(runner.os.environ, {"MURM_RTC_TEST_SERVER": "stale-fixture", "GENET_RTC_LAN_BIND": "stale-interface"}))
            stack.enter_context(patch.object(runner, "preflight", side_effect=lambda: events.append("preflight")))
            def execute(command, **kwargs):
                events.append("execute")
                self.assertNotIn("MURM_RTC_TEST_SERVER", kwargs["env"])
                self.assertNotIn("GENET_RTC_LAN_BIND", kwargs["env"])
                self.assertNotIn("--include-ignored", command)
                self.assertIn("boa::constructors", command)
                return types.SimpleNamespace(returncode=0)
            stack.enter_context(patch.object(runner.subprocess, "run", side_effect=execute))
            self.assertEqual(runner.main(), 0)
            self.assertEqual(events, ["preflight", "execute"])

    def test_lan_selection_is_explicit_and_clears_ambient_binding(self):
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, ["--lan-bind", "192.0.2.10", "--filter", "local_mdns_lan"])
            stack.enter_context(patch.object(runner, "preflight"))
            execute = stack.enter_context(patch.object(runner.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)))
            self.assertEqual(runner.main(), 0)
            self.assertEqual(execute.call_args.kwargs["env"]["GENET_RTC_LAN_BIND"], "192.0.2.10")
            self.assertIn("--include-ignored", execute.call_args.args[0])

    def test_executable_changes_after_a_run_are_refused(self):
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, ["--list"])
            stack.enter_context(patch.object(runner, "digest", return_value="changed"))
            stack.enter_context(patch.object(runner.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)))
            with self.assertRaisesRegex(RuntimeError, "changed during the run"):
                runner.main()

    def test_network_execution_has_the_callers_deadline(self):
        with contextlib.ExitStack() as stack:
            self.enter_runner(stack, ["--timeout-seconds", "42"])
            stack.enter_context(patch.object(runner, "preflight"))
            execute = stack.enter_context(patch.object(runner.subprocess, "run", return_value=types.SimpleNamespace(returncode=0)))
            self.assertEqual(runner.main(), 0)
            self.assertEqual(execute.call_args.kwargs["timeout"], 42)


if __name__ == "__main__":
    unittest.main()
