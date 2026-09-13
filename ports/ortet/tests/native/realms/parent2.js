// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Completion requires fetching and executing this external resource after a
// top-level navigation. Keep the same document shape as the first pass so the
// live-node census still compares equivalent trees.
if (document.body.dataset.inlineReady !== "yes") {
  throw new Error("the inline setup did not precede the external script");
}
document.getElementById("state").firstChild.data = "Ortet G5 realms sequence complete";
document.documentElement.className = "complete";
