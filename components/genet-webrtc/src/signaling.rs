use super::*;

pub(super) fn perform(
    state: &mut State,
    operation: WebRtcOperation,
    data: Value,
    now: Instant,
) -> Result<Value, WebRtcError> {
    Ok(match operation {
        WebRtcOperation::CreateOffer => {
            json!(
                state
                    .endpoint
                    .core_mut()
                    .create_offer(Some(RTCOfferOptions {
                        ice_restart: data["iceRestart"] == true
                    }))
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
        WebRtcOperation::SetLocalDescription => set_local(state, data, now)?,
        WebRtcOperation::SetRemoteDescription => {
            let description =
                serde_json::from_value::<RTCSessionDescription>(data).map_err(protocol_error)?;
            state
                .endpoint
                .core_mut()
                .set_remote_description(now, description)
                .map_err(protocol_error)?;
            Value::Null
        },
        WebRtcOperation::AddIceCandidate => add_candidate(state, data, now)?,
        WebRtcOperation::RestartIce => {
            state.endpoint.core_mut().restart_ice();
            Value::Null
        },
        WebRtcOperation::GetConfiguration => state.config.clone(),
        WebRtcOperation::SetConfiguration => configure(state, data, now)?,
        _ => return Err(failure("OperationError", "Invalid signaling operation")),
    })
}

fn set_local(state: &mut State, data: Value, now: Instant) -> Result<Value, WebRtcError> {
    let rollback = data["type"] == "rollback";
    let description = if data["type"].is_null() {
        let mut description = if state.observable["signalingState"] == "have-remote-offer" {
            state.endpoint.core_mut().create_answer(None)
        } else {
            state
                .endpoint
                .core_mut()
                .create_offer(Some(RTCOfferOptions {
                    ice_restart: data["iceRestart"] == true,
                }))
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
    if !rollback {
        state.begin_gathering()?;
    }
    Ok(Value::Null)
}

fn add_candidate(state: &mut State, data: Value, _now: Instant) -> Result<Value, WebRtcError> {
    if let Some(ufrag) = data["usernameFragment"].as_str()
        && state.endpoint.core().remote_description().is_some_and(|d| {
            !d.sdp
                .lines()
                .any(|line| line.strip_prefix("a=ice-ufrag:") == Some(ufrag))
        })
    {
        return Err(failure(
            "OperationError",
            "ICE candidate belongs to a different generation",
        ));
    }
    let candidate = if data.is_null() {
        RTCIceCandidateInit::default()
    } else {
        serde_json::from_value(data).map_err(protocol_error)?
    };
    if candidate
        .candidate
        .split_whitespace()
        .nth(4)
        .is_some_and(|address| address.to_ascii_lowercase().ends_with(".local"))
    {
        return Err(failure(
            "OperationError",
            "mDNS ICE candidate resolution is not yet supported",
        ));
    }
    state
        .endpoint
        .core_mut()
        .add_remote_candidate(candidate)
        .map_err(protocol_error)?;
    Ok(Value::Null)
}

fn configure(state: &mut State, data: Value, _now: Instant) -> Result<Value, WebRtcError> {
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
    Ok(Value::Null)
}
