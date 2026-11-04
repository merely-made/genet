// SPDX-License-Identifier: MPL-2.0
use super::*;
use hickory_proto::op::{Message, MessageType, OpCode, Query, ResponseCode};
use hickory_proto::rr::{DNSClass, Name, RData, RecordType};

pub(super) enum Answer {
    Addresses(Vec<IpAddr>, Vec<Name>),
    Alias(Vec<Name>),
    Truncated,
    Ignore,
}
pub(super) fn name(host: &str) -> Result<Name, ResolveError> {
    let lower = host.trim_end_matches('.').to_ascii_lowercase();
    if lower == "local" || lower.ends_with(".local") {
        return Err(ResolveError::Invalid);
    }
    let mut name = Name::from_utf8(host).map_err(|_| ResolveError::Invalid)?;
    // ICE URLs name an absolute DNS host. Wire-decoded questions carry the
    // root label too; preserve that identity when validating the reply.
    name.set_fqdn(true);
    Ok(name)
}
pub(super) fn request(name: &Name, family: Family) -> Result<(Message, Vec<u8>), ResolveError> {
    let mut nonce = [0; 2];
    getrandom::fill(&mut nonce).map_err(|_| ResolveError::Unavailable)?;
    let mut message = Message::new(u16::from_ne_bytes(nonce), MessageType::Query, OpCode::Query);
    message.metadata.recursion_desired = true;
    message.queries.push(Query::query(
        name.clone(),
        if family == Family::V4 {
            RecordType::A
        } else {
            RecordType::AAAA
        },
    ));
    let bytes = message.to_vec().map_err(|_| ResolveError::Invalid)?;
    Ok((message, bytes))
}
pub(super) fn answer(
    bytes: &[u8],
    request: &Message,
    family: Family,
    limits: DnsLimits,
) -> Result<Answer, ResolveError> {
    if bytes.len() < 12 || bytes.len() > limits.max_response_bytes {
        return Ok(Answer::Ignore);
    }
    let count = |n| usize::from(u16::from_be_bytes([bytes[n], bytes[n + 1]]));
    if count(4) != 1 || count(6) + count(8) + count(10) > limits.max_records {
        return Ok(Answer::Ignore);
    }
    let Ok(message) = Message::from_vec(bytes) else {
        return Ok(Answer::Ignore);
    };
    if message.metadata.id != request.metadata.id
        || message.metadata.message_type != MessageType::Response
        || message.metadata.op_code != OpCode::Query
        || message.queries != request.queries
    {
        return Ok(Answer::Ignore);
    }
    if message.metadata.truncation {
        return Ok(Answer::Truncated);
    }
    if message.metadata.response_code == ResponseCode::NXDomain {
        return Err(ResolveError::NotFound);
    }
    if message.metadata.response_code != ResponseCode::NoError {
        return Err(ResolveError::Io);
    }
    let mut current = request.queries[0].name().clone();
    let mut seen = vec![current.clone()];
    for _ in 0..=limits.max_aliases {
        let mut addresses = Vec::new();
        for record in &message.answers {
            if record.name != current || record.dns_class != DNSClass::IN {
                continue;
            }
            let ip = match &record.data {
                RData::A(a) => IpAddr::V4(a.0),
                RData::AAAA(a) => IpAddr::V6(a.0),
                _ => continue,
            };
            if family.accepts(ip)
                && !ip.is_unspecified()
                && !addresses.contains(&ip)
                && addresses.len() < limits.max_addresses
            {
                addresses.push(ip);
            }
        }
        if !addresses.is_empty() {
            return Ok(Answer::Addresses(addresses, seen));
        }
        let alias = message.answers.iter().find_map(|record| {
            if record.name == current
                && record.dns_class == DNSClass::IN
                && let RData::CNAME(alias) = &record.data
            {
                return Some(alias.0.clone());
            }
            None
        });
        let Some(alias) = alias else {
            return if current != request.queries[0].name().clone() {
                Ok(Answer::Alias(seen))
            } else {
                Err(ResolveError::NotFound)
            };
        };
        if seen.contains(&alias) {
            return Err(ResolveError::Invalid);
        }
        seen.push(alias.clone());
        current = alias;
    }
    Err(ResolveError::Limit)
}
