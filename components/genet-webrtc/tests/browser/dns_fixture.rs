//! Bounded, caller-driven DNS fixture shared by address and service tests.
use super::*;
use hickory_proto::op::{Message, OpCode};
use hickory_proto::rr::RecordType;
use murm_rtc::resolve::{DnsConfig, DnsLimits, DnsResolver};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;

pub(super) struct DnsFixture {
    pub socket: UdpSocket,
    pub requests: Vec<SocketAddr>,
    pub queries: Vec<(String, RecordType)>,
}
impl DnsFixture {
    pub fn new() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            requests: Vec::new(),
            queries: Vec::new(),
        }
    }
    pub fn resolver(&self, limits: DnsLimits) -> Arc<DnsResolver> {
        Arc::new(
            DnsResolver::new(DnsConfig {
                servers: vec![self.socket.local_addr().unwrap()],
                limits,
            })
            .unwrap(),
        )
    }
    pub fn poll(&mut self, mut answer: impl FnMut(&Message) -> Option<Message>) {
        let mut bytes = [0; 4096];
        for _ in 0..8 {
            let Ok((n, from)) = self.socket.recv_from(&mut bytes) else {
                break;
            };
            assert!(self.requests.len() < 1024, "DNS fixture query budget");
            let request = Message::from_vec(&bytes[..n]).unwrap();
            self.requests.push(from);
            self.queries.push((
                request.queries[0].name().to_ascii(),
                request.queries[0].query_type(),
            ));
            if let Some(response) = answer(&request) {
                self.socket
                    .send_to(&response.to_vec().unwrap(), from)
                    .unwrap();
            }
        }
    }
}
pub(super) fn response(request: &Message) -> Message {
    let mut response = Message::response(request.metadata.id, OpCode::Query);
    response.queries = request.queries.clone();
    response
}
pub(super) fn drive<E: ScriptEngine>(
    rt: &mut Runtime<E>,
    expression: &str,
    mut io: impl FnMut(&mut Runtime<E>),
) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while read(rt, expression) != "true" {
        assert!(
            Instant::now() < deadline,
            "DNS browser deadline: {}",
            read(rt, "problem")
        );
        io(rt);
        rt.pump_webrtc(16);
        let now = rt.clock_elapsed_ms();
        rt.run_timers(16, now).unwrap();
        rt.run_microtasks();
        std::thread::sleep(Duration::from_millis(1));
    }
}
