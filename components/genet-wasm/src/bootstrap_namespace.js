// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// The bootstrap ends: modules, instances and the namespace.
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
      own(MODULE, handle, this);
    }
    static imports(module) {
      return parse(N.Describe(moduleHandle(module))).imports
        .map(({ module, name, kind, type }) => ({ module, name, kind, type: reflect(type) }));
    }
    static exports(module) {
      return parse(N.Describe(moduleHandle(module))).exports
        .map(({ name, kind, type }) => ({ name, kind, type: reflect(type) }));
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

  // One import as the natives take it: a plan entry. A script function goes in
  // the group's keeper. What is merged into the group, or must outlive the
  // wait for instantiation, is kept in `held`.
  const linkImport = (imp, value, group, held, temps) => {
    const where = `${imp.module}.${imp.name}`;
    const shared = (kind, handle) => {
      join(group, groups.get(value));
      held.push(value);
      return `${kind}:${handle}`;
    };
    switch (imp.kind) {
      case 'function': {
        if (typeof value !== 'function') throw new LinkError(`${where} must be a function`);
        const info = funcInfo.get(value);
        if (info !== undefined) {
          join(group, info.group);
          held.push(value);
          return `${FUNC}:${info.handle}`;
        }
        return `${PLAN_SCRIPT}:${holdFn(group, scriptWrapper(value, imp.type, group))}`;
      }
      case 'memory': {
        const handle = memories.handles.get(value);
        if (handle === undefined) throw new LinkError(`${where} must be a WebAssembly.Memory`);
        return shared(MEMORY, handle);
      }
      case 'table': {
        const handle = tables.handles.get(value);
        if (handle === undefined) throw new LinkError(`${where} must be a WebAssembly.Table`);
        return shared(TABLE, handle);
      }
      default: {
        const known = globals.handles.get(value);
        if (known !== undefined) return shared(GLOBAL, known);
        const { value: type, mutable } = imp.type;
        const numeric = typeof value === 'number' && type !== 'i64' && type !== 'v128' && type !== 'externref' && type !== 'anyfunc';
        const big = typeof value === 'bigint' && type === 'i64';
        const ref = type === 'externref' || type === 'anyfunc';
        if (!numeric && !big && !ref) throw new LinkError(`${where} must be a WebAssembly.Global or a ${type}`);
        if (mutable) throw new LinkError(`${where} is mutable, so it must be a WebAssembly.Global`);
        const handle = N.GlobalNew(type, false, to(type)(value, group));
        N.Retain(GLOBAL, handle);
        temps.push(handle);
        return `${GLOBAL}:${handle}`;
      }
    }
  };

  const exportObject = (instance, { name, kind }, group) => {
    const handle = N.ExportOf(instance, name, KIND[kind]);
    switch (kind) {
      case 'function': return exportedFunction(handle, group);
      case 'memory': return memories.object(Memory, handle, group);
      case 'table': return tables.object(Table, handle, group);
      default: return globals.object(Global, handle, group);
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
    const group = new Keeper();
    const held = [module];
    const temps = [];
    try {
      const plan = imports.map((imp) => {
        const namespace = importObject[imp.module];
        if (!isObject(namespace)) throw new TypeError(`import module "${imp.module}" must be an object`);
        return linkImport(imp, namespace[imp.name], group, held, temps);
      });
      return { handle, exports, plan, group, held, temps };
    } catch (e) {
      for (const temp of temps) release(GLOBAL, temp);
      throw e;
    }
  };
  // The instance is needed only to read its exports, so it is released after.
  const instantiateRead = (target, { handle, exports, plan, group, held, temps }) => {
    try {
      const instance = N.Instantiate(handle, plan.join(','), group.call);
      if (typeof instance === 'string') {
        const message = instance.slice(5);
        throw instance.startsWith('trap:') ? new RuntimeError(message) : new LinkError(message);
      }
      const object = create(null);
      try {
        for (const exp of exports) object[exp.name] = exportObject(instance, exp, group);
      } finally {
        release(INSTANCE, instance);
      }
      instanceExports.set(target, freeze(object));
      return target;
    } finally {
      sync();
      for (const temp of temps) release(GLOBAL, temp);
      held.length = 0;
    }
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
      const handle = N.Compile(bufferBytes(bytes));
      if (typeof handle === 'string') return false;
      N.Retain(MODULE, handle);
      release(MODULE, handle);
      return true;
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
