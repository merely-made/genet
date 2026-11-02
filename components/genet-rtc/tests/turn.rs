//! Controlled-server qualification. Start tests/coturn.sh in WSL, then set
//! MURM_RTC_TEST_SERVER to its reachable numeric IP:port on the Windows host.
#![cfg(feature = "host")]

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use murm_rtc::rtc::crypto::default_provider;
use murm_rtc::rtc::ice::url::Url;
use murm_rtc::rtc::peer_connection::RTCPeerConnectionBuilder;
use murm_rtc::rtc::peer_connection::configuration::setting_engine::SettingEngine;
use murm_rtc::rtc::peer_connection::event::{RTCDataChannelEvent, RTCPeerConnectionEvent};
use murm_rtc::rtc::peer_connection::message::RTCMessage;
use murm_rtc::udp::{DiscoveryConfig, IoLimits, Output, ReadInterest, Server, UdpPeer};

fn server() -> SocketAddr {
    std::env::var("MURM_RTC_TEST_SERVER")
        .expect("controlled coturn endpoint")
        .parse()
        .unwrap()
}

fn peer(turn: bool, password: &str, timeout: Duration) -> UdpPeer {
    peer_at(server(), turn, password, timeout)
}

fn peer_at(server: SocketAddr, turn: bool, password: &str, timeout: Duration) -> UdpPeer {
    let probe = UdpSocket::bind("0.0.0.0:0").unwrap();
    probe.connect(server).unwrap();
    let bind = SocketAddr::new(probe.local_addr().unwrap().ip(), 0);
    let crypto = default_provider().unwrap();
    let mut settings = SettingEngine::default();
    settings.set_crypto_provider(crypto.clone());
    let core = RTCPeerConnectionBuilder::new()
        .with_setting_engine(settings)
        .build(Instant::now())
        .unwrap();
    let mut url =
        Url::parse_url(&format!("{}:{server}", if turn { "turn" } else { "stun" })).unwrap();
    url.username = "rtc-test".into();
    url.password = password.into();
    UdpPeer::bind_host_with_discovery(
        core,
        bind,
        IoLimits {
            datagram_bytes: 65_535,
            packets_per_turn: 8,
            timer_floor: Duration::from_millis(1),
        },
        DiscoveryConfig {
            servers: vec![Server {
                url,
                address: server,
            }],
            relay_only: turn,
            timeout,
            allocation_refresh_interval_cap: Some(Duration::from_secs(1)),
            crypto_provider: crypto,
        },
    )
    .unwrap()
}

#[test]
fn unavailable_relay_has_a_bounded_gathering_deadline_and_no_direct_candidate() {
    let blackhole = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut peer = peer_at(
        blackhole.local_addr().unwrap(),
        true,
        "unused",
        Duration::from_millis(40),
    );
    peer.core_mut()
        .create_data_channel("unreachable", None)
        .unwrap();
    let offer = peer.core_mut().create_offer(None).unwrap();
    peer.core_mut()
        .set_local_description(Instant::now(), offer)
        .unwrap();
    peer.finish_host_candidates().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut errors = 0;
    loop {
        match peer.poll(deadline, ReadInterest::All).unwrap() {
            Some(Output::GatheringError { code, .. }) => {
                assert_eq!(code, 701);
                errors += 1;
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceCandidateEvent(_))) => {
                panic!("direct candidate leaked")
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceGatheringStateChangeEvent(state)))
                if state.to_string() == "complete" =>
            {
                break;
            }
            Some(Output::Deadline | Output::Closed) => panic!("gathering did not settle"),
            _ => std::thread::sleep(Duration::from_millis(1)),
        }
    }
    assert_eq!(errors, 1);
    let local = peer.local_addr();
    drop(peer);
    UdpSocket::bind(local).expect("failed discovery releases its socket");
}

fn gather(
    peer: &mut UdpPeer,
) -> Vec<murm_rtc::rtc::peer_connection::transport::RTCIceCandidateInit> {
    peer.finish_host_candidates().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut candidates = Vec::new();
    loop {
        match peer.poll(deadline, ReadInterest::All).unwrap() {
            Some(Output::Event(RTCPeerConnectionEvent::OnIceGatheringStateChangeEvent(state)))
                if state.to_string() == "complete" =>
            {
                return candidates;
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceCandidateEvent(event))) => {
                let candidate = event.candidate.to_json().unwrap();
                if candidate.candidate.contains(" typ relay ")
                    || candidate.candidate.contains(" typ srflx ")
                {
                    assert!(!event.url.is_empty(), "local server metadata lost");
                    assert!(
                        candidate.url.is_none(),
                        "local metadata must stay out of signaling"
                    );
                }
                if candidate.candidate.is_empty() {
                    return candidates;
                }
                candidates.push(candidate);
            }
            Some(Output::GatheringError { .. }) => panic!("controlled server gathering failed"),
            Some(Output::Deadline | Output::Closed) => panic!("gathering stopped"),
            _ => std::thread::sleep(Duration::from_millis(1)),
        }
    }
}

#[test]
#[ignore = "requires controlled coturn; see tests/coturn.sh"]
fn stun_reflexive_candidate() {
    let mut peer = peer(false, "rtc-test-password", Duration::from_secs(3));
    peer.core_mut().create_data_channel("stun", None).unwrap();
    let offer = peer.core_mut().create_offer(None).unwrap();
    peer.core_mut()
        .set_local_description(Instant::now(), offer)
        .unwrap();
    let candidates = gather(&mut peer);
    assert!(
        candidates
            .iter()
            .any(|c| c.candidate.contains(" typ srflx "))
    );
    let local = peer.local_addr();
    drop(peer);
    UdpSocket::bind(local).expect("STUN socket released");
}

#[test]
#[ignore = "requires controlled coturn; see tests/coturn.sh"]
fn forced_relay_encrypted_fragmentation_refresh_and_release() {
    let mut left = peer(true, "rtc-test-password", Duration::from_secs(3));
    let mut right = peer(true, "rtc-test-password", Duration::from_secs(3));
    let id = left
        .core_mut()
        .create_data_channel("relay", None)
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
    let a = gather(&mut left);
    let b = gather(&mut right);
    assert!(!a.is_empty() && !b.is_empty());
    assert!(
        a.iter()
            .chain(&b)
            .all(|c| c.candidate.contains(" typ relay ")
                && c.candidate.contains(" raddr 0.0.0.0 rport 0"))
    );
    for candidate in a {
        right.core_mut().add_remote_candidate(candidate).unwrap();
    }
    for candidate in b {
        left.core_mut().add_remote_candidate(candidate).unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let send_after = Instant::now() + Duration::from_millis(2200);
    let payload = vec![0x5a; 48_000];
    let mut open = false;
    let mut sent = false;
    let mut received = false;
    loop {
        for (peer, receiving) in [(&mut left, false), (&mut right, true)] {
            match peer.poll(deadline, ReadInterest::All).unwrap() {
                Some(Output::Event(RTCPeerConnectionEvent::OnDataChannel(
                    RTCDataChannelEvent::OnOpen(_),
                ))) if !receiving => open = true,
                Some(Output::Message(message)) if receiving => {
                    if let RTCMessage::DataChannelMessage(_, message) = message.message {
                        assert!(!message.is_string);
                        assert_eq!(message.data.as_ref(), payload);
                        received = true;
                    }
                }
                Some(Output::GatheringError { .. }) => panic!("relay gathering failed"),
                Some(Output::Deadline | Output::Closed) => {
                    panic!("relay exchange stopped; open={open} sent={sent}")
                }
                _ => {}
            }
        }
        if open && !sent && Instant::now() >= send_after {
            left.core_mut()
                .data_channel(id)
                .unwrap()
                .send(Instant::now(), bytes::BytesMut::from(payload.as_slice()))
                .unwrap();
            sent = true;
        }
        if received {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let a = left.local_addr();
    let b = right.local_addr();
    left.close().unwrap();
    drop(left);
    drop(right);
    UdpSocket::bind(a).expect("closed sender socket released");
    UdpSocket::bind(b).expect("dropped receiver socket released");
}

#[test]
#[ignore = "requires controlled coturn; see tests/coturn.sh"]
fn relay_authentication_failure_completes_without_direct_candidate() {
    let mut peer = peer(true, "incorrect-password", Duration::from_millis(700));
    peer.core_mut()
        .create_data_channel("refused", None)
        .unwrap();
    let offer = peer.core_mut().create_offer(None).unwrap();
    peer.core_mut()
        .set_local_description(Instant::now(), offer)
        .unwrap();
    peer.finish_host_candidates().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut errors = 0;
    loop {
        match peer.poll(deadline, ReadInterest::All).unwrap() {
            Some(Output::Event(RTCPeerConnectionEvent::OnIceGatheringStateChangeEvent(state)))
                if state.to_string() == "complete" =>
            {
                break;
            }
            Some(Output::GatheringError { url, code }) => {
                assert_eq!(code, 701);
                assert!(!url.contains("password"));
                errors += 1;
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceCandidateEvent(event))) => {
                assert!(event.candidate.to_json().unwrap().candidate.is_empty());
                break;
            }
            Some(Output::Deadline | Output::Closed) => panic!("failure did not finish gathering"),
            _ => std::thread::sleep(Duration::from_millis(1)),
        }
    }
    assert_eq!(errors, 1);
}
