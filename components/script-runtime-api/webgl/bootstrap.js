// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

(function() {
  'use strict';
  var O = Object, W = WeakMap, N = Number, S = String;
  var create = O.create, define = O.defineProperty;
  var floor = Math.floor, max = Math.max, min = Math.min, parseInteger = parseInt;
  var charCode = S.prototype.charCodeAt;
  var wmGet = W.prototype.get, wmSet = W.prototype.set;
  var wmHas = W.prototype.has, apply = Reflect.apply;
  function get(map, key) { return apply(wmGet, map, [key]); }
  function set(map, key, value) { apply(wmSet, map, [key, value]); return value; }
  function has(map, key) { return apply(wmHas, map, [key]); }
  var agent = globalThis.__agentTimers;
  var contexts = agent.webglContexts || (agent.webglContexts = new W());
  var resources = agent.webglResources || (agent.webglResources = new W());
  var lifetimes = agent.webglRealmLifetimes || (agent.webglRealmLifetimes = create(null));
  var realm = globalThis.__realmId || 0;
  // Each bootstrap owns its record even if the realm id is later reused.
  // Snapshot restoration rotates only the record's current token, leaving
  // inherited contexts tied to the old token.
  var realmRecord = lifetimes[realm] = { current: { live: true }, closed: false };
  function lifetime() {
    if (realmRecord.closed || !realmRecord.current.live)
      throw new TypeError('WebGL realm is gone');
    return realmRecord.current;
  }
  // Surface::set_function makes these setup-only globals configurable. After
  // capture, authored script cannot call the registry-indexed native sinks.
  var native = create(null);
  var sinkNames = 'create_context external_texture_key resize_context clear_color clear enable disable is_enabled color_mask viewport get_error read_pixels_rgba8 create_buffer bind_buffer buffer_data_f32 create_shader shader_source compile_shader get_shader_compile_status get_shader_info_log create_program attach_shader link_program get_program_link_status get_program_info_log use_program get_attrib_location get_uniform_location enable_vertex_attrib_array vertex_attrib_pointer uniform4f uniform_matrix4fv uniform1i create_texture bind_texture_2d active_texture tex_image_2d_rgba8 draw_arrays'.split(' ');
  for (var si = 0; si < sinkNames.length; si++) {
    var name = '__webgl_' + sinkNames[si];
    native[sinkNames[si]] = globalThis[name];
    if (typeof native[sinkNames[si]] !== 'function' || !delete globalThis[name])
      throw new TypeError('WebGL native sink was not hidden');
  }
  // -----------------------------------------------------------------
  // GLenum constants. Numbers only — JS spec values; the host
  // re-maps them to its own enums.
  // -----------------------------------------------------------------
  var K = {
    DEPTH_BUFFER_BIT: 0x0100, STENCIL_BUFFER_BIT: 0x0400, COLOR_BUFFER_BIT: 0x4000,
    POINTS: 0x0000, LINES: 0x0001, LINE_LOOP: 0x0002, LINE_STRIP: 0x0003,
    TRIANGLES: 0x0004, TRIANGLE_STRIP: 0x0005, TRIANGLE_FAN: 0x0006,
    ARRAY_BUFFER: 0x8892, ELEMENT_ARRAY_BUFFER: 0x8893,
    STATIC_DRAW: 0x88E4, DYNAMIC_DRAW: 0x88E8, STREAM_DRAW: 0x88E0,
    FLOAT: 0x1406, INT: 0x1404, UNSIGNED_BYTE: 0x1401, UNSIGNED_SHORT: 0x1403,
    VERTEX_SHADER: 0x8B31, FRAGMENT_SHADER: 0x8B30,
    COMPILE_STATUS: 0x8B81, LINK_STATUS: 0x8B82,
    NO_ERROR: 0x0000, INVALID_ENUM: 0x0500, INVALID_VALUE: 0x0501,
    INVALID_OPERATION: 0x0502, INVALID_FRAMEBUFFER_OPERATION: 0x0506,
    CONTEXT_LOST_WEBGL: 0x9242,
    // Capabilities for enable/disable/isEnabled.
    CULL_FACE: 0x0B44, DEPTH_TEST: 0x0B71, BLEND: 0x0BE2,
    DITHER: 0x0BD0, SCISSOR_TEST: 0x0C11, STENCIL_TEST: 0x0B90,
    POLYGON_OFFSET_FILL: 0x8037, SAMPLE_ALPHA_TO_COVERAGE: 0x809E,
    SAMPLE_COVERAGE: 0x80A0,
    // Textures.
    TEXTURE_2D: 0x0DE1, TEXTURE_CUBE_MAP: 0x8513, RGBA: 0x1908, RGB: 0x1907,
    TEXTURE0: 0x84C0, TEXTURE1: 0x84C1, TEXTURE2: 0x84C2,
    TEXTURE_MIN_FILTER: 0x2801, TEXTURE_MAG_FILTER: 0x2800,
    TEXTURE_WRAP_S: 0x2802, TEXTURE_WRAP_T: 0x2803,
    NEAREST: 0x2600, LINEAR: 0x2601, CLAMP_TO_EDGE: 0x812F, REPEAT: 0x2901,
  };

  // -----------------------------------------------------------------
  // Public constructors carry no authority. Only the private maps do.
  // -----------------------------------------------------------------
  function WebGLBuffer() { throw new TypeError('Illegal constructor'); }
  function WebGLShader() { throw new TypeError('Illegal constructor'); }
  function WebGLProgram() { throw new TypeError('Illegal constructor'); }
  function WebGLTexture() { throw new TypeError('Illegal constructor'); }
  function WebGLUniformLocation() { throw new TypeError('Illegal constructor'); }
  function contextState(value) {
    if (!has(contexts, value)) throw new TypeError('Illegal WebGL receiver');
    var state = get(contexts, value);
    if (!state.life.live) throw new TypeError('WebGL context is gone');
    return state;
  }
  function checkedSink(state, sink) {
    return function() {
      if (!state.life.live) throw new TypeError('WebGL context is gone');
      return apply(sink, undefined, arguments);
    };
  }
  function checkedOps(state) {
    var ops = create(null);
    for (var i = 0; i < sinkNames.length; i++) {
      var name = sinkNames[i];
      ops[name] = checkedSink(state, native[name]);
    }
    return ops;
  }
  function resourceState(ctx, value, kind) {
    if (!has(resources, value)) throw new TypeError('Invalid WebGL resource');
    var state = get(resources, value);
    if (state.kind !== kind || state.owner !== ctx || !state.live)
      throw new TypeError('WebGL resource belongs to another context or is deleted');
    return state;
  }
  function resourceId(ctx, value, kind, nullable) {
    if (nullable && value == null) return '0';
    return resourceState(ctx, value, kind).id;
  }
  function makeResource(ctx, constructor, kind, id, program) {
    if (id === '0' && kind !== 'uniform') return null;
    var value = create(constructor.prototype);
    set(resources, value, { owner: ctx, kind: kind, id: id, live: true,
      program: program || null, epoch: program ? program.epoch : 0 });
    return value;
  }
  function isResource(ctx, value, kind) {
    var state = has(resources, value) ? get(resources, value) : null;
    return !!(state && state.kind === kind && state.owner === ctx && state.live);
  }
  function deleteResource(ctx, value, kind) {
    if (value == null) return;
    var state = resourceState(ctx, value, kind);
    state.live = false;
    if (kind === 'program') state.epoch++;
  }
  function locationId(ctx, loc) {
    var state = resourceState(ctx, loc, 'uniform');
    if (!state.program.live || state.program.epoch !== state.epoch)
      throw new TypeError('Stale WebGL uniform location');
    return state.id;
  }
  function finite(value) {
    var n = N(value);
    if (!N.isFinite(n)) throw new TypeError('Finite WebGL number required');
    return n;
  }
  function number(value) { return N(value); }
  function u32(value) {
    var n = finite(value);
    if (n < 0 || n > 4294967295 || floor(n) !== n)
      throw new TypeError('WebGL unsigned integer required');
    return n;
  }
  function i32(value) {
    var n = finite(value);
    if (n < -2147483648 || n > 2147483647 || floor(n) !== n)
      throw new TypeError('WebGL signed integer required');
    return n;
  }
  var F32 = Float32Array, U8 = Uint8Array;
  function floatView(value) {
    var view;
    if (typeof value === 'number') {
      var bytes = u32(value);
      if (bytes % 4) throw new TypeError('WebGL buffer byte size must be divisible by four');
      view = new F32(bytes / 4);
    } else {
      if (value == null) throw new TypeError('WebGL buffer data required');
      view = new F32(value);
    }
    return view;
  }

  // -----------------------------------------------------------------
  // Readback still crosses as a byte string until the sink's return
  // shape becomes a native Uint8Array. Input views cross as genuine views.
  // -----------------------------------------------------------------
  function unpackBinary(s) {
    var n = s.length | 0;
    var out = new U8(n);
    for (var i = 0; i < n; i++) out[i] = apply(charCode, s, [i]) & 0xFF;
    return out;
  }
  // -----------------------------------------------------------------
  // WebGLRenderingContext: the Triangle-class subset. Native ids are
  // only reachable through the context's private state.
  // -----------------------------------------------------------------
  var constructionToken = {};
  function WebGLRenderingContext(token, contextId, width, height) {
    if (token !== constructionToken) throw new TypeError('Illegal constructor');
    // GLenum constants on the instance, per the WebGL IDL.
    for (var k in K) { if (K.hasOwnProperty(k)) this[k] = K[k]; }
    // Creation is attempted before this constructor runs, so an unavailable
    // host context never becomes a context-shaped JS object.
    var state = { id: contextId, life: lifetime() };
    state.ops = checkedOps(state);
    set(contexts, this, state);
    this.drawingBufferWidth = width;
    this.drawingBufferHeight = height;
    state.externalTextureKey = state.ops.external_texture_key(S(contextId));
    define(this, '_externalTextureKey', { configurable: false,
      get: function() { return contextState(this).externalTextureKey; } });
  }
  var P = WebGLRenderingContext.prototype;

  P._resizeDrawingBuffer = function(width, height) {
    var ctx = contextState(this), w = max(1, u32(width)), h = max(1, u32(height));
    ctx.ops.resize_context(S(ctx.id), w, h);
    this.drawingBufferWidth = w;
    this.drawingBufferHeight = h;
  };

  // State / framebuffer.
  P.clearColor = function(r, g, b, a) {
    var ctx = contextState(this);
    ctx.ops.clear_color(S(ctx.id), number(r), number(g), number(b), number(a));
  };
  P.clear = function(mask) { var ctx = contextState(this); ctx.ops.clear(S(ctx.id), u32(mask)); };
  P.enable = function(cap) { var ctx = contextState(this); ctx.ops.enable(S(ctx.id), u32(cap)); };
  P.disable = function(cap) { var ctx = contextState(this); ctx.ops.disable(S(ctx.id), u32(cap)); };
  P.isEnabled = function(cap) {
    var ctx = contextState(this);
    return ctx.ops.is_enabled(S(ctx.id), u32(cap)) === '1';
  };
  P.colorMask = function(r, g, b, a) {
    var ctx = contextState(this);
    ctx.ops.color_mask(S(ctx.id),
      r ? '1' : '0', g ? '1' : '0', b ? '1' : '0', a ? '1' : '0');
  };
  P.viewport = function(x, y, w, h) {
    var ctx = contextState(this);
    ctx.ops.viewport(S(ctx.id), i32(x), i32(y), u32(w), u32(h));
  };
  P.getError = function() { var ctx = contextState(this); return parseInteger(ctx.ops.get_error(S(ctx.id)), 10) | 0; };
  P.readPixels = function(x, y, w, h, format, type, dst) {
    // The smoke uses RGBA / UNSIGNED_BYTE only — a richer pixel-pack
    // path lands with the broader read-back conformance.
    var ctx = contextState(this);
    var packed = ctx.ops.read_pixels_rgba8(S(ctx.id), i32(x), i32(y), u32(w), u32(h));
    var bytes = unpackBinary(packed);
    if (dst) {
      var n = min(dst.length | 0, bytes.length | 0);
      for (var i = 0; i < n; i++) dst[i] = bytes[i];
    }
    return bytes;
  };

  // Buffers.
  P.createBuffer = function() {
    var ctx = contextState(this);
    return makeResource(ctx, WebGLBuffer, 'buffer', ctx.ops.create_buffer(S(ctx.id)));
  };
  P.bindBuffer = function(target, buf) {
    var ctx = contextState(this);
    var targetValue = u32(target);
    ctx.ops.bind_buffer(S(ctx.id), targetValue, resourceId(ctx, buf, 'buffer', true));
  };
  P.bufferData = function(target, srcOrSize, usage) {
    var ctx = contextState(this);
    ctx.ops.buffer_data_f32(S(ctx.id), u32(target), floatView(srcOrSize),
      usage === undefined ? K.STATIC_DRAW : u32(usage));
  };

  // Shaders.
  P.createShader = function(stage) {
    var ctx = contextState(this);
    return makeResource(ctx, WebGLShader, 'shader', ctx.ops.create_shader(S(ctx.id), u32(stage)));
  };
  P.shaderSource = function(shader, source) {
    var ctx = contextState(this);
    var text = S(source);
    ctx.ops.shader_source(S(ctx.id), resourceId(ctx, shader, 'shader'), text);
  };
  P.compileShader = function(shader) {
    var ctx = contextState(this);
    ctx.ops.compile_shader(S(ctx.id), resourceId(ctx, shader, 'shader'));
  };
  P.getShaderParameter = function(shader, pname) {
    var ctx = contextState(this), id = resourceId(ctx, shader, 'shader');
    if (pname === K.COMPILE_STATUS) {
      return ctx.ops.get_shader_compile_status(S(ctx.id), id) === '1';
    }
    return null;
  };
  P.getShaderInfoLog = function(shader) {
    var ctx = contextState(this);
    return ctx.ops.get_shader_info_log(S(ctx.id), resourceId(ctx, shader, 'shader'));
  };

  // Programs.
  P.createProgram = function() {
    var ctx = contextState(this);
    var program = makeResource(ctx, WebGLProgram, 'program', ctx.ops.create_program(S(ctx.id)));
    if (program) get(resources, program).epoch = 0;
    return program;
  };
  P.attachShader = function(program, shader) {
    var ctx = contextState(this);
    ctx.ops.attach_shader(S(ctx.id), resourceId(ctx, program, 'program'), resourceId(ctx, shader, 'shader'));
  };
  P.linkProgram = function(program) {
    var ctx = contextState(this), state = resourceState(ctx, program, 'program');
    state.epoch++;
    ctx.ops.link_program(S(ctx.id), state.id);
  };
  P.getProgramParameter = function(program, pname) {
    var ctx = contextState(this), id = resourceId(ctx, program, 'program');
    if (pname === K.LINK_STATUS) {
      return ctx.ops.get_program_link_status(S(ctx.id), id) === '1';
    }
    return null;
  };
  P.getProgramInfoLog = function(program) {
    var ctx = contextState(this);
    return ctx.ops.get_program_info_log(S(ctx.id), resourceId(ctx, program, 'program'));
  };
  P.useProgram = function(program) {
    var ctx = contextState(this);
    ctx.ops.use_program(S(ctx.id), resourceId(ctx, program, 'program', true));
  };

  // Attributes / uniforms.
  P.getAttribLocation = function(program, name) {
    var ctx = contextState(this);
    var text = S(name);
    var s = ctx.ops.get_attrib_location(S(ctx.id), resourceId(ctx, program, 'program'), text);
    return parseInteger(s, 10) | 0;
  };
  P.getUniformLocation = function(program, name) {
    var ctx = contextState(this), text = S(name);
    var state = resourceState(ctx, program, 'program');
    var s = ctx.ops.get_uniform_location(S(ctx.id), state.id, text);
    var loc = parseInteger(s, 10) | 0;
    return loc < 0 ? null : makeResource(ctx, WebGLUniformLocation, 'uniform', S(loc), state);
  };
  P.enableVertexAttribArray = function(index) {
    var ctx = contextState(this);
    ctx.ops.enable_vertex_attrib_array(S(ctx.id), u32(index));
  };
  P.vertexAttribPointer = function(index, size, type, normalized, stride, offset) {
    // type is always FLOAT for the Triangle-class smoke; the broader
    // surface gates on it.
    if (u32(type) !== K.FLOAT) {
      throw new TypeError('vertexAttribPointer: only FLOAT supported at this layer');
    }
    var ctx = contextState(this);
    ctx.ops.vertex_attrib_pointer(
      S(ctx.id), u32(index), u32(size),
      normalized ? '1' : '0',
      u32(stride), u32(offset)
    );
  };
  P.uniform4f = function(loc, x, y, z, w) {
    var ctx = contextState(this);
    if (loc == null) return;
    var values = [number(x), number(y), number(z), number(w)];
    ctx.ops.uniform4f(S(ctx.id), i32(locationId(ctx, loc)),
      values[0], values[1], values[2], values[3]);
  };
  P.uniformMatrix4fv = function(loc, transpose, value) {
    var ctx = contextState(this);
    if (loc == null) return;
    var matrix = floatView(value);
    if (matrix.length < 16) throw new TypeError('At least 16 matrix values required');
    ctx.ops.uniform_matrix4fv(S(ctx.id), i32(locationId(ctx, loc)),
      transpose ? '1' : '0', matrix);
  };
  P.uniform1i = function(loc, v) {
    var ctx = contextState(this);
    if (loc == null) return;
    var value = i32(v);
    ctx.ops.uniform1i(S(ctx.id), i32(locationId(ctx, loc)), value);
  };

  // Textures.
  P.createTexture = function() {
    var ctx = contextState(this);
    return makeResource(ctx, WebGLTexture, 'texture', ctx.ops.create_texture(S(ctx.id)));
  };
  P.bindTexture = function(target, tex) {
    // Only TEXTURE_2D is wired through this layer; cube maps land with
    // the broader texture conformance.
    var ctx = contextState(this);
    if (u32(target) !== K.TEXTURE_2D) return;
    ctx.ops.bind_texture_2d(S(ctx.id), resourceId(ctx, tex, 'texture', true));
  };
  P.activeTexture = function(unit) {
    // unit is TEXTURE0 + n; the host wants the 0-based index.
    var ctx = contextState(this), value = u32(unit);
    if (value < K.TEXTURE0) throw new TypeError('Invalid texture unit');
    ctx.ops.active_texture(S(ctx.id), u32(value - K.TEXTURE0));
  };
  // texParameteri is accepted but a no-op: webgl-wgpu samples NEAREST /
  // CLAMP_TO_EDGE today, which matches the conformance smoke's needs.
  P.texParameteri = function(target, pname, param) { contextState(this); };
  P.texImage2D = function() {
    var ctx = contextState(this);
    // Support the 9-arg pixel-store form
    //   texImage2D(target, level, internalformat, width, height, border,
    //              format, type, pixels)
    // with an RGBA / UNSIGNED_BYTE Uint8Array (or null for an allocate).
    var a = arguments;
    if (a.length < 9) {
      throw new TypeError('texImage2D: only the 9-arg RGBA8 form is supported here');
    }
    if (u32(a[0]) !== K.TEXTURE_2D) return;
    var width = u32(a[3]), height = u32(a[4]);
    var pixels = a[8] == null ? new U8(0) : new U8(a[8]);
    if (pixels.length && pixels.length !== width * height * 4)
      throw new TypeError('WebGL texture byte length mismatch');
    ctx.ops.tex_image_2d_rgba8(S(ctx.id), width, height, pixels);
  };

  // Draw.
  P.drawArrays = function(mode, first, count) {
    var ctx = contextState(this);
    ctx.ops.draw_arrays(S(ctx.id), u32(mode), i32(first), i32(count));
  };

  // -----------------------------------------------------------------
  // Best-effort conveniences the Khronos test utilities reach for.
  // None have a backend effect yet; they exist so create3DContext /
  // setupProgram / setupTexturedQuad run instead of throwing on a
  // missing method.
  // -----------------------------------------------------------------
  // No extensions are exposed at this layer.
  P.getExtension = function(name) { contextState(this); return null; };
  P.getSupportedExtensions = function() { contextState(this); return []; };
  // getParameter: the conformance utils mostly read this behind feature
  // gates the smoke doesn't hit. Return null (unknown) for now; specific
  // pnames get real values as conformance needs them.
  P.getParameter = function(pname) { contextState(this); return null; };
  // webgl-essl assigns attribute locations in declaration order, which
  // matches how the utils bind them (0, 1, ...), so this is a safe no-op.
  P.bindAttribLocation = function(program, index, name) {
    resourceState(contextState(this), program, 'program');
  };
  // Sampler/pixel-store parameters: we sample NEAREST / CLAMP and treat
  // RGBA8 tightly-packed, which covers the smoke.
  P.texParameterf = function(target, pname, param) { contextState(this); };
  P.pixelStorei = function(pname, param) { contextState(this); };
  // Object deletion: the backend reclaims on context drop; explicit
  // deletes are accepted as no-ops so teardown code runs.
  P.deleteProgram = function(p) { deleteResource(contextState(this), p, 'program'); };
  P.deleteShader = function(s) { deleteResource(contextState(this), s, 'shader'); };
  P.deleteBuffer = function(b) { deleteResource(contextState(this), b, 'buffer'); };
  P.deleteTexture = function(t) { deleteResource(contextState(this), t, 'texture'); };
  P.isProgram = function(p) { return isResource(contextState(this), p, 'program'); };
  P.isShader = function(s) { return isResource(contextState(this), s, 'shader'); };
  P.isBuffer = function(b) { return isResource(contextState(this), b, 'buffer'); };
  P.isTexture = function(t) { return isResource(contextState(this), t, 'texture'); };
  // Cooperative scheduling hooks some tests call; harmless here.
  P.flush = function() { contextState(this); };
  P.finish = function() { contextState(this); };

  // Constructors on the global so tests can `instanceof` them.
  globalThis.WebGLRenderingContext = WebGLRenderingContext;
  globalThis.WebGLBuffer = WebGLBuffer;
  globalThis.WebGLShader = WebGLShader;
  globalThis.WebGLProgram = WebGLProgram;
  globalThis.WebGLTexture = WebGLTexture;
  globalThis.WebGLUniformLocation = WebGLUniformLocation;

  // Host / test helper: a bare context at the given drawing-buffer size
  // (defaults to the HTML canvas 300x150). A missing or refusing factory
  // returns null without constructing a JS context.
  globalThis.__createWebGLContext = function(width, height) {
    lifetime();
    // A canvas resize can deliberately pass zero, which the host clamps to
    // one pixel. Keep the same dimensions for bare and canvas contexts.
    var w = width === undefined ? 300 : max(1, u32(width));
    var h = height === undefined ? 150 : max(1, u32(height));
    lifetime();
    var contextId = parseInteger(native.create_context(w, h), 10);
    if (!(contextId >= 0)) return null;
    return new WebGLRenderingContext(constructionToken, contextId, w, h);
  };
})();
