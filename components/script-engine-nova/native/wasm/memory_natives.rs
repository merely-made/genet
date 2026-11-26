// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Memory`: limits, growth, and the buffers script reads it
//! through, fixed-length or resizable (`genet-wasm`'s `memory.rs`). The
//! bootstrap holds the buffers and decides which to detach; these natives make
//! a buffer, detach one, or repoint one.

use std::ptr::NonNull;

use nova_vm::ecmascript::{
    Agent, ArgumentsList, ArrayBuffer, DetachKey, ExceptionType, JsResult, ResizeArrayBufferResult,
    Value,
};
use nova_vm::engine::{Bindable, GcScope, NoGcScope};

use super::bridge::{error, handle_arg, host, number, require_host, string, type_error};
use super::entities_natives::{maximum_arg, u64_arg};
use genet_wasm::{PAGE, json};

/// `memoryNew(initial, maximum?, is64)`: a memory handle; bad limits throw
/// `RangeError`.
pub(super) fn memory_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let initial = u64_arg(agent, args.get(0));
    let maximum = maximum_arg(agent, args.get(1));
    let (Some(initial), Ok(maximum)) = (initial, maximum) else {
        return Err(type_error(agent, "expected page counts", gc));
    };
    match host.memory_new(initial, maximum, args.get(2) == Value::Boolean(true)) {
        Ok(handle) => Ok(number(agent, handle as f64, gc)),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `memoryGrow(memory, delta)`: the old size in pages. The bootstrap then
/// detaches the buffer, even when no page was added.
pub(super) fn memory_grow<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let memory = handle_arg(agent, args.get(0));
    let Some((memory, delta)) = memory.zip(u64_arg(agent, args.get(1))) else {
        return Err(type_error(agent, "expected a memory and a page count", gc));
    };
    match host.memory_grow(memory, delta) {
        Ok(old) => Ok(number(agent, old as f64, gc)),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `memoryBuffer(memory, resizable)`: a new buffer over the memory's bytes,
/// resizable up to the memory's maximum if asked, recorded as the memory's
/// current one. A memory without a maximum throws `TypeError` for a resizable
/// buffer.
pub(super) fn memory_buffer<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some(memory) = handle_arg(agent, args.get(0)) else {
        return Err(type_error(agent, "not a memory handle", gc));
    };
    let max = if args.get(1) == Value::Boolean(true) {
        let maximum = host.memory_type(memory).and_then(|ty| ty.maximum());
        let bytes = maximum.and_then(|pages| {
            pages
                .checked_mul(PAGE)
                .and_then(|bytes| usize::try_from(bytes).ok())
        });
        let Some(bytes) = bytes else {
            return Err(type_error(
                agent,
                "a memory without a maximum cannot have a resizable buffer",
                gc,
            ));
        };
        Some(bytes)
    } else {
        None
    };
    let Some((ptr, len)) = host.memory_bytes(memory) else {
        return Err(type_error(agent, "not a memory handle", gc));
    };
    let at = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the bytes are the store's until it is rebuilt, which happens only
    // when no buffer is attached, and every path that can move them detaches or
    // repoints this buffer before script runs (the bootstrap's `sync`).
    let buffer = unsafe {
        match max {
            Some(max) => {
                ArrayBuffer::new_over_foreign_resizable(agent, at, len, max, DetachKey {}, gc)
            },
            None => ArrayBuffer::new_over_foreign(agent, at, len, DetachKey {}, gc),
        }
    };
    let Some(buffer) = buffer else {
        return Err(type_error(
            agent,
            "memory is not aligned for typed arrays",
            gc,
        ));
    };
    host.set_record(memory, max.is_some(), max.unwrap_or(0));
    Ok(buffer.into())
}

/// `stale(force)`: the buffers to bring up to date as `d<memory>` (detach)
/// and `r<memory>` (repoint), comma-separated. `force`'s fixed-length buffer
/// is detached whether or not its memory moved; pass -1 for none.
pub(super) fn stale<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let list: Vec<String> = host
        .stale(handle_arg(agent, args.get(0)))
        .into_iter()
        .map(|(memory, detach)| format!("{}{memory}", if detach { 'd' } else { 'r' }))
        .collect();
    Ok(string(agent, list.join(","), gc))
}

/// `detach(buffer)`: detach a buffer over a memory.
pub(super) fn detach<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    if let Ok(buffer) = ArrayBuffer::try_from(args.get(0)) {
        // The key is ours, so this detach cannot fail.
        let _ = buffer.detach(agent, Some(DetachKey {}), gc);
    }
    Ok(Value::Undefined)
}

/// `repoint(memory, buffer)`: point a resizable buffer at its memory's bytes.
pub(super) fn repoint<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let bytes = handle_arg(agent, args.get(0)).and_then(|m| host.memory_bytes(m));
    if let (Some((ptr, len)), Ok(buffer)) = (bytes, ArrayBuffer::try_from(args.get(1))) {
        let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
        // SAFETY: the store's bytes, which the bootstrap follows every time
        // they can move.
        let repointed = unsafe { buffer.repoint_foreign(agent, ptr, len) };
        debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
    }
    Ok(Value::Undefined)
}

/// HostResizeArrayBuffer for a memory's resizable buffer: `resize` grows the
/// memory by whole pages and never shrinks it. Any other buffer is not ours.
pub(crate) fn resize_hook<'gc>(
    agent: &mut Agent,
    buffer: ArrayBuffer,
    new_byte_length: usize,
    gc: NoGcScope<'gc, '_>,
) -> JsResult<'gc, ResizeArrayBufferResult> {
    let Some(host) = host(agent) else {
        return Ok(ResizeArrayBufferResult::Unhandled);
    };
    let buffer = buffer.bind(gc);
    let ptr = buffer.as_slice(agent).as_ptr();
    let (len, max) = (buffer.byte_length(agent), buffer.max_byte_length(agent));
    let found = host.resizable_memory(ptr, len, Some(max));
    let range = |agent: &mut Agent, message| error(agent, ExceptionType::RangeError, message, gc);
    let Some(memory) = found else {
        return Ok(ResizeArrayBufferResult::Unhandled);
    };
    let current = host.memory_bytes(memory).map_or(0, |(_, len)| len);
    if !new_byte_length.is_multiple_of(PAGE as usize) {
        return Err(range(agent, "a memory's buffer resizes by whole pages"));
    }
    if new_byte_length < current {
        return Err(range(agent, "a memory's buffer cannot shrink"));
    }
    let delta = ((new_byte_length - current) / PAGE as usize) as u64;
    if let Err(message) = host.memory_grow(memory, delta) {
        return Err(range(agent, &message));
    }
    let (ptr, len) = host.memory_bytes(memory).expect("the memory just grew");
    let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the memory's new bytes, which this buffer is over.
    let repointed = unsafe { buffer.repoint_foreign(agent, ptr, len) };
    debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
    host.sync_record(memory);
    Ok(ResizeArrayBufferResult::Handled)
}

/// `memoryType(memory)`: its limits as JSON.
pub(super) fn memory_type<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let Some((ty, (_, bytes))) = handle.and_then(|h| host.memory_type(h).zip(host.memory_bytes(h)))
    else {
        return Err(type_error(agent, "not a memory handle", gc));
    };
    let mut out = String::from("{");
    json::memory(&mut out, &ty, (bytes / 65536) as u64);
    out.push('}');
    Ok(string(agent, out, gc))
}
