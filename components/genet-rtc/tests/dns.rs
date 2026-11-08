//! Synchronous resolver failure must remain observable before gathering ends.
#![cfg(feature = "host")]
use murm_rtc::resolve::{Family, Lookup, ResolveError, Resolver};
use murm_rtc::rtc::crypto::default_provider;
use murm_rtc::rtc::ice::url::Url;
use murm_rtc::rtc::peer_connection::RTCPeerConnectionBuilder;
use murm_rtc::rtc::peer_connection::configuration::setting_engine::SettingEngine;
use murm_rtc::rtc::peer_connection::event::RTCPeerConnectionEvent;
use murm_rtc::udp::{DiscoveryConfig, IoLimits, Output, ReadInterest, UdpPeer};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Unavailable;
impl Resolver for Unavailable {
    fn start(
        &self,
        _: &str,
        _: Family,
        _: Instant,
        _: Instant,
    ) -> Result<Box<dyn Lookup>, ResolveError> {
        Err(ResolveError::Unavailable)
    }
}

#[test]
fn immediate_resolution_refusal_precedes_gathering_completion_without_a_route() {
    let crypto = default_provider().unwrap();
    let mut settings = SettingEngine::default();
    settings.set_crypto_provider(crypto.clone());
    let core = RTCPeerConnectionBuilder::new()
        .with_setting_engine(settings)
        .build(Instant::now())
        .unwrap();
    let mut peer = UdpPeer::bind_host_with_discovery(
        core,
        "127.0.0.1:0".parse().unwrap(),
        IoLimits {
            datagram_bytes: 65535,
            packets_per_turn: 8,
            timer_floor: Duration::from_millis(1),
        },
        DiscoveryConfig {
            servers: Vec::new(),
            named_servers: vec![Url::parse_url("turn:rtc.example.test:3478").unwrap().into()],
            resolver: Some(Arc::new(Unavailable)),
            relay_only: true,
            timeout: Duration::from_secs(1),
            allocation_refresh_interval_cap: None,
            crypto_provider: crypto,
        },
    )
    .unwrap();
    peer.core_mut()
        .create_data_channel("refused", None)
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
            Some(Output::GatheringError { url, code }) => {
                assert_eq!(url, "turn:rtc.example.test:3478?transport=udp");
                assert_eq!(code, 701);
                errors += 1;
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceGatheringStateChangeEvent(state)))
                if state.to_string() == "complete" =>
            {
                assert_eq!(errors, 1, "completion overtook the terminal resolver error");
                break;
            }
            Some(Output::Event(RTCPeerConnectionEvent::OnIceCandidateEvent(_))) => {
                panic!("resolver failure invented a route")
            }
            Some(Output::Deadline | Output::Closed) => panic!("gathering failed to settle"),
            _ => {}
        }
    }
}
