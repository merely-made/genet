#![cfg(feature = "host")]

use std::time::Instant;

use murm_rtc::rtc::peer_connection::RTCPeerConnectionBuilder;

#[test]
fn changing_the_restart_offerer_preserves_the_existing_dtls_role() {
    for changed_identity in [false, true] {
        let now = Instant::now();
        let mut left = RTCPeerConnectionBuilder::new().build(now).unwrap();
        let mut right = RTCPeerConnectionBuilder::new().build(now).unwrap();
        left.create_data_channel("role", None).unwrap();
        let offer = left.create_offer(None).unwrap();
        left.set_local_description(now, offer.clone()).unwrap();
        right.set_remote_description(now, offer).unwrap();
        let answer = right.create_answer(None).unwrap();
        assert!(answer.sdp.contains("a=setup:active"));
        right.set_local_description(now, answer.clone()).unwrap();
        left.set_remote_description(now, answer).unwrap();
        // Swap who offers. ICE credentials change; the established DTLS roles do not.
        right.restart_ice();
        let offer = if changed_identity {
            let mut newcomer = RTCPeerConnectionBuilder::new().build(now).unwrap();
            newcomer.create_data_channel("role", None).unwrap();
            newcomer.create_offer(None).unwrap()
        } else {
            right.create_offer(None).unwrap()
        };
        if !changed_identity {
            right.set_local_description(now, offer.clone()).unwrap();
        }
        left.set_remote_description(now, offer).unwrap();
        let answer = left.create_answer(None).unwrap();
        assert!(
            answer.sdp.contains(if changed_identity {
                "a=setup:active"
            } else {
                "a=setup:passive"
            }),
            "re-answer changed the DTLS role"
        );
    }
}
