use super::*;
use rtc::stun::message::BINDING_SUCCESS;

#[test]
fn restarted_stun_discards_the_old_transaction_but_preserves_ice_demux() {
    let local = "127.0.0.1:10000".parse().unwrap();
    let server = "127.0.0.1:3478".parse().unwrap();
    let mut discovery = Discovery::new(
        local,
        DiscoveryConfig {
            named_servers: Vec::new(),
            resolver: None,
            servers: vec![Server {
                url: Url::parse_url("stun:127.0.0.1:3478").unwrap(),
                address: server,
            }],
            relay_only: false,
            timeout: Duration::from_secs(1),
            allocation_refresh_interval_cap: None,
            crypto_provider: rtc::crypto::default_provider().unwrap(),
        },
    )
    .unwrap();
    discovery.start(local, Instant::now()).unwrap();
    let transaction = |packet: TaggedBytesMut| {
        let mut message = Message::new();
        message.raw = packet.message.to_vec();
        message.decode().unwrap();
        message.transaction_id
    };
    let old = transaction(discovery.poll_write().unwrap());
    discovery.restart(local, Instant::now()).unwrap();
    let fresh = transaction(discovery.poll_write().unwrap());
    assert_ne!(old, fresh);
    let response = |tid| {
        let mut message = Message::new();
        message
            .build(&[Box::new(tid), Box::new(BINDING_SUCCESS)])
            .unwrap();
        TaggedBytesMut {
            now: Instant::now(),
            message: bytes::BytesMut::from(message.raw.as_slice()),
            transport: TransportContext {
                local_addr: local,
                peer_addr: server,
                transport_protocol: TransportProtocol::UDP,
                ecn: None,
            },
        }
    };
    assert!(discovery.read(response(old)).unwrap().is_some());
    assert!(
        discovery.clients[0]
            .client
            .as_ref()
            .unwrap()
            .has_transaction(&fresh)
    );
    assert!(discovery.read(response(fresh)).unwrap().is_none());
}

#[test]
fn shared_server_response_demux_checks_transaction_and_source_but_preserves_ice_requests() {
    let local: SocketAddr = "127.0.0.1:10000".parse().unwrap();
    let server: SocketAddr = "127.0.0.1:3478".parse().unwrap();
    let mut discovery = Discovery::new(
        local,
        DiscoveryConfig {
            named_servers: Vec::new(),
            resolver: None,
            servers: vec![Server {
                url: Url::parse_url("stun:127.0.0.1:3478").unwrap(),
                address: server,
            }],
            relay_only: false,
            timeout: Duration::from_secs(1),
            allocation_refresh_interval_cap: None,
            crypto_provider: rtc::crypto::default_provider().unwrap(),
        },
    )
    .unwrap();
    discovery.start(local, Instant::now()).unwrap();
    let request = discovery.poll_write().unwrap();
    let mut message = Message::new();
    message.raw = request.message.to_vec();
    message.decode().unwrap();
    let tid = message.transaction_id;
    let packet = |raw: Vec<u8>, peer_addr| TaggedBytesMut {
        now: Instant::now(),
        message: bytes::BytesMut::from(raw.as_slice()),
        transport: TransportContext {
            local_addr: local,
            peer_addr,
            transport_protocol: TransportProtocol::UDP,
            ecn: None,
        },
    };
    // Even a Binding request with the same ID belongs to ICE, not discovery.
    assert!(
        discovery
            .read(packet(message.raw.clone(), server))
            .unwrap()
            .is_some()
    );
    let mut response = Message::new();
    response
        .build(&[Box::new(tid), Box::new(BINDING_SUCCESS)])
        .unwrap();
    let wrong_source: SocketAddr = "127.0.0.1:3479".parse().unwrap();
    assert!(
        discovery
            .read(packet(response.raw.clone(), wrong_source))
            .unwrap()
            .is_some()
    );
    assert!(
        discovery.clients[0]
            .client
            .as_ref()
            .unwrap()
            .has_transaction(&tid)
    );
    assert!(
        discovery
            .read(packet(response.raw, server))
            .unwrap()
            .is_none()
    );
    assert!(
        !discovery.clients[0]
            .client
            .as_ref()
            .unwrap()
            .has_transaction(&tid)
    );
}
