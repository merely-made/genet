// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Values across the boundary, and the call bridge, on Bole. The same design
//! as `genet-vano`'s: wasm is called resumably, so script a wasm call reaches
//! can call back into the same store; buffers over memory that moved are
//! brought up to date whenever control passes from wasm back to script.

use std::ptr::NonNull;
use std::rc::Rc;

use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::{Context, JsBigInt, JsError, JsNativeError, JsResult, JsString, JsValue};
use genet_wasm::ImportCall;
use genet_wasm::wasmi::{
    ExternRef, F32, F64, Func, Nullable, ResumableCall, TrapCode, Val, ValType,
};

use super::{WasmHost, WasmSlot};

const FUEL_EXHAUSTED: &str = "fuel exhausted";

pub(super) enum Outcome {
    Done(Vec<Val>),
    Trap(String),
}

pub(super) fn host(ctx: &Context) -> Option<Rc<WasmHost>> {
    ctx.get_data::<WasmSlot>()?.borrow().clone()
}

pub(super) fn type_error(message: &str) -> JsError {
    JsNativeError::typ()
        .with_message(message.to_string())
        .into()
}

pub(super) fn range_error(message: &str) -> JsError {
    JsNativeError::range()
        .with_message(message.to_string())
        .into()
}

pub(super) fn require_host(ctx: &Context) -> JsResult<Rc<WasmHost>> {
    host(ctx).ok_or_else(|| type_error("WebAssembly is not available"))
}

pub(super) fn arg(args: &[JsValue], i: usize) -> JsValue {
    args.get(i).cloned().unwrap_or_default()
}

pub(super) fn handle_arg(value: &JsValue) -> Option<usize> {
    let n = value.as_number()?;
    (n >= 0.0 && n.fract() == 0.0 && n < usize::MAX as f64).then_some(n as usize)
}

/// A string argument, or `None` if it is not a string or has a lone surrogate.
pub(super) fn string_arg(value: &JsValue) -> Option<String> {
    value.as_string()?.to_std_string().ok()
}

pub(super) fn number(n: usize) -> JsValue {
    JsValue::new(n as f64)
}

pub(super) fn string(s: String) -> JsValue {
    JsValue::from(JsString::from(s))
}

/// The detach key of a memory's buffers, so script can neither transfer nor
/// detach them.
pub(super) fn detach_key() -> JsValue {
    string("WebAssembly.Memory".to_string())
}

fn array_buffer(value: &JsValue) -> Option<JsArrayBuffer> {
    JsArrayBuffer::from_object(value.as_object()?.clone()).ok()
}

/// Bring every buffer over memory up to date before script runs: detach the
/// fixed-length ones whose memory moved, and `force`'s in any case, and
/// repoint the resizable ones at their memory's bytes.
pub(super) fn detach_stale(host: &WasmHost, force: Option<usize>) {
    let refresh = host.refresh(force);
    for value in refresh.detach {
        if let Some(buffer) = array_buffer(&value) {
            // The key is ours, so this detach cannot fail.
            let _ = buffer.detach_foreign(&detach_key());
        }
    }
    for (memory, ptr, len) in refresh.repoint {
        let Some(buffer) = host.with_buffer(memory, array_buffer).flatten() else {
            continue;
        };
        let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
        // SAFETY: the store's bytes, which this function is called to follow
        // every time they can move.
        let repointed = unsafe { buffer.repoint_foreign(ptr, len) };
        debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
    }
}

pub(super) fn take_buffer_and_detach(host: &WasmHost, memory: usize) {
    if let Some(buffer) = host.take_buffer(memory).as_ref().and_then(array_buffer) {
        let _ = buffer.detach_foreign(&detach_key());
    }
}

/// A wasm value as the bootstrap receives it.
pub(super) fn to_js(host: &WasmHost, value: &Val) -> JsValue {
    match value {
        Val::I32(v) => JsValue::new(*v),
        Val::I64(v) => JsValue::from(JsBigInt::new(*v)),
        Val::F32(v) => JsValue::new(f64::from(v.to_float())),
        Val::F64(v) => JsValue::new(v.to_float()),
        Val::FuncRef(Nullable::Val(func)) => number(host.func_handle(*func, None)),
        Val::ExternRef(Nullable::Val(r)) => {
            let slot = *r
                .data(&*host.store.borrow())
                .downcast_ref::<usize>()
                .expect("every externref holds a slot");
            host.externs.borrow()[slot].clone()
        },
        Val::FuncRef(Nullable::Null) | Val::ExternRef(Nullable::Null) => JsValue::null(),
        Val::V128(_) => JsValue::undefined(),
    }
}

/// A script value the bootstrap has already converted for `ty`.
pub(super) fn to_val(host: &WasmHost, value: &JsValue, ty: ValType) -> Option<Val> {
    Some(match ty {
        ValType::I32 => Val::I32(value.as_number()? as i32),
        ValType::I64 => Val::I64(i64::try_from(value.as_bigint()?.to_i128()).ok()?),
        ValType::F32 => Val::F32(F32::from_float(value.as_number()? as f32)),
        ValType::F64 => Val::F64(F64::from_float(value.as_number()?)),
        ValType::FuncRef if value.is_null() => Val::FuncRef(Nullable::Null),
        ValType::FuncRef => Val::FuncRef(Nullable::Val(host.func(handle_arg(value)?)?)),
        ValType::ExternRef if value.is_null() => Val::ExternRef(Nullable::Null),
        ValType::ExternRef => {
            let slot = host.push(&host.externs, value.clone());
            Val::ExternRef(Nullable::Val(ExternRef::new(
                &mut *host.store.borrow_mut(),
                slot,
            )))
        },
        ValType::V128 => return None,
    })
}

/// Arguments `first..` converted for `types`, or `None` if one does not fit.
pub(super) fn args_for(
    host: &WasmHost,
    args: &[JsValue],
    first: usize,
    types: &[ValType],
) -> Option<Vec<Val>> {
    types
        .iter()
        .enumerate()
        .map(|(i, ty)| to_val(host, &arg(args, first + i), *ty))
        .collect()
}

fn call_script(host: &WasmHost, call: &ImportCall, ctx: &mut Context) -> JsResult<JsValue> {
    detach_stale(host, None);
    let wrapper = host.imports.borrow()[call.slot].clone();
    let Some(function) = wrapper.as_callable() else {
        return Err(type_error("import is not callable"));
    };
    let args: Vec<JsValue> = call.params.iter().map(|v| to_js(host, v)).collect();
    function.call(&JsValue::undefined(), &args, ctx)
}

/// What a script function returned, as the values its wasm type wants. A
/// multi-value result arrives staged by the wrapper (`stage`).
fn script_results(host: &WasmHost, value: &JsValue, types: &[ValType]) -> JsResult<Vec<Val>> {
    let wrong = || type_error("a script function returned the wrong type");
    match types {
        [] => Ok(Vec::new()),
        [ty] => to_val(host, value, *ty).map(|v| vec![v]).ok_or_else(wrong),
        types => {
            let staged: Vec<JsValue> = std::mem::take(&mut *host.staged.borrow_mut());
            if staged.len() != types.len() {
                return Err(wrong());
            }
            staged
                .iter()
                .zip(types)
                .map(|(value, ty)| to_val(host, value, *ty).ok_or_else(wrong))
                .collect()
        },
    }
}

/// Run `func` to completion, calling script for every import on the way, and
/// leave no stale buffer behind for the script that runs next.
pub(super) fn drive(
    host: &WasmHost,
    func: Func,
    inputs: &[Val],
    ctx: &mut Context,
) -> JsResult<Outcome> {
    let outcome = run(host, func, inputs, ctx);
    detach_stale(host, None);
    outcome
}

fn run(host: &WasmHost, func: Func, inputs: &[Val], ctx: &mut Context) -> JsResult<Outcome> {
    let ty = host.func_type(&func);
    let mut results: Vec<Val> = ty
        .results()
        .iter()
        .map(|t| Val::default_for_ty(*t))
        .collect();
    let mut step = func.call_resumable(&mut *host.store.borrow_mut(), inputs, &mut results);
    loop {
        match step {
            Ok(ResumableCall::Finished) => return Ok(Outcome::Done(results)),
            Ok(ResumableCall::OutOfFuel(_)) => return Ok(Outcome::Trap(FUEL_EXHAUSTED.into())),
            Ok(ResumableCall::HostTrap(pending)) => {
                let Some(call) = pending.host_error().downcast_ref::<ImportCall>() else {
                    return Ok(Outcome::Trap(pending.host_error().to_string()));
                };
                let types = host.func_type(&pending.host_func()).results().to_vec();
                // Dropping `pending` on an error unwinds the wasm frames.
                let value = call_script(host, call, ctx)?;
                let resumed = script_results(host, &value, &types)?;
                step = pending.resume(&mut *host.store.borrow_mut(), &resumed, &mut results);
            },
            Err(error) => {
                // A function made from script reached with no wasm frame to
                // suspend: its result is the call's.
                if let Some(call) = error.downcast_ref::<ImportCall>() {
                    let value = call_script(host, call, ctx)?;
                    return script_results(host, &value, ty.results()).map(Outcome::Done);
                }
                if error.as_trap_code() == Some(TrapCode::OutOfFuel) {
                    return Ok(Outcome::Trap(FUEL_EXHAUSTED.into()));
                }
                return Ok(Outcome::Trap(error.to_string()));
            },
        }
    }
}
