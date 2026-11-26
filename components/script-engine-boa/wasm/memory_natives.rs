// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Memory`, on Bole: limits, growth, and the buffers script
//! reads it through, fixed-length or resizable. The bootstrap holds the
//! buffers and decides which to detach; these natives make a buffer, detach
//! one, or repoint one.

use std::ptr::NonNull;

use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::{Context, JsResult, JsValue};
use genet_wasm::{PAGE, json};

use super::bridge::{
    arg, array_buffer, detach_key, handle_arg, host, number, range_error, require_host, string,
    type_error,
};
use super::entities_natives::{maximum_arg, u64_arg};

/// `memoryNew(initial, maximum?, is64)`: a memory handle.
pub(super) fn memory_new(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let initial = u64_arg(&arg(args, 0));
    let maximum = maximum_arg(&arg(args, 1));
    let (Some(initial), Ok(maximum)) = (initial, maximum) else {
        return Err(type_error("expected page counts"));
    };
    let handle = host
        .memory_new(initial, maximum, arg(args, 2).as_boolean() == Some(true))
        .map_err(|m| range_error(&m))?;
    Ok(number(handle))
}

/// `memoryGrow(memory, delta)`: the old size in pages. The bootstrap then
/// detaches the buffer, even when no page was added.
pub(super) fn memory_grow(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let memory = handle_arg(&arg(args, 0));
    let Some((memory, delta)) = memory.zip(u64_arg(&arg(args, 1))) else {
        return Err(type_error("expected a memory and a page count"));
    };
    let old = host
        .memory_grow(memory, delta)
        .map_err(|m| range_error(&m))?;
    Ok(JsValue::new(old as f64))
}

/// `memoryBuffer(memory, resizable)`: a new buffer over the memory's bytes,
/// resizable up to the memory's maximum if asked, recorded as the memory's
/// current one. A memory without a maximum throws `TypeError` for a resizable
/// buffer.
pub(super) fn memory_buffer(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let memory = handle_arg(&arg(args, 0)).ok_or_else(|| type_error("not a memory handle"))?;
    let max = if arg(args, 1).as_boolean() == Some(true) {
        let Some(max) = host.memory_type(memory).and_then(|ty| ty.maximum()) else {
            return Err(type_error(
                "a memory without a maximum cannot have a resizable buffer",
            ));
        };
        Some(
            max.checked_mul(PAGE)
                .ok_or_else(|| range_error("the maximum is too large for a buffer"))?,
        )
    } else {
        None
    };
    let (ptr, len) = host
        .memory_bytes(memory)
        .ok_or_else(|| type_error("not a memory handle"))?;
    let at = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the bytes are the store's until it is rebuilt, which happens only
    // when no buffer is attached, and every path that can move them detaches or
    // repoints this buffer before script runs (the bootstrap's `sync`).
    let buffer = unsafe { JsArrayBuffer::from_foreign(at, len, max, detach_key(), ctx)? };
    let Some(buffer) = buffer else {
        return Err(type_error("memory is not aligned for typed arrays"));
    };
    host.set_record(memory, max.is_some(), max.map_or(0, |m| m as usize));
    Ok(buffer.into())
}

/// `stale(force)`: the buffers to bring up to date as `d<memory>` (detach)
/// and `r<memory>` (repoint), comma-separated. `force`'s fixed-length buffer
/// is detached whether or not its memory moved; pass -1 for none.
pub(super) fn stale(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let list: Vec<String> = host
        .stale(handle_arg(&arg(args, 0)))
        .into_iter()
        .map(|(memory, detach)| format!("{}{memory}", if detach { 'd' } else { 'r' }))
        .collect();
    Ok(string(list.join(",")))
}

/// `detach(buffer)`: detach a buffer over a memory.
pub(super) fn detach(_: &JsValue, args: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    if let Some(buffer) = array_buffer(&arg(args, 0)) {
        // The key is ours, so this detach cannot fail.
        let _ = buffer.detach_foreign(&detach_key());
    }
    Ok(JsValue::undefined())
}

/// `repoint(memory, buffer)`: point a resizable buffer at its memory's bytes.
pub(super) fn repoint(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let bytes = handle_arg(&arg(args, 0)).and_then(|m| host.memory_bytes(m));
    if let (Some((ptr, len)), Some(buffer)) = (bytes, array_buffer(&arg(args, 1))) {
        let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
        // SAFETY: the store's bytes, which the bootstrap follows every time
        // they can move.
        let repointed = unsafe { buffer.repoint_foreign(ptr, len) };
        debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
    }
    Ok(JsValue::undefined())
}

/// `memoryType(memory)`: its limits as JSON, `minimum` its current size.
pub(super) fn memory_type(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let Some((ty, (_, bytes))) = handle.and_then(|h| host.memory_type(h).zip(host.memory_bytes(h)))
    else {
        return Err(type_error("not a memory handle"));
    };
    let mut out = String::from("{");
    json::memory(&mut out, &ty, bytes as u64 / PAGE);
    out.push('}');
    Ok(string(out))
}

/// HostResizeArrayBuffer for a memory's resizable buffer: `resize` grows the
/// memory by whole pages and never shrinks it. Any other buffer is not ours.
pub(crate) fn resize_hook(
    buffer: &JsArrayBuffer,
    new_byte_length: u64,
    ctx: &mut Context,
) -> JsResult<bool> {
    let Some(host) = host(ctx) else {
        return Ok(false);
    };
    let Some((ptr, len)) = buffer.data().map(|d| (d.as_ptr(), d.len())) else {
        return Ok(false);
    };
    let Some(memory) = host.resizable_memory(ptr, len, None) else {
        return Ok(false);
    };
    let current = host.memory_bytes(memory).map_or(0, |(_, len)| len as u64);
    if !new_byte_length.is_multiple_of(PAGE) {
        return Err(range_error("a memory's buffer resizes by whole pages"));
    }
    if new_byte_length < current {
        return Err(range_error("a memory's buffer cannot shrink"));
    }
    let delta = (new_byte_length - current) / PAGE;
    host.memory_grow(memory, delta)
        .map_err(|m| range_error(&m))?;
    let (ptr, len) = host.memory_bytes(memory).expect("the memory just grew");
    let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the memory's new bytes, which this buffer is over.
    let repointed = unsafe { buffer.repoint_foreign(ptr, len) };
    debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
    host.sync_record(memory);
    Ok(true)
}
