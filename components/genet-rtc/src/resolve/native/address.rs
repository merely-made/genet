// SPDX-License-Identifier: MPL-2.0
use super::*;
use hickory_proto::rr::{RData, Record};

pub(super) struct AddressQuery {
    pub query: query::Query,
    pub family: Family,
    pub limit: usize,
}
pub(super) fn addresses(records: &[Record], family: Family, limit: usize) -> Vec<IpAddr> {
    let mut result = Vec::new();
    for record in records {
        let ip = match &record.data {
            RData::A(a) => IpAddr::V4(a.0),
            RData::AAAA(a) => IpAddr::V6(a.0),
            _ => continue,
        };
        if family.accepts(ip)
            && !ip.is_unspecified()
            && !result.contains(&ip)
            && result.len() < limit
        {
            result.push(ip);
        }
    }
    result
}
impl Lookup for AddressQuery {
    fn next_deadline(&self) -> Instant {
        self.query.next_deadline()
    }
    fn poll(&mut self, now: Instant) -> Poll<Result<Vec<IpAddr>, ResolveError>> {
        match self.query.poll(now) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(records) => Poll::Ready(records.and_then(|records| {
                let values = addresses(&records, self.family, self.limit);
                if values.is_empty() {
                    Err(ResolveError::NotFound)
                } else {
                    Ok(values)
                }
            })),
        }
    }
}
