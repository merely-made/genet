//! Reverse a generated local SDP projection without guessing concealed routes.
//! Match the core's two latest-description slots. Observed snapshots must not
//! replace these entries, and edited public SDP must still reach core validation.
#[derive(Default)]
pub(crate) struct Descriptions {
    offer: Option<(String, String)>,
    answer: Option<(String, String)>,
}

impl Descriptions {
    pub(crate) fn remember(&mut self, kind: &str, public: String, private: String) {
        let slot = match kind {
            "offer" => &mut self.offer,
            "answer" => &mut self.answer,
            _ => return,
        };
        *slot = Some((public, private));
    }

    pub(crate) fn restore(&self, kind: &str, public: &str) -> Option<&str> {
        let slot = match kind {
            "offer" => &self.offer,
            "answer" | "pranswer" => &self.answer,
            _ => return None,
        };
        slot.as_ref()
            .filter(|(projected, _)| projected == public)
            .map(|(_, private)| private.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_latest_generated_description_of_each_kind_is_restored() {
        let mut d = Descriptions::default();
        d.remember("offer", "public offer".into(), "private offer".into());
        d.remember("answer", "public answer".into(), "private answer".into());
        assert_eq!(d.restore("offer", "public offer"), Some("private offer"));
        assert_eq!(d.restore("answer", "public answer"), Some("private answer"));
        assert_eq!(
            d.restore("pranswer", "public answer"),
            Some("private answer")
        );
        assert_eq!(d.restore("answer", "public offer"), None);
        assert_eq!(d.restore("offer", "edited public offer"), None);
        d.remember("offer", "restart offer".into(), "restart private".into());
        assert_eq!(d.restore("offer", "public offer"), None);
        assert_eq!(d.restore("offer", "restart offer"), Some("restart private"));
        assert_eq!(d.restore("answer", "public answer"), Some("private answer"));
        d.remember("rollback", "ignored".into(), "ignored".into());
        assert_eq!(d.restore("rollback", "ignored"), None);
    }
}
