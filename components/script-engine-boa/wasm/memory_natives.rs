// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Memory`, on Bole: limits, growth, and the buffers script
//! reads it through, fixed-length or resizable.

use std::ptr::NonNull;

use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::{Context, JsResult, JsValue};
use genet_wasm::{PAGE, json};

use super::WasmHost;
use super::bridge::{
    arg, detach_key, detach_stale, handle_arg, host, number, range_error, require_host, string,
    take_buffer_and_detach, type_error, u32_arg,
};
use super::entities_natives::maximum_arg;

/// `memoryNew(initial, maximum?)`: a memory handle.
pub(super) fn memory_new(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let initial = u32_arg(&arg(args, 0));
    let maximum = maximum_arg(&arg(args, 1));
    let (Some(initial), Ok(maximum)) = (initial, maximum) else {
        return Err(type_error("expected page counts"));
    };
    let handle = host
        .memory_new(initial, maximum)
        .map_err(|m| range_error(&m))?;
    Ok(number(handle))
}

/// `memoryGrow(memory, delta)`: the old size in pages.
pub(super) fn memory_grow(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let memory = handle_arg(&arg(args, 0));
    let Some((memory, delta)) = memory.zip(u32_arg(&arg(args, 1))) else {
        return Err(type_error("expected a memory and a page count"));
    };
    let old = host
        .memory_grow(memory, delta)
        .map_err(|m| range_error(&m))?;
    detach_stale(&host, Some(memory));
    Ok(JsValue::new(old as f64))
}

/// A new buffer over the memory's bytes, resizable up to `max` bytes if given,
/// made the memory's current buffer.
fn make_buffer(
    host: &WasmHost,
    memory: usize,
    max: Option<u64>,
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let (ptr, len) = host
        .memory_bytes(memory)
        .ok_or_else(|| type_error("not a memory handle"))?;
    let at = NonNull::new(ptr).unwrap_or(NonNull::dangling());
    // SAFETY: the bytes are the store's for as long as the engine lives, and
    // every path that can move them detaches or repoints this buffer before
    // script runs.
    let buffer = unsafe { JsArrayBuffer::from_foreign(at, len, max, detach_key(), ctx)? };
    let Some(buffer) = buffer else {
        return Err(type_error("memory is not aligned for typed arrays"));
    };
    let value: JsValue = buffer.into();
    host.set_buffer(memory, value.clone(), (ptr, len), max.is_some());
    Ok(value)
}

fn memory_arg(args: &[JsValue], ctx: &Context) -> JsResult<(std::rc::Rc<WasmHost>, usize)> {
    let host = require_host(ctx)?;
    let memory = handle_arg(&arg(args, 0)).ok_or_else(|| type_error("not a memory handle"))?;
    detach_stale(&host, None);
    Ok((host, memory))
}

/// `memoryBuffer(memory)`: the memory's current buffer.
pub(super) fn memory_buffer(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let (host, memory) = memory_arg(args, ctx)?;
    match host.with_buffer(memory, Clone::clone) {
        Some(value) => Ok(value),
        None => make_buffer(&host, memory, None, ctx),
    }
}

/// `memoryToFixed(memory)`: `toFixedLengthBuffer`.
pub(super) fn memory_to_fixed(
    _: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let (host, memory) = memory_arg(args, ctx)?;
    if host.buffer_is_resizable(memory) == Some(false) {
        if let Some(value) = host.with_buffer(memory, Clone::clone) {
            return Ok(value);
        }
    }
    take_buffer_and_detach(&host, memory);
    make_buffer(&host, memory, None, ctx)
}

/// `memoryToResizable(memory)`: `toResizableBuffer`, up to the maximum.
pub(super) fn memory_to_resizable(
    _: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let (host, memory) = memory_arg(args, ctx)?;
    if host.buffer_is_resizable(memory) == Some(true) {
        if let Some(value) = host.with_buffer(memory, Clone::clone) {
            return Ok(value);
        }
    }
    let Some(max) = host.memory_type(memory).and_then(|ty| ty.maximum()) else {
        return Err(type_error(
            "a memory without a maximum cannot have a resizable buffer",
        ));
    };
    take_buffer_and_detach(&host, memory);
    make_buffer(&host, memory, Some(max * PAGE), ctx)
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
    let target: JsValue = buffer.clone().into();
    let Some(memory) = host.memory_with_buffer(|v| JsValue::same_value(v, &target)) else {
        return Ok(false);
    };
    let current = host.memory_bytes(memory).map_or(0, |(_, len)| len as u64);
    if !new_byte_length.is_multiple_of(PAGE) {
        return Err(range_error("a memory's buffer resizes by whole pages"));
    }
    if new_byte_length < current {
        return Err(range_error("a memory's buffer cannot shrink"));
    }
    let delta = u32::try_from((new_byte_length - current) / PAGE).unwrap_or(u32::MAX);
    host.memory_grow(memory, delta)
        .map_err(|m| range_error(&m))?;
    detach_stale(&host, Some(memory));
    Ok(true)
}
