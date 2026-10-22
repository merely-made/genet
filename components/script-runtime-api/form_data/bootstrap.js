// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
(function () {
  var Core = globalThis.__streamsFetch.core, DOM = globalThis.__formsDOM;
  delete globalThis.__formsDOM;
  var TE = Core.TypeError, DE = DOMException, N = Core.Number, Str = Core.String;
  var parse = JSON.parse, charCode = String.prototype.charCodeAt, fromChar = String.fromCharCode;
  var lower = String.prototype.toLowerCase, floor = Math.floor, ceil = Math.ceil, max = Math.max, min = Math.min, finite = isFinite;
  var nativeAllocate = __formFileAllocate, nativeReplace = __formFilesReplace, nativeFiles = __formFilesSnapshot;
  var nativeEntries = __formEntrySnapshot, nativeEncode = __formDataEncode;
  var nativeCheck = __formSubmitterCheck, nativeInfo = __formControlInfo, nativeMeta = __formMeta;
  var nativeImplicit = __formImplicitDecision, nativePlan = __formSubmissionPlan;
  var nativeMarkUserValidity = __formSubmissionUserValidity;
  var NativeProxy = Proxy, reflectGet = Reflect.get, nativeNow = Date.now;
  var blobSlots = Core.sharedMap('forms.blobs'), fileSlots = Core.sharedMap('forms.files');
  var fdSlots = Core.sharedMap('forms.data'), listSlots = Core.sharedMap('forms.lists');
  var dragSlots = Core.sharedMap('forms.drag'), itemSlots = Core.sharedMap('forms.items');
  var selectionSlots = Core.sharedMap('forms.selection'), flags = Core.sharedMap('forms.flags');
  var positions = Core.sharedMap('forms.clickPositions'), clicking = Core.sharedMap('forms.clicking');
  function list() { var q = Core.record(); q.length = 0; return q; }
  function add(q, x) { q[q.length++] = x; }
  function remove(q, i) { for (var j = i; j + 1 < q.length; j++) q[j] = q[j + 1]; if (i < q.length) delete q[--q.length]; }
  function copyList(q) { var out = list(); for (var i = 0; i < q.length; i++) add(out, q[i]); return out; }
  function record(map, value, label) { if (!Core.has(map, value)) throw new TE('Illegal ' + label + ' receiver'); return Core.get(map, value); }
  function usv(value) {
    if (typeof value === 'symbol') throw new TE('Cannot convert a Symbol to a string');
    var text = Str(value), out = '';
    for (var i = 0; i < text.length; i++) {
      var c = Core.call(charCode, text, [i]);
      if (c >= 0xD800 && c <= 0xDBFF) {
        var d = i + 1 < text.length ? Core.call(charCode, text, [i + 1]) : 0;
        if (d >= 0xDC00 && d <= 0xDFFF) { out += Core.call(fromChar, undefined, [c, d]); i++; continue; }
        c = 0xFFFD;
      } else if (c >= 0xDC00 && c <= 0xDFFF) c = 0xFFFD;
      out += Core.call(fromChar, undefined, [c]);
    }
    return out;
  }
  function quote(s) {
    var out = '"';
    for (var i = 0; i < s.length; i++) {
      var c = Core.call(charCode, s, [i]);
      if (c === 34) out += '\\"'; else if (c === 92) out += '\\\\';
      else if (c < 32 || c >= 0xD800 && c <= 0xDFFF) {
        var digits = '0123456789abcdef'; out += '\\u' + digits[(c >> 12) & 15] + digits[(c >> 8) & 15] + digits[(c >> 4) & 15] + digits[c & 15];
      } else out += Core.call(fromChar, undefined, [c]);
    }
    return out + '"';
  }
  function bytesFromArray(a) { var out = new Core.Uint8Array(a.length); for (var i = 0; i < a.length; i++) out[i] = a[i]; return out; }
  function bytesWire(bytes) { var out = '['; for (var i = 0, n = Core.viewInfo(bytes).byteLength; i < n; i++) out += (i ? ',' : '') + Str(bytes[i]); return out + ']'; }
  function type(value) { var s = usv(value), valid = true; for (var i = 0; i < s.length; i++) { var c = Core.call(charCode, s, [i]); if (c < 32 || c > 126) valid = false; } return valid ? Core.call(lower, s, []) : ''; }
  function blobBytes(value) { return record(blobSlots, value, 'Blob').bytes; }
  function partBytes(part) {
    if (Core.has(blobSlots, part)) return Core.copyBytes(blobBytes(part));
    if (Core.isArrayBuffer(part)) return Core.copyBytes(new Core.Uint8Array(part));
    var view = false; try { Core.viewInfo(part); view = true; } catch (_) {}
    if (view) return Core.copyBytes(part);
    return encodeString(usv(part));
  }
  // Pure UTF-8 operations remain usable after the File's creator host retires.
  function encodeString(text) {
    var values = list();
    for (var i = 0; i < text.length; i++) {
      var c = Core.call(charCode, text, [i]);
      if (c >= 0xD800 && c <= 0xDBFF) { var d = Core.call(charCode, text, [++i]); c = 0x10000 + ((c - 0xD800) << 10) + d - 0xDC00; }
      if (c < 128) add(values, c);
      else if (c < 2048) { add(values, 0xC0 | c >> 6); add(values, 0x80 | c & 63); }
      else if (c < 65536) { add(values, 0xE0 | c >> 12); add(values, 0x80 | c >> 6 & 63); add(values, 0x80 | c & 63); }
      else { add(values, 0xF0 | c >> 18); add(values, 0x80 | c >> 12 & 63); add(values, 0x80 | c >> 6 & 63); add(values, 0x80 | c & 63); }
    }
    return bytesFromArray(values);
  }
  function decodeBytes(bytes) {
    var out = '', needed = 0, seen = 0, point = 0, low = 128, high = 191, first = true;
    function emit(c) { if (first) { first = false; if (c === 0xFEFF) return; } out += c <= 65535 ? Core.call(fromChar, undefined, [c]) : Core.call(fromChar, undefined, [0xD800 + ((c - 65536) >> 10), 0xDC00 + ((c - 65536) & 1023)]); }
    for (var i = 0; i < bytes.length; i++) {
      var c = bytes[i];
      if (needed === 0) {
        if (c < 128) emit(c);
        else if (c >= 194 && c <= 223) { needed = 1; point = c & 31; }
        else if (c >= 224 && c <= 239) { needed = 2; point = c & 15; if (c === 224) low = 160; if (c === 237) high = 159; }
        else if (c >= 240 && c <= 244) { needed = 3; point = c & 7; if (c === 240) low = 144; if (c === 244) high = 143; }
        else emit(65533);
      } else if (c < low || c > high) { needed = seen = point = 0; low = 128; high = 191; emit(65533); i--; }
      else { low = 128; high = 191; point = (point << 6) | c & 63; if (++seen === needed) { emit(point); needed = seen = point = 0; } }
    }
    if (needed) emit(65533); return out;
  }
  function initBlob(object, parts, options) {
    options = options == null ? Core.record() : options;
    var chunks = list(), total = 0;
    if (parts !== undefined) {
      if (parts === null || typeof parts !== 'object') throw new TE('Blob parts must be a sequence');
      var iterator = parts[Core.Symbol.iterator]; if (typeof iterator !== 'function') throw new TE('Blob parts must be iterable');
      var it = Core.call(iterator, parts, []), step;
      while (!(step = it.next()).done) { var bytes = partBytes(step.value); add(chunks, bytes); total += Core.viewInfo(bytes).byteLength; }
    }
    var out = new Core.Uint8Array(total), offset = 0;
    for (var i = 0; i < chunks.length; i++) { var bytes = chunks[i]; for (var j = 0; j < Core.viewInfo(bytes).byteLength; j++) out[offset++] = bytes[j]; }
    var state = Core.record(); state.bytes = out; state.type = options.type === undefined ? '' : type(options.type); Core.set(blobSlots, object, state);
  }
  function Blob(parts, options) { if (!new.target) throw new TE('Blob requires new'); initBlob(this, parts, options); }
  function getter(proto, name, read) { Core.define(proto, name, { configurable: true, enumerable: true, get: read }); }
  getter(Blob.prototype, 'size', function () { return Core.viewInfo(blobBytes(this)).byteLength; });
  getter(Blob.prototype, 'type', function () { return record(blobSlots, this, 'Blob').type; });
  Blob.prototype.arrayBuffer = function () { return Core.resolve(Core.viewInfo(Core.copyBytes(blobBytes(this))).buffer); };
  Blob.prototype.text = function () { return Core.resolve(decodeBytes(Core.copyBytes(blobBytes(this)))); };
  Blob.prototype.slice = function (start, end, contentType) {
    var bytes = blobBytes(this), info = Core.viewInfo(bytes), size = info.byteLength;
    function index(n, fallback) { n = n === undefined ? fallback : N(n); if (n !== n) n = 0; n = n < 0 ? ceil(n) : floor(n); return n < 0 ? max(size + n, 0) : min(n, size); }
    var a = index(start, 0), b = max(a, index(end, size)), object = Core.create(blobPrototype), state = Core.record();
    state.bytes = Core.copyBytes(new Core.Uint8Array(info.buffer, info.byteOffset + a, b - a)); state.type = contentType === undefined ? '' : type(contentType); Core.set(blobSlots, object, state); return object;
  };
  var blobPrototype = Blob.prototype;
  function blobFactory(bytes, mime) {
    var object = Core.create(blobPrototype), state = Core.record();
    state.bytes = Core.copyBytes(bytes); state.type = type(mime); Core.set(blobSlots, object, state); return object;
  }
  function fileFactory(bytes, name, mime, modified, token) {
    var object = Core.create(filePrototype), blob = Core.record(), file = Core.record();
    blob.bytes = Core.copyBytes(bytes); blob.type = type(mime); file.name = name; file.lastModified = modified; file.token = token === undefined ? null : token;
    Core.set(blobSlots, object, blob); Core.set(fileSlots, object, file);
    if (file.token === null) file.token = nativeAllocate(object);
    return object;
  }
  function File(parts, name, options) {
    if (!new.target || arguments.length < 2) throw new TE('File requires parts and name');
    initBlob(this, parts, options); var state = Core.record(); state.name = usv(name);
    var modified = options && options.lastModified !== undefined ? N(options.lastModified) : Core.call(nativeNow, undefined, []);
    state.lastModified = !finite(modified) ? 0 : modified < 0 ? ceil(modified) : floor(modified); state.token = null; Core.set(fileSlots, this, state); state.token = nativeAllocate(this);
  }
  File.prototype = Core.create(blobPrototype); Core.define(File.prototype, 'constructor', { value: File, configurable: true, writable: true });
  var filePrototype = File.prototype;
  getter(filePrototype, 'name', function () { return record(fileSlots, this, 'File').name; });
  getter(filePrototype, 'lastModified', function () { return record(fileSlots, this, 'File').lastModified; });
  function fileWire(value) {
    var file = record(fileSlots, value, 'File'), blob = record(blobSlots, value, 'File');
    return '{"name":' + quote(file.name) + ',"type":' + quote(blob.type) + ',"bytes":' + bytesWire(blob.bytes) + ',"token":' + (file.token === null ? 'null' : quote(file.token)) + '}';
  }
  function duplicateFile(value, name) { var file = record(fileSlots, value, 'File'), blob = record(blobSlots, value, 'File'); return fileFactory(blob.bytes, name === undefined ? file.name : name, blob.type, file.lastModified); }
  function FileList() { throw new TE('Illegal FileList constructor'); }
  function indexKey(key) { if (typeof key !== 'string' || key === '') return -1; var n = N(key); return n >= 0 && n < 4294967295 && floor(n) === n && Str(n) === key ? n : -1; }
  function makeFileList(reader) {
    var target = Core.create(fileListPrototype), proxy = new NativeProxy(target, {
      get: function (t, key, receiver) { var i = indexKey(key); if (i >= 0) { var q = reader(); return i < q.length ? q[i] : undefined; } return Core.call(reflectGet, undefined, [t, key, receiver]); },
      ownKeys: function () { var q = reader(), keys = []; for (var i = 0; i < q.length; i++) keys[i] = Str(i); return keys; },
      getOwnPropertyDescriptor: function (_, key) { var i = indexKey(key), q = reader(); if (i >= 0 && i < q.length) return { value: q[i], enumerable: true, configurable: true }; },
      set: function () { return false; }
    }); Core.set(listSlots, proxy, reader); return proxy;
  }
  var fileListPrototype = FileList.prototype;
  function fileListValues(value) { return Core.call(record(listSlots, value, 'FileList'), undefined, []); }
  getter(fileListPrototype, 'length', function () { return fileListValues(this).length; });
  fileListPrototype.item = function (index) { var q = fileListValues(this); if (arguments.length < 1) throw new TE('Index required'); index = N(index) >>> 0; return index < q.length ? q[index] : null; };
  fileListPrototype[Core.Symbol.iterator] = function () { var self = this, i = 0; return { next: function () { var q = fileListValues(self); return i < q.length ? { value: q[i++], done: false } : { value: undefined, done: true }; }, [Core.Symbol.iterator]: function () { return this; } }; };
  function selectedState(ref) { var state = Core.get(selectionSlots, ref); if (!state) { state = Core.record(); state.files = list(); state.list = makeFileList(function () { var live = refreshSelection(ref); return live === null ? list() : live.files; }); Core.set(selectionSlots, ref, state); } return state; }
  // Native JSON records must contribute only known own data. Missing fields
  // never consult author-modified prototypes, including the absent text/file tag.
  function nativeOwnData(object, key) {
    var descriptor = Core.descriptor(object, key);
    if (descriptor === undefined || Core.descriptor(descriptor, 'value') === undefined) throw new TE('Invalid native form record');
    return descriptor.value;
  }
  function nativeString(object, key) {
    var value = nativeOwnData(object, key); if (typeof value !== 'string') throw new TE('Invalid native form string'); return value;
  }
  function nativeFileRecord(object) {
    var out = Core.record(); out.name = nativeString(object, 'name'); out.type = nativeString(object, 'type'); out.token = nativeString(object, 'token');
    var bytes = nativeOwnData(object, 'bytes'), length = nativeOwnData(bytes, 'length');
    if (typeof length !== 'number' || !finite(length) || length < 0 || floor(length) !== length) throw new TE('Invalid native byte length');
    out.bytes = new Core.Uint8Array(length);
    for (var i = 0; i < length; i++) {
      var byte = nativeOwnData(bytes, Str(i));
      if (typeof byte !== 'number' || byte < 0 || byte > 255 || floor(byte) !== byte) throw new TE('Invalid native byte');
      out.bytes[i] = byte;
    }
    return out;
  }
  function nativeFile(record, ref) {
    if (ref !== null) { var state = refreshSelection(ref); for (var i = 0; i < state.files.length; i++) if (Core.get(fileSlots, state.files[i]).token === record.token) return state.files[i]; }
    return fileFactory(record.bytes, record.name, record.type, 0, record.token);
  }
  function refreshSelection(ref) {
    var wire = nativeFiles(ref); if (wire === null) { var prior = Core.get(selectionSlots, ref); if (prior) prior.files = list(); return null; }
    var records = Core.call(parse, undefined, [wire]), state = selectedState(ref), files = list();
    var length = nativeOwnData(records, 'length');
    for (var i = 0; i < length; i++) {
      var record = nativeFileRecord(nativeOwnData(records, Str(i)));
      var value = null; for (var j = 0; j < state.files.length; j++) if (Core.get(fileSlots, state.files[j]).token === record.token) { value = state.files[j]; break; }
      if (value === null) value = fileFactory(record.bytes, record.name, record.type, 0, record.token); add(files, value);
    }
    state.files = files; return state;
  }
  function DataTransferItem() { throw new TE('Illegal DataTransferItem constructor'); }
  getter(DataTransferItem.prototype, 'kind', function () { return record(itemSlots, this, 'DataTransferItem').kind; });
  getter(DataTransferItem.prototype, 'type', function () { return record(itemSlots, this, 'DataTransferItem').type; });
  DataTransferItem.prototype.getAsFile = function () { var r = record(itemSlots, this, 'DataTransferItem'); return r.kind === 'file' ? duplicateFile(r.value) : null; };
  DataTransferItem.prototype.getAsString = function (callback) { var r = record(itemSlots, this, 'DataTransferItem'); if (r.kind === 'string' && typeof callback === 'function') DOM.task(function () { callback(r.value); }); };
  var itemPrototype = DataTransferItem.prototype;
  function DataTransferItemList() { throw new TE('Illegal DataTransferItemList constructor'); }
  var itemListPrototype = DataTransferItemList.prototype;
  var dragItemLists = Core.sharedMap('forms.dragItemLists');
  function dragItems(value) { return record(dragItemLists, value, 'DataTransferItemList'); }
  getter(DataTransferItemList.prototype, 'length', function () { return dragItems(this).items.length; });
  DataTransferItemList.prototype.add = function (value, kind) {
    var state = dragItems(this), item = Core.create(itemPrototype), r = Core.record();
    if (Core.has(fileSlots, value)) { r.kind = 'file'; r.type = Core.get(blobSlots, value).type; r.value = value; }
    else { if (arguments.length < 2) throw new TE('String type required'); r.kind = 'string'; r.type = Core.call(lower, usv(kind), []); r.value = usv(value); for (var i = 0; i < state.items.length; i++) { var other = Core.get(itemSlots, state.items[i]); if (other.kind === 'string' && other.type === r.type) throw new DE('Duplicate type', 'NotSupportedError'); } }
    Core.set(itemSlots, item, r); add(state.items, item); return item;
  };
  DataTransferItemList.prototype.remove = function (index) { var state = dragItems(this); remove(state.items, N(index) >>> 0); };
  DataTransferItemList.prototype.clear = function () { dragItems(this).items = list(); };
  function DataTransfer() {
    if (!new.target) throw new TE('DataTransfer requires new');
    var state = Core.record(); state.items = list(); state.dropEffect = 'none'; state.effectAllowed = 'none';
    var target = Core.create(itemListPrototype), proxy = new NativeProxy(target, { get: function (t, key, receiver) { var i = indexKey(key); return i >= 0 ? state.items[i] : Core.call(reflectGet, undefined, [t, key, receiver]); } });
    Core.set(dragItemLists, proxy, state); state.itemList = proxy;
    state.files = makeFileList(function () { var q = list(); for (var i = 0; i < state.items.length; i++) { var r = Core.get(itemSlots, state.items[i]); if (r.kind === 'file') add(q, r.value); } return q; }); Core.set(dragSlots, this, state);
  }
  getter(DataTransfer.prototype, 'items', function () { return record(dragSlots, this, 'DataTransfer').itemList; });
  getter(DataTransfer.prototype, 'files', function () { return record(dragSlots, this, 'DataTransfer').files; });
  getter(DataTransfer.prototype, 'types', function () { var state = record(dragSlots, this, 'DataTransfer'), out = [], hasFiles = false; for (var i = 0; i < state.items.length; i++) { var r = Core.get(itemSlots, state.items[i]); if (r.kind === 'file') hasFiles = true; else out[out.length] = r.type; } if (hasFiles) out[out.length] = 'Files'; return out; });
  DataTransfer.prototype.clearData = function (kind) { var state = record(dragSlots, this, 'DataTransfer'), t = kind === undefined ? null : Core.call(lower, usv(kind), []); for (var i = state.items.length - 1; i >= 0; i--) { var r = Core.get(itemSlots, state.items[i]); if (r.kind === 'string' && (t === null || r.type === t)) remove(state.items, i); } };
  var clearDragData = DataTransfer.prototype.clearData, addDragItem = DataTransferItemList.prototype.add;
  DataTransfer.prototype.setData = function (kind, value) { Core.call(clearDragData, this, [kind]); Core.call(addDragItem, record(dragSlots, this, 'DataTransfer').itemList, [value, kind]); };
  DataTransfer.prototype.getData = function (kind) { var state = record(dragSlots, this, 'DataTransfer'); kind = Core.call(lower, usv(kind), []); for (var i = 0; i < state.items.length; i++) { var r = Core.get(itemSlots, state.items[i]); if (r.kind === 'string' && r.type === kind) return r.value; } return ''; };
  ['dropEffect', 'effectAllowed'].forEach(function (key) { Core.define(DataTransfer.prototype, key, { configurable: true, enumerable: true, get: function () { return record(dragSlots, this, 'DataTransfer')[key]; }, set: function (value) { record(dragSlots, this, 'DataTransfer')[key] = usv(value); } }); });
  function fdFactory() { var fd = Core.create(fdPrototype); Core.set(fdSlots, fd, list()); return fd; }
  function pair(name, value) { var p = Core.record(); p.name = name; p.value = value; return p; }
  function formFlags(form) { var ref = DOM.ref(form, 'form'), state = Core.get(flags, ref); if (!state) { state = Core.record(); state.constructing = false; state.firing = false; Core.set(flags, ref, state); } return state; }
  function validateSubmitter(form, submitter) { var f = DOM.ref(form, 'form'), b = submitter === null ? null : DOM.elementRef(submitter); var error = nativeCheck(f, b); if (error === 'TypeError') throw new TE('Not a submit button'); if (error) throw new DE('Submitter belongs to another form', error); return b; }
  function construction(form, submitter, encoding, x, y) {
    var state = formFlags(form); if (state.constructing) throw new DE('Recursive entry construction', 'InvalidStateError'); state.constructing = true;
    try {
      var fd = fdFactory(), q = Core.get(fdSlots, fd);
      nativeEntries(DOM.ref(form, 'form'), submitter === null ? null : DOM.elementRef(submitter), encoding, Str(x), Str(y), function (wire, input) {
        var r = Core.call(parse, undefined, [wire]), name = nativeString(r, 'name'), value;
        if (Core.descriptor(r, 'file') === undefined) value = nativeString(r, 'text');
        else {
          if (Core.descriptor(r, 'text') !== undefined) throw new TE('Invalid native form entry');
          value = nativeFile(nativeFileRecord(nativeOwnData(r, 'file')), input);
        }
        add(q, pair(name, value));
      });
      DOM.dispatch(form, DOM.formEvent(form, 'formdata', fd));
      var finalData = fdFactory(); Core.set(fdSlots, finalData, copyList(Core.get(fdSlots, fd))); return finalData;
    } finally { state.constructing = false; }
  }
  function FormData(form, submitter) {
    if (!new.target) throw new TE('FormData requires new'); Core.set(fdSlots, this, list());
    if (form !== undefined) { submitter = submitter == null ? null : submitter; validateSubmitter(form, submitter); var result = construction(form, submitter, 'UTF-8', 0, 0); Core.set(fdSlots, this, copyList(Core.get(fdSlots, result))); }
  }
  var fdPrototype = FormData.prototype;
  function converted(value, filename, hasFilename) {
    if (!Core.has(blobSlots, value)) { if (hasFilename) throw new TE('Filename requires Blob'); return usv(value); }
    if (Core.has(fileSlots, value)) return hasFilename ? duplicateFile(value, usv(filename)) : value;
    var blob = Core.get(blobSlots, value); return fileFactory(blob.bytes, hasFilename ? usv(filename) : 'blob', blob.type, Core.call(nativeNow, undefined, []));
  }
  fdPrototype.append = function (name, value, filename) { var q = record(fdSlots, this, 'FormData'); if (arguments.length < 2) throw new TE('Two arguments required'); add(q, pair(usv(name), converted(value, filename, arguments.length > 2))); };
  fdPrototype.set = function (name, value, filename) { var q = record(fdSlots, this, 'FormData'); if (arguments.length < 2) throw new TE('Two arguments required'); name = usv(name); value = converted(value, filename, arguments.length > 2); var first = -1; for (var i = 0; i < q.length; i++) if (q[i].name === name) { if (first < 0) { q[i] = pair(name, value); first = i; } else remove(q, i--); } if (first < 0) add(q, pair(name, value)); };
  fdPrototype['delete'] = function (name) { var q = record(fdSlots, this, 'FormData'); if (arguments.length < 1) throw new TE('Name required'); name = usv(name); for (var i = q.length - 1; i >= 0; i--) if (q[i].name === name) remove(q, i); };
  fdPrototype.get = function (name) { var q = record(fdSlots, this, 'FormData'); if (arguments.length < 1) throw new TE('Name required'); name = usv(name); for (var i = 0; i < q.length; i++) if (q[i].name === name) return q[i].value; return null; };
  fdPrototype.getAll = function (name) { var q = record(fdSlots, this, 'FormData'), out = []; if (arguments.length < 1) throw new TE('Name required'); name = usv(name); for (var i = 0; i < q.length; i++) if (q[i].name === name) out[out.length] = q[i].value; return out; };
  fdPrototype.has = function (name) { var q = record(fdSlots, this, 'FormData'); if (arguments.length < 1) throw new TE('Name required'); name = usv(name); for (var i = 0; i < q.length; i++) if (q[i].name === name) return true; return false; };
  fdPrototype.forEach = function (callback, thisArg) { var q = record(fdSlots, this, 'FormData'); if (typeof callback !== 'function') throw new TE('Callback required'); for (var i = 0; i < q.length; i++) Core.call(callback, thisArg, [q[i].value, q[i].name, this]); };
  function iterator(fd, kind) { record(fdSlots, fd, 'FormData'); var i = 0; return { next: function () { var q = Core.get(fdSlots, fd); if (i >= q.length) return { value: undefined, done: true }; var p = q[i++]; return { value: kind === 'keys' ? p.name : kind === 'values' ? p.value : [p.name, p.value], done: false }; }, [Core.Symbol.iterator]: function () { return this; } }; }
  fdPrototype.entries = function () { return iterator(this, 'entries'); }; fdPrototype.keys = function () { return iterator(this, 'keys'); }; fdPrototype.values = function () { return iterator(this, 'values'); }; fdPrototype[Core.Symbol.iterator] = fdPrototype.entries;
  function initializeEvent(event, type, bubbles, cancelable, composed, trusted, key, value) {
    var fields = Core.record(); fields.type = type; fields.bubbles = !!bubbles; fields.cancelable = !!cancelable; fields.composed = !!composed; fields.defaultPrevented = false; fields.__canceled = false; fields.eventPhase = 0; fields.target = null; fields.currentTarget = null; fields.srcElement = null; fields.__initialized = true; fields.__dispatch = false; fields.__stop = false; fields.__stopImmediate = false; fields.__inPassive = false;
    for (var field in fields) Core.define(event, field, { value: fields[field], writable: true, enumerable: true, configurable: true });
    Core.define(event, key, { get: function () { return value; }, enumerable: true }); DOM.trust(event, trusted); return event;
  }
  function eventDictionary(init) {
    if (init === undefined || init === null) return Core.record();
    if (typeof init !== 'object' && typeof init !== 'function') throw new TE('Event dictionary required');
    return init;
  }
  function SubmitEvent(type, init) {
    if (!new.target || arguments.length < 1) throw new TE('Event type required'); type = usv(type); init = eventDictionary(init);
    var bubbles = !!init.bubbles, cancelable = !!init.cancelable, composed = !!init.composed, submitter = init.submitter;
    if (submitter == null) submitter = null; else DOM.elementRef(submitter);
    // The engine allocated this with new.target's prototype, including derived
    // classes. Dictionary getters run once; the initializer reads no author input.
    return initializeEvent(this, type, bubbles, cancelable, composed, false, 'submitter', submitter);
  }
  SubmitEvent.prototype = Core.create(DOM.eventPrototype); Core.define(SubmitEvent.prototype, 'constructor', { value: SubmitEvent, configurable: true, writable: true }); var submitEventPrototype = SubmitEvent.prototype;
  function FormDataEvent(type, init) {
    if (!new.target || arguments.length < 2) throw new TE('formData required'); type = usv(type); init = eventDictionary(init);
    var bubbles = !!init.bubbles, cancelable = !!init.cancelable, composed = !!init.composed, data = init.formData;
    if (!Core.has(fdSlots, data)) throw new TE('formData required');
    return initializeEvent(this, type, bubbles, cancelable, composed, false, 'formData', data);
  }
  FormDataEvent.prototype = Core.create(DOM.eventPrototype); Core.define(FormDataEvent.prototype, 'constructor', { value: FormDataEvent, configurable: true, writable: true }); var dataEventPrototype = FormDataEvent.prototype;
  // UA dispatch uses fixed captured creator prototypes, independent of public
  // constructors and of the realm executing a borrowed form method.
  function creatorFormEvent(kind, value) {
    if (kind === 'submit') return initializeEvent(Core.create(submitEventPrototype), 'submit', true, true, false, true, 'submitter', value);
    if (kind === 'formdata') return initializeEvent(Core.create(dataEventPrototype), 'formdata', true, false, false, true, 'formData', value);
    throw new TE('Invalid private form event');
  }
  function meta(form, submitter) { return Core.call(parse, undefined, [nativeMeta(DOM.ref(form, 'form'), submitter === null ? null : DOM.elementRef(submitter))]); }
  function submit(form, submitter, bypass, x, y) {
    validateSubmitter(form, submitter); var state = formFlags(form); if (state.constructing || !bypass && state.firing) return;
    var m = meta(form, submitter); if (!m.eligible) return;
    if (!bypass) {
      state.firing = true;
      try {
        nativeMarkUserValidity(DOM.ref(form, 'form'));
        if (!m.noValidate && !DOM.validate(form)) return;
        if (!DOM.dispatch(form, DOM.formEvent(form, 'submit', submitter))) return;
        if (!meta(form, submitter).eligible) return;
      } finally { state.firing = false; }
    }
    // A nested bypass never owns the outer submit listener's firing flag.
    // Selection precedes construction and survives mutations in formdata.
    m = meta(form, submitter); var encoding = m.encoding, fd = construction(form, submitter, encoding, x, y);
    if (!meta(form, submitter).eligible) return;
    nativePlan(DOM.ref(form, 'form'), submitter === null ? null : DOM.elementRef(submitter), fd, encoding);
  }
  DOM.formPrototype.submit = function () { DOM.ref(this, 'form'); submit(this, null, true, 0, 0); };
  DOM.formPrototype.requestSubmit = function (submitter) { DOM.ref(this, 'form'); submitter = submitter == null ? null : submitter; validateSubmitter(this, submitter); submit(this, submitter, false, 0, 0); };
  function info(target) { if (!DOM.isElement(target)) return null; return Core.call(parse, undefined, [nativeInfo(DOM.nodeRef(target))]); }
  function activate(target, event) { var s = info(target); if (!s || !s.submit || s.disabled) return; var form = DOM.owner(target); if (!form) return; var p = Core.get(positions, event); submit(form, target, false, p ? p[0] : 0, p ? p[1] : 0); }
  function click(target, trusted, x, y) { DOM.elementRef(target); var s = info(target); if (s && s.disabled) return true; var event = DOM.pointer(trusted); Core.set(positions, event, [x, y]); return DOM.dispatch(target, event); }
  DOM.htmlPrototype.click = function () { DOM.elementRef(this); if (Core.get(clicking, this)) return; Core.set(clicking, this, true); try { click(this, false, 0, 0); } finally { Core.delete(clicking, this); } };
  function key(target, details) { var d = Core.call(parse, undefined, [details]); if (!DOM.key(target, d)) return false; if (d.down === false || d.key !== 'Enter' || d.alt || d.ctrl || d.meta || d.shift || !DOM.focused(target)) return true; var state = info(target); if (!state || !state.implicit || state.disabled) return true; var form = DOM.owner(target); if (!form) return true; var choice = nativeImplicit(DOM.ref(form, 'form')); if (choice === 'form') submit(form, null, false, 0, 0); else if (choice !== 'blocked') click(DOM.wrap(choice), true, 0, 0); return true; }
  Core.define(DOM.inputPrototype, 'files', { configurable: true, enumerable: true, get: function () { var state = refreshSelection(DOM.ref(this, 'input')); return state === null ? null : state.list; }, set: function (value) { var ref = DOM.ref(this, 'input'); if (value === null) return; var q = copyList(fileListValues(value)); if (nativeReplace(ref, value) === 'true') selectedState(ref).files = q; } });
  function encode(fd, encoding, kind) { record(fdSlots, fd, 'FormData'); var r = Core.call(parse, undefined, [nativeEncode(fd, encoding, kind)]); var result = Core.record(); result.bytes = bytesFromArray(r.bytes); result.type = r.type; return result; }
  var Ops = Core.record(); Ops.isBlob = function (x) { return Core.has(blobSlots, x); }; Ops.isFile = function (x) { return Core.has(fileSlots, x); }; Ops.isData = function (x) { return Core.has(fdSlots, x); }; Ops.isList = function (x) { return Core.has(listSlots, x); }; Ops.listValues = fileListValues; Ops.makeList = function (q) { var copy = list(); for (var i = 0; i < q.length; i++) add(copy, q[i]); return makeFileList(function () { return copy; }); };
  Ops.blob = function (x) { var b = record(blobSlots, x, 'Blob'), out = Core.record(); out.bytes = Core.copyBytes(b.bytes); out.type = b.type; if (Core.has(fileSlots, x)) { var f = Core.get(fileSlots, x); out.name = f.name; out.lastModified = f.lastModified; } return out; }; Ops.file = fileFactory; Ops.makeBlob = blobFactory; Ops.Blob = Blob; Ops.File = File; Ops.FormData = FormData; Ops.encode = encode; Ops.usv = usv; Ops.bytes = bytesFromArray; Ops.binary = function(bytes) { var out = ''; for (var i = 0; i < bytes.length; i++) out += Core.call(fromChar, undefined, [bytes[i] & 255]); return out; };
  Ops.private = function (kind, value, token) {
    if (kind === 'reject') throw new TE(value);
    if (kind === 'commitSelection') { selectedState(value).files = copyList(fileListValues(token)); return; }
    if (kind === 'testToken') { record(fileSlots, value, 'File').token = token; return; }
    if (kind === 'file') return fileWire(value);
    var q, out = '[';
    if (kind === 'list') { q = fileListValues(value); for (var i = 0; i < q.length; i++) out += (i ? ',' : '') + fileWire(q[i]); }
    else if (kind === 'fd') { q = record(fdSlots, value, 'FormData'); for (var i = 0; i < q.length; i++) out += (i ? ',' : '') + '{"name":' + quote(q[i].name) + ',' + (Core.has(fileSlots, q[i].value) ? '"file":' + fileWire(q[i].value) : '"text":' + quote(q[i].value)) + '}'; }
    else throw new TE('Unknown private operation'); return out + ']';
  };
  Ops.activation = activate; Ops.activationTarget = function (target) { var s = info(target); return s && s.submit; }; Ops.click = click; Ops.hostClick = function(target,x,y) { DOM.nodeRef(target); if (DOM.isElement(target)) return click(target,true,x,y); return DOM.dispatch(target,DOM.pointer(true)); }; Ops.key = key;
  Ops.event = creatorFormEvent; DOM.complete(Ops); globalThis.__formsPrivate = Ops;
  var interfaces = { Blob: Blob, File: File, FileList: FileList, DataTransfer: DataTransfer, DataTransferItem: DataTransferItem, DataTransferItemList: DataTransferItemList, FormData: FormData, SubmitEvent: SubmitEvent, FormDataEvent: FormDataEvent };
  var lengths = { Blob: 0, File: 2, FileList: 0, DataTransfer: 0, DataTransferItem: 0, DataTransferItemList: 0, FormData: 0, SubmitEvent: 1, FormDataEvent: 2 };
  for (var name in lengths) Core.define(interfaces[name], 'length', { value: lengths[name], configurable: true });
  Core.define(fdPrototype.append, 'length', { value: 2, configurable: true });
  Core.define(fdPrototype.set, 'length', { value: 2, configurable: true });
  Core.define(fdPrototype.forEach, 'length', { value: 1, configurable: true });
  Core.define(blobPrototype.slice, 'length', { value: 0, configurable: true });
  Core.define(DataTransferItemList.prototype.add, 'length', { value: 1, configurable: true });
  Core.define(DataTransfer.prototype.clearData, 'length', { value: 0, configurable: true });
  for (var name in interfaces) { Core.define(interfaces[name].prototype, Core.Symbol.toStringTag, { value: name, configurable: true }); Core.define(globalThis, name, { value: interfaces[name], writable: true, configurable: true }); }
  var transient = ['__hostFormEdit','__formFileAllocate','__formFilesReplace','__formFilesSnapshot','__formEntrySnapshot','__formDataEncode','__formControlInfo','__formSubmitterCheck','__formImplicitDecision','__formMeta','__formSubmissionPlan','__formSubmissionUserValidity']; for (var i = 0; i < transient.length; i++) delete globalThis[transient[i]];
})();
