// WebAssembly.compileStreaming and instantiateStreaming (WebAssembly Web
// API). Evaluated in a realm right after its `WebAssembly` object exists.
(function () {
  var W = WebAssembly, Res = Response, Stream = ReadableStream;
  var compile = W.compile, instantiate = W.instantiate;
  var TE = TypeError, resolve = Promise.resolve.bind(Promise);
  var apply = Function.prototype.call.bind(Function.prototype.call);
  var getter = function (proto, name) {
    return Object.getOwnPropertyDescriptor(proto, name).get;
  };
  // Response keeps ok, type and headers as own data properties; the body
  // getters throw on anything that is not a Response.
  var usedOf = getter(Res.prototype, 'bodyUsed'), bodyOf = getter(Res.prototype, 'body');
  var lockedOf = getter(Stream.prototype, 'locked');
  var headerGet = Headers.prototype.get, readBytes = Res.prototype.arrayBuffer;

  // The body's bytes, once the response passes the spec's checks.
  function bytesOf(source) {
    return resolve(source).then(function (response) {
      var body, mime;
      try {
        body = apply(bodyOf, response);
        mime = apply(headerGet, response.headers, 'Content-Type');
        var type = response.type;
        var ok = response.ok === true && type !== 'opaque' && type !== 'opaqueredirect';
        var usable = !apply(usedOf, response) && (body === null || !apply(lockedOf, body));
      } catch (e) {
        throw new TE('WebAssembly streaming needs a Response');
      }
      if (mime === null || apply(String.prototype.toLowerCase, mime) !== 'application/wasm')
        throw new TE("Response Content-Type is not 'application/wasm'");
      if (!ok) throw new TE('Response is not ok');
      if (!usable) throw new TE('Response body is used or locked');
      return apply(readBytes, response);
    });
  }

  var streaming = {
    compileStreaming(source) {
      return bytesOf(source).then(function (bytes) { return apply(compile, W, bytes); });
    },
    instantiateStreaming(source) {
      var imports = arguments[1];
      return bytesOf(source).then(function (bytes) { return apply(instantiate, W, bytes, imports); });
    },
  };
  for (var name of ['compileStreaming', 'instantiateStreaming'])
    Object.defineProperty(W, name, Object.getOwnPropertyDescriptor(streaming, name));
})();
