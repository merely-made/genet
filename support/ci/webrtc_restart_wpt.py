"""Frozen, unchanged WPT cases for the bounded ICE-restart receiving gate."""

import argparse
import collections
import datetime
import hashlib
import itertools
import json
from pathlib import Path
import re
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[2]
RECEIPT = ROOT / "design_docs/archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md"
CURRENT = ROOT.parents[1] / "testing/genet/design_docs/archive_docs/receipts/2026-11-03_webrtc_conformance_plan_receipts.md"
FILES = [
    "RTCPeerConnection-constructor.html",
    "RTCIceCandidate-constructor.html",
    "RTCPeerConnection-createDataChannel.html",
    "RTCPeerConnectionIceErrorEvent.html",
    "RTCDataChannel-iceRestart.html",
    "RTCPeerConnection-restartIce.https.html",
    "RTCPeerConnection-restartIce-onnegotiationneeded.https.html",
]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def records(text):
    for block in re.findall(r"```json\n(.*?)\n```", text, re.S):
        try:
            value = json.loads(block)
        except ValueError:
            continue
        if isinstance(value, dict):
            yield value


def case_map(rows):
    seen = collections.Counter()
    result = {}
    for row in rows:
        seen[row["name"]] += 1
        result[(row["name"], seen[row["name"]])] = row["status"]
    return result


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--phase", choices=["before", "after"], required=True)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--compact", action="store_true", help="retain deltas against an archived complete after map")
    parser.add_argument("--receipt", type=Path, default=CURRENT)
    parser.add_argument("--reference-receipt", type=Path, default=RECEIPT)
    parser.add_argument("--gate", choices=["E2d", "E2r"], default="E2d")
    args = parser.parse_args()
    if args.compact and args.phase != "after":
        parser.error("--compact requires --phase after")
    if args.receipt.resolve() == RECEIPT.resolve():
        parser.error("the completed E2r packet is read-only; select a fresh receipt")
    text = args.receipt.read_text(encoding="utf-8")
    reference_text = args.reference_receipt.read_text(encoding="utf-8")
    tests = ROOT / "tests/wpt/tests"
    sources = [tests / "webrtc" / name for name in FILES]
    sources += [tests / "resources/testharness.js", tests / "webrtc/RTCPeerConnection-helper.js"]
    result = {
        "gate": args.gate + "-WPT", "phase": args.phase,
        "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "source": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "binarySHA256": digest(args.runner), "lockSHA256": digest(ROOT / "Cargo.lock"),
        "sourceSHA256": {str(p.relative_to(ROOT)): digest(p) for p in sources},
        "outcomes": [], "comparisons": [],
    }
    if args.phase == "before":
        assert any(result["lockSHA256"] in [r.get("lockSHA256"), r.get("resultSHA256")] for r in records(reference_text)), "matching archived lock reference required"
        result["lockReference"] = {"file": str(args.reference_receipt), "packetSHA256": digest(args.reference_receipt)}
    failure = None
    try:
        for engine, name in itertools.product(["boa", "nova"], FILES):
            test = "webrtc/" + name
            command = [str(args.runner), "testharness-one", test, "--test-path", str(tests / test), "--tests-root", str(tests), "--engine", engine, "--drive-deadline", "5"]
            process = subprocess.run(command, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=40)
            reports = [json.loads(line.split("__thresult ", 1)[1]) for line in process.stdout.splitlines() if "__thresult " in line]
            assert len(reports) == 1, (engine, name, process.returncode, "missing harness result")
            report = reports[0]
            # Retain exact assertion identities and statuses, never SDP/credentials
            # that a failed assertion's diagnostic text might contain.
            outcome = {key: report.get(key) for key in ["status", "subtests_passed", "subtests_total", "reason"]}
            outcome.update(engine=engine, test=test, exit=process.returncode,
                           subtests=[{key: row[key] for key in ["name", "status"]} for row in report.get("subtests", [])])
            if name == "RTCDataChannel-iceRestart.html" and outcome["status"] == "error":
                inventory = ["Data channel remains usable after ICE restart", "Data channel remains usable at each step of an ICE restart"]
                outcome["pendingSourceCases"] = [n for n in inventory if n not in {r["name"] for r in outcome["subtests"]}]
            result["outcomes"].append(outcome)
            print(engine, name, outcome["subtests_passed"], "/", outcome["subtests_total"], outcome["status"], flush=True)
            assert process.returncode == 0
        if args.phase == "after":
            before = next(r for r in reversed(list(records(text))) if r.get("gate") == result["gate"] and r.get("phase") == "before" and not r.get("failure"))
            assert result["sourceSHA256"] == before["sourceSHA256"], "WPT source drift"
            result["beforeBinarySHA256"] = before["binarySHA256"]
            for old, new in zip(before["outcomes"], result["outcomes"], strict=True):
                a, b = case_map(old["subtests"]), case_map(new["subtests"])
                compare = {"engine": new["engine"], "test": new["test"], "missing": sorted(a.keys() - b.keys()), "added": sorted(b.keys() - a.keys()), "changed": [[key, a[key], b[key]] for key in a.keys() & b.keys() if a[key] != b[key]]}
                result["comparisons"].append(compare)
                recovered = {(name, 1) for name in old.get("pendingSourceCases", [])}
                # An ERROR result reports only completed rows. A newly completed
                # row must be an identified pending case from the unchanged source.
                if old["test"].endswith("RTCDataChannel-iceRestart.html") and old["status"] == "error":
                    recovered.add(("Data channel remains usable at each step of an ICE restart", 1))
                compare["recoveredPending"] = [key for key in compare["added"] if tuple(key) in recovered]
                assert not compare["missing"] and not any(tuple(key) not in recovered for key in compare["added"]), "case identity drift"
                assert not any(a[key] == "pass" and b[key] != "pass" for key in a.keys() & b.keys()), "prior pass regressed"
                if new["test"].endswith("RTCDataChannel-iceRestart.html"):
                    assert new["status"] == "pass" and new["subtests_passed"] == new["subtests_total"] == 2
                if new["test"].endswith("RTCPeerConnection-restartIce.https.html"):
                    assert b[("restartIce() has no effect on a closed peer connection", 1)] == "pass"
    except Exception as error:
        failure = str(error)[:300]
    result["failure"] = failure
    if args.compact:
        reference = next(r for r in reversed(list(records(reference_text)) + list(records(text))) if r.get("gate") in ["E2r-WPT", result["gate"]] and r.get("phase") == "after" and not r.get("failure") and all("subtests" in o for o in r["outcomes"]))
        result["caseMapReference"] = {key: reference[key] for key in ["utc", "binarySHA256"]}
        normalized = lambda rows: json.dumps([{**row, "changed": sorted(row["changed"])} for row in rows], sort_keys=True, separators=(",", ":"))
        comparison = normalized(result["comparisons"])
        if comparison == normalized(reference["comparisons"]):
            result["comparisonMatchesReference"] = True
            result["comparisonSHA256"] = hashlib.sha256(comparison.encode()).hexdigest()
            del result["comparisons"]
        by_case = {(o["engine"], o["test"]): o for o in reference["outcomes"]}
        for outcome in result["outcomes"]:
            prior = by_case[(outcome["engine"], outcome["test"])]
            a, b = case_map(prior["subtests"]), case_map(outcome["subtests"])
            outcome["caseDeltaFromReference"] = {
                "missing": sorted(a.keys() - b.keys()), "added": sorted(b.keys() - a.keys()),
                "changed": [[key, a[key], b[key]] for key in sorted(a.keys() & b.keys()) if a[key] != b[key]],
            }
            outcome["subtestsSHA256"] = hashlib.sha256(json.dumps(outcome.pop("subtests"), separators=(",", ":")).encode()).hexdigest()
    entry = "\n\n## " + args.gate + " WPT " + args.phase + " - " + result["utc"] + "\n\n```json\n" + json.dumps(result, separators=(",", ":")) + "\n```\n"
    current = args.receipt.read_text(encoding="utf-8")
    assert len((current + entry).encode()) <= 220_000
    with args.receipt.open("a", encoding="utf-8", newline="\n") as stream:
        stream.write(entry)
    if failure:
        print(failure, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
