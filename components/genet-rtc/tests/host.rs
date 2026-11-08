#![cfg(feature = "host")]

use murm_rtc::rtc::peer_connection::RTCPeerConnectionBuilder;
use murm_rtc::rtc::peer_connection::event::{RTCDataChannelEvent, RTCPeerConnectionEvent};
use murm_rtc::rtc::peer_connection::message::RTCMessage;
use murm_rtc::udp::{IoLimits, Output, ReadInterest, UdpPeer};
use std::time::{Duration, Instant};

fn peer() -> UdpPeer {
    let core = RTCPeerConnectionBuilder::new()
        .build(Instant::now())
        .unwrap();
    let mut peer = UdpPeer::bind_host(
        core,
        "127.0.0.1:0".parse().unwrap(),
        IoLimits {
            datagram_bytes: 65_535,
            packets_per_turn: 8,
            timer_floor: Duration::from_millis(1),
        },
    )
    .unwrap();
    peer.finish_host_candidates().unwrap();
    peer
}

#[test]
fn host_drives_encrypted_data_without_an_async_runtime() {
    let mut left = peer();
    let mut right = peer();
    let channel = left
        .core_mut()
        .create_data_channel("host", None)
        .unwrap()
        .id();
    let now = Instant::now();
    let offer = left.core_mut().create_offer(None).unwrap();
    left.core_mut()
        .set_local_description(now, offer.clone())
        .unwrap();
    right.core_mut().set_remote_description(now, offer).unwrap();
    let answer = right.core_mut().create_answer(None).unwrap();
    right
        .core_mut()
        .set_local_description(now, answer.clone())
        .unwrap();
    left.core_mut().set_remote_description(now, answer).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut sent = false;
    loop {
        if let Some(output) = left.poll(deadline, ReadInterest::All).unwrap() {
            match output {
                Output::Event(RTCPeerConnectionEvent::OnDataChannel(
                    RTCDataChannelEvent::OnOpen(id),
                )) if id == channel => {
                    left.core_mut()
                        .data_channel(id)
                        .unwrap()
                        .send_text(Instant::now(), "host-driven")
                        .unwrap();
                    sent = true;
                }
                Output::Deadline | Output::Closed => panic!("host peer stopped before delivery"),
                _ => {}
            }
        }
        if let Some(output) = right.poll(deadline, ReadInterest::All).unwrap() {
            match output {
                Output::Message(message) => {
                    if let RTCMessage::DataChannelMessage(_, message) = message.message {
                        assert!(sent);
                        assert!(message.is_string);
                        assert_eq!(message.data.as_ref(), b"host-driven");
                        break;
                    }
                }
                Output::Deadline | Output::Closed => {
                    panic!("receiving host peer stopped before delivery")
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    left.close().unwrap();
    right.close().unwrap();
}
