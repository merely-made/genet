//! WebRTC web objects over a per-realm, host-supplied peer factory.
//!
//! The host protocol carries JSON descriptions of standard RTC operations and
//! observations. It carries no VM values, socket types or application admission.

use std::cell::RefCell;
use std::rc::Rc;

use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::{HostState, Runtime};

/// A browser-visible host failure. Messages must not contain credentials/SDP.
#[derive(Debug)]
pub struct WebRtcError {
    /// Standard JavaScript/DOM exception name.
    pub name: &'static str,
    /// Public diagnostic without private transport material.
    pub message: String,
}

/// Standard operations accepted by a host peer, independently of a JS engine.
#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebRtcOperation {
    /// Create an SDP offer.
    CreateOffer,
    /// Create an SDP answer.
    CreateAnswer,
    /// Apply or implicitly create the local SDP.
    SetLocalDescription,
    /// Apply remote SDP.
    SetRemoteDescription,
    /// Add a remote candidate or its end-of-gathering marker.
    AddIceCandidate,
    /// Request an ICE restart.
    RestartIce,
    /// Read the selected configuration.
    GetConfiguration,
    /// Update configuration.
    SetConfiguration,
    /// Create a data channel.
    CreateDataChannel,
    /// Send a channel message.
    Send,
    /// Close a channel.
    CloseDataChannel,
    /// Update the low-water threshold.
    BufferedAmountLowThreshold,
    /// Obtain connection stats.
    GetStats,
    /// Read current observable state without performing network I/O.
    Snapshot,
    /// Close this connection.
    Close,
}

/// One native peer owned by one document realm.
pub trait WebRtcPeer {
    /// Execute an operation. Payload and reply are JSON values; replies contain
    /// `value` and `state`. The host must retain its own network policy/limits.
    fn call(&self, operation: WebRtcOperation, payload: &str) -> Result<String, WebRtcError>;
    /// Poll at most `budget` host turns. Returned JSON observations are delivered
    /// as realm tasks with a microtask checkpoint after each task.
    fn poll(&self, budget: usize) -> Vec<String>;
    /// Host-selected delay until the next network poll or protocol deadline.
    fn wake_delay(&self) -> std::time::Duration;
}

/// Creates independent peer resources; a factory may be shared by child realms.
pub trait WebRtcFactory {
    /// Construct a peer from normalized RTCConfiguration JSON. Dropping the
    /// returned peer must release its native resources without a VM finalizer.
    fn create(&self, configuration: &str) -> Result<Rc<dyn WebRtcPeer>, WebRtcError>;
}

fn arg<E: ScriptEngine>(cx: &mut E::CallCx<'_>, index: usize) -> Result<String, E::Error> {
    let value = cx.arg(index);
    cx.value_to_string(&value)
}

fn envelope(result: Result<String, WebRtcError>) -> String {
    match result {
        Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(value) => serde_json::json!({"ok":true,"result":value}).to_string(),
            Err(_) => serde_json::json!({"ok":false,"name":"OperationError","message":"Invalid RTC host reply"}).to_string(),
        },
        Err(error) => serde_json::json!({"ok":false,"name":error.name,"message":error.message}).to_string(),
    }
}

fn active_document(host: &RefCell<HostState>, realm: script_engine_api::RealmId) -> bool {
    host.borrow()
        .agent
        .upgrade()
        .is_some_and(|agent| agent.borrow().hosts.contains_key(&realm))
}

fn discarded() -> WebRtcError {
    WebRtcError {
        name: "InvalidStateError",
        message: "RTC document was discarded".into(),
    }
}

struct Create;
impl<E: ScriptEngine> NativeFn<E> for Create {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let configuration = arg::<E>(cx, 0)?;
        let data = cx
            .host_data()
            .ok_or_else(|| cx.error("missing RTC document"))?;
        let host = data
            .downcast_ref::<RefCell<HostState>>()
            .ok_or_else(|| cx.error("invalid RTC document"))?;
        if !active_document(host, cx.current_realm()) {
            return cx.make_string(&envelope(Err(discarded())));
        }
        let factory = host.borrow().webrtc_factory.clone();
        let result = match factory {
            Some(factory) => factory.create(&configuration).and_then(|peer| {
                let mut host = host.borrow_mut();
                let id = host
                    .webrtc_next_id
                    .checked_add(1)
                    .ok_or_else(|| WebRtcError {
                        name: "QuotaExceededError",
                        message: "RTC handle space exhausted".into(),
                    })?;
                host.webrtc_next_id = id;
                host.webrtc_peers.insert(id, peer);
                Ok(serde_json::json!({"id":id.to_string()}).to_string())
            }),
            None => Err(WebRtcError {
                name: "NotSupportedError",
                message: "No WebRTC host is installed".into(),
            }),
        };
        cx.make_string(&envelope(result))
    }
}

struct Command;
impl<E: ScriptEngine> NativeFn<E> for Command {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = arg::<E>(cx, 0)?.parse::<u64>().ok();
        let operation = arg::<E>(cx, 1)?;
        let payload = arg::<E>(cx, 2)?;
        let data = cx
            .host_data()
            .ok_or_else(|| cx.error("missing RTC document"))?;
        let host = data
            .downcast_ref::<RefCell<HostState>>()
            .ok_or_else(|| cx.error("invalid RTC document"))?;
        if !active_document(host, cx.current_realm()) {
            return cx.make_string(&envelope(Err(discarded())));
        }
        let peer = id.and_then(|id| host.borrow().webrtc_peers.get(&id).cloned());
        let operation =
            serde_json::from_value::<WebRtcOperation>(serde_json::Value::String(operation));
        let result = match (peer, operation) {
            (Some(peer), Ok(operation)) => {
                let result = peer.call(operation, &payload);
                if matches!(operation, WebRtcOperation::Close) {
                    if let Some(id) = id {
                        host.borrow_mut().webrtc_peers.remove(&id);
                    }
                }
                result
            },
            _ => Err(WebRtcError {
                name: "InvalidStateError",
                message: "RTC peer is unavailable".into(),
            }),
        };
        cx.make_string(&envelope(result))
    }
}

pub(crate) fn install<E: ScriptEngine>(
    surface: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    surface.set_function::<Create>("__rtc_new", 1)?;
    surface.set_function::<Command>("__rtc_command", 3)?;
    surface.eval(include_str!("webrtc/bootstrap.js"))?;
    Ok(())
}

/// Revoke a discarded document even when another realm retains its global.
pub(crate) fn revoke(host: &crate::SharedHost) {
    let mut host = host.borrow_mut();
    host.webrtc_peers.clear();
    host.webrtc_factory = None;
}

impl<E: ScriptEngine> Runtime<E> {
    /// Install the document's peer factory. Children inherit the factory, while
    /// connection handles and native peers remain owned by each realm's host.
    pub fn set_webrtc_factory(&mut self, factory: Rc<dyn WebRtcFactory>) {
        self.host().borrow_mut().webrtc_factory = Some(factory);
    }

    /// Poll native peers and deliver bounded observations in their owning realm.
    pub fn pump_webrtc(&mut self, budget: usize) -> usize {
        let peers: Vec<_> = self
            .agent
            .borrow()
            .hosts
            .iter()
            .flat_map(|(&realm, host)| {
                host.borrow()
                    .webrtc_peers
                    .iter()
                    .map(|(&id, peer)| (realm, id, peer.clone()))
                    .collect::<Vec<_>>()
            })
            .collect();
        if peers.is_empty() {
            return 0;
        }
        let cursor = self.host.borrow().webrtc_poll_cursor % peers.len();
        let mut delivered = 0;
        for turn in 0..budget {
            let (realm, id, peer) = &peers[(cursor + turn) % peers.len()];
            let active = self
                .agent
                .borrow()
                .hosts
                .get(realm)
                .is_some_and(|host| host.borrow().webrtc_peers.contains_key(id));
            if !active {
                continue;
            }
            for observation in peer.poll(1) {
                if let Err(error) =
                    self.call_task(*realm, "__rtcDeliver", &[&id.to_string(), &observation])
                {
                    self.report_script_exception(&format!("RTC event delivery: {error}"), None);
                }
                self.run_microtasks();
                delivered += 1;
            }
        }
        self.host.borrow_mut().webrtc_poll_cursor = (cursor + budget) % peers.len();
        delivered
    }

    /// Native peers still owned by active document realms.
    pub fn pending_webrtc(&self) -> usize {
        self.agent
            .borrow()
            .hosts
            .values()
            .map(|host| host.borrow().webrtc_peers.len())
            .sum()
    }

    /// Earliest host-selected RTC wakeup across active document realms, in ms.
    pub fn next_webrtc_delay(&self) -> Option<f64> {
        self.agent
            .borrow()
            .hosts
            .values()
            .flat_map(|host| {
                host.borrow()
                    .webrtc_peers
                    .values()
                    .map(|peer| peer.wake_delay())
                    .collect::<Vec<_>>()
            })
            .min()
            .map(|delay| delay.as_secs_f64() * 1000.0)
    }
}
