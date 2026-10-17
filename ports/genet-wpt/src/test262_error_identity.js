// Installed after sta.js, only for tests whose negative type is Test262Error.
// The returned predicate stays in Rust. Authored code cannot reach the set.
(function () {
  const Original = globalThis.Test262Error;
  if (typeof Original !== 'function') throw new TypeError('missing Test262Error harness');
  const seen = new WeakSet();
  const add = WeakSet.prototype.add;
  const has = WeakSet.prototype.has;
  const apply = Reflect.apply;
  // sta.js's constructor sets `this.message` and returns no object. Calling it
  // with the same receiver preserves both ordinary calls and `new` behavior.
  function Wrapped(message) {
    const result = apply(Original, this, arguments);
    if (new.target !== undefined) apply(add, seen, [this]);
    return result;
  }
  Wrapped.prototype = Original.prototype;
  // Copy the harness constructor's own statics, name, and length. Do not put
  // Original in Wrapped's prototype chain: that would expose an untracked
  // constructor and make `thrower` cease to be an own property.
  for (const key of Reflect.ownKeys(Original)) {
    if (key === 'prototype' || key === 'arguments' || key === 'caller') continue;
    Object.defineProperty(Wrapped, key, Object.getOwnPropertyDescriptor(Original, key));
  }
  // Preserve the constructor identity exposed by the harness prototype, so
  // assert.throws-style checks still compare against the public constructor.
  Original.prototype.constructor = Wrapped;
  globalThis.Test262Error = Wrapped;
  return function (value) {
    return apply(has, seen, [value]) ? '1' : '0';
  };
})()
