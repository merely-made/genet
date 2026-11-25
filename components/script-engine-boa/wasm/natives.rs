// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for modules, instances and functions, on Bole. They keep the
//! protocol `genet-wasm`'s bootstrap expects: a failure it turns into a
//! WebAssembly error class comes back as a string, and a trap as a sentinel.

use boa_engine::object::builtins::{JsArrayBuffer, JsUint8Array};
use boa_engine::{Context, JsResult, JsValue};
use genet_wasm::wasmi::{FuncType, ValType};
use genet_wasm::{ImportValue, json};

use super::bridge::{
    Outcome, arg, args_for, drive, handle_arg, number, require_host, string, string_arg, to_js,
    type_error,
};

/// `compile(bytes: Uint8Array)`: a module handle, or the compile error.
pub(super) fn compile(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let Some(object) = arg(args, 0).as_object() else {
        return Err(type_error("expected a Uint8Array"));
    };
    let bytes = JsUint8Array::from_object(object.clone())?.to_vec(ctx)?;
    Ok(match host.compile(bytes) {
        Ok(handle) => number(handle),
        Err(message) => string(message),
    })
}

/// `describe(module)`: the module's imports and exports with their types.
pub(super) fn describe(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    match handle_arg(&arg(args, 0)).and_then(|h| host.describe(h)) {
        Some(json) => Ok(string(json)),
        None => Err(type_error("not a module handle")),
    }
}

/// `instantiate(module, plan, ...wrappers)`; see `genet-vano`'s twin.
pub(super) fn instantiate(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let module = handle_arg(&arg(args, 0));
    let plan = string_arg(&arg(args, 1));
    let Some((module, plan)) = module.zip(plan) else {
        return Err(type_error("expected a module and a plan"));
    };
    let mut wrappers = 2..args.len();
    let mut values = Vec::new();
    for entry in plan.split(',').filter(|e| !e.is_empty()) {
        let value = match entry.split_once(':') {
            Some((kind, handle)) => match (kind.parse(), handle.parse()) {
                (Ok(kind), Ok(handle)) => Some(ImportValue::Entity(kind, handle)),
                _ => None,
            },
            None => wrappers.next().map(|i| ImportValue::Script(arg(args, i))),
        };
        values.push(value.ok_or_else(|| type_error("malformed import plan"))?);
    }
    let (instance, start) = match host.instantiate(module, values) {
        Ok(linked) => linked,
        Err(message) => return Ok(string(message)),
    };
    if let Some(start) = start {
        if let Outcome::Trap(message) = drive(&host, start, &[], ctx)? {
            return Ok(string(format!("trap:{message}")));
        }
    }
    Ok(number(instance))
}

/// `exportOf(instance, name, kind)`: the export's handle, or undefined.
pub(super) fn export_of(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let instance = handle_arg(&arg(args, 0));
    let name = string_arg(&arg(args, 1));
    let kind = handle_arg(&arg(args, 2)).and_then(|k| u8::try_from(k).ok());
    let found = instance
        .zip(name)
        .zip(kind)
        .and_then(|((i, n), k)| host.export_of(i, &n, k));
    Ok(found.map_or_else(JsValue::undefined, number))
}

/// `signature(func)`: its index and types as JSON.
pub(super) fn signature(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    match handle_arg(&arg(args, 0)).and_then(|h| host.signature(h)) {
        Some(json) => Ok(string(json)),
        None => Err(type_error("not a function handle")),
    }
}

/// `call(trapped, func, ...args)`: the result; for several, their count,
/// read with `resultAt`; on a trap, `trapped`, with the message kept for
/// `trapMessage`.
pub(super) fn call(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let Some(func) = handle_arg(&arg(args, 1)).and_then(|h| host.func(h)) else {
        return Err(type_error("not a function handle"));
    };
    let ty = host.func_type(&func);
    let Some(inputs) = args_for(&host, args, 2, ty.params()) else {
        return Err(type_error("an argument of the wrong type"));
    };
    match drive(&host, func, &inputs, ctx)? {
        Outcome::Done(values) => Ok(match values.as_slice() {
            [] => JsValue::undefined(),
            [value] => to_js(&host, value),
            many => {
                let count = many.len();
                *host.results.borrow_mut() = values;
                number(count)
            },
        }),
        Outcome::Trap(message) => {
            *host.trap.borrow_mut() = message;
            Ok(arg(args, 0))
        },
    }
}

/// `trapMessage()`: the message of the last trap `call` returned.
pub(super) fn trap_message(_: &JsValue, _: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let message = std::mem::take(&mut *host.trap.borrow_mut());
    Ok(string(message))
}

/// `resultAt(i)`: one result of the last multi-value call.
pub(super) fn result_at(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let value = handle_arg(&arg(args, 0)).and_then(|i| host.results.borrow().get(i).cloned());
    Ok(value.map_or_else(JsValue::undefined, |v| to_js(&host, &v)))
}

/// `stage(...values)`: a script function's multi-value result.
pub(super) fn stage(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    *host.staged.borrow_mut() = args.to_vec();
    Ok(JsValue::undefined())
}

fn type_list(list: &str) -> Option<Vec<ValType>> {
    list.split(',')
        .filter(|t| !t.is_empty())
        .map(json::val_type)
        .collect()
}

/// `funcNew(params, results, wrapper)`: `new WebAssembly.Function`.
pub(super) fn func_new(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let params = string_arg(&arg(args, 0)).and_then(|l| type_list(&l));
    let results = string_arg(&arg(args, 1)).and_then(|l| type_list(&l));
    let Some((params, results)) = params.zip(results) else {
        return Err(type_error("expected value types"));
    };
    let func = host.script_func(
        &mut host.store.borrow_mut(),
        FuncType::new(params, results),
        arg(args, 2),
    );
    Ok(number(host.func_handle(func, None)))
}

/// `customSection(module, name, nth)`: a copy of that custom section.
pub(super) fn custom_section(
    _: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let module = handle_arg(&arg(args, 0));
    let nth = handle_arg(&arg(args, 2));
    let Some((module, nth)) = module.zip(nth) else {
        return Err(type_error("expected a module, a name and an index"));
    };
    // A name with a lone surrogate matches no section.
    let Some(name) = string_arg(&arg(args, 1)) else {
        return Ok(JsValue::undefined());
    };
    let Some(bytes) = host.custom_section(module, &name, nth) else {
        return Ok(JsValue::undefined());
    };
    let block = boa_engine::object::builtins::AlignedVec::from_slice(64, &bytes);
    Ok(JsArrayBuffer::from_byte_block(block, ctx)?.into())
}
