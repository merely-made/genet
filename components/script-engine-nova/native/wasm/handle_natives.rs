// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives that own and free handles (`genet-wasm`'s `handles.rs`).

use nova_vm::ecmascript::{Agent, ArgumentsList, JsResult, Value};
use nova_vm::engine::GcScope;

use super::bridge::{handle_arg, number, require_host, string, type_error};

fn kind_and_handle(agent: &Agent, args: &ArgumentsList) -> Option<(u8, usize)> {
    let kind = handle_arg(agent, args.get(0)).and_then(|k| u8::try_from(k).ok())?;
    Some((kind, handle_arg(agent, args.get(1))?))
}

/// `retain(kind, handle)`: add an owner to a handle.
pub(super) fn retain<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some((kind, handle)) = kind_and_handle(agent, &args) else {
        return Err(type_error(agent, "expected a kind and a handle", gc));
    };
    host.retain(kind, handle);
    Ok(Value::Undefined)
}

/// `release(kind, handle, current)`: drop an owner. Returns the buffers to
/// detach, as `stale` does, if that rebuilt the store.
pub(super) fn release<'gc>(
    agent: &mut Agent,
    _this: Value,
    args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    let Some((kind, handle)) = kind_and_handle(agent, &args) else {
        return Err(type_error(agent, "expected a kind and a handle", gc));
    };
    let left = host.release(kind, handle, args.get(2) == Value::Boolean(true));
    let list: Vec<String> = left.iter().map(|memory| format!("d{memory}")).collect();
    Ok(string(agent, list.join(","), gc))
}

/// `extern()`: a fresh slot for a value an `externref` holds.
pub(super) fn extern_new<'gc>(
    agent: &mut Agent,
    _this: Value,
    _args: ArgumentsList,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let gc = gc.into_nogc();
    let host = require_host(agent, gc)?;
    Ok(number(agent, host.extern_new() as f64, gc))
}
