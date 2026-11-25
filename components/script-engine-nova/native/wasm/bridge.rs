// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Values across the boundary, and the call bridge.
//!
//! The bridge calls wasm resumably. A function made from script suspends the
//! call and ends the store borrow, so the script it runs can call back into
//! wasm on the same store before the outer call resumes. Every time control
//! passes from wasm back to script, buffers over memory that moved are
//! detached first (`memory.rs`).
//!
//! Values cross already converted by the bootstrap. A `funcref` crosses as its
//! function handle and the bootstrap gives it its script function; an
//! `externref` crosses as the script value it holds.

use std::ptr::NonNull;
use std::rc::Rc;

use nova_vm::ecmascript::{
    Agent, ArgumentsList, ArrayBuffer, BigInt, DetachKey, ExceptionType, Function, JsError,
    JsResult, Number, String as JsString, Value,
};
use nova_vm::engine::{Bindable, GcScope, Global, NoGcScope};
use wasmi::{ExternRef, F32, F64, Func, Nullable, ResumableCall, TrapCode, Val, ValType};

use super::super::{NovaHostSlot, NovaValue, ReleaseQueue};
use super::WasmSlot;
use super::host::{ImportCall, WasmHost};

const FUEL_EXHAUSTED: &str = "fuel exhausted";

pub(super) enum Outcome {
    Done(Vec<Val>),
    Trap(String),
}

pub(super) fn host(agent: &Agent) -> Option<Rc<WasmHost>> {
    agent
        .get_host_data()
        .downcast_ref::<WasmSlot>()?
        .borrow()
        .clone()
}

pub(super) fn error<'gc>(
    agent: &mut Agent,
    kind: ExceptionType,
    message: &str,
    gc: NoGcScope<'gc, '_>,
) -> JsError<'gc> {
    agent.throw_exception(kind, message.to_string(), gc)
}

pub(super) fn type_error<'gc>(
    agent: &mut Agent,
    message: &str,
    gc: NoGcScope<'gc, '_>,
) -> JsError<'gc> {
    error(agent, ExceptionType::TypeError, message, gc)
}

pub(super) fn require_host<'gc>(
    agent: &mut Agent,
    gc: NoGcScope<'gc, '_>,
) -> JsResult<'gc, Rc<WasmHost>> {
    host(agent).ok_or_else(|| type_error(agent, "WebAssembly is not available", gc))
}

pub(super) fn release_queue(agent: &Agent, gc: NoGcScope<'_, '_>) -> ReleaseQueue {
    agent
        .current_realm(gc)
        .host_defined(agent)
        .and_then(|hd| {
            hd.downcast_ref::<NovaHostSlot>()
                .map(|slot| slot.release.clone())
        })
        .expect("host slot present")
}

pub(super) fn root(agent: &mut Agent, value: Value, gc: NoGcScope<'_, '_>) -> NovaValue {
    let release = release_queue(agent, gc);
    NovaValue::new(Global::new(agent, value.unbind()), &release)
}

/// Bring every buffer over memory up to date before script runs: detach the
/// fixed-length ones whose memory moved, and `force`'s in any case, and
/// repoint the resizable ones at their memory's bytes.
pub(super) fn detach_stale(
    agent: &mut Agent,
    host: &WasmHost,
    force: Option<usize>,
    gc: NoGcScope<'_, '_>,
) {
    let refresh = host.refresh(force);
    for value in refresh.detach {
        if let Ok(buffer) = ArrayBuffer::try_from(value.get(agent, gc)) {
            // The key is ours, so this detach cannot fail.
            let _ = buffer.detach(agent, Some(DetachKey {}), gc);
        }
    }
    for (memory, ptr, len) in refresh.repoint {
        let Some(value) = host.with_buffer(memory, |v| v.get(agent, gc)) else {
            continue;
        };
        if let Ok(buffer) = ArrayBuffer::try_from(value) {
            let ptr = NonNull::new(ptr).unwrap_or(NonNull::dangling());
            // SAFETY: the store's bytes, which this function is called to
            // follow every time they can move.
            let repointed = unsafe { buffer.repoint_foreign(agent, ptr, len) };
            debug_assert!(repointed, "a memory's resizable buffer is over its bytes");
        }
    }
}

pub(super) fn handle_arg(agent: &Agent, value: Value) -> Option<usize> {
    let n = Number::try_from(value).ok()?.into_f64(agent);
    (n >= 0.0 && n.fract() == 0.0 && n < usize::MAX as f64).then_some(n as usize)
}

pub(super) fn string_arg(agent: &Agent, value: Value) -> Option<String> {
    Some(
        JsString::try_from(value)
            .ok()?
            .to_string_lossy(agent)
            .into_owned(),
    )
}

pub(super) fn number<'gc>(agent: &mut Agent, n: f64, gc: NoGcScope<'gc, '_>) -> Value<'gc> {
    Value::from_f64(agent, n, gc)
}

pub(super) fn string<'gc>(agent: &mut Agent, s: String, gc: NoGcScope<'gc, '_>) -> Value<'gc> {
    JsString::from_string(agent, s, gc).into()
}

/// A wasm value as the bootstrap receives it.
pub(super) fn to_js<'gc>(
    agent: &mut Agent,
    host: &WasmHost,
    value: &Val,
    gc: NoGcScope<'gc, '_>,
) -> Value<'gc> {
    match value {
        Val::I32(v) => Value::from_f64(agent, f64::from(*v), gc),
        Val::I64(v) => BigInt::from_i64(agent, *v).into(),
        Val::F32(v) => Value::from_f64(agent, f64::from(v.to_float()), gc),
        Val::F64(v) => Value::from_f64(agent, v.to_float(), gc),
        Val::FuncRef(Nullable::Val(func)) => {
            number(agent, host.func_handle(*func, None) as f64, gc)
        },
        Val::ExternRef(Nullable::Val(r)) => {
            let slot = *r
                .data(&*host.store.borrow())
                .downcast_ref::<usize>()
                .expect("every externref holds a slot");
            host.externs.borrow()[slot].get(agent, gc)
        },
        Val::FuncRef(Nullable::Null) | Val::ExternRef(Nullable::Null) => Value::Null,
        Val::V128(_) => Value::Undefined,
    }
}

/// A script value the bootstrap has already converted for `ty`.
pub(super) fn to_val(
    agent: &mut Agent,
    host: &WasmHost,
    value: Value,
    ty: ValType,
    gc: NoGcScope<'_, '_>,
) -> Option<Val> {
    let float = |agent: &Agent, value| Some(Number::try_from(value).ok()?.into_f64(agent));
    Some(match ty {
        ValType::I32 => Val::I32(float(agent, value)? as i32),
        ValType::I64 => Val::I64(BigInt::try_from(value).ok()?.try_into_i64(agent).ok()?),
        ValType::F32 => Val::F32(F32::from_float(float(agent, value)? as f32)),
        ValType::F64 => Val::F64(F64::from_float(float(agent, value)?)),
        ValType::FuncRef if value == Value::Null => Val::FuncRef(Nullable::Null),
        ValType::FuncRef => Val::FuncRef(Nullable::Val(host.func(handle_arg(agent, value)?)?)),
        ValType::ExternRef if value == Value::Null => Val::ExternRef(Nullable::Null),
        ValType::ExternRef => {
            let rooted = root(agent, value, gc);
            let slot = host.push(&host.externs, rooted);
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
    agent: &mut Agent,
    host: &WasmHost,
    args: &ArgumentsList,
    first: usize,
    types: &[ValType],
    gc: NoGcScope<'_, '_>,
) -> Option<Vec<Val>> {
    let mut out = Vec::with_capacity(types.len());
    for (i, ty) in types.iter().enumerate() {
        out.push(to_val(agent, host, args.get(first + i), *ty, gc)?);
    }
    Some(out)
}

fn call_script(
    agent: &mut Agent,
    host: &WasmHost,
    call: &ImportCall,
    gc: GcScope<'_, '_>,
) -> Result<Value<'static>, JsError<'static>> {
    detach_stale(agent, host, None, gc.nogc());
    let wrapper = host.imports.borrow()[call.slot].get(agent, gc.nogc());
    let Ok(function) = Function::try_from(wrapper) else {
        return Err(type_error(agent, "import is not callable", gc.nogc()).unbind());
    };
    let mut args: Vec<Value> = call
        .params
        .iter()
        .map(|v| to_js(agent, host, v, gc.nogc()).unbind())
        .collect();
    function
        .call(agent, Value::Undefined, &mut args, gc)
        .map(|v| v.unbind())
        .map_err(|e| e.unbind())
}

/// What a script function returned, as the values its wasm type wants. A
/// multi-value result arrives staged by the wrapper (`stage`).
fn script_results(
    agent: &mut Agent,
    host: &WasmHost,
    value: Value<'static>,
    types: &[ValType],
    gc: NoGcScope<'_, '_>,
) -> Result<Vec<Val>, JsError<'static>> {
    let wrong = |agent: &mut Agent| {
        type_error(agent, "a script function returned the wrong type", gc).unbind()
    };
    match types {
        [] => Ok(Vec::new()),
        [ty] => to_val(agent, host, value, *ty, gc)
            .map(|v| vec![v])
            .ok_or_else(|| wrong(agent)),
        types => {
            let staged: Vec<NovaValue> = std::mem::take(&mut *host.staged.borrow_mut());
            if staged.len() != types.len() {
                return Err(wrong(agent));
            }
            let mut out = Vec::new();
            for (value, ty) in staged.iter().zip(types) {
                let value = value.get(agent, gc);
                out.push(to_val(agent, host, value, *ty, gc).ok_or_else(|| wrong(agent))?);
            }
            Ok(out)
        },
    }
}

/// Run `func` to completion, calling script for every import on the way, and
/// leave no stale buffer behind for the script that runs next.
pub(super) fn drive(
    agent: &mut Agent,
    host: &WasmHost,
    func: Func,
    inputs: &[Val],
    mut gc: GcScope<'_, '_>,
) -> Result<Outcome, JsError<'static>> {
    let outcome = run(agent, host, func, inputs, gc.reborrow());
    detach_stale(agent, host, None, gc.nogc());
    outcome
}

fn run(
    agent: &mut Agent,
    host: &WasmHost,
    func: Func,
    inputs: &[Val],
    mut gc: GcScope<'_, '_>,
) -> Result<Outcome, JsError<'static>> {
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
                let value = call_script(agent, host, call, gc.reborrow())?;
                let resumed = script_results(agent, host, value, &types, gc.nogc())?;
                step = pending.resume(&mut *host.store.borrow_mut(), &resumed, &mut results);
            },
            Err(error) => {
                // A function made from script reached with no wasm frame to
                // suspend, as when it is called directly or tail-called from
                // the root frame: its result is the call's.
                if let Some(call) = error.downcast_ref::<ImportCall>() {
                    let value = call_script(agent, host, call, gc.reborrow())?;
                    return script_results(agent, host, value, ty.results(), gc.nogc())
                        .map(Outcome::Done);
                }
                if error.as_trap_code() == Some(TrapCode::OutOfFuel) {
                    return Ok(Outcome::Trap(FUEL_EXHAUSTED.into()));
                }
                return Ok(Outcome::Trap(error.to_string()));
            },
        }
    }
}
