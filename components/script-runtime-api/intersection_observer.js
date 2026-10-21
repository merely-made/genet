// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// IntersectionObserver and IntersectionObserverEntry.
//
// Script owns the registry, the init validation and the entry queue; the host
// owns geometry. `__ioUpdate` is the "run the update intersection observations
// steps" algorithm: the host calls it after a rendering turn's rAF callbacks
// and relayout, it asks the host's geometry seam for each target's rects, and
// it queues an entry where the intersecting state or the threshold index moved.
// Delivery is a task, never a callback from inside the update.
(function() {
  var setTimeoutTask = globalThis.setTimeout;
  var reportException = globalThis.__reportListenerException;
  var now = globalThis.performance.now.bind(globalThis.performance);
  var geomIntersection = globalThis.__geomIntersection;
  var RectReadOnly = globalThis.DOMRectReadOnly;
  var slots = new WeakMap();
  var observers = [];           // observers with at least one target
  var deliveryQueued = false;

  // Brand checks that hold for a node from another realm, where `instanceof`
  // does not: a node reflector has its `nodeType` and its host reference.
  function isNode(value, type) {
    return value !== null && typeof value === 'object' && value.nodeType === type &&
           value.__ref !== undefined;
  }

  function internals(object, expected) {
    var slot = slots.get(object);
    if (!slot || slot.kind !== expected) throw new TypeError('Illegal invocation');
    return slot;
  }

  // ---- margins -------------------------------------------------------------

  var NUMBER = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?/;

  function parseMargin(value, name) {
    var text = String(value).replace(/^[ \t\n\f\r]+|[ \t\n\f\r]+$/g, '');
    var out = [];
    if (text !== '') {
      var tokens = text.split(/[ \t\n\f\r]+/);
      if (tokens.length > 4) {
        throw new DOMException(name + ' must have at most four components', 'SyntaxError');
      }
      for (var i = 0; i < tokens.length; i++) {
        var match = NUMBER.exec(tokens[i]);
        var unit = match ? tokens[i].slice(match[0].length) : null;
        var number = match ? Number(match[0]) : NaN;
        if (!match || !isFinite(number) ||
            (unit !== 'px' && unit !== '%' && !(unit === '' && number === 0))) {
          throw new DOMException(name + ' must be specified in pixels or percent', 'SyntaxError');
        }
        out.push({ value: number, percent: unit === '%' });
      }
    }
    if (out.length === 0) out.push({ value: 0, percent: false });
    // top right bottom left, by CSS margin shorthand
    if (out.length === 1) out = [out[0], out[0], out[0], out[0]];
    else if (out.length === 2) out = [out[0], out[1], out[0], out[1]];
    else if (out.length === 3) out = [out[0], out[1], out[2], out[1]];
    return out;
  }

  function serializeMargin(list) {
    return list.map(function(side) { return side.value + (side.percent ? '%' : 'px'); }).join(' ');
  }

  function encodeMargin(list) {
    return list.map(function(side) { return side.value + ',' + (side.percent ? 1 : 0); }).join(',');
  }

  // ---- the entry ----------------------------------------------------------

  function numberOf(value, name) {
    var number = Number(value);
    if (!isFinite(number)) throw new TypeError(name + ' must be a finite number');
    return number;
  }

  function rectInit(init, name) {
    if (init === null || typeof init !== 'object') {
      throw new TypeError(name + ' must be a DOMRectInit');
    }
    function field(key) { return init[key] === undefined ? 0 : Number(init[key]); }
    return new RectReadOnly(field('x'), field('y'), field('width'), field('height'));
  }

  function IntersectionObserverEntry(init) {
    if (!(this instanceof IntersectionObserverEntry)) {
      throw new TypeError("Failed to construct 'IntersectionObserverEntry': Please use the 'new' operator");
    }
    if (arguments.length === 0 || init === null || typeof init !== 'object') {
      throw new TypeError("Failed to construct 'IntersectionObserverEntry': 1 argument required");
    }
    var required = ['time', 'rootBounds', 'boundingClientRect', 'intersectionRect',
                    'isIntersecting', 'isVisible', 'intersectionRatio', 'target'];
    for (var i = 0; i < required.length; i++) {
      if (init[required[i]] === undefined) {
        throw new TypeError("Failed to construct 'IntersectionObserverEntry': required member " + required[i] + ' is undefined');
      }
    }
    if (!isNode(init.target, 1)) {
      throw new TypeError("Failed to construct 'IntersectionObserverEntry': target is not an Element");
    }
    slots.set(this, {
      kind: 'entry',
      time: numberOf(init.time, 'time'),
      rootBounds: init.rootBounds === null ? null : rectInit(init.rootBounds, 'rootBounds'),
      boundingClientRect: rectInit(init.boundingClientRect, 'boundingClientRect'),
      intersectionRect: rectInit(init.intersectionRect, 'intersectionRect'),
      isIntersecting: !!init.isIntersecting,
      isVisible: !!init.isVisible,
      intersectionRatio: Number(init.intersectionRatio),
      target: init.target
    });
  }
  ['time', 'rootBounds', 'boundingClientRect', 'intersectionRect', 'isIntersecting',
   'isVisible', 'intersectionRatio', 'target'].forEach(function(name) {
    Object.defineProperty(IntersectionObserverEntry.prototype, name, {
      configurable: true, enumerable: true,
      get: function() { return internals(this, 'entry')[name]; }
    });
  });
  Object.defineProperty(IntersectionObserverEntry.prototype, Symbol.toStringTag,
    { configurable: true, value: 'IntersectionObserverEntry' });

  function makeEntry(observation, observer, time, report, ratio) {
    var entry = Object.create(IntersectionObserverEntry.prototype);
    slots.set(entry, {
      kind: 'entry',
      time: time,
      rootBounds: report.rootBounds,
      boundingClientRect: report.target,
      intersectionRect: report.intersection,
      isIntersecting: report.isIntersecting,
      isVisible: false,
      intersectionRatio: ratio,
      target: observation.target
    });
    return entry;
  }

  // ---- the observer -------------------------------------------------------

  function IntersectionObserver(callback) {
    if (!(this instanceof IntersectionObserver)) {
      throw new TypeError("Failed to construct 'IntersectionObserver': Please use the 'new' operator");
    }
    if (arguments.length === 0) {
      throw new TypeError("Failed to construct 'IntersectionObserver': 1 argument required");
    }
    if (typeof callback !== 'function') {
      throw new TypeError("Failed to construct 'IntersectionObserver': The callback provided as parameter 1 is not a function");
    }
    var options = arguments[1];
    if (options !== undefined && options !== null && typeof options !== 'object' && typeof options !== 'function') {
      throw new TypeError("Failed to construct 'IntersectionObserver': options is not an object");
    }
    options = options || {};
    var root = options.root === undefined ? null : options.root;
    if (root !== null && !isNode(root, 1) && !isNode(root, 9)) {
      throw new TypeError("Failed to construct 'IntersectionObserver': root must be an Element or a Document");
    }
    var rootMargin = parseMargin(options.rootMargin === undefined ? '0px' : options.rootMargin, 'rootMargin');
    var scrollMargin = parseMargin(options.scrollMargin === undefined ? '0px' : options.scrollMargin, 'scrollMargin');
    var thresholds = options.threshold === undefined ? [0] : options.threshold;
    var list = [];
    if (thresholds !== null && typeof thresholds === 'object' && typeof thresholds[Symbol.iterator] === 'function') {
      for (var item of thresholds) list.push(numberOf(item, 'threshold'));
    } else {
      list.push(numberOf(thresholds, 'threshold'));
    }
    for (var i = 0; i < list.length; i++) {
      if (list[i] < 0 || list[i] > 1) {
        throw new RangeError("Failed to construct 'IntersectionObserver': Threshold values must be numbers between 0 and 1");
      }
    }
    list.sort(function(a, b) { return a - b; });
    if (list.length === 0) list.push(0);
    var delay = options.delay === undefined ? 0 : Math.trunc(Number(options.delay)) || 0;
    var trackVisibility = !!options.trackVisibility;
    if (trackVisibility && delay < 100) {
      throw new DOMException('To enable trackVisibility option, you must also use a delay of at least 100', 'NotSupportedError');
    }
    slots.set(this, {
      kind: 'observer', callback: callback, root: root, rootMargin: rootMargin,
      scrollMargin: scrollMargin, thresholds: Object.freeze(list), delay: delay,
      trackVisibility: trackVisibility, observations: [], queue: []
    });
  }

  function getter(name, read) {
    Object.defineProperty(IntersectionObserver.prototype, name, {
      configurable: true, enumerable: true,
      get: function() { return read(internals(this, 'observer')); }
    });
  }
  getter('root', function(s) { return s.root; });
  getter('rootMargin', function(s) { return serializeMargin(s.rootMargin); });
  getter('scrollMargin', function(s) { return serializeMargin(s.scrollMargin); });
  getter('thresholds', function(s) { return s.thresholds; });
  getter('delay', function(s) { return s.delay; });
  getter('trackVisibility', function(s) { return s.trackVisibility; });

  function register(observer, slot) {
    if (slot.observations.length && observers.indexOf(observer) < 0) observers.push(observer);
    if (!slot.observations.length) {
      var at = observers.indexOf(observer);
      if (at >= 0) observers.splice(at, 1);
    }
  }

  Object.defineProperty(IntersectionObserver.prototype, 'observe', {
    configurable: true, writable: true, enumerable: true,
    value: function observe(target) {
      var slot = internals(this, 'observer');
      if (arguments.length === 0) throw new TypeError("Failed to execute 'observe': 1 argument required");
      if (!isNode(target, 1)) {
        throw new TypeError("Failed to execute 'observe' on 'IntersectionObserver': parameter 1 is not of type 'Element'");
      }
      for (var i = 0; i < slot.observations.length; i++) {
        if (slot.observations[i].target === target) return;
      }
      slot.observations.push({ target: target, previousIndex: -1, previousIntersecting: false });
      register(this, slot);
    }
  });
  Object.defineProperty(IntersectionObserver.prototype, 'unobserve', {
    configurable: true, writable: true, enumerable: true,
    value: function unobserve(target) {
      var slot = internals(this, 'observer');
      if (arguments.length === 0) throw new TypeError("Failed to execute 'unobserve': 1 argument required");
      if (!isNode(target, 1)) {
        throw new TypeError("Failed to execute 'unobserve' on 'IntersectionObserver': parameter 1 is not of type 'Element'");
      }
      slot.observations = slot.observations.filter(function(o) { return o.target !== target; });
      register(this, slot);
    }
  });
  Object.defineProperty(IntersectionObserver.prototype, 'disconnect', {
    configurable: true, writable: true, enumerable: true,
    value: function disconnect() {
      var slot = internals(this, 'observer');
      slot.observations = [];
      register(this, slot);
    }
  });
  Object.defineProperty(IntersectionObserver.prototype, 'takeRecords', {
    configurable: true, writable: true, enumerable: true,
    value: function takeRecords() {
      var slot = internals(this, 'observer');
      var out = slot.queue;
      slot.queue = [];
      return out;
    }
  });
  Object.defineProperty(IntersectionObserver.prototype, Symbol.toStringTag,
    { configurable: true, value: 'IntersectionObserver' });

  // ---- the update and delivery ---------------------------------------------

  function rect(fields) { return new RectReadOnly(fields[0], fields[1], fields[2], fields[3]); }

  function readReport(blob) {
    // target;intersection;rootBounds;isIntersecting
    var parts = String(blob).split(';');
    function numbers(text) { return text === '' ? null : text.split(',').map(Number); }
    var target = numbers(parts[0]) || [0, 0, 0, 0];
    var intersection = numbers(parts[1]) || [0, 0, 0, 0];
    var rootBounds = numbers(parts[2]);
    return {
      targetNumbers: target,
      intersectionNumbers: intersection,
      target: rect(target),
      intersection: rect(intersection),
      rootBounds: rootBounds ? rect(rootBounds) : null,
      isIntersecting: parts[3] === '1'
    };
  }

  function thresholdIndex(thresholds, ratio) {
    for (var i = 0; i < thresholds.length; i++) {
      if (thresholds[i] > ratio) return i;
    }
    return thresholds.length;
  }

  function deliver() {
    deliveryQueued = false;
    var snapshot = observers.slice();
    for (var i = 0; i < snapshot.length; i++) {
      var observer = snapshot[i];
      var slot = slots.get(observer);
      if (!slot.queue.length) continue;
      var entries = slot.queue;
      slot.queue = [];
      try { slot.callback.call(observer, entries, observer); }
      catch (ex) { reportException(ex); }
    }
  }

  // Returns how many entries were queued.
  globalThis.__ioUpdate = function() {
    if (!observers.length || typeof geomIntersection !== 'function') return 0;
    var time = now();
    var queued = 0;
    var snapshot = observers.slice();
    for (var i = 0; i < snapshot.length; i++) {
      var observer = snapshot[i];
      var slot = slots.get(observer);
      var rootRef = slot.root === null || isNode(slot.root, 9) ? '' : slot.root.__ref;
      var margin = encodeMargin(slot.rootMargin);
      var scrollMargin = encodeMargin(slot.scrollMargin);
      for (var j = 0; j < slot.observations.length; j++) {
        var observation = slot.observations[j];
        // A target that never was in the document has nothing to report yet.
        if (observation.previousIndex === -1 && !observation.target.isConnected) continue;
        var blob = geomIntersection(observation.target.__ref, rootRef, margin, scrollMargin);
        if (!blob) continue;
        var report = readReport(blob);
        var targetArea = report.targetNumbers[2] * report.targetNumbers[3];
        var ratio;
        if (targetArea === 0) ratio = report.isIntersecting ? 1 : 0;
        else ratio = Math.min(1, report.intersectionNumbers[2] * report.intersectionNumbers[3] / targetArea);
        var index = report.isIntersecting ? thresholdIndex(slot.thresholds, ratio) : 0;
        // Below the first threshold the target does not count as intersecting,
        // however its edges touch (w3c/IntersectionObserver#432).
        var intersecting = report.isIntersecting && index > 0;
        if (index !== observation.previousIndex || intersecting !== observation.previousIntersecting) {
          report.isIntersecting = intersecting;
          slot.queue.push(makeEntry(observation, observer, time, report, ratio));
          queued++;
        }
        observation.previousIndex = index;
        observation.previousIntersecting = intersecting;
      }
    }
    if (queued && !deliveryQueued) {
      deliveryQueued = true;
      setTimeoutTask(deliver, 0);
    }
    return queued;
  };

  globalThis.IntersectionObserver = IntersectionObserver;
  globalThis.IntersectionObserverEntry = IntersectionObserverEntry;
  delete globalThis.__geomIntersection;
})();
