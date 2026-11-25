// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The engine-free half of Genet's `WebAssembly` object (its WebAssembly
//! object plan, D12): one wasmi store per JavaScript agent, the handle tables
//! script reaches it through, and the bootstrap that builds the namespace.
//!
//! Each JavaScript backend supplies the rest: natives that convert its values,
//! and the call bridge that runs script while a wasm call is suspended. `V`
//! is the backend's rooted script value.

pub mod entities;
pub mod host;
pub mod json;
pub mod memory;
pub mod sections;

pub use host::{ImportCall, ImportValue, WasmHost};
pub use memory::Refresh;
pub use wasmi;

/// The script that builds `WebAssembly` over a backend's natives. It takes
/// each `__genetWasm…` global it needs and deletes it.
pub const BOOTSTRAP: &str = include_str!("bootstrap.js");

/// The natives the bootstrap expects, by the name after `__genetWasm`, with
/// their `length`.
pub const NATIVES: [(&str, u32); 27] = [
    ("Compile", 1),
    ("Describe", 1),
    ("Instantiate", 2),
    ("ExportOf", 3),
    ("Signature", 1),
    ("Call", 2),
    ("TrapMessage", 0),
    ("ResultAt", 1),
    ("Stage", 0),
    ("FuncNew", 3),
    ("CustomSection", 3),
    ("MemoryNew", 2),
    ("MemoryGrow", 2),
    ("MemoryBuffer", 1),
    ("MemoryType", 1),
    ("MemoryToFixed", 1),
    ("MemoryToResizable", 1),
    ("GlobalNew", 3),
    ("GlobalGet", 1),
    ("GlobalSet", 2),
    ("GlobalType", 1),
    ("TableNew", 4),
    ("TableGet", 2),
    ("TableSet", 3),
    ("TableGrow", 3),
    ("TableLength", 1),
    ("TableType", 1),
];

/// A JS API page, in bytes.
pub const PAGE: u64 = 65536;
