/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Per-document capabilities installed before authored scripts execute.

use script_runtime_api::{ValidationMessageCatalog, WebGlFactory};

/// Host capabilities for one live scripted document.
///
/// Each navigation receives a fresh value. Fetching and realm routing remain
/// owned by the document's `ScriptResourceBridge`.
#[derive(Default)]
pub struct ScriptedDocumentOptions {
    pub webgl: Option<WebGlFactory>,
    /// Built-in constraint-validation wording installed before any authored
    /// scripts run. Custom validity text remains exact and bypasses this.
    pub validation_message_catalog: ValidationMessageCatalog,
}
