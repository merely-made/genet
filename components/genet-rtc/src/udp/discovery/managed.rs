// SPDX-License-Identifier: MPL-2.0
use super::*;

impl Managed {
    pub(super) fn new(
        url: Url,
        address: Option<SocketAddr>,
        port: Option<u16>,
        transport_specified: bool,
        local: SocketAddr,
        config: &DiscoveryConfig,
    ) -> Result<Self, Error> {
        if url.proto != ProtoType::Udp
            || !matches!(url.scheme, SchemeType::Stun | SchemeType::Turn)
            || url.port == 0
            || url.host.is_empty()
            || port.is_some_and(|p| p == 0 || p != url.port)
            || address.is_some_and(|a| {
                a.ip().is_unspecified() || a.port() == 0 || a.is_ipv4() != local.is_ipv4()
            })
        {
            return Err(Error::Configuration("usable UDP ICE server required"));
        }
        let turn = url.scheme == SchemeType::Turn;
        let client = address
            .map(|a| Self::client(&url, a, local, config))
            .transpose()?;
        Ok(Self {
            url: NamedServer::metadata(&url, port, transport_specified),
            request: ServerRequest {
                host: url.host.clone(),
                service: if turn { Service::Turn } else { Service::Stun },
                port,
                transport_specified,
            },
            remaining_targets: 0,
            seen_addresses: Vec::new(),
            spec: url,
            address,
            client,
            turn,
            relay: None,
            reflexive: None,
            done: false,
            started: false,
            addresses: VecDeque::new(),
            lookup: None,
            attempt_deadline: None,
            failed: false,
        })
    }
    fn client(
        url: &Url,
        address: SocketAddr,
        local: SocketAddr,
        config: &DiscoveryConfig,
    ) -> Result<Client, Error> {
        let turn = url.scheme == SchemeType::Turn;
        Ok(Client::new(
            ClientConfig {
                stun_serv_addr: if turn {
                    String::new()
                } else {
                    address.to_string()
                },
                turn_serv_addr: if turn {
                    address.to_string()
                } else {
                    String::new()
                },
                local_addr: local,
                transport_protocol: TransportProtocol::UDP,
                username: url.username.clone(),
                password: url.password.clone(),
                allocation_refresh_interval_cap: config.allocation_refresh_interval_cap,
                ..Default::default()
            },
            config.crypto_provider.clone(),
        )?)
    }
    pub(super) fn begin(
        &mut self,
        local: SocketAddr,
        config: &DiscoveryConfig,
        now: Instant,
        deadline: Instant,
    ) -> Result<(), Error> {
        if self.started {
            return Ok(());
        }
        self.started = true;
        if self.client.is_some() {
            self.start_transaction(now, deadline)?;
        } else {
            let family = if local.is_ipv4() {
                Family::V4
            } else {
                Family::V6
            };
            match config
                .resolver
                .as_ref()
                .map(|r| r.start_server(&self.request, family, now, deadline))
            {
                Some(Ok(lookup)) => self.lookup = Some(lookup),
                _ => self.fail(),
            }
        }
        Ok(())
    }
    fn start_transaction(&mut self, now: Instant, deadline: Instant) -> Result<(), Error> {
        // Share the remaining gathering budget among untried addresses. A
        // blackholed first address cannot consume every alternative's time.
        let share = deadline.saturating_duration_since(now)
            / (self.addresses.len() as u32 + 1 + self.remaining_targets.min(32) as u32);
        self.attempt_deadline = now.checked_add(share).map(|d| d.min(deadline));
        if let Some(client) = &mut self.client {
            if self.turn {
                client.allocate(now)?;
            } else {
                client.send_binding_request(now)?;
            }
        }
        Ok(())
    }
    pub(super) fn next_address(
        &mut self,
        local: SocketAddr,
        config: &DiscoveryConfig,
        now: Instant,
        deadline: Instant,
        blocked: &[SocketAddr],
    ) {
        self.client = None;
        self.address = None;
        self.attempt_deadline = None;
        while let Some(address) = self.addresses.pop_front() {
            if self.turn && blocked.contains(&address) {
                continue;
            }
            if let Ok(client) = Self::client(&self.spec, address, local, config) {
                self.client = Some(client);
                self.address = Some(address);
                if self.start_transaction(now, deadline).is_ok() {
                    return;
                }
                self.client = None;
                self.address = None;
            }
        }
        if self.lookup.is_none() {
            self.fail();
        }
    }
    pub(super) fn fail(&mut self) {
        self.lookup = None;
        self.client = None;
        self.address = None;
        self.attempt_deadline = None;
        self.done = true;
        self.failed = true;
    }
    pub(super) fn progress(
        &mut self,
        local: SocketAddr,
        config: &DiscoveryConfig,
        now: Instant,
        deadline: Instant,
        blocked: &[SocketAddr],
    ) {
        if self.done {
            return;
        }
        if self.client.is_none()
            && let Some(lookup) = &mut self.lookup
        {
            match lookup.poll_next(now) {
                Poll::Pending => {}
                Poll::Ready(Ok(Some(batch))) => {
                    self.remaining_targets = batch.remaining_targets.min(32);
                    for address in batch.addresses {
                        if address.ip().is_unspecified()
                            || address.port() == 0
                            || address.is_ipv4() != local.is_ipv4()
                            || self.seen_addresses.contains(&address)
                        {
                            continue;
                        }
                        if self.seen_addresses.len() == 32 {
                            break;
                        }
                        self.seen_addresses.push(address);
                        self.addresses.push_back(address);
                    }
                    if self.seen_addresses.len() == 32 {
                        self.lookup = None;
                        self.remaining_targets = 0;
                    }
                    self.next_address(local, config, now, deadline, blocked);
                }
                Poll::Ready(Ok(None) | Err(_)) => self.fail(),
            }
        }
        if !self.done && self.attempt_deadline.is_some_and(|d| now >= d) {
            self.next_address(local, config, now, deadline, blocked);
        }
    }
}
