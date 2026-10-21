/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `performance`: High Resolution Time, User Timing and the Performance Timeline.
//!
//! Every runtime in a process reads one [`SharedClock`] (hr-time's "shared
//! monotonic clock"): a system-wide monotonic source that pauses during system
//! sleep, one estimate of where the Unix epoch falls on it, and an optional
//! skip a test harness adds instead of sleeping to a timer's due time. Each
//! realm's time origin is a reading of it; `performance.now()`, `timeOrigin`
//! and entry times are coarsened to the clock's [`TimeResolution`], and the
//! engine's `Date` reads the same clock (see `ScriptEngine::set_date_source`).
//!
//! `PerformanceObserver` records are delivered as a task on the drive loop (a
//! zero-delay timer), so a callback never runs inside `mark()`.
//!
//! Absent: resource, navigation, paint, longtask and element entry types — those
//! need a network and a rendering timeline the scripted tier does not report.
//! `performance.timing` / `performance.navigation` are present as the legacy
//! zero-valued shapes `performance.toJSON()` is specified to carry.

use std::cell::RefCell;
use std::sync::{Arc, Mutex, OnceLock};

use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::HostState;

/// hr-time's coarsening resolutions, in milliseconds. The spec allows "a
/// higher implementation-defined value", so a host may coarsen further but
/// never below the spec's 100 µs (5 µs when cross-origin isolated).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeResolution {
    pub default_ms: f64,
    pub isolated_ms: f64,
}

impl TimeResolution {
    pub const SPEC: Self = Self {
        default_ms: 0.1,
        isolated_ms: 0.005,
    };

    fn clamped(self) -> Self {
        Self {
            default_ms: self.default_ms.max(Self::SPEC.default_ms),
            isolated_ms: self.isolated_ms.max(Self::SPEC.isolated_ms),
        }
    }
}

impl Default for TimeResolution {
    fn default() -> Self {
        Self::SPEC
    }
}

type Reader = Arc<dyn Fn() -> f64 + Send + Sync>;

struct ClockState {
    read: Reader,
    /// Wall-clock epoch milliseconds minus the monotonic reading, both taken
    /// once when the clock was anchored.
    epoch_offset_ms: f64,
    /// Waits a harness skipped instead of sleeping; zero for a product clock.
    skipped_ms: Mutex<f64>,
    skips: bool,
    resolution: TimeResolution,
}

/// The process's shared monotonic clock. Cheap to clone and `Send`, so one
/// clock reaches every runtime, worker thread and child realm.
#[derive(Clone)]
pub struct SharedClock(Arc<ClockState>);

impl SharedClock {
    /// The process-wide clock: the system monotonic source, anchored once.
    pub fn process() -> Self {
        static PROCESS: OnceLock<SharedClock> = OnceLock::new();
        PROCESS
            .get_or_init(|| Self::anchored(Arc::new(system_monotonic_ms), 0.0, false))
            .clone()
    }

    /// A clock for a test harness: the system source plus the waits the
    /// harness skips (see [`skip`](Self::skip)), so synchronous work still
    /// takes real time while timers fire without sleeping.
    pub fn skipping() -> Self {
        Self::anchored(Arc::new(system_monotonic_ms), 0.0, true)
    }

    /// A clock over a controlled reader, for an embedder or a deterministic
    /// test. `epoch_ms_at_zero` is the Unix time the reader's zero stands for.
    pub fn from_reader(epoch_ms_at_zero: f64, reader: impl Fn() -> f64 + Send + Sync + 'static) -> Self {
        Self(Arc::new(ClockState {
            read: Arc::new(reader),
            epoch_offset_ms: epoch_ms_at_zero,
            skipped_ms: Mutex::new(0.0),
            skips: true,
            resolution: TimeResolution::SPEC,
        }))
    }

    /// Rebuild a clock in another process from [`anchor`](Self::anchor). The
    /// system source is system-wide, so readings line up with the parent's.
    pub fn from_anchor(epoch_offset_ms: f64, skipped_ms: f64) -> Self {
        Self(Arc::new(ClockState {
            read: Arc::new(system_monotonic_ms),
            epoch_offset_ms,
            skipped_ms: Mutex::new(skipped_ms),
            skips: false,
            resolution: TimeResolution::SPEC,
        }))
    }

    fn anchored(read: Reader, skipped_ms: f64, skips: bool) -> Self {
        let mono = read();
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64() * 1_000.0)
            .unwrap_or(0.0);
        Self(Arc::new(ClockState {
            read,
            epoch_offset_ms: wall - mono,
            skipped_ms: Mutex::new(skipped_ms),
            skips,
            resolution: TimeResolution::SPEC,
        }))
    }

    /// The same clock with a coarser resolution; values below the spec's are
    /// raised to it.
    pub fn with_resolution(&self, resolution: TimeResolution) -> Self {
        let state = &self.0;
        Self(Arc::new(ClockState {
            read: state.read.clone(),
            epoch_offset_ms: state.epoch_offset_ms,
            skipped_ms: Mutex::new(self.skipped()),
            skips: state.skips,
            resolution: resolution.clamped(),
        }))
    }

    /// The unsafe (uncoarsened) shared current time, in milliseconds.
    pub fn now_ms(&self) -> f64 {
        (self.0.read)() + self.skipped()
    }

    /// The Unix-epoch milliseconds `Date` reports for the current time.
    pub fn epoch_now_ms(&self) -> f64 {
        self.now_ms() + self.0.epoch_offset_ms
    }

    /// Add a skipped wait. A product clock ignores this.
    pub fn skip(&self, ms: f64) {
        if self.0.skips && ms > 0.0 {
            *self.0.skipped_ms.lock().expect("clock skip") += ms;
        }
    }

    /// Skip just far enough that [`now_ms`](Self::now_ms) reaches `moment_ms`.
    pub fn skip_to(&self, moment_ms: f64) {
        self.skip(moment_ms - self.now_ms());
    }

    pub fn skips(&self) -> bool {
        self.0.skips
    }

    /// What another process needs to rebuild this clock.
    pub fn anchor(&self) -> (f64, f64) {
        (self.0.epoch_offset_ms, self.skipped())
    }

    pub fn resolution(&self) -> TimeResolution {
        self.0.resolution
    }

    /// hr-time "coarsen time": floor to the resolution, no jitter.
    pub fn coarsen(&self, moment_ms: f64, isolated: bool) -> f64 {
        let step_us = if isolated {
            self.0.resolution.isolated_ms
        } else {
            self.0.resolution.default_ms
        } * 1_000.0;
        // Work in microseconds so a value already on the grid stays on it.
        let ticks = (moment_ms * 1_000.0 / step_us + 1e-9).floor();
        ticks * step_us / 1_000.0
    }

    /// A moment as a coarsened Unix-epoch timestamp (`timeOrigin`).
    pub fn coarsen_epoch(&self, moment_ms: f64, isolated: bool) -> f64 {
        self.coarsen(moment_ms + self.0.epoch_offset_ms, isolated)
    }

    fn skipped(&self) -> f64 {
        *self.0.skipped_ms.lock().expect("clock skip")
    }
}

impl Default for SharedClock {
    fn default() -> Self {
        Self::process()
    }
}

/// System-wide monotonic milliseconds that exclude time the system slept.
#[cfg(windows)]
fn system_monotonic_ms() -> f64 {
    use windows_sys::Win32::System::WindowsProgramming::QueryUnbiasedInterruptTimePrecise;
    let mut ticks = 0u64;
    // SAFETY: writes one u64 through a valid pointer.
    unsafe { QueryUnbiasedInterruptTimePrecise(&mut ticks) };
    ticks as f64 / 10_000.0
}

#[cfg(all(unix, not(target_vendor = "apple")))]
fn system_monotonic_ms() -> f64 {
    clock_ms(libc::CLOCK_MONOTONIC)
}

#[cfg(target_vendor = "apple")]
fn system_monotonic_ms() -> f64 {
    clock_ms(libc::CLOCK_UPTIME_RAW)
}

#[cfg(unix)]
fn clock_ms(id: libc::clockid_t) -> f64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: writes one timespec through a valid pointer.
    unsafe { libc::clock_gettime(id, &mut ts) };
    ts.tv_sec as f64 * 1_000.0 + ts.tv_nsec as f64 / 1_000_000.0
}

/// Elsewhere (wasm) the embedder supplies its clock through
/// [`SharedClock::from_reader`]; this fallback only keeps reads monotonic.
#[cfg(not(any(windows, unix)))]
fn system_monotonic_ms() -> f64 {
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_secs_f64() * 1_000.0
}

fn clock_from_call<E: ScriptEngine>(cx: &E::CallCx<'_>) -> Option<SharedClock> {
    let data = cx.host_data()?;
    let host = data.downcast_ref::<RefCell<HostState>>()?;
    let agent = host.borrow().agent.upgrade()?;
    let clock = agent.borrow().clock.clone();
    Some(clock)
}

fn isolated_arg<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Result<bool, E::Error> {
    let value = cx.arg(0);
    Ok(cx.value_to_string(&value)? == "true")
}

/// `__hrNow(isolated)` → the coarsened shared current time.
struct HrNow;
impl<E: ScriptEngine> NativeFn<E> for HrNow {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let isolated = isolated_arg::<E>(cx)?;
        let now = clock_from_call::<E>(cx).map_or(0.0, |c| c.coarsen(c.now_ms(), isolated));
        cx.make_string(&now.to_string())
    }
}

/// `__hrMoment()` → the uncoarsened shared current time: a realm's time origin.
struct HrMoment;
impl<E: ScriptEngine> NativeFn<E> for HrMoment {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let now = clock_from_call::<E>(cx).map_or(0.0, |c| c.now_ms());
        cx.make_string(&now.to_string())
    }
}

/// `__hrEpoch(isolated, moment)` → `moment` as a coarsened Unix-epoch timestamp.
struct HrEpoch;
impl<E: ScriptEngine> NativeFn<E> for HrEpoch {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let isolated = isolated_arg::<E>(cx)?;
        let moment = cx.arg(1);
        let moment = cx.value_to_number(&moment)?;
        let epoch = clock_from_call::<E>(cx).map_or(0.0, |c| c.coarsen_epoch(moment, isolated));
        cx.make_string(&epoch.to_string())
    }
}

pub(crate) fn install_timing_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<HrNow>("__hrNow", 1)?;
    engine.set_function::<HrMoment>("__hrMoment", 0)?;
    engine.set_function::<HrEpoch>("__hrEpoch", 2)?;
    engine.eval(TIMING_BOOTSTRAP)?;
    Ok(())
}

const TIMING_BOOTSTRAP: &str = r#"
(function() {
  // HTML's crossOriginIsolated: no realm here has the COOP/COEP-backed
  // capability yet, so it is false until a host grants it.
  if (!('crossOriginIsolated' in globalThis)) {
    Object.defineProperty(globalThis, 'crossOriginIsolated', {
      configurable: true, enumerable: true, get: function() { return false; }
    });
  }
  var readHrNow = __hrNow;
  var readHrEpoch = __hrEpoch;
  // This realm's time origin: the shared clock when its global was created.
  var origin = Number(__hrMoment());
  function isolated() { return globalThis.crossOriginIsolated === true; }
  var timeOrigin = Number(readHrEpoch(isolated(), origin));
  var schedulePerformanceTask = globalThis.setTimeout;
  var reportPerformanceException = globalThis.__reportListenerException;
  var lastNow = 0;
  function now() {
    var elapsed = Number(readHrNow(isolated())) - origin;
    // A configured reader may briefly regress; never report time going back.
    if (isFinite(elapsed) && elapsed > lastNow) lastNow = elapsed;
    return lastNow;
  }

  function defineTag(ctor, name) {
    if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
      ctor.prototype[Symbol.toStringTag] = name;
    }
  }

  // ---- PerformanceEntry ----
  function PerformanceEntry(name, entryType, startTime, duration) {
    this.name = String(name);
    this.entryType = entryType;
    this.startTime = startTime;
    this.duration = duration;
  }
  PerformanceEntry.prototype.toJSON = function() {
    return { name: this.name, entryType: this.entryType,
             startTime: this.startTime, duration: this.duration };
  };
  defineTag(PerformanceEntry, 'PerformanceEntry');
  globalThis.PerformanceEntry = PerformanceEntry;
  // Not [Serializable]: postMessage / structuredClone of an entry is a DataCloneError.
  if (typeof globalThis.__sc_registerUncloneable === 'function') {
    globalThis.__sc_registerUncloneable(PerformanceEntry, 'PerformanceEntry');
  }

  function PerformanceMark(name, options) {
    var startTime = now();
    var detail = null;
    if (options !== undefined && options !== null) {
      if (options.startTime !== undefined) {
        startTime = Number(options.startTime);
        if (!(startTime >= 0)) throw new TypeError("mark startTime must be non-negative");
      }
      if (options.detail !== undefined && options.detail !== null) {
        detail = globalThis.__structuredClone(options.detail);
      }
    }
    PerformanceEntry.call(this, name, 'mark', startTime, 0);
    this.detail = detail;
  }
  PerformanceMark.prototype = Object.create(PerformanceEntry.prototype);
  PerformanceMark.prototype.constructor = PerformanceMark;
  PerformanceMark.prototype.toJSON = function() {
    var j = PerformanceEntry.prototype.toJSON.call(this);
    j.detail = this.detail;
    return j;
  };
  defineTag(PerformanceMark, 'PerformanceMark');
  globalThis.PerformanceMark = PerformanceMark;

  function PerformanceMeasure(name, startTime, duration, detail) {
    PerformanceEntry.call(this, name, 'measure', startTime, duration);
    this.detail = detail === undefined ? null : detail;
  }
  PerformanceMeasure.prototype = Object.create(PerformanceEntry.prototype);
  PerformanceMeasure.prototype.constructor = PerformanceMeasure;
  PerformanceMeasure.prototype.toJSON = function() {
    var j = PerformanceEntry.prototype.toJSON.call(this);
    j.detail = this.detail;
    return j;
  };
  defineTag(PerformanceMeasure, 'PerformanceMeasure');
  globalThis.PerformanceMeasure = PerformanceMeasure;

  // ---- the timeline ----
  var entries = [];               // in insertion order; getEntries sorts by startTime
  var observers = [];             // registered PerformanceObservers
  var deliveryScheduled = false;

  function sortedCopy(list) {
    var out = list.slice();
    // Stable sort by startTime; insertion order breaks ties.
    out.sort(function(a, b) { return a.startTime - b.startTime; });
    return out;
  }

  function scheduleDelivery() {
    if (deliveryScheduled) return;
    deliveryScheduled = true;
    // The performance timeline task source: never run a callback synchronously
    // inside mark()/measure().
    schedulePerformanceTask(function() {
      deliveryScheduled = false;
      var snapshot = observers.slice();
      for (var i = 0; i < snapshot.length; i++) {
        var o = snapshot[i];
        if (o._buffer.length === 0) continue;
        var records = o._buffer;
        o._buffer = [];
        var dropped = o._dropped; o._dropped = 0;
        try { o._callback.call(o, new PerformanceObserverEntryList(records), o, { droppedEntriesCount: dropped }); }
        catch (ex) { reportPerformanceException(ex); }
      }
    }, 0);
  }

  function addEntry(entry) {
    entries.push(entry);
    for (var i = 0; i < observers.length; i++) {
      if (observers[i]._types[entry.entryType]) observers[i]._buffer.push(entry);
    }
    scheduleDelivery();
    return entry;
  }

  function filterEntries(list, name, type) {
    var out = [];
    for (var i = 0; i < list.length; i++) {
      var e = list[i];
      if (name !== undefined && e.name !== name) continue;
      if (type !== undefined && type !== null && e.entryType !== type) continue;
      out.push(e);
    }
    return out;
  }

  // ---- PerformanceObserverEntryList ----
  function PerformanceObserverEntryList(list) { this._l = sortedCopy(list); }
  PerformanceObserverEntryList.prototype.getEntries = function() { return this._l.slice(); };
  PerformanceObserverEntryList.prototype.getEntriesByType = function(type) {
    return filterEntries(this._l, undefined, String(type));
  };
  PerformanceObserverEntryList.prototype.getEntriesByName = function(name, type) {
    return filterEntries(this._l, String(name), type === undefined ? undefined : String(type));
  };
  defineTag(PerformanceObserverEntryList, 'PerformanceObserverEntryList');
  globalThis.PerformanceObserverEntryList = PerformanceObserverEntryList;

  // ---- PerformanceObserver ----
  var SUPPORTED = ['mark', 'measure'];
  function PerformanceObserver(callback) {
    if (typeof callback !== 'function') {
      throw new TypeError("PerformanceObserver requires a callback");
    }
    this._callback = callback;
    this._buffer = [];
    this._types = {};
    this._dropped = 0;
    this._mode = null;             // 'entryTypes' | 'type', once chosen it is fixed
  }
  PerformanceObserver.prototype.observe = function(options) {
    options = options || {};
    var hasList = options.entryTypes !== undefined;
    var hasType = options.type !== undefined;
    if (hasList && hasType) {
      throw new TypeError("observe(): entryTypes and type are mutually exclusive");
    }
    if (!hasList && !hasType) {
      throw new TypeError("observe(): either entryTypes or type is required");
    }
    if (hasList && this._mode === 'type') {
      throw new DOMException("This observer has performed observe({type}).", "InvalidModificationError");
    }
    if (hasType && this._mode === 'entryTypes') {
      throw new DOMException("This observer has performed observe({entryTypes}).", "InvalidModificationError");
    }
    if (hasList) {
      // A fresh entryTypes list replaces the previous one wholesale.
      this._mode = 'entryTypes';
      this._types = {};
      var list = options.entryTypes;
      if (typeof list !== 'object' || typeof list.length !== 'number') {
        throw new TypeError("observe(): entryTypes is not a sequence");
      }
      var any = false;
      for (var i = 0; i < list.length; i++) {
        if (SUPPORTED.indexOf(String(list[i])) !== -1) { this._types[String(list[i])] = true; any = true; }
      }
      if (!any) { unregister(this); return; }   // nothing supported: no-op, no throw
    } else {
      this._mode = 'type';
      var type = String(options.type);
      if (SUPPORTED.indexOf(type) === -1) return; // unsupported type: no-op, no throw
      this._types[type] = true;
      if (options.buffered) {
        var buffered = filterEntries(entries, undefined, type);
        for (var j = 0; j < buffered.length; j++) this._buffer.push(buffered[j]);
        if (buffered.length) scheduleDelivery();
      }
    }
    if (observers.indexOf(this) === -1) observers.push(this);
  };
  function unregister(o) {
    var i = observers.indexOf(o);
    if (i !== -1) observers.splice(i, 1);
  }
  PerformanceObserver.prototype.disconnect = function() {
    unregister(this);
    this._buffer = [];
    this._types = {};
    this._mode = null;
  };
  PerformanceObserver.prototype.takeRecords = function() {
    var out = sortedCopy(this._buffer);
    this._buffer = [];
    return out;
  };
  // [SameObject]: one frozen array per realm.
  var SUPPORTED_FROZEN = Object.freeze(SUPPORTED.slice());
  Object.defineProperty(PerformanceObserver, 'supportedEntryTypes', {
    configurable: true,
    get: function() { return SUPPORTED_FROZEN; }
  });
  defineTag(PerformanceObserver, 'PerformanceObserver');
  globalThis.PerformanceObserver = PerformanceObserver;

  // ---- legacy PerformanceTiming / PerformanceNavigation ----
  var TIMING_KEYS = ['navigationStart', 'unloadEventStart', 'unloadEventEnd', 'redirectStart',
    'redirectEnd', 'fetchStart', 'domainLookupStart', 'domainLookupEnd', 'connectStart',
    'connectEnd', 'secureConnectionStart', 'requestStart', 'responseStart', 'responseEnd',
    'domLoading', 'domInteractive', 'domContentLoadedEventStart', 'domContentLoadedEventEnd',
    'domComplete', 'loadEventStart', 'loadEventEnd'];
  function PerformanceTiming() {
    for (var i = 0; i < TIMING_KEYS.length; i++) { this[TIMING_KEYS[i]] = 0; }
    this.navigationStart = Math.floor(timeOrigin);
  }
  PerformanceTiming.prototype.toJSON = function() {
    var j = {};
    for (var i = 0; i < TIMING_KEYS.length; i++) { j[TIMING_KEYS[i]] = this[TIMING_KEYS[i]]; }
    return j;
  };
  defineTag(PerformanceTiming, 'PerformanceTiming');
  globalThis.PerformanceTiming = PerformanceTiming;

  function PerformanceNavigation() { this.type = 0; this.redirectCount = 0; }
  PerformanceNavigation.prototype.TYPE_NAVIGATE = PerformanceNavigation.TYPE_NAVIGATE = 0;
  PerformanceNavigation.prototype.TYPE_RELOAD = PerformanceNavigation.TYPE_RELOAD = 1;
  PerformanceNavigation.prototype.TYPE_BACK_FORWARD = PerformanceNavigation.TYPE_BACK_FORWARD = 2;
  PerformanceNavigation.prototype.TYPE_RESERVED = PerformanceNavigation.TYPE_RESERVED = 255;
  PerformanceNavigation.prototype.toJSON = function() {
    return { type: this.type, redirectCount: this.redirectCount };
  };
  defineTag(PerformanceNavigation, 'PerformanceNavigation');
  globalThis.PerformanceNavigation = PerformanceNavigation;

  // ---- Performance ----
  var timing = new PerformanceTiming();
  var navigation = new PerformanceNavigation();

  function markNameIsReserved(name) {
    return TIMING_KEYS.indexOf(name) !== -1;
  }
  function convertMarkToTimestamp(mark) {
    if (typeof mark === 'string') {
      if (markNameIsReserved(mark)) {
        // A navigation-timing attribute name resolves to its (zero) value; a
        // zero one is not a valid measure endpoint.
        throw new DOMException("'" + mark + "' has a value of zero.", "SyntaxError");
      }
      var found = filterEntries(entries, mark, 'mark');
      if (found.length === 0) {
        throw new DOMException("The mark '" + mark + "' does not exist.", "SyntaxError");
      }
      return found[found.length - 1].startTime;
    }
    var n = Number(mark);
    if (!(n >= 0)) throw new TypeError("A timestamp must be non-negative");
    return n;
  }

  function Performance() { EventTarget.call(this); }
  Performance.prototype = Object.create(EventTarget.prototype);
  Performance.prototype.constructor = Performance;
  Performance.prototype.now = function() { return now(); };
  Object.defineProperty(Performance.prototype, 'timeOrigin', {
    configurable: true, enumerable: true, get: function() { return timeOrigin; }
  });
  Object.defineProperty(Performance.prototype, 'timing', {
    configurable: true, enumerable: true, get: function() { return timing; }
  });
  Object.defineProperty(Performance.prototype, 'navigation', {
    configurable: true, enumerable: true, get: function() { return navigation; }
  });
  Performance.prototype.toJSON = function() {
    return { timeOrigin: timeOrigin, timing: timing.toJSON(), navigation: navigation.toJSON() };
  };
  Performance.prototype.getEntries = function() { return sortedCopy(entries); };
  Performance.prototype.getEntriesByType = function(type) {
    return filterEntries(sortedCopy(entries), undefined, String(type));
  };
  Performance.prototype.getEntriesByName = function(name, type) {
    return filterEntries(sortedCopy(entries), String(name), type === undefined || type === null ? undefined : String(type));
  };
  Performance.prototype.mark = function(name, options) {
    if (arguments.length === 0) throw new TypeError("mark requires a name");
    name = String(name);
    if (markNameIsReserved(name)) {
      throw new DOMException("'" + name + "' is a navigation timing attribute.", "SyntaxError");
    }
    return addEntry(new PerformanceMark(name, options));
  };
  Performance.prototype.measure = function(name, startOrOptions, endMark) {
    if (arguments.length === 0) throw new TypeError("measure requires a name");
    name = String(name);
    var start, end, detail = null;
    var isDict = startOrOptions !== null && typeof startOrOptions === 'object';
    if (isDict) {
      if (endMark !== undefined) {
        throw new TypeError("measure(): endMark cannot accompany a measure-options dictionary");
      }
      var hasStart = startOrOptions.start !== undefined;
      var hasEnd = startOrOptions.end !== undefined;
      var hasDur = startOrOptions.duration !== undefined;
      if (hasDur && !(hasStart || hasEnd)) {
        throw new TypeError("measure(): duration needs a start or an end");
      }
      if (hasStart && hasEnd && hasDur) {
        throw new TypeError("measure(): start, end and duration are over-determined");
      }
      if (startOrOptions.detail !== undefined && startOrOptions.detail !== null) {
        detail = globalThis.__structuredClone(startOrOptions.detail);
      }
      if (hasStart) start = convertMarkToTimestamp(startOrOptions.start);
      if (hasEnd) end = convertMarkToTimestamp(startOrOptions.end);
      if (hasDur) {
        var d = convertMarkToTimestamp(startOrOptions.duration);
        if (start === undefined) start = end - d; else end = start + d;
      }
      if (start === undefined) start = 0;
      if (end === undefined) end = now();
    } else {
      start = (startOrOptions === undefined) ? 0 : convertMarkToTimestamp(startOrOptions);
      end = (endMark === undefined) ? now() : convertMarkToTimestamp(endMark);
    }
    return addEntry(new PerformanceMeasure(name, start, end - start, detail));
  };
  Performance.prototype.clearMarks = function(name) {
    entries = filterOut(entries, 'mark', name);
  };
  Performance.prototype.clearMeasures = function(name) {
    entries = filterOut(entries, 'measure', name);
  };
  function filterOut(list, type, name) {
    var out = [];
    for (var i = 0; i < list.length; i++) {
      var e = list[i];
      if (e.entryType === type && (name === undefined || e.name === String(name))) continue;
      out.push(e);
    }
    return out;
  }
  // No resource timeline yet; the buffer controls are inert but present so a
  // feature-detecting test does not fault.
  Performance.prototype.clearResourceTimings = function() {};
  Performance.prototype.setResourceTimingBufferSize = function() {};
  defineTag(Performance, 'Performance');
  globalThis.Performance = Performance;
  globalThis.performance = new Performance();
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coarsening_floors_to_the_spec_grid() {
        let clock = SharedClock::from_reader(0.0, || 0.0);
        assert_eq!(clock.coarsen(12.5, false), 12.5);
        assert_eq!(clock.coarsen(12.5999, false), 12.5);
        assert_eq!(clock.coarsen(12.5049, true), 12.5);
        assert_eq!(clock.coarsen(12.5051, true), 12.505);
    }

    #[test]
    fn host_resolution_never_goes_below_the_spec() {
        let clock = SharedClock::from_reader(0.0, || 0.0).with_resolution(TimeResolution {
            default_ms: 0.001,
            isolated_ms: 0.0001,
        });
        assert_eq!(clock.resolution(), TimeResolution::SPEC);
    }

    #[test]
    fn only_a_skipping_clock_skips() {
        let harness = SharedClock::from_reader(0.0, || 10.0);
        harness.skip_to(25.0);
        assert_eq!(harness.now_ms(), 25.0);
        let product = SharedClock::process();
        let before = product.now_ms();
        product.skip(60_000.0);
        assert!(product.now_ms() - before < 60_000.0);
    }

    #[test]
    fn the_system_source_is_monotonic_and_anchored_to_the_epoch() {
        let clock = SharedClock::process();
        let a = clock.now_ms();
        let b = clock.now_ms();
        assert!(b >= a);
        let wall = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            * 1_000.0;
        assert!((clock.epoch_now_ms() - wall).abs() < 1_000.0);
        // Another process rebuilds the same timeline from the anchor.
        let (offset, skipped) = clock.anchor();
        let child = SharedClock::from_anchor(offset, skipped);
        assert!((child.epoch_now_ms() - clock.epoch_now_ms()).abs() < 50.0);
    }
}
