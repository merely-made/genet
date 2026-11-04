// SPDX-License-Identifier: MPL-2.0
use super::*;
use hickory_proto::rr::{RData, rdata::SRV};

fn target(name: &Name) -> bool {
    !name.is_root() && wire::name(&name.to_ascii()).is_ok()
}
pub(super) fn naptr(
    records: &[Record],
    path: &[Name],
    limits: DnsLimits,
) -> Result<(Vec<Task>, bool), ResolveError> {
    let mut rows = Vec::new();
    let mut matched = false;
    for record in records {
        let RData::NAPTR(row) = &record.data else {
            continue;
        };
        let mut tags = row.services.split(|b| *b == b':');
        if !tags
            .next()
            .is_some_and(|t| t.eq_ignore_ascii_case(b"RELAY"))
            || !tags.any(|t| t.eq_ignore_ascii_case(b"turn.udp"))
        {
            continue;
        }
        matched = true;
        if !row.regexp.is_empty() || !target(&row.replacement) {
            continue;
        }
        let task = if row.flags.is_empty() {
            if path.len() >= limits.max_delegations || path.contains(&row.replacement) {
                continue;
            }
            let mut path = path.to_vec();
            path.push(row.replacement.clone());
            Task::Naptr {
                name: row.replacement.clone(),
                path,
                initial: false,
            }
        } else if row.flags.eq_ignore_ascii_case(b"S") {
            Task::Srv {
                name: row.replacement.clone(),
                fallback: None,
            }
        } else if row.flags.eq_ignore_ascii_case(b"A") {
            Task::Address {
                name: row.replacement.clone(),
                port: 3478,
                allow_alias: true,
            }
        } else {
            continue;
        };
        rows.push((row.order, row.preference, task));
        if rows.len() > limits.max_targets {
            return Err(ResolveError::Limit);
        }
    }
    rows.sort_by_key(|(order, pref, _)| (*order, *pref));
    Ok((rows.into_iter().map(|(_, _, task)| task).collect(), matched))
}
pub(super) fn srv(records: &[Record]) -> Result<Vec<Task>, ResolveError> {
    let mut rows = Vec::new();
    for record in records {
        if let RData::SRV(row) = &record.data {
            // A positive unavailable (or malformed mixed) set is not absence.
            if row.target.is_root() {
                return Ok(Vec::new());
            }
            if row.port != 0 && target(&row.target) {
                rows.push(row.clone());
            }
        }
    }
    Ok(weighted(rows, random)?
        .into_iter()
        .map(|row| Task::Address {
            name: row.target,
            port: row.port,
            allow_alias: false,
        })
        .collect())
}
fn random(max: u32) -> Result<u32, ResolveError> {
    if max == 0 {
        return Ok(0);
    }
    let span = max.checked_add(1).ok_or(ResolveError::Limit)?;
    let threshold = span.wrapping_neg() % span;
    loop {
        let mut bytes = [0; 4];
        getrandom::fill(&mut bytes).map_err(|_| ResolveError::Unavailable)?;
        let value = u32::from_ne_bytes(bytes);
        if value >= threshold {
            return Ok(value % span);
        }
    }
}
pub(super) fn weighted(
    mut rows: Vec<SRV>,
    mut draw: impl FnMut(u32) -> Result<u32, ResolveError>,
) -> Result<Vec<SRV>, ResolveError> {
    rows.sort_by_key(|r| (r.priority, r.weight != 0));
    let mut result = Vec::new();
    while let Some(first) = rows.first() {
        let priority = first.priority;
        let count = rows.iter().take_while(|r| r.priority == priority).count();
        let sum = rows[..count].iter().try_fold(0u32, |sum, r| {
            sum.checked_add(u32::from(r.weight))
                .ok_or(ResolveError::Limit)
        })?;
        let chosen = draw(sum)?;
        if chosen > sum {
            return Err(ResolveError::Invalid);
        }
        let mut running = 0;
        let index = rows[..count]
            .iter()
            .position(|r| {
                running += u32::from(r.weight);
                running >= chosen
            })
            .expect("draw within cumulative weight");
        result.push(rows.remove(index));
    }
    Ok(result)
}
