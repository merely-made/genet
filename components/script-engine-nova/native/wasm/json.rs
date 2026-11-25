// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Types described as JSON for the bootstrap, shaped like type reflection's
//! descriptors and named as it names them (`funcref`; the bootstrap also
//! takes the older `anyfunc`).

use wasmi::{
    ExternType, FuncType, GlobalType, MemoryType, Mutability, RefType, TableType, ValType,
};

pub(super) fn val_type_name(ty: ValType) -> &'static str {
    match ty {
        ValType::I32 => "i32",
        ValType::I64 => "i64",
        ValType::F32 => "f32",
        ValType::F64 => "f64",
        ValType::V128 => "v128",
        ValType::FuncRef => "funcref",
        ValType::ExternRef => "externref",
    }
}

/// The value type a JS API name stands for.
pub(super) fn val_type(name: &str) -> Option<ValType> {
    Some(match name {
        "i32" => ValType::I32,
        "i64" => ValType::I64,
        "f32" => ValType::F32,
        "f64" => ValType::F64,
        "v128" => ValType::V128,
        "anyfunc" | "funcref" => ValType::FuncRef,
        "externref" => ValType::ExternRef,
        _ => return None,
    })
}

pub(super) fn ref_type(name: &str) -> Option<RefType> {
    match val_type(name)? {
        ValType::FuncRef => Some(RefType::Func),
        ValType::ExternRef => Some(RefType::Extern),
        _ => None,
    }
}

fn ref_type_name(ty: RefType) -> &'static str {
    match ty {
        RefType::Func => "funcref",
        RefType::Extern => "externref",
    }
}

pub(super) fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn types(out: &mut String, types: &[ValType]) {
    out.push('[');
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(out, val_type_name(*ty));
    }
    out.push(']');
}

fn maximum(out: &mut String, max: Option<u64>) {
    if let Some(max) = max {
        out.push_str(&format!(",\"maximum\":{max}"));
    }
}

/// `"parameters":[…],"results":[…]`.
pub(super) fn func(out: &mut String, ty: &FuncType) {
    out.push_str("\"parameters\":");
    types(out, ty.params());
    out.push_str(",\"results\":");
    types(out, ty.results());
}

/// A memory's type, with `minimum` its current size in pages as type
/// reflection reports it.
pub(super) fn memory(out: &mut String, ty: &MemoryType, size: u64) {
    out.push_str(&format!("\"minimum\":{size}"));
    maximum(out, ty.maximum());
    out.push_str(",\"shared\":false,\"index\":\"i32\"");
}

/// A table's type, with `minimum` its current length.
pub(super) fn table(out: &mut String, ty: &TableType, size: u64) {
    out.push_str(&format!("\"minimum\":{size}"));
    maximum(out, ty.maximum());
    out.push_str(",\"element\":");
    string(out, ref_type_name(ty.element()));
}

pub(super) fn global(out: &mut String, ty: &GlobalType) {
    out.push_str(&format!(
        "\"mutable\":{},\"value\":",
        ty.mutability() == Mutability::Var
    ));
    string(out, val_type_name(ty.content()));
}

pub(super) fn kind_name(ty: &ExternType) -> &'static str {
    match ty {
        ExternType::Func(_) => "function",
        ExternType::Table(_) => "table",
        ExternType::Memory(_) => "memory",
        ExternType::Global(_) => "global",
    }
}

/// `"kind":…,"type":{…}` for an import or export.
pub(super) fn extern_type(out: &mut String, ty: &ExternType) {
    out.push_str("\"kind\":");
    string(out, kind_name(ty));
    out.push_str(",\"type\":{");
    match ty {
        ExternType::Func(ty) => func(out, ty),
        ExternType::Table(ty) => table(out, ty, ty.minimum()),
        ExternType::Memory(ty) => memory(out, ty, ty.minimum()),
        ExternType::Global(ty) => global(out, ty),
    }
    out.push('}');
}
