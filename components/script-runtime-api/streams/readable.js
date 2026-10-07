var ReadableOps = (function (Core) {
  'use strict';

  var streamSlots = Core.sharedMap('readable-stream');
  var defaultControllerSlots = Core.sharedMap('readable-default-controller');
  var byteControllerSlots = Core.sharedMap('readable-byte-controller');
  var defaultReaderSlots = Core.sharedMap('readable-default-reader');
  var byobReaderSlots = Core.sharedMap('readable-byob-reader');
  var byobRequestSlots = Core.sharedMap('readable-byob-request');
  var TypeErrorCtor = Core.TypeError;
  var RangeErrorCtor = Core.RangeError;
  var Uint8ArrayCtor = Core.Uint8Array;
  var ArrayBufferCtor = Core.ArrayBuffer;
  var SymbolCtor = Core.Symbol;
  var undefinedValue;
  var readableStreamPrototype;
  var defaultReaderPrototype;
  var byobReaderPrototype;
  var defaultControllerPrototype;
  var byteControllerPrototype;
  var byobRequestPrototype;

  function typeError(message) { return new TypeErrorCtor(message); }
  function rangeError(message) { return new RangeErrorCtor(message); }
  function call(fn, receiver, args) { return Core.call(fn, receiver, args); }
  function toDOMString(value) {
    if (typeof value === 'symbol') throw typeError('Cannot convert a Symbol to a string');
    return call(Core.String, undefinedValue, [value]);
  }
  function mapGet(map, key) { return Core.get(map, key); }
  function mapSet(map, key, value) { Core.set(map, key, value); }
  function own(obj, key, value) {
    Core.define(obj, key, { value: value, configurable: true, writable: true, enumerable: false });
  }
  function defineMethod(proto, key, fn, length, enumerable) {
    Core.define(fn, 'name', { value: key, configurable: true });
    Core.define(fn, 'length', { value: length, configurable: true });
    Core.define(proto, key, { value: fn, writable: true, configurable: true, enumerable: !!enumerable });
  }
  function defineGetter(proto, key, getter) {
    Core.define(getter, 'name', { value: 'get ' + key, configurable: true });
    Core.define(proto, key, { get: getter, configurable: true, enumerable: true });
  }
  function illegalConstructor(name) {
    throw typeError("Illegal constructor: " + name);
  }
  function nullRecord() { return Core.record(); }
  function linkedQueue() { var q = Core.record(); q.head = null; q.tail = null; q.length = 0; return q; }
  function queuePush(q, value) {
    var node = Core.record(); node.value = value; node.next = null;
    if (q.tail) q.tail.next = node; else q.head = node;
    q.tail = node;
    q.length++;
  }
  function queueShift(q) {
    if (!q.head) return undefinedValue;
    var node = q.head;
    q.head = node.next;
    if (!q.head) q.tail = null;
    q.length--;
    return node.value;
  }
  function queuePeek(q) { return q.head ? q.head.value : undefinedValue; }
  function queueClear(q) { q.head = null; q.tail = null; q.length = 0; }
  function createStreamObject() {
    var object = Core.create(readableStreamPrototype);
    var slot = Core.record();
    slot.state = 'readable'; slot.reader = undefinedValue; slot.disturbed = false;
    slot.storedError = undefinedValue; slot.controller = undefinedValue;
    slot.controllerKind = undefinedValue; slot.cancelled = false;
    mapSet(streamSlots, object, slot);
    return object;
  }
  function streamSlot(stream) { return mapGet(streamSlots, stream); }
  function defaultController(stream) {
    var slot = streamSlot(stream);
    if (!slot || slot.controllerKind !== 'default') throw typeError('Invalid ReadableStream receiver');
    return mapGet(defaultControllerSlots, slot.controller);
  }
  function byteController(stream) {
    var slot = streamSlot(stream);
    if (!slot || slot.controllerKind !== 'byte') throw typeError('Invalid ReadableStream receiver');
    return mapGet(byteControllerSlots, slot.controller);
  }
  function currentReader(reader) {
    var slot = mapGet(defaultReaderSlots, reader);
    if (slot) { var defaultInfo = Core.record(); defaultInfo.slot = slot; defaultInfo.kind = 'default'; return defaultInfo; }
    slot = mapGet(byobReaderSlots, reader);
    if (slot) { var byobInfo = Core.record(); byobInfo.slot = slot; byobInfo.kind = 'byob'; return byobInfo; }
    throw typeError('Invalid reader receiver');
  }
  function readerStream(reader) {
    var info = currentReader(reader);
    if (!info.slot.stream) throw typeError('Reader has been released');
    var result = Core.record(); result.info = info; result.stream = info.slot.stream; result.state = streamSlot(info.slot.stream); return result;
  }
  function readerIsBYOB(reader) { return !!mapGet(byobReaderSlots, reader); }
  function setReaderClosed(reader, state, reason) {
    var info = currentReader(reader), slot = info.slot;
    if (!slot.closedSettled) {
      slot.closedSettled = true;
      if (state === 'closed') slot.closedDeferred.resolve(undefinedValue);
      else slot.closedDeferred.reject(reason);
    }
  }
  function makeReaderClosed(streamState) {
    var d = Core.deferred();
    var slot = Core.record(); slot.closedDeferred = d; slot.closedSettled = false;
    if (streamState.state === 'closed') {
      slot.closedSettled = true;
      d.resolve(undefinedValue);
    } else if (streamState.state === 'errored') {
      slot.closedSettled = true;
      d.reject(streamState.storedError);
    }
    Core.handled(d.promise);
    return slot;
  }
  function settleReadRequest(request, kind, value) {
    if (!request || request.settled) return;
    request.settled = true;
    if (kind === 'chunk') request.steps.chunk(value);
    else if (kind === 'close') request.steps.close(value);
    else request.steps.error(value);
  }
  function resolveReadRequest(request, kind, value) {
    settleReadRequest(request, kind, value);
  }
  function normalizeReadSteps(steps) {
    var normalized = Core.record();
    var names = ['chunk', 'close', 'error'];
    for (var i = 0; i < names.length; ++i) {
      var descriptor = steps == null ? undefinedValue : Core.descriptor(steps, names[i]);
      normalized[names[i]] = descriptor === undefinedValue ? null : descriptor.value;
    }
    return normalized;
  }
  function rejectPendingReads(readerSlot, reason) {
    while (readerSlot.readRequests && readerSlot.readRequests.length) {
      resolveReadRequest(queueShift(readerSlot.readRequests), 'error', reason);
    }
    while (readerSlot.readIntoRequests && readerSlot.readIntoRequests.length) {
      resolveReadRequest(queueShift(readerSlot.readIntoRequests), 'error', reason);
    }
  }
  function finishReaderClosed(stream, error) {
    var slot = streamSlot(stream), reader = slot && slot.reader;
    if (!reader) return;
    setReaderClosed(reader, error ? 'errored' : 'closed', error ? slot.storedError : undefinedValue);
  }
  function makeDefaultController(stream, algorithms, strategy) {
    var controller = Core.create(defaultControllerPrototype);
    var c = Core.record();
    c.stream = stream; c.queue = Core.queue(); c.closeRequested = false;
    c.started = false; c.pulling = false; c.pullAgain = false;
    c.pullAlgorithm = algorithms.pull || null; c.cancelAlgorithm = algorithms.cancel || null;
    c.sizeAlgorithm = strategy.sizeAlgorithm; c.highWaterMark = strategy.highWaterMark;
    c.publicObject = controller;
    mapSet(defaultControllerSlots, controller, c);
    var s = streamSlot(stream);
    s.controller = controller;
    s.controllerKind = 'default';
    return controller;
  }
  function makeByteController(stream, algorithms, strategy, autoAllocateChunkSize) {
    var controller = Core.create(byteControllerPrototype);
    var c = Core.record();
    c.stream = stream; c.queue = linkedQueue(); c.queueTotalSize = 0;
    c.pendingPullIntos = linkedQueue(); c.closeRequested = false;
    c.started = false; c.pulling = false; c.pullAgain = false;
    c.pullAlgorithm = algorithms.pull || null; c.cancelAlgorithm = algorithms.cancel || null;
    c.highWaterMark = strategy.highWaterMark;
    c.autoAllocateChunkSize = autoAllocateChunkSize || 0;
    c.byobRequest = null; c.publicObject = controller;
    mapSet(byteControllerSlots, controller, c);
    var s = streamSlot(stream);
    s.controller = controller;
    s.controllerKind = 'byte';
    return controller;
  }
  function invokeStart(stream, algorithms, controller) {
    var slot = streamSlot(stream), result;
    try {
      result = algorithms.start ? call(algorithms.start, undefinedValue, [controller]) : undefinedValue;
    } catch (e) {
      throw e;
    }
    var p = Core.resolve(result);
    Core.react(p, function () {
      if (!slot || slot.state !== 'readable') return undefinedValue;
      var c = slot.controllerKind === 'byte'
        ? mapGet(byteControllerSlots, slot.controller)
        : mapGet(defaultControllerSlots, slot.controller);
      if (!c) return undefinedValue;
      c.started = true;
      callPullIfNeeded(c, slot.controllerKind);
      return undefinedValue;
    }, function (reason) {
      if (slot && slot.state === 'readable') errorStream(stream, reason);
      return undefinedValue;
    });
  }
  function normalizeAlgorithms(algorithms) {
    var normalized = Core.record();
    var names = ['start', 'pull', 'cancel'];
    for (var i = 0; i < names.length; ++i) {
      var descriptor = algorithms == null ? undefinedValue : Core.descriptor(algorithms, names[i]);
      normalized[names[i]] = descriptor === undefinedValue ? null : descriptor.value;
    }
    return normalized;
  }
  function createDefault(algorithms, strategy) {
    var stream = createStreamObject();
    algorithms = normalizeAlgorithms(algorithms);
    var controller = makeDefaultController(stream, algorithms, strategy);
    invokeStart(stream, algorithms, controller);
    return stream;
  }
  function createByte(algorithms, strategy, autoAllocateChunkSize) {
    var stream = createStreamObject();
    algorithms = normalizeAlgorithms(algorithms);
    var controller = makeByteController(stream, algorithms, strategy, autoAllocateChunkSize);
    invokeStart(stream, algorithms, controller);
    return stream;
  }
  function streamDesiredSize(stream) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Invalid ReadableStream receiver');
    if (slot.state === 'errored') return null;
    if (slot.state === 'closed') return 0;
    if (slot.controllerKind === 'byte') {
      var bc = mapGet(byteControllerSlots, slot.controller);
      return bc.highWaterMark - bc.queueTotalSize;
    }
    var dc = mapGet(defaultControllerSlots, slot.controller);
    return dc.highWaterMark - dc.queue.totalSize;
  }
  function canCloseOrEnqueue(stream) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Invalid ReadableStream receiver');
    if (slot.controllerKind === 'byte') {
      var bc = mapGet(byteControllerSlots, slot.controller);
      return slot.state === 'readable' && !bc.closeRequested;
    }
    var dc = mapGet(defaultControllerSlots, slot.controller);
    return slot.state === 'readable' && !dc.closeRequested;
  }
  function callPullIfNeeded(controller, kind) {
    var stream = streamSlot(controller.stream);
    if (!stream || !stream.state || stream.state !== 'readable' || controller.closeRequested || !controller.started) return;
    var shouldPull;
    if (kind === 'byte') {
      var reader = stream.reader;
      var hasRead = reader && (readerIsBYOB(reader)
        ? mapGet(byobReaderSlots, reader).readIntoRequests.length > 0
        : mapGet(defaultReaderSlots, reader).readRequests.length > 0);
      shouldPull = !!hasRead || (controller.highWaterMark > controller.queueTotalSize);
    } else {
      var r = stream.reader;
      shouldPull = !!(r && mapGet(defaultReaderSlots, r) && mapGet(defaultReaderSlots, r).readRequests.length) ||
        (controller.highWaterMark - controller.queue.totalSize > 0);
    }
    if (!shouldPull || !controller.pullAlgorithm) return;
    if (controller.pulling) { controller.pullAgain = true; return; }
    controller.pulling = true;
    var promise;
    try { promise = Core.resolve(call(controller.pullAlgorithm, undefinedValue, [controller.publicObject])); }
    catch (e) {
      controller.pulling = false;
      errorStream(controller.stream, e);
      return;
    }
    Core.react(promise, function () {
      controller.pulling = false;
      if (controller.pullAgain) {
        controller.pullAgain = false;
        callPullIfNeeded(controller, kind);
      }
      return undefinedValue;
    }, function (reason) {
      controller.pulling = false;
      errorStream(controller.stream, reason);
      return undefinedValue;
    });
  }
  function readDefaultPullSteps(controller, request) {
    var stream = streamSlot(controller.stream);
    if (Core.length(controller.queue) > 0) {
      var chunk = Core.shift(controller.queue);
      resolveReadRequest(request, 'chunk', chunk);
      if (controller.closeRequested && Core.length(controller.queue) === 0) closeStream(controller.stream);
      else callPullIfNeeded(controller, 'default');
      return;
    }
    if (stream.state === 'closed') { resolveReadRequest(request, 'close', undefinedValue); return; }
    if (stream.state === 'errored') { resolveReadRequest(request, 'error', stream.storedError); return; }
    queuePush(mapGet(defaultReaderSlots, stream.reader).readRequests, request);
    callPullIfNeeded(controller, 'default');
  }
  function appendByteEntry(controller, view) {
    var info = Core.viewInfo(view);
    var entry = Core.record(); entry.view = view; entry.offset = 0; entry.length = info.byteLength; entry.next = null;
    var q = controller.queue;
    if (q.tail) q.tail.next = entry; else q.head = entry;
    q.tail = entry;
    controller.queueTotalSize += info.byteLength;
  }
  function byteEntryShift(controller) {
    var q = controller.queue;
    if (!q.head) return null;
    var e = q.head;
    q.head = e.next;
    if (!q.head) q.tail = null;
    e.next = null;
    controller.queueTotalSize -= e.length - e.offset;
    return e;
  }
  function byteEntryPrepend(controller, view) {
    var info = Core.viewInfo(view);
    var entry = Core.record(); entry.view = view; entry.offset = 0; entry.length = info.byteLength; entry.next = controller.queue.head;
    controller.queue.head = entry;
    if (!controller.queue.tail) controller.queue.tail = entry;
    controller.queueTotalSize += info.byteLength;
  }
  function copyQueueIntoDescriptor(controller, descriptor) {
    var target = Core.view('Uint8Array', descriptor.buffer, descriptor.byteOffset + descriptor.bytesFilled,
      descriptor.byteLength - descriptor.bytesFilled);
    var targetLength = descriptor.byteLength - descriptor.bytesFilled;
    var written = 0;
    while (controller.queue.head && written < targetLength) {
      var entry = controller.queue.head;
      var source = entry.view;
      var sourceLength = entry.length - entry.offset;
      var count = sourceLength < targetLength - written ? sourceLength : targetLength - written;
      var sourceInfo = Core.viewInfo(source);
      var sourceBytes = Core.view('Uint8Array', sourceInfo.buffer,
        sourceInfo.byteOffset + entry.offset, count);
      for (var i = 0; i < count; i++) target[written + i] = sourceBytes[i];
      entry.offset += count;
      written += count;
      controller.queueTotalSize -= count;
      if (entry.offset >= entry.length) {
        controller.queue.head = entry.next;
        if (!controller.queue.head) controller.queue.tail = null;
        entry.next = null;
      }
    }
    descriptor.bytesFilled += written;
    return written;
  }
  function makeDescriptor(view, readerType, request, minimumFill) {
    var info = Core.viewInfo(view);
    var buffer = Core.transfer(info.buffer);
    var descriptor = Core.record();
    descriptor.buffer = buffer; descriptor.bufferByteLength = Core.bufferByteLength(buffer);
    descriptor.byteOffset = info.byteOffset; descriptor.byteLength = info.byteLength;
    descriptor.bytesFilled = 0; descriptor.minimumFill = minimumFill;
    descriptor.elementSize = info.elementSize; descriptor.kind = info.kind;
    descriptor.readerType = readerType; descriptor.request = request;
    return descriptor;
  }
  function descriptorView(descriptor, byteLength) {
    if (descriptor.kind === 'DataView') {
      return Core.view('DataView', descriptor.buffer, descriptor.byteOffset, byteLength);
    }
    return Core.view(descriptor.kind, descriptor.buffer, descriptor.byteOffset, byteLength);
  }
  function shiftPullInto(controller) {
    return queueShift(controller.pendingPullIntos);
  }
  function removeDescriptorRequest(stream, descriptor) {
    var slot = streamSlot(stream), reader = slot && slot.reader;
    if (!reader || !descriptor.request) return;
    var rslot = readerIsBYOB(reader) ? mapGet(byobReaderSlots, reader) : mapGet(defaultReaderSlots, reader);
    if (!rslot) return;
    var requests = readerIsBYOB(reader) ? rslot.readIntoRequests : rslot.readRequests;
    if (queuePeek(requests) === descriptor.request) queueShift(requests);
  }
  function processByteQueue(controller) {
    while (controller.pendingPullIntos.length > 0 && controller.queue.head) {
      var descriptor = queuePeek(controller.pendingPullIntos);
      copyQueueIntoDescriptor(controller, descriptor);
      var aligned = descriptor.bytesFilled - (descriptor.bytesFilled % descriptor.elementSize);
      if (aligned >= descriptor.minimumFill || (controller.closeRequested && !controller.queue.head)) {
        shiftPullInto(controller);
        removeDescriptorRequest(controller.stream, descriptor);
        var remainder = descriptor.bytesFilled - aligned;
        var out;
        if (descriptor.readerType === 'none') {
          out = descriptorView(descriptor, descriptor.bytesFilled);
          if (descriptor.bytesFilled > 0) byteEntryPrepend(controller, out);
        } else {
          if (remainder > 0) {
            var remStart = descriptor.byteOffset + aligned;
            var rem = Core.view('Uint8Array', descriptor.buffer, remStart, remainder);
            appendByteEntry(controller, rem);
          }
          descriptor.bytesFilled = aligned;
          out = descriptorView(descriptor, aligned);
          resolveReadRequest(descriptor.request, 'chunk', out);
        }
      } else {
        break;
      }
    }
    while (controller.queue.head) {
      var slot = streamSlot(controller.stream), reader = slot.reader;
      if (!reader || readerIsBYOB(reader)) break;
      var rs = mapGet(defaultReaderSlots, reader);
      if (!rs.readRequests.length) break;
      var request = queueShift(rs.readRequests);
      var entry = byteEntryShift(controller);
      var info = Core.viewInfo(entry.view);
      var chunk = Core.view('Uint8Array', info.buffer, info.byteOffset + entry.offset, entry.length - entry.offset);
      resolveReadRequest(request, 'chunk', chunk);
    }
    if (controller.closeRequested && !controller.queue.head) closeStream(controller.stream);
  }
  function invalidateBYOBRequest(controller) {
    var request = controller.byobRequest;
    if (!request) return;
    var slot = mapGet(byobRequestSlots, request);
    if (slot) { slot.controller = null; slot.view = null; }
    controller.byobRequest = null;
  }
  function currentPullInto(controller) { return queuePeek(controller.pendingPullIntos); }
  function getBYOBRequest(controller) {
    var descriptor = currentPullInto(controller), stream = streamSlot(controller.stream);
    if (!descriptor || !stream) return null;
    if (controller.byobRequest) return controller.byobRequest;
    if (stream.state !== 'readable') return null;
    var remaining = descriptor.byteLength - descriptor.bytesFilled;
    var view = Core.view('Uint8Array', descriptor.buffer,
      descriptor.byteOffset + descriptor.bytesFilled, remaining);
    var request = Core.create(byobRequestPrototype);
    var requestSlot = Core.record(); requestSlot.controller = controller.publicObject; requestSlot.view = view;
    mapSet(byobRequestSlots, request, requestSlot);
    controller.byobRequest = request;
    return request;
  }
  function closeByteController(controller, otherSpec) {
    var stream = streamSlot(controller.stream);
    if (controller.closeRequested || stream.state !== 'readable') throw typeError('ReadableByteStreamController is already closed');
    if (controller.queueTotalSize > 0) {
      controller.closeRequested = true;
      return;
    }
    if (controller.pendingPullIntos.length > 0) {
      var pending = currentPullInto(controller);
      if (pending.bytesFilled > 0 && pending.bytesFilled % pending.elementSize !== 0) {
        var closeError = typeError('Byte stream closed with a partial element');
        errorStream(controller.stream, closeError);
        throw closeError;
      }
      controller.closeRequested = true;
      closeStream(controller.stream);
      return;
    }
    controller.closeRequested = true;
    closeStream(controller.stream);
  }
  function closeStream(stream) {
    var slot = streamSlot(stream);
    if (!slot || slot.state !== 'readable') return;
    if (slot.controllerKind === 'default') {
      var dc = mapGet(defaultControllerSlots, slot.controller);
      if (Core.length(dc.queue) > 0) return;
      slot.state = 'closed';
      finishReaderClosed(stream, false);
      if (slot.reader && mapGet(defaultReaderSlots, slot.reader)) {
        var dr = mapGet(defaultReaderSlots, slot.reader);
        while (dr.readRequests.length) resolveReadRequest(queueShift(dr.readRequests), 'close', undefinedValue);
      }
      clearControllerAlgorithms(dc);
      return;
    }
    var bc = mapGet(byteControllerSlots, slot.controller);
    if (bc.queue.head || bc.queueTotalSize > 0) return;
    for (var pendingNode = bc.pendingPullIntos.head; pendingNode; pendingNode = pendingNode.next) {
      var pendingDescriptor = pendingNode.value;
      if (pendingDescriptor.bytesFilled > 0 && pendingDescriptor.bytesFilled % pendingDescriptor.elementSize !== 0) {
        errorStream(stream, typeError('Byte stream closed with a partial element'));
        return;
      }
    }
    slot.state = 'closed';
    finishReaderClosed(stream, false);
    if (slot.reader && mapGet(defaultReaderSlots, slot.reader) && bc.pendingPullIntos.length === 0) {
      var drb = mapGet(defaultReaderSlots, slot.reader);
      while (drb.readRequests.length) resolveReadRequest(queueShift(drb.readRequests), 'close', undefinedValue);
    }
    clearControllerAlgorithms(bc);
  }
  function clearControllerAlgorithms(controller) {
    controller.pullAlgorithm = null;
    controller.cancelAlgorithm = null;
    controller.sizeAlgorithm = null;
  }
  function errorStream(stream, reason) {
    var slot = streamSlot(stream);
    if (!slot || slot.state !== 'readable') return;
    slot.state = 'errored';
    slot.storedError = reason;
    var c = slot.controllerKind === 'byte'
      ? mapGet(byteControllerSlots, slot.controller)
      : mapGet(defaultControllerSlots, slot.controller);
    if (c) {
      if (slot.controllerKind === 'byte') {
        c.queue.head = null; c.queue.tail = null; c.queueTotalSize = 0;
        queueClear(c.pendingPullIntos); invalidateBYOBRequest(c);
      } else Core.clear(c.queue);
      clearControllerAlgorithms(c);
    }
    finishReaderClosed(stream, true);
    if (slot.reader) rejectPendingReads(mapGet(defaultReaderSlots, slot.reader) || mapGet(byobReaderSlots, slot.reader), reason);
  }
  function cancelStream(stream, reason) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Invalid ReadableStream receiver');
    slot.disturbed = true;
    if (slot.state === 'closed') return Core.resolve(undefinedValue);
    if (slot.state === 'errored') return Core.reject(slot.storedError);
    slot.cancelled = true;
    slot.state = 'closed';
    finishReaderClosed(stream, false);
    if (slot.reader) {
      var readerSlot = mapGet(defaultReaderSlots, slot.reader) || mapGet(byobReaderSlots, slot.reader);
      if (readerSlot) {
        var reads = readerSlot.readRequests, readIntos = readerSlot.readIntoRequests;
        readerSlot.readRequests = linkedQueue();
        readerSlot.readIntoRequests = linkedQueue();
        while (reads.length) resolveReadRequest(queueShift(reads), 'close', undefinedValue);
        while (readIntos.length) resolveReadRequest(queueShift(readIntos), 'close', undefinedValue);
      }
    }
    var controller = slot.controller;
    var cancelAlgorithm, controllerSlot;
    if (slot.controllerKind === 'byte') {
      var bc = mapGet(byteControllerSlots, controller);
      controllerSlot = bc;
      cancelAlgorithm = bc.cancelAlgorithm;
      bc.queue.head = null; bc.queue.tail = null; bc.queueTotalSize = 0;
      invalidateBYOBRequest(bc);
      queueClear(bc.pendingPullIntos);
    } else {
      var dc = mapGet(defaultControllerSlots, controller);
      controllerSlot = dc;
      cancelAlgorithm = dc.cancelAlgorithm;
      Core.clear(dc.queue);
    }
    if (!cancelAlgorithm) {
      clearControllerAlgorithms(controllerSlot);
      return Core.resolve(undefinedValue);
    }
    var sourceCancelPromise;
    try { sourceCancelPromise = Core.resolve(call(cancelAlgorithm, undefinedValue, [reason])); }
    catch (e) { clearControllerAlgorithms(controllerSlot); return Core.reject(e); }
    clearControllerAlgorithms(controllerSlot);
    var cancellation = Core.deferred();
    Core.react(sourceCancelPromise, function () { cancellation.resolve(undefinedValue); },
      function (e) { cancellation.reject(e); });
    return cancellation.promise;
  }
  function defaultControllerEnqueue(stream, chunk) {
    var c = defaultController(stream), slot = streamSlot(stream);
    if (!canCloseOrEnqueue(stream)) throw typeError('Cannot enqueue on a closed stream');
    var size;
    try { size = c.sizeAlgorithm ? call(c.sizeAlgorithm, undefinedValue, [chunk]) : 1; }
    catch (e) { errorStream(stream, e); throw e; }
    if (typeof size !== 'number' || size < 0 || size !== size || size === Infinity) {
      var sizeError = rangeError('Chunk size must be a finite non-negative number');
      errorStream(stream, sizeError);
      throw sizeError;
    }
    if (slot.reader && mapGet(defaultReaderSlots, slot.reader)) {
      var r = mapGet(defaultReaderSlots, slot.reader);
      if (r.readRequests.length) {
        resolveReadRequest(queueShift(r.readRequests), 'chunk', chunk);
        callPullIfNeeded(c, 'default');
        return;
      }
    }
    Core.push(c.queue, chunk, size);
    callPullIfNeeded(c, 'default');
  }
  function byteControllerEnqueue(stream, chunk, otherSpec) {
    var c = byteController(stream), slot = streamSlot(stream), info;
    if (!canCloseOrEnqueue(stream)) throw typeError('Cannot enqueue on a closed byte stream');
    try { info = Core.viewInfo(chunk); } catch (e) { throw typeError('Byte stream chunks must be ArrayBufferViews'); }
    if (info.kind === 'DataView' && !info.byteLength) throw typeError('Byte stream chunks must not be empty');
    if (info.byteLength === 0) throw typeError('Byte stream chunks must not be empty');
    if (Core.isDetached(info.buffer)) throw typeError('Byte stream chunk is detached');
    var activeRequest = getBYOBRequest(c);
    if (activeRequest) {
      var active = mapGet(byobRequestSlots, activeRequest);
      var activeInfo = Core.viewInfo(active.view);
      if (info.buffer === activeInfo.buffer && info.byteOffset === activeInfo.byteOffset &&
          info.byteLength <= activeInfo.byteLength) {
        respondByteController(c, info.byteLength, false);
        return;
      }
    }
    var transferred = Core.transfer(info.buffer);
    var copy = Core.view('Uint8Array', transferred, info.byteOffset, info.byteLength);
    invalidateBYOBRequest(c);
    if (c.pendingPullIntos.length) {
      appendByteEntry(c, copy);
      processByteQueue(c);
    } else if (slot.reader && mapGet(defaultReaderSlots, slot.reader)) {
      var r = mapGet(defaultReaderSlots, slot.reader);
      if (r.readRequests.length) {
        resolveReadRequest(queueShift(r.readRequests), 'chunk', copy);
        callPullIfNeeded(c, 'byte');
        return;
      }
      appendByteEntry(c, copy);
    } else appendByteEntry(c, copy);
    processByteQueue(c);
    callPullIfNeeded(c, 'byte');
  }
  function enqueue(stream, chunk) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Invalid ReadableStream receiver');
    if (slot.controllerKind === 'byte') {
      if (!canCloseOrEnqueue(stream)) throw typeError('Cannot enqueue on a closed byte stream');
      var c = mapGet(byteControllerSlots, slot.controller), info = Core.viewInfo(chunk);
      var request = getBYOBRequest(c);
      if (request) {
        var requestSlot = mapGet(byobRequestSlots, request), requestInfo = Core.viewInfo(requestSlot.view);
        if (info.buffer === requestInfo.buffer && info.byteOffset === requestInfo.byteOffset &&
            info.byteLength <= requestInfo.byteLength) {
          respondByteController(c, info.byteLength, false);
          return;
        }
      }
      return byteControllerEnqueue(stream, chunk, true);
    }
    return defaultControllerEnqueue(stream, chunk);
  }
  function close(stream) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Invalid ReadableStream receiver');
    if (slot.controllerKind === 'byte') {
      var bc = mapGet(byteControllerSlots, slot.controller);
      if (slot.state === 'closed') {
        if (currentPullInto(bc)) respondByteController(bc, 0, false);
        return;
      }
      if (!canCloseOrEnqueue(stream)) return;
      closeByteController(bc, true);
      if (currentPullInto(bc)) respondByteController(bc, 0, false);
    } else {
      var dc = mapGet(defaultControllerSlots, slot.controller);
      if (!canCloseOrEnqueue(stream)) return;
      dc.closeRequested = true;
      if (Core.length(dc.queue) === 0) closeStream(stream);
    }
  }
  function error(stream, reason) { errorStream(stream, reason); }
  function internalByteClose(stream) {
    var slot = streamSlot(stream);
    if (!slot || slot.controllerKind !== 'byte') throw typeError('Not a byte stream');
    if (slot.state === 'readable')
      closeByteController(mapGet(byteControllerSlots, slot.controller), false);
  }
  function internalByteEnqueue(stream, chunk) { return byteControllerEnqueue(stream, chunk, false); }
  function acquireReader(stream, mode) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('ReadableStream receiver is not a stream');
    if (mode === undefinedValue) mode = 'default';
    if (mode !== 'default' && mode !== 'byob') throw typeError('Invalid reader mode');
    if (slot.reader) throw typeError('ReadableStream is locked');
    if (mode === 'byob' && slot.controllerKind !== 'byte') throw typeError('BYOB reader requires a byte stream');
    var reader = Core.create(mode === 'byob' ? byobReaderPrototype : defaultReaderPrototype);
    setupReader(reader, stream, mode);
    return reader;
  }
  function setupReader(reader, stream, kind) {
    var slot = streamSlot(stream);
    if (!slot) throw typeError('Reader requires a ReadableStream');
    if (slot.reader) throw typeError('ReadableStream is already locked');
    var rs = makeReaderClosed(slot);
    rs.stream = stream;
    rs.readRequests = linkedQueue();
    rs.readIntoRequests = linkedQueue();
    if (kind === 'default') mapSet(defaultReaderSlots, reader, rs);
    else mapSet(byobReaderSlots, reader, rs);
    slot.reader = reader;
  }
  function readRequest(reader, steps) {
    var r = readerStream(reader), slot = r.state, request;
    if (r.info.kind !== 'default') throw typeError('readRequest requires a default reader');
    slot.disturbed = true;
    request = Core.record(); request.steps = normalizeReadSteps(steps); request.settled = false;
    if (slot.state === 'closed') { resolveReadRequest(request, 'close', undefinedValue); return; }
    if (slot.state === 'errored') { resolveReadRequest(request, 'error', slot.storedError); return; }
    var controller = slot.controllerKind === 'byte'
      ? mapGet(byteControllerSlots, slot.controller)
      : mapGet(defaultControllerSlots, slot.controller);
    if (slot.controllerKind === 'default') {
      readDefaultPullSteps(controller, request);
      return;
    }
    if (controller.queue.head) {
      var entry = byteEntryShift(controller), info = Core.viewInfo(entry.view);
      var chunk = Core.view('Uint8Array', info.buffer, info.byteOffset + entry.offset, entry.length - entry.offset);
      resolveReadRequest(request, 'chunk', chunk);
      if (controller.closeRequested && !controller.queue.head) closeStream(r.stream);
      else callPullIfNeeded(controller, 'byte');
      return;
    }
    if (controller.closeRequested) { closeStream(r.stream); resolveReadRequest(request, 'close', undefinedValue); return; }
    var defaultReader = mapGet(defaultReaderSlots, reader);
    if (controller.autoAllocateChunkSize > 0) {
      var allocated = new Uint8ArrayCtor(new ArrayBufferCtor(controller.autoAllocateChunkSize));
      var descriptor = makeDescriptor(allocated, 'default', request, 1);
      queuePush(controller.pendingPullIntos, descriptor);
    }
    queuePush(defaultReader.readRequests, request);
    callPullIfNeeded(controller, 'byte');
  }
  function readIntoRequest(reader, view, min, steps) {
    var r = readerStream(reader), slot = r.state;
    if (r.info.kind !== 'byob') throw typeError('readIntoRequest requires a BYOB reader');
    if (slot.controllerKind !== 'byte') throw typeError('BYOB reader requires a byte stream');
    slot.disturbed = true;
    var request = Core.record(); request.steps = normalizeReadSteps(steps); request.settled = false;
    var info;
    try { info = validatePullIntoView(view, min); }
    catch (e) { resolveReadRequest(request, 'error', e); return; }
    if (slot.state === 'errored') { resolveReadRequest(request, 'error', slot.storedError); return; }
    var c = mapGet(byteControllerSlots, slot.controller);
    var descriptor;
    try { descriptor = makeDescriptor(view, 'byob', request, min * info.elementSize); }
    catch (e2) { resolveReadRequest(request, 'error', e2); return; }
    if (slot.state === 'closed') {
      resolveReadRequest(request, 'close', descriptorView(descriptor, 0));
      return;
    }
    queuePush(c.pendingPullIntos, descriptor);
    queuePush(mapGet(byobReaderSlots, reader).readIntoRequests, request);
    if (c.queue.head) processByteQueue(c);
    if (slot.state === 'readable') callPullIfNeeded(c, 'byte');
  }
  function publicReadResult(value, done) {
    return { value: value, done: done };
  }
  function readerRead(reader, view, options) {
    var info = currentReader(reader), slot = info.slot;
    if (info.kind === 'default') {
      if (!slot.stream) return Core.reject(typeError('Reader has been released'));
      var d = Core.deferred();
      readRequest(reader, {
        chunk: function (chunk) { d.resolve(publicReadResult(chunk, false)); },
        close: function () { d.resolve(publicReadResult(undefinedValue, true)); },
        error: function (reason) { d.reject(reason); }
      });
      return d.promise;
    }
    Core.viewInfo(view);
    var minimum = 1;
    if (options !== undefinedValue && options !== null) {
      var rawMin = options.min;
      if (rawMin !== undefinedValue) minimum = toEnforceRangeUnsignedLongLong(rawMin);
    }
    var vi = Core.viewInfo(view);
    if (vi.byteLength === 0 || Core.bufferByteLength(vi.buffer) === 0 || Core.isDetached(vi.buffer))
      return Core.reject(typeError('BYOB view must have a non-empty attached buffer'));
    if (minimum === 0) return Core.reject(typeError('min must be greater than zero'));
    if (minimum > (vi.kind === 'DataView' ? vi.byteLength : vi.length))
      return Core.reject(rangeError('min is greater than the view length'));
    if (!slot.stream) return Core.reject(typeError('Reader has been released'));
    var d2 = Core.deferred();
    readIntoRequest(reader, view, minimum, {
      chunk: function (value) { d2.resolve(publicReadResult(value, false)); },
      close: function (value) { d2.resolve(publicReadResult(value, true)); },
      error: function (reason) { d2.reject(reason); }
    });
    return d2.promise;
  }
  function toEnforceRangeUnsignedLongLong(value) {
    var number = +value;
    if (number !== number || number === Infinity || number === -Infinity)
      throw typeError('Value is outside the unsigned long long range');
    var absolute = number < 0 ? -number : number;
    var integer = number < 0 ? -(absolute - absolute % 1) : absolute - absolute % 1;
    if (integer < 0 || integer >= 18446744073709551616)
      throw typeError('Value is outside the unsigned long long range');
    return integer;
  }
  function readerCancel(reader, reason) {
    var info = currentReader(reader);
    if (!info.slot.stream) return Core.reject(typeError('Reader has been released'));
    return cancelStream(info.slot.stream, reason);
  }
  function readerRelease(reader) {
    var info = currentReader(reader), slot = info.slot;
    if (!slot.stream) return;
    var stream = slot.stream, ss = streamSlot(stream);
    if (ss && ss.controllerKind === 'byte') {
      var controller = mapGet(byteControllerSlots, ss.controller);
      for (var node = controller.pendingPullIntos.head; node; node = node.next) {
        if (node.value.request && node.value.readerType !== 'none') {
          node.value.readerType = 'none';
          node.value.request = undefinedValue;
        }
      }
    }
    if (ss && ss.reader === reader) ss.reader = undefinedValue;
    slot.stream = undefinedValue;
    var err = typeError('Reader lock was released');
    if (!slot.closedSettled) {
      slot.closedDeferred.reject(err);
    } else {
      slot.closedDeferred = Core.deferred();
      slot.closedDeferred.reject(err);
    }
    slot.closedSettled = true;
    Core.handled(slot.closedDeferred.promise);
    rejectPendingReads(slot, err);
  }
  function readerClosed(reader) { return currentReader(reader).slot.closedDeferred.promise; }
  function defaultReaderReadPublic() { try { return readerRead(this); } catch (e) { return Core.reject(e); } }
  function byobReaderReadPublic(view, options) { try { return readerRead(this, view, options); } catch (e) { return Core.reject(e); } }
  function defaultReaderCancelPublic(reason) { try { return readerCancel(this, reason); } catch (e) { return Core.reject(e); } }
  function defaultReaderReleasePublic() { return readerRelease(this); }
  function readableCancelPublic(reason) {
    var slot = streamSlot(this);
    if (!slot) return Core.reject(typeError('Invalid ReadableStream receiver'));
    if (slot.reader) return Core.reject(typeError('Cannot cancel a locked ReadableStream'));
    return cancelStream(this, reason);
  }
  function getReaderPublic(options) {
    if (!streamSlot(this)) throw typeError('Invalid ReadableStream receiver');
    var mode = 'default';
    if (options !== undefinedValue && options !== null) {
      var rawMode = options.mode;
      if (rawMode !== undefinedValue) mode = toDOMString(rawMode);
    }
    return acquireReader(this, mode);
  }
  function isReadable(stream) { return !!streamSlot(stream); }
  function isLocked(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); return !!s.reader; }
  function isDisturbed(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); return s.disturbed; }
  function isByteStream(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); return s.controllerKind === 'byte'; }
  function state(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); return s.state; }
  function storedError(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); return s.storedError; }
  function disturb(stream) { var s = streamSlot(stream); if (!s) throw typeError('Invalid ReadableStream receiver'); s.disturbed = true; }
  function defaultControllerClosePublic() {
    var c = mapGet(defaultControllerSlots, this);
    if (!c || !canCloseOrEnqueue(c.stream)) throw typeError('ReadableStreamDefaultController is not active');
    c.closeRequested = true;
    if (Core.length(c.queue) === 0) closeStream(c.stream);
  }
  function defaultControllerEnqueuePublic(chunk) {
    var c = mapGet(defaultControllerSlots, this);
    if (!c) throw typeError('Invalid ReadableStreamDefaultController receiver');
    defaultControllerEnqueue(c.stream, chunk);
  }
  function defaultControllerErrorPublic(reason) {
    var c = mapGet(defaultControllerSlots, this);
    if (!c || !canCloseOrEnqueue(c.stream)) return;
    errorStream(c.stream, reason);
  }
  function defaultControllerDesiredSize() {
    var c = mapGet(defaultControllerSlots, this);
    if (!c) throw typeError('Invalid ReadableStreamDefaultController receiver');
    return streamDesiredSize(c.stream);
  }
  function byteControllerClosePublic() {
    var c = mapGet(byteControllerSlots, this);
    if (!c || !canCloseOrEnqueue(c.stream)) throw typeError('ReadableByteStreamController is not active');
    closeByteController(c, false);
  }
  function byteControllerEnqueuePublic(chunk) {
    var c = mapGet(byteControllerSlots, this);
    if (!c) throw typeError('Invalid ReadableByteStreamController receiver');
    byteControllerEnqueue(c.stream, chunk, false);
  }
  function byteControllerErrorPublic(reason) {
    var c = mapGet(byteControllerSlots, this);
    if (!c || !canCloseOrEnqueue(c.stream)) return;
    errorStream(c.stream, reason);
  }
  function byteControllerDesiredSize() {
    var c = mapGet(byteControllerSlots, this);
    if (!c) throw typeError('Invalid ReadableByteStreamController receiver');
    return streamDesiredSize(c.stream);
  }
  function byobRequestView() {
    var s = mapGet(byobRequestSlots, this);
    if (!s) throw typeError('Invalid ReadableStreamBYOBRequest receiver');
    return s.controller ? s.view : null;
  }
  function byobRespond(bytesWritten) {
    var req = mapGet(byobRequestSlots, this);
    if (!req || !req.controller) throw typeError('BYOB request is invalid');
    var controller = mapGet(byteControllerSlots, req.controller);
    if (!controller) throw typeError('BYOB request is invalid');
    var descriptor = currentPullInto(controller);
    if (!descriptor) throw typeError('No pending BYOB read');
    var count = toEnforceRangeUnsignedLongLong(bytesWritten);
    respondByteController(controller, count, false);
  }
  function byobRespondWithNewView(view) {
    var req = mapGet(byobRequestSlots, this);
    if (!req || !req.controller) throw typeError('BYOB request is invalid');
    var controller = mapGet(byteControllerSlots, req.controller);
    var descriptor = controller && currentPullInto(controller);
    if (!descriptor) throw typeError('No pending BYOB read');
    var info = Core.viewInfo(view);
    if (Core.isDetached(info.buffer)) throw typeError('View buffer is detached');
    if (info.byteOffset !== descriptor.byteOffset + descriptor.bytesFilled) throw rangeError('View offset does not match BYOB request');
    if (Core.bufferByteLength(info.buffer) !== descriptor.bufferByteLength) throw rangeError('View buffer length does not match BYOB request');
    if (info.byteLength === 0 && streamSlot(controller.stream).state !== 'closed') throw typeError('View must not be empty');
    if (descriptor.bytesFilled + info.byteLength > descriptor.byteLength) throw rangeError('View exceeds BYOB request');
    var copiedBuffer = Core.transfer(info.buffer);
    descriptor.buffer = copiedBuffer;
    respondByteController(controller, info.byteLength, true);
  }
  function commitClosedPullIntos(controller) {
    var stream = controller.stream;
    var slot = streamSlot(stream);
    while (controller.pendingPullIntos.length) {
      var descriptor = shiftPullInto(controller);
      descriptor.buffer = Core.transfer(descriptor.buffer);
      descriptor.bufferByteLength = Core.bufferByteLength(descriptor.buffer);
      if (descriptor.readerType === 'none') {
        if (descriptor.bytesFilled > 0)
          byteEntryPrepend(controller, descriptorView(descriptor, descriptor.bytesFilled));
        continue;
      }
      if (controller.queue.head && descriptor.bytesFilled < descriptor.byteLength)
        copyQueueIntoDescriptor(controller, descriptor);
      removeDescriptorRequest(stream, descriptor);
      if (descriptor.bytesFilled > 0) {
        var aligned = descriptor.bytesFilled - (descriptor.bytesFilled % descriptor.elementSize);
        if (descriptor.readerType === 'default')
          resolveReadRequest(descriptor.request, 'chunk', descriptorView(descriptor, aligned));
        else
          resolveReadRequest(descriptor.request, 'chunk', descriptorView(descriptor, aligned));
      } else if (descriptor.readerType === 'default') {
        resolveReadRequest(descriptor.request, 'close', undefinedValue);
      } else {
        resolveReadRequest(descriptor.request, 'close', descriptorView(descriptor, 0));
      }
    }
    if (slot.reader && mapGet(defaultReaderSlots, slot.reader)) {
      var reader = mapGet(defaultReaderSlots, slot.reader);
      while (controller.queue.head && reader.readRequests.length) {
        var request = queueShift(reader.readRequests);
        var entry = byteEntryShift(controller), info = Core.viewInfo(entry.view);
        resolveReadRequest(request, 'chunk', Core.view('Uint8Array', info.buffer,
          info.byteOffset + entry.offset, entry.length - entry.offset));
      }
      while (reader.readRequests.length) resolveReadRequest(queueShift(reader.readRequests), 'close', undefinedValue);
    }
  }
  function respondByteController(controller, bytesWritten, alreadyTransferred) {
    var stream = streamSlot(controller.stream), descriptor = currentPullInto(controller);
    if (!descriptor) throw typeError('No pending BYOB read');
    if (stream.state === 'closed') {
      if (bytesWritten !== 0) throw typeError('Cannot respond with bytes after close');
    } else if (controller.closeRequested) {
      if (bytesWritten !== 0) throw typeError('Cannot respond with bytes after close was requested');
    } else {
      if (bytesWritten === 0) throw typeError('A readable byte stream must respond with at least one byte');
      if (descriptor.bytesFilled + bytesWritten > descriptor.byteLength) throw rangeError('BYOB response is too large');
    }
    if (Core.isDetached(descriptor.buffer)) throw typeError('BYOB buffer is detached');
    invalidateBYOBRequest(controller);
    if (!alreadyTransferred) descriptor.buffer = Core.transfer(descriptor.buffer);
    descriptor.bufferByteLength = Core.bufferByteLength(descriptor.buffer);
    descriptor.bytesFilled += bytesWritten;
    var aligned = descriptor.bytesFilled - (descriptor.bytesFilled % descriptor.elementSize);
    if (stream.state === 'closed') {
      commitClosedPullIntos(controller);
    } else if (controller.closeRequested && bytesWritten === 0) {
      queueShift(controller.pendingPullIntos);
      removeDescriptorRequest(controller.stream, descriptor);
      if (descriptor.readerType === 'none') {
        if (descriptor.bytesFilled > 0)
          byteEntryPrepend(controller, descriptorView(descriptor, descriptor.bytesFilled));
      } else if (descriptor.bytesFilled > 0) {
        resolveReadRequest(descriptor.request, 'chunk', descriptorView(descriptor, aligned));
      } else {
        resolveReadRequest(descriptor.request, 'close', descriptorView(descriptor, 0));
      }
      processByteQueue(controller);
    } else if (aligned >= descriptor.minimumFill) {
      queueShift(controller.pendingPullIntos);
      removeDescriptorRequest(controller.stream, descriptor);
      var remainder = descriptor.bytesFilled - aligned;
      if (descriptor.readerType === 'none') {
        if (descriptor.bytesFilled > 0)
          byteEntryPrepend(controller, descriptorView(descriptor, descriptor.bytesFilled));
      } else {
        if (remainder > 0) {
          var tail = Core.view('Uint8Array', descriptor.buffer, descriptor.byteOffset + aligned, remainder);
          appendByteEntry(controller, tail);
        }
        var resultView = descriptorView(descriptor, aligned);
        resolveReadRequest(descriptor.request, 'chunk', resultView);
      }
      processByteQueue(controller);
    }
    callPullIfNeeded(controller, 'byte');
  }
  function controllerByobRequest() {
    var c = mapGet(byteControllerSlots, this);
    if (!c) throw typeError('Invalid ReadableByteStreamController receiver');
    return getBYOBRequest(c);
  }
  function readableLockedGetter() { return isLocked(this); }
  function defaultReaderClosedGetter() { return readerClosed(this); }
  function byobReaderClosedGetter() { return readerClosed(this); }
  function publicTee() { return tee(this, false, null); }
  function tee(stream, cloneForBranch2, cloneChunk) {
    var ss = streamSlot(stream);
    if (!ss) throw typeError('Invalid ReadableStream receiver');
    if (ss.reader) throw typeError('ReadableStream is locked');
    cloneForBranch2 = !!cloneForBranch2;
    if (ss.controllerKind === 'byte') return byteTee(stream);
    return defaultTee(stream, cloneForBranch2, cloneChunk);
  }
  function defaultTee(stream, cloneForBranch2, cloneChunk) {
    var reader = acquireReader(stream, 'default');
    var reading = false, readAgain = false, canceled1 = false, canceled2 = false;
    var reason1, reason2, cancelDeferred = Core.deferred();
    var branch1, branch2;
    function cancelBranch(which, reason) {
      if (which === 1) { canceled1 = true; reason1 = reason; }
      else { canceled2 = true; reason2 = reason; }
      if (canceled1 && canceled2) {
        Core.react(cancelStream(stream, [reason1, reason2]), function (v) { cancelDeferred.resolve(v); return undefinedValue; },
          function (e) { cancelDeferred.reject(e); return undefinedValue; });
      }
      return cancelDeferred.promise;
    }
    function pull() {
      if (reading) { readAgain = true; return Core.resolve(undefinedValue); }
      reading = true;
      readRequest(reader, {
        chunk: function (chunk) {
          var value2 = chunk;
          var work = function () {
            readAgain = false;
            if (!canceled2 && cloneForBranch2) {
              try { value2 = cloneChunk ? call(cloneChunk, undefinedValue, [chunk]) : chunk; }
              catch (cloneError) {
                errorStream(branch1, cloneError); errorStream(branch2, cloneError);
                Core.react(cancelStream(stream, cloneError), function (x) { cancelDeferred.resolve(x); return undefinedValue; },
                  function (e) { cancelDeferred.reject(e); return undefinedValue; });
                reading = false;
                return undefinedValue;
              }
            }
            if (!canceled1) enqueue(branch1, chunk);
            if (!canceled2) enqueue(branch2, value2);
            reading = false;
            if (readAgain) { readAgain = false; pull(); }
            return undefinedValue;
          };
          Core.react(Core.resolve(undefinedValue), work, work);
        },
        close: function () {
          reading = false;
          if (!canceled1) close(branch1);
          if (!canceled2) close(branch2);
          if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
        },
        error: function (reason) {
          reading = false;
          if (!canceled1) error(branch1, reason);
          if (!canceled2) error(branch2, reason);
          if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
        }
      });
      return Core.resolve(undefinedValue);
    }
    var algorithms1 = Core.record(); algorithms1.start = null; algorithms1.pull = pull;
    algorithms1.cancel = function (reason) { return cancelBranch(1, reason); };
    var algorithms2 = Core.record(); algorithms2.start = null; algorithms2.pull = pull;
    algorithms2.cancel = function (reason) { return cancelBranch(2, reason); };
    var strategy1 = Core.record(); strategy1.highWaterMark = 1; strategy1.sizeAlgorithm = function () { return 1; };
    var strategy2 = Core.record(); strategy2.highWaterMark = 1; strategy2.sizeAlgorithm = function () { return 1; };
    branch1 = createDefault(algorithms1, strategy1);
    branch2 = createDefault(algorithms2, strategy2);
    Core.react(readerClosed(reader), function () { return undefinedValue; }, function (reason) {
      if (!canceled1) error(branch1, reason);
      if (!canceled2) error(branch2, reason);
      if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
      return undefinedValue;
    });
    return [branch1, branch2];
  }
  function byteTee(stream) {
    var reader = acquireReader(stream, 'default');
    var reading = false, readAgain1 = false, readAgain2 = false;
    var canceled1 = false, canceled2 = false, reason1, reason2;
    var cancelDeferred = Core.deferred();
    var branch1, branch2;
    function settleCancel() {
      if (canceled1 && canceled2) {
        Core.react(cancelStream(stream, [reason1, reason2]), function (v) { cancelDeferred.resolve(v); return undefinedValue; },
          function (e) { cancelDeferred.reject(e); return undefinedValue; });
      }
    }
    function cancelBranch(which, reason) {
      if (which === 1) { canceled1 = true; reason1 = reason; } else { canceled2 = true; reason2 = reason; }
      settleCancel();
      return cancelDeferred.promise;
    }
    function forwardError(r) {
      if (!canceled1) error(branch1, r);
      if (!canceled2) error(branch2, r);
      if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
    }
    function forwardReaderError(thisReader) {
      Core.react(readerClosed(thisReader), function () { return undefinedValue; }, function (r) {
        if (thisReader === reader) forwardError(r);
        return undefinedValue;
      });
    }
    forwardReaderError(reader);
    function pullWithDefault() {
      if (readerIsBYOB(reader)) {
        var old = reader;
        readerRelease(old);
        reader = acquireReader(stream, 'default');
        forwardReaderError(reader);
      }
      readRequest(reader, {
        chunk: function (chunk) {
          var run = function () {
            readAgain1 = false; readAgain2 = false;
            var clone;
            try { clone = Core.copyBytes(chunk); }
            catch (e) { error(branch1, e); error(branch2, e); Core.react(cancelStream(stream, e), function () {}, function () {}); reading = false; return undefinedValue; }
            if (!canceled1) internalByteEnqueue(branch1, chunk);
            if (!canceled2) internalByteEnqueue(branch2, clone);
            reading = false;
            if (readAgain1) { readAgain1 = false; pull1(); }
            else if (readAgain2) { readAgain2 = false; pull2(); }
            return undefinedValue;
          };
          Core.react(Core.resolve(undefinedValue), run, run);
        },
        close: function () {
          reading = false;
          if (!canceled1) internalByteClose(branch1);
          if (!canceled2) internalByteClose(branch2);
          var c1 = mapGet(byteControllerSlots, streamSlot(branch1).controller);
          var c2 = mapGet(byteControllerSlots, streamSlot(branch2).controller);
          if (c1.pendingPullIntos.length) respondByteController(c1, 0, false);
          if (c2.pendingPullIntos.length) respondByteController(c2, 0, false);
          if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
        },
        error: function (r) { reading = false; forwardError(r); }
      });
    }
    function pullWithBYOB(view, forBranch2) {
      if (!readerIsBYOB(reader)) {
        var old = reader;
        readerRelease(old);
        reader = acquireReader(stream, 'byob');
        forwardReaderError(reader);
      }
      function byobCanceled() { return forBranch2 ? canceled2 : canceled1; }
      function otherCanceled() { return forBranch2 ? canceled1 : canceled2; }
      readIntoRequest(reader, view, 1, {
        chunk: function (chunk) {
          var run = function () {
            readAgain1 = false; readAgain2 = false;
            if (!otherCanceled()) {
              var clone;
              try { clone = Core.copyBytes(chunk); }
              catch (e) {
                error(branch1, e); error(branch2, e);
                Core.react(cancelStream(stream, e), function () { return undefinedValue; }, function () { return undefinedValue; });
                reading = false;
                return undefinedValue;
              }
              if (!byobCanceled()) respondWithNewView(forBranch2 ? branch2 : branch1, chunk);
              internalByteEnqueue(forBranch2 ? branch1 : branch2, clone);
            } else if (!byobCanceled()) respondWithNewView(forBranch2 ? branch2 : branch1, chunk);
            reading = false;
            if (readAgain1) { readAgain1 = false; pull1(); }
            else if (readAgain2) { readAgain2 = false; pull2(); }
            return undefinedValue;
          };
          Core.react(Core.resolve(undefinedValue), run, run);
        },
        close: function (emptyView) {
          reading = false;
          if (!byobCanceled()) internalByteClose(forBranch2 ? branch2 : branch1);
          if (!otherCanceled()) internalByteClose(forBranch2 ? branch1 : branch2);
          if (emptyView !== undefinedValue) {
            if (!byobCanceled()) respondWithNewView(forBranch2 ? branch2 : branch1, emptyView);
            var otherController = mapGet(byteControllerSlots, streamSlot(forBranch2 ? branch1 : branch2).controller);
            if (!otherCanceled() && otherController.pendingPullIntos.length)
              respondByteController(otherController, 0, false);
          }
          if (!canceled1 || !canceled2) cancelDeferred.resolve(undefinedValue);
        },
        error: function (r) { reading = false; forwardError(r); }
      });
    }
    function respondWithNewView(streamBranch, view) {
      var slot = streamSlot(streamBranch), c = mapGet(byteControllerSlots, slot.controller);
      var d = currentPullInto(c);
      if (!d) throw typeError('No pending branch BYOB request');
      var info = Core.viewInfo(view);
      if (Core.isDetached(info.buffer)) throw typeError('Branch BYOB view is detached');
      if (info.byteOffset !== d.byteOffset + d.bytesFilled || Core.bufferByteLength(info.buffer) !== d.bufferByteLength)
        throw rangeError('Branch BYOB view does not match the pending request');
      d.buffer = Core.transfer(info.buffer);
      respondByteController(c, info.byteLength, true);
    }
    function pull1() {
      if (reading) { readAgain1 = true; return Core.resolve(undefinedValue); }
      reading = true;
      var c = mapGet(byteControllerSlots, streamSlot(branch1).controller);
      var request = getBYOBRequest(c);
      if (request) pullWithBYOB(mapGet(byobRequestSlots, request).view, false);
      else pullWithDefault();
      return Core.resolve(undefinedValue);
    }
    function pull2() {
      if (reading) { readAgain2 = true; return Core.resolve(undefinedValue); }
      reading = true;
      var c = mapGet(byteControllerSlots, streamSlot(branch2).controller);
      var request = getBYOBRequest(c);
      if (request) pullWithBYOB(mapGet(byobRequestSlots, request).view, true);
      else pullWithDefault();
      return Core.resolve(undefinedValue);
    }
    var byteStrategy = Core.record(); byteStrategy.highWaterMark = 0; byteStrategy.sizeAlgorithm = function () { return 1; };
    var byteAlgorithms1 = Core.record(); byteAlgorithms1.start = null; byteAlgorithms1.pull = pull1;
    byteAlgorithms1.cancel = function (reason) { return cancelBranch(1, reason); };
    var byteAlgorithms2 = Core.record(); byteAlgorithms2.start = null; byteAlgorithms2.pull = pull2;
    byteAlgorithms2.cancel = function (reason) { return cancelBranch(2, reason); };
    branch1 = createByte(byteAlgorithms1, byteStrategy, 0);
    branch2 = createByte(byteAlgorithms2, byteStrategy, 0);
    return [branch1, branch2];
  }

  function ReadableStream(underlyingSource, strategy) {
    if (!new.target) throw typeError("ReadableStream constructor requires 'new'");
    if (underlyingSource !== undefinedValue &&
        (underlyingSource === null || (typeof underlyingSource !== 'object' && typeof underlyingSource !== 'function')))
      throw typeError('underlyingSource must be an object');
    var source = underlyingSource === undefinedValue ? nullRecord() : underlyingSource;
    var convertedStrategy = Core.convertStrategy(strategy);
    var rawAutoAllocate = source.autoAllocateChunkSize;
    var auto = rawAutoAllocate == null ? 0 : toEnforceRangeUnsignedLongLong(rawAutoAllocate);
    var cancel = source.cancel;
    if (cancel !== undefinedValue && typeof cancel !== 'function') throw typeError('cancel must be callable');
    var pull = source.pull;
    if (pull !== undefinedValue && typeof pull !== 'function') throw typeError('pull must be callable');
    var start = source.start;
    if (start !== undefinedValue && typeof start !== 'function') throw typeError('start must be callable');
    var type = source.type;
    var byte = false;
    if (type !== undefinedValue && type !== null) {
      type = toDOMString(type);
      if (type === 'bytes') byte = true;
      else throw typeError('Invalid underlying source type');
    }
    var normalized;
    if (byte) {
      if (convertedStrategy.size !== undefinedValue) throw rangeError('Byte streams cannot have a size algorithm');
      normalized = Core.record(); normalized.highWaterMark = Core.extractHWM(convertedStrategy, 0); normalized.sizeAlgorithm = null;
    } else {
      var sizeAlgorithm = Core.extractSize(convertedStrategy);
      var highWaterMark = Core.extractHWM(convertedStrategy, 1);
      normalized = Core.record(); normalized.highWaterMark = highWaterMark; normalized.sizeAlgorithm = sizeAlgorithm;
    }
    var algorithms = Core.record();
    algorithms.start = start === undefinedValue ? null : function (controller) { return call(start, source, [controller]); };
    algorithms.pull = pull === undefinedValue ? null : function (controller) { return call(pull, source, [controller]); };
    algorithms.cancel = cancel === undefinedValue ? null : function (reason) { return call(cancel, source, [reason]); };
    if (byte && rawAutoAllocate !== undefinedValue && auto === 0)
      throw rangeError('autoAllocateChunkSize must be greater than zero');
    initializePublicStream(this, algorithms, normalized, byte, auto);
  }

  function ReadableStreamDefaultReader(stream) {
    if (!new.target) throw typeError("ReadableStreamDefaultReader constructor requires 'new'");
    setupReader(this, stream, 'default');
  }
  function ReadableStreamBYOBReader(stream) {
    if (!new.target) throw typeError("ReadableStreamBYOBReader constructor requires 'new'");
    setupReader(this, stream, 'byob');
  }
  function ReadableStreamDefaultController() { illegalConstructor('ReadableStreamDefaultController'); }
  function ReadableByteStreamController() { illegalConstructor('ReadableByteStreamController'); }
  function ReadableStreamBYOBRequest() { illegalConstructor('ReadableStreamBYOBRequest'); }

  ReadableStream.prototype = Core.create(Core.objectPrototype);
  ReadableStreamDefaultReader.prototype = Core.create(Core.objectPrototype);
  ReadableStreamBYOBReader.prototype = Core.create(Core.objectPrototype);
  ReadableStreamDefaultController.prototype = Core.create(Core.objectPrototype);
  ReadableByteStreamController.prototype = Core.create(Core.objectPrototype);
  ReadableStreamBYOBRequest.prototype = Core.create(Core.objectPrototype);
  readableStreamPrototype = ReadableStream.prototype;
  defaultReaderPrototype = ReadableStreamDefaultReader.prototype;
  byobReaderPrototype = ReadableStreamBYOBReader.prototype;
  defaultControllerPrototype = ReadableStreamDefaultController.prototype;
  byteControllerPrototype = ReadableByteStreamController.prototype;
  byobRequestPrototype = ReadableStreamBYOBRequest.prototype;
  Core.define(ReadableStream, 'length', { value: 0, configurable: true });
  Core.define(ReadableStream, 'prototype', { value: readableStreamPrototype, writable: false });
  Core.define(ReadableStreamDefaultReader, 'prototype', { value: defaultReaderPrototype, writable: false });
  Core.define(ReadableStreamBYOBReader, 'prototype', { value: byobReaderPrototype, writable: false });
  Core.define(ReadableStreamDefaultController, 'prototype', { value: defaultControllerPrototype, writable: false });
  Core.define(ReadableByteStreamController, 'prototype', { value: byteControllerPrototype, writable: false });
  Core.define(ReadableStreamBYOBRequest, 'prototype', { value: byobRequestPrototype, writable: false });
  Core.define(ReadableStream.prototype, 'constructor', { value: ReadableStream, writable: true, configurable: true, enumerable: false });
  own(ReadableStreamDefaultReader.prototype, 'constructor', ReadableStreamDefaultReader);
  own(ReadableStreamBYOBReader.prototype, 'constructor', ReadableStreamBYOBReader);
  own(ReadableStreamDefaultController.prototype, 'constructor', ReadableStreamDefaultController);
  own(ReadableByteStreamController.prototype, 'constructor', ReadableByteStreamController);
  own(ReadableStreamBYOBRequest.prototype, 'constructor', ReadableStreamBYOBRequest);
  defineGetter(ReadableStream.prototype, 'locked', readableLockedGetter);
  defineMethod(ReadableStream.prototype, 'getReader', getReaderPublic, 0, true);
  defineMethod(ReadableStream.prototype, 'cancel', readableCancelPublic, 0, true);
  defineMethod(ReadableStream.prototype, 'tee', publicTee, 0, true);
  defineGetter(ReadableStreamDefaultReader.prototype, 'closed', defaultReaderClosedGetter);
  defineMethod(ReadableStreamDefaultReader.prototype, 'cancel', defaultReaderCancelPublic, 0, true);
  defineMethod(ReadableStreamDefaultReader.prototype, 'read', defaultReaderReadPublic, 0, true);
  defineMethod(ReadableStreamDefaultReader.prototype, 'releaseLock', defaultReaderReleasePublic, 0, true);
  defineGetter(ReadableStreamBYOBReader.prototype, 'closed', byobReaderClosedGetter);
  defineMethod(ReadableStreamBYOBReader.prototype, 'cancel', defaultReaderCancelPublic, 0, true);
  defineMethod(ReadableStreamBYOBReader.prototype, 'read', byobReaderReadPublic, 1, true);
  defineMethod(ReadableStreamBYOBReader.prototype, 'releaseLock', defaultReaderReleasePublic, 0, true);
  defineGetter(ReadableStreamDefaultController.prototype, 'desiredSize', defaultControllerDesiredSize);
  defineMethod(ReadableStreamDefaultController.prototype, 'close', defaultControllerClosePublic, 0, true);
  defineMethod(ReadableStreamDefaultController.prototype, 'enqueue', defaultControllerEnqueuePublic, 0, true);
  defineMethod(ReadableStreamDefaultController.prototype, 'error', defaultControllerErrorPublic, 0, true);
  defineGetter(ReadableByteStreamController.prototype, 'byobRequest', controllerByobRequest);
  defineGetter(ReadableByteStreamController.prototype, 'desiredSize', byteControllerDesiredSize);
  defineMethod(ReadableByteStreamController.prototype, 'close', byteControllerClosePublic, 0, true);
  defineMethod(ReadableByteStreamController.prototype, 'enqueue', byteControllerEnqueuePublic, 0, true);
  defineMethod(ReadableByteStreamController.prototype, 'error', byteControllerErrorPublic, 0, true);
  defineGetter(ReadableStreamBYOBRequest.prototype, 'view', byobRequestView);
  defineMethod(ReadableStreamBYOBRequest.prototype, 'respond', byobRespond, 1, true);
  defineMethod(ReadableStreamBYOBRequest.prototype, 'respondWithNewView', byobRespondWithNewView, 1, true);
  var toStringTag = SymbolCtor && SymbolCtor.toStringTag;
  if (toStringTag) {
    Core.define(ReadableStream.prototype, toStringTag, { value: 'ReadableStream', configurable: true });
    Core.define(ReadableStreamDefaultReader.prototype, toStringTag, { value: 'ReadableStreamDefaultReader', configurable: true });
    Core.define(ReadableStreamBYOBReader.prototype, toStringTag, { value: 'ReadableStreamBYOBReader', configurable: true });
    Core.define(ReadableStreamDefaultController.prototype, toStringTag, { value: 'ReadableStreamDefaultController', configurable: true });
    Core.define(ReadableByteStreamController.prototype, toStringTag, { value: 'ReadableByteStreamController', configurable: true });
    Core.define(ReadableStreamBYOBRequest.prototype, toStringTag, { value: 'ReadableStreamBYOBRequest', configurable: true });
  }
  Core.define(globalThis, 'ReadableStream', { value: ReadableStream, configurable: true, writable: true, enumerable: false });
  Core.define(globalThis, 'ReadableStreamDefaultReader', { value: ReadableStreamDefaultReader, configurable: true, writable: true, enumerable: false });
  Core.define(globalThis, 'ReadableStreamBYOBReader', { value: ReadableStreamBYOBReader, configurable: true, writable: true, enumerable: false });
  Core.define(globalThis, 'ReadableStreamDefaultController', { value: ReadableStreamDefaultController, configurable: true, writable: true, enumerable: false });
  Core.define(globalThis, 'ReadableByteStreamController', { value: ReadableByteStreamController, configurable: true, writable: true, enumerable: false });
  Core.define(globalThis, 'ReadableStreamBYOBRequest', { value: ReadableStreamBYOBRequest, configurable: true, writable: true, enumerable: false });

  function validatePullIntoView(view, min) {
    var info = Core.viewInfo(view);
    if (info.byteLength === 0 || Core.bufferByteLength(info.buffer) === 0 || Core.isDetached(info.buffer))
      throw typeError('BYOB view must be non-empty and attached');
    if (min === 0) throw typeError('min must be greater than zero');
    if (min > (info.kind === 'DataView' ? info.byteLength : info.length)) throw rangeError('min exceeds view length');
    return info;
  }
  function initializePublicStream(receiver, algorithms, strategy, isByte, auto) {
    var slot = Core.record();
    slot.state = 'readable'; slot.reader = undefinedValue; slot.disturbed = false;
    slot.storedError = undefinedValue; slot.controller = undefinedValue;
    slot.controllerKind = isByte ? 'byte' : 'default'; slot.cancelled = false;
    mapSet(streamSlots, receiver, slot);
    var controller = isByte
      ? makeByteController(receiver, algorithms, strategy, auto)
      : makeDefaultController(receiver, algorithms, strategy);
    invokeStart(receiver, algorithms, controller);
  }
  var ops = Core.record();
  ops.constructor = ReadableStream;
  ops.ReadableStream = ReadableStream;
  ops.ReadableStreamDefaultReader = ReadableStreamDefaultReader;
  ops.ReadableStreamBYOBReader = ReadableStreamBYOBReader;
  ops.ReadableStreamDefaultController = ReadableStreamDefaultController;
  ops.ReadableByteStreamController = ReadableByteStreamController;
  ops.ReadableStreamBYOBRequest = ReadableStreamBYOBRequest;
  ops.isReadable = isReadable; ops.isLocked = isLocked; ops.isDisturbed = isDisturbed;
  ops.isByteStream = isByteStream; ops.state = state; ops.storedError = storedError; ops.disturb = disturb;
  ops.createDefault = createDefault; ops.createByte = createByte;
  ops.enqueue = enqueue; ops.close = close; ops.error = error;
  ops.canCloseOrEnqueue = canCloseOrEnqueue; ops.desiredSize = streamDesiredSize;
  ops.acquireReader = acquireReader; ops.readerRead = readerRead; ops.readerClosed = readerClosed;
  ops.readerCancel = readerCancel; ops.readerRelease = readerRelease; ops.cancel = cancelStream;
  ops.readRequest = readRequest; ops.readIntoRequest = readIntoRequest;
  ops.tee = tee; ops.internalByteClose = internalByteClose; ops.internalByteEnqueue = internalByteEnqueue;
  return ops;
})(Core);
