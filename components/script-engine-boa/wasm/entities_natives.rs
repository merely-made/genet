// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives for `Global` and `Table`, on Bole.

use boa_engine::{Context, JsResult, JsValue};
use genet_wasm::entities::{table_type_of, val_ref};
use genet_wasm::json;
use genet_wasm::wasmi::{RefType, ValType};

use super::WasmHost;
use super::bridge::{
    arg, handle_arg, number, range_error, require_host, string, string_arg, to_js, to_val,
    type_error,
};

/// A count as a number, or as the decimal string the bootstrap sends for a
/// 64-bit limit.
pub(super) fn u64_arg(value: &JsValue) -> Option<u64> {
    match string_arg(value) {
        Some(text) => text.parse().ok(),
        None => handle_arg(value).map(|n| n as u64),
    }
}

/// An optional maximum: undefined, or a count.
pub(super) fn maximum_arg(value: &JsValue) -> Result<Option<u64>, ()> {
    if value.is_undefined() {
        return Ok(None);
    }
    u64_arg(value).map(Some).ok_or(())
}

/// `globalNew(type, mutable, value)`: a global handle.
pub(super) fn global_new(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let Some(ty) = string_arg(&arg(args, 0)).and_then(|t| json::val_type(&t)) else {
        return Err(type_error("not a value type"));
    };
    let mutable = arg(args, 1).as_boolean() == Some(true);
    let Some(value) = to_val(&host, &arg(args, 2), ty) else {
        return Err(type_error("a value of the wrong type"));
    };
    Ok(number(host.global_new(value, mutable)))
}

/// `globalGet(global)`: its value.
pub(super) fn global_get(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    match handle_arg(&arg(args, 0)).and_then(|h| host.global_get(h)) {
        Some(value) => Ok(to_js(&host, &value)),
        None => Err(type_error("not a global handle")),
    }
}

/// `globalSet(global, value)`; an immutable global throws `TypeError`.
pub(super) fn global_set(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let Some((handle, ty)) = handle.and_then(|h| Some((h, host.global_type(h)?))) else {
        return Err(type_error("not a global handle"));
    };
    let Some(value) = to_val(&host, &arg(args, 1), ty.content()) else {
        return Err(type_error("a value of the wrong type"));
    };
    host.global_set(handle, value).map_err(|m| type_error(&m))?;
    Ok(JsValue::undefined())
}

/// `globalType(global)`: its type as JSON.
pub(super) fn global_type(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let Some(ty) = handle_arg(&arg(args, 0)).and_then(|h| host.global_type(h)) else {
        return Err(type_error("not a global handle"));
    };
    let mut out = String::from("{");
    json::global(&mut out, &ty);
    out.push('}');
    Ok(string(out))
}

fn element_type(element: RefType) -> ValType {
    match element {
        RefType::Func => ValType::FuncRef,
        RefType::Extern => ValType::ExternRef,
    }
}

fn table_element(host: &WasmHost, handle: usize) -> Option<ValType> {
    Some(element_type(host.table_type(handle)?.element()))
}

/// `tableNew(element, initial, maximum?, init, is64)`: a table handle.
pub(super) fn table_new(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let element = string_arg(&arg(args, 0)).and_then(|t| json::ref_type(&t));
    let initial = u64_arg(&arg(args, 1));
    let maximum = maximum_arg(&arg(args, 2));
    let (Some(element), Some(initial), Ok(maximum)) = (element, initial, maximum) else {
        return Err(type_error("expected an element type and limits"));
    };
    let Some(init) = to_val(&host, &arg(args, 3), element_type(element)).and_then(val_ref) else {
        return Err(type_error("a value of the wrong type"));
    };
    let is64 = arg(args, 4).as_boolean() == Some(true);
    let ty = table_type_of(element, initial, maximum, is64)
        .ok_or_else(|| range_error("invalid table limits"))?;
    let handle = host.table_new(ty, init).map_err(|m| range_error(&m))?;
    Ok(number(handle))
}

/// `tableGet(table, index)`: the element.
pub(super) fn table_get(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let index = u64_arg(&arg(args, 1));
    match handle.zip(index).and_then(|(h, i)| host.table_get(h, i)) {
        Some(Some(element)) => Ok(to_js(&host, &element.into())),
        Some(None) => Err(range_error("table index out of bounds")),
        None => Err(type_error("not a table handle")),
    }
}

/// `tableSet(table, index, value)`.
pub(super) fn table_set(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let index = u64_arg(&arg(args, 1));
    let Some(((handle, index), ty)) = handle
        .zip(index)
        .and_then(|(h, i)| Some(((h, i), table_element(&host, h)?)))
    else {
        return Err(type_error("expected a table and an index"));
    };
    let Some(value) = to_val(&host, &arg(args, 2), ty).and_then(val_ref) else {
        return Err(type_error("a value of the wrong type"));
    };
    host.table_set(handle, index, value)
        .map_err(|m| range_error(&m))?;
    Ok(JsValue::undefined())
}

/// `tableGrow(table, delta, init)`: the old length.
pub(super) fn table_grow(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let delta = u64_arg(&arg(args, 1));
    let Some(((handle, delta), ty)) = handle
        .zip(delta)
        .and_then(|(h, d)| Some(((h, d), table_element(&host, h)?)))
    else {
        return Err(type_error("expected a table and a count"));
    };
    let Some(init) = to_val(&host, &arg(args, 2), ty).and_then(val_ref) else {
        return Err(type_error("a value of the wrong type"));
    };
    let old = host
        .table_grow(handle, delta, init)
        .map_err(|m| range_error(&m))?;
    Ok(JsValue::new(old as f64))
}

/// `tableLength(table)`.
pub(super) fn table_length(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    match handle_arg(&arg(args, 0)).and_then(|h| host.table_size(h)) {
        Some(size) => Ok(JsValue::new(size as f64)),
        None => Err(type_error("not a table handle")),
    }
}

/// `tableType(table)`: its type as JSON, `minimum` its current length.
pub(super) fn table_type(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let handle = handle_arg(&arg(args, 0));
    let Some((ty, size)) = handle.and_then(|h| host.table_type(h).zip(host.table_size(h))) else {
        return Err(type_error("not a table handle"));
    };
    let mut out = String::from("{");
    json::table(&mut out, &ty, size);
    out.push('}');
    Ok(string(out))
}
