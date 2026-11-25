// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The native functions the bootstrap closes over, and the call bridge.
//!
//! Each returns a number or a value on success and a string on a failure the
//! bootstrap turns into a WebAssembly error class, because Vano gives an
//! embedder no way to throw a value it did not make itself. A script
//! exception raised by an import passes through unchanged.
//!
//! The bridge calls wasm resumably. An import suspends the call and ends the
//! store borrow, so the script function it names can call back into wasm on
//! the same store before the outer call resumes.

use std::rc::Rc;

use nova_vm::ecmascript::{
    Agent, ArgumentsList, BigInt, ExceptionType, Function, JsError, JsResult, Number,
    String as JsString, TypedArray, Value,
};
use nova_vm::engine::{Bindable, GcScope, Global, NoGcScope};
use wasmi::{F32, F64, Func, ResumableCall, TrapCode, Val, ValType};

use super::super::{NovaHostSlot, NovaValue};
use super::WasmSlot;
use super::host::{ImportCall, WasmHost};

const FUEL_EXHAUSTED: &str = "fuel exhausted";

enum Outcome {
    Done(Vec<Val>),
    Trap(String),
}

fn host(agent: &Agent) -> Option<Rc<WasmHost>> {
    agent
        .get_host_data()
        .downcast_ref::<WasmSlot>()?
        .borrow()
        .clone()
}

fn type_error<'gc>(agent: &mut Agent, message: &str, gc: NoGcScope<'gc, '_>) -> JsError<'gc> {
    agent.throw_exception(ExceptionType::TypeError, message.to_string(), gc)
}

fn require_host<'gc>(agent: &mut Agent, gc: NoGcScope<'gc, '_>) -> JsResult<'gc, Rc<WasmHost>> {
    host(agent).ok_or_else(|| type_error(agent, "WebAssembly is not available", gc))
}

fn handle_arg(agent: &Agent, value: Value) -> Option<usize> {
    let n = Number::try_from(value).ok()?.into_f64(agent);
    (n >= 0.0 && n.fract() == 0.0 && n < usize::MAX as f64).then_some(n as usize)
}

fn string_arg(agent: &Agent, value: Value) -> Option<String> {
    Some(
        JsString::try_from(value)
            .ok()?
            .to_string_lossy(agent)
            .into_owned(),
    )
}

fn bytes_arg(agent: &Agent, value: Value) -> Option<Vec<u8>> {
    let Value::Uint8Array(array) = value else {
        return None;
    };
    let typed = TypedArray::from(array);
    let buffer = typed.get_viewed_array_buffer(agent);
    if buffer.is_detached(agent) {
        return None;
    }
    let offset = typed.byte_offset(agent);
    let bytes = buffer.as_slice(agent);
    let length = typed
        .array_length(agent)
        .unwrap_or_else(|| bytes.len().saturating_sub(offset));
    bytes
        .get(offset..offset.checked_add(length)?)
        .map(<[u8]>::to_vec)
}

fn number<'gc>(agent: &mut Agent, n: usize, gc: NoGcScope<'gc, '_>) -> Value<'gc> {
    Value::from_f64(agent, n as f64, gc)
}

fn string<'gc>(agent: &mut Agent, s: String, gc: NoGcScope<'gc, '_>) -> Value<'gc> {
    JsString::from_string(agent, s, gc).into()
}

/// A wasm value as script sees it: numbers, and `i64` as a BigInt.
fn to_js<'gc>(agent: &mut Agent, value: &Val, gc: NoGcScope<'gc, '_>) -> Value<'gc> {
    match value {
        Val::I32(v) => Value::from_f64(agent, f64::from(*v), gc),
        Val::I64(v) => BigInt::from_i64(agent, *v).into(),
        Val::F32(v) => Value::from_f64(agent, f64::from(v.to_float()), gc),
        Val::F64(v) => Value::from_f64(agent, v.to_float(), gc),
        _ => Value::Undefined,
    }
}

/// A script value the bootstrap has already converted for `ty`.
fn to_val(agent: &Agent, value: Value, ty: ValType) -> Option<Val> {
    let float = |value| Some(Number::try_from(value).ok()?.into_f64(agent));
    Some(match ty {
        ValType::I32 => Val::I32(float(value)? as i32),
        ValType::I64 => Val::I64(BigInt::try_from(value).ok()?.try_into_i64(agent).ok()?),
        ValType::F32 => Val::F32(F32::from_float(float(value)? as f32)),
        ValType::F64 => Val::F64(F64::from_float(float(value)?)),
        _ => return None,
    })
}

fn call_import(
    agent: &mut Agent,
    host: &WasmHost,
    call: &ImportCall,
    gc: GcScope<'_, '_>,
) -> Result<Value<'static>, JsError<'static>> {
    let wrapper = host.imports.borrow()[call.slot].get(agent, gc.nogc());
    let Ok(function) = Function::try_from(wrapper) else {
        return Err(type_error(agent, "import is not callable", gc.nogc()).unbind());
    };
    let mut args: Vec<Value> = call
        .params
        .iter()
        .map(|v| to_js(agent, v, gc.nogc()).unbind())
        .collect();
    function
        .call(agent, Value::Undefined, &mut args, gc)
        .map(|v| v.unbind())
        .map_err(|e| e.unbind())
}

fn import_results(
    agent: &mut Agent,
    value: Value<'static>,
    types: &[ValType],
    gc: NoGcScope<'_, '_>,
) -> Result<Vec<Val>, JsError<'static>> {
    match types {
        [] => Ok(Vec::new()),
        [ty] => to_val(agent, value, *ty).map(|v| vec![v]).ok_or_else(|| {
            type_error(agent, "import returned a value of the wrong type", gc).unbind()
        }),
        _ => Err(type_error(agent, "multi-value imports are not supported yet", gc).unbind()),
    }
}

/// Run `func` to completion, calling script for every import on the way.
fn drive(
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
                let value = call_import(agent, host, call, gc.reborrow())?;
                let resumed = import_results(agent, value, &types, gc.nogc())?;
                step = pending.resume(&mut *host.store.borrow_mut(), &resumed, &mut results);
            },
            Err(error) => {
                // An import reached with no wasm frame to suspend, as when the
                // export is the import itself or tail-calls it: its result is
                // the call's.
                if let Some(call) = error.downcast_ref::<ImportCall>() {
                    let value = call_import(agent, host, call, gc.reborrow())?;
                    return import_results(agent, value, ty.results(), gc.nogc())
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

/// `compile(bytes: Uint8Array)`: a module handle, or the compile error.
pub(super) fn compile<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some(bytes) = bytes_arg(agent, args.get(0)) else {
        return Err(type_error(agent, "expected a Uint8Array", gc));
    };
    Ok(match host.compile(bytes) {
        Ok(handle) => number(agent, handle, gc),
        Err(message) => string(agent, message, gc),
    })
}

/// `describe(module)`: the module's imports and exports as JSON.
pub(super) fn describe<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let json = handle_arg(agent, args.get(0)).and_then(|h| host.describe(h));
    match json {
        Some(json) => Ok(string(agent, json, gc)),
        None => Err(type_error(agent, "not a module handle", gc)),
    }
}

/// `instantiate(module, ...wrappers)`: an instance handle, or `link:`/`trap:`
/// and the message. Runs a moved start function through the bridge.
pub(super) fn instantiate<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let Some(host) = host(agent) else {
        return Err(type_error(
            agent,
            "WebAssembly is not available",
            gc.into_nogc(),
        ));
    };
    let Some(module) = handle_arg(agent, args.get(0)) else {
        return Err(type_error(agent, "not a module handle", gc.into_nogc()));
    };
    let release = agent
        .current_realm(gc.nogc())
        .host_defined(agent)
        .and_then(|hd| {
            hd.downcast_ref::<NovaHostSlot>()
                .map(|slot| slot.release.clone())
        })
        .expect("host slot present");
    let wrappers = (1..args.len())
        .map(|i| NovaValue::new(Global::new(agent, args.get(i).unbind()), &release))
        .collect();
    let (instance, start) = match host.instantiate(module, wrappers) {
        Ok(linked) => linked,
        Err(message) => return Ok(string(agent, message, gc.into_nogc())),
    };
    if let Some(start) = start {
        match drive(agent, &host, start, &[], gc.reborrow()) {
            Ok(Outcome::Done(_)) => {},
            Ok(Outcome::Trap(message)) => {
                return Ok(string(agent, format!("trap:{message}"), gc.into_nogc()));
            },
            Err(error) => return Err(error.bind(gc.into_nogc())),
        }
    }
    Ok(number(agent, instance, gc.into_nogc()))
}

/// `exportFunc(instance, name)`: a function handle, or undefined.
pub(super) fn export_func<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let instance = handle_arg(agent, args.get(0));
    let name = string_arg(agent, args.get(1));
    match instance
        .zip(name)
        .and_then(|(i, n)| host.export_func(i, &n))
    {
        Some(handle) => Ok(number(agent, handle, gc)),
        None => Ok(Value::Undefined),
    }
}

/// `signature(func)`: its index and types as JSON.
pub(super) fn signature<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    match handle_arg(agent, args.get(0)).and_then(|h| host.signature(h)) {
        Some(json) => Ok(string(agent, json, gc)),
        None => Err(type_error(agent, "not a function handle", gc)),
    }
}

/// `call(func, ...args)`: the result, or the trap's message as a string.
/// Arguments arrive converted by the bootstrap.
pub(super) fn call<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let Some(host) = host(agent) else {
        return Err(type_error(
            agent,
            "WebAssembly is not available",
            gc.into_nogc(),
        ));
    };
    let Some(func) = handle_arg(agent, args.get(0)).and_then(|h| host.func(h)) else {
        return Err(type_error(agent, "not a function handle", gc.into_nogc()));
    };
    let ty = host.func_type(&func);
    let inputs: Option<Vec<Val>> = ty
        .params()
        .iter()
        .enumerate()
        .map(|(i, t)| to_val(agent, args.get(i + 1), *t))
        .collect();
    let Some(inputs) = inputs else {
        return Err(type_error(
            agent,
            "argument of an unsupported type",
            gc.into_nogc(),
        ));
    };
    match drive(agent, &host, func, &inputs, gc.reborrow()) {
        Ok(Outcome::Done(values)) => {
            let gc = gc.into_nogc();
            match values.as_slice() {
                [] => Ok(Value::Undefined),
                [value] => Ok(to_js(agent, value, gc)),
                _ => Err(type_error(
                    agent,
                    "multi-value results are not supported yet",
                    gc,
                )),
            }
        },
        Ok(Outcome::Trap(message)) => Ok(string(agent, message, gc.into_nogc())),
        Err(error) => Err(error.bind(gc.into_nogc())),
    }
}
