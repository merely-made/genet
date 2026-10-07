// Private per-realm operations. The agent root retains only ephemeron maps;
// algorithms and instances are reachable through their branded live objects.
var Core = (function () {
  'use strict';
  var O = Object, P = Promise, W = WeakMap, R = Reflect;
  var AB = ArrayBuffer, U8 = Uint8Array, DV = DataView;
  var TE = TypeError, RE = RangeError, S = Symbol, N = Number, Str = String;
  var apply = R.apply, create = O.create, getDescriptor = O.getOwnPropertyDescriptor;
  var defineProperty = O.defineProperty, ownKeys = R.ownKeys, setPrototypeOf = O.setPrototypeOf;
  var objectPrototype = O.prototype;
  var wmHas = W.prototype.has, wmGet = W.prototype.get, wmSet = W.prototype.set;
  var wmDelete = W.prototype.delete;
  var promiseResolve = P.resolve, promiseReject = P.reject, promiseThen = P.prototype.then;
  var getPrototypeOf = O.getPrototypeOf;
  var asyncIteratorPrototype = getPrototypeOf(getPrototypeOf(getPrototypeOf((async function* () {})())));
  var typedPrototype = getPrototypeOf(U8.prototype);
  var typedBuffer = getDescriptor(typedPrototype, 'buffer').get;
  var typedOffset = getDescriptor(typedPrototype, 'byteOffset').get;
  var typedBytes = getDescriptor(typedPrototype, 'byteLength').get;
  var typedLength = getDescriptor(typedPrototype, 'length').get;
  var typedName = getDescriptor(typedPrototype, S.toStringTag).get;
  var typedSet = typedPrototype.set;
  var dataBuffer = getDescriptor(DV.prototype, 'buffer').get;
  var dataOffset = getDescriptor(DV.prototype, 'byteOffset').get;
  var dataBytes = getDescriptor(DV.prototype, 'byteLength').get;
  var abBytes = getDescriptor(AB.prototype, 'byteLength').get;
  var abTransfer = AB.prototype.transferToFixedLength;
  var sabBytes = typeof SharedArrayBuffer === 'function' ?
    getDescriptor(SharedArrayBuffer.prototype, 'byteLength').get : undefined;
  var views = create(null), sizes = create(null);
  var names = ['Int8Array', 'Uint8Array', 'Uint8ClampedArray', 'Int16Array', 'Uint16Array',
    'Int32Array', 'Uint32Array', 'Float16Array', 'Float32Array', 'Float64Array',
    'BigInt64Array', 'BigUint64Array'];
  for (var i = 0; i < names.length; ++i) {
    var ctor = globalThis[names[i]];
    if (typeof ctor === 'function') {
      views[names[i]] = ctor;
      sizes[names[i]] = ctor.BYTES_PER_ELEMENT;
    }
  }
  views.DataView = DV; sizes.DataView = 1;
  function call(fn, receiver, args) { return apply(fn, receiver, args); }
  function record() { return create(null); }
  function define(object, key, descriptor) {
    var d = record(), fields = ['value', 'writable', 'get', 'set', 'enumerable', 'configurable'];
    for (var j = 0; j < fields.length; ++j) {
      var field = getDescriptor(descriptor, fields[j]);
      if (field !== undefined) d[fields[j]] = field.value;
    }
    return defineProperty(object, key, d);
  }
  var agent = globalThis.__agentTimers;
  var existing = getDescriptor(agent, '__streams');
  var maps = existing === undefined ? record() : existing.value;
  if (existing === undefined) define(agent, '__streams', {
    value: maps, writable: false, enumerable: false, configurable: false
  });
  function sharedMap(name) {
    if (getDescriptor(maps, name) === undefined) maps[name] = new W();
    return maps[name];
  }
  function has(map, key) { return call(wmHas, map, [key]); }
  function get(map, key) { return call(wmGet, map, [key]); }
  function set(map, key, value) { call(wmSet, map, [key, value]); return value; }
  function del(map, key) { return call(wmDelete, map, [key]); }
  var internalPromises = sharedMap('internalPromises');
  function markPromise(promise) { set(internalPromises, promise, true); return promise; }
  function deferred() {
    var d = record();
    d.promise = markPromise(new P(function (resolve, reject) { d.resolve = resolve; d.reject = reject; }));
    return d;
  }
  function resolve(value) {
    if (has(internalPromises, value)) return value;
    return markPromise(call(promiseResolve, P, [value]));
  }
  function reject(reason) { return markPromise(call(promiseReject, P, [reason])); }
  function react(promise, fulfilled, rejected) { return markPromise(call(promiseThen, promise, [fulfilled, rejected])); }
  function handled(promise) { react(promise, undefined, function () {}); }
  function queue() {
    var q = record(); q.items = record(); q.head = 0; q.tail = 0; q.totalSize = 0; return q;
  }
  function push(q, value, size) {
    if (size === undefined) size = 0;
    size = +size;
    if (!(size >= 0) || size === Infinity) throw new RE('Invalid queue size');
    var entry = record(); entry.value = value; entry.size = size;
    q.items[q.tail++] = entry; q.totalSize += size;
  }
  function shift(q) {
    if (q.head === q.tail) return undefined;
    var entry = q.items[q.head]; delete q.items[q.head++];
    q.totalSize -= entry.size;
    if (q.totalSize < 0) q.totalSize = 0;
    if (q.head === q.tail) { q.head = 0; q.tail = 0; q.totalSize = 0; }
    return entry.value;
  }
  function peek(q) { return q.head === q.tail ? undefined : q.items[q.head].value; }
  function length(q) { return q.tail - q.head; }
  function clear(q) { q.items = record(); q.head = 0; q.tail = 0; q.totalSize = 0; }
  // Web IDL converts dictionary members before the constructor's prose.
  // HWM range validation happens later, after source/sink conversion.
  function convertStrategy(dictionary) {
    var d = record();
    if (dictionary === undefined || dictionary === null) return d;
    if (typeof dictionary !== 'object' && typeof dictionary !== 'function')
      throw new TE('Queuing strategy must be an object');
    var hwm = dictionary.highWaterMark;
    if (hwm !== undefined) d.highWaterMark = +hwm;
    var size = dictionary.size;
    if (size !== undefined) {
      if (typeof size !== 'function') throw new TE('size must be callable');
      d.size = size;
    }
    return d;
  }
  function extractHWM(converted, defaultHWM) {
    var hwm = converted.highWaterMark;
    if (hwm === undefined) return defaultHWM;
    if (!(hwm >= 0)) throw new RE('Invalid high water mark');
    return hwm;
  }
  function extractSize(converted) {
    var callback = converted.size;
    if (callback === undefined) return function () { return 1; };
    return function (chunk) { return +call(callback, undefined, [chunk]); };
  }
  function strategy(dictionary, defaultHWM) {
    var converted = convertStrategy(dictionary), result = record();
    result.sizeAlgorithm = extractSize(converted);
    result.highWaterMark = extractHWM(converted, defaultHWM);
    return result;
  }
  function bufferByteLength(buffer) {
    try { return call(abBytes, buffer, []); }
    catch (e) { if (sabBytes === undefined) throw e; return call(sabBytes, buffer, []); }
  }
  function isDetached(buffer) {
    try { call(abBytes, buffer, []); }
    catch (_) { if (sabBytes !== undefined) { call(sabBytes, buffer, []); return false; } throw _; }
    try { new U8(buffer, 0, 0); return false; } catch (_) { return true; }
  }
  function viewInfo(value) {
    var info = record(), kind;
    try {
      info.buffer = call(typedBuffer, value, []);
      kind = call(typedName, value, []);
      info.byteOffset = call(typedOffset, value, []);
      info.byteLength = call(typedBytes, value, []);
      info.length = call(typedLength, value, []);
    } catch (_) {
      info.buffer = call(dataBuffer, value, []); kind = 'DataView';
      info.byteOffset = call(dataOffset, value, []);
      info.byteLength = call(dataBytes, value, []); info.length = info.byteLength;
    }
    info.kind = kind; info.elementSize = sizes[kind];
    if (info.elementSize === undefined) throw new TE('Expected an ArrayBuffer view');
    info.bufferByteLength = bufferByteLength(info.buffer);
    info.isDetached = isDetached(info.buffer);
    return info;
  }
  function transfer(buffer) { return call(abTransfer, buffer, []); }
  function view(kind, buffer, offset, byteLength) {
    var Constructor = views[kind];
    if (Constructor === undefined) throw new TE('Unknown intrinsic view');
    return new Constructor(buffer, offset, kind === 'DataView' ? byteLength : byteLength / sizes[kind]);
  }
  function copyBytes(value) {
    var info = viewInfo(value);
    if (info.isDetached) throw new TE('Detached buffer');
    var result = new U8(info.byteLength);
    call(typedSet, result, [new U8(info.buffer, info.byteOffset, info.byteLength), 0]);
    return result;
  }
  var core = record();
  core.record = record; core.call = call; core.create = create; core.define = define;
  core.descriptor = getDescriptor; core.ownKeys = ownKeys; core.setPrototypeOf = setPrototypeOf;
  core.objectPrototype = objectPrototype;
  core.asyncIteratorPrototype = asyncIteratorPrototype;
  core.sharedMap = sharedMap; core.has = has; core.get = get; core.set = set; core.delete = del;
  core.deferred = deferred; core.resolve = resolve; core.reject = reject;
  core.react = react; core.handled = handled;
  core.isInternalPromise = function (promise) { return has(internalPromises, promise); };
  core.queue = queue; core.push = push; core.shift = shift; core.peek = peek;
  core.length = length; core.clear = clear;
  core.convertStrategy = convertStrategy; core.extractHWM = extractHWM;
  core.extractSize = extractSize; core.strategy = strategy;
  core.TypeError = TE; core.RangeError = RE; core.Number = N; core.String = Str;
  core.Symbol = S; core.Promise = P; core.ArrayBuffer = AB; core.Uint8Array = U8;
  core.viewInfo = viewInfo; core.bufferByteLength = bufferByteLength; core.isDetached = isDetached;
  core.isArrayBuffer = function (value) {
    try { call(abBytes, value, []); return true; } catch (_) { return false; }
  };
  core.transfer = transfer; core.view = view; core.copyBytes = copyBytes;
  core.signal = record();
  core.clone = undefined;
  return core;
})();
