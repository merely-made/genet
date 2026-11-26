// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Types described as JSON for the bootstrap, shaped like type reflection's
//! descriptors and named as it names them (`funcref`; the bootstrap also
//! takes the older `anyfunc`).

use wasmi::{
    ExternType, FuncType, GlobalType, MemoryType, Mutability, RefType, TableType, ValType,
};

pub fn val_type_name(ty: ValType) -> &'static str {
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
pub fn val_type(name: &str) -> Option<ValType> {
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

pub fn ref_type(name: &str) -> Option<RefType> {
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

pub fn string(out: &mut String, s: &str) {
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

/// A limit as a JSON number, or a string for a 64-bit type, which the
/// bootstrap turns into a BigInt.
fn limit(out: &mut String, name: &str, value: u64, is64: bool) {
    let quote = if is64 { "\"" } else { "" };
    out.push_str(&format!("\"{name}\":{quote}{value}{quote}"));
}

fn maximum(out: &mut String, max: Option<u64>, is64: bool) {
    if let Some(max) = max {
        out.push(',');
        limit(out, "maximum", max, is64);
    }
}

fn address(out: &mut String, is64: bool) {
    out.push_str(if is64 {
        ",\"address\":\"i64\""
    } else {
        ",\"address\":\"i32\""
    });
}

/// `"parameters":[…],"results":[…]`.
pub fn func(out: &mut String, ty: &FuncType) {
    out.push_str("\"parameters\":");
    types(out, ty.params());
    out.push_str(",\"results\":");
    types(out, ty.results());
}

/// A memory's type, with `minimum` its current size in pages as type
/// reflection reports it.
pub fn memory(out: &mut String, ty: &MemoryType, size: u64) {
    limit(out, "minimum", size, ty.is_64());
    maximum(out, ty.maximum(), ty.is_64());
    out.push_str(",\"shared\":false");
    address(out, ty.is_64());
}

/// A table's type, with `minimum` its current length.
pub fn table(out: &mut String, ty: &TableType, size: u64) {
    limit(out, "minimum", size, ty.is_64());
    maximum(out, ty.maximum(), ty.is_64());
    out.push_str(",\"element\":");
    string(out, ref_type_name(ty.element()));
    address(out, ty.is_64());
}

pub fn global(out: &mut String, ty: &GlobalType) {
    out.push_str(&format!(
        "\"mutable\":{},\"value\":",
        ty.mutability() == Mutability::Var
    ));
    string(out, val_type_name(ty.content()));
}

pub fn kind_name(ty: &ExternType) -> &'static str {
    match ty {
        ExternType::Func(_) => "function",
        ExternType::Table(_) => "table",
        ExternType::Memory(_) => "memory",
        ExternType::Global(_) => "global",
    }
}

/// `"kind":…,"type":{…}` for an import or export.
pub fn extern_type(out: &mut String, ty: &ExternType) {
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
