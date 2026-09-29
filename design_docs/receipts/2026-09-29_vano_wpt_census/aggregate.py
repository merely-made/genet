import collections,gzip,hashlib,json,pathlib
from checkpoint import validate_checkpoint
ROOT=pathlib.Path(__file__).resolve().parent
PROVENANCE=json.loads((ROOT/"provenance.json").read_text(encoding="utf-8"))
HISTORICAL=json.loads(gzip.decompress((ROOT/"historical-membership.json.gz").read_bytes()))
historical_counts=collections.Counter();new_counts=collections.Counter()
rows=[];totals=collections.Counter();reasons=collections.Counter();substatuses=collections.Counter();changes=collections.Counter();all_current=set();all_old=set()
for scope in json.loads((ROOT/"scope.json").read_text(encoding="utf-8")):
    path=ROOT/"disk-fixed"/scope["baseline_file"].replace("_boa.json","_vano.json")
    if not path.exists(): continue
    data=json.loads(path.read_text(encoding="utf-8"))
    validate_checkpoint(data,scope,PROVENANCE)
    tests=data["tests"]
    old=HISTORICAL[scope["baseline_file"]]["tests"]
    counts=collections.Counter(record["status"] for record in tests.values())
    counts["files"]=len(tests)
    counts["subtests_passed"]=sum(r.get("subtests_passed",0) for r in tests.values())
    counts["subtests_total"]=sum(r.get("subtests_total",0) for r in tests.values())
    counts["external_timeout"]=sum(r.get("reason")=="hang-killed" for r in tests.values())
    counts["worker_crashed"]=sum(r.get("reason")=="worker-crashed" for r in tests.values())
    for r in tests.values():
        if "reason" in r: reasons[r["status"]+":"+r["reason"]]+=1
        substatuses.update(sub.get("status","unknown") for sub in r.get("subtests",[]))
    for name, record in tests.items():
        group = historical_counts if name in old else new_counts
        group["files"] += 1; group[record["status"]] += 1
        group["subtests_passed"] += record.get("subtests_passed",0)
        group["subtests_total"] += record.get("subtests_total",0)
        group["external_timeout"] += record.get("reason") == "hang-killed"
    all_current.update(tests);all_old.update(old)
    for name in tests.keys() & old.keys(): changes[old[name]+" -> "+tests[name]["status"]]+=1
    rows.append({"subset":scope["subset"],"counts":dict(counts),"result_sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"added_files":sorted(tests.keys()-old.keys()),"removed_files":sorted(old.keys()-tests.keys())})
    totals.update(counts)
summary={"historical_membership_current_results":dict(historical_counts),"newly_discovered_variant_results":dict(new_counts),"completed_shards":len(rows),"totals":dict(totals),"reasons":dict(reasons.most_common()),"subtest_statuses":dict(substatuses),"file_status_transitions_from_historical_boa":dict(changes),"unique_current_files":len(all_current),"unique_historical_files":len(all_old),"rows":rows}
(ROOT/"summary.json").write_text(json.dumps(summary,indent=2)+"\n",encoding="utf-8")
lines=["| Directory | Files | Pass | Fail | Error | No results | Skip | Timeouts (error subset) | Subtests passed / total |", "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
for row in rows+[{"subset":"TOTAL","counts":totals}]:
 c=collections.Counter(row["counts"]);lines.append(f"| {row['subset']} | {c['files']} | {c['pass']} | {c['fail']} | {c['error']} | {c['no-results']} | {c['skip']} | {c['external_timeout']} | {c['subtests_passed']} / {c['subtests_total']} |")
(ROOT/"summary.md").write_text("\n".join(lines)+"\n",encoding="utf-8")
print(json.dumps({k:v for k,v in summary.items() if k!='rows'},indent=2))
