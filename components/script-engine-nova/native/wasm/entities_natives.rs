// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Global` and `Table`. Values arrive converted by
//! the bootstrap for the entity's type; a bad handle or a value of the wrong
//! type throws `TypeError`, and a bound exceeded throws `RangeError`.

use genet_wasm::wasmi::ValType;
use nova_vm::ecmascript::{Agent, ArgumentsList, ExceptionType, JsResult, Value};
use nova_vm::engine::GcScope;

use super::bridge::{
    error, handle_arg, number, require_host, string, string_arg, to_js, to_val, type_error,
};
use genet_wasm::entities::{table_type_of, val_ref};
use genet_wasm::json;

/// A count as a number, or as the decimal string the bootstrap sends for a
/// 64-bit limit.
pub(super) fn u64_arg(agent: &Agent, value: Value) -> Option<u64> {
    match string_arg(agent, value) {
        Some(text) => text.parse().ok(),
        None => handle_arg(agent, value).map(|n| n as u64),
    }
}

/// An optional maximum: undefined, or a page or element count.
pub(super) fn maximum_arg(agent: &Agent, value: Value) -> Result<Option<u64>, ()> {
    match value {
        Value::Undefined => Ok(None),
        value => u64_arg(agent, value).map(Some).ok_or(()),
    }
}

/// `globalNew(type, mutable, value)`: a global handle.
pub(super) fn global_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some(ty) = string_arg(agent, args.get(0)).and_then(|t| json::val_type(&t)) else {
        return Err(type_error(agent, "not a value type", gc));
    };
    let mutable = args.get(1) == Value::Boolean(true);
    let Some(value) = to_val(agent, &host, args.get(2), ty, gc) else {
        return Err(type_error(agent, "a value of the wrong type", gc));
    };
    Ok(number(agent, host.global_new(value, mutable) as f64, gc))
}

/// `globalGet(global)`: its value.
pub(super) fn global_get<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    match handle_arg(agent, args.get(0)).and_then(|h| host.global_get(h)) {
        Some(value) => Ok(to_js(agent, &host, &value, gc)),
        None => Err(type_error(agent, "not a global handle", gc)),
    }
}

/// `globalSet(global, value)`; an immutable global throws `TypeError`.
pub(super) fn global_set<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let Some((handle, ty)) = handle.and_then(|h| Some((h, host.global_type(h)?))) else {
        return Err(type_error(agent, "not a global handle", gc));
    };
    let Some(value) = to_val(agent, &host, args.get(1), ty.content(), gc) else {
        return Err(type_error(agent, "a value of the wrong type", gc));
    };
    match host.global_set(handle, value) {
        Ok(()) => Ok(Value::Undefined),
        Err(message) => Err(type_error(agent, &message, gc)),
    }
}

/// `globalType(global)`: its type as JSON.
pub(super) fn global_type<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some(ty) = handle_arg(agent, args.get(0)).and_then(|h| host.global_type(h)) else {
        return Err(type_error(agent, "not a global handle", gc));
    };
    let mut out = String::from("{");
    json::global(&mut out, &ty);
    out.push('}');
    Ok(string(agent, out, gc))
}

fn table_element(host: &super::WasmHost, handle: usize) -> Option<ValType> {
    Some(match host.table_type(handle)?.element() {
        genet_wasm::wasmi::RefType::Func => ValType::FuncRef,
        genet_wasm::wasmi::RefType::Extern => ValType::ExternRef,
    })
}

/// `tableNew(element, initial, maximum?, init, is64)`: a table handle; a
/// maximum below the initial size throws `RangeError`.
pub(super) fn table_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let element = string_arg(agent, args.get(0)).and_then(|t| json::ref_type(&t));
    let initial = u64_arg(agent, args.get(1));
    let maximum = maximum_arg(agent, args.get(2));
    let (Some(element), Some(initial), Ok(maximum)) = (element, initial, maximum) else {
        return Err(type_error(agent, "expected an element type and limits", gc));
    };
    let is64 = args.get(4) == Value::Boolean(true);
    let Some(ty) = table_type_of(element, initial, maximum, is64) else {
        return Err(error(
            agent,
            ExceptionType::RangeError,
            "invalid table limits",
            gc,
        ));
    };
    let value_ty = match element {
        genet_wasm::wasmi::RefType::Func => ValType::FuncRef,
        genet_wasm::wasmi::RefType::Extern => ValType::ExternRef,
    };
    let Some(init) = to_val(agent, &host, args.get(3), value_ty, gc).and_then(val_ref) else {
        return Err(type_error(agent, "a value of the wrong type", gc));
    };
    match host.table_new(ty, init) {
        Ok(handle) => Ok(number(agent, handle as f64, gc)),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `tableGet(table, index)`: the element.
pub(super) fn table_get<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let index = u64_arg(agent, args.get(1));
    match handle.zip(index).and_then(|(h, i)| host.table_get(h, i)) {
        Some(Some(element)) => Ok(to_js(agent, &host, &element.into(), gc)),
        Some(None) => Err(error(
            agent,
            ExceptionType::RangeError,
            "table index out of bounds",
            gc,
        )),
        None => Err(type_error(agent, "not a table handle", gc)),
    }
}

/// `tableSet(table, index, value)`.
pub(super) fn table_set<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let index = u64_arg(agent, args.get(1));
    let Some(((handle, index), ty)) = handle
        .zip(index)
        .and_then(|(h, i)| Some(((h, i), table_element(&host, h)?)))
    else {
        return Err(type_error(agent, "expected a table and an index", gc));
    };
    let Some(value) = to_val(agent, &host, args.get(2), ty, gc).and_then(val_ref) else {
        return Err(type_error(agent, "a value of the wrong type", gc));
    };
    match host.table_set(handle, index, value) {
        Ok(()) => Ok(Value::Undefined),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `tableGrow(table, delta, init)`: the old length.
pub(super) fn table_grow<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let delta = u64_arg(agent, args.get(1));
    let Some(((handle, delta), ty)) = handle
        .zip(delta)
        .and_then(|(h, d)| Some(((h, d), table_element(&host, h)?)))
    else {
        return Err(type_error(agent, "expected a table and a count", gc));
    };
    let Some(init) = to_val(agent, &host, args.get(2), ty, gc).and_then(val_ref) else {
        return Err(type_error(agent, "a value of the wrong type", gc));
    };
    match host.table_grow(handle, delta, init) {
        Ok(old) => Ok(number(agent, old as f64, gc)),
        Err(message) => Err(error(agent, ExceptionType::RangeError, &message, gc)),
    }
}

/// `tableLength(table)`.
pub(super) fn table_length<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    match handle_arg(agent, args.get(0)).and_then(|h| host.table_size(h)) {
        Some(size) => Ok(number(agent, size as f64, gc)),
        None => Err(type_error(agent, "not a table handle", gc)),
    }
}

/// `tableType(table)`: its type as JSON.
pub(super) fn table_type<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let handle = handle_arg(agent, args.get(0));
    let Some((ty, size)) = handle.and_then(|h| host.table_type(h).zip(host.table_size(h))) else {
        return Err(type_error(agent, "not a table handle", gc));
    };
    let mut out = String::from("{");
    json::table(&mut out, &ty, size);
    out.push('}');
    Ok(string(agent, out, gc))
}
