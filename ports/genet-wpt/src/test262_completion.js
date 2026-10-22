// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// Evaluated after doneprintHandle.js. The returned callable stays only in the
// Rust host; test source can replace $DONE or print but cannot edit this state.
(function () {
  const harnessDone = globalThis.$DONE;
  if (typeof harnessDone !== 'function') {
    throw new TypeError('Test262 async harness did not install $DONE');
  }
  const apply = Reflect.apply;
  let calls = 0;
  let failed = false;
  globalThis.$DONE = function (error) {
    const result = apply(harnessDone, this, arguments);
    calls += 1;
    if (error) failed = true;
    return result;
  };
  // 0 = missing/duplicate, 1 = one success, 2 = one truthy error.
  return function () {
    return calls === 1 ? (failed ? 2 : 1) : 0;
  };
})()
