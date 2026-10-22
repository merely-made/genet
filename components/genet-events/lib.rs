// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! One DOM event dispatch algorithm for every listener owner.
//!
//! The layer holds a [`Registry`] of listeners keyed by [`TargetKey`], builds the
//! shadow-including event path ([`build_path`]) and walks it in the DOM's phases
//! ([`dispatch`]), asking an [`Invoker`] to run each listener. It names no script
//! engine: a listener is `(owner, token)` and the owner decides what the token
//! means (a rooted JS callback, a toolkit view path). Plan:
//! `docs/2026-10-17_native_event_dispatch_plan.md`.

#![deny(unsafe_code)]

mod dispatch;
mod path;
mod registry;
#[cfg(test)]
mod tests;

pub use dispatch::{
    dispatch, DispatchOutcome, EventInit, Invocation, InvokeResult, Invoker, Phase,
};
pub use path::{build_path, composed_path, EventPath, PathItem};
pub use registry::{Listener, ListenerSpec, Owner, Registry, TargetKey};
