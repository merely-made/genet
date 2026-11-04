// SPDX-License-Identifier: MPL-2.0
use super::*;
use hickory_proto::op::{Message, MessageType, OpCode, Query, ResponseCode};
use hickory_proto::rr::{DNSClass, Name, RData, Record, RecordType};

pub(super) enum Answer {
    Records(Vec<Record>, Vec<Name>),
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
pub(super) fn request(
    name: &Name,
    record_type: RecordType,
) -> Result<(Message, Vec<u8>), ResolveError> {
    let mut nonce = [0; 2];
    getrandom::fill(&mut nonce).map_err(|_| ResolveError::Unavailable)?;
    let mut message = Message::new(u16::from_ne_bytes(nonce), MessageType::Query, OpCode::Query);
    message.metadata.recursion_desired = true;
    message
        .queries
        .push(Query::query(name.clone(), record_type));
    let bytes = message.to_vec().map_err(|_| ResolveError::Invalid)?;
    Ok((message, bytes))
}
pub(super) fn answer(
    bytes: &[u8],
    request: &Message,
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
        let records: Vec<Record> = message
            .answers
            .iter()
            .filter(|r| {
                r.name == current
                    && r.dns_class == DNSClass::IN
                    && r.record_type() == request.queries[0].query_type()
            })
            .cloned()
            .collect();
        if !records.is_empty() {
            return Ok(Answer::Records(records, seen));
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
                Ok(Answer::Records(Vec::new(), seen))
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
