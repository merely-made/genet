"""Run the real RTC integration-test artifact from a stable Windows firewall path."""

import argparse
import contextlib
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path("C:/t/cargo-targets/genet")
CACHE = TARGET / "rtc-interop"
EXECUTABLE = CACHE / "native-browser-tests.exe"
MARKER = CACHE / "native-browser-tests-owner.json"
OWNER = "genet-rtc-native-tests"


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def artifact(message):
    """Select Cargo's exact browser test harness, never a dependency or example."""
    target = message.get("target", {})
    if (message.get("reason") == "compiler-artifact"
            and target.get("name") == "browser"
            and target.get("kind") == ["test"]
            and message.get("profile", {}).get("test") is True
            and message.get("executable")):
        return Path(message["executable"])
    return None


def prepare(env):
    command = ["cargo", "test", "-p", "genet-webrtc", "--test", "browser",
               "--no-run", "--message-format=json", "--locked", "--offline", "-j1"]
    found = []
    with subprocess.Popen(command, cwd=ROOT, env=env, stdout=subprocess.PIPE,
                          text=True, encoding="utf-8") as process:
        for line in process.stdout:
            message = json.loads(line)
            selected = artifact(message)
            if selected is not None:
                found.append(selected)
            if message.get("reason") == "compiler-message":
                rendered = message.get("message", {}).get("rendered")
                if rendered:
                    print(rendered, file=sys.stderr, end="")
        if process.wait():
            raise RuntimeError("Cargo did not produce a successful test build")
    if len(found) != 1:
        raise RuntimeError("expected exactly one Cargo browser test artifact")
    original = found[0].resolve(strict=True)
    original.relative_to((TARGET / "build" / "debug" / "deps").resolve())
    if original.suffix.lower() != ".exe":
        raise RuntimeError("expected a Windows test executable")
    sha = digest(original)
    if EXECUTABLE.exists() and not MARKER.exists():
        raise RuntimeError("stable test path exists without its ownership marker")
    if MARKER.exists():
        previous = json.loads(MARKER.read_text(encoding="utf-8"))
        if previous.get("owner") != OWNER:
            raise RuntimeError("stable test path has another owner")
        if EXECUTABLE.exists() and digest(EXECUTABLE) != previous.get("sha256"):
            raise RuntimeError("stable executable differs from its ownership marker")
    partial = CACHE / "native-browser-tests.partial"
    if partial.exists():
        raise RuntimeError("an incomplete previous copy needs review")
    # Windows refuses replacing a running executable. The lock additionally
    # serializes preparation and execution through this runner.
    with partial.open("xb") as stream, original.open("rb") as source:
        shutil.copyfileobj(source, stream)
    if digest(partial) != sha:
        raise RuntimeError("copied test artifact hash mismatch")
    os.replace(partial, EXECUTABLE)
    metadata = {"owner": OWNER, "generated": True, "sourceArtifact": str(original),
                "executable": str(EXECUTABLE), "sha256": sha,
                "source": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()}
    MARKER.write_text(json.dumps(metadata, separators=(",", ":")) + "\n", encoding="utf-8")
    return metadata


@contextlib.contextmanager
def ownership_lock():
    import msvcrt
    CACHE.mkdir(parents=True, exist_ok=True)
    with (CACHE / "native-browser-tests.lock").open("a+b") as stream:
        if stream.tell() == 0:
            stream.write(b"0")
            stream.flush()
        stream.seek(0)
        msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
        try:
            yield
        finally:
            stream.seek(0)
            msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)


def preflight():
    command = ["pwsh", "-NoLogo", "-NoProfile", "-NonInteractive", "-File",
               str(ROOT / "support/ci/webrtc_permissions.ps1"), "-Kind", "Native"]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True,
                            encoding="utf-8", timeout=30)
    if result.returncode:
        print(result.stdout, end="")
        raise RuntimeError("native firewall prerequisite is missing or unreadable; no tests or listeners started")
    report = json.loads(result.stdout)
    if report.get("applicationRuleCoverage") is not True:
        raise RuntimeError("native preflight did not confirm application rules")


def main():
    parser = argparse.ArgumentParser(__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--prepare", action="store_true", help="build/copy only; opens no listeners")
    mode.add_argument("--list", action="store_true", help="enumerate tests only; opens no listeners")
    parser.add_argument("--server", help="controlled numeric IPv4 coturn endpoint; includes ignored tests")
    parser.add_argument("--lan-bind", help="selected nonloopback IPv4; requires --filter local_mdns_lan")
    parser.add_argument("--filter", help="Rust test-name filter")
    parser.add_argument("--timeout-seconds", type=int, default=600, help="positive execution deadline after compilation (default:600)")
    args = parser.parse_args()
    if args.timeout_seconds <= 0:
        parser.error("--timeout-seconds must be positive")
    if os.name != "nt":
        parser.error("this runner qualifies Windows executable/firewall ownership")
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(TARGET), CARGO_BUILD_BUILD_DIR=str(TARGET / "build"),
               CARGO_PROFILE_DEV_DEBUG="0", CARGO_BUILD_JOBS="1", CARGO_CACHE_AUTO_CLEAN_FREQUENCY="never")
    env.pop("MURM_RTC_TEST_SERVER", None)
    env.pop("GENET_RTC_LAN_BIND", None)
    if args.lan_bind:
        ip = ipaddress.IPv4Address(args.lan_bind)
        if ip.is_loopback or ip.is_unspecified or ip.is_multicast or args.filter != "local_mdns_lan":
            parser.error("--lan-bind requires a selected LAN IPv4 and --filter local_mdns_lan")
        env["GENET_RTC_LAN_BIND"] = str(ip)
    if args.server:
        address, port = args.server.rsplit(":", 1)
        ipaddress.IPv4Address(address)
        if not 1 <= int(port) <= 65535:
            parser.error("invalid controlled server port")
        env["MURM_RTC_TEST_SERVER"] = args.server
    with ownership_lock():
        metadata = prepare(env)
        print("RTC_NATIVE_ARTIFACT " + json.dumps(metadata, separators=(",", ":")), flush=True)
        if args.prepare:
            return 0
        if not args.list:
            preflight()
        # Match Cargo's DLL search placement for a standalone test invocation.
        env["PATH"] = os.pathsep.join([str(TARGET / "build/debug/deps"),
                                       str(TARGET / "debug/deps"), env.get("PATH", "")])
        command = [str(EXECUTABLE)]
        if args.list:
            command.append("--list")
        else:
            command += ["--test-threads=1"]
            if args.server or args.lan_bind:
                command.append("--include-ignored")
            if args.filter:
                command.append(args.filter)
        result = subprocess.run(command, cwd=ROOT, env=env, timeout=30 if args.list else args.timeout_seconds)
        if digest(EXECUTABLE) != metadata["sha256"]:
            raise RuntimeError("test executable changed during the run")
        return result.returncode


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        print("RTC native runner: " + str(error), file=sys.stderr)
        sys.exit(2)
