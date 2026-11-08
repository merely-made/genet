// SPDX-License-Identifier: MPL-2.0
use super::*;
use hickory_proto::rr::{Name, Record, RecordType};
use std::collections::VecDeque;

mod records;
#[cfg(test)]
mod tests;

#[derive(Clone)]
enum Task {
    Naptr {
        name: Name,
        path: Vec<Name>,
        initial: bool,
    },
    Srv {
        name: Name,
        fallback: Option<Name>,
    },
    Address {
        name: Name,
        port: u16,
        allow_alias: bool,
    },
}
impl Task {
    fn query(&self, family: Family) -> (&Name, RecordType, bool) {
        match self {
            Self::Naptr { name, .. } => (name, RecordType::NAPTR, true),
            Self::Srv { name, .. } => (name, RecordType::SRV, true),
            Self::Address {
                name, allow_alias, ..
            } => (
                name,
                if family == Family::V4 {
                    RecordType::A
                } else {
                    RecordType::AAAA
                },
                *allow_alias,
            ),
        }
    }
}
pub(super) struct Cursor {
    resolver: DnsResolver,
    family: Family,
    deadline: Instant,
    wake: Instant,
    tasks: VecDeque<Task>,
    active: Option<(Task, query::Query)>,
    queries: usize,
    endpoints: Vec<SocketAddr>,
    lease: Option<Lease>,
    finished: bool,
}
impl Cursor {
    pub fn new(
        resolver: DnsResolver,
        request: &ServerRequest,
        name: Name,
        family: Family,
        now: Instant,
        deadline: Instant,
        lease: Lease,
    ) -> Result<Self, ResolveError> {
        if name.is_root() {
            return Err(ResolveError::Invalid);
        }
        let task = if request.service == Service::Stun || request.transport_specified {
            Self::srv_task(name, request.service)?
        } else {
            Task::Naptr {
                name: name.clone(),
                path: vec![name],
                initial: true,
            }
        };
        Ok(Self {
            resolver,
            family,
            deadline,
            wake: now,
            tasks: VecDeque::from([task]),
            active: None,
            queries: 0,
            endpoints: Vec::new(),
            lease: Some(lease),
            finished: false,
        })
    }
    fn srv_task(name: Name, service: Service) -> Result<Task, ResolveError> {
        let prefix = if service == Service::Stun {
            "stun"
        } else {
            "turn"
        };
        let srv = Name::from_ascii(format!("_{prefix}._udp.{}", name.to_ascii()))
            .map_err(|_| ResolveError::Invalid)?;
        Ok(Task::Srv {
            name: srv,
            fallback: Some(name),
        })
    }
    fn finish(
        &mut self,
        result: Result<Option<EndpointBatch>, ResolveError>,
    ) -> Poll<Result<Option<EndpointBatch>, ResolveError>> {
        self.active = None;
        self.tasks.clear();
        self.lease = None;
        self.finished = true;
        Poll::Ready(result)
    }
    fn push(&mut self, tasks: Vec<Task>) -> Result<(), ResolveError> {
        if self.tasks.len() + tasks.len() > self.resolver.config.limits.max_targets {
            return Err(ResolveError::Limit);
        }
        for task in tasks.into_iter().rev() {
            self.tasks.push_front(task);
        }
        Ok(())
    }
    fn begin(&mut self, task: Task, now: Instant) -> Result<(), ResolveError> {
        let limits = self.resolver.config.limits;
        if self.queries >= limits.max_service_queries {
            return Err(ResolveError::Limit);
        }
        self.queries += 1;
        // Reserve time for untried branches and an actual protocol attempt.
        let share = self.deadline.saturating_duration_since(now) / (self.tasks.len() as u32 + 2);
        let end = now
            .checked_add(limits.timeout.min(share))
            .ok_or(ResolveError::Limit)?
            .min(self.deadline);
        let (name, record_type, allow_alias) = task.query(self.family);
        let query = query::Query::new(
            self.resolver.config.clone(),
            name.clone(),
            record_type,
            allow_alias,
            now,
            end,
            None,
        )?;
        self.active = Some((task, query));
        Ok(())
    }
    fn process(
        &mut self,
        task: Task,
        result: Result<Vec<Record>, ResolveError>,
    ) -> Result<Option<EndpointBatch>, ResolveError> {
        if matches!(result, Err(ResolveError::Limit | ResolveError::Unavailable)) {
            return Err(result.err().unwrap());
        }
        match task {
            Task::Naptr {
                name,
                path,
                initial,
            } => {
                let (tasks, matched) = match result {
                    Ok(records) => records::naptr(&records, &path, self.resolver.config.limits)?,
                    Err(_) => (Vec::new(), false),
                };
                if initial && !matched {
                    self.push(vec![Self::srv_task(name, Service::Turn)?])?;
                } else {
                    self.push(tasks)?;
                }
            }
            Task::Srv { fallback, .. } => match result {
                Ok(records) if !records.is_empty() => self.push(records::srv(&records)?)?,
                _ => {
                    if let Some(name) = fallback {
                        self.push(vec![Task::Address {
                            name,
                            port: 3478,
                            allow_alias: true,
                        }])?;
                    }
                }
            },
            Task::Address { port, .. } => {
                if let Ok(records) = result {
                    let limits = self.resolver.config.limits;
                    let mut addresses = Vec::new();
                    for ip in address::addresses(&records, self.family, limits.max_addresses) {
                        let endpoint = SocketAddr::new(ip, port);
                        if !self.endpoints.contains(&endpoint)
                            && self.endpoints.len() < limits.max_targets
                        {
                            self.endpoints.push(endpoint);
                            addresses.push(endpoint);
                        }
                    }
                    if self.endpoints.len() == limits.max_targets {
                        self.tasks.clear();
                    }
                    if !addresses.is_empty() {
                        if self.tasks.is_empty() {
                            self.lease = None;
                        }
                        return Ok(Some(EndpointBatch {
                            addresses,
                            remaining_targets: self.tasks.len(),
                        }));
                    }
                }
            }
        }
        Ok(None)
    }
}
impl EndpointLookup for Cursor {
    fn next_deadline(&self) -> Option<Instant> {
        if self.finished {
            None
        } else {
            Some(
                self.active
                    .as_ref()
                    .map_or(self.wake, |(_, q)| q.next_deadline())
                    .min(self.deadline),
            )
        }
    }
    fn poll_next(&mut self, now: Instant) -> Poll<Result<Option<EndpointBatch>, ResolveError>> {
        if self.finished {
            return Poll::Ready(Ok(None));
        }
        if now >= self.deadline {
            return self.finish(Err(ResolveError::Timeout));
        }
        self.wake = now
            .checked_add(self.resolver.config.limits.poll_interval)
            .unwrap_or(self.deadline)
            .min(self.deadline);
        for _ in 0..self.resolver.config.limits.packets_per_poll {
            if self.active.is_none() {
                let Some(task) = self.tasks.pop_front() else {
                    return self.finish(Ok(None));
                };
                if let Err(error) = self.begin(task.clone(), now) {
                    if let Err(e) = self.process(task, Err(error)) {
                        return self.finish(Err(e));
                    }
                    continue;
                }
            }
            let result = self
                .active
                .as_mut()
                .expect("active service query")
                .1
                .poll(now);
            let Poll::Ready(result) = result else {
                return Poll::Pending;
            };
            let task = self.active.take().expect("completed service query").0;
            match self.process(task, result) {
                Ok(Some(batch)) => return Poll::Ready(Ok(Some(batch))),
                Ok(None) => {}
                Err(error) => return self.finish(Err(error)),
            }
        }
        Poll::Pending
    }
}
