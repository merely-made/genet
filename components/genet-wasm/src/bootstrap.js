// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// The `WebAssembly` namespace over the natives in natives.rs and
// entities_natives.rs (Genet's WebAssembly object plan, W1, W2 and W4). The
// conversions of the JS API live here: values reach the natives converted for
// their wasm type, a `funcref` as its function handle, an `externref` as a
// slot number, and error strings become the namespace's error classes. A
// memory's buffer is its linear memory, not a copy.
//
// Freeing (D6). Every wrapper, and every buffer over a memory, retains its
// handle with the host and registers with one FinalizationRegistry, whose
// cleanup releases it. Script values wasm holds (import functions, externrefs)
// are not rooted by the host. They live in a keeper, one for each linked group
// of instances and what they share, which the group's wrappers reference. The
// tracing collector then frees a group, cycles through its imports included,
// and a group that two wrappers come to share is merged into one keeper.
(() => {
  'use strict';
  const g = globalThis;
  const N = {};
  for (const name of [
    'Compile', 'Describe', 'Instantiate', 'ExportOf', 'Signature', 'Call', 'TrapMessage',
    'ResultAt', 'Stage', 'FuncNew', 'CustomSection', 'MemoryNew', 'MemoryGrow', 'MemoryBuffer', 'MemoryType',
    'Stale', 'Detach', 'Repoint', 'Retain', 'Release', 'Extern',
    'GlobalNew', 'GlobalGet', 'GlobalSet', 'GlobalType', 'TableNew', 'TableGet', 'TableSet',
    'TableGrow', 'TableLength', 'TableType',
  ]) {
    N[name] = g[`__genetWasm${name}`];
    delete g[`__genetWasm${name}`];
  }
  const { create, defineProperty, freeze, setPrototypeOf } = Object;
  const parse = JSON.parse;
  const KIND = { function: 0, table: 1, memory: 2, global: 3 };
  // What a handle names, as `genet_wasm::handles` numbers them.
  const FUNC = 0, TABLE = 1, MEMORY = 2, GLOBAL = 3, MODULE = 4, INSTANCE = 5, BUFFER = 6, EXTERN = 7;
  // An import plan entry of this kind names a script function's keeper slot.
  const PLAN_SCRIPT = 4;
  const TRAPPED = freeze(create(null));
  const MAX_PAGES = 65536;
  const MAX_PAGES_64 = 1n << 48n;
  const MAX_U64 = (1n << 64n) - 1n;
  const MAX_TABLE = 10000000;

  const errorClass = (name) => {
    const C = class extends Error {};
    defineProperty(C, 'name', { value: name, configurable: true });
    defineProperty(C.prototype, 'name', { value: name, writable: true, configurable: true });
    return C;
  };
  const CompileError = errorClass('CompileError');
  const LinkError = errorClass('LinkError');
  const RuntimeError = errorClass('RuntimeError');

  const isObject = (v) => (typeof v === 'object' && v !== null) || typeof v === 'function';
  // [EnforceRange] unsigned long.
  const enforceRange = (value, what) => {
    const n = Number(value);
    if (!Number.isFinite(n)) throw new TypeError(`${what} must be a finite number`);
    const t = Math.trunc(n);
    if (t < 0 || t > 0xffffffff) throw new TypeError(`${what} is out of range`);
    return t;
  };

  // Buffers script holds over memories, by memory handle: the current one,
  // weakly, and the record that retains the memory while it is attached.
  const bufferCache = new Map();
  // One wrapper per handle while it lives, by kind (FUNC to GLOBAL).
  const identity = [new Map(), new Map(), new Map(), new Map()];
  // The keeper of every wrapper and of every buffer over a memory.
  const groups = new WeakMap();

  const applyStale = (list) => {
    for (const item of list.split(',')) {
      const handle = Number(item.slice(1));
      const entry = bufferCache.get(handle);
      const buffer = entry?.ref.deref();
      if (item[0] === 'r') {
        if (buffer) N.Repoint(handle, buffer);
        continue;
      }
      if (entry === undefined) continue;
      bufferCache.delete(handle);
      if (buffer) N.Detach(buffer);
      if (entry.rec.live) {
        entry.rec.live = false;
        N.Release(BUFFER, handle, false);
      }
    }
  };
  const release = (kind, handle, current = false) => {
    const stale = N.Release(kind, handle, current);
    if (stale) applyStale(stale);
  };
  // Detach every buffer whose memory has moved, and `force`'s in any case.
  const sync = (force = -1) => {
    if (bufferCache.size === 0) return;
    const list = N.Stale(force);
    if (list !== '') applyStale(list);
  };

  const registry = new FinalizationRegistry((held) => {
    if (held.done) return;
    held.done = true;
    if (held.kind === EXTERN) {
      for (const id of held.ids) N.Release(EXTERN, id, false);
    } else if (held.kind === BUFFER) {
      if (!held.live) return;
      held.live = false;
      const current = bufferCache.get(held.handle)?.rec === held;
      if (current) bufferCache.delete(held.handle);
      release(BUFFER, held.handle, current);
    } else {
      release(held.kind, held.handle);
      const wrappers = identity[held.kind];
      if (wrappers?.get(held.handle)?.deref() === undefined) wrappers?.delete(held.handle);
    }
  });
  const watch = (object, held) => registry.register(object, held);
  const own = (kind, handle, object) => {
    N.Retain(kind, handle);
    watch(object, { kind, handle, done: false });
    return object;
  };

  // A keeper holds what wasm in its group refers to by number: externref
  // values by slot, and the wrappers of imported script functions. `into` is
  // set once it is merged into another.
  let nextFn = 0;
  const root = (keeper) => {
    while (keeper.into !== null) keeper = keeper.into;
    return keeper;
  };
  function Keeper() {
    this.values = new Map();
    this.slots = new Map();
    this.fns = new Map();
    this.ledger = { kind: EXTERN, ids: [], done: false };
    this.into = null;
    // What the natives call to reach an import: the wrapper in `slot`.
    this.call = (slot, ...args) => root(this).fns.get(slot)(...args);
    watch(this, this.ledger);
  }
  const join = (a, b) => {
    a = root(a);
    b = root(b);
    if (a === b) return a;
    if (a.values.size + a.fns.size < b.values.size + b.fns.size) [a, b] = [b, a];
    for (const [slot, value] of b.values) a.values.set(slot, value);
    for (const [value, slot] of b.slots) a.slots.set(value, slot);
    for (const [slot, fn] of b.fns) a.fns.set(slot, fn);
    for (const id of b.ledger.ids) a.ledger.ids.push(id);
    b.ledger.ids = [];
    b.values = b.slots = b.fns = null;
    b.into = a;
    return a;
  };
  // The slot an externref value holds in the group, the same for the same value.
  const hold = (keeper, value) => {
    const k = root(keeper);
    const negativeZero = value === 0 && 1 / value < 0; // a Map would take it for 0
    let slot = negativeZero ? undefined : k.slots.get(value);
    if (slot === undefined) {
      slot = N.Extern();
      k.values.set(slot, value);
      if (!negativeZero) k.slots.set(value, slot);
      k.ledger.ids.push(slot);
    }
    return slot;
  };
  const holdFn = (keeper, fn) => {
    const slot = nextFn++;
    root(keeper).fns.set(slot, fn);
    return slot;
  };
