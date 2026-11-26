// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// The bootstrap continues: conversions, functions and the entity classes.
  // Script values to wasm values, and back; each takes the keeper of the group
  // it crosses in. A wasm function's wrapper is known by its handle and group.
  const funcInfo = new WeakMap();
  const toWasm = {
    i32: (v) => v | 0,
    i64: (v) => BigInt.asIntN(64, v),
    f32: (v) => +v,
    f64: (v) => +v,
    externref: (v, group) => (v === null ? null : hold(group, v)),
    anyfunc: (v, group) => {
      if (v === null) return null;
      const info = funcInfo.get(v);
      if (info === undefined) throw new TypeError('expected an exported WebAssembly function or null');
      join(group, info.group);
      return info.handle;
    },
    v128: () => {
      throw new TypeError('v128 values cannot cross to or from script');
    },
  };
  toWasm.funcref = toWasm.anyfunc;
  const to = (type) => toWasm[type] ?? (() => {
    throw new TypeError(`${type} values cannot cross`);
  });
  const from = (type) => {
    if (type === 'anyfunc' || type === 'funcref') return (h, group) => (h === null ? null : exportedFunction(h, group));
    if (type === 'externref') return (slot, group) => (slot === null ? null : root(group).values.get(slot));
    if (type === 'v128') return toWasm.v128;
    return (v) => v;
  };
  const VALUE_TYPES = ['i32', 'i64', 'f32', 'f64', 'v128', 'externref', 'anyfunc', 'funcref'];
  const valueType = (value, what) => {
    const type = String(value);
    if (!VALUE_TYPES.includes(type)) throw new TypeError(`${what} must be a value type`);
    return type;
  };
  const defaultValue = (type) => ({ i32: 0, i64: 0n, f32: 0, f64: 0, anyfunc: null, funcref: null })[type];
  // The memory64 additions: an `address` of "i32" or "i64" makes limits, deltas,
  // indices and sizes numbers or BigInts. An i64 value is a BigInt in 0..2^64-1,
  // or a number standing for one.
  const addressType = (value) => {
    if (value === undefined) return 'i32';
    const type = String(value);
    if (type !== 'i32' && type !== 'i64') throw new TypeError('address must be "i32" or "i64"');
    return type;
  };
  const addressValue = (value, what, is64) => {
    if (!is64) return enforceRange(value, what);
    let n = value;
    if (typeof n !== 'bigint') {
      n = Number(n);
      if (!Number.isFinite(n)) throw new TypeError(`${what} must be a finite number`);
      n = BigInt(Math.trunc(n));
    }
    if (n < 0n || n > MAX_U64) throw new TypeError(`${what} is out of range`);
    return n;
  };
  // What a native takes for a count: a decimal string for i64 limits, which may
  // not fit a double, and a number, saturated, for deltas and indices.
  const limitArg = (n) => (n === undefined ? undefined : String(n));
  const countArg = (n) => (typeof n === 'bigint' ? Number(n < 2n ** 53n ? n : 2n ** 53n - 1n) : n);
  const sizeResult = (n, is64) => (is64 ? BigInt(n) : n);
  // A reflected type's limits are BigInts for an i64 address.
  const reflect = (type) => {
    if (type.address !== 'i64') return type;
    const out = { ...type, minimum: BigInt(type.minimum) };
    if ('maximum' in type) out.maximum = BigInt(type.maximum);
    return out;
  };
  // A descriptor's limits, each member read once and in WebIDL's order; type
  // reflection's `minimum` stands in for `initial`.
  const limits = (descriptor, is64) => {
    const read = (name) => {
      const value = descriptor[name];
      return value === undefined ? undefined : addressValue(value, name, is64);
    };
    const initial = read('initial');
    const maximum = read('maximum');
    const minimum = read('minimum');
    if (initial !== undefined && minimum !== undefined) throw new TypeError('give initial or minimum, not both');
    const min = initial ?? minimum;
    if (min === undefined) throw new TypeError('initial is required');
    return { initial: min, maximum };
  };

  // A wasm function crossing into script, one script function per handle
  // while it lives.
  const callWasm = (handle, results, args, group) => {
    let r;
    try {
      r = N.Call(TRAPPED, group.call, handle, ...args);
    } finally {
      sync();
    }
    if (r === TRAPPED) throw new RuntimeError(N.TrapMessage());
    if (results.length === 0) return undefined;
    if (results.length === 1) return results[0](r, group);
    return results.map((convert, i) => convert(N.ResultAt(i), group));
  };
  const exportedFunction = (handle, group) => {
    const known = identity[FUNC].get(handle)?.deref();
    if (known) {
      join(group, funcInfo.get(known).group);
      return known;
    }
    const sig = parse(N.Signature(handle));
    const params = sig.parameters.map(to);
    const results = sig.results.map(from);
    // A v128 anywhere in the signature refuses the call before it runs.
    const crossing = [...sig.parameters, ...sig.results].includes('v128');
    const f = (...args) => {
      if (crossing) throw new TypeError('v128 values cannot cross to or from script');
      return callWasm(handle, results, params.map((convert, i) => convert(args[i], group)), group);
    };
    setPrototypeOf(f, WasmFunction.prototype);
    defineProperty(f, 'name', { value: String(sig.index ?? handle), configurable: true });
    defineProperty(f, 'length', { value: sig.parameters.length, configurable: true });
    funcInfo.set(f, { handle, group });
    identity[FUNC].set(handle, new WeakRef(f));
    return own(FUNC, handle, f);
  };
  // A script function wasm calls: arguments from wasm, results to wasm. Several
  // results are staged for the bridge, which reads them as the wrapper returns.
  const scriptWrapper = (fn, type, group) => {
    const params = type.parameters.map(from);
    const results = type.results.map(to);
    return (...args) => {
      sync();
      const value = fn(...params.map((convert, i) => convert(args[i], group)));
      if (results.length === 0) return undefined;
      if (results.length === 1) return results[0](value, group);
      const values = [...value];
      if (values.length !== results.length) throw new TypeError('a script function returned the wrong number of values');
      N.Stage(...values.map((v, i) => results[i](v, group)));
      return undefined;
    };
  };

  // Each entity class keeps its handle in a WeakMap, and one object per handle
  // while it lives.
  const entityClass = (name, kind) => {
    const handles = new WeakMap();
    const objects = identity[kind];
    return {
      handles,
      handle(object) {
        const handle = handles.get(object);
        if (handle === undefined) throw new TypeError(`expected a WebAssembly.${name}`);
        return handle;
      },
      register(object, handle, group) {
        handles.set(object, handle);
        groups.set(object, group);
        objects.set(handle, new WeakRef(object));
        return own(kind, handle, object);
      },
      object(C, handle, group) {
        const known = objects.get(handle)?.deref();
        if (known) {
          join(group, groups.get(known));
          return known;
        }
        return this.register(create(C.prototype), handle, group);
      },
    };
  };
  const memories = entityClass('Memory', MEMORY);
  const globals = entityClass('Global', GLOBAL);
  const tables = entityClass('Table', TABLE);

  // The memory's current buffer, or a new one: fixed-length or resizable as
  // asked, or whichever is current if not. A new one detaches the old.
  const bufferOf = (memory, resizable) => {
    const handle = memories.handle(memory);
    sync();
    const entry = bufferCache.get(handle);
    const old = entry?.ref.deref();
    if (old && (resizable === undefined || entry.resizable === resizable)) return old;
    const buffer = N.MemoryBuffer(handle, resizable === true);
    if (entry !== undefined) {
      bufferCache.delete(handle);
      if (old) N.Detach(old);
      if (entry.rec.live) {
        entry.rec.live = false;
        N.Release(BUFFER, handle, false);
      }
    }
    const rec = { kind: BUFFER, handle, live: true, done: false };
    N.Retain(BUFFER, handle);
    watch(buffer, rec);
    bufferCache.set(handle, { ref: new WeakRef(buffer), rec, resizable: resizable === true });
    groups.set(buffer, groups.get(memory));
    return buffer;
  };

  class Memory {
    constructor(descriptor) {
      if (!isObject(descriptor)) throw new TypeError('the memory descriptor must be an object');
      const is64 = addressType(descriptor.address) === 'i64';
      const { initial, maximum } = limits(descriptor, is64);
      if (descriptor.shared) throw new TypeError('shared memory is not supported yet');
      const cap = is64 ? MAX_PAGES_64 : MAX_PAGES;
      if (initial > cap || (maximum !== undefined && (maximum > cap || maximum < initial))) {
        throw new RangeError('invalid memory limits');
      }
      memories.register(this, N.MemoryNew(limitArg(initial), limitArg(maximum), is64), new Keeper());
    }
    grow(delta) {
      const handle = memories.handle(this);
      const is64 = parse(N.MemoryType(handle)).address === 'i64';
      const old = N.MemoryGrow(handle, countArg(addressValue(delta, 'delta', is64)));
      sync(handle);
      return sizeResult(old, is64);
    }
    get buffer() {
      return bufferOf(this);
    }
    toFixedLengthBuffer() {
      return bufferOf(this, false);
    }
    toResizableBuffer() {
      return bufferOf(this, true);
    }
    type() {
      return reflect(parse(N.MemoryType(memories.handle(this))));
    }
  }

  class Global {
    constructor(descriptor, v = undefined) {
      if (!isObject(descriptor)) throw new TypeError('the global descriptor must be an object');
      const mutable = Boolean(descriptor.mutable);
      const type = valueType(descriptor.value, 'value');
      if (type === 'v128') throw new TypeError('v128 globals cannot be made from script');
      const value = v === undefined ? defaultValue(type) : v;
      const group = new Keeper();
      globals.register(this, N.GlobalNew(type, mutable, to(type)(value, group)), group);
    }
    get value() {
      const handle = globals.handle(this);
      return from(parse(N.GlobalType(handle)).value)(N.GlobalGet(handle), groups.get(this));
    }
    set value(v) {
      const handle = globals.handle(this);
      const type = parse(N.GlobalType(handle));
      if (!type.mutable) throw new TypeError('the global is immutable');
      N.GlobalSet(handle, to(type.value)(v, groups.get(this)));
    }
    valueOf() {
      globals.handle(this);
      return this.value;
    }
    type() {
      return parse(N.GlobalType(globals.handle(this)));
    }
  }

  const elementType = (handle) => parse(N.TableType(handle)).element;
  const is64Table = (handle) => parse(N.TableType(handle)).address === 'i64';
  class Table {
    constructor(descriptor, value = undefined) {
      if (!isObject(descriptor)) throw new TypeError('the table descriptor must be an object');
      const is64 = addressType(descriptor.address) === 'i64';
      const element = String(descriptor.element);
      if (!['anyfunc', 'funcref', 'externref'].includes(element)) {
        throw new TypeError('element must be anyfunc, funcref or externref');
      }
      const { initial, maximum } = limits(descriptor, is64);
      if (initial > MAX_TABLE || (maximum !== undefined && maximum < initial)) {
        throw new RangeError('invalid table limits');
      }
      const init = value === undefined ? defaultValue(element) : value;
      const group = new Keeper();
      tables.register(this, N.TableNew(element, limitArg(initial), limitArg(maximum), to(element)(init, group), is64), group);
    }
    get length() {
      const handle = tables.handle(this);
      return sizeResult(N.TableLength(handle), is64Table(handle));
    }
    get(index) {
      const handle = tables.handle(this);
      const i = countArg(addressValue(index, 'index', is64Table(handle)));
      return from(elementType(handle))(N.TableGet(handle, i), groups.get(this));
    }
    set(index, value = undefined) {
      const handle = tables.handle(this);
      const i = countArg(addressValue(index, 'index', is64Table(handle)));
      const element = elementType(handle);
      const v = arguments.length < 2 ? defaultValue(element) : value;
      N.TableSet(handle, i, to(element)(v, groups.get(this)));
    }
    grow(delta, value = undefined) {
      const handle = tables.handle(this);
      const is64 = is64Table(handle);
      const d = countArg(addressValue(delta, 'delta', is64));
      const element = elementType(handle);
      const v = arguments.length < 2 ? defaultValue(element) : value;
      return sizeResult(N.TableGrow(handle, d, to(element)(v, groups.get(this))), is64);
    }
    type() {
      return reflect(parse(N.TableType(tables.handle(this))));
    }
  }

  // Type reflection: exported functions are instances of WebAssembly.Function.
  function WasmFunction(type, fn) {
    if (new.target === undefined) throw new TypeError('WebAssembly.Function must be called with new');
    if (!isObject(type)) throw new TypeError('the function type must be an object');
    const parameters = [...type.parameters].map((t) => valueType(t, 'parameter'));
    const results = [...type.results].map((t) => valueType(t, 'result'));
    if (typeof fn !== 'function') throw new TypeError('the function must be callable');
    const group = new Keeper();
    const slot = holdFn(group, scriptWrapper(fn, { parameters, results }, group));
    return exportedFunction(N.FuncNew(parameters.join(','), results.join(','), slot), group);
  }
  setPrototypeOf(WasmFunction.prototype, Function.prototype);
  defineProperty(WasmFunction.prototype, 'type', {
    value: function type() {
      const info = funcInfo.get(this);
      if (info === undefined) throw new TypeError('expected a WebAssembly.Function');
      const { parameters, results } = parse(N.Signature(info.handle));
      return { parameters, results };
    },
    writable: true,
    configurable: true,
  });
  defineProperty(WasmFunction, 'name', { value: 'Function' });
