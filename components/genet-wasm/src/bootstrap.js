// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// The `WebAssembly` namespace over the natives in natives.rs and
// entities_natives.rs (Genet's WebAssembly object plan, W1, W2 and W4). The
// conversions of the JS API live here: values reach the natives converted for
// their wasm type, a `funcref` as its function handle, and error strings
// become the namespace's error classes. A memory's buffer is its linear
// memory, not a copy.
(() => {
  'use strict';
  const g = globalThis;
  const N = {};
  for (const name of [
    'Compile', 'Describe', 'Instantiate', 'ExportOf', 'Signature', 'Call', 'TrapMessage',
    'ResultAt', 'Stage', 'FuncNew', 'CustomSection', 'MemoryNew', 'MemoryGrow', 'MemoryBuffer', 'MemoryType',
    'MemoryToFixed', 'MemoryToResizable',
    'GlobalNew', 'GlobalGet', 'GlobalSet', 'GlobalType', 'TableNew', 'TableGet', 'TableSet',
    'TableGrow', 'TableLength', 'TableType',
  ]) {
    N[name] = g[`__genetWasm${name}`];
    delete g[`__genetWasm${name}`];
  }
  const { create, defineProperty, freeze, setPrototypeOf } = Object;
  const parse = JSON.parse;
  const KIND = { function: 0, table: 1, memory: 2, global: 3 };
  const TRAPPED = freeze(create(null));
  const MAX_PAGES = 65536;
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

  // Script values to wasm values, and back.
  const funcObjects = new Map();
  const funcHandles = new WeakMap();
  const toWasm = {
    i32: (v) => v | 0,
    i64: (v) => BigInt.asIntN(64, v),
    f32: (v) => +v,
    f64: (v) => +v,
    externref: (v) => v,
    anyfunc: (v) => {
      if (v === null) return null;
      const handle = funcHandles.get(v);
      if (handle === undefined) throw new TypeError('expected an exported WebAssembly function or null');
      return handle;
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
    if (type === 'anyfunc' || type === 'funcref') return (h) => (h === null ? null : exportedFunction(h));
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
  // A descriptor's limits, each member read once and in WebIDL's order; type
  // reflection's `minimum` stands in for `initial`.
  const limits = (descriptor) => {
    const read = (name) => {
      const value = descriptor[name];
      return value === undefined ? undefined : enforceRange(value, name);
    };
    const initial = read('initial');
    const maximum = read('maximum');
    const minimum = read('minimum');
    if (initial !== undefined && minimum !== undefined) throw new TypeError('give initial or minimum, not both');
    const min = initial ?? minimum;
    if (min === undefined) throw new TypeError('initial is required');
    return { initial: min, maximum };
  };

  // A wasm function crossing into script, one script function per handle.
  const callWasm = (handle, results, args) => {
    const r = N.Call(TRAPPED, handle, ...args);
    if (r === TRAPPED) throw new RuntimeError(N.TrapMessage());
    if (results.length === 0) return undefined;
    if (results.length === 1) return results[0](r);
    return results.map((convert, i) => convert(N.ResultAt(i)));
  };
  const exportedFunction = (handle) => {
    let f = funcObjects.get(handle);
    if (f) return f;
    const sig = parse(N.Signature(handle));
    const params = sig.parameters.map(to);
    const results = sig.results.map(from);
    f = (...args) => callWasm(handle, results, params.map((convert, i) => convert(args[i])));
    setPrototypeOf(f, WasmFunction.prototype);
    defineProperty(f, 'name', { value: String(sig.index ?? handle), configurable: true });
    defineProperty(f, 'length', { value: sig.parameters.length, configurable: true });
    funcObjects.set(handle, f);
    funcHandles.set(f, handle);
    return f;
  };
  // A script function wasm calls: arguments from wasm, results to wasm. Several
  // results are staged for the bridge, which reads them as the wrapper returns.
  const scriptWrapper = (fn, type) => {
    const params = type.parameters.map(from);
    const results = type.results.map(to);
    return (...args) => {
      const value = fn(...params.map((convert, i) => convert(args[i])));
      if (results.length === 0) return undefined;
      if (results.length === 1) return results[0](value);
      const values = [...value];
      if (values.length !== results.length) throw new TypeError('a script function returned the wrong number of values');
      N.Stage(...values.map((v, i) => results[i](v)));
      return undefined;
    };
  };

  // Each entity class keeps its handle in a WeakMap, and one object per handle.
  const entityClass = (name) => {
    const handles = new WeakMap();
    const objects = new Map();
    return {
      handles,
      handle(object) {
        const handle = handles.get(object);
        if (handle === undefined) throw new TypeError(`expected a WebAssembly.${name}`);
        return handle;
      },
      register(object, handle) {
        handles.set(object, handle);
        objects.set(handle, object);
        return object;
      },
      object(C, handle) {
        return objects.get(handle) ?? this.register(create(C.prototype), handle);
      },
    };
  };
  const memories = entityClass('Memory');
  const globals = entityClass('Global');
  const tables = entityClass('Table');

  class Memory {
    constructor(descriptor) {
      if (!isObject(descriptor)) throw new TypeError('the memory descriptor must be an object');
      const { initial, maximum } = limits(descriptor);
      if (descriptor.shared) throw new TypeError('shared memory is not supported yet');
      if (initial > MAX_PAGES || (maximum !== undefined && (maximum > MAX_PAGES || maximum < initial))) {
        throw new RangeError('invalid memory limits');
      }
      memories.register(this, N.MemoryNew(initial, maximum));
    }
    grow(delta) {
      const handle = memories.handle(this);
      return N.MemoryGrow(handle, enforceRange(delta, 'delta'));
    }
    get buffer() {
      return N.MemoryBuffer(memories.handle(this));
    }
    toFixedLengthBuffer() {
      return N.MemoryToFixed(memories.handle(this));
    }
    toResizableBuffer() {
      return N.MemoryToResizable(memories.handle(this));
    }
    type() {
      return parse(N.MemoryType(memories.handle(this)));
    }
  }

  class Global {
    constructor(descriptor, v = undefined) {
      if (!isObject(descriptor)) throw new TypeError('the global descriptor must be an object');
      const mutable = Boolean(descriptor.mutable);
      const type = valueType(descriptor.value, 'value');
      if (type === 'v128') throw new TypeError('v128 globals cannot be made from script');
      const value = v === undefined ? defaultValue(type) : v;
      globals.register(this, N.GlobalNew(type, mutable, to(type)(value)));
    }
    get value() {
      const handle = globals.handle(this);
      return from(parse(N.GlobalType(handle)).value)(N.GlobalGet(handle));
    }
    set value(v) {
      const handle = globals.handle(this);
      const type = parse(N.GlobalType(handle));
      if (!type.mutable) throw new TypeError('the global is immutable');
      N.GlobalSet(handle, to(type.value)(v));
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
  class Table {
    constructor(descriptor, value = undefined) {
      if (!isObject(descriptor)) throw new TypeError('the table descriptor must be an object');
      const element = String(descriptor.element);
      if (!['anyfunc', 'funcref', 'externref'].includes(element)) {
        throw new TypeError('element must be anyfunc, funcref or externref');
      }
      const { initial, maximum } = limits(descriptor);
      if (initial > MAX_TABLE || (maximum !== undefined && maximum < initial)) {
        throw new RangeError('invalid table limits');
      }
      const init = value === undefined ? defaultValue(element) : value;
      tables.register(this, N.TableNew(element, initial, maximum, to(element)(init)));
    }
    get length() {
      return N.TableLength(tables.handle(this));
    }
    get(index) {
      const handle = tables.handle(this);
      return from(elementType(handle))(N.TableGet(handle, enforceRange(index, 'index')));
    }
    set(index, value = undefined) {
      const handle = tables.handle(this);
      const i = enforceRange(index, 'index');
      const element = elementType(handle);
      const v = arguments.length < 2 ? defaultValue(element) : value;
      N.TableSet(handle, i, to(element)(v));
    }
    grow(delta, value = undefined) {
      const handle = tables.handle(this);
      const d = enforceRange(delta, 'delta');
      const element = elementType(handle);
      const v = arguments.length < 2 ? defaultValue(element) : value;
      return N.TableGrow(handle, d, to(element)(v));
    }
    type() {
      return parse(N.TableType(tables.handle(this)));
    }
  }

  // Type reflection: exported functions are instances of WebAssembly.Function.
  function WasmFunction(type, fn) {
    if (new.target === undefined) throw new TypeError('WebAssembly.Function must be called with new');
    if (!isObject(type)) throw new TypeError('the function type must be an object');
    const parameters = [...type.parameters].map((t) => valueType(t, 'parameter'));
    const results = [...type.results].map((t) => valueType(t, 'result'));
    if (typeof fn !== 'function') throw new TypeError('the function must be callable');
    const wrapper = scriptWrapper(fn, { parameters, results });
    return exportedFunction(N.FuncNew(parameters.join(','), results.join(','), wrapper));
  }
  setPrototypeOf(WasmFunction.prototype, Function.prototype);
  defineProperty(WasmFunction.prototype, 'type', {
    value: function type() {
      const handle = funcHandles.get(this);
      if (handle === undefined) throw new TypeError('expected a WebAssembly.Function');
      const { parameters, results } = parse(N.Signature(handle));
      return { parameters, results };
    },
    writable: true,
    configurable: true,
  });
  defineProperty(WasmFunction, 'name', { value: 'Function' });

  const modules = new WeakMap();
  const instanceExports = new WeakMap();

  const bufferBytes = (source) => {
    if (ArrayBuffer.isView(source)) {
      const end = source.byteOffset + source.byteLength;
      return new Uint8Array(source.buffer.slice(source.byteOffset, end));
    }
    if (source instanceof ArrayBuffer) return new Uint8Array(source.slice(0));
    throw new TypeError('expected an ArrayBuffer or a view of one');
  };
  const moduleHandle = (module) => {
    const handle = modules.get(module);
    if (handle === undefined) throw new TypeError('expected a WebAssembly.Module');
    return handle;
  };

  class Module {
    constructor(bytes) {
      const handle = N.Compile(bufferBytes(bytes));
      if (typeof handle === 'string') throw new CompileError(handle);
      modules.set(this, handle);
    }
    static imports(module) {
      return parse(N.Describe(moduleHandle(module))).imports
        .map(({ module, name, kind, type }) => ({ module, name, kind, type }));
    }
    static exports(module) {
      return parse(N.Describe(moduleHandle(module))).exports
        .map(({ name, kind, type }) => ({ name, kind, type }));
    }
    static customSections(module, sectionName) {
      const handle = moduleHandle(module);
      if (arguments.length < 2) throw new TypeError('a section name is required');
      const name = String(sectionName);
      const sections = [];
      for (let i = 0; ; i++) {
        const bytes = N.CustomSection(handle, name, i);
        if (bytes === undefined) return sections;
        sections.push(bytes);
      }
    }
  }

  // One import as the natives take it: a plan entry, and a wrapper if any.
  const linkImport = (imp, value, wrappers) => {
    const where = `${imp.module}.${imp.name}`;
    switch (imp.kind) {
      case 'function': {
        if (typeof value !== 'function') throw new LinkError(`${where} must be a function`);
        const handle = funcHandles.get(value);
        if (handle !== undefined) return `0:${handle}`;
        wrappers.push(scriptWrapper(value, imp.type));
        return 's';
      }
      case 'memory': {
        const handle = memories.handles.get(value);
        if (handle === undefined) throw new LinkError(`${where} must be a WebAssembly.Memory`);
        return `2:${handle}`;
      }
      case 'table': {
        const handle = tables.handles.get(value);
        if (handle === undefined) throw new LinkError(`${where} must be a WebAssembly.Table`);
        return `1:${handle}`;
      }
      default: {
        const known = globals.handles.get(value);
        if (known !== undefined) return `3:${known}`;
        const { value: type, mutable } = imp.type;
        const numeric = typeof value === 'number' && type !== 'i64' && type !== 'v128' && type !== 'externref' && type !== 'anyfunc';
        const big = typeof value === 'bigint' && type === 'i64';
        const ref = type === 'externref' || type === 'anyfunc';
        if (!numeric && !big && !ref) throw new LinkError(`${where} must be a WebAssembly.Global or a ${type}`);
        if (mutable) throw new LinkError(`${where} is mutable, so it must be a WebAssembly.Global`);
        return `3:${N.GlobalNew(type, false, to(type)(value))}`;
      }
    }
  };

  const exportObject = (instance, { name, kind }) => {
    const handle = N.ExportOf(instance, name, KIND[kind]);
    switch (kind) {
      case 'function': return exportedFunction(handle);
      case 'memory': return memories.object(Memory, handle);
      case 'table': return tables.object(Table, handle);
      default: return globals.object(Global, handle);
    }
  };

  // Reading the imports and instantiating are two steps, because
  // `WebAssembly.instantiate` with a module reads the import object at once
  // and instantiates on a later turn.
  const readImports = (module, importObject) => {
    const handle = moduleHandle(module);
    const { imports, exports } = parse(N.Describe(handle));
    if (importObject !== undefined && !isObject(importObject)) {
      throw new TypeError('the import object must be an object');
    }
    if (imports.length > 0 && importObject === undefined) {
      throw new TypeError('the module has imports but no import object was given');
    }
    const wrappers = [];
    const plan = imports.map((imp) => {
      const namespace = importObject[imp.module];
      if (!isObject(namespace)) throw new TypeError(`import module "${imp.module}" must be an object`);
      return linkImport(imp, namespace[imp.name], wrappers);
    });
    return { handle, exports, plan, wrappers };
  };
  const instantiateRead = (target, { handle, exports, plan, wrappers }) => {
    const instance = N.Instantiate(handle, plan.join(','), ...wrappers);
    if (typeof instance === 'string') {
      const message = instance.slice(5);
      throw instance.startsWith('trap:') ? new RuntimeError(message) : new LinkError(message);
    }
    const object = create(null);
    for (const exp of exports) object[exp.name] = exportObject(instance, exp);
    instanceExports.set(target, freeze(object));
    return target;
  };

  class Instance {
    constructor(module, importObject = undefined) {
      instantiateRead(this, readImports(module, importObject));
    }
    get exports() {
      const exports = instanceExports.get(this);
      if (exports === undefined) throw new TypeError('expected a WebAssembly.Instance');
      return exports;
    }
  }

  // WebIDL members are enumerable, unlike class members.
  const enumerable = (target, skip) => {
    for (const key of Object.getOwnPropertyNames(target)) {
      if (!skip.includes(key)) defineProperty(target, key, { enumerable: true });
    }
  };
  for (const C of [Module, Instance, Memory, Global, Table]) {
    enumerable(C.prototype, ['constructor']);
    enumerable(C, ['length', 'name', 'prototype']);
  }
  enumerable(WasmFunction.prototype, ['constructor']);
  const tag = (target, value) => defineProperty(target, Symbol.toStringTag, { value, configurable: true });
  tag(Module.prototype, 'WebAssembly.Module');
  tag(Instance.prototype, 'WebAssembly.Instance');
  tag(Memory.prototype, 'WebAssembly.Memory');
  tag(Global.prototype, 'WebAssembly.Global');
  tag(Table.prototype, 'WebAssembly.Table');
  tag(WasmFunction.prototype, 'WebAssembly.Function');

  const namespace = {
    validate(bytes) {
      return typeof N.Compile(bufferBytes(bytes)) !== 'string';
    },
    compile(bytes) {
      return new Promise((resolve) => resolve(new Module(bytes)));
    },
    instantiate(source, importObject = undefined) {
      // From bytes, the import object is read on a later turn, after
      // compiling, as the JS API's asynchronous instantiation does.
      if (modules.has(source)) {
        let read;
        try {
          read = readImports(source, importObject);
        } catch (e) {
          return Promise.reject(e);
        }
        return Promise.resolve().then(() => instantiateRead(create(Instance.prototype), read));
      }
      return namespace.compile(source).then((module) => ({ module, instance: new Instance(module, importObject) }));
    },
  };
  const classes = { Module, Instance, Memory, Global, Table, Function: WasmFunction, CompileError, LinkError, RuntimeError };
  for (const [name, value] of Object.entries(classes)) {
    defineProperty(namespace, name, { value, writable: true, configurable: true });
  }
  tag(namespace, 'WebAssembly');
  defineProperty(g, 'WebAssembly', { value: namespace, writable: true, configurable: true });
})();
