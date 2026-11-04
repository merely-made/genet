//! Test-only counters. Protocol log text, addresses and credentials are never emitted.
use std::sync::atomic::{AtomicU64, Ordering};

const NAMES: [&str; 13] = [
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
];
static COUNTS: [AtomicU64; 13] = [const { AtomicU64::new(0) }; 13];
struct Counters;
impl log::Log for Counters {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target().starts_with("rtc_ice")
    }
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let text = record.args().to_string();
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
