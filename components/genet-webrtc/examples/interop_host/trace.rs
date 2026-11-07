//! Test-only counters. Protocol log text, addresses and credentials are never emitted.
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

const NAMES: [&str; 31] = [
    "freshRequests",
    "freshResponses",
    "usernameRefusals",
    "integrityRefusals",
    "credentialsPending",
    "unknownTransactions",
    "roleConflicts",
    "checksSent",
    "pairsFailed",
    "successfulChecks",
    "nominationsSent",
    "selectedPairs",
    "invalidPairResponses",
    "sctpOrderedData",
    "sctpUnorderedData",
    "sctpForwardTsn",
    "sctpBufferDrops",
    "sctpInvalid",
    "dtlsReplayDrops",
    "dtlsDecryptErrors",
    "sctpPackets",
    "iceDataReads",
    "iceDataWrites",
    "sctpLastDataAhead",
    "sctpMaxDataAhead",
    "sctpSacksSent",
    "sctpLastAdvertisedWindow",
    "sctpSackGapBlocks",
    "iceReadRouteChanges",
    "iceWriteRouteChanges",
    "iceReadWriteRouteMatch",
];
static COUNTS: [AtomicU64; 31] = [const { AtomicU64::new(0) }; 31];
static ROUTES: Mutex<(String, String)> = Mutex::new((String::new(), String::new()));
struct Counters;
impl log::Log for Counters {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target().starts_with("rtc")
    }
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let text = record.args().to_string();
        if let Some((writing, route)) = text
            .strip_prefix("bypass ice read ")
            .map(|s| (false, s))
            .or_else(|| text.strip_prefix("Bypass ice write ").map(|s| (true, s)))
        {
            let mut routes = ROUTES.lock().unwrap();
            let old = if writing {
                &mut routes.1
            } else {
                &mut routes.0
            };
            if !old.is_empty() && old != route {
                COUNTS[if writing { 29 } else { 28 }].fetch_add(1, Ordering::Relaxed);
            }
            *old = route.to_owned();
            COUNTS[30].store(u64::from(routes.0 == routes.1), Ordering::Relaxed);
        }
        let number = |prefix: &str| {
            text.split_once(prefix)
                .and_then(|(_, tail)| tail.split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|n| n.parse::<u32>().ok())
        };
        if text.contains("DATA: tsn=") {
            if let (Some(tsn), Some(last)) = (number("tsn="), number("peer_last_tsn=")) {
                let ahead = tsn.wrapping_sub(last);
                if ahead < (1 << 31) {
                    COUNTS[23].store(ahead.into(), Ordering::Relaxed);
                    COUNTS[24].fetch_max(ahead.into(), Ordering::Relaxed);
                }
            }
        } else if text.contains("sending SACK:") {
            COUNTS[25].fetch_add(1, Ordering::Relaxed);
            if let Some(window) = number("arwnd=") {
                COUNTS[26].store(window.into(), Ordering::Relaxed);
            }
            COUNTS[27].fetch_add(text.matches("gap ack:").count() as u64, Ordering::Relaxed);
        }
        let index = if text.contains("inbound STUN (Request)") {
            Some(0)
        } else if text.contains("inbound STUN (SuccessResponse)") {
            Some(1)
        } else if text.contains("MismatchUsername") {
            Some(2)
        } else if text.to_ascii_lowercase().contains("integrity") {
            Some(3)
        } else if text.contains("remote_credentials.is_none") {
            Some(4)
        } else if text.contains("unknown TransactionID") {
            Some(5)
        } else if text.to_ascii_lowercase().contains("role conflict") {
            Some(6)
        } else if text.contains("ping STUN from") {
            Some(7)
        } else if text.contains("marking it as failed") {
            Some(8)
        } else if text.contains("Found valid candidate pair:") {
            Some(9)
        } else if text.contains("ping STUN (nominate candidate pair") {
            Some(10)
        } else if text.contains("Set selected candidate pair:") {
            Some(11)
        } else if text.contains("Success response from invalid candidate pair") {
            Some(12)
        } else if text.contains("DATA: tsn=") {
            Some(if text.contains("unordered=true") {
                14
            } else {
                13
            })
        } else if text.contains("FwdTSN:") {
            Some(15)
        } else if text.contains("receive buffer full. dropping") {
            Some(16)
        } else if text.contains("handle_inbound got err")
            || text.contains("failed validating packet")
        {
            Some(17)
        } else if text.contains("discarded duplicated packet") {
            Some(18)
        } else if text.contains("decrypt failed:") {
            Some(19)
        } else if text.contains("recving") && record.target().starts_with("rtc_sctp") {
            Some(20)
        } else if text.contains("bypass ice read") {
            Some(21)
        } else if text.contains("Bypass ice write") {
            Some(22)
        } else {
            None
        };
        if let Some(index) = index {
            COUNTS[index].fetch_add(1, Ordering::Relaxed);
        }
    }
    fn flush(&self) {}
}
pub(super) fn install() {
    static LOGGER: Counters = Counters;
    log::set_logger(&LOGGER).expect("fixture logger");
    log::set_max_level(log::LevelFilter::Trace);
}
pub(super) fn snapshot() -> serde_json::Value {
    NAMES
        .iter()
        .zip(COUNTS.iter())
        .map(|(name, value)| {
            (
                (*name).to_owned(),
                serde_json::json!(value.load(Ordering::Relaxed)),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}
