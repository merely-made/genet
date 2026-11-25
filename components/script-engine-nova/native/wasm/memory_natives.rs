// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Memory`: limits, growth, and the buffers script reads it
//! through, fixed-length or resizable (`memory.rs`).

use std::ptr::NonNull;
use std::rc::Rc;

use nova_vm::ecmascript::{
    Agent, ArgumentsList, ArrayBuffer, DetachKey, ExceptionType, JsResult, ResizeArrayBufferResult,
    Value,
};
use nova_vm::engine::{Bindable, GcScope, NoGcScope};

use super::bridge::{
    detach_stale, error, handle_arg, host, number, require_host, root, string, type_error,
};
use super::entities_natives::{maximum_arg, u32_arg};
use super::host::WasmHost;
use super::json;

const PAGE: u64 = 65536;

/// `memoryNew(initial, maximum?)`: a memory handle; bad limits throw
/// `RangeError`.
pub(super) fn memory_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let initial = u32_arg(agent, args.get(0));
    let maximum = maximum_arg(agent, args.get(1));
    let (Some(initial), Ok(maximum)) = (initial, maximum) else {
        return Err(type_error(agent, "expected page counts", gc));
    };
    match host.memory_new(initial, maximum) {
        Ok(handle) => Ok(number(agent, handle as f64, gc)),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `memoryGrow(memory, delta)`: the old size in pages. A success detaches the
/// buffer even when no page was added.
pub(super) fn memory_grow<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let memory = handle_arg(agent, args.get(0));
    let Some((memory, delta)) = memory.zip(u32_arg(agent, args.get(1))) else {
        return Err(type_error(agent, "expected a memory and a page count", gc));
    };
    match host.memory_grow(memory, delta) {
        Ok(old) => {
            detach_stale(agent, &host, Some(memory), gc);
            Ok(number(agent, old as f64, gc))
        },
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// A new buffer over the memory's bytes, resizable up to `max` bytes if given,
/// made the memory's current buffer.
fn make_buffer<'gc>(
    agent: &mut Agent,
    host: &WasmHost,
    memory: usize,
    max: Option<usize>,
    gc: NoGcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let Some((ptr, len)) = host.memory_bytes(memory) else {
        return Err(type_error(agent, "not a memory handle", gc));
    };
    let at = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the bytes are the store's for as long as the engine lives, and
    // every path that can move them detaches or repoints this buffer before
    // script runs.
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
    let value: Value = buffer.into();
    let rooted = root(agent, value, gc);
    host.set_buffer(memory, rooted, (ptr, len), max.is_some());
    Ok(value)
}

/// Detach the memory's current buffer, as switching between a fixed-length
/// and a resizable one does.
fn drop_buffer(agent: &mut Agent, host: &WasmHost, memory: usize, gc: NoGcScope<'_, '_>) {
    if let Some(value) = host.take_buffer(memory) {
        if let Ok(buffer) = ArrayBuffer::try_from(value.get(agent, gc)) {
            let _ = buffer.detach(agent, Some(DetachKey {}), gc);
        }
    }
}

fn memory_arg<'gc>(
    agent: &mut Agent,
    args: &ArgumentsList,
    gc: NoGcScope<'gc, '_>,
) -> JsResult<'gc, (Rc<WasmHost>, usize)> {
    let host = require_host(agent, gc)?;
    let Some(memory) = handle_arg(agent, args.get(0)) else {
        return Err(type_error(agent, "not a memory handle", gc));
    };
    detach_stale(agent, &host, None, gc);
    Ok((host, memory))
}

/// `memoryBuffer(memory)`: the memory's current buffer, fixed-length unless
/// script asked for a resizable one.
pub(super) fn memory_buffer<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let (host, memory) = memory_arg(agent, &args, gc)?;
    match host.with_buffer(memory, |v| v.get(agent, gc)) {
        Some(value) => Ok(value),
        None => make_buffer(agent, &host, memory, None, gc),
    }
}

/// `memoryToFixed(memory)`: `toFixedLengthBuffer`.
pub(super) fn memory_to_fixed<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let (host, memory) = memory_arg(agent, &args, gc)?;
    if host.buffer_is_resizable(memory) == Some(false) {
        if let Some(value) = host.with_buffer(memory, |v| v.get(agent, gc)) {
            return Ok(value);
        }
    }
    drop_buffer(agent, &host, memory, gc);
    make_buffer(agent, &host, memory, None, gc)
}

/// `memoryToResizable(memory)`: `toResizableBuffer`, up to the memory's
/// maximum; a memory without one throws `TypeError`.
pub(super) fn memory_to_resizable<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let (host, memory) = memory_arg(agent, &args, gc)?;
    if host.buffer_is_resizable(memory) == Some(true) {
        if let Some(value) = host.with_buffer(memory, |v| v.get(agent, gc)) {
            return Ok(value);
        }
    }
    let maximum = host.memory_type(memory).and_then(|ty| ty.maximum());
    let Some(max) = maximum.and_then(|pages| usize::try_from(pages * PAGE).ok()) else {
        return Err(type_error(
            agent,
            "a memory without a maximum cannot have a resizable buffer",
            gc,
        ));
    };
    drop_buffer(agent, &host, memory, gc);
    make_buffer(agent, &host, memory, Some(max), gc)
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
    let target: Value = buffer.unbind().into();
    let Some(memory) = host.memory_with_buffer(|v| v.get(agent, gc) == target) else {
        return Ok(ResizeArrayBufferResult::Unhandled);
    };
    let current = host.memory_bytes(memory).map_or(0, |(_, len)| len);
    let range = |agent: &mut Agent, message| error(agent, ExceptionType::RangeError, message, gc);
    if !new_byte_length.is_multiple_of(PAGE as usize) {
        return Err(range(agent, "a memory's buffer resizes by whole pages"));
    }
    if new_byte_length < current {
        return Err(range(agent, "a memory's buffer cannot shrink"));
    }
    let delta = u32::try_from((new_byte_length - current) / PAGE as usize).unwrap_or(u32::MAX);
    if let Err(message) = host.memory_grow(memory, delta) {
        return Err(range(agent, &message));
    }
    detach_stale(agent, &host, Some(memory), gc);
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
