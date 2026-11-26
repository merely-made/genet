// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The natives that own and free handles (`genet-wasm`'s `handles.rs`).

use boa_engine::{Context, JsResult, JsValue};

use super::bridge::{arg, handle_arg, number, require_host, string, type_error};

fn kind_and_handle(args: &[JsValue]) -> Option<(u8, usize)> {
    let kind = handle_arg(&arg(args, 0)).and_then(|k| u8::try_from(k).ok())?;
    Some((kind, handle_arg(&arg(args, 1))?))
}

/// `retain(kind, handle)`: add an owner to a handle.
pub(super) fn retain(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let (kind, handle) =
        kind_and_handle(args).ok_or_else(|| type_error("expected a kind and a handle"))?;
    host.retain(kind, handle);
    Ok(JsValue::undefined())
}

/// `release(kind, handle, current)`: drop an owner. Returns the buffers to
/// detach, as `stale` does, if that rebuilt the store.
pub(super) fn release(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let host = require_host(ctx)?;
    let (kind, handle) =
        kind_and_handle(args).ok_or_else(|| type_error("expected a kind and a handle"))?;
    let left = host.release(kind, handle, arg(args, 2).as_boolean() == Some(true));
    let list: Vec<String> = left.iter().map(|memory| format!("d{memory}")).collect();
    Ok(string(list.join(",")))
}

/// `extern()`: a fresh slot for a value an `externref` holds.
pub(super) fn extern_new(_: &JsValue, _: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    Ok(number(require_host(ctx)?.extern_new()))
}
