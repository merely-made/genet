// Private piping, async-iteration, and queuing-strategy algorithms for Streams.
// This source is evaluated after Core, ReadableOps, and WritableOps exist and
// before author script runs. It deliberately uses only their captured/private
// surfaces while manipulating stream state.
var PipingOps = (function (Core, ReadableOps, WritableOps) {
    "use strict";

    var signalOps = Core.signal;
    var ReadableStream = ReadableOps.constructor;
    var iteratorSlots = Core.sharedMap("ReadableStreamAsyncIterator");
    var countStrategySlots = Core.sharedMap("CountQueuingStrategy");
    var byteLengthStrategySlots = Core.sharedMap("ByteLengthQueuingStrategy");
    var asyncIteratorSymbol = Core.Symbol.asyncIterator;
    var toStringTagSymbol = Core.Symbol.toStringTag;
    var countSizeFunction = ({ size() { return 1; } }).size;
    var byteLengthSizeFunction = ({ size(chunk) { return chunk.byteLength; } }).size;
    var defaultControllerPrototype = ReadableOps.ReadableStreamDefaultController.prototype;
    var controllerEnqueue = Core.descriptor(defaultControllerPrototype, "enqueue").value;
    var controllerClose = Core.descriptor(defaultControllerPrototype, "close").value;
    var controllerError = Core.descriptor(defaultControllerPrototype, "error").value;

    function typeError(message) {
        return new Core.TypeError(message);
    }

    function rejected(error) {
        var result = Core.deferred();
        result.reject(error);
        return result.promise;
    }

    function list() {
        var result = Core.record();
        result.length = 0;
        return result;
    }

    function append(target, value) {
        target[target.length] = value;
        target.length += 1;
    }

    function descriptor(value, writable, enumerable, configurable) {
        var result = Core.record();
        result.value = value;
        result.writable = writable;
        result.enumerable = enumerable;
        result.configurable = configurable;
        return result;
    }

    function getterDescriptor(getter, enumerable, configurable) {
        var result = Core.record();
        result.get = getter;
        result.enumerable = enumerable;
        result.configurable = configurable;
        return result;
    }

    function defineMethod(target, key, fn, length) {
        Core.define(fn, "length", descriptor(length, undefined, undefined, true));
        Core.define(target, key, descriptor(fn, true, true, true));
    }

    function defineFunctionName(fn, name) {
        Core.define(fn, "name", descriptor(name, undefined, undefined, true));
    }

    function defineGetter(target, key, getter) {
        Core.define(target, key, getterDescriptor(getter, true, true));
    }

    function callable(value) {
        return typeof value === "function";
    }

    function isObject(value) {
        return (typeof value === "object" && value !== null) || typeof value === "function";
    }

    function dictionaryValue(value, key) {
        if (value === undefined || value === null) {
            return undefined;
        }
        return value[key];
    }

    function convertPipeOptions(value) {
        // Web IDL dictionary members are converted in lexicographic order.
        var preventAbort = !!dictionaryValue(value, "preventAbort");
        var preventCancel = !!dictionaryValue(value, "preventCancel");
        var preventClose = !!dictionaryValue(value, "preventClose");
        var signal = dictionaryValue(value, "signal");
        if (signal !== undefined && !signalOps.isSignal(signal)) {
            throw typeError("The signal option is not an AbortSignal");
        }
        var result = Core.record();
        result.preventClose = preventClose;
        result.preventAbort = preventAbort;
        result.preventCancel = preventCancel;
        result.signal = signal;
        return result;
    }

    function invokeActions(actions) {
        var promises = list();
        for (var i = 0; i < actions.length; i += 1) {
            try {
                append(promises, Core.resolve(actions[i]()));
            } catch (error) {
                append(promises, rejected(error));
            }
        }
        return promises;
    }

    function allSettled(promises, rejectOnFailure) {
        var result = Core.deferred();
        var count = promises.length;
        if (count === 0) {
            result.resolve(undefined);
            return result.promise;
        }
        var remaining = count;
        var failed = false;
        var firstError;
        function settled() {
            remaining -= 1;
            if (remaining === 0) {
                if (failed && rejectOnFailure) {
                    result.reject(firstError);
                } else {
                    result.resolve(undefined);
                }
            }
        }
        for (var i = 0; i < count; i += 1) {
            Core.react(promises[i], function () {
                settled();
                return undefined;
            }, function (error) {
                if (!failed) {
                    failed = true;
                    firstError = error;
                    if (rejectOnFailure) {
                        result.reject(firstError);
                    }
                }
                settled();
                return undefined;
            });
        }
        return result.promise;
    }

    function pipeToInternal(source, destination, options) {
        if (!ReadableOps.isReadable(source)) {
            throw typeError("ReadableStream.pipeTo called on an incompatible receiver");
        }
        if (!WritableOps.isWritable(destination)) {
            throw typeError("The pipe destination is not a WritableStream");
        }
        if (ReadableOps.isLocked(source)) {
            return rejected(typeError("Cannot pipe a locked ReadableStream"));
        }
        if (WritableOps.isLocked(destination)) {
            return rejected(typeError("Cannot pipe to a locked WritableStream"));
        }

        var reader;
        var writer;
        try {
            reader = ReadableOps.acquireReader(source, "default");
            writer = WritableOps.acquireDefaultWriter(destination);
        } catch (error) {
            if (reader !== undefined) {
                try { ReadableOps.readerRelease(reader); } catch (_) { /* best effort */ }
            }
            return rejected(error);
        }

        var completion = Core.deferred();
        var state = Core.record();
        state.reader = reader;
        state.writer = writer;
        state.source = source;
        state.destination = destination;
        state.preventClose = options.preventClose;
        state.preventAbort = options.preventAbort;
        state.preventCancel = options.preventCancel;
        state.signal = options.signal;
        state.shuttingDown = false;
        state.finalized = false;
        state.pendingWritesHead = undefined;
        state.pendingWritesTail = undefined;
        state.writeFailure = undefined;
        state.hasWriteFailure = false;
        state.abortAlgorithm = undefined;

        // This is deliberately synchronous, including at HWM 0. It is visible
        // to bodyUsed and other consumers before the first ready reaction.
        ReadableOps.disturb(source);

        function finalize(error, hasError) {
            if (state.finalized) {
                return;
            }
            state.finalized = true;
            if (state.signal !== undefined && state.abortAlgorithm !== undefined) {
                try { signalOps.removeAbortAlgorithm(state.signal, state.abortAlgorithm); } catch (_) { /* finalization must continue */ }
            }
            try { WritableOps.releaseDefaultWriter(writer); } catch (_) { /* release once */ }
            try { ReadableOps.readerRelease(reader); } catch (_) { /* release once */ }
            if (hasError) {
                completion.reject(error);
            } else {
                completion.resolve(undefined);
            }
        }

        function snapshotPendingWrites() {
            var promises = list();
            var node = state.pendingWritesHead;
            while (node !== undefined) {
                append(promises, node.promise);
                node = node.next;
            }
            return promises;
        }

        function shutdown(error, hasError, actions) {
            if (state.shuttingDown || state.finalized) {
                return;
            }
            state.shuttingDown = true;
            var pending = allSettled(snapshotPendingWrites(), false);
            Core.react(pending, function () {
                var actionPromises = invokeActions(actions || list());
                var actionResult = allSettled(actionPromises, true);
                Core.react(actionResult, function () {
                    if (state.hasWriteFailure) {
                        finalize(state.writeFailure, true);
                    } else {
                        finalize(error, hasError);
                    }
                    return undefined;
                }, function (actionError) {
                    finalize(actionError, true);
                    return undefined;
                });
                return undefined;
            }, function (pendingError) {
                // allSettled normally fulfills only after observing all writes.
                finalize(pendingError, true);
                return undefined;
            });
        }

        function sourceErrored(error) {
            var actions = list();
            if (!state.preventAbort) {
                append(actions, function () { return WritableOps.writerAbort(writer, error); });
            }
            shutdown(error, true, actions);
        }

        function destinationErrored(error) {
            var actions = list();
            if (!state.preventCancel) {
                append(actions, function () {
                    if (ReadableOps.state(source) === "readable") {
                        return ReadableOps.readerCancel(reader, error);
                    }
                    return Core.resolve(undefined);
                });
            }
            shutdown(error, true, actions);
        }

        function sourceClosed() {
            var actions = list();
            if (!state.preventClose) {
                append(actions, function () { return WritableOps.writerCloseWithErrorPropagation(writer); });
            }
            shutdown(undefined, false, actions);
        }

        function destinationClosedOrClosing() {
            var error = typeError("The destination WritableStream closed before the source completed");
            var actions = list();
            if (!state.preventCancel) {
                append(actions, function () {
                    if (ReadableOps.state(source) === "readable") {
                        return ReadableOps.readerCancel(reader, error);
                    }
                    return Core.resolve(undefined);
                });
            }
            shutdown(error, true, actions);
        }

        function abortAlgorithm() {
            if (state.shuttingDown || state.finalized) {
                return;
            }
            var reason = signalOps.reason(state.signal);
            var actions = list();
            if (!state.preventAbort) {
                append(actions, function () { return WritableOps.writerAbort(writer, reason); });
            }
            if (!state.preventCancel) {
                append(actions, function () {
                    if (ReadableOps.state(source) === "readable") {
                        return ReadableOps.readerCancel(reader, reason);
                    }
                    return Core.resolve(undefined);
                });
            }
            shutdown(reason, true, actions);
        }

        function removeWrite(target) {
            var previous;
            var node = state.pendingWritesHead;
            while (node !== undefined) {
                if (node === target) {
                    if (previous === undefined) {
                        state.pendingWritesHead = node.next;
                    } else {
                        previous.next = node.next;
                    }
                    if (state.pendingWritesTail === node) {
                        state.pendingWritesTail = previous;
                    }
                    return;
                }
                previous = node;
                node = node.next;
            }
        }

        function completePendingWrite(node, error, hasError) {
            removeWrite(node);
            if (hasError) {
                node.deferred.reject(error);
            } else {
                node.deferred.resolve(undefined);
            }
        }

        function startWrite(node, chunk) {
            if (node.started) {
                return;
            }
            node.started = true;

            // A read may complete just as the destination begins closing or
            // erroring. The pipe loop writes an already-read chunk only while
            // the destination is still writable and no close is queued.
            if (WritableOps.state(destination) !== "writable" ||
                WritableOps.isClosing(destination)) {
                completePendingWrite(node, undefined, false);
                return;
            }

            var writePromise;
            try {
                writePromise = Core.resolve(WritableOps.writerWrite(writer, chunk));
            } catch (error) {
                if (!state.hasWriteFailure) {
                    state.hasWriteFailure = true;
                    state.writeFailure = error;
                }
                node.deferred.reject(error);
                destinationErrored(error);
                removeWrite(node);
                return;
            }
            Core.handled(writePromise);
            Core.react(writePromise, function () {
                completePendingWrite(node, undefined, false);
                if (!state.shuttingDown) {
                    pump();
                }
                return undefined;
            }, function (error) {
                if (!state.hasWriteFailure) {
                    state.hasWriteFailure = true;
                    state.writeFailure = error;
                }
                node.deferred.reject(error);
                destinationErrored(error);
                removeWrite(node);
                return undefined;
            });
            // Queueing the write can itself create backpressure. Start another
            // read only when the current writer.ready allows it.
            pump();
        }

        function writeChunk(chunk) {
            var node = Core.record();
            node.deferred = Core.deferred();
            node.promise = node.deferred.promise;
            node.next = undefined;
            node.started = false;
            Core.handled(node.promise);
            if (state.pendingWritesTail === undefined) {
                state.pendingWritesHead = node;
                state.pendingWritesTail = node;
            } else {
                state.pendingWritesTail.next = node;
                state.pendingWritesTail = node;
            }

            // ReadableOps.readRequest delivers synchronously when enqueue()
            // satisfies an outstanding read. Pipe-to's read/write loop runs
            // as a promise reaction, so the sink's write algorithm must not
            // run inside the source's enqueue() call.
            Core.react(Core.resolve(undefined), function () {
                startWrite(node, chunk);
                return undefined;
            }, function (error) {
                completePendingWrite(node, error, true);
                return undefined;
            });
        }

        function readOne() {
            if (state.shuttingDown || state.finalized) {
                return;
            }
            try {
                ReadableOps.readRequest(reader, {
                    chunk: function (chunk) {
                        if (!state.shuttingDown && !state.finalized) {
                            writeChunk(chunk);
                        }
                    },
                    close: function () { sourceClosed(); },
                    error: function (error) { sourceErrored(error); }
                });
            } catch (error) {
                sourceErrored(error);
            }
        }

        function checkShutdownConditions() {
            if (state.shuttingDown || state.finalized) {
                return true;
            }
            var sourceState = ReadableOps.state(source);
            if (sourceState === "errored") {
                sourceErrored(ReadableOps.storedError(source));
                return true;
            }
            var destinationState = WritableOps.state(destination);
            if (destinationState === "errored") {
                destinationErrored(WritableOps.storedError(destination));
                return true;
            }
            if (sourceState === "closed") {
                sourceClosed();
                return true;
            }
            if (destinationState === "closed" || WritableOps.isClosing(destination)) {
                destinationClosedOrClosing();
                return true;
            }
            return false;
        }

        function pump() {
            if (checkShutdownConditions()) {
                return;
            }
            var ready;
            try {
                ready = WritableOps.writerReady(writer);
            } catch (error) {
                destinationErrored(error);
                return;
            }
            Core.react(ready, function () {
                if (!checkShutdownConditions()) {
                    readOne();
                }
                return undefined;
            }, function (error) {
                destinationErrored(error);
                return undefined;
            });
        }

        // Attach both watches before the first pump. Their reactions only
        // classify a state change; the read/write callbacks remain authoritative
        // for normal completion and exact stored errors.
        try {
            Core.react(ReadableOps.readerClosed(reader), function () {
                if (ReadableOps.state(source) === "closed") {
                    sourceClosed();
                }
                return undefined;
            }, function (error) {
                if (ReadableOps.state(source) === "errored") {
                    sourceErrored(ReadableOps.storedError(source));
                } else if (!state.shuttingDown && !state.finalized) {
                    sourceErrored(error);
                }
                return undefined;
            });
            Core.react(WritableOps.writerClosed(writer), function () {
                if (!state.shuttingDown && !state.finalized &&
                    (WritableOps.state(destination) === "closed" || WritableOps.isClosing(destination))) {
                    destinationClosedOrClosing();
                }
                return undefined;
            }, function (error) {
                if (WritableOps.state(destination) === "errored") {
                    destinationErrored(WritableOps.storedError(destination));
                } else if (!state.shuttingDown && !state.finalized) {
                    destinationErrored(error);
                }
                return undefined;
            });
            Core.react(WritableOps.writerClosing(writer), function () {
                if (!state.shuttingDown && !state.finalized) {
                    checkShutdownConditions();
                }
                return undefined;
            });
        } catch (error) {
            shutdown(error, true, []);
        }

        if (state.signal !== undefined) {
            state.abortAlgorithm = abortAlgorithm;
            if (signalOps.isAborted(state.signal)) {
                abortAlgorithm();
            } else {
                try {
                    signalOps.addAbortAlgorithm(state.signal, abortAlgorithm);
                    if (signalOps.isAborted(state.signal)) {
                        abortAlgorithm();
                    }
                } catch (error) {
                    shutdown(error, true, []);
                }
            }
        }
        pump();
        return completion.promise;
    }

    function pipeTo(destination, optionsValue) {
        if (!ReadableOps.isReadable(this)) {
            return rejected(typeError("ReadableStream.pipeTo called on an incompatible receiver"));
        }
        if (!WritableOps.isWritable(destination)) {
            return rejected(typeError("The pipe destination is not a WritableStream"));
        }
        var options;
        try {
            options = convertPipeOptions(optionsValue);
        } catch (error) {
            return rejected(error);
        }
        return pipeToInternal(this, destination, options);
    }

    function pipeThrough(transform, optionsValue) {
        var readable;
        var writable;
        var options;
        try {
            readable = dictionaryValue(transform, "readable");
            if (!ReadableOps.isReadable(readable)) {
                throw typeError("The transform must provide readable and writable streams");
            }
            writable = dictionaryValue(transform, "writable");
            if (!WritableOps.isWritable(writable)) {
                throw typeError("The transform must provide readable and writable streams");
            }
            options = convertPipeOptions(optionsValue);
        } catch (error) {
            throw error;
        }
        if (ReadableOps.isLocked(this) || WritableOps.isLocked(writable)) {
            throw typeError("Cannot pipe a locked stream");
        }
        var promise = pipeToInternal(this, writable, options);
        Core.handled(promise);
        return readable;
    }

    function resultObject(value, done) {
        var result = Core.create(Core.objectPrototype);
        Core.define(result, "value", descriptor(value, true, true, true));
        Core.define(result, "done", descriptor(done, true, true, true));
        return result;
    }

    function asyncIteratorNext() {
        var slot = Core.get(iteratorSlots, this);
        if (slot === undefined) {
            return rejected(typeError("ReadableStream async iterator next called on an incompatible receiver"));
        }
        var request = Core.deferred();
        var node = Core.record();
        node.deferred = request;
        node.next = undefined;
        if (slot.tail === undefined) {
            slot.head = node;
            slot.tail = node;
        } else {
            slot.tail.next = node;
            slot.tail = node;
        }
        processIterator(slot);
        return request.promise;
    }

    function settleQueued(slot, isError, value) {
        while (slot.head !== undefined) {
            var node = slot.head;
            slot.head = node.next;
            if (slot.head === undefined) {
                slot.tail = undefined;
            }
            if (isError) {
                node.deferred.reject(value);
            } else {
                node.deferred.resolve(value);
            }
        }
    }

    function releaseIteratorReader(slot) {
        if (slot.released) {
            return;
        }
        slot.released = true;
        slot.done = true;
        try { ReadableOps.readerRelease(slot.reader); } catch (_) { /* release exactly once */ }
    }

    function processIterator(slot) {
        if (slot.busy || slot.head === undefined) {
            return;
        }
        if (slot.done) {
            if (slot.returnPending) {
                return;
            }
            if (slot.returnGeneration !== 0) {
                while (slot.head !== undefined) {
                    var returnedNode = slot.head;
                    slot.head = returnedNode.next;
                    if (slot.head === undefined) {
                        slot.tail = undefined;
                    }
                    if (returnedNode.error !== undefined) {
                        returnedNode.deferred.reject(returnedNode.error);
                    } else {
                        returnedNode.deferred.resolve(resultObject(undefined, true));
                    }
                }
                return;
            }
            settleQueued(slot, false, resultObject(undefined, true));
            return;
        }
        slot.busy = true;
        var node = slot.head;
        slot.head = node.next;
        if (slot.head === undefined) {
            slot.tail = undefined;
        }
        try {
            ReadableOps.readRequest(slot.reader, {
                chunk: function (chunk) {
                    node.deferred.resolve(resultObject(chunk, false));
                    slot.busy = false;
                    processIterator(slot);
                },
                close: function () {
                    releaseIteratorReader(slot);
                    if (slot.returnPending) {
                        node.next = slot.head;
                        slot.head = node;
                        if (slot.tail === undefined) {
                            slot.tail = node;
                        }
                        slot.busy = false;
                        return;
                    }
                    node.deferred.resolve(resultObject(undefined, true));
                    slot.busy = false;
                    settleQueued(slot, false, resultObject(undefined, true));
                },
                error: function (error) {
                    releaseIteratorReader(slot);
                    if (slot.returnPending) {
                        node.error = error;
                        node.next = slot.head;
                        slot.head = node;
                        if (slot.tail === undefined) {
                            slot.tail = node;
                        }
                        slot.busy = false;
                        return;
                    }
                    node.deferred.reject(error);
                    slot.busy = false;
                    settleQueued(slot, false, resultObject(undefined, true));
                }
            });
        } catch (error) {
            releaseIteratorReader(slot);
            node.deferred.reject(error);
            slot.busy = false;
            settleQueued(slot, false, resultObject(undefined, true));
        }
    }

    function asyncIteratorReturn(value) {
        var slot = Core.get(iteratorSlots, this);
        if (slot === undefined) {
            return rejected(typeError("ReadableStream async iterator return called on an incompatible receiver"));
        }
        var result = resultObject(value, true);
        if (slot.returnPending) {
            var priorReturn = slot.returnPromise;
            var chained = Core.deferred();
            slot.returnPromise = chained.promise;
            var chainedGeneration = ++slot.returnGeneration;
            Core.react(priorReturn, function () {
                chained.resolve(result);
                if (chainedGeneration === slot.returnGeneration) {
                    slot.returnPending = false;
                    processIterator(slot);
                }
                return undefined;
            }, function (error) {
                chained.reject(error);
                if (chainedGeneration === slot.returnGeneration) {
                    slot.returnPending = false;
                    processIterator(slot);
                }
                return undefined;
            });
            return chained.promise;
        }
        if (slot.done || slot.released) {
            return Core.resolve(result);
        }
        slot.done = true;
        slot.returnPending = true;
        var generation = ++slot.returnGeneration;
        if (slot.preventCancel) {
            releaseIteratorReader(slot);
            var preventCancelReturn = Core.resolve(result);
            slot.returnPromise = preventCancelReturn;
            Core.react(preventCancelReturn, function () {
                if (generation === slot.returnGeneration) {
                    slot.returnPending = false;
                    processIterator(slot);
                }
                return undefined;
            }, function () {
                if (generation === slot.returnGeneration) {
                    slot.returnPending = false;
                    processIterator(slot);
                }
                return undefined;
            });
            return preventCancelReturn;
        }
        var cancelPromise;
        var completion = Core.deferred();
        slot.returnPromise = completion.promise;
        try {
            cancelPromise = Core.resolve(ReadableOps.readerCancel(slot.reader, value));
        } catch (error) {
            releaseIteratorReader(slot);
            completion.reject(error);
            if (generation === slot.returnGeneration) {
                slot.returnPending = false;
                processIterator(slot);
            }
            return completion.promise;
        }
        // Cancellation closes the readable side synchronously, so release the
        // lock now; the returned promise still waits for the cancel algorithm.
        releaseIteratorReader(slot);
        Core.react(cancelPromise, function () {
            completion.resolve(result);
            if (generation === slot.returnGeneration) {
                slot.returnPending = false;
                processIterator(slot);
            }
            return undefined;
        }, function (error) {
            completion.reject(error);
            if (generation === slot.returnGeneration) {
                slot.returnPending = false;
                processIterator(slot);
            }
            return undefined;
        });
        return completion.promise;
    }

    function values(optionsValue) {
        if (!ReadableOps.isReadable(this)) {
            throw typeError("ReadableStream.values called on an incompatible receiver");
        }
        var preventCancel = !!dictionaryValue(optionsValue, "preventCancel");
        var reader = ReadableOps.acquireReader(this, "default");
        var iterator = Core.create(iteratorPrototype);
        var slot = Core.record();
        slot.reader = reader;
        slot.preventCancel = preventCancel;
        slot.done = false;
        slot.released = false;
        slot.busy = false;
        slot.head = undefined;
        slot.tail = undefined;
        slot.returnPromise = undefined;
        slot.returnPending = false;
        slot.returnGeneration = 0;
        Core.set(iteratorSlots, iterator, slot);
        return iterator;
    }

    function from(input) {
        if (input === undefined || input === null) {
            throw typeError("ReadableStream.from requires an iterable");
        }
        var asyncMethod;
        var syncMethod;
        var iterator;
        var nextMethod;
        var synchronous = false;
        try {
            asyncMethod = input[asyncIteratorSymbol];
            if (asyncMethod !== undefined && asyncMethod !== null) {
                if (!callable(asyncMethod)) {
                    throw typeError("The async iterator method is not callable");
                }
                iterator = Core.call(asyncMethod, input, []);
            } else {
                syncMethod = input[Core.Symbol.iterator];
                if (syncMethod === undefined || syncMethod === null || !callable(syncMethod)) {
                    throw typeError("The value is not async iterable or iterable");
                }
                iterator = Core.call(syncMethod, input, []);
                synchronous = true;
            }
            if (!isObject(iterator)) {
                throw typeError("The iterator method did not return an object");
            }
            nextMethod = iterator.next;
            if (!callable(nextMethod)) {
                throw typeError("The iterator next method is not callable");
            }
        } catch (error) {
            throw error;
        }

        var stream;
        var finished = false;
        var pulling = false;

        function next(controller) {
            var operation = Core.deferred();
            if (finished) {
                operation.resolve(undefined);
                return operation.promise;
            }
            if (pulling) {
                operation.reject(typeError("The iterator is already being advanced"));
                return operation.promise;
            }
            pulling = true;
            var nextResult;
            try {
                nextResult = Core.call(nextMethod, iterator, []);
            } catch (error) {
                pulling = false;
                Core.call(controllerError, controller, [error]);
                operation.reject(error);
                return operation.promise;
            }
            Core.react(Core.resolve(nextResult), function (result) {
                if (!isObject(result)) {
                    pulling = false;
                    var error = typeError("Iterator result is not an object");
                    Core.call(controllerError, controller, [error]);
                    operation.reject(error);
                    return undefined;
                }
                var done;
                var value;
                try {
                    done = !!result.done;
                    value = result.value;
                } catch (error) {
                    pulling = false;
                    Core.call(controllerError, controller, [error]);
                    operation.reject(error);
                    return undefined;
                }
                var enqueueValue = function (item) {
                    try {
                        if (done) {
                            finished = true;
                            Core.call(controllerClose, controller, []);
                        } else {
                            Core.call(controllerEnqueue, controller, [item]);
                        }
                        operation.resolve(undefined);
                    } catch (error) {
                        Core.call(controllerError, controller, [error]);
                        operation.reject(error);
                    }
                    pulling = false;
                };
                if (synchronous) {
                    var valuePromise;
                    try { valuePromise = Core.resolve(value); }
                    catch (error) {
                        pulling = false;
                        Core.call(controllerError, controller, [error]);
                        operation.reject(error);
                        return undefined;
                    }
                    Core.react(valuePromise, function (resolvedValue) {
                        enqueueValue(resolvedValue);
                        return undefined;
                    }, function (error) {
                        pulling = false;
                        Core.call(controllerError, controller, [error]);
                        operation.reject(error);
                        return undefined;
                    });
                } else {
                    enqueueValue(value);
                }
                return undefined;
            }, function (error) {
                pulling = false;
                Core.call(controllerError, controller, [error]);
                operation.reject(error);
                return undefined;
            });
            return operation.promise;
        }

        function cancel(reason) {
            if (finished) {
                return Core.resolve(undefined);
            }
            finished = true;
            var returnMethod;
            try {
                returnMethod = iterator.return;
            } catch (error) {
                return rejected(error);
            }
            if (returnMethod === undefined || returnMethod === null) {
                return Core.resolve(undefined);
            }
            if (!callable(returnMethod)) {
                return rejected(typeError("The iterator return method is not callable"));
            }
            var returned;
            try {
                returned = Core.call(returnMethod, iterator, [reason]);
            } catch (error) {
                return rejected(error);
            }
            var cancelPromise = Core.resolve(returned);
            var completion = Core.deferred();
            Core.react(cancelPromise, function (result) {
                if (!isObject(result)) {
                    completion.reject(typeError("Iterator return result is not an object"));
                } else {
                    completion.resolve(undefined);
                }
                return undefined;
            }, function (error) {
                completion.reject(error);
                return undefined;
            });
            return completion.promise;
        }

        var underlyingSource = Core.record();
        underlyingSource.pull = function (controller) {
            return next(controller);
        };
        underlyingSource.cancel = cancel;
        var strategy = Core.record();
        strategy.highWaterMark = 0;
        stream = new ReadableStream(underlyingSource, strategy);
        return stream;
    }

    function requiredHighWaterMark(init) {
        if (init === undefined || init === null) {
            throw typeError("Queuing strategy init requires highWaterMark");
        }
        var value = init.highWaterMark;
        if (value === undefined) {
            throw typeError("Queuing strategy init requires highWaterMark");
        }
        // Unary plus follows Web IDL's ToNumber and rejects BigInt.
        return +value;
    }

    function createStrategyConstructor(name, slotMap, sizeFunction) {
        var constructor = name === "CountQueuingStrategy"
            ? function CountQueuingStrategy(init) {
                if (!new.target) {
                    throw typeError("Constructor must be called with new");
                }
                Core.set(slotMap, this, {
                    highWaterMark: requiredHighWaterMark(init),
                    size: sizeFunction
                });
            }
            : function ByteLengthQueuingStrategy(init) {
                if (!new.target) {
                    throw typeError("Constructor must be called with new");
                }
                Core.set(slotMap, this, {
                    highWaterMark: requiredHighWaterMark(init),
                    size: sizeFunction
                });
            };
        var prototype = constructor.prototype;
        defineGetter(prototype, "highWaterMark", function highWaterMark() {
            var slot = Core.get(slotMap, this);
            if (slot === undefined) {
                throw typeError("Illegal invocation");
            }
            return slot.highWaterMark;
        });
        defineGetter(prototype, "size", function size() {
            var slot = Core.get(slotMap, this);
            if (slot === undefined) {
                throw typeError("Illegal invocation");
            }
            return slot.size;
        });
        Core.define(prototype, "constructor", descriptor(constructor, true, false, true));
        Core.define(prototype, toStringTagSymbol, descriptor(name, false, false, true));
        Core.define(constructor, "prototype", descriptor(prototype, false, false, false));
        return constructor;
    }

    var CountQueuingStrategy = createStrategyConstructor(
        "CountQueuingStrategy", countStrategySlots, countSizeFunction);
    var ByteLengthQueuingStrategy = createStrategyConstructor(
        "ByteLengthQueuingStrategy", byteLengthStrategySlots, byteLengthSizeFunction);

    var iteratorPrototype = Core.create(Core.asyncIteratorPrototype || Core.objectPrototype);
    defineFunctionName(asyncIteratorNext, "next");
    defineFunctionName(asyncIteratorReturn, "return");
    defineMethod(iteratorPrototype, "next", asyncIteratorNext, 0);
    defineMethod(iteratorPrototype, "return", asyncIteratorReturn, 1);
    Core.define(iteratorPrototype, toStringTagSymbol,
        descriptor("ReadableStreamAsyncIterator", false, false, true));

    var readablePrototype = ReadableStream.prototype;
    defineMethod(readablePrototype, "pipeTo", pipeTo, 1);
    defineMethod(readablePrototype, "pipeThrough", pipeThrough, 1);
    defineMethod(readablePrototype, "values", values, 0);
    defineMethod(readablePrototype, asyncIteratorSymbol, values, 0);
    defineMethod(ReadableStream, "from", from, 1);

    Core.define(globalThis, "CountQueuingStrategy",
        descriptor(CountQueuingStrategy, true, false, true));
    Core.define(globalThis, "ByteLengthQueuingStrategy",
        descriptor(ByteLengthQueuingStrategy, true, false, true));

    function createProxy(stream) {
        if (!ReadableOps.isReadable(stream)) {
            throw typeError("Request body proxy requires a ReadableStream");
        }
        var transform = WritableOps.createIdentityTransform();
        var options = Core.record();
        options.preventClose = false;
        options.preventAbort = false;
        options.preventCancel = false;
        options.signal = undefined;
        var pipePromise = pipeToInternal(stream, transform.writable, options);
        Core.handled(pipePromise);
        return transform.readable;
    }

    return {
        createProxy: createProxy
    };
})(Core, ReadableOps, WritableOps);
