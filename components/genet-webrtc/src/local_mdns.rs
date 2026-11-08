//! Local name lifetime and web-visible projection, separate from numeric ICE routes.
use super::*;
use genet_rtc::registration::{Registration, candidate_name};

/// Host-configurable local-address exposure.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum LocalAddressPolicy {
    /// Preserve numeric candidates for explicitly trusted host environments.
    #[default]
    Expose,
    /// Conceal the selected IPv4 interface behind a private per-peer UUID.
    Mdns,
}
pub(super) struct Local {
    name: String,
    address: String,
    registration: Option<Registration>,
    foundation: RefCell<Option<String>>,
    descriptions: RefCell<local_descriptions::Descriptions>,
}
impl Local {
    pub fn new(settings: Settings, relay: bool) -> Result<Option<Self>, WebRtcError> {
        if settings.local_address_policy != LocalAddressPolicy::Mdns || relay {
            return Ok(None);
        }
        let IpAddr::V4(ip) = settings.bind else {
            return Err(failure("NotSupportedError", "Local mDNS requires IPv4"));
        };
        let name = candidate_name()
            .map_err(|_| failure("OperationError", "Local candidate entropy unavailable"))?;
        // Keep projecting even when the interface cannot register. Failure must
        // not become a script-visible numeric fallback or registration oracle.
        let registration = MdnsConfig::for_interface(ip)
            .ok()
            .and_then(|c| Registration::new(&name, ip, c.prefix_len, Instant::now()).ok());
        Ok(Some(Self {
            name,
            address: ip.to_string(),
            registration,
            foundation: RefCell::new(None),
            descriptions: RefCell::default(),
        }))
    }
    pub fn poll(&mut self, now: Instant, budget: usize) {
        if let Some(r) = &mut self.registration
            && (r.poll(now, budget).is_err() || r.conflicted())
        {
            self.registration = None;
        }
    }
    pub fn close(&mut self) {
        self.registration = None;
        *self.descriptions.borrow_mut() = Default::default();
    }
    pub fn generated(&self, value: &mut Value) {
        let private = value["sdp"].as_str().unwrap_or_default().to_owned();
        self.project(value);
        if let (Some(kind), Some(public)) = (value["type"].as_str(), value["sdp"].as_str()) {
            self.descriptions
                .borrow_mut()
                .remember(kind, public.to_owned(), private);
        }
    }
    pub fn restore(&self, kind: &str, public: &str) -> String {
        if let Some(private) = self.descriptions.borrow().restore(kind, public) {
            return private.to_owned();
        }
        self.sdp(public, true)
    }
    fn candidate(&self, text: &str, internal: bool) -> String {
        let mut fields: Vec<_> = text.split_whitespace().map(str::to_owned).collect();
        if fields.len() < 8 || fields[6] != "typ" {
            return text.to_owned();
        }
        if fields[7] == "host" {
            let (from, to) = if internal {
                (&self.name, &self.address)
            } else {
                (&self.address, &self.name)
            };
            if fields[4] == *from {
                let prefixed = fields[0].starts_with("candidate:");
                let prefix = if prefixed { "candidate:" } else { "" };
                if internal {
                    if let Some(original) = self.foundation.borrow().as_ref() {
                        fields[0] = format!("{prefix}{original}");
                    }
                } else {
                    let original = fields[0]
                        .strip_prefix("candidate:")
                        .unwrap_or(&fields[0])
                        .to_owned();
                    if original.len() <= 32 {
                        self.foundation.borrow_mut().get_or_insert(original);
                    }
                    fields[0] = format!("{prefix}{}", self.name[..36].replace('-', ""));
                }
                fields[4] = to.clone();
            }
        }
        if !internal {
            for i in 8..fields.len().saturating_sub(1) {
                if fields[i] == "raddr" {
                    fields[i + 1] = "0.0.0.0".into();
                }
                if fields[i] == "rport" {
                    fields[i + 1] = "9".into();
                }
            }
        }
        fields.join(" ")
    }
    pub fn sdp(&self, text: &str, internal: bool) -> String {
        text.lines()
            .map(|line| {
                if let Some(c) = line.strip_prefix("a=candidate:") {
                    format!("a=candidate:{}\r\n", self.candidate(c, internal))
                } else if !internal && line.strip_prefix("c=IN IP4 ") == Some(self.address.as_str())
                {
                    "c=IN IP4 0.0.0.0\r\n".into()
                } else {
                    format!("{line}\r\n")
                }
            })
            .collect()
    }
    pub fn project(&self, value: &mut Value) {
        match value {
            Value::Array(a) => {
                for v in a {
                    self.project(v);
                }
            },
            Value::Object(o) => {
                if o.get("relatedAddress").is_some_and(|v| !v.is_null()) {
                    o.insert("relatedAddress".into(), json!("0.0.0.0"));
                    o.insert("relatedPort".into(), json!(9));
                }
                let host = o.get("candidateType").is_none_or(|v| v == "host");
                if host && o.contains_key("foundation") {
                    o.insert("foundation".into(), json!(self.name[..36].replace('-', "")));
                }
                for (key, v) in o {
                    if let Some(s) = v.as_str() {
                        let replacement = match key.as_str() {
                            "candidate" => Some(self.candidate(s, false)),
                            "sdp" => Some(self.sdp(s, false)),
                            "address" | "ip" if host && s == self.address => {
                                Some(self.name.clone())
                            },
                            "relatedAddress" if s == self.address => Some("0.0.0.0".into()),
                            _ => None,
                        };
                        if let Some(s) = replacement {
                            *v = json!(s);
                        }
                    } else {
                        self.project(v);
                    }
                }
            },
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn failed() -> Local {
        Local {
            name: "53caef04-4218-4fb0-815d-f91a511bb8d2.local".into(),
            address: "192.0.2.10".into(),
            registration: None,
            foundation: RefCell::new(None),
            descriptions: RefCell::default(),
        }
    }
    #[test]
    fn unavailable_interface_keeps_a_private_projection() {
        let value = Local::new(
            Settings {
                bind: "192.0.2.254".parse().unwrap(),
                local_address_policy: LocalAddressPolicy::Mdns,
                ..Settings::default()
            },
            false,
        )
        .unwrap()
        .unwrap();
        assert!(value.registration.is_none());
        let mut v = json!({"address":"192.0.2.254"});
        value.project(&mut v);
        assert!(v["address"].as_str().unwrap().ends_with(".local"));
    }
    #[test]
    fn failed_registration_still_conceals_and_roundtrips_host_route() {
        let l = failed();
        let private = "candidate:1 1 udp 100 192.0.2.10 3333 typ host";
        let public = l.candidate(private, false);
        assert!(!public.contains("192.0.2.10"));
        assert!(!public.contains("candidate:1 "));
        assert_eq!(l.candidate(&public, true), private);
        assert_eq!(
            l.candidate("candidate:1 1 udp 100 foreign.local 3333 typ host", true),
            "candidate:1 1 udp 100 foreign.local 3333 typ host"
        );
    }
    #[test]
    fn related_routes_are_sanitized_without_losing_public_route() {
        let l = failed();
        let public = l.candidate(
            "candidate:1 1 udp 100 198.51.100.20 3333 typ srflx raddr 192.0.2.10 rport 4567",
            false,
        );
        assert!(public.contains("198.51.100.20 3333"));
        assert!(public.ends_with("raddr 0.0.0.0 rport 9"));
        let mut stats =
            json!({"address":"192.0.2.10","relatedAddress":"192.0.2.10","relatedPort":4567});
        l.project(&mut stats);
        assert_eq!(stats["address"], l.name);
        assert_eq!(stats["relatedPort"], 9);
    }
}

#[cfg(test)]
mod description_tests {
    use super::*;
    #[test]
    fn unchanged_generated_sdp_restores_concealed_related_routes() {
        let l = Local {
            name: "53caef04-4218-4fb0-815d-f91a511bb8d2.local".into(),
            address: "192.0.2.10".into(),
            registration: None,
            foundation: RefCell::new(None),
            descriptions: RefCell::default(),
        };
        let private = "v=0\r\nc=IN IP4 192.0.2.10\r\na=ice-ufrag:original\r\na=candidate:1 1 udp 100 192.0.2.10 3333 typ host\r\na=candidate:2 1 udp 90 198.51.100.20 4444 typ srflx raddr 192.0.2.10 rport 3333\r\na=candidate:3 1 udp 80 203.0.113.20 5555 typ relay raddr 198.51.100.20 rport 4444\r\n";
        let mut description = json!({"type":"offer","sdp":private});
        l.generated(&mut description);
        let public = description["sdp"].as_str().unwrap();
        assert!(!public.contains("192.0.2.10"));
        assert_eq!(l.restore("offer", public), private);
        // Snapshots may acquire candidates after generation. Their projection
        // must not overwrite the canonical generated offer.
        let mut snapshot =
            json!({"type":"offer","sdp":format!("{private}a=end-of-candidates\r\n")});
        l.project(&mut snapshot);
        assert_eq!(l.restore("offer", public), private);
        let edited = public.replace("ice-ufrag:original", "ice-ufrag:edited");
        assert!(l.restore("offer", &edited).contains("ice-ufrag:edited"));
        assert_ne!(l.restore("offer", &edited), private);
        let mut l = l;
        l.close();
        assert_ne!(l.restore("offer", public), private);
    }
    #[test]
    fn description_roundtrip_keeps_numeric_routes_internal() {
        let l = Local {
            name: "53caef04-4218-4fb0-815d-f91a511bb8d2.local".into(),
            address: "192.0.2.10".into(),
            registration: None,
            foundation: RefCell::new(None),
            descriptions: RefCell::default(),
        };
        let private = "m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\nc=IN IP4 192.0.2.10\r\na=candidate:1 1 udp 100 192.0.2.10 3333 typ host\r\n";
        let public = l.sdp(private, false);
        assert!(!public.contains("192.0.2.10"));
        assert!(!public.contains("candidate:1 "));
        assert!(l.sdp(&public, true).contains("192.0.2.10 3333 typ host"));
        let mut stats = json!({"candidateType":"srflx","address":"192.0.2.10","relatedAddress":"192.0.2.10","relatedPort":3333});
        l.project(&mut stats);
        assert_eq!(stats["address"], "192.0.2.10");
        assert_eq!(stats["relatedAddress"], "0.0.0.0");
        assert_eq!(stats["relatedPort"], 9);
    }
}
