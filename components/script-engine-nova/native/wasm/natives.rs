// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for modules, instances and functions that the bootstrap closes
//! over.
//!
//! A failure the bootstrap turns into a WebAssembly error class comes back as
//! a string, because Vano lets an embedder throw only the built-in error kinds
//! it makes itself. `TypeError` and `RangeError` are thrown here directly. A
//! script exception raised inside wasm passes through unchanged.

use genet_wasm::wasmi::FuncType;
use nova_vm::ecmascript::{Agent, ArgumentsList, ArrayBuffer, JsResult, TypedArray, Value};
use nova_vm::engine::{Bindable, GcScope};

use super::bridge::{
    Outcome, args_for, drive, handle_arg, host, number, require_host, root, string, string_arg,
    to_js, type_error,
};
use genet_wasm::ImportValue;
use genet_wasm::json;

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
        Ok(handle) => number(agent, handle as f64, gc),
        Err(message) => string(agent, message, gc),
    })
}

/// `describe(module)`: the module's imports and exports with their types.
pub(super) fn describe<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    match handle_arg(agent, args.get(0)).and_then(|h| host.describe(h)) {
        Some(json) => Ok(string(agent, json, gc)),
        None => Err(type_error(agent, "not a module handle", gc)),
    }
}

/// `instantiate(module, plan, ...wrappers)`: an instance handle, or `link:`
/// or `trap:` and the message. `plan` has one comma-separated entry per
/// import: `s` takes the next wrapper, and `kind:handle` names an entity.
/// Runs a moved start function through the bridge.
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
    let module = handle_arg(agent, args.get(0));
    let plan = string_arg(agent, args.get(1));
    let Some((module, plan)) = module.zip(plan) else {
        return Err(type_error(
            agent,
            "expected a module and a plan",
            gc.into_nogc(),
        ));
    };
    let mut wrappers = 2..args.len();
    let mut values = Vec::new();
    for entry in plan.split(',').filter(|e| !e.is_empty()) {
        let value = match entry.split_once(':') {
            Some((kind, handle)) => match (kind.parse(), handle.parse()) {
                (Ok(kind), Ok(handle)) => Some(ImportValue::Entity(kind, handle)),
                _ => None,
            },
            None => wrappers
                .next()
                .map(|i| ImportValue::Script(root(agent, args.get(i), gc.nogc()))),
        };
        let Some(value) = value else {
            return Err(type_error(agent, "malformed import plan", gc.into_nogc()));
        };
        values.push(value);
    }
    let (instance, start) = match host.instantiate(module, values) {
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
    Ok(number(agent, instance as f64, gc.into_nogc()))
}

/// `exportOf(instance, name, kind)`: the export's handle, the same one each
/// time, or undefined.
pub(super) fn export_of<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let instance = handle_arg(agent, args.get(0));
    let name = string_arg(agent, args.get(1));
    let kind = handle_arg(agent, args.get(2)).and_then(|k| u8::try_from(k).ok());
    let found = instance
        .zip(name)
        .zip(kind)
        .and_then(|((i, n), k)| host.export_of(i, &n, k));
    Ok(match found {
        Some(handle) => number(agent, handle as f64, gc),
        None => Value::Undefined,
    })
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

/// `call(trapped, func, ...args)`: the result; for several, their count, read
/// with `resultAt`. On a trap it returns `trapped`, a sentinel the bootstrap
/// owns, since any value can be a result, and keeps the message for
/// `trapMessage`.
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
    let Some(func) = handle_arg(agent, args.get(1)).and_then(|h| host.func(h)) else {
        return Err(type_error(agent, "not a function handle", gc.into_nogc()));
    };
    let ty = host.func_type(&func);
    let Some(inputs) = args_for(agent, &host, &args, 2, ty.params(), gc.nogc()) else {
        return Err(type_error(
            agent,
            "an argument of the wrong type",
            gc.into_nogc(),
        ));
    };
    match drive(agent, &host, func, &inputs, gc.reborrow()) {
        Ok(Outcome::Done(values)) => {
            let gc = gc.into_nogc();
            match values.as_slice() {
                [] => Ok(Value::Undefined),
                [value] => Ok(to_js(agent, &host, value, gc)),
                many => {
                    let count = many.len();
                    *host.results.borrow_mut() = values;
                    Ok(number(agent, count as f64, gc))
                },
            }
        },
        Ok(Outcome::Trap(message)) => {
            *host.trap.borrow_mut() = message;
            Ok(args.get(0).bind(gc.into_nogc()))
        },
        Err(error) => Err(error.bind(gc.into_nogc())),
    }
}

/// `trapMessage()`: the message of the last trap `call` returned.
pub(super) fn trap_message<'gc>(
    agent: &mut Agent,
    _this: Value,
    _args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let message = std::mem::take(&mut *host.trap.borrow_mut());
    Ok(string(agent, message, gc))
}

/// `resultAt(i)`: one result of the last multi-value call.
pub(super) fn result_at<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let value = handle_arg(agent, args.get(0)).and_then(|i| host.results.borrow().get(i).cloned());
    Ok(match value {
        Some(value) => to_js(agent, &host, &value, gc),
        None => Value::Undefined,
    })
}

/// `stage(...values)`: a script function's multi-value result, converted by
/// its wrapper, for the bridge to take when the function returns.
pub(super) fn stage<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let staged = (0..args.len())
        .map(|i| root(agent, args.get(i), gc))
        .collect();
    *host.staged.borrow_mut() = staged;
    Ok(Value::Undefined)
}

fn type_list(list: &str) -> Option<Vec<genet_wasm::wasmi::ValType>> {
    list.split(',')
        .filter(|t| !t.is_empty())
        .map(json::val_type)
        .collect()
}

/// `funcNew(params, results, wrapper)`: a function handle for a wasm function
/// of that type that calls `wrapper`, as `new WebAssembly.Function` makes.
/// The types are comma-separated JS API names.
pub(super) fn func_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let params = string_arg(agent, args.get(0)).and_then(|l| type_list(&l));
    let results = string_arg(agent, args.get(1)).and_then(|l| type_list(&l));
    let Some((params, results)) = params.zip(results) else {
        return Err(type_error(agent, "expected value types", gc));
    };
    let wrapper = root(agent, args.get(2), gc);
    let func = host.script_func(
        &mut host.store.borrow_mut(),
        FuncType::new(params, results),
        wrapper,
    );
    let handle = host.func_handle(func, None);
    Ok(number(agent, handle as f64, gc))
}

/// `customSection(module, name, nth)`: a copy of the `nth` custom section
/// called `name`, or undefined past the last.
pub(super) fn custom_section<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let module = handle_arg(agent, args.get(0));
    let nth = handle_arg(agent, args.get(2));
    let Some((module, nth)) = module.zip(nth) else {
        return Err(type_error(
            agent,
            "expected a module, a name and an index",
            gc,
        ));
    };
    // A name with a lone surrogate matches no section: no UTF-8 name is it.
    let Ok(name) = nova_vm::ecmascript::String::try_from(args.get(1)) else {
        return Err(type_error(agent, "expected a section name", gc));
    };
    let Some(name) = name.as_str(agent).map(str::to_string) else {
        return Ok(Value::Undefined);
    };
    let Some(bytes) = host.custom_section(module, &name, nth) else {
        return Ok(Value::Undefined);
    };
    let buffer = ArrayBuffer::new(agent, bytes.len(), gc)?;
    buffer.as_mut_slice(agent).copy_from_slice(&bytes);
    Ok(buffer.into())
}
