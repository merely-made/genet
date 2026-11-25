// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// The `WebAssembly` namespace over the natives in natives.rs (W1 of Genet's
// WebAssembly object plan). Values cross already converted: the closures here
// apply ToInt32, ToBigInt64 and ToNumber, and turn the natives' error strings
// into the namespace's error classes. Functions and memories cross; a memory's
// buffer is its linear memory, not a copy.
(() => {
  'use strict';
  const g = globalThis;
  const take = (name) => {
    const f = g[name];
    delete g[name];
    return f;
  };
  const compile = take('__genetWasmCompile');
  const describe = take('__genetWasmDescribe');
  const instantiate = take('__genetWasmInstantiate');
  const exportFunc = take('__genetWasmExportFunc');
  const signature = take('__genetWasmSignature');
  const call = take('__genetWasmCall');
  const memoryNew = take('__genetWasmMemoryNew');
  const memoryGrow = take('__genetWasmMemoryGrow');
  const memoryBuffer = take('__genetWasmMemoryBuffer');
  const exportMemory = take('__genetWasmExportMemory');
  const { create, defineProperty, freeze } = Object;
  const parse = JSON.parse;

  const errorClass = (name) => {
    const C = class extends Error {};
    defineProperty(C, 'name', { value: name, configurable: true });
    defineProperty(C.prototype, 'name', { value: name, writable: true, configurable: true });
    return C;
  };
  const CompileError = errorClass('CompileError');
  const LinkError = errorClass('LinkError');
  const RuntimeError = errorClass('RuntimeError');

  const toWasm = {
    i32: (v) => v | 0,
    i64: (v) => BigInt.asIntN(64, v),
    f32: (v) => +v,
    f64: (v) => +v,
  };
  const converter = (type) => toWasm[type] ?? (() => {
    throw new TypeError(`${type} values cannot cross yet`);
  });
  const isObject = (v) => (typeof v === 'object' && v !== null) || typeof v === 'function';

  // [EnforceRange] unsigned long, as the JS API reads page counts.
  const pages = (value, what) => {
    const n = Number(value);
    if (!Number.isFinite(n)) throw new TypeError(`${what} must be a finite number`);
    const t = Math.trunc(n);
    if (t < 0 || t > 0xffffffff) throw new TypeError(`${what} is out of range`);
    return t;
  };

  const modules = new WeakMap();
  const memoryHandles = new WeakMap();
  const memoryObjects = new Map();
  const instanceExports = new WeakMap();
  const exportedFunctions = new Map();

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

  const exportedFunction = (handle) => {
    let f = exportedFunctions.get(handle);
    if (f) return f;
    const sig = parse(signature(handle));
    const params = sig.params.map(converter);
    f = (...args) => {
      const result = call(handle, ...params.map((convert, i) => convert(args[i])));
      if (typeof result === 'string') throw new RuntimeError(result);
      return result;
    };
    defineProperty(f, 'name', { value: String(sig.index ?? handle), configurable: true });
    defineProperty(f, 'length', { value: sig.params.length, configurable: true });
    exportedFunctions.set(handle, f);
    return f;
  };

  const importWrapper = (fn, imp) => {
    if (imp.results.length > 1) {
      throw new LinkError(`${imp.module}.${imp.name}: multi-value imports are not supported yet`);
    }
    const result = imp.results.length ? converter(imp.results[0]) : null;
    return (...args) => {
      const value = fn(...args);
      return result ? result(value) : undefined;
    };
  };

  class Memory {
    constructor(descriptor) {
      if (!isObject(descriptor)) throw new TypeError('the memory descriptor must be an object');
      if (descriptor.shared) throw new TypeError('shared memory is not supported yet');
      if (descriptor.initial === undefined) throw new TypeError('initial is required');
      const initial = pages(descriptor.initial, 'initial');
      const maximum = descriptor.maximum === undefined ? undefined : pages(descriptor.maximum, 'maximum');
      if (initial > 65536 || (maximum !== undefined && (maximum > 65536 || maximum < initial))) {
        throw new RangeError('invalid memory limits');
      }
      const handle = memoryNew(initial, maximum);
      if (typeof handle === 'string') throw new RangeError(handle);
      memoryHandles.set(this, handle);
      memoryObjects.set(handle, this);
    }
    grow(delta) {
      const handle = memoryHandle(this);
      const old = memoryGrow(handle, pages(delta, 'delta'));
      if (typeof old === 'string') throw new RangeError(old);
      return old;
    }
    get buffer() {
      return memoryBuffer(memoryHandle(this));
    }
  }
  const memoryHandle = (memory) => {
    const handle = memoryHandles.get(memory);
    if (handle === undefined) throw new TypeError('expected a WebAssembly.Memory');
    return handle;
  };
  // One script object per memory, so an imported memory exported again is the
  // object that was imported.
  const memoryObject = (handle) => {
    let memory = memoryObjects.get(handle);
    if (memory) return memory;
    memory = create(Memory.prototype);
    memoryHandles.set(memory, handle);
    memoryObjects.set(handle, memory);
    return memory;
  };

  class Module {
    constructor(bytes) {
      const handle = compile(bufferBytes(bytes));
      if (typeof handle === 'string') throw new CompileError(handle);
      modules.set(this, handle);
    }
    static imports(module) {
      return parse(describe(moduleHandle(module))).imports
        .map(({ module, name, kind }) => ({ module, name, kind }));
    }
    static exports(module) {
      return parse(describe(moduleHandle(module))).exports.map(({ name, kind }) => ({ name, kind }));
    }
  }

  class Instance {
    constructor(module, importObject = undefined) {
      const handle = moduleHandle(module);
      const { imports, exports } = parse(describe(handle));
      if (importObject !== undefined && !isObject(importObject)) {
        throw new TypeError('the import object must be an object');
      }
      if (imports.length > 0 && importObject === undefined) {
        throw new TypeError('the module has imports but no import object was given');
      }
      const wrappers = imports.map((imp) => {
        const namespace = importObject[imp.module];
        if (!isObject(namespace)) throw new TypeError(`import module "${imp.module}" must be an object`);
        const value = namespace[imp.name];
        if (imp.kind === 'memory') {
          const handle = memoryHandles.get(value);
          if (handle === undefined) throw new LinkError(`${imp.module}.${imp.name} must be a WebAssembly.Memory`);
          return handle;
        }
        if (imp.kind !== 'function') {
          throw new LinkError(`${imp.module}.${imp.name}: ${imp.kind} imports are not supported yet`);
        }
        if (typeof value !== 'function') {
          throw new LinkError(`${imp.module}.${imp.name} must be a function`);
        }
        return importWrapper(value, imp);
      });
      const instance = instantiate(handle, ...wrappers);
      if (typeof instance === 'string') {
        const message = instance.slice(5);
        throw instance.startsWith('trap:') ? new RuntimeError(message) : new LinkError(message);
      }
      const object = create(null);
      for (const { name, kind } of exports) {
        if (kind === 'function') object[name] = exportedFunction(exportFunc(instance, name));
        if (kind === 'memory') object[name] = memoryObject(exportMemory(instance, name));
      }
      instanceExports.set(this, freeze(object));
    }
    get exports() {
      const exports = instanceExports.get(this);
      if (exports === undefined) throw new TypeError('expected a WebAssembly.Instance');
      return exports;
    }
  }

  const tag = (target, value) =>
    defineProperty(target, Symbol.toStringTag, { value, configurable: true });
  tag(Module.prototype, 'WebAssembly.Module');
  tag(Instance.prototype, 'WebAssembly.Instance');
  tag(Memory.prototype, 'WebAssembly.Memory');

  const namespace = {
    validate(bytes) {
      return typeof compile(bufferBytes(bytes)) !== 'string';
    },
    compile(bytes) {
      return new Promise((resolve) => resolve(new Module(bytes)));
    },
    instantiate(source, importObject = undefined) {
      return new Promise((resolve) => {
        if (modules.has(source)) {
          resolve(new Instance(source, importObject));
          return;
        }
        const module = new Module(source);
        resolve({ module, instance: new Instance(module, importObject) });
      });
    },
  };
  for (const [name, value] of Object.entries({ Module, Instance, Memory, CompileError, LinkError, RuntimeError })) {
    defineProperty(namespace, name, { value, writable: true, configurable: true });
  }
  tag(namespace, 'WebAssembly');
  defineProperty(g, 'WebAssembly', { value: namespace, writable: true, configurable: true });
})();
