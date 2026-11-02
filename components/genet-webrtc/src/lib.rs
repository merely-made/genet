//! Native WebRTC host for Genet's engine-neutral browser bindings.
//! The caller pumps I/O. No runtime, worker thread, application identity or
//! invitation is created by this adapter.

#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use murm_rtc::rtc::crypto::{RTCCryptoProvider, default_provider};
use murm_rtc::rtc::data_channel::RTCDataChannelInit;
use murm_rtc::rtc::ice::url::{ProtoType, SchemeType, Url};
use murm_rtc::rtc::peer_connection::configuration::setting_engine::SettingEngine;
use murm_rtc::rtc::peer_connection::configuration::{
    RTCBundlePolicy, RTCConfiguration, RTCConfigurationBuilder, RTCIceTransportPolicy,
};
use murm_rtc::rtc::peer_connection::event::{RTCDataChannelEvent, RTCPeerConnectionEvent};
use murm_rtc::rtc::peer_connection::message::RTCMessage;
use murm_rtc::rtc::peer_connection::sdp::RTCSessionDescription;
use murm_rtc::rtc::peer_connection::transport::{
    RTCIceCandidate, RTCIceCandidateInit, RTCIceServer,
};
use murm_rtc::rtc::peer_connection::{RTCPeerConnection, RTCPeerConnectionBuilder};
use murm_rtc::rtc::sansio::Protocol;
use murm_rtc::rtc::statistics::StatsSelector;
use murm_rtc::rtc::statistics::report::RTCStatsReportEntry;
use murm_rtc::udp::{DiscoveryConfig, IoLimits, Output, ReadInterest, Server, UdpPeer};
use script_runtime_api::{WebRtcError, WebRtcFactory, WebRtcOperation, WebRtcPeer};
use serde_json::{Value, json};

/// Network placement and resource budgets supplied by the document host.
#[derive(Clone, Copy)]
pub struct Settings {
    pub bind: IpAddr,
    pub io: IoLimits,
    pub max_peers: usize,
    pub max_channels: usize,
    pub max_notifications: usize,
    pub poll_interval: Duration,
    pub gathering_timeout: Duration,
    pub allocation_refresh_interval_cap: Option<Duration>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bind: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            io: IoLimits {
                datagram_bytes: 65_535,
                packets_per_turn: 8,
                timer_floor: Duration::from_millis(1),
            },
            max_peers: 64,
            max_channels: 256,
            max_notifications: 256,
            poll_interval: Duration::from_millis(1),
            gathering_timeout: Duration::from_secs(8),
            allocation_refresh_interval_cap: None,
        }
    }
}

/// Native peer factory; sharing it between realms shares its resource ceiling.
pub struct NativeWebRtcFactory {
    settings: Settings,
    live: Rc<Cell<usize>>,
}

impl NativeWebRtcFactory {
    pub fn new(settings: Settings) -> Result<Self, WebRtcError> {
        if settings.bind.is_unspecified()
            || settings.max_peers == 0
            || settings.max_channels == 0
            || settings.max_notifications < 4
            || settings.poll_interval.is_zero()
            || settings.gathering_timeout.is_zero()
        {
            return Err(failure("TypeError", "Invalid RTC host budgets"));
        }
        Ok(Self {
            settings,
            live: Rc::new(Cell::new(0)),
        })
    }
}

fn discovery(
    config: &Value,
    settings: Settings,
    crypto: Arc<dyn RTCCryptoProvider>,
) -> Result<(DiscoveryConfig, Vec<String>), WebRtcError> {
    let mut servers: Vec<Server> = Vec::new();
    let mut refused = Vec::new();
    let relay_only = config["iceTransportPolicy"] == "relay";
    let mut count = 0;
    for server in config["iceServers"].as_array().into_iter().flatten() {
        for raw in server["urls"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            count += 1;
            if count > 32 {
                return Err(failure(
                    "QuotaExceededError",
                    "RTC ICE server limit reached",
                ));
            }
            let mut url = Url::parse_url(raw).map_err(protocol_error)?;
            if relay_only && matches!(url.scheme, SchemeType::Stun | SchemeType::Stuns) {
                continue;
            }
            url.username = server["username"].as_str().unwrap_or("").into();
            url.password = server["credential"].as_str().unwrap_or("").into();
            let ip = url.host.parse::<IpAddr>().ok();
            if url.proto != ProtoType::Udp
                || !matches!(url.scheme, SchemeType::Stun | SchemeType::Turn)
                || ip
                    .is_none_or(|ip| ip.is_unspecified() || ip.is_ipv4() != settings.bind.is_ipv4())
            {
                refused.push(url.to_string());
                continue;
            }
            let address = SocketAddr::new(ip.expect("validated numeric host"), url.port);
            if url.scheme == SchemeType::Turn
                && servers
                    .iter()
                    .any(|s| s.url.scheme == SchemeType::Turn && s.address == address)
            {
                // One allocation per physical socket/server five-tuple. Preserve
                // getConfiguration, and report the extra unsupported attempt.
                refused.push(url.to_string());
                continue;
            }
            servers.push(Server { url, address });
        }
    }
    Ok((
        DiscoveryConfig {
            servers,
            relay_only,
            timeout: settings.gathering_timeout,
            allocation_refresh_interval_cap: settings.allocation_refresh_interval_cap,
            crypto_provider: crypto,
        },
        refused,
    ))
}

fn failure(name: &'static str, message: &str) -> WebRtcError {
    WebRtcError {
        name,
        message: message.into(),
    }
}
fn protocol_error(_: impl std::fmt::Display) -> WebRtcError {
    failure("OperationError", "RTC protocol operation failed")
}
fn candidate_value(candidate: RTCIceCandidate, local: bool) -> Option<Value> {
    let mut value = serde_json::to_value(candidate.to_json().ok()?).ok()?;
    if local {
        value["url"] = json!(candidate.url);
        if value["candidate"]
            .as_str()
            .is_some_and(|s| s.contains(" typ relay "))
        {
            value["relayProtocol"] = json!("udp");
        }
    }
    Some(value)
}
fn configuration(v: &Value) -> Result<RTCConfiguration, WebRtcError> {
    let servers = v["iceServers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|server| RTCIceServer {
            urls: server["urls"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            username: server["username"].as_str().unwrap_or("").into(),
            credential: server["credential"].as_str().unwrap_or("").into(),
        })
        .collect();
    let policy = if v["iceTransportPolicy"] == "relay" {
        RTCIceTransportPolicy::Relay
    } else {
        RTCIceTransportPolicy::All
    };
    let bundle = match v["bundlePolicy"].as_str() {
        Some("max-bundle") => RTCBundlePolicy::MaxBundle,
        Some("max-compat") => RTCBundlePolicy::MaxCompat,
        _ => RTCBundlePolicy::Balanced,
    };
    Ok(RTCConfigurationBuilder::new()
        .with_ice_servers(servers)
        .with_ice_transport_policy(policy)
        .with_bundle_policy(bundle)
        .with_ice_candidate_pool_size(v["iceCandidatePoolSize"].as_u64().unwrap_or(0) as u8)
        .build())
}

enum Endpoint {
    Host(Box<UdpPeer>),
    // No usable relay server is configured. Keep negotiation available without
    // registering a local candidate or owning a direct route.
    NoRoute(Box<RTCPeerConnection>),
}
impl Endpoint {
    fn core(&self) -> &RTCPeerConnection {
        match self {
            Self::Host(peer) => peer.core(),
            Self::NoRoute(peer) => peer,
        }
    }
    fn core_mut(&mut self) -> &mut RTCPeerConnection {
        match self {
            Self::Host(peer) => peer.core_mut(),
            Self::NoRoute(peer) => peer,
        }
    }
}

struct NativePeer {
    state: RefCell<State>,
    live: Rc<Cell<usize>>,
}
struct State {
    endpoint: Endpoint,
    settings: Settings,
    config: Value,
    observable: Value,
    channels: BTreeMap<usize, Value>,
    notifications: VecDeque<Value>,
    gathering: bool,
    held_candidates: Vec<Value>,
    transports: Value,
    revision: u64,
    refused_servers: Vec<String>,
    candidate_urls: BTreeMap<String, String>,
}

impl WebRtcFactory for NativeWebRtcFactory {
    fn create(&self, configuration_json: &str) -> Result<Rc<dyn WebRtcPeer>, WebRtcError> {
        if self.live.get() >= self.settings.max_peers {
            return Err(failure("QuotaExceededError", "RTC peer limit reached"));
        }
        let config: Value = serde_json::from_str(configuration_json).map_err(protocol_error)?;
        let crypto = default_provider().map_err(protocol_error)?;
        let (discovery, refused_servers) = discovery(&config, self.settings, crypto.clone())?;
        let mut settings_engine = SettingEngine::default();
        settings_engine.set_crypto_provider(crypto);
        let mut engine_config = config.clone();
        // The core rejects relay-only configurations with no TURN server,
        // whereas WebRTC permits creating them. The I/O driver enforces relay
        // policy independently, and NoRoute owns no socket or local candidate.
        if config["iceTransportPolicy"] == "relay" {
            engine_config["iceTransportPolicy"] = json!("all");
        }
        let core = RTCPeerConnectionBuilder::new()
            .with_setting_engine(settings_engine)
            .with_configuration(configuration(&engine_config)?)
            .build(Instant::now())
            .map_err(protocol_error)?;
        let endpoint = if config["iceTransportPolicy"] == "relay" && discovery.servers.is_empty() {
            Endpoint::NoRoute(Box::new(core))
        } else {
            Endpoint::Host(Box::new(
                UdpPeer::bind_host_with_discovery(
                    core,
                    SocketAddr::new(self.settings.bind, 0),
                    self.settings.io,
                    discovery,
                )
                .map_err(protocol_error)?,
            ))
        };
        self.live.set(self.live.get() + 1);
        Ok(Rc::new(NativePeer {
            live: self.live.clone(),
            state: RefCell::new(State {
                endpoint,
                settings: self.settings,
                config,
                channels: BTreeMap::new(),
                notifications: VecDeque::new(),
                gathering: false,
                held_candidates: Vec::new(),
                transports: Value::Null,
                revision: 0,
                refused_servers,
                candidate_urls: BTreeMap::new(),
                observable: json!({"signalingState":"stable","iceGatheringState":"new","iceConnectionState":"new","connectionState":"new"}),
            }),
        }))
    }
}
impl Drop for NativePeer {
    fn drop(&mut self) {
        self.live.set(self.live.get().saturating_sub(1));
    }
}

impl State {
    fn transports(&self) -> Value {
        self.endpoint.core().sctp().map_or(Value::Null, |sctp| {
            let dtls=sctp.transport(); let ice=dtls.ice_transport();
            let mut state=sctp.state().to_string();
            // The web "connected procedure" assigns stream IDs before exposing
            // Connected. The raw association can become established before its
            // completion event has crossed the protocol pipeline.
            if state=="connected" && self.transports["state"]!="connected" && self.channels.values().any(|c|c["id"].is_null() && c["readyState"]!="closed") {
                state="connecting".into();
            }
            let candidates=|values:Vec<RTCIceCandidate>,local| values.into_iter().filter_map(|mut c|{
                if local { c.url=self.candidate_urls.get(&c.id).cloned(); }
                candidate_value(c,local)
            }).collect::<Vec<_>>();
            json!({"state":state,"maxMessageSize":sctp.max_message_size(),"maxChannels":sctp.max_channels(),
                "dtls":{"state":dtls.state().to_string(),"certificates":dtls.get_remote_certificates(),
                    "ice":{"state":ice.state().to_string(),"gatheringState":ice.gathering_state().to_string(),
                        "role":ice.role().to_string(),"component":ice.component().to_string(),
                        "localCandidates":candidates(ice.get_local_candidates(),true),"remoteCandidates":candidates(ice.get_remote_candidates(),false),
                        "localParameters":ice.get_local_parameters(),"remoteParameters":ice.get_remote_parameters()}}})
        })
    }
    fn harvest_transports(&mut self) {
        if self.notifications.len() + 3 >= self.settings.max_notifications {
            return;
        }
        self.refresh_channel_ids();
        let value = self.transports();
        if self.transports != value {
            self.revision = self.revision.saturating_add(1);
        }
        if !self.transports.is_null() && !value.is_null() {
            for (kind, path) in [
                ("sctp", "/state"),
                ("dtls", "/dtls/state"),
                ("ice", "/dtls/ice/state"),
            ] {
                if self.transports.pointer(path) != value.pointer(path) {
                    self.notify(json!({"event":"transportstatechange","transport":kind}));
                }
            }
        }
        self.transports = value;
    }
    fn refresh_channel_ids(&mut self) {
        let ids: Vec<_> = self.channels.keys().copied().collect();
        for id in ids {
            if let Some(channel) = self.endpoint.core_mut().data_channel(id)
                && let Some(meta) = self.channels.get_mut(&id)
            {
                meta["id"] = json!(channel.stream_id());
                meta["readyState"] = json!(channel.ready_state().to_string());
            }
        }
    }
    fn channel(&mut self, id: usize) -> Option<Value> {
        if let Some(dc) = self.endpoint.core_mut().data_channel(id) {
            let meta = json!({"handle":id,"label":dc.label(),"ordered":dc.ordered(),
                "maxPacketLifeTime":dc.max_packet_life_time(),"maxRetransmits":dc.max_retransmits(),
                "protocol":dc.protocol(),"negotiated":dc.negotiated(),"id":dc.stream_id(),
                "readyState":dc.ready_state().to_string(),"bufferedAmount":dc.outstanding_bytes()});
            self.channels.insert(id, meta);
        }
        self.channels.get(&id).cloned()
    }
    fn snapshot(&mut self) -> Value {
        let ids: Vec<_> = self.channels.keys().copied().collect();
        for id in ids {
            self.channel(id);
        }
        let core = self.endpoint.core();
        let remote = core.remote_description();
        let mut value = self.observable.clone();
        value["revision"] = json!(self.revision);
        value["localDescription"] = json!(core.local_description());
        value["currentLocalDescription"] = json!(core.current_local_description());
        value["pendingLocalDescription"] = json!(core.pending_local_description());
        value["remoteDescription"] = json!(remote);
        value["currentRemoteDescription"] = json!(core.current_remote_description());
        value["pendingRemoteDescription"] = json!(core.pending_remote_description());
        value["canTrickleIceCandidates"] = remote.map_or(Value::Null, |d| {
            json!(
                d.sdp
                    .lines()
                    .any(|l| l.starts_with("a=ice-options:") && l.contains("trickle"))
            )
        });
        value["channels"] = json!(self.channels.values().collect::<Vec<_>>());
        value["sctp"] = self.transports();
        value
    }
    fn notify(&mut self, mut value: Value) {
        value["state"] = self.snapshot();
        self.notifications.push_back(value);
    }
    fn observe(&mut self, event: RTCPeerConnectionEvent) {
        self.revision = self.revision.saturating_add(1);
        let name = match event {
            RTCPeerConnectionEvent::OnSignalingStateChangeEvent(state) => {
                self.observable["signalingState"] = json!(state.to_string());
                "signalingstatechange"
            },
            RTCPeerConnectionEvent::OnConnectionStateChangeEvent(state) => {
                self.observable["connectionState"] = json!(state.to_string());
                "connectionstatechange"
            },
            RTCPeerConnectionEvent::OnIceConnectionStateChangeEvent(state) => {
                self.observable["iceConnectionState"] = json!(state.to_string());
                "iceconnectionstatechange"
            },
            RTCPeerConnectionEvent::OnIceGatheringStateChangeEvent(state) => {
                if !self.gathering {
                    return;
                }
                self.observable["iceGatheringState"] = json!(state.to_string());
                if state.to_string() == "complete" {
                    self.notify(json!({"event":"icegatheringstatechange"}));
                    self.notify(json!({"event":"icecandidate","candidate":null}));
                    return;
                }
                "icegatheringstatechange"
            },
            RTCPeerConnectionEvent::OnIceCandidateEvent(event) => {
                let mut candidate = event.candidate;
                if !event.url.is_empty() {
                    self.candidate_urls
                        .insert(candidate.id.clone(), event.url.clone());
                    candidate.url = Some(event.url);
                }
                let mut value = candidate_value(candidate, true).unwrap_or(Value::Null);
                if value["candidate"].as_str().is_some_and(str::is_empty) {
                    value = Value::Null;
                }
                if self.gathering {
                    self.notify(json!({"event":"icecandidate","candidate":value}));
                } else {
                    self.held_candidates.push(value);
                }
                return;
            },
            RTCPeerConnectionEvent::OnNegotiationNeededEvent => "negotiationneeded",
            RTCPeerConnectionEvent::OnDataChannel(event) => {
                let (id, name) = match event {
                    RTCDataChannelEvent::OnOpen(id) => (id, "open"),
                    RTCDataChannelEvent::OnClose(id) => (id, "close"),
                    RTCDataChannelEvent::OnClosing(id) => (id, "closing"),
                    RTCDataChannelEvent::OnBufferedAmountLow(id) => (id, "bufferedamountlow"),
                    RTCDataChannelEvent::OnError(id) => (id, "error"),
                    _ => return,
                };
                let incoming = !self.channels.contains_key(&id);
                if let Some(mut channel) = self.channel(id) {
                    if name == "close" {
                        channel["readyState"] = json!("closed");
                        self.channels.insert(id, channel.clone());
                    }
                    if incoming {
                        self.notify(json!({"event":"datachannel","channel":channel}));
                    }
                    self.notify(json!({"event":name,"channel":channel}));
                }
                return;
            },
            _ => return,
        };
        self.notify(json!({"event":name}));
    }
    fn harvest(&mut self) {
        while self.notifications.len() + 2 < self.settings.max_notifications {
            let Some(event) = self.endpoint.core_mut().poll_event() else {
                break;
            };
            self.observe(event);
        }
    }
}

impl WebRtcPeer for NativePeer {
    fn wake_delay(&self) -> Duration {
        let mut state = self.state.borrow_mut();
        if !state.notifications.is_empty() {
            return Duration::ZERO;
        }
        let interval = state.settings.poll_interval;
        let deadline = match &mut state.endpoint {
            Endpoint::Host(peer) => peer.next_deadline(),
            Endpoint::NoRoute(peer) => peer.poll_timeout(),
        };
        deadline.map_or(interval, |time| {
            time.saturating_duration_since(Instant::now()).min(interval)
        })
    }
    fn call(&self, operation: WebRtcOperation, payload: &str) -> Result<String, WebRtcError> {
        let data: Value = serde_json::from_str(payload).map_err(protocol_error)?;
        let mut state = self.state.borrow_mut();
        if state.notifications.len() + 4 >= state.settings.max_notifications
            && !matches!(
                operation,
                WebRtcOperation::Snapshot | WebRtcOperation::Close
            )
        {
            return Err(failure("OperationError", "RTC host event queue is full"));
        }
        state.revision = state.revision.saturating_add(1);
        let now = Instant::now();
        let value = match operation {
            WebRtcOperation::Snapshot => Value::Null,
            WebRtcOperation::CreateOffer => {
                if data["iceRestart"] == true {
                    if state.config["iceServers"]
                        .as_array()
                        .is_some_and(|s| !s.is_empty())
                    {
                        return Err(failure(
                            "NotSupportedError",
                            "ICE server restart is not yet supported",
                        ));
                    }
                    state.endpoint.core_mut().restart_ice();
                }
                json!(
                    state
                        .endpoint
                        .core_mut()
                        .create_offer(None)
                        .map_err(protocol_error)?
                )
            },
            WebRtcOperation::CreateAnswer => json!(
                state
                    .endpoint
                    .core_mut()
                    .create_answer(None)
                    .map_err(protocol_error)?
            ),
            WebRtcOperation::SetLocalDescription => {
                let description = if data["type"].is_null() {
                    let mut description =
                        if state.observable["signalingState"] == "have-remote-offer" {
                            state.endpoint.core_mut().create_answer(None)
                        } else {
                            state.endpoint.core_mut().create_offer(None)
                        }
                        .map_err(protocol_error)?;
                    if let Some(sdp) = data["sdp"].as_str().filter(|s| !s.is_empty()) {
                        description.sdp = sdp.to_owned();
                    }
                    description
                } else {
                    serde_json::from_value::<RTCSessionDescription>(data).map_err(protocol_error)?
                };
                state
                    .endpoint
                    .core_mut()
                    .set_local_description(now, description)
                    .map_err(protocol_error)?;
                if !state.gathering {
                    state.gathering = true;
                    state.observable["iceGatheringState"] = json!("gathering");
                    state.notify(json!({"event":"icegatheringstatechange"}));
                    for candidate in std::mem::take(&mut state.held_candidates) {
                        state.notify(json!({"event":"icecandidate","candidate":candidate}));
                    }
                    match &mut state.endpoint {
                        Endpoint::Host(peer) => {
                            peer.finish_host_candidates().map_err(protocol_error)?
                        },
                        Endpoint::NoRoute(peer) => peer
                            .add_local_candidate(Default::default())
                            .map_err(protocol_error)?,
                    }
                }
                Value::Null
            },
            WebRtcOperation::SetRemoteDescription => {
                let description = serde_json::from_value::<RTCSessionDescription>(data)
                    .map_err(protocol_error)?;
                state
                    .endpoint
                    .core_mut()
                    .set_remote_description(now, description)
                    .map_err(protocol_error)?;
                Value::Null
            },
            WebRtcOperation::AddIceCandidate => {
                let candidate = if data.is_null() {
                    RTCIceCandidateInit::default()
                } else {
                    serde_json::from_value(data).map_err(protocol_error)?
                };
                state
                    .endpoint
                    .core_mut()
                    .add_remote_candidate(candidate)
                    .map_err(protocol_error)?;
                Value::Null
            },
            WebRtcOperation::RestartIce => {
                if state.config["iceServers"]
                    .as_array()
                    .is_some_and(|s| !s.is_empty())
                {
                    return Err(failure(
                        "NotSupportedError",
                        "ICE server restart is not yet supported",
                    ));
                }
                state.endpoint.core_mut().restart_ice();
                Value::Null
            },
            WebRtcOperation::GetConfiguration => state.config.clone(),
            WebRtcOperation::SetConfiguration => {
                if state.config["iceTransportPolicy"] != data["iceTransportPolicy"] {
                    return Err(failure(
                        "NotSupportedError",
                        "Changing the host route policy is not yet supported",
                    ));
                }
                if state.config["iceServers"] != data["iceServers"] {
                    return Err(failure(
                        "NotSupportedError",
                        "Replacing ICE servers is not yet supported",
                    ));
                }
                state
                    .endpoint
                    .core_mut()
                    .set_configuration(configuration(&data)?)
                    .map_err(protocol_error)?;
                state.config = data;
                Value::Null
            },
            WebRtcOperation::CreateDataChannel => {
                if data["label"].as_str().unwrap_or("").len() > u16::MAX as usize
                    || data["protocol"].as_str().unwrap_or("").len() > u16::MAX as usize
                {
                    return Err(failure("TypeError", "RTC channel string is too long"));
                }
                if state.channels.len() >= state.settings.max_channels {
                    return Err(failure("OperationError", "RTC channel limit reached"));
                }
                state.refresh_channel_ids();
                if data["negotiated"] == true
                    && state.channels.values().any(|channel| {
                        channel["id"] == data["id"] && channel["readyState"] != "closed"
                    })
                {
                    return Err(failure(
                        "OperationError",
                        "RTC channel ID is already in use",
                    ));
                }
                let init = RTCDataChannelInit {
                    ordered: data["ordered"].as_bool().unwrap_or(true),
                    protocol: data["protocol"].as_str().unwrap_or("").into(),
                    max_packet_life_time: data["maxPacketLifeTime"].as_u64().map(|v| v as u16),
                    max_retransmits: data["maxRetransmits"].as_u64().map(|v| v as u16),
                    negotiated: if data["negotiated"] == true {
                        data["id"].as_u64().map(|v| v as u16)
                    } else {
                        None
                    },
                };
                let id = state
                    .endpoint
                    .core_mut()
                    .create_data_channel(data["label"].as_str().unwrap_or(""), Some(init))
                    .map_err(protocol_error)?
                    .id();
                state
                    .channel(id)
                    .ok_or_else(|| failure("OperationError", "RTC channel unavailable"))?
            },
            WebRtcOperation::Send => {
                let id = data["channel"]
                    .as_u64()
                    .ok_or_else(|| failure("TypeError", "Invalid RTC channel"))?
                    as usize;
                let bytes: Vec<u8> =
                    serde_json::from_value(data["data"].clone()).map_err(protocol_error)?;
                let mut channel =
                    state.endpoint.core_mut().data_channel(id).ok_or_else(|| {
                        failure("InvalidStateError", "RTC channel is unavailable")
                    })?;
                if data["isString"] == true {
                    channel
                        .send_text(now, String::from_utf8(bytes).map_err(protocol_error)?)
                        .map_err(protocol_error)?;
                } else {
                    channel
                        .send(now, bytes::BytesMut::from(bytes.as_slice()))
                        .map_err(protocol_error)?;
                }
                Value::Null
            },
            WebRtcOperation::CloseDataChannel | WebRtcOperation::BufferedAmountLowThreshold => {
                let id = data["channel"]
                    .as_u64()
                    .ok_or_else(|| failure("TypeError", "Invalid RTC channel"))?
                    as usize;
                if let Some(mut channel) = state.endpoint.core_mut().data_channel(id) {
                    if matches!(operation, WebRtcOperation::CloseDataChannel) {
                        channel.close().map_err(protocol_error)?;
                    } else {
                        channel.set_buffered_amount_low_threshold(
                            data["value"].as_u64().unwrap_or(0) as u32,
                        );
                    }
                }
                Value::Null
            },
            WebRtcOperation::GetStats => {
                let report = state
                    .endpoint
                    .core_mut()
                    .get_stats(now, StatsSelector::None);
                let entries: Vec<Value> = report
                    .iter()
                    .filter_map(|entry| match entry {
                        RTCStatsReportEntry::PeerConnection(v) => serde_json::to_value(v).ok(),
                        RTCStatsReportEntry::Transport(v) => serde_json::to_value(v).ok(),
                        RTCStatsReportEntry::LocalCandidate(v)
                        | RTCStatsReportEntry::RemoteCandidate(v) => serde_json::to_value(v).ok(),
                        RTCStatsReportEntry::IceCandidatePair(v) => serde_json::to_value(v).ok(),
                        RTCStatsReportEntry::Certificate(v) => serde_json::to_value(v).ok(),
                        RTCStatsReportEntry::DataChannel(v) => serde_json::to_value(v).ok(),
                        _ => None,
                    })
                    .collect();
                json!(entries)
            },
            WebRtcOperation::Close => {
                let _ = state.endpoint.core_mut().close();
                // Nonblocking flush; dropping the per-realm handle then releases
                // the socket independently of VM collection or native threads.
                if let Endpoint::Host(peer) = &mut state.endpoint {
                    for _ in 0..4 {
                        if matches!(
                            peer.poll(now + Duration::from_millis(10), ReadInterest::EventsOnly),
                            Ok(Some(Output::Closed))
                        ) {
                            break;
                        }
                    }
                }
                Value::Null
            },
        };
        state.harvest();
        state.harvest_transports();
        // Applying SDP changes signaling state and fires its event in the same
        // task, before the operation promise resolves. Other observations stay
        // queued for separate host tasks.
        let mut immediate = Vec::new();
        if matches!(
            operation,
            WebRtcOperation::SetLocalDescription | WebRtcOperation::SetRemoteDescription
        ) {
            state.notifications.retain(|event| {
                if event["event"] == "signalingstatechange" {
                    immediate.push(event.clone());
                    false
                } else {
                    true
                }
            });
        }
        Ok(json!({"value":value,"state":state.snapshot(),"events":immediate}).to_string())
    }
    fn poll(&self, budget: usize) -> Vec<String> {
        let mut state = self.state.borrow_mut();
        let mut out = Vec::new();
        for _ in 0..budget {
            state.harvest_transports();
            if let Some(value) = state.notifications.pop_front() {
                out.push(value.to_string());
                continue;
            }
            if state.gathering && !state.refused_servers.is_empty() {
                let url = state.refused_servers.remove(0);
                state.notify(
                    json!({"event":"icecandidateerror","url":url,"errorCode":701,
                    "errorText":"ICE server transport or address is not supported by this host"}),
                );
                if let Some(value) = state.notifications.pop_front() {
                    out.push(value.to_string());
                }
                continue;
            }
            let output = match &mut state.endpoint {
                Endpoint::Host(peer) => peer.poll(
                    Instant::now() + Duration::from_millis(10),
                    ReadInterest::All,
                ),
                Endpoint::NoRoute(peer) => {
                    let now = Instant::now();
                    let result = if peer.poll_timeout().is_some_and(|time| time <= now) {
                        peer.handle_timeout(now)
                    } else {
                        Ok(())
                    };
                    // No configured relay exists. Discard protocol output here;
                    // this endpoint owns no socket and cannot fall back to a
                    // direct route. Timers still reach their failure states.
                    for _ in 0..8 {
                        if peer.poll_write().is_none() {
                            break;
                        }
                    }
                    result
                        .map(|()| peer.poll_event().map(Output::Event))
                        .map_err(Into::into)
                },
            };
            match output {
                Ok(Some(Output::GatheringError { url, code })) => state.notify(json!({
                    "event":"icecandidateerror","url":url,"errorCode":code,
                    "errorText":"ICE server gathering failed"})),
                Ok(Some(Output::Event(event))) => state.observe(event),
                Ok(Some(Output::Message(message))) => {
                    if let RTCMessage::DataChannelMessage(id, message) = message.message {
                        state.notify(json!({"event":"message","handle":id,"isString":message.is_string,"data":message.data.to_vec()}));
                    }
                },
                Err(_) => {
                    state.observable["connectionState"] = json!("failed");
                    state.notify(json!({"event":"connectionstatechange"}));
                },
                _ => {},
            }
            if let Some(value) = state.notifications.pop_front() {
                out.push(value.to_string());
            }
        }
        out
    }
}
