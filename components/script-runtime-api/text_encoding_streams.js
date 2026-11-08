// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, you can obtain one at https://mozilla.org/MPL/2.0/.

  // Included in the Encoding bootstrap to reuse its captured realm operations.
  var encoderStreamBrands = Core.sharedMap('encoding.encoderStreams');
  var decoderStreamBrands = Core.sharedMap('encoding.decoderStreams');
  var encUnregister = encRegistry ? EncFinalizationRegistry.prototype.unregister : null;
  var encResizableBuffer = encGetDesc(encArrayBuffer.prototype, 'resizable');
  var encGrowableBuffer = encSharedBufferLength
    ? encGetDesc(SharedArrayBuffer.prototype, 'growable') : undefined;

  function encBufferFlag(descriptor, buffer) {
    if (descriptor === undefined) return false;
    try { return encApply(descriptor.get, buffer, []); }
    catch (_) { return false; }
  }
  function encStreamBytes(chunk) {
    var source = encByteSource(chunk, false);
    var buffer = source.value;
    if (source.kind === 'view') {
      var getter = encApply(encTypedArrayTag, chunk, []) === undefined ? encDataBuffer : encTypedBuffer;
      buffer = encApply(getter, chunk, []);
    }
    if (encBufferFlag(encResizableBuffer, buffer) || encBufferFlag(encGrowableBuffer, buffer))
      throw encTypeError('TextDecoderStream requires a fixed-length buffer');
    return encByteString(source);
  }

  function TextEncoderStream() {
    if (!new.target) throw encTypeError("Constructor TextEncoderStream requires 'new'");
    var pending = '';
    var pair = WritableOps.createTransform(function (chunk, controller) {
      var input = encToString(chunk);
      if (input.length === 0) return;
      input = pending + input;
      var last = encApply(encCharCodeAt, input, [input.length - 1]);
      pending = last >= 0xD800 && last <= 0xDBFF ? input[input.length - 1] : '';
      if (pending) input = encApply(encStringSlice, input, [0, -1]);
      if (input.length !== 0)
        WritableOps.transformEnqueue(controller, encUtf8Bytes(encScalarString(input)));
    }, function (controller) {
      if (pending) WritableOps.transformEnqueue(controller, encUtf8Bytes('\uFFFD'));
    });
    pair.encoding = 'utf-8';
    encSet(encoderStreamBrands, this, pair);
  }

  function TextDecoderStream(label, options) {
    if (!new.target) throw encTypeError("Constructor TextDecoderStream requires 'new'");
    var decoder = encCreateDecoder(label, options);
    // The transform callbacks own the native decoder while either side can use
    // it, even when the public wrapper has already been collected.
    encRegisterDecoder(decoder, decoder.id, decoder);
    function release() {
      var id = decoder.id;
      if (id === undefined) return;
      decoder.id = undefined;
      encApply(nativeDecoderRelease, undefined, [id]);
      if (encUnregister) encApply(encUnregister, encRegistry, [decoder]);
    }
    function decode(controller, bytes, stream) {
      var text = encDecodeString(decoder, bytes, stream);
      if (text.length !== 0) WritableOps.transformEnqueue(controller, text);
    }
    var pair;
    try {
      pair = WritableOps.createTransform(function (chunk, controller) {
        try { decode(controller, encStreamBytes(chunk), true); }
        catch (error) { release(); throw error; }
      }, function (controller) {
        try { decode(controller, '', false); }
        finally { release(); }
      }, release);
    } catch (error) {
      release();
      throw error;
    }
    pair.encoding = decoder.encoding;
    pair.fatal = decoder.fatal;
    pair.ignoreBOM = decoder.ignoreBOM;
    encSet(decoderStreamBrands, this, pair);
  }

  function defineEncodingStream(ctor, name, brands, attrs) {
    var prototype = ctor.prototype;
    for (var i = 0; i < attrs.length; i++) {
      var attr = attrs[i];
      encDefine(prototype, attr, { get: encAttribute(brands, attr), enumerable: true, configurable: true });
    }
    encDefine(prototype, Core.Symbol.toStringTag, { value: name, configurable: true });
    defineInterface(name, ctor, 0);
  }
  defineEncodingStream(TextEncoderStream, 'TextEncoderStream', encoderStreamBrands,
    ['encoding', 'readable', 'writable']);
  defineEncodingStream(TextDecoderStream, 'TextDecoderStream', decoderStreamBrands,
    ['encoding', 'fatal', 'ignoreBOM', 'readable', 'writable']);
