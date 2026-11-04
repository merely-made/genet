use super::*;

pub(super) fn report(state: &mut State, now: Instant) -> Value {
    let report = state
        .endpoint
        .core_mut()
        .get_stats(now, StatsSelector::None);
    let mut entries: Vec<Value> = report
        .iter()
        .filter_map(|entry| match entry {
            RTCStatsReportEntry::PeerConnection(v) => serde_json::to_value(v).ok(),
            RTCStatsReportEntry::Transport(v) => serde_json::to_value(v).ok(),
            RTCStatsReportEntry::LocalCandidate(v) | RTCStatsReportEntry::RemoteCandidate(v) => {
                serde_json::to_value(v).ok()
            },
            RTCStatsReportEntry::IceCandidatePair(v) => serde_json::to_value(v).ok(),
            RTCStatsReportEntry::Certificate(v) => serde_json::to_value(v).ok(),
            RTCStatsReportEntry::DataChannel(v) => serde_json::to_value(v).ok(),
            _ => None,
        })
        .collect();
    add_missing_candidates(state, &mut entries);
    normalize_references(&mut entries);
    json!(entries)
}

fn add_missing_candidates(state: &State, entries: &mut Vec<Value>) {
    // Peer-reflexive candidates can be discovered inside ICE,
    // without passing through the core's stats registration hook.
    // Describe those real candidates before repairing references.
    if let Some(sctp) = state.endpoint.core().sctp() {
        let ice = sctp.transport().ice_transport();
        let mut local_candidates = ice.get_local_candidates();
        let mut remote_candidates = ice.get_remote_candidates();
        // The previous nominated path remains real while a restart
        // is checked, although it is outside the new candidate lists.
        if let Some(pair) = ice.get_selected_candidate_pair() {
            if !local_candidates.iter().any(|c| c.id == pair.local().id) {
                local_candidates.push(pair.local().clone());
            }
            if !remote_candidates.iter().any(|c| c.id == pair.remote().id) {
                remote_candidates.push(pair.remote().clone());
            }
        }
        let pairs: Vec<Value> = entries
            .iter()
            .filter(|v| v["type"] == "candidate-pair")
            .cloned()
            .collect();
        for (local, candidates, field, prefix) in [
            (
                true,
                local_candidates,
                "localCandidateId",
                "RTCLocalIceCandidate_",
            ),
            (
                false,
                remote_candidates,
                "remoteCandidateId",
                "RTCRemoteIceCandidate_",
            ),
        ] {
            for pair in &pairs {
                let Some(id) = pair[field].as_str() else {
                    continue;
                };
                let raw = id.strip_prefix(prefix).unwrap_or(id);
                let registered = format!("{prefix}{raw}");
                if entries
                    .iter()
                    .any(|v| v["id"] == id || v["id"] == registered)
                {
                    continue;
                }
                if let Some(candidate) = candidates.iter().find(|c| c.id == raw) {
                    entries.push(json!({
                        "id":registered,"type":if local {"local-candidate"} else {"remote-candidate"},
                        "timestamp":pair["timestamp"],"transportId":pair["transportId"],
                        "address":candidate.address,"port":candidate.port,
                        "protocol":candidate.protocol.to_string(),"candidateType":candidate.typ.to_string(),
                        "priority":candidate.priority,"foundation":candidate.foundation,
                    }));
                }
            }
        }
    }
}

fn normalize_references(entries: &mut Vec<Value>) {
    // The current core registers namespaced candidate statistics
    // but its pair accumulator still refers to raw candidate IDs.
    // Every web-visible reference must identify an actual map row.
    let ids: std::collections::BTreeSet<String> = entries
        .iter()
        .filter_map(|v| v["id"].as_str().map(str::to_owned))
        .collect();
    for entry in entries.iter_mut() {
        // W3C RTCIceCandidateStats defaults remote addresses to
        // null. Keep this conservative even for signaled addresses;
        // remote server URLs and relay protocols are never exposed.
        if entry["type"] == "remote-candidate"
            && let Some(object) = entry.as_object_mut()
        {
            object.insert("address".into(), Value::Null);
            object.remove("url");
            object.remove("relayProtocol");
        }
        if entry["type"] != "candidate-pair" {
            continue;
        }
        for (field, prefix) in [
            ("localCandidateId", "RTCLocalIceCandidate_"),
            ("remoteCandidateId", "RTCRemoteIceCandidate_"),
        ] {
            if let Some(raw) = entry[field].as_str()
                && !ids.contains(raw)
            {
                let registered = format!("{prefix}{raw}");
                if ids.contains(&registered) {
                    entry[field] = json!(registered);
                }
            }
        }
    }
    // The core may retain historical pair accumulators after their
    // peer-reflexive candidates have retired. Omit unresolvable
    // history instead of manufacturing candidate observations.
    entries.retain(|entry| {
        entry["type"] != "candidate-pair"
            || ["localCandidateId", "remoteCandidateId"]
                .iter()
                .all(|field| entry[field].as_str().is_some_and(|id| ids.contains(id)))
    });
    let pairs: std::collections::BTreeSet<String> = entries
        .iter()
        .filter(|entry| entry["type"] == "candidate-pair")
        .filter_map(|entry| entry["id"].as_str().map(str::to_owned))
        .collect();
    for entry in entries.iter_mut() {
        if entry["type"] == "transport"
            && entry["selectedCandidatePairId"]
                .as_str()
                .is_some_and(|id| !pairs.contains(id))
            && let Some(object) = entry.as_object_mut()
        {
            object.remove("selectedCandidatePairId");
        }
    }
}
