"""Reproduce the archived platform census scope with the current Vano runner."""
import collections, datetime, hashlib, json, pathlib, subprocess, time
ROOT = pathlib.Path(__file__).resolve().parent
REPO = pathlib.Path(r"C:\Users\mark_\Code\worktrees\genet-vano-census")
BIN = pathlib.Path(r"C:\t\cargo-targets\genet\debug\genet-wpt-vano-census.exe")
TESTS = pathlib.Path(r"C:\Users\mark_\Code\repos\genet\tests\wpt\tests")
SCOPE = json.loads((ROOT / "scope.json").read_text(encoding="utf-8"))
OUT = ROOT / "disk-fixed"
OUT.mkdir(exist_ok=True)
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
metadata = {"started": now(), "genet_revision": "19c206873ab08ae227217892d9e74d0df18b349a", "vano_revision": "8ad0841255c2cbb679f7c704417d427b3cb3e961", "runner": str(BIN), "runner_sha256": digest(BIN), "manifest_sha256": digest(TESTS.parent / "meta/MANIFEST.json"), "profile": "dev", "source_description": "Genet19c206 plus isolated-worker reporting patch", "runner_patch_sha256": "8a5cd5918483f3ddb515b105e6adc65d21206e7a071590f552a07a564cbfb9b2", "genet_tree": "56a490f4343bc6cdcb57d23c7a972f083e186709", "cargo_lock_sha256": "db8ff719fbd101c4a909ce76441dad4aacc2b2f66af5b1184c65673bcc72a4e6", "rustc": "1.97.1 (8bab26f4f 2026-07-14)", "host": "x86_64-pc-windows-msvc", "backend": "Vano", "technical_engine_name": "nova", "scope_shards":len(SCOPE), "baseline_files":sum(row["baseline_files"] for row in SCOPE), "jobs":4,"timeout_seconds":30,"drive_deadline_seconds":15,"harness_gc":True,"disk_mode":True}
(ROOT / "provenance.json").write_text(json.dumps(metadata, indent=2)+"\n", encoding="utf-8")
def progress(current):
    count = 0; files = 0; historical = 0; timeouts = 0
    for scope in SCOPE:
        path = OUT / scope["baseline_file"].replace("_boa.json", "_vano.json")
        if path.exists():
            records = json.loads(path.read_text(encoding="utf-8"))["tests"]
            old = json.loads((ROOT.parent / "2026-09-06_platform_census" / "disk" / scope["baseline_file"]).read_text(encoding="utf-8"))["tests"]
            historical += len(records.keys() & old.keys())
            count += 1; files += len(records)
            timeouts += sum(r.get("reason") == "hang-killed" for r in records.values())
    value = {"updated": now(), "completed_shards": count, "total_shards": len(SCOPE), "completed_files_in_checkpointed_shards": files, "completed_historical_files":historical,"baseline_total_files":21672, "external_timeouts_in_checkpointed_shards":timeouts, "current_subset":current}
    (ROOT / "progress.json").write_text(json.dumps(value, indent=2)+"\n", encoding="utf-8")
# Keep each shard independently restartable; validate its runner identity.
for row in SCOPE:
    name = row["baseline_file"].replace("_boa.json", "_vano.json")
    result = OUT / name
    if result.exists() and json.loads(result.read_text(encoding="utf-8")).get("runner_sha256") == metadata["runner_sha256"]:
        print("EXISTING", row["subset"], flush=True)
        continue
    command = [str(BIN), "testharness", row["subset"], "--tests-root", str(TESTS), "--engine", "nova", "--renderer", "livery", "--jobs", "4", "--timeout", "30", "--drive-deadline", "15", "--write-expectations", str(result)]
    start = time.monotonic()
    progress(row["subset"])
    print("START", row["subset"], now(), flush=True)
    with result.with_suffix(".log").open("wb") as log:
        done = subprocess.run(command, cwd=REPO, stdout=log, stderr=subprocess.STDOUT)
    receipt = {"subset":row["subset"], "command":command,"exit_code":done.returncode,"seconds":round(time.monotonic()-start,3),"finished":now(),"result_present":result.exists()}
    with (ROOT / "runs-fixed.jsonl").open("a", encoding="utf-8") as log: log.write(json.dumps(receipt)+"\n")
    print("DONE", row["subset"], receipt["exit_code"], receipt["seconds"], flush=True)
    if not result.exists():
        raise SystemExit("Missing result map; inspect log before continuing")
progress(None)
metadata["finished"] = now()
metadata["runner_sha256_after"] = digest(BIN)
(ROOT / "provenance.json").write_text(json.dumps(metadata, indent=2)+"\n", encoding="utf-8")
print("ALLDONE", flush=True)
