// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// A small regression for the actual vendored Test262 $DONE harness and the
// private completion tracker embedded by genet-wpt.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import vm from 'node:vm';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const harness = readFileSync(resolve(root, 'tests/wpt/tests/third_party/test262/harness/doneprintHandle.js'), 'utf8');
const bootstrap = readFileSync(resolve(root, 'ports/genet-wpt/src/test262_completion.js'), 'utf8');

function scenario(body) {
  const messages = [];
  const context = vm.createContext({ print: message => messages.push(String(message)) });
  vm.runInContext(harness, context);
  const poll = vm.runInContext(bootstrap, context);
  assert.equal(typeof poll, 'function');
  assert.equal(Object.hasOwn(context, '__262poll'), false);
  vm.runInContext(body, context);
  return { status: poll(), messages };
}

assert.equal(scenario('').status, 0, 'no $DONE call is incomplete');
assert.equal(scenario("print('Test262:AsyncTestComplete'); globalThis.__262log='Test262:AsyncTestComplete';").status, 0,
  'printed and writable global markers cannot complete a test');
assert.equal(scenario('$DONE();').status, 1, 'one $DONE call completes');
assert.equal(scenario('$DONE(0);').status, 1, 'falsey error matches doneprintHandle semantics');
assert.equal(scenario("$DONE(new TypeError('bad'));").status, 2, 'truthy $DONE error fails');
assert.equal(scenario('$DONE(); $DONE();').status, 0, 'duplicate completion is invalid');
assert.equal(scenario("globalThis.$DONE = function() { print('Test262:AsyncTestComplete'); }; $DONE();").status, 0,
  'replacing global $DONE cannot complete through the private tracker');
assert.equal(scenario("globalThis.print = function() { throw new Error('print failed'); }; try { $DONE(); } catch (_) {}").status, 0,
  'a harness $DONE call that throws before reporting is incomplete');

console.log('test262 private completion: passed');
