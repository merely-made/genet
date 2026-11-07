use super::*;

pub(super) fn perform(
    state: &mut State,
    operation: WebRtcOperation,
    data: Value,
    now: Instant,
) -> Result<Value, WebRtcError> {
    Ok(match operation {
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
                && state
                    .channels
                    .values()
                    .any(|channel| channel["id"] == data["id"] && channel["readyState"] != "closed")
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
            let mut channel = state
                .endpoint
                .core_mut()
                .data_channel(id)
                .ok_or_else(|| failure("InvalidStateError", "RTC channel is unavailable"))?;
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
                        data["value"].as_u64().unwrap_or(0) as u32
                    );
                }
            }
            Value::Null
        },
        _ => return Err(failure("OperationError", "Invalid channel operation")),
    })
}
