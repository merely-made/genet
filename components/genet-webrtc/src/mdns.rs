//! Host-owned remote candidate resolution and success-independent observations.
use super::*;
use genet_rtc::resolve::{Lookup, is_mdns_candidate_name};
use genet_rtc::rtc::ice::candidate::unmarshal_candidate;
use std::task::Poll;

struct Entry {
    key: String,
    candidate: RTCIceCandidate,
    lookup: Option<Box<dyn Lookup>>,
}
pub(super) struct Candidates {
    resolver: Arc<dyn Resolver>,
    generation: Option<(String, String)>,
    entries: Vec<Entry>,
    cursor: usize,
    interface: IpAddr,
}
impl Candidates {
    pub fn new(resolver: Arc<dyn Resolver>, interface: IpAddr) -> Self {
        Self {
            resolver,
            generation: None,
            entries: Vec::new(),
            cursor: 0,
            interface,
        }
    }
    pub fn cancel_pending(&mut self) {
        for entry in &mut self.entries {
            entry.lookup = None;
        }
    }
    pub fn remote_description(
        &mut self,
        sdp: &str,
        settings: Settings,
        relay_only: bool,
        now: Instant,
    ) {
        let generation = ice_credentials(sdp);
        if self.generation != generation {
            self.entries.clear();
            self.cursor = 0;
            self.generation = generation;
        }
        // SDP candidates belong to their actual media section, not a synthetic
        // global mid. The core retains the original description unchanged.
        let mut section = Vec::new();
        let mut mid = None;
        let mut index: Option<u16> = None;
        for line in sdp.lines().chain(std::iter::once("m=")) {
            if line.starts_with("m=") {
                for candidate in section.drain(..) {
                    let _ = self.submit(
                        RTCIceCandidateInit {
                            candidate,
                            sdp_mid: mid.clone(),
                            sdp_mline_index: index,
                            username_fragment: self.generation.as_ref().map(|g| g.0.clone()),
                            ..Default::default()
                        },
                        settings,
                        relay_only,
                        now,
                    );
                }
                index = Some(index.map_or(0, |i| i.saturating_add(1)));
                mid = None;
            } else if let Some(value) = line.strip_prefix("a=mid:") {
                mid = Some(value.to_owned());
            } else if let Some(value) = line.strip_prefix("a=candidate:")
                && index.is_some()
                && section.len() < settings.max_remote_candidates
            {
                section.push(format!("candidate:{value}"));
            }
        }
    }
    pub fn submit(
        &mut self,
        init: RTCIceCandidateInit,
        settings: Settings,
        relay_only: bool,
        now: Instant,
    ) -> Result<(), WebRtcError> {
        if init.candidate.len() > settings.max_candidate_bytes {
            return Err(failure(
                "QuotaExceededError",
                "Remote ICE candidate byte limit reached",
            ));
        }
        let mut fields: Vec<_> = init
            .candidate
            .split_whitespace()
            .map(str::to_owned)
            .collect();
        let Some(address) = fields.get(4) else {
            return Err(failure("OperationError", "Invalid ICE candidate"));
        };
        if !address.to_ascii_lowercase().ends_with(".local") {
            return Ok(());
        }
        fields[4] = address.to_ascii_lowercase();
        let key = fields.join(" ");
        let parsed = unmarshal_candidate(key.strip_prefix("candidate:").unwrap_or(&key))
            .map_err(protocol_error)?;
        if !is_mdns_candidate_name(parsed.address()) {
            return Ok(());
        }
        if !fields[2].eq_ignore_ascii_case("udp") {
            return Ok(());
        }
        let key = parsed.marshal();
        if self.entries.iter().any(|e| e.key == key) {
            return Ok(());
        }
        if self.entries.len() >= settings.max_remote_candidates {
            return Err(failure(
                "QuotaExceededError",
                "Remote ICE candidate limit reached",
            ));
        }
        let candidate = RTCIceCandidate::from(&parsed);
        // Only direct UDP host candidates use this capability. Keep a stable
        // observation even when policy/capacity/lookup failure prevents routing.
        let eligible = candidate.typ
            == genet_rtc::rtc::peer_connection::transport::RTCIceCandidateType::Host
            && candidate.protocol.to_string() == "udp"
            && candidate.component == 1
            && candidate.port != 0;
        let lookup = if eligible && !relay_only {
            self.resolver
                .start_mdns(
                    &candidate.address,
                    settings.bind,
                    now,
                    now.checked_add(settings.mdns_timeout).unwrap_or(now),
                )
                .ok()
        } else {
            None
        };
        self.entries.push(Entry {
            key,
            candidate,
            lookup,
        });
        Ok(())
    }
    pub fn poll(&mut self, core: &mut RTCPeerConnection, now: Instant, budget: usize) {
        if self.entries.is_empty() {
            return;
        }
        for _ in 0..budget.min(self.entries.len()) {
            self.cursor %= self.entries.len();
            let entry = &mut self.entries[self.cursor];
            self.cursor += 1;
            let Some(lookup) = &mut entry.lookup else {
                continue;
            };
            if let Poll::Ready(result) = lookup.poll(now) {
                entry.lookup = None;
                if let Ok(ips) = result
                    && ips.len() == 1
                    && ips[0].is_ipv4() == self.interface.is_ipv4()
                {
                    // Family filtering is the resolver's contract; the core also
                    // checks compatibility with each actual local candidate.
                    let _ = core.add_resolved_remote_candidate(entry.candidate.clone(), ips[0]);
                }
            }
        }
    }
    pub fn project(&self, mut core: Vec<RTCIceCandidate>) -> Vec<RTCIceCandidate> {
        for entry in &self.entries {
            if !core.iter().any(|c| c.id == entry.candidate.id) {
                core.push(entry.candidate.clone());
            }
        }
        core
    }
    pub fn stats(&self, entries: &mut Vec<Value>) {
        let timestamp = entries.first().map_or(json!(0), |v| v["timestamp"].clone());
        let transport_id = entries
            .iter()
            .find(|v| v["type"] == "transport")
            .map(|v| v["id"].clone());
        for entry in &self.entries {
            let id = format!("RTCRemoteIceCandidate_{}", entry.candidate.id);
            if entries.iter().any(|v| v["id"] == id) {
                continue;
            }
            entries.push(json!({"id":id,"type":"remote-candidate","timestamp":timestamp,
                "address":null,"port":entry.candidate.port,"protocol":entry.candidate.protocol.to_string(),
                "candidateType":entry.candidate.typ.to_string()}));
            if let Some(id) = &transport_id {
                entries.last_mut().unwrap()["transportId"] = id.clone();
            }
        }
    }
}

pub(super) fn address_is_local(candidate: &str) -> bool {
    candidate
        .split_whitespace()
        .nth(4)
        .is_some_and(|a| a.to_ascii_lowercase().ends_with(".local"))
}
