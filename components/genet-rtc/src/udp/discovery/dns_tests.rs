use super::*;
use crate::resolve::{Lookup, ResolveError};
use std::net::IpAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Controlled {
    answer: Mutex<Option<Result<Vec<IpAddr>, ResolveError>>>,
    live: Arc<AtomicUsize>,
    started: AtomicUsize,
}
struct Ticket(Arc<Controlled>);
impl Lookup for Ticket {
    fn next_deadline(&self) -> Instant {
        Instant::now()
    }
    fn poll(&mut self, _: Instant) -> Poll<Result<Vec<IpAddr>, ResolveError>> {
        self.0
            .answer
            .lock()
            .unwrap()
            .clone()
            .map_or(Poll::Pending, Poll::Ready)
    }
}
impl Drop for Ticket {
    fn drop(&mut self) {
        self.0.live.fetch_sub(1, Ordering::SeqCst);
    }
}
struct ResolverHandle(Arc<Controlled>);
impl Resolver for ResolverHandle {
    fn start(
        &self,
        _: &str,
        family: Family,
        _: Instant,
        _: Instant,
    ) -> Result<Box<dyn Lookup>, ResolveError> {
        assert_eq!(family, Family::V4);
        self.0.started.fetch_add(1, Ordering::SeqCst);
        self.0.live.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Ticket(self.0.clone())))
    }
}
fn setup(turn: bool, count: usize) -> (Discovery, Arc<Controlled>, SocketAddr, Instant) {
    let control = Arc::new(Controlled::default());
    let local = "127.0.0.1:10000".parse().unwrap();
    let names = (0..count)
        .map(|i| {
            Url::parse_url(&format!(
                "{}:rtc{i}.example.test:3478",
                if turn { "turn" } else { "stun" }
            ))
            .unwrap()
            .into()
        })
        .collect();
    let discovery = Discovery::new(
        local,
        DiscoveryConfig {
            servers: Vec::new(),
            named_servers: names,
            resolver: Some(Arc::new(ResolverHandle(control.clone()))),
            relay_only: turn,
            timeout: Duration::from_secs(1),
            allocation_refresh_interval_cap: None,
            crypto_provider: rtc::crypto::default_provider().unwrap(),
        },
    )
    .unwrap();
    (discovery, control, local, Instant::now())
}

#[test]
fn pending_names_do_not_finish_gathering_and_restart_close_drop_cancel_tickets() {
    let (mut discovery, control, local, now) = setup(false, 1);
    assert_eq!(control.started.load(Ordering::SeqCst), 0);
    discovery.start(local, now).unwrap();
    discovery.progress(now);
    assert!(discovery.poll_write().is_none());
    assert!(discovery.gathering_pending());
    assert!(!discovery.clients[0].done);
    assert!(discovery.next_deadline().is_some());
    discovery.restart(local, now).unwrap();
    assert_eq!(control.started.load(Ordering::SeqCst), 2);
    assert_eq!(control.live.load(Ordering::SeqCst), 1);
    discovery.close();
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
    discovery.restart(local, now).unwrap();
    assert_eq!(control.live.load(Ordering::SeqCst), 1);
    drop(discovery);
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
}

#[test]
fn address_fallback_is_bounded_deduplicated_and_reports_one_original_url_error() {
    let (mut discovery, control, local, now) = setup(false, 1);
    *control.answer.lock().unwrap() = Some(Ok(vec![
        "::1".parse().unwrap(),
        "0.0.0.0".parse().unwrap(),
        "127.0.0.2".parse().unwrap(),
        "127.0.0.2".parse().unwrap(),
        "127.0.0.3".parse().unwrap(),
    ]));
    discovery.start(local, now).unwrap();
    discovery.progress(now);
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
    assert_eq!(
        discovery.poll_write().unwrap().transport.peer_addr,
        "127.0.0.2:3478".parse::<SocketAddr>().unwrap()
    );
    assert!(discovery.poll_error().is_none());
    // A silent first endpoint gets half the remaining budget.
    discovery.progress(now + Duration::from_millis(501));
    assert_eq!(
        discovery.poll_write().unwrap().transport.peer_addr,
        "127.0.0.3:3478".parse::<SocketAddr>().unwrap()
    );
    assert!(discovery.poll_error().is_none());
    discovery.progress(now + Duration::from_secs(1));
    assert_eq!(
        discovery.poll_error().as_deref(),
        Some("stun:rtc0.example.test:3478")
    );
    discovery.progress(now + Duration::from_secs(2));
    assert!(discovery.poll_error().is_none());
    assert!(discovery.clients[0].done);
}

#[test]
fn pending_dns_deadline_and_lookup_failure_produce_one_terminal_error() {
    for answer in [
        None,
        Some(Err(ResolveError::NotFound)),
        Some(Ok(Vec::new())),
    ] {
        let (mut discovery, control, local, now) = setup(true, 1);
        *control.answer.lock().unwrap() = answer;
        discovery.start(local, now).unwrap();
        discovery.progress(now);
        discovery.progress(now + Duration::from_secs(1));
        assert_eq!(control.live.load(Ordering::SeqCst), 0);
        assert_eq!(
            discovery.poll_error().as_deref(),
            Some("turn:rtc0.example.test:3478?transport=udp")
        );
        assert!(discovery.poll_error().is_none());
        assert!(discovery.poll_write().is_none());
    }
}

#[test]
fn dns_aliases_cannot_allocate_twice_on_the_same_turn_five_tuple() {
    let (mut discovery, control, local, now) = setup(true, 2);
    *control.answer.lock().unwrap() = Some(Ok(vec![
        "127.0.0.2".parse().unwrap(),
        "127.0.0.3".parse().unwrap(),
    ]));
    discovery.start(local, now).unwrap();
    discovery.progress(now);
    let a = discovery.poll_write().unwrap().transport.peer_addr;
    let b = discovery.poll_write().unwrap().transport.peer_addr;
    assert_ne!(a, b);
    assert!(discovery.poll_write().is_none());
    assert_eq!(control.live.load(Ordering::SeqCst), 0);
}
