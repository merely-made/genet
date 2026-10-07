// Writable and transform streams. This file is assembled inside the private
// Streams bootstrap, after Core and ReadableOps have been initialized.
var WritableOps = (function (Core, ReadableOps) {
  'use strict';

  var writableSlots = Core.sharedMap('writableSlots');
  var writerSlots = Core.sharedMap('defaultWriterSlots');
  var controllerSlots = Core.sharedMap('writableControllerSlots');
  var transformSlots = Core.sharedMap('transformSlots');
  var transformControllerSlots = Core.sharedMap('transformControllerSlots');
  var closeSentinel = Core.record();

  function rejected(error) {
    var result = deferred();
    result.reject(error);
    return result.promise;
  }
  function resolved(value) {
    var result = deferred();
    result.resolve(value);
    return result.promise;
  }
  function typeError(message) { return new Core.TypeError(message); }
  function rangeError(message) { return new Core.RangeError(message); }
  function deferred() {
    var inner = Core.deferred();
    var result = Core.record();
    result.promise = inner.promise;
    result.state = 'pending';
    result.resolve = function (value) {
      if (result.state !== 'pending') return;
      result.state = 'fulfilled';
      inner.resolve(value);
    };
    result.reject = function (reason) {
      if (result.state !== 'pending') return;
      result.state = 'rejected';
      inner.reject(reason);
    };
    return result;
  }
  function extractWritableStrategy(converted, defaultHWM) {
    var sizeAlgorithm = Core.extractSize(converted);
    var highWaterMark = Core.extractHWM(converted, defaultHWM);
    var result = Core.record();
    result.sizeAlgorithm = sizeAlgorithm;
    result.highWaterMark = highWaterMark;
    return result;
  }
  function extractTransformStrategy(converted, defaultHWM) {
    var highWaterMark = Core.extractHWM(converted, defaultHWM);
    var sizeAlgorithm = Core.extractSize(converted);
    var result = Core.record();
    result.sizeAlgorithm = sizeAlgorithm;
    result.highWaterMark = highWaterMark;
    return result;
  }
  function queueItem(chunk) {
    var item = Core.record();
    item.chunk = chunk;
    return item;
  }

  function streamSlot(stream) {
    var slot = Core.get(writableSlots, stream);
    if (slot === undefined) throw typeError('WritableStream method called on an incompatible receiver');
    return slot;
  }
  function writerSlot(writer) {
    var slot = Core.get(writerSlots, writer);
    if (slot === undefined) throw typeError('WritableStreamDefaultWriter method called on an incompatible receiver');
    return slot;
  }
  function controllerSlot(controller) {
    var slot = Core.get(controllerSlots, controller);
    if (slot === undefined) throw typeError('WritableStreamDefaultController method called on an incompatible receiver');
    return slot;
  }
  function transformSlot(stream) {
    var slot = Core.get(transformSlots, stream);
    if (slot === undefined) throw typeError('TransformStream method called on an incompatible receiver');
    return slot;
  }
  function transformControllerSlot(controller) {
    var slot = Core.get(transformControllerSlots, controller);
    if (slot === undefined) throw typeError('TransformStreamDefaultController method called on an incompatible receiver');
    return slot;
  }

  function isClosingSlot(slot) {
    return slot.closeRequest !== undefined || slot.inFlightCloseRequest !== undefined;
  }
  function isClosing(stream) { return isClosingSlot(streamSlot(stream)); }
  function isWritable(stream) { return Core.has(writableSlots, stream); }
  function isLocked(stream) { return streamSlot(stream).writer !== undefined; }

  function writerEnsureReadyRejected(writer, error) {
    var slot = writerSlot(writer);
    if (slot.ready.state !== 'pending') slot.ready = deferred();
    slot.ready.reject(error);
    Core.handled(slot.ready.promise);
  }
  function writerEnsureClosedRejected(writer, error) {
    var slot = writerSlot(writer);
    if (slot.closed.state !== 'pending') slot.closed = deferred();
    slot.closed.reject(error);
    Core.handled(slot.closed.promise);
  }
  function writerResolveReady(writer) {
    var slot = writerSlot(writer);
    if (slot.ready.state === 'pending') slot.ready.resolve(undefined);
  }
  function writerResolveClosed(writer) {
    var slot = writerSlot(writer);
    if (slot.closed.state === 'pending') slot.closed.resolve(undefined);
  }
  function writerResolveClosing(writer) {
    var slot = writerSlot(writer);
    if (slot.closing.state === 'pending') slot.closing.resolve(undefined);
  }
  function resolveWriterClosingForStream(stream) {
    var s = streamSlot(stream);
    if (s.writer !== undefined) writerResolveClosing(s.writer);
  }

  function updateBackpressure(stream, backpressure) {
    var slot = streamSlot(stream);
    if (slot.state !== 'writable' || isClosingSlot(slot) || slot.backpressure === backpressure) return;
    slot.backpressure = backpressure;
    if (slot.writer !== undefined) {
      if (backpressure) {
        writerSlot(slot.writer).ready = deferred();
      } else {
        writerResolveReady(slot.writer);
      }
    }
  }

  function controllerDesiredSize(controller) {
    var c = controllerSlot(controller);
    return c.strategyHWM - c.queue.totalSize;
  }
  function writerDesiredSize(writer) {
    var w = writerSlot(writer);
    if (w.stream === undefined) throw typeError('Writer has been released');
    var stream = streamSlot(w.stream);
    if (stream.state === 'errored' || stream.state === 'erroring') return null;
    if (stream.state === 'closed') return 0;
    return controllerDesiredSize(stream.controller);
  }
  function updateControllerBackpressure(controller) {
    var desired = controllerDesiredSize(controller);
    var stream = controllerSlot(controller).stream;
    if (!isClosing(stream) && streamSlot(stream).state === 'writable') {
      updateBackpressure(stream, desired <= 0);
    }
  }

  function rejectRequests(queue, reason) {
    while (Core.length(queue) > 0) {
      var request = Core.shift(queue);
      request.reject(reason);
    }
  }

  function clearControllerAlgorithms(controller) {
    var c = controllerSlot(controller);
    c.writeAlgorithm = undefined;
    c.closeAlgorithm = undefined;
    c.abortAlgorithm = undefined;
    c.sizeAlgorithm = undefined;
  }

  function startErroring(stream, reason) {
    var s = streamSlot(stream);
    if (s.state !== 'writable') return;
    s.state = 'erroring';
    s.storedError = reason;
    if (s.writer !== undefined) writerEnsureReadyRejected(s.writer, reason);
    if (s.inFlightWriteRequest === undefined && s.inFlightCloseRequest === undefined &&
        controllerSlot(s.controller).started) finishErroring(stream);
  }

  function finishErroring(stream) {
    var s = streamSlot(stream);
    if (s.state !== 'erroring' || s.inFlightWriteRequest !== undefined ||
        s.inFlightCloseRequest !== undefined) return;
    s.state = 'errored';
    resolveWriterClosingForStream(stream);
    var c = controllerSlot(s.controller);
    Core.clear(c.queue);
    rejectRequests(s.writeRequests, s.storedError);
    if (s.closeRequest !== undefined) {
      s.closeRequest.reject(s.storedError);
      s.closeRequest = undefined;
    }
    if (s.writer !== undefined) writerEnsureClosedRejected(s.writer, s.storedError);

    var abortRequest = s.pendingAbortRequest;
    if (abortRequest === undefined) return;
    s.pendingAbortRequest = undefined;
    if (abortRequest.wasAlreadyErroring) {
      clearControllerAlgorithms(s.controller);
      abortRequest.deferred.reject(s.storedError);
      return;
    }
    var abortResult;
    var abortAlgorithm = c.abortAlgorithm;
    clearControllerAlgorithms(s.controller);
    try {
      abortResult = abortAlgorithm === undefined ? undefined : abortAlgorithm(abortRequest.reason);
    } catch (error) {
      abortRequest.deferred.reject(error);
      return;
    }
    observe(abortResult, function () { abortRequest.deferred.resolve(undefined); },
      function (error) { abortRequest.deferred.reject(error); });
  }

  function dealWithRejection(stream, reason) {
    var s = streamSlot(stream);
    if (s.state === 'writable') {
      startErroring(stream, reason);
      return;
    }
    if (s.state === 'erroring') finishErroring(stream);
  }

  function observe(value, onFulfilled, onRejected) {
    var promise;
    try { promise = Core.isInternalPromise(value) ? value : Core.resolve(value); }
    catch (error) { onRejected(error); return; }
    Core.react(promise, function (result) {
      onFulfilled(result);
      return undefined;
    }, function (reason) {
      onRejected(reason);
      return undefined;
    });
  }

  function processWrite(controller) {
    var c = controllerSlot(controller);
    var stream = c.stream;
    var s = streamSlot(stream);
    var chunk = Core.peek(c.queue).chunk;
    var request = Core.shift(s.writeRequests);
    s.inFlightWriteRequest = request;
    var result;
    try {
      result = c.writeAlgorithm === undefined ? undefined : c.writeAlgorithm(chunk);
    } catch (error) {
      finishInFlightWriteWithError(stream, error);
      return;
    }
    observe(result, function () { finishInFlightWrite(stream); },
      function (error) { finishInFlightWriteWithError(stream, error); });
  }

  function processClose(controller) {
    var c = controllerSlot(controller);
    var stream = c.stream;
    var s = streamSlot(stream);
    s.inFlightCloseRequest = s.closeRequest;
    s.closeRequest = undefined;
    Core.shift(c.queue);
    var closeAlgorithm = c.closeAlgorithm;
    clearControllerAlgorithms(controller);
    var result;
    try { result = closeAlgorithm === undefined ? undefined : closeAlgorithm(); }
    catch (error) { finishInFlightCloseWithError(stream, error); return; }
    observe(result, function () { finishInFlightClose(stream); },
      function (error) { finishInFlightCloseWithError(stream, error); });
  }

  function advanceQueue(controller) {
    var c = controllerSlot(controller);
    var stream = c.stream;
    var s = streamSlot(stream);
    if (!c.started || s.inFlightWriteRequest !== undefined || s.inFlightCloseRequest !== undefined) return;
    if (s.state === 'erroring') { finishErroring(stream); return; }
    if (Core.length(c.queue) === 0) return;
    var item = Core.peek(c.queue).chunk;
    if (item === closeSentinel) processClose(controller);
    else processWrite(controller);
  }

  function writeController(controller, chunk, chunkSize) {
    var c = controllerSlot(controller);
    try { Core.push(c.queue, queueItem(chunk), chunkSize); }
    catch (error) {
      errorIfNeeded(controller, error);
      return;
    }
    updateControllerBackpressure(controller);
    advanceQueue(controller);
  }

  function errorIfNeeded(controller, reason) {
    var c = controllerSlot(controller);
    if (streamSlot(c.stream).state === 'writable') controllerErrorInternal(controller, reason);
  }

  function controllerErrorInternal(controller, reason) {
    var c = controllerSlot(controller);
    var s = streamSlot(c.stream);
    if (s.state !== 'writable') return;
    clearControllerAlgorithms(controller);
    startErroring(c.stream, reason);
  }

  function controllerError(controller, reason) {
    var c = controllerSlot(controller);
    var s = streamSlot(c.stream);
    if (s.state !== 'writable') return;
    controllerErrorInternal(controller, reason);
  }

  function getChunkSize(controller, chunk) {
    var c = controllerSlot(controller);
    if (c.sizeAlgorithm === undefined) return 1;
    var result;
    try { result = c.sizeAlgorithm(chunk); }
    catch (error) { errorIfNeeded(controller, error); return 1; }
    return result;
  }

  function controllerClose(controller) {
    var c = controllerSlot(controller);
    var stream = c.stream;
    var s = streamSlot(stream);
    if (s.state !== 'writable') return;
    try { Core.push(c.queue, queueItem(closeSentinel), 0); }
    catch (error) { errorIfNeeded(controller, error); return; }
    advanceQueue(controller);
  }

  function finishInFlightWrite(stream) {
    var s = streamSlot(stream);
    var request = s.inFlightWriteRequest;
    if (request === undefined) return;
    request.resolve(undefined);
    s.inFlightWriteRequest = undefined;
    var c = controllerSlot(s.controller);
    if (Core.length(c.queue) > 0) Core.shift(c.queue);
    if (!isClosingSlot(s) && s.state === 'writable') updateControllerBackpressure(s.controller);
    advanceQueue(s.controller);
  }

  function finishInFlightWriteWithError(stream, reason) {
    var s = streamSlot(stream);
    var request = s.inFlightWriteRequest;
    if (request === undefined) return;
    if (s.state === 'writable') clearControllerAlgorithms(s.controller);
    request.reject(reason);
    s.inFlightWriteRequest = undefined;
    dealWithRejection(stream, reason);
  }

  function finishInFlightClose(stream) {
    var s = streamSlot(stream);
    var request = s.inFlightCloseRequest;
    if (request === undefined) return;
    request.resolve(undefined);
    s.inFlightCloseRequest = undefined;
    if (s.state === 'erroring') {
      s.storedError = undefined;
      if (s.pendingAbortRequest !== undefined) {
        s.pendingAbortRequest.deferred.resolve(undefined);
        s.pendingAbortRequest = undefined;
      }
    }
    s.state = 'closed';
    resolveWriterClosingForStream(stream);
    if (s.writer !== undefined) writerResolveClosed(s.writer);
    Core.clear(controllerSlot(s.controller).queue);
  }

  function finishInFlightCloseWithError(stream, reason) {
    var s = streamSlot(stream);
    var request = s.inFlightCloseRequest;
    if (request === undefined) return;
    request.reject(reason);
    s.inFlightCloseRequest = undefined;
    if (s.pendingAbortRequest !== undefined) {
      s.pendingAbortRequest.deferred.reject(reason);
      s.pendingAbortRequest = undefined;
    }
    dealWithRejection(stream, reason);
  }

  function closeStream(stream) {
    var s = streamSlot(stream);
    if (s.state === 'errored' || s.state === 'erroring') return rejected(s.storedError);
    if (s.state === 'closed') return rejected(typeError('Cannot close a closed stream'));
    if (isClosingSlot(s)) return rejected(typeError('Stream is already closing'));
    var request = deferred();
    s.closeRequest = request;
    resolveWriterClosingForStream(stream);
    if (s.writer !== undefined && s.backpressure && s.state === 'writable') writerResolveReady(s.writer);
    controllerClose(s.controller);
    return request.promise;
  }

  function abortStream(stream, reason) {
    var s = streamSlot(stream);
    if (s.state === 'closed' || s.state === 'errored') return resolved(undefined);
    // DOM's abort steps synchronously run abort listeners. They may reenter
    // WritableStreamAbort and reserve the pending request before this call.
    Core.signal.abort(controllerSlot(s.controller).signal, reason);
    s = streamSlot(stream);
    if (s.state === 'closed' || s.state === 'errored') return resolved(undefined);
    if (s.pendingAbortRequest !== undefined) return s.pendingAbortRequest.deferred.promise;
    var wasAlreadyErroring = s.state === 'erroring';
    if (wasAlreadyErroring) reason = undefined;
    var abortDeferred = deferred();
    var abortRequest = Core.record();
    abortRequest.deferred = abortDeferred;
    abortRequest.reason = reason;
    abortRequest.wasAlreadyErroring = wasAlreadyErroring;
    s.pendingAbortRequest = abortRequest;
    if (!wasAlreadyErroring) startErroring(stream, reason);
    return abortDeferred.promise;
  }

  function acquireWriter(stream) {
    var s = streamSlot(stream);
    if (s.writer !== undefined) throw typeError('WritableStream is already locked to a writer');
    var writer = Core.create(writerPrototype);
    var state = s.state;
    var ready;
    var closed;
    var closing = deferred();
    if (isClosingSlot(s) || state === 'closed' || state === 'errored') closing.resolve(undefined);
    if (state === 'writable') {
      ready = deferred();
      if (isClosingSlot(s) || !s.backpressure) ready.resolve(undefined);
      closed = deferred();
    } else if (state === 'erroring') {
      ready = deferred(); ready.reject(s.storedError); Core.handled(ready.promise);
      closed = deferred();
    } else if (state === 'closed') {
      ready = deferred(); ready.resolve(undefined);
      closed = deferred(); closed.resolve(undefined);
    } else {
      ready = deferred(); ready.reject(s.storedError); Core.handled(ready.promise);
      closed = deferred(); closed.reject(s.storedError); Core.handled(closed.promise);
    }
    var writerRecord = Core.record();
    writerRecord.stream = stream; writerRecord.ready = ready; writerRecord.closed = closed;
    writerRecord.closing = closing;
    Core.set(writerSlots, writer, writerRecord);
    s.writer = writer;
    return writer;
  }

  function releaseWriter(writer) {
    var w = writerSlot(writer);
    if (w.stream === undefined) return;
    var stream = w.stream;
    var s = streamSlot(stream);
    if (s.writer !== writer) throw typeError('Writer lock mismatch');
    var error = typeError('Writer was released');
    writerEnsureReadyRejected(writer, error);
    writerEnsureClosedRejected(writer, error);
    s.writer = undefined;
    w.stream = undefined;
  }

  function writerWrite(writer, chunk) {
    var w = writerSlot(writer);
    if (w.stream === undefined) return rejected(typeError('Writer has been released'));
    var stream = w.stream;
    var s = streamSlot(stream);
    var c = s.controller;
    var chunkSize = getChunkSize(c, chunk);
    if (w.stream !== stream) return rejected(typeError('Writer has been released'));
    s = streamSlot(stream);
    if (s.state === 'errored') return rejected(s.storedError);
    if (s.state === 'erroring') return rejected(s.storedError);
    if (isClosingSlot(s) || s.state === 'closed') return rejected(typeError('WritableStream is closing or closed'));
    var request = deferred();
    Core.push(s.writeRequests, request, 0);
    writeController(c, chunk, chunkSize);
    return request.promise;
  }

  function writerClose(writer) {
    var w = writerSlot(writer);
    if (w.stream === undefined) return rejected(typeError('Writer has been released'));
    var s = streamSlot(w.stream);
    if (s.state === 'errored' || s.state === 'erroring') return rejected(s.storedError);
    if (s.state === 'closed') return rejected(typeError('WritableStream is closed'));
    if (isClosingSlot(s)) return rejected(typeError('WritableStream is already closing'));
    return closeStream(w.stream);
  }

  function writerAbort(writer, reason) {
    var w = writerSlot(writer);
    if (w.stream === undefined) return rejected(typeError('Writer has been released'));
    return abortStream(w.stream, reason);
  }

  function closeWithErrorPropagation(writer) {
    var w = writerSlot(writer);
    if (w.stream === undefined) return rejected(typeError('Writer has been released'));
    var s = streamSlot(w.stream);
    if (s.state === 'errored' || s.state === 'erroring') return rejected(s.storedError);
    if (isClosingSlot(s) || s.state === 'closed') return resolved(undefined);
    return closeStream(w.stream);
  }

  function createWritable(startAlgorithm, writeAlgorithm, closeAlgorithm, abortAlgorithm, strategy) {
    var stream = Core.create(writableStreamPrototype);
    var s = Core.record();
    s.state = 'writable'; s.storedError = undefined; s.writer = undefined; s.controller = undefined;
    s.writeRequests = Core.queue(); s.inFlightWriteRequest = undefined;
    s.closeRequest = undefined; s.inFlightCloseRequest = undefined;
    s.pendingAbortRequest = undefined; s.backpressure = false;
    Core.set(writableSlots, stream, s);
    var controller = Core.create(writableControllerPrototype);
    var c = Core.record();
    c.stream = stream; c.queue = Core.queue(); c.started = false;
    c.strategyHWM = strategy.highWaterMark; c.sizeAlgorithm = strategy.sizeAlgorithm;
    c.writeAlgorithm = writeAlgorithm; c.closeAlgorithm = closeAlgorithm;
    c.abortAlgorithm = abortAlgorithm; c.signal = Core.signal.makeSignal();
    Core.set(controllerSlots, controller, c);
    s.controller = controller;
    s.backpressure = controllerDesiredSize(controller) <= 0;
    var startResult;
    try { startResult = startAlgorithm === undefined ? undefined : startAlgorithm(); }
    catch (error) { throw error; }
    observe(startResult, function () {
      c.started = true;
      advanceQueue(controller);
    }, function (reason) {
      c.started = true;
      dealWithRejection(stream, reason);
    });
    return stream;
  }

  function WritableStream() {
    if (!new.target) throw typeError('WritableStream must be constructed');
    var sink = arguments.length > 0 ? arguments[0] : undefined;
    var strategyInput = arguments.length > 1 ? arguments[1] : undefined;
    if (sink !== undefined && (sink === null ||
        (typeof sink !== 'object' && typeof sink !== 'function'))) {
      throw typeError('underlying sink must be an object');
    }
    // Web IDL converts the strategy dictionary before the underlying sink
    // getters run. Range-checking the HWM is later algorithm extraction.
    var convertedStrategy = Core.convertStrategy(strategyInput);
    // Convert the full underlying-sink dictionary in Web IDL member order
    // before checking its reserved type or extracting strategy values.
    var abort = sink === undefined ? undefined : sink.abort;
    if (abort !== undefined && typeof abort !== 'function') throw typeError('underlying sink abort must be callable');
    var close = sink === undefined ? undefined : sink.close;
    if (close !== undefined && typeof close !== 'function') throw typeError('underlying sink close must be callable');
    var start = sink === undefined ? undefined : sink.start;
    if (start !== undefined && typeof start !== 'function') throw typeError('underlying sink start must be callable');
    var type = sink === undefined ? undefined : sink.type;
    var write = sink === undefined ? undefined : sink.write;
    if (write !== undefined && typeof write !== 'function') throw typeError('underlying sink write must be callable');
    if (type !== undefined) throw rangeError('WritableStream underlying sink type is reserved');
    var strategy = extractWritableStrategy(convertedStrategy, 1);
    var slot = Core.record();
    slot.state = 'writable'; slot.storedError = undefined; slot.writer = undefined;
    slot.controller = undefined; slot.writeRequests = Core.queue();
    slot.inFlightWriteRequest = undefined; slot.closeRequest = undefined;
    slot.inFlightCloseRequest = undefined; slot.pendingAbortRequest = undefined;
    slot.backpressure = false;
    Core.set(writableSlots, this, slot);
    var controller = Core.create(writableControllerPrototype);
    var c = Core.record();
    c.stream = this; c.queue = Core.queue(); c.started = false;
    c.strategyHWM = strategy.highWaterMark; c.sizeAlgorithm = strategy.sizeAlgorithm;
    c.writeAlgorithm = undefined; c.closeAlgorithm = undefined; c.abortAlgorithm = undefined;
    c.signal = Core.signal.makeSignal();
    Core.set(controllerSlots, controller, c);
    slot.controller = controller;
    var receiver = sink;
    c.writeAlgorithm = write === undefined ? undefined : function (chunk) { return Core.call(write, receiver, [chunk, controller]); };
    c.closeAlgorithm = close === undefined ? undefined : function () { return Core.call(close, receiver, []); };
    c.abortAlgorithm = abort === undefined ? undefined : function (reason) { return Core.call(abort, receiver, [reason]); };
    slot.backpressure = controllerDesiredSize(controller) <= 0;
    var startResult = start === undefined ? undefined : Core.call(start, receiver, [controller]);
    var stream = this;
    observe(startResult, function () { c.started = true; advanceQueue(controller); }, function (reason) {
      c.started = true; dealWithRejection(stream, reason);
    });
  }

  function define(target, key, descriptor) { Core.define(target, key, descriptor); }
  function setFunctionName(fn, name) {
    var d = Core.record(); d.value = name; d.configurable = true;
    Core.define(fn, 'name', d);
  }
  function method(target, key, value) {
    var d = Core.record(); d.value = value; d.writable = true; d.configurable = true; d.enumerable = true;
    define(target, key, d);
  }
  function getter(target, key, value) {
    var d = Core.record(); d.get = value; d.configurable = true; d.enumerable = true;
    define(target, key, d);
  }
  function toStringTag(proto, value) {
    var d = Core.record(); d.value = value; d.configurable = true; d.enumerable = false; d.writable = false;
    define(proto, Core.Symbol.toStringTag, d);
  }

  function streamLockedGetter() { return isLocked(this); }
  function streamGetWriter() { return acquireWriter(this); }
  function streamAbort(reason) {
    try {
    var s = streamSlot(this);
    if (s.writer !== undefined) return rejected(typeError('Cannot abort a locked WritableStream'));
    return abortStream(this, reason);
    } catch (e) { return rejected(e); }
  }
  function streamClose() {
    try {
    var s = streamSlot(this);
    if (s.writer !== undefined) return rejected(typeError('Cannot close a locked WritableStream'));
    return closeStream(this);
    } catch (e) { return rejected(e); }
  }
  function writerDesiredSizeGetter() { return writerDesiredSize(this); }
  function writerReadyGetter() { return writerSlot(this).ready.promise; }
  function writerClosedGetter() { return writerSlot(this).closed.promise; }
  function writerWriteMethod(chunk) { try { return writerWrite(this, chunk); } catch (e) { return rejected(e); } }
  function writerCloseMethod() { try { return writerClose(this); } catch (e) { return rejected(e); } }
  function writerAbortMethod(reason) { try { return writerAbort(this, reason); } catch (e) { return rejected(e); } }
  function writerReleaseMethod() { releaseWriter(this); }
  function controllerSignalGetter() { return controllerSlot(this).signal; }
  function controllerErrorMethod(reason) { controllerError(this, reason); }

  function TransformStream() {
    if (!new.target) throw typeError('TransformStream must be constructed');
    var transformer = arguments.length > 0 ? arguments[0] : undefined;
    var writableStrategyInput = arguments.length > 1 ? arguments[1] : undefined;
    var readableStrategyInput = arguments.length > 2 ? arguments[2] : undefined;
    if (transformer !== undefined && (transformer === null ||
        (typeof transformer !== 'object' && typeof transformer !== 'function'))) {
      throw typeError('transformer must be an object');
    }
    var writableConverted = Core.convertStrategy(writableStrategyInput);
    var readableConverted = Core.convertStrategy(readableStrategyInput);
    var transformerValue = transformer === undefined ? null : transformer;
    // Web IDL dictionary conversion observes all transformer getters in
    // lexicographic member order before constructor type checks/extraction.
    var cancel = transformerValue === null ? undefined : transformerValue.cancel;
    if (cancel !== undefined && typeof cancel !== 'function') throw typeError('cancel must be callable');
    var flush = transformerValue === null ? undefined : transformerValue.flush;
    if (flush !== undefined && typeof flush !== 'function') throw typeError('flush must be callable');
    var readableType = transformerValue === null ? undefined : transformerValue.readableType;
    var start = transformerValue === null ? undefined : transformerValue.start;
    if (start !== undefined && typeof start !== 'function') throw typeError('transformer start must be callable');
    var transform = transformerValue === null ? undefined : transformerValue.transform;
    if (transform !== undefined && typeof transform !== 'function') throw typeError('transform must be callable');
    var writableType = transformerValue === null ? undefined : transformerValue.writableType;
    if (readableType !== undefined) throw rangeError('TransformStream readableType is reserved');
    if (writableType !== undefined) throw rangeError('TransformStream writableType is reserved');
    var readableStrategy = extractTransformStrategy(readableConverted, 0);
    var writableStrategy = extractTransformStrategy(writableConverted, 1);
    initializeTransform(this, transformerValue, writableStrategy, readableStrategy, start, transform, flush, cancel);
  }

  function transformReadableGetter() { return transformSlot(this).readable; }
  function transformWritableGetter() { return transformSlot(this).writable; }
  function transformControllerDesiredSizeGetter() {
    var controller = transformControllerSlot(this);
    return ReadableOps.desiredSize(controller.readable);
  }
  function transformControllerEnqueue(chunk) { transformEnqueue(this, chunk); }
  function transformControllerError(reason) { transformError(this, reason); }
  function transformControllerTerminate() { transformTerminate(this); }

  var writableStreamPrototype = WritableStream.prototype;
  var writerPrototype = WritableStreamDefaultWriter.prototype;
  var writableControllerPrototype = WritableStreamDefaultController.prototype;
  var transformStreamPrototype = TransformStream.prototype;
  var transformControllerPrototype = TransformStreamDefaultController.prototype;

  setFunctionName(streamLockedGetter, 'get locked');
  setFunctionName(writerDesiredSizeGetter, 'get desiredSize');
  setFunctionName(writerReadyGetter, 'get ready');
  setFunctionName(writerClosedGetter, 'get closed');
  setFunctionName(controllerSignalGetter, 'get signal');
  setFunctionName(transformReadableGetter, 'get readable');
  setFunctionName(transformWritableGetter, 'get writable');
  setFunctionName(transformControllerDesiredSizeGetter, 'get desiredSize');
  setFunctionName(streamGetWriter, 'getWriter');
  setFunctionName(streamAbort, 'abort');
  setFunctionName(streamClose, 'close');
  setFunctionName(writerWriteMethod, 'write');
  setFunctionName(writerCloseMethod, 'close');
  setFunctionName(writerAbortMethod, 'abort');
  setFunctionName(writerReleaseMethod, 'releaseLock');
  setFunctionName(controllerErrorMethod, 'error');
  setFunctionName(transformControllerEnqueue, 'enqueue');
  setFunctionName(transformControllerError, 'error');
  setFunctionName(transformControllerTerminate, 'terminate');

  // controller.signal
  getter(writableControllerPrototype, 'signal', controllerSignalGetter);
  method(writableControllerPrototype, 'error', controllerErrorMethod);
  getter(writableStreamPrototype, 'locked', streamLockedGetter);
  method(writableStreamPrototype, 'getWriter', streamGetWriter);
  method(writableStreamPrototype, 'abort', streamAbort);
  method(writableStreamPrototype, 'close', streamClose);
  getter(writerPrototype, 'desiredSize', writerDesiredSizeGetter);
  getter(writerPrototype, 'ready', writerReadyGetter);
  getter(writerPrototype, 'closed', writerClosedGetter);
  method(writerPrototype, 'write', writerWriteMethod);
  method(writerPrototype, 'close', writerCloseMethod);
  method(writerPrototype, 'abort', writerAbortMethod);
  method(writerPrototype, 'releaseLock', writerReleaseMethod);
  getter(transformStreamPrototype, 'readable', transformReadableGetter);
  getter(transformStreamPrototype, 'writable', transformWritableGetter);
  getter(transformControllerPrototype, 'desiredSize', transformControllerDesiredSizeGetter);
  method(transformControllerPrototype, 'enqueue', transformControllerEnqueue);
  method(transformControllerPrototype, 'error', transformControllerError);
  method(transformControllerPrototype, 'terminate', transformControllerTerminate);
  toStringTag(writableStreamPrototype, 'WritableStream');
  toStringTag(writerPrototype, 'WritableStreamDefaultWriter');
  toStringTag(writableControllerPrototype, 'WritableStreamDefaultController');
  toStringTag(transformStreamPrototype, 'TransformStream');
  toStringTag(transformControllerPrototype, 'TransformStreamDefaultController');

  function WritableStreamDefaultController() { throw typeError('Illegal constructor'); }
  function WritableStreamDefaultWriter(stream) {
    if (!new.target) throw typeError('WritableStreamDefaultWriter requires new');
    var writer = acquireWriter(stream), slot = writerSlot(writer);
    Core.delete(writerSlots, writer);
    Core.set(writerSlots, this, slot);
    streamSlot(stream).writer = this;
  }
  function TransformStreamDefaultController() { throw typeError('Illegal constructor'); }

  // Replace the forward references created above with the public constructor
  // identities before any stream can be instantiated.
  // (Function declarations are hoisted, so the operations above refer to these.)

  function setupInternalWritable(start, write, close, abort, strategy) {
    return createWritable(start, write, close, abort, strategy);
  }

  function sourcePull(stream) {
    unblockTransformWrite(stream);
  }

  function normalizeInternalStrategy(hwm) {
    var strategy = Core.record();
    strategy.highWaterMark = hwm;
    strategy.sizeAlgorithm = function () { return 1; };
    return strategy;
  }

  function initializeTransform(stream, transformer, writableStrategy, readableStrategy,
      startMethod, transformMethod, flushMethod, cancelMethod) {
    var transform = Core.record();
    transform.readable = undefined; transform.writable = undefined;
    transform.backpressure = true; transform.backpressureChange = deferred();
    transform.controller = undefined;
    Core.set(transformSlots, stream, transform);
    var controller = Core.create(transformControllerPrototype);
    var tcontroller = Core.record();
    tcontroller.stream = stream; tcontroller.readable = undefined; tcontroller.writable = undefined;
    tcontroller.transformAlgorithm = undefined; tcontroller.flushAlgorithm = undefined;
    tcontroller.cancelAlgorithm = undefined; tcontroller.finishPromise = undefined;
    Core.set(transformControllerSlots, controller, tcontroller);
    transform.controller = controller;

    var startPromise = deferred();
    var writable = setupInternalWritable(function () { return startPromise.promise; },
      function (chunk) { return sinkWrite(stream, chunk); },
      function () { return sinkClose(stream); },
      function (reason) { return sinkAbort(stream, reason); }, writableStrategy);
    var readableAlgorithms = Core.record();
    readableAlgorithms.start = function () { return startPromise.promise; };
    readableAlgorithms.pull = function () { return sourcePull(stream); };
    readableAlgorithms.cancel = function (reason) { return sourceCancel(stream, reason); };
    var readable = ReadableOps.createDefault(readableAlgorithms, readableStrategy);
    transform.readable = readable;
    transform.writable = writable;
    tcontroller.readable = readable;
    tcontroller.writable = writable;

    if (transformMethod === undefined) {
      tcontroller.transformAlgorithm = function (chunk) {
        try { transformEnqueue(controller, chunk); return resolved(undefined); }
        catch (error) { return rejected(error); }
      };
    } else {
      tcontroller.transformAlgorithm = function (chunk) { return Core.call(transformMethod, transformer, [chunk, controller]); };
    }
    tcontroller.flushAlgorithm = flushMethod === undefined ? function () { return undefined; } :
      function () { return Core.call(flushMethod, transformer, [controller]); };
    tcontroller.cancelAlgorithm = cancelMethod === undefined ? function () { return undefined; } :
      function (reason) { return Core.call(cancelMethod, transformer, [reason]); };

    var startResult = startMethod === undefined ? undefined : Core.call(startMethod, transformer, [controller]);
    observe(startResult, function () { startPromise.resolve(undefined); },
      function (reason) { startPromise.reject(reason); });
  }

  function setTransformBackpressure(stream, backpressure) {
    var t = transformSlot(stream);
    if (t.backpressure === backpressure) return;
    t.backpressureChange.resolve(undefined);
    t.backpressureChange = deferred();
    t.backpressure = backpressure;
  }
  function unblockTransformWrite(stream) {
    if (transformSlot(stream).backpressure) setTransformBackpressure(stream, false);
  }
  function transformErrorWritable(stream, reason) {
    var t = transformSlot(stream);
    clearTransformAlgorithms(t.controller);
    errorIfNeeded(streamSlot(t.writable).controller, reason);
    unblockTransformWrite(stream);
  }
  function transformError(controller, reason) {
    var c = transformControllerSlot(controller);
    var t = transformSlot(c.stream);
    ReadableOps.error(t.readable, reason);
    transformErrorWritable(c.stream, reason);
  }
  function transformEnqueue(controller, chunk) {
    var c = transformControllerSlot(controller);
    var t = transformSlot(c.stream);
    if (!ReadableOps.canCloseOrEnqueue(t.readable)) throw typeError('TransformStream is not enqueueable');
    try { ReadableOps.enqueue(t.readable, chunk); }
    catch (error) { transformErrorWritable(c.stream, error); throw error; }
    var desired = ReadableOps.desiredSize(t.readable);
    var backpressure = desired === null || desired <= 0;
    if (backpressure !== t.backpressure) setTransformBackpressure(c.stream, backpressure);
  }
  function transformTerminate(controller) {
    var c = transformControllerSlot(controller);
    var t = transformSlot(c.stream);
    if (!ReadableOps.canCloseOrEnqueue(t.readable)) return;
    ReadableOps.close(t.readable);
    transformErrorWritable(c.stream, typeError('TransformStream has been terminated'));
  }
  function clearTransformAlgorithms(controller) {
    var c = transformControllerSlot(controller);
    c.transformAlgorithm = undefined;
    c.flushAlgorithm = undefined;
    c.cancelAlgorithm = undefined;
  }
  function transformFailure(stream, reason) {
    var t = transformSlot(stream);
    ReadableOps.error(t.readable, reason);
    transformErrorWritable(stream, reason);
  }
  function performTransform(stream, chunk) {
    var t = transformSlot(stream);
    var c = transformControllerSlot(t.controller);
    var result;
    try { result = c.transformAlgorithm === undefined ? undefined : c.transformAlgorithm(chunk); }
    catch (error) { transformFailure(stream, error); return rejected(error); }
    var operation = deferred();
    observe(result, function () { operation.resolve(undefined); }, function (reason) {
      transformFailure(stream, reason);
      operation.reject(reason);
    });
    return operation.promise;
  }
  function sinkWrite(stream, chunk) {
    var t = transformSlot(stream);
    if (!t.backpressure) return performTransform(stream, chunk);
    var change = t.backpressureChange.promise;
    var operation = deferred();
    Core.react(change, function () {
      var writableState = streamSlot(t.writable);
      if (writableState.state === 'erroring') {
        operation.reject(writableState.storedError);
      } else if (writableState.state !== 'writable') {
        operation.reject(writableState.storedError || typeError('TransformStream writable is not writable'));
      } else {
        var promise;
        try { promise = performTransform(stream, chunk); }
        catch (error) { operation.reject(error); return undefined; }
        observe(promise, function () { operation.resolve(undefined); },
          function (reason) { operation.reject(reason); });
      }
      return undefined;
    }, function (reason) { operation.reject(reason); return undefined; });
    return operation.promise;
  }
  function sinkClose(stream) {
    var c = transformControllerSlot(transformSlot(stream).controller);
    if (c.finishPromise !== undefined) return c.finishPromise.promise;
    c.finishPromise = deferred();
    var result;
    try { result = c.flushAlgorithm === undefined ? undefined : c.flushAlgorithm(); }
    catch (error) { transformFailure(stream, error); c.finishPromise.reject(error); return c.finishPromise.promise; }
    observe(result, function () {
      clearTransformAlgorithms(transformSlot(stream).controller);
      try { ReadableOps.close(transformSlot(stream).readable); c.finishPromise.resolve(undefined); }
      catch (error) { transformFailure(stream, error); c.finishPromise.reject(error); }
    }, function (reason) {
      transformFailure(stream, reason);
      c.finishPromise.reject(reason);
    });
    return c.finishPromise.promise;
  }
  function sinkAbort(stream, reason) { return transformCancel(stream, reason, false); }
  function sourceCancel(stream, reason) { return transformCancel(stream, reason, true); }
  function transformCancel(stream, reason, fromReadable) {
    var c = transformControllerSlot(transformSlot(stream).controller);
    if (c.finishPromise !== undefined) return c.finishPromise.promise;
    c.finishPromise = deferred();
    var result;
    try { result = c.cancelAlgorithm === undefined ? undefined : c.cancelAlgorithm(reason); }
    catch (error) { result = rejected(error); }
    clearTransformAlgorithms(transformSlot(stream).controller);
    observe(result, function () {
      var t = transformSlot(stream);
      if (fromReadable) {
        var w = streamSlot(t.writable);
        if (w.state === 'errored') c.finishPromise.reject(w.storedError);
        else {
          errorIfNeeded(w.controller, reason);
          unblockTransformWrite(stream);
          c.finishPromise.resolve(undefined);
        }
      } else {
        var rstate = ReadableOps.state(t.readable);
        if (rstate === 'errored') c.finishPromise.reject(ReadableOps.storedError(t.readable));
        else {
          ReadableOps.error(t.readable, reason);
          unblockTransformWrite(stream);
          c.finishPromise.resolve(undefined);
        }
      }
    }, function (error) {
      var t = transformSlot(stream);
      if (fromReadable) ReadableOps.error(t.readable, error);
      else errorIfNeeded(streamSlot(t.writable).controller, error);
      unblockTransformWrite(stream);
      c.finishPromise.reject(error);
    });
    return c.finishPromise.promise;
  }

  function createIdentityTransform() {
    var pair = Core.record();
    var stream = Core.record();
    initializeTransform(stream, null, normalizeInternalStrategy(1), normalizeInternalStrategy(0));
    pair.readable = transformSlot(stream).readable;
    pair.writable = transformSlot(stream).writable;
    return pair;
  }

  function acquireDefaultWriter(stream) { return acquireWriter(stream); }
  function writerReady(writer) { return writerSlot(writer).ready.promise; }
  function writerClosed(writer) { return writerSlot(writer).closed.promise; }
  function writerClosing(writer) { return writerSlot(writer).closing.promise; }
  function abortWritable(stream, reason) { return abortStream(stream, reason); }

  function installGlobal(name, value) {
    Core.define(value, 'prototype', { writable: false });
    var d = Core.record(); d.value = value; d.writable = true; d.configurable = true; d.enumerable = false;
    Core.define(globalThis, name, d);
  }

  // Optional operation arguments do not contribute to Web IDL function length.
  Core.define(streamAbort, 'length', { value: 0, configurable: true });
  Core.define(writerWriteMethod, 'length', { value: 0, configurable: true });
  Core.define(writerAbortMethod, 'length', { value: 0, configurable: true });
  Core.define(controllerErrorMethod, 'length', { value: 0, configurable: true });
  Core.define(transformControllerError, 'length', { value: 0, configurable: true });

  installGlobal('WritableStream', WritableStream);
  installGlobal('WritableStreamDefaultWriter', WritableStreamDefaultWriter);
  installGlobal('WritableStreamDefaultController', WritableStreamDefaultController);
  installGlobal('TransformStream', TransformStream);
  installGlobal('TransformStreamDefaultController', TransformStreamDefaultController);

  var ops = Core.record();
  ops.isWritable = isWritable;
  ops.isLocked = isLocked;
  ops.state = function (stream) { return streamSlot(stream).state; };
  ops.storedError = function (stream) { return streamSlot(stream).storedError; };
  ops.isClosing = isClosing;
  ops.acquireDefaultWriter = acquireDefaultWriter;
  ops.writerDesiredSize = writerDesiredSize;
  ops.writerReady = writerReady;
  ops.writerClosed = writerClosed;
  ops.writerClosing = writerClosing;
  ops.writerWrite = writerWrite;
  ops.writerCloseWithErrorPropagation = closeWithErrorPropagation;
  ops.writerAbort = writerAbort;
  ops.releaseDefaultWriter = releaseWriter;
  ops.abortWritable = abortWritable;
  ops.createIdentityTransform = createIdentityTransform;
  return ops;
})(Core, ReadableOps);
