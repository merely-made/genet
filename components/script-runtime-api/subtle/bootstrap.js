// SubtleCrypto and CryptoKey over the native `__subtle_*` sinks (subtle.rs).
//
// This file holds Web Crypto's WebIDL conversions, algorithm normalisation,
// usage and format rules, JWK handling, error names and promises. Key material
// and every digest, MAC and key derivation stay native; a key here is an opaque id. The native
// registry rechecks extractability and usages, so this side's brand data is
// bookkeeping, not the authority.
//
// Each operation runs synchronously inside the promise executor. A conversion
// failure therefore rejects rather than throws, as WebIDL requires of
// promise-returning operations.
(function() {
  var Obj = Object, defineProperty = Obj.defineProperty, create = Obj.create;
  var getDesc = Obj.getOwnPropertyDescriptor, freeze = Obj.freeze;
  var apply = Reflect.apply, PromiseCtor = Promise, TypeErrorCtor = TypeError;
  var WeakMapCtor = WeakMap, weakGet = WeakMap.prototype.get, weakSet = WeakMap.prototype.set;
  var weakHas = WeakMap.prototype.has;
  var U8 = Uint8Array, ArrayBufferCtor = ArrayBuffer, StringCtor = String;
  var fromCharCode = String.fromCharCode, charCodeAt = String.prototype.charCodeAt;
  var NumberCtor = Number, mathFloor = Math.floor, mathAbs = Math.abs, isFiniteFn = isFinite;
  var arrayPush = Array.prototype.push, arrayJoin = Array.prototype.join;
  var SymbolIterator = typeof Symbol === 'function' ? Symbol.iterator : undefined;
  var SymbolToStringTag = typeof Symbol === 'function' ? Symbol.toStringTag : undefined;
  var DOMExceptionCtor = DOMException, btoaFn = btoa, atobFn = atob;

  var typedArrayProto = Obj.getPrototypeOf(U8.prototype);
  var typedTag = getDesc(typedArrayProto, SymbolToStringTag).get;
  var typedBuffer = getDesc(typedArrayProto, 'buffer').get;
  var typedOffset = getDesc(typedArrayProto, 'byteOffset').get;
  var typedLength = getDesc(typedArrayProto, 'byteLength').get;
  var dataBuffer = getDesc(DataView.prototype, 'buffer').get;
  var dataOffset = getDesc(DataView.prototype, 'byteOffset').get;
  var dataLength = getDesc(DataView.prototype, 'byteLength').get;
  var bufferLength = getDesc(ArrayBufferCtor.prototype, 'byteLength').get;
  var sharedLength = typeof SharedArrayBuffer === 'function'
    ? getDesc(SharedArrayBuffer.prototype, 'byteLength').get : null;

  var native = {
    digest: globalThis.__subtle_digest,
    hmacStore: globalThis.__subtle_hmac_store,
    hmacGenerate: globalThis.__subtle_hmac_generate,
    exportKey: globalThis.__subtle_export,
    sign: globalThis.__subtle_sign,
    verify: globalThis.__subtle_verify,
    release: globalThis.__subtle_release,
    secure: globalThis.__subtle_secure,
    secretStore: globalThis.__subtle_secret_store,
    derive: globalThis.__subtle_derive,
    deriveKey: globalThis.__subtle_derive_key
  };
  try {
    delete globalThis.__subtle_digest; delete globalThis.__subtle_hmac_store;
    delete globalThis.__subtle_hmac_generate; delete globalThis.__subtle_export;
    delete globalThis.__subtle_sign; delete globalThis.__subtle_verify;
    delete globalThis.__subtle_release; delete globalThis.__subtle_secure;
    delete globalThis.__subtle_secret_store; delete globalThis.__subtle_derive;
    delete globalThis.__subtle_derive_key;
  } catch (deleteError) {}
  // `call(fn, receiver, ...args)` for captured intrinsics; `invoke(fn, ...args)`
  // for the receiver-less native sinks.
  function call(fn, receiver) {
    var args = [];
    for (var i = 2; i < arguments.length; i++) args[i - 2] = arguments[i];
    return apply(fn, receiver, args);
  }
  function invoke(fn) {
    var args = [];
    for (var i = 1; i < arguments.length; i++) args[i - 1] = arguments[i];
    return apply(fn, undefined, args);
  }

  // Brands live on the agent so a key minted in one same-agent realm works in
  // another; each brand carries the sinks of the realm that holds its bytes.
  var shared = globalThis.__agentTimers, brands;
  if (shared && (typeof shared === 'object' || typeof shared === 'function')) {
    var sharedDesc = getDesc(shared, '__subtleState');
    if (!sharedDesc) {
      var fresh = { keys: new WeakMapCtor(), subtles: new WeakMapCtor() };
      defineProperty(shared, '__subtleState', { value: fresh });
      sharedDesc = { value: fresh };
    }
    brands = sharedDesc.value;
  } else {
    brands = { keys: new WeakMapCtor(), subtles: new WeakMapCtor() };
  }
  var FinReg = globalThis.FinalizationRegistry;
  var finalizer = typeof FinReg === 'function'
    ? new FinReg(function(held) { invoke(held.release, held.id); }) : null;
  var finRegister = finalizer ? FinReg.prototype.register : null;

  function typeError(message) { return new TypeErrorCtor(message); }
  function dom(name, message) { return new DOMExceptionCtor(message, name); }

  // ---- WebIDL conversions ------------------------------------------------

  function toDOMString(value) {
    if (typeof value === 'symbol') throw typeError('Cannot convert a Symbol value to a string');
    return StringCtor(value);
  }
  function isObject(value) {
    return value !== null && (typeof value === 'object' || typeof value === 'function');
  }
  // [EnforceRange] unsigned long.
  function toEnforcedULong(value, what) {
    var x = NumberCtor(value);
    if (x !== x || !isFiniteFn(x)) throw typeError(what + ' is not a finite number');
    x = x < 0 ? -mathFloor(mathAbs(x)) : mathFloor(x);
    if (x < 0 || x > 4294967295) throw typeError(what + ' is out of range');
    return x;
  }
  // [EnforceRange] unsigned short.
  function toEnforcedUShort(value, what) {
    var x = toEnforcedULong(value, what);
    if (x > 65535) throw typeError(what + ' is out of range');
    return x;
  }
  function toSequence(value, what, convert) {
    if (!isObject(value)) throw typeError(what + ' is not a sequence');
    var method = value[SymbolIterator];
    if (typeof method !== 'function') throw typeError(what + ' is not iterable');
    var iterator = apply(method, value, []), out = [];
    for (;;) {
      var step = iterator.next();
      if (!isObject(step)) throw typeError('Iterator result is not an object');
      if (step.done) return out;
      apply(arrayPush, out, [convert(step.value)]);
    }
  }
  function toEnum(value, allowed, what) {
    var s = toDOMString(value);
    for (var i = 0; i < allowed.length; i++) if (allowed[i] === s) return s;
    throw typeError("'" + s + "' is not a valid " + what);
  }

  var KEY_USAGES = ['encrypt', 'decrypt', 'sign', 'verify', 'deriveKey', 'deriveBits',
    'wrapKey', 'unwrapKey', 'encapsulateKey', 'encapsulateBits', 'decapsulateKey',
    'decapsulateBits'];
  var KEY_FORMATS = ['raw-public', 'raw-private', 'raw-seed', 'raw-secret', 'raw',
    'spki', 'pkcs8', 'jwk'];
  function toUsages(value) {
    return toSequence(value, 'keyUsages', function(v) { return toEnum(v, KEY_USAGES, 'KeyUsage'); });
  }
  function toFormat(value) { return toEnum(value, KEY_FORMATS, 'KeyFormat'); }
  function toAlgorithmIdentifier(value) { return isObject(value) ? value : toDOMString(value); }

  // A BufferSource, classified now (the WebIDL conversion) and copied later
  // ("get a copy of the bytes held by the buffer source"), after algorithm
  // normalisation has run any author getters. Captured intrinsic getters keep
  // author overrides out; a detached buffer copies as empty.
  function toBufferSource(value, what) {
    if (isObject(value)) {
      if (call(typedTag, value) !== undefined) {
        if (sharedLength && isShared(call(typedBuffer, value))) throw typeError(what + ' is backed by a SharedArrayBuffer');
        return { kind: 'typed', value: value };
      }
      try { var buffer = call(dataBuffer, value); } catch (notDataView) { buffer = null; }
      if (buffer !== null) {
        if (sharedLength && isShared(buffer)) throw typeError(what + ' is backed by a SharedArrayBuffer');
        return { kind: 'data', value: value };
      }
      try { call(bufferLength, value); return { kind: 'buffer', value: value }; } catch (notBuffer) {}
    }
    throw typeError(what + ' is not a BufferSource');
  }
  function isShared(buffer) {
    try { call(sharedLength, buffer); return true; } catch (notShared) { return false; }
  }
  function copyBytes(source) {
    if (source.kind === 'binary') return source.value;
    var buffer, offset, length;
    try {
      if (source.kind === 'buffer') {
        buffer = source.value; offset = 0; length = call(bufferLength, buffer);
      } else if (source.kind === 'typed') {
        buffer = call(typedBuffer, source.value);
        offset = call(typedOffset, source.value);
        length = call(typedLength, source.value);
      } else {
        buffer = call(dataBuffer, source.value);
        offset = call(dataOffset, source.value);
        length = call(dataLength, source.value);
      }
    } catch (detached) { return ''; }
    if (!length) return '';
    var bytes = new U8(buffer, offset, length), out = '', chunk = [], cap = 8192;
    for (var i = 0; i < length; i++) {
      chunk[i % cap] = bytes[i];
      if (i % cap === cap - 1 || i === length - 1) {
        chunk.length = i % cap + 1;
        out += apply(fromCharCode, undefined, chunk);
        chunk = [];
      }
    }
    return out;
  }
  function toArrayBuffer(binary) {
    var out = new U8(binary.length);
    for (var i = 0; i < binary.length; i++) out[i] = call(charCodeAt, binary, i);
    return call(typedBuffer, out);
  }

  // ---- Algorithm normalisation --------------------------------------------

  function asciiUpper(s) {
    var out = '';
    for (var i = 0; i < s.length; i++) {
      var c = call(charCodeAt, s, i);
      out += c >= 97 && c <= 122 ? fromCharCode(c - 32) : s[i];
    }
    return out;
  }

  // Dictionary converters by IDL type. Members are read in lexicographic order
  // after the inherited `name`, as WebIDL dictionary conversion does.
  var HASHED = function(alg, out) {
    var hash = alg.hash;
    if (hash === undefined) throw typeError("Algorithm: 'hash' is required");
    out.hash = normalize(toAlgorithmIdentifier(hash), 'digest');
  };
  var DICTIONARIES = {
    Algorithm: function() {},
    HmacImportParams: function(alg, out) {
      HASHED(alg, out);
      var length = alg.length;
      if (length !== undefined) out.length = toEnforcedULong(length, "Algorithm: 'length'");
    }
  };
  DICTIONARIES.HmacKeyGenParams = DICTIONARIES.HmacImportParams;
  // Members are converted in lexicographic order (hash, info/iterations, salt),
  // so a bad hash name reports before a missing salt does.
  DICTIONARIES.HkdfParams = function(alg, out) {
    HASHED(alg, out);
    if (alg.info === undefined) throw typeError("Algorithm: 'info' is required");
    out.info = toBufferSource(alg.info, "Algorithm: 'info'");
    if (alg.salt === undefined) throw typeError("Algorithm: 'salt' is required");
    out.salt = toBufferSource(alg.salt, "Algorithm: 'salt'");
  };
  DICTIONARIES.Pbkdf2Params = function(alg, out) {
    HASHED(alg, out);
    if (alg.iterations === undefined) throw typeError("Algorithm: 'iterations' is required");
    out.iterations = toEnforcedULong(alg.iterations, "Algorithm: 'iterations'");
    if (alg.salt === undefined) throw typeError("Algorithm: 'salt' is required");
    out.salt = toBufferSource(alg.salt, "Algorithm: 'salt'");
  };
  DICTIONARIES.AesDerivedKeyParams = function(alg, out) {
    if (alg.length === undefined) throw typeError("Algorithm: 'length' is required");
    out.length = toEnforcedUShort(alg.length, "Algorithm: 'length'");
  };

  // The registered algorithms, per operation, with each one's parameter type.
  // A name missing here normalises to NotSupportedError; that is how every
  // algorithm not yet implemented reports itself.
  var SUPPORTED = {
    digest: { 'SHA-1': 'Algorithm', 'SHA-256': 'Algorithm', 'SHA-384': 'Algorithm', 'SHA-512': 'Algorithm' },
    sign: { 'HMAC': 'Algorithm' },
    verify: { 'HMAC': 'Algorithm' },
    generateKey: { 'HMAC': 'HmacKeyGenParams' },
    importKey: {
      'HMAC': 'HmacImportParams', 'HKDF': 'Algorithm', 'PBKDF2': 'Algorithm',
      'AES-CBC': 'Algorithm', 'AES-CTR': 'Algorithm', 'AES-GCM': 'Algorithm', 'AES-KW': 'Algorithm'
    },
    deriveBits: { 'HKDF': 'HkdfParams', 'PBKDF2': 'Pbkdf2Params' },
    'get key length': {
      'HMAC': 'HmacImportParams', 'HKDF': 'Algorithm', 'PBKDF2': 'Algorithm',
      'AES-CBC': 'AesDerivedKeyParams', 'AES-CTR': 'AesDerivedKeyParams',
      'AES-GCM': 'AesDerivedKeyParams', 'AES-KW': 'AesDerivedKeyParams'
    },
    encrypt: {}, decrypt: {}, wrapKey: {}, unwrapKey: {}
  };

  function normalize(alg, op) {
    if (typeof alg === 'string') return normalize({ name: alg }, op);
    if (!isObject(alg)) throw typeError('Algorithm is not an object');
    var nameValue = alg.name;
    if (nameValue === undefined) throw typeError("Algorithm: 'name' is required");
    var wanted = asciiUpper(toDOMString(nameValue)), table = SUPPORTED[op], registered = null;
    for (var key in table) {
      if (asciiUpper(key) === wanted) { registered = key; break; }
    }
    if (registered === null) throw dom('NotSupportedError', 'Unrecognized or unsupported algorithm for ' + op);
    var out = { name: registered };
    var dictionary = table[registered];
    if (dictionary !== 'Algorithm') {
      // The second conversion reads `name` again, as the spec's does.
      toDOMString(alg.name);
      DICTIONARIES[dictionary](alg, out);
    }
    return out;
  }

  // ---- CryptoKey -----------------------------------------------------------

  function CryptoKey() { throw typeError('Illegal constructor'); }
  function keyBrand(value) {
    var brand = isObject(value) ? call(weakGet, brands.keys, value) : undefined;
    if (!brand) throw typeError('Illegal invocation');
    return brand;
  }
  function getter(name, read) {
    defineProperty(CryptoKey.prototype, name, {
      get: function() { return read(keyBrand(this)); },
      enumerable: true, configurable: true
    });
  }
  getter('type', function(b) { return b.type; });
  getter('extractable', function(b) { return b.extractable; });
  getter('algorithm', function(b) { return b.algorithm; });
  getter('usages', function(b) { return b.usagesObject; });
  if (SymbolToStringTag) defineProperty(CryptoKey.prototype, SymbolToStringTag, { value: 'CryptoKey', configurable: true });

  var USAGE_ORDER = KEY_USAGES;
  // The usage intersection, in the KeyUsage enum's order, without repeats.
  function intersect(usages, allowed) {
    var out = [];
    for (var i = 0; i < USAGE_ORDER.length; i++) {
      var u = USAGE_ORDER[i], wanted = false, ok = false;
      for (var j = 0; j < usages.length; j++) if (usages[j] === u) wanted = true;
      for (var k = 0; k < allowed.length; k++) if (allowed[k] === u) ok = true;
      if (wanted && ok) apply(arrayPush, out, [u]);
    }
    return out;
  }
  function onlyAllowed(usages, allowed) {
    for (var i = 0; i < usages.length; i++) {
      var ok = false;
      for (var j = 0; j < allowed.length; j++) if (allowed[j] === usages[i]) ok = true;
      if (!ok) return false;
    }
    return true;
  }
  // `owner` is the sink set of the realm whose registry holds the bytes; it is
  // this realm's own except for a key derived from another realm's base key.
  function makeKey(id, type, extractable, algorithm, usages, owner) {
    owner = owner || native;
    var key = create(CryptoKey.prototype);
    var usagesObject = [];
    for (var i = 0; i < usages.length; i++) usagesObject[i] = usages[i];
    call(weakSet, brands.keys, key, {
      id: id, type: type, extractable: extractable, algorithm: algorithm,
      usages: usages, usagesObject: usagesObject, native: owner
    });
    if (finalizer) call(finRegister, finalizer, key, { id: id, release: owner.release });
    return key;
  }
  function hasUsage(brand, usage) {
    for (var i = 0; i < brand.usages.length; i++) if (brand.usages[i] === usage) return true;
    return false;
  }

  // ---- HMAC ----------------------------------------------------------------

  var HMAC_USAGES = ['sign', 'verify'];
  function hashBlockBits(name) { return name === 'SHA-384' || name === 'SHA-512' ? 1024 : 512; }
  function hmacJwkAlg(name) {
    return { 'SHA-1': 'HS1', 'SHA-256': 'HS256', 'SHA-384': 'HS384', 'SHA-512': 'HS512' }[name];
  }
  function hmacAlgorithm(hashName, length) {
    return { name: 'HMAC', hash: { name: hashName }, length: length };
  }
  function storedKey(id) {
    if (id === null) throw dom('OperationError', 'The key could not be stored');
    if (id === '!R') throw dom('NotSupportedError', 'No random source is installed on this host');
    return id;
  }
  function usageList(usages) { return apply(arrayJoin, usages, [',']); }

  function hmacGenerate(alg, extractable, usages) {
    if (!onlyAllowed(usages, HMAC_USAGES)) throw dom('SyntaxError', 'HMAC keys only sign and verify');
    var length;
    if (alg.length === undefined) length = hashBlockBits(alg.hash.name);
    else if (alg.length !== 0) length = alg.length;
    else throw dom('OperationError', 'HMAC key length must not be zero');
    usages = intersect(usages, HMAC_USAGES);
    var id = storedKey(invoke(native.hmacGenerate, alg.hash.name, extractable ? '1' : '0',
      usageList(usages), mathFloor((length + 7) / 8)));
    return makeKey(id, 'secret', extractable, hmacAlgorithm(alg.hash.name, length), usages);
  }

  function hmacImport(format, keyData, alg, extractable, usages) {
    if (!onlyAllowed(usages, HMAC_USAGES)) throw dom('SyntaxError', 'HMAC keys only sign and verify');
    var data, hashName = alg.hash.name;
    if (format === 'raw' || format === 'raw-secret') {
      data = copyBytes(keyData);
    } else if (format === 'jwk') {
      var jwk = keyData;
      if (jwk.kty !== 'oct') throw dom('DataError', "JWK 'kty' must be 'oct'");
      if (jwk.k === undefined) throw dom('DataError', "JWK 'k' is missing");
      data = base64UrlDecode(jwk.k);
      if (jwk.alg !== undefined && jwk.alg !== hmacJwkAlg(hashName))
        throw dom('DataError', "JWK 'alg' does not match the hash");
      checkJwkCommon(jwk, usages, extractable, 'sig');
    } else {
      throw dom('NotSupportedError', 'HMAC keys import from raw or jwk');
    }
    var length = data.length * 8;
    if (length === 0) throw dom('DataError', 'HMAC key data is empty');
    if (alg.length !== undefined) {
      if (alg.length > length) throw dom('DataError', 'HMAC length exceeds the key data');
      if (alg.length <= length - 8) throw dom('DataError', 'HMAC length is too short for the key data');
      length = alg.length;
    }
    usages = intersect(usages, HMAC_USAGES);
    var id = storedKey(invoke(native.hmacStore, hashName, extractable ? '1' : '0', usageList(usages), data));
    return makeKey(id, 'secret', extractable, hmacAlgorithm(hashName, length), usages);
  }

  function hmacExport(format, brand) {
    var data = invoke(brand.native.exportKey, brand.id);
    if (data === null) throw dom('OperationError', 'The key material is unavailable');
    if (format === 'raw' || format === 'raw-secret') return toArrayBuffer(data);
    if (format === 'jwk') {
      var usages = [];
      for (var i = 0; i < brand.usages.length; i++) usages[i] = brand.usages[i];
      return {
        kty: 'oct', k: base64UrlEncode(data), alg: hmacJwkAlg(brand.algorithm.hash.name),
        key_ops: usages, ext: brand.extractable
      };
    }
    throw dom('NotSupportedError', 'HMAC keys export to raw or jwk');
  }

  // ---- KDF base keys and AES keys ---------------------------------------------
  // HKDF and PBKDF2 keys are raw secrets that only derive. AES keys are created
  // here from raw bytes, as deriveKey needs; the ciphers, jwk and generateKey
  // belong to the AES phase.

  var KDF_USAGES = ['deriveKey', 'deriveBits'];
  var AES_USAGES = {
    'AES-CBC': ['encrypt', 'decrypt', 'wrapKey', 'unwrapKey'],
    'AES-CTR': ['encrypt', 'decrypt', 'wrapKey', 'unwrapKey'],
    'AES-GCM': ['encrypt', 'decrypt', 'wrapKey', 'unwrapKey'],
    'AES-KW': ['wrapKey', 'unwrapKey']
  };
  var HASH_BYTES = { 'SHA-1': 20, 'SHA-256': 32, 'SHA-384': 48, 'SHA-512': 64 };

  function kdfImport(format, keyData, alg, extractable, usages) {
    if (format !== 'raw' && format !== 'raw-secret')
      throw dom('NotSupportedError', alg.name + ' keys import from raw');
    if (!onlyAllowed(usages, KDF_USAGES)) throw dom('SyntaxError', alg.name + ' keys only derive');
    if (extractable) throw dom('SyntaxError', alg.name + ' keys are not extractable');
    usages = intersect(usages, KDF_USAGES);
    var id = storedKey(invoke(native.secretStore, alg.name, '0', usageList(usages), copyBytes(keyData)));
    return makeKey(id, 'secret', false, { name: alg.name }, usages);
  }

  function aesImport(format, keyData, alg, extractable, usages) {
    var allowed = AES_USAGES[alg.name];
    if (!onlyAllowed(usages, allowed)) throw dom('SyntaxError', alg.name + ' keys do not allow these usages');
    if (format !== 'raw' && format !== 'raw-secret')
      throw dom('NotSupportedError', alg.name + ' keys import from raw');
    var data = copyBytes(keyData), length = data.length * 8;
    if (length !== 128 && length !== 192 && length !== 256)
      throw dom('DataError', 'AES key data must be 128, 192 or 256 bits');
    usages = intersect(usages, allowed);
    var id = storedKey(invoke(native.secretStore, 'AES', extractable ? '1' : '0', usageList(usages), data));
    return makeKey(id, 'secret', extractable, { name: alg.name, length: length }, usages);
  }

  function aesExport(format, brand) {
    if (format !== 'raw' && format !== 'raw-secret') throw dom('NotSupportedError', 'AES keys export to raw');
    var data = invoke(brand.native.exportKey, brand.id);
    if (data === null) throw dom('OperationError', 'The key material is unavailable');
    return toArrayBuffer(data);
  }

  // Import by the normalised algorithm's name.
  function importSecret(format, keyData, alg, extractable, usages) {
    if (alg.name === 'HMAC') return hmacImport(format, keyData, alg, extractable, usages);
    if (alg.name === 'HKDF' || alg.name === 'PBKDF2') return kdfImport(format, keyData, alg, extractable, usages);
    return aesImport(format, keyData, alg, extractable, usages);
  }

  // "Get key length" for a derived key's algorithm; null for the KDFs.
  function derivedKeyBits(alg) {
    if (alg.name === 'HMAC') {
      if (alg.length === undefined) return hashBlockBits(alg.hash.name);
      if (alg.length === 0) throw typeError("Algorithm: 'length' must not be zero");
      return alg.length;
    }
    if (alg.name === 'HKDF' || alg.name === 'PBKDF2') return null;
    if (alg.length !== 128 && alg.length !== 192 && alg.length !== 256)
      throw dom('OperationError', 'AES key length must be 128, 192 or 256');
    return alg.length;
  }

  // The derive-bits preconditions that fail with OperationError.
  function deriveCheck(alg, bits) {
    if (bits === null || bits % 8 !== 0) throw dom('OperationError', 'The length must be a multiple of 8');
    if (alg.name === 'HKDF') {
      if (bits > 255 * HASH_BYTES[alg.hash.name] * 8) throw dom('OperationError', 'HKDF length is too large');
    } else if (alg.iterations === 0) {
      throw dom('OperationError', 'PBKDF2 iterations must not be zero');
    }
  }
  // The native parameters: HKDF's info, or PBKDF2's iterations, in one slot.
  function deriveParams(alg) {
    return [alg.name, alg.hash.name, copyBytes(alg.salt),
      alg.name === 'HKDF' ? copyBytes(alg.info) : StringCtor(alg.iterations)];
  }

  // ---- JSON Web Key --------------------------------------------------------

  function toJsonWebKey(value) {
    if (value !== undefined && value !== null && !isObject(value)) throw typeError('keyData is not a JsonWebKey');
    var src = value === undefined || value === null ? {} : value, jwk = {};
    var strings = ['alg', 'crv', 'd', 'dp', 'dq', 'e', 'k', 'kty', 'n', 'p', 'q', 'qi', 'use', 'x', 'y'];
    for (var i = 0; i < strings.length; i++) {
      var v = src[strings[i]];
      if (v !== undefined) jwk[strings[i]] = toDOMString(v);
    }
    var ext = src.ext;
    if (ext !== undefined) jwk.ext = !!ext;
    var ops = src.key_ops;
    if (ops !== undefined) jwk.key_ops = toSequence(ops, 'key_ops', toDOMString);
    return jwk;
  }
  function checkJwkCommon(jwk, usages, extractable, use) {
    if (usages.length && jwk.use !== undefined && jwk.use !== use)
      throw dom('DataError', "JWK 'use' does not allow these usages");
    if (jwk.key_ops !== undefined) {
      var ops = jwk.key_ops;
      for (var i = 0; i < ops.length; i++) {
        for (var k = 0; k < i; k++) if (ops[k] === ops[i]) throw dom('DataError', "JWK 'key_ops' repeats an operation");
      }
      for (var u = 0; u < usages.length; u++) {
        var listed = false;
        for (var m = 0; m < ops.length; m++) if (ops[m] === usages[u]) listed = true;
        if (!listed) throw dom('DataError', "JWK 'key_ops' does not include every usage");
      }
    }
    if (jwk.ext === false && extractable) throw dom('DataError', 'JWK is not extractable');
  }
  function base64UrlDecode(text) {
    var b64 = '';
    for (var i = 0; i < text.length; i++) {
      var c = text[i];
      if (c === '-') b64 += '+';
      else if (c === '_') b64 += '/';
      else if ((c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9')) b64 += c;
      else throw dom('DataError', 'JWK member is not base64url');
    }
    if (b64.length % 4 === 1) throw dom('DataError', 'JWK member is not base64url');
    while (b64.length % 4) b64 += '=';
    try { return atobFn(b64); } catch (badBase64) { throw dom('DataError', 'JWK member is not base64url'); }
  }
  function base64UrlEncode(binary) {
    var b64 = btoaFn(binary), out = '';
    for (var i = 0; i < b64.length; i++) {
      var c = b64[i];
      if (c === '+') out += '-';
      else if (c === '/') out += '_';
      else if (c !== '=') out += c;
    }
    return out;
  }

  // ---- SubtleCrypto --------------------------------------------------------

  function SubtleCrypto() { throw typeError('Illegal constructor'); }
  function promise(self, body) {
    return new PromiseCtor(function(resolve, reject) {
      try {
        if (!isObject(self) || !call(weakHas, brands.subtles, self)) throw typeError('Illegal invocation');
        resolve(body());
      } catch (error) { reject(error); }
    });
  }
  function requireArgs(args, count, op) {
    if (args.length < count)
      throw typeError("SubtleCrypto." + op + ": " + count + ' arguments required, but only ' + args.length + ' present');
  }
  function requireKey(value, what) {
    if (!isObject(value) || !call(weakHas, brands.keys, value)) throw typeError(what + ' is not a CryptoKey');
    return call(weakGet, brands.keys, value);
  }
  function method(name, length, body) {
    var fn = function() {
      var args = arguments, self = this;
      return promise(self, function() { return body(args); });
    };
    defineProperty(fn, 'name', { value: name, configurable: true });
    defineProperty(fn, 'length', { value: length, configurable: true });
    defineProperty(SubtleCrypto.prototype, name, { value: fn, writable: true, enumerable: true, configurable: true });
  }
  function notSupported(op) { return dom('NotSupportedError', 'No supported algorithm for ' + op); }

  method('encrypt', 3, function(a) {
    requireArgs(a, 3, 'encrypt');
    var alg = toAlgorithmIdentifier(a[0]); requireKey(a[1], 'key'); toBufferSource(a[2], 'data');
    normalize(alg, 'encrypt');
    throw notSupported('encrypt');
  });
  method('decrypt', 3, function(a) {
    requireArgs(a, 3, 'decrypt');
    var alg = toAlgorithmIdentifier(a[0]); requireKey(a[1], 'key'); toBufferSource(a[2], 'data');
    normalize(alg, 'decrypt');
    throw notSupported('decrypt');
  });
  method('sign', 3, function(a) {
    requireArgs(a, 3, 'sign');
    var alg = toAlgorithmIdentifier(a[0]), brand = requireKey(a[1], 'key');
    var source = toBufferSource(a[2], 'data');
    var normalized = normalize(alg, 'sign'), data = copyBytes(source);
    if (normalized.name !== brand.algorithm.name) throw dom('InvalidAccessError', 'The key is for a different algorithm');
    if (!hasUsage(brand, 'sign')) throw dom('InvalidAccessError', "The key does not allow 'sign'");
    var tag = invoke(brand.native.sign, brand.id, data);
    if (tag === null) throw dom('OperationError', 'The key could not sign');
    return toArrayBuffer(tag);
  });
  method('verify', 4, function(a) {
    requireArgs(a, 4, 'verify');
    var alg = toAlgorithmIdentifier(a[0]), brand = requireKey(a[1], 'key');
    var sigSource = toBufferSource(a[2], 'signature'), dataSource = toBufferSource(a[3], 'data');
    var normalized = normalize(alg, 'verify');
    var signature = copyBytes(sigSource), data = copyBytes(dataSource);
    if (normalized.name !== brand.algorithm.name) throw dom('InvalidAccessError', 'The key is for a different algorithm');
    if (!hasUsage(brand, 'verify')) throw dom('InvalidAccessError', "The key does not allow 'verify'");
    var ok = invoke(brand.native.verify, brand.id, data, signature);
    if (ok === null) throw dom('OperationError', 'The key could not verify');
    return ok === '1';
  });
  method('digest', 2, function(a) {
    requireArgs(a, 2, 'digest');
    var alg = toAlgorithmIdentifier(a[0]), source = toBufferSource(a[1], 'data');
    var normalized = normalize(alg, 'digest');
    var result = invoke(native.digest, normalized.name, copyBytes(source));
    if (result === null) throw dom('OperationError', 'Digest failed');
    return toArrayBuffer(result);
  });
  method('generateKey', 3, function(a) {
    requireArgs(a, 3, 'generateKey');
    var alg = toAlgorithmIdentifier(a[0]), extractable = !!a[1], usages = toUsages(a[2]);
    var normalized = normalize(alg, 'generateKey');
    var key = hmacGenerate(normalized, extractable, usages);
    if (keyBrand(key).usages.length === 0) throw dom('SyntaxError', 'A secret key needs at least one usage');
    return key;
  });
  method('deriveKey', 5, function(a) {
    requireArgs(a, 5, 'deriveKey');
    var alg = toAlgorithmIdentifier(a[0]), brand = requireKey(a[1], 'baseKey');
    var derived = toAlgorithmIdentifier(a[2]), extractable = !!a[3], usages = toUsages(a[4]);
    var normalized = normalize(alg, 'deriveBits');
    var importAlg = normalize(derived, 'importKey'), lengthAlg = normalize(derived, 'get key length');
    if (normalized.name !== brand.algorithm.name) throw dom('InvalidAccessError', 'The key is for a different algorithm');
    if (!hasUsage(brand, 'deriveKey')) throw dom('InvalidAccessError', "The key does not allow 'deriveKey'");
    var bits = derivedKeyBits(lengthAlg);
    deriveCheck(normalized, bits);
    // The import step's preconditions, ahead of the native work. A KDF target
    // has no length, so it never gets here.
    var allowed = importAlg.name === 'HMAC' ? HMAC_USAGES : AES_USAGES[importAlg.name];
    if (!onlyAllowed(usages, allowed)) throw dom('SyntaxError', 'The derived key does not allow these usages');
    usages = intersect(usages, allowed);
    if (usages.length === 0) throw dom('SyntaxError', 'A secret key needs at least one usage');
    var params = deriveParams(normalized), hmac = importAlg.name === 'HMAC';
    var id = invoke(brand.native.deriveKey, brand.id, 'deriveKey', params[0], params[1], params[2], params[3],
      StringCtor(bits), hmac ? 'HMAC' : 'AES', hmac ? importAlg.hash.name : '', extractable ? '1' : '0',
      usageList(usages));
    if (id === null || id === '!O') throw dom('OperationError', 'The key could not be derived');
    return makeKey(id, 'secret', extractable,
      hmac ? hmacAlgorithm(importAlg.hash.name, bits) : { name: importAlg.name, length: bits }, usages, brand.native);
  });
  method('deriveBits', 2, function(a) {
    requireArgs(a, 2, 'deriveBits');
    var alg = toAlgorithmIdentifier(a[0]), brand = requireKey(a[1], 'baseKey');
    var length = a[2] === undefined || a[2] === null ? null : toEnforcedULong(a[2], 'length');
    var normalized = normalize(alg, 'deriveBits');
    if (normalized.name !== brand.algorithm.name) throw dom('InvalidAccessError', 'The key is for a different algorithm');
    if (!hasUsage(brand, 'deriveBits')) throw dom('InvalidAccessError', "The key does not allow 'deriveBits'");
    deriveCheck(normalized, length);
    var params = deriveParams(normalized);
    var bytes = invoke(brand.native.derive, brand.id, 'deriveBits', params[0], params[1], params[2], params[3],
      StringCtor(length));
    if (bytes === null || bytes === '!O') throw dom('OperationError', 'The bits could not be derived');
    return toArrayBuffer(bytes);
  });
  method('importKey', 5, function(a) {
    requireArgs(a, 5, 'importKey');
    var format = toFormat(a[0]), keyData = a[1];
    var isBytes = true, source = null, jwk = null;
    try { source = toBufferSource(keyData, 'keyData'); } catch (notBytes) { isBytes = false; }
    if (!isBytes) jwk = toJsonWebKey(keyData);
    var alg = toAlgorithmIdentifier(a[2]), extractable = !!a[3], usages = toUsages(a[4]);
    var normalized = normalize(alg, 'importKey');
    if (format === 'jwk') {
      if (isBytes) throw typeError('A jwk import needs a JsonWebKey');
    } else if (!isBytes) {
      throw typeError('A ' + format + ' import needs a BufferSource');
    }
    var key = importSecret(format, isBytes ? source : jwk, normalized, extractable, usages);
    var brand = keyBrand(key);
    if ((brand.type === 'secret' || brand.type === 'private') && brand.usages.length === 0)
      throw dom('SyntaxError', 'A secret key needs at least one usage');
    return key;
  });
  method('exportKey', 2, function(a) {
    requireArgs(a, 2, 'exportKey');
    var format = toFormat(a[0]), brand = requireKey(a[1], 'key');
    var name = brand.algorithm.name;
    if (name !== 'HMAC' && !AES_USAGES[name]) throw dom('NotSupportedError', 'This key cannot be exported');
    if (!brand.extractable) throw dom('InvalidAccessError', 'The key is not extractable');
    return name === 'HMAC' ? hmacExport(format, brand) : aesExport(format, brand);
  });
  method('wrapKey', 4, function(a) {
    requireArgs(a, 4, 'wrapKey');
    toFormat(a[0]); requireKey(a[1], 'key'); requireKey(a[2], 'wrappingKey');
    var alg = toAlgorithmIdentifier(a[3]);
    try { normalize(alg, 'wrapKey'); } catch (noWrap) { normalize(alg, 'encrypt'); }
    throw notSupported('wrapKey');
  });
  method('unwrapKey', 7, function(a) {
    requireArgs(a, 7, 'unwrapKey');
    toFormat(a[0]); toBufferSource(a[1], 'wrappedKey'); requireKey(a[2], 'unwrappingKey');
    var alg = toAlgorithmIdentifier(a[3]), keyAlg = toAlgorithmIdentifier(a[4]); toUsages(a[6]);
    try { normalize(alg, 'unwrapKey'); } catch (noUnwrap) { normalize(alg, 'decrypt'); }
    normalize(keyAlg, 'importKey');
    throw notSupported('unwrapKey');
  });
  if (SymbolToStringTag) defineProperty(SubtleCrypto.prototype, SymbolToStringTag, { value: 'SubtleCrypto', configurable: true });

  var subtle = create(SubtleCrypto.prototype);
  call(weakSet, brands.subtles, subtle, true);
  function defineInterface(ctor) {
    defineProperty(ctor, 'prototype', { writable: false, enumerable: false, configurable: false });
    defineProperty(ctor.prototype, 'constructor', { value: ctor, writable: true, enumerable: false, configurable: true });
  }
  defineInterface(CryptoKey);
  defineInterface(SubtleCrypto);

  // CryptoKey, SubtleCrypto and Crypto.prototype.subtle are [SecureContext]:
  // present only in a secure context. The realm's flag decides at install;
  // the host's `set_secure_context` re-decides through the private hook below
  // before author script runs.
  var CryptoCtor = typeof Crypto === 'function' ? Crypto : null;
  function expose(secure) {
    if (secure) {
      defineProperty(globalThis, 'CryptoKey', { value: CryptoKey, writable: true, enumerable: false, configurable: true });
      defineProperty(globalThis, 'SubtleCrypto', { value: SubtleCrypto, writable: true, enumerable: false, configurable: true });
      if (CryptoCtor) defineProperty(CryptoCtor.prototype, 'subtle', {
        get: function() { return subtle; }, enumerable: true, configurable: true
      });
    } else {
      delete globalThis.CryptoKey;
      delete globalThis.SubtleCrypto;
      if (CryptoCtor) delete CryptoCtor.prototype.subtle;
    }
  }
  expose(invoke(native.secure) === '1');
  globalThis.__subtleExposure = function(flag) { expose(flag === '1'); return flag; };
})();
