// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! What the bridge needs from a module's sections before wasmi sees it: the
//! index behind each export (an exported function's `name` is its index, and
//! an exported memory below the imported count is an import passed through),
//! and the start function moved out of instantiation.
//!
//! wasmi runs a start function inside `Instance::new` with a plain call, so an
//! import called there cannot suspend for script. The rewrite drops the start
//! section and exports the start function under [`START_EXPORT`]; the bridge
//! calls it resumably once the instance exists. Malformed bytes are passed
//! through untouched, so wasmi reports the compile error.

/// Private export name for a moved start function. A NUL is valid in an export
/// name, so no module can collide with it by accident and still be shown it.
pub(super) const START_EXPORT: &str = "\0genet-start";

const EXPORT_SECTION: u8 = 7;
const START_SECTION: u8 = 8;
pub(super) const FUNC_KIND: u8 = 0;
pub(super) const MEMORY_KIND: u8 = 2;

/// The parts of a module the bridge uses, and the bytes to compile.
pub(super) struct Prepared {
    pub(super) bytes: Vec<u8>,
    pub(super) has_start: bool,
    /// `(name, kind, index)` for every export, in order.
    pub(super) exports: Vec<(String, u8, u32)>,
}

struct Section {
    id: u8,
    /// Range of the whole section, id byte included.
    whole: std::ops::Range<usize>,
    payload: std::ops::Range<usize>,
}

fn leb_u32(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let mut result: u64 = 0;
    for shift in (0..35).step_by(7) {
        let byte = *bytes.get(*at)?;
        *at += 1;
        result |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return u32::try_from(result).ok();
        }
    }
    None
}

fn put_leb(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn sections(bytes: &[u8]) -> Option<Vec<Section>> {
    if bytes.len() < 8 || &bytes[..4] != b"\0asm" {
        return None;
    }
    let mut at = 8;
    let mut out = Vec::new();
    while at < bytes.len() {
        let start = at;
        let id = bytes[at];
        at += 1;
        let size = leb_u32(bytes, &mut at)? as usize;
        let end = at.checked_add(size).filter(|end| *end <= bytes.len())?;
        out.push(Section {
            id,
            whole: start..end,
            payload: at..end,
        });
        at = end;
    }
    Some(out)
}

fn read_exports(payload: &[u8]) -> Option<Vec<(String, u8, u32)>> {
    let mut at = 0;
    let count = leb_u32(payload, &mut at)?;
    let mut out = Vec::new();
    for _ in 0..count {
        let len = leb_u32(payload, &mut at)? as usize;
        let name = payload.get(at..at.checked_add(len)?)?;
        at += len;
        let kind = *payload.get(at)?;
        at += 1;
        let index = leb_u32(payload, &mut at)?;
        out.push((String::from_utf8(name.to_vec()).ok()?, kind, index));
    }
    (at == payload.len()).then_some(out)
}

fn export_section(entries: &[(String, u8, u32)]) -> Vec<u8> {
    let mut payload = Vec::new();
    put_leb(&mut payload, entries.len() as u32);
    for (name, kind, index) in entries {
        put_leb(&mut payload, name.len() as u32);
        payload.extend_from_slice(name.as_bytes());
        payload.push(*kind);
        put_leb(&mut payload, *index);
    }
    let mut section = vec![EXPORT_SECTION];
    put_leb(&mut section, payload.len() as u32);
    section.extend(payload);
    section
}

/// Read the exports and move the start function, or pass the bytes through.
pub(super) fn prepare(bytes: Vec<u8>) -> Prepared {
    let untouched = |bytes| Prepared {
        bytes,
        has_start: false,
        exports: Vec::new(),
    };
    let Some(list) = sections(&bytes) else {
        return untouched(bytes);
    };
    let export = list.iter().find(|s| s.id == EXPORT_SECTION);
    let mut entries = match export {
        Some(s) => match read_exports(&bytes[s.payload.clone()]) {
            Some(entries) => entries,
            None => return untouched(bytes),
        },
        None => Vec::new(),
    };
    let exports = entries.clone();
    let Some(start) = list.iter().find(|s| s.id == START_SECTION) else {
        return Prepared {
            bytes,
            has_start: false,
            exports,
        };
    };
    let mut at = start.payload.start;
    let Some(start_index) = leb_u32(&bytes, &mut at) else {
        return untouched(bytes);
    };
    entries.push((START_EXPORT.to_string(), FUNC_KIND, start_index));
    let replacement = export_section(&entries);
    // Sections keep their order: the export section goes where it was, or just
    // before the start section it replaces when the module had none.
    let mut out = bytes[..8].to_vec();
    for section in &list {
        match section.id {
            EXPORT_SECTION => out.extend_from_slice(&replacement),
            START_SECTION => {
                if export.is_none() {
                    out.extend_from_slice(&replacement);
                }
            },
            _ => out.extend_from_slice(&bytes[section.whole.clone()]),
        }
    }
    Prepared {
        bytes: out,
        has_start: true,
        exports,
    }
}
