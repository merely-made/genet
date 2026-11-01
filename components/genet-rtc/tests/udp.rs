#![cfg(feature = "udp")]

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use bytes::BytesMut;
use murm_rtc::rtc::data_channel::RTCDataChannelInit;
use murm_rtc::rtc::peer_connection::RTCPeerConnectionBuilder;
use murm_rtc::rtc::peer_connection::event::{RTCDataChannelEvent, RTCPeerConnectionEvent};
use murm_rtc::rtc::peer_connection::message::RTCMessage;
use murm_rtc::udp::{Error, IoLimits, Output, ReadInterest, UdpPeer};
use sha2::{Digest, Sha256};

fn limits() -> IoLimits {
    IoLimits {
        datagram_bytes: 65_535,
        packets_per_turn: 8,
        timer_floor: Duration::from_millis(1),
    }
}

async fn peer() -> UdpPeer {
    let core = RTCPeerConnectionBuilder::new()
        .build(Instant::now())
        .unwrap();
    let mut peer = UdpPeer::bind(core, "127.0.0.1:0".parse().unwrap(), limits())
        .await
        .unwrap();
    peer.finish_host_candidates().unwrap();
    peer
}

fn fingerprint(sdp: &str) -> String {
    sdp.lines()
        .find_map(|line| line.strip_prefix("a=fingerprint:sha-256 "))
        .unwrap()
        .to_string()
}

fn certificate_fingerprint(peer: &UdpPeer) -> String {
    let sctp = peer.core().sctp().unwrap();
    let dtls = sctp.transport();
    let certificate = dtls.get_remote_certificates().first().unwrap();
    Sha256::digest(certificate)
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":")
}

#[tokio::test]
async fn encrypted_udp_preserves_multiple_channel_modes_and_message_boundaries() {
    let mut left = peer().await;
    let mut right = peer().await;
    let ordered = left
        .core_mut()
        .create_data_channel("ordered", None)
        .unwrap()
        .id();
    let unordered = left
        .core_mut()
        .create_data_channel(
            "unordered",
            Some(RTCDataChannelInit {
                ordered: false,
                max_retransmits: Some(0),
                ..Default::default()
            }),
        )
        .unwrap()
        .id();
    let now = Instant::now();
    let offer = left.core_mut().create_offer(None).unwrap();
    left.core_mut()
        .set_local_description(now, offer.clone())
        .unwrap();
    right
        .core_mut()
        .set_remote_description(now, offer.clone())
        .unwrap();
    let answer = right.core_mut().create_answer(None).unwrap();
    right
        .core_mut()
        .set_local_description(now, answer.clone())
        .unwrap();
    left.core_mut()
        .set_remote_description(now, answer.clone())
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut opened_left = Vec::new();
    let mut opened_right = Vec::new();
    let mut received_right = Vec::new();
    let mut received_left = Vec::new();
    let mut sent = false;
    let binary: Vec<u8> = (0..48_000).map(|n| (n % 251) as u8).collect();

    while received_right.len() < 3 || received_left.is_empty() {
        // Keep receiving packets and sending the reverse reply while the
        // right application's data queue is paused. Resume it only once the
        // reply crosses the wire, without a timing/performance assumption.
        let right_interest = if received_left.is_empty() {
            ReadInterest::MediaOnly
        } else {
            ReadInterest::All
        };
        tokio::select! {
            output = left.next(deadline) => match output.unwrap() {
                Output::Event(RTCPeerConnectionEvent::OnDataChannel(RTCDataChannelEvent::OnOpen(id))) => opened_left.push(id),
                Output::Message(message) => {
                    if let RTCMessage::DataChannelMessage(id, data) = message.message {
                        assert!(received_right.is_empty(), "paused application messages were drained");
                        received_left.push((id, data));
                    }
                }
                Output::Deadline | Output::Closed => panic!("left stopped before the data exchange finished"),
                _ => {}
            },
            output = right.next_with_interest(deadline, right_interest) => match output.unwrap() {
                Output::Event(RTCPeerConnectionEvent::OnDataChannel(RTCDataChannelEvent::OnOpen(id))) => opened_right.push(id),
                Output::Message(message) => {
                    if let RTCMessage::DataChannelMessage(id, data) = message.message {
                        let label = right.core_mut().data_channel(id).unwrap().label().to_owned();
                        received_right.push((label, data));
                    }
                }
                Output::Deadline | Output::Closed => panic!("right stopped before the data exchange finished"),
                _ => {}
            },
        }
        if !sent && opened_left.len() == 2 && opened_right.len() == 2 {
            let now = Instant::now();
            let mut channel = left.core_mut().data_channel(ordered).unwrap();
            channel.send_text(now, "héllo").unwrap();
            channel
                .send(now, BytesMut::from(binary.as_slice()))
                .unwrap();
            left.core_mut()
                .data_channel(unordered)
                .unwrap()
                .send_text(now, "datagram mode")
                .unwrap();
            for &id in &opened_right {
                let mut channel = right.core_mut().data_channel(id).unwrap();
                if channel.label() == "ordered" {
                    assert!(channel.ordered());
                    channel.send_text(now, "reply").unwrap();
                } else {
                    assert!(!channel.ordered());
                    assert_eq!(channel.max_retransmits(), Some(0));
                }
            }
            sent = true;
        }
    }
    assert_eq!(received_left.len(), 1);
    assert_eq!(received_left[0].0, ordered);
    assert!(received_left[0].1.is_string);
    assert_eq!(received_left[0].1.data.as_ref(), b"reply");
    let ordered_messages: Vec<_> = received_right
        .iter()
        .filter(|(label, _)| label == "ordered")
        .collect();
    assert_eq!(ordered_messages.len(), 2);
    assert!(ordered_messages[0].1.is_string);
    assert_eq!(ordered_messages[0].1.data.as_ref(), "héllo".as_bytes());
    assert!(!ordered_messages[1].1.is_string);
    assert_eq!(ordered_messages[1].1.data.as_ref(), binary.as_slice());
    let unordered_message = received_right
        .iter()
        .find(|(label, _)| label == "unordered")
        .unwrap();
    assert_eq!(unordered_message.1.data.as_ref(), b"datagram mode");

    // The certificate from the handshake, not a copied SDP field, is exposed.
    assert_eq!(certificate_fingerprint(&left), fingerprint(&answer.sdp));
    assert_eq!(certificate_fingerprint(&right), fingerprint(&offer.sdp));

    left.close().unwrap();
    let close_deadline = Instant::now() + Duration::from_secs(2);
    let mut left_closed = false;
    let mut right_closed = false;
    while !left_closed || !right_closed {
        tokio::select! {
            output = left.next(close_deadline), if !left_closed => match output.unwrap() {
                Output::Closed => left_closed = true,
                Output::Deadline => panic!("local graceful close did not finish"),
                _ => {}
            },
            output = right.next(close_deadline), if !right_closed => match output.unwrap() {
                Output::Closed => right_closed = true,
                Output::Deadline => panic!("remote did not observe DTLS close"),
                _ => {}
            },
        }
    }
}

#[tokio::test]
async fn cancellation_and_drop_release_the_socket_without_a_background_owner() {
    let mut peer = peer().await;
    let address = peer.local_addr();
    // Drive past the finite gathering notifications, then cancel while waiting.
    let outer = tokio::time::timeout(Duration::from_millis(25), async {
        loop {
            peer.next(Instant::now() + Duration::from_secs(1))
                .await
                .unwrap();
        }
    })
    .await;
    assert!(outer.is_err());
    drop(peer);
    let rebound = tokio::net::UdpSocket::bind(address).await.unwrap();
    assert_eq!(rebound.local_addr().unwrap(), address);
}

#[tokio::test]
async fn oversize_input_is_refused_before_protocol_parsing() {
    let core = RTCPeerConnectionBuilder::new()
        .build(Instant::now())
        .unwrap();
    let mut small = limits();
    small.datagram_bytes = 32;
    let mut peer = UdpPeer::bind(core, "127.0.0.1:0".parse().unwrap(), small)
        .await
        .unwrap();
    let sender = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    sender.send_to(&[1; 64], peer.local_addr()).await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match peer.next(deadline).await {
            Err(Error::DatagramLimit { actual, limit }) => {
                assert!(actual > limit);
                assert_eq!(limit, 32);
                break;
            }
            Ok(Output::Deadline) => panic!("oversize packet was not observed"),
            Ok(_) => {}
            Err(error) => panic!("wrong error: {error}"),
        }
    }
}

#[tokio::test]
async fn unspecified_routes_and_zero_budgets_are_refused() {
    for (address, packets) in [("0.0.0.0:0", 8), ("127.0.0.1:0", 0)] {
        let core = RTCPeerConnectionBuilder::new()
            .build(Instant::now())
            .unwrap();
        let mut config = limits();
        config.packets_per_turn = packets;
        assert!(matches!(
            UdpPeer::bind(core, address.parse::<SocketAddr>().unwrap(), config).await,
            Err(Error::Configuration(_))
        ));
    }
}
