// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! WHATWG Streams algorithms over per-agent weak brands and per-realm intrinsics.
//! Fetch captures the operation table during synchronous host installation.
//! Neither the table nor its records are retained on the author global.

use script_engine_api::ScriptEngine;

pub(crate) fn install_streams_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.eval(STREAMS_BOOTSTRAP)?;
    Ok(())
}

/// Structured clone is installed after Fetch's Blob/File constructors. Finish
/// the private capture before any script can run and remove the one-shot bridge.
pub(crate) fn finish_streams_install<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.eval("globalThis.__finishStreamsClone(globalThis.__structuredClone); delete globalThis.__finishStreamsClone")?;
    Ok(())
}

const STREAMS_BOOTSTRAP: &str = concat!(
    "(function () { 'use strict';\n",
    include_str!("streams/core.js"),
    include_str!("streams/readable.js"),
    include_str!("streams/writable.js"),
    include_str!("streams/piping.js"),
    r#"
  var bridge = Core.record();
  bridge.core = Core; bridge.readable = ReadableOps;
  bridge.writable = WritableOps; bridge.piping = PipingOps;
  Core.define(globalThis, '__streamsFetch', {
    value: bridge, configurable: true, writable: false, enumerable: false
  });
  Core.define(globalThis, '__finishStreamsClone', {
    value: function (clone) {
      Core.clone = function (chunk) {
        // The Fetch byte path must not consult mutable view constructors,
        // accessors, or structuredClone. Preserve the full backing buffer and
        // offset when cloning a view for the second tee branch.
        var info;
        try { info = Core.viewInfo(chunk); } catch (_) {}
        if (info !== undefined) {
          var bytes = Core.copyBytes(new Core.Uint8Array(info.buffer));
          return Core.view(info.kind, Core.viewInfo(bytes).buffer, info.byteOffset, info.byteLength);
        }
        return Core.call(clone, undefined, [chunk]);
      };
    }, configurable: true, writable: false, enumerable: false
  });
})();
"#
);
