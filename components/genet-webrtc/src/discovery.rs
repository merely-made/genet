// ICE URL placement and unsupported-attempt reporting belong to Genet.
use super::{Settings, failure, protocol_error};
use murm_rtc::resolve::Resolver;
use murm_rtc::rtc::crypto::RTCCryptoProvider;
use murm_rtc::rtc::ice::url::{ProtoType, SchemeType, Url};
use murm_rtc::udp::{DiscoveryConfig, NamedServer, Server};
use script_runtime_api::WebRtcError;
use serde_json::Value;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

// The core parser supplies defaults, so preserve omission before handing a
// hostname to the resolver. Only a validated, accepted query selects UDP.
fn explicit_port(raw: &str) -> bool {
    raw.split_once(':')
        .and_then(|(_, rest)| rest.split('?').next())
        .and_then(|authority| authority.rsplit('@').next())
        .is_some_and(|authority| authority.rsplit_once(':').is_some())
}

pub(super) fn configuration(
    config: &Value,
    settings: Settings,
    crypto: Arc<dyn RTCCryptoProvider>,
    resolver: Arc<dyn Resolver>,
) -> Result<(DiscoveryConfig, Vec<String>), WebRtcError> {
    let mut servers: Vec<Server> = Vec::new();
    let mut named_servers = Vec::new();
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
                || url.port == 0
                || ip.is_some_and(|ip| {
                    ip.is_unspecified() || ip.is_ipv4() != settings.bind.is_ipv4()
                })
            {
                refused.push(url.to_string());
                continue;
            }
            let Some(ip) = ip else {
                let port = explicit_port(raw).then_some(url.port);
                let transport_specified = raw
                    .split_once('?')
                    .is_some_and(|(_, q)| q.split('&').any(|part| !part.is_empty()));
                named_servers.push(NamedServer {
                    url,
                    port,
                    transport_specified,
                });
                continue;
            };
            let address = SocketAddr::new(ip, url.port);
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
            named_servers,
            resolver: Some(resolver),
            relay_only,
            timeout: settings.gathering_timeout,
            allocation_refresh_interval_cap: settings.allocation_refresh_interval_cap,
            crypto_provider: crypto,
        },
        refused,
    ))
}
