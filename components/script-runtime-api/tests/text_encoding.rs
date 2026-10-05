// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Focused controls for the Window and Worker TextDecoder/TextEncoder surfaces.

use std::time::{Duration, Instant};

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct Scripts(Vec<(String, String)>);

impl ScriptResourceLoader for Scripts {
    fn load(&self, url: &str) -> Option<String> {
        self.0
            .iter()
            .find(|(name, _)| name == url)
            .map(|(_, source)| source.clone())
    }
}

fn scripts(pairs: &[(&str, &str)]) -> Box<Scripts> {
    Box::new(Scripts(
        pairs
            .iter()
            .map(|(name, source)| ((*name).to_owned(), (*source).to_owned()))
            .collect(),
    ))
}

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://encoding.test/page.html")
        .expect("base URL");
    runtime.parse_document_interleaved("<html><head></head><body></body></html>", &NoScriptLoader);
    runtime
}

fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, expression: &str) -> String {
    let value = runtime.eval(expression).expect("eval");
    runtime.value_to_string(&value).expect("stringify")
}

fn decode_labels_and_view_offsets<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var d=new TextDecoder('  LaTiN1  ');var bytes=new Uint8Array([65,128,66]);var view=new Uint8Array(bytes.buffer,1,1);return d.encoding+':'+d.decode(view);})()"
        ),
        "windows-1252:€"
    );
    assert_eq!(
        read(&mut runtime, "new TextDecoder(' UTF-8 ').encoding"),
        "utf-8"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var bytes=new Uint8Array([27,36,66,70,124,75,92,27,40,66]);var d=new TextDecoder(' ISO-2022-JP ');return d.encoding+':'+d.decode(bytes);})()"
        ),
        "iso-2022-jp:日本"
    );
}

fn invalid_and_replacement_labels_throw_range_error<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{new TextDecoder('not-a-real-encoding');return 'no error';}catch(e){return e.name;}})()"
        ),
        "RangeError"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{new TextDecoder('replacement');return 'no error';}catch(e){return e.name;}})()"
        ),
        "RangeError"
    );
}

fn fatal_decode_throws_type_error<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array([195]));return 'no error';}catch(e){return e.name;}})()"
        ),
        "TypeError"
    );
}

fn fatal_stream_replays_valid_tail_after_exception<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var d=new TextDecoder('utf-8',{fatal:true});var error='';try{d.decode(new Uint8Array([255,65]),{stream:true});}catch(e){error=e.name;}var tail=d.decode(new Uint8Array(),{stream:true});return error+':'+tail;})()"
        ),
        "TypeError:A"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var d=new TextDecoder('utf-8',{fatal:true});var error='';try{d.decode(new Uint8Array([255]));}catch(e){error=e.name;}return error+':'+d.decode(new Uint8Array([65]));})()"
        ),
        "TypeError:A"
    );
}

fn iso_2022_jp_fatal_stream_replays_escape_tail<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var d=new TextDecoder('iso-2022-jp',{fatal:true});var error='';try{d.decode(new Uint8Array([27,40,65,66]),{stream:true});}catch(e){error=e.name;}return error+':'+d.decode(new Uint8Array(),{stream:true});})()"
        ),
        "TypeError:(AB"
    );
}

fn bom_modes_follow_ignore_bom<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "new TextDecoder().decode(new Uint8Array([239,187,191,65]))"
        ),
        "A"
    );
    assert_eq!(
        read(
            &mut runtime,
            "new TextDecoder('utf-8',{ignoreBOM:true}).decode(new Uint8Array([239,187,191,65]))"
        ),
        "\u{feff}A"
    );
    assert_eq!(
        read(
            &mut runtime,
            "new TextDecoder('utf-8').decode(new Uint8Array([255,254,65,0]))"
        ),
        "��A\u{0}"
    );
}

fn streaming_decode_flushes_and_resets<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var d=new TextDecoder();var a=d.decode(new Uint8Array([240,159]),{stream:true});var b=d.decode(new Uint8Array([152,128]),{stream:true});var c=d.decode();d.decode(new Uint8Array([226]),{stream:true});var flushed=d.decode();var reset=d.decode(new Uint8Array([65]));return [a,b,c,flushed,reset].join('|');})()"
        ),
        "|😀||�|A"
    );
}

fn empty_stream_chunks_preserve_legacy_multibyte_leads<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            r#"(function(){var cases=[['big5',0xfe,0x40],['shift_jis',0x81,0x87],['euc-kr',0x81,0x41]];for(var i=0;i<cases.length;i++){var c=cases[i];var decoder=new TextDecoder(c[0]);var lead=decoder.decode(new Uint8Array([c[1]]),{stream:true});var empty=decoder.decode(new Uint8Array(0),{stream:true});var tail=decoder.decode(new Uint8Array([c[2]]));var whole=new TextDecoder(c[0]).decode(new Uint8Array([c[1],c[2]]));if(lead!==''||empty!==''||tail!==whole)return c[0]+':'+JSON.stringify([lead,empty,tail,whole]);}return 'true';})()"#
        ),
        "true"
    );
}

fn shared_views_cover_sab_buffers_offsets_and_encode_into<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            r#"(function(){var decoder=new TextDecoder();var direct=decoder.decode(new SharedArrayBuffer(3));var directCodes=[direct.charCodeAt(0),direct.charCodeAt(1),direct.charCodeAt(2)].join(',');var shared=new SharedArrayBuffer(8);new Uint8Array(shared).set([0,65,66,67,68,0,0,0]);var typed=decoder.decode(new Uint8Array(shared,2,2));var data=decoder.decode(new DataView(shared,1,4));var ordinary=decoder.decode(new Uint8Array([88,89]));var destBuffer=new SharedArrayBuffer(10);var dest=new Uint8Array(destBuffer,2,6);var result=new TextEncoder().encodeInto('A😀B',dest);var written=[dest[0],dest[1],dest[2],dest[3],dest[4],dest[5]].join(',');return directCodes+'|'+typed+'|'+data+'|'+ordinary+'|'+result.read+'|'+result.written+'|'+written;})()"#
        ),
        "0,0,0|BC|ABCD|XY|4|6|65,240,159,152,128,66"
    );
}

fn text_encoding_raw_protocol_survives_delimiters_and_intrinsic_poison<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            r#"(function(){var slice=Object.getOwnPropertyDescriptor(String.prototype,'slice');var indexOf=Object.getOwnPropertyDescriptor(String.prototype,'indexOf');var parse=Object.getOwnPropertyDescriptor(JSON,'parse');var fail=function(){throw new Error('author replacement observed');};var payload='\0: "quoted" \\ 😀 \uFEFF';var result='';try{Object.defineProperty(String.prototype,'slice',{value:fail,writable:true,enumerable:false,configurable:true});Object.defineProperty(String.prototype,'indexOf',{value:fail,writable:true,enumerable:false,configurable:true});Object.defineProperty(JSON,'parse',{value:fail,writable:true,enumerable:false,configurable:true});var bytes=new TextEncoder().encode(payload);var decoded=new TextDecoder('utf-8',{ignoreBOM:true}).decode(bytes);result=String(decoded===payload);}catch(error){result='error:'+error.message;}finally{Object.defineProperty(String.prototype,'slice',slice);Object.defineProperty(String.prototype,'indexOf',indexOf);Object.defineProperty(JSON,'parse',parse);}return result;})()"#
        ),
        "true"
    );
}

fn encode_into_counts_utf16_and_keeps_character_boundaries<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var e=new TextEncoder();var small=new Uint8Array(4);var partial=e.encodeInto('A😀B',small);var full=new Uint8Array(8);var complete=e.encodeInto('A😀B',full);return [partial.read,partial.written,Array.from(small).join(','),complete.read,complete.written,Array.from(full).slice(0,complete.written).join(',')].join('|');})()"
        ),
        "1|1|65,0,0,0|4|6|65,240,159,152,128,66"
    );
}

fn encoder_replaces_lone_surrogates<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "Array.from(new TextEncoder().encode('\\uD800')).join(',')"
        ),
        "239,191,189"
    );
}

fn encoding_interfaces_enforce_brands_and_constructor_calls<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{TextDecoder();return 'no error';}catch(e){return e.name;}})()"
        ),
        "TypeError"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{TextDecoder.prototype.decode.call({},new Uint8Array());return 'no error';}catch(e){return e.name;}})()"
        ),
        "TypeError"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{TextEncoder.prototype.encode.call({},'x');return 'no error';}catch(e){return e.name;}})()"
        ),
        "TypeError"
    );
    assert_eq!(
        read(
            &mut runtime,
            "(function(){try{TextEncoder.prototype.encodeInto.call({},'x',new Uint8Array());return 'no error';}catch(e){return e.name;}})()"
        ),
        "TypeError"
    );
}

fn borrowed_decoder_methods_and_getters_share_agent_brands<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval("globalThis.frame=document.createElement('iframe');frame.srcdoc='<body></body>';document.body.appendChild(frame);")
        .expect("insert same-origin iframe");
    runtime
        .run_event_loop(100)
        .expect("initialize iframe realm");
    let child = runtime
        .frame_realms(runtime.top_realm())
        .first()
        .map(|(_, realm)| *realm)
        .expect("child realm");
    runtime
        .eval_in_realm(child, "globalThis.decoder=new TextDecoder('utf-8');")
        .expect("create child decoder");

    assert_eq!(
        read(
            &mut runtime,
            "(function(){var child=frame.contentWindow;var parentDecoder=new TextDecoder('utf-8');var parentPrefix=parentDecoder.decode(new Uint8Array([226]),{stream:true});var parentTail=child.TextDecoder.prototype.decode.call(parentDecoder,new Uint8Array([130,172]));var childDecoder=child.decoder;var childPrefix=TextDecoder.prototype.decode.call(childDecoder,new Uint8Array([226]),{stream:true});var childTail=childDecoder.decode(new Uint8Array([130,172]));var parentLegacy=new TextDecoder('latin1');var childGetter=Object.getOwnPropertyDescriptor(child.TextDecoder.prototype,'encoding').get;var parentGetter=Object.getOwnPropertyDescriptor(TextDecoder.prototype,'encoding').get;return [parentPrefix,parentTail,childPrefix,childTail,childGetter.call(parentLegacy),parentGetter.call(childDecoder)].join('|');})()"
        ),
        "|€||€|windows-1252|utf-8"
    );
}

fn borrowed_encoder_methods_accept_foreign_instances_and_views<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval("globalThis.frame=document.createElement('iframe');frame.srcdoc='<body></body>';document.body.appendChild(frame);")
        .expect("insert same-origin iframe");
    runtime
        .run_event_loop(100)
        .expect("initialize iframe realm");
    let child = runtime
        .frame_realms(runtime.top_realm())
        .first()
        .map(|(_, realm)| *realm)
        .expect("child realm");
    runtime
        .eval_in_realm(
            child,
            "globalThis.encoder=new TextEncoder();globalThis.destination=new Uint8Array(4);",
        )
        .expect("create child encoder and view");

    assert_eq!(
        read(
            &mut runtime,
            "(function(){var child=frame.contentWindow;var parentEncoder=new TextEncoder();var childEncoder=child.encoder;var childGetter=Object.getOwnPropertyDescriptor(child.TextEncoder.prototype,'encoding').get;var parentGetter=Object.getOwnPropertyDescriptor(TextEncoder.prototype,'encoding').get;var astral=child.TextEncoder.prototype.encode.call(parentEncoder,'😀');var childDestination=child.destination;var viaParent=TextEncoder.prototype.encodeInto.call(childEncoder,'A😀B',childDestination);var parentDestination=new Uint8Array(4);var viaChild=child.TextEncoder.prototype.encodeInto.call(parentEncoder,'Z😀',parentDestination);return [childGetter.call(parentEncoder),parentGetter.call(childEncoder),Array.from(astral).join(','),viaParent.read,viaParent.written,Array.from(childDestination).join(','),viaChild.read,viaChild.written,Array.from(parentDestination).join(',')].join('|');})()"
        ),
        "utf-8|utf-8|240,159,152,128|1|1|65,0,0,0|1|1|90,0,0,0"
    );
}

fn retained_iframe_decoder_keeps_stream_state_after_removal<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval("globalThis.frame=document.createElement('iframe');frame.srcdoc='<body></body>';document.body.appendChild(frame);")
        .expect("insert same-origin iframe");
    runtime
        .run_event_loop(100)
        .expect("initialize iframe realm");
    let child = runtime
        .frame_realms(runtime.top_realm())
        .first()
        .map(|(_, realm)| *realm)
        .expect("child realm");
    runtime
        .eval_in_realm(
            child,
            "globalThis.decoder=new TextDecoder('utf-8');decoder.decode(new Uint8Array([226]),{stream:true});parent.retainedDecoder=decoder;",
        )
        .expect("retain streaming decoder in parent");
    runtime.eval("frame.remove();").expect("remove iframe");
    runtime
        .run_event_loop(100)
        .expect("finish iframe destruction");

    assert_eq!(
        read(
            &mut runtime,
            "TextDecoder.prototype.decode.call(retainedDecoder,new Uint8Array([130,172]))"
        ),
        "€"
    );
}

fn captured_encoding_internals_survive_author_intrinsic_replacement<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var decode=TextDecoder.prototype.decode;var encode=TextEncoder.prototype.encode;var encodeInto=TextEncoder.prototype.encodeInto;var originals={registry:FinalizationRegistry,uint8:Uint8Array,dataView:DataView,parse:JSON.parse,wmGet:WeakMap.prototype.get,wmSet:WeakMap.prototype.set,wsHas:WeakSet.prototype.has,wsAdd:WeakSet.prototype.add,decode:TextDecoder.prototype.decode,encode:TextEncoder.prototype.encode,encodeInto:TextEncoder.prototype.encodeInto};var nativeNames=Object.getOwnPropertyNames(globalThis).filter(function(name){return /^__(?:text|encoding)/i.test(name);});var nativeDescriptors=[];for(var i=0;i<nativeNames.length;i++){var name=nativeNames[i];var descriptor=Object.getOwnPropertyDescriptor(globalThis,name);if(descriptor&&'value'in descriptor&&(descriptor.writable||descriptor.configurable)){nativeDescriptors.push([name,descriptor]);}}var input=new Uint8Array([65]);var destination=new Uint8Array(1);var poison=function(){throw new Error('author replaced intrinsic');};var result='';try{globalThis.FinalizationRegistry=poison;globalThis.Uint8Array=poison;globalThis.DataView=poison;JSON.parse=poison;WeakMap.prototype.get=poison;WeakMap.prototype.set=poison;WeakSet.prototype.has=poison;WeakSet.prototype.add=poison;for(var j=0;j<nativeDescriptors.length;j++){var native=nativeDescriptors[j];Object.defineProperty(globalThis,native[0],{value:poison,writable:native[1].writable,enumerable:native[1].enumerable,configurable:native[1].configurable});}var decoder=new TextDecoder('utf-8');var encoder=new TextEncoder();var written=encodeInto.call(encoder,'C',destination);var encoded=encode.call(encoder,'B');result=decode.call(decoder,input)+':'+encoded[0]+':'+written.read+':'+written.written+':'+destination[0];}catch(error){result='error:'+error.message;}finally{globalThis.FinalizationRegistry=originals.registry;globalThis.Uint8Array=originals.uint8;globalThis.DataView=originals.dataView;JSON.parse=originals.parse;WeakMap.prototype.get=originals.wmGet;WeakMap.prototype.set=originals.wmSet;WeakSet.prototype.has=originals.wsHas;WeakSet.prototype.add=originals.wsAdd;TextDecoder.prototype.decode=originals.decode;TextEncoder.prototype.encode=originals.encode;TextEncoder.prototype.encodeInto=originals.encodeInto;for(var k=0;k<nativeDescriptors.length;k++){Object.defineProperty(globalThis,nativeDescriptors[k][0],nativeDescriptors[k][1]);}}return result;})()"
        ),
        "A:66:1:1:67"
    );
}

fn encoder_and_decoder_private_arrays_do_not_use_author_array_hooks<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    assert_eq!(
        read(
            &mut runtime,
            "(function(){var arrayZero=Object.getOwnPropertyDescriptor(Array.prototype,'0');var species=Object.getOwnPropertyDescriptor(Array,Symbol.species);var setterCalls=0;var speciesCalls=0;var encoded=-1;var decoded='';var operationError='';try{Object.defineProperty(Array.prototype,'0',{configurable:true,set:function(){setterCalls++;}});Object.defineProperty(Array,Symbol.species,{configurable:true,get:function(){speciesCalls++;throw new Error('Array species observed');}});try{encoded=new TextEncoder().encode('A')[0];decoded=new TextDecoder().decode(new Uint8Array([65]));}catch(error){operationError=error.message;}}finally{if(arrayZero)Object.defineProperty(Array.prototype,'0',arrayZero);else delete Array.prototype[0];if(species)Object.defineProperty(Array,Symbol.species,species);else delete Array[Symbol.species];}return encoded+':'+decoded+':'+setterCalls+':'+speciesCalls+':'+operationError;})()"
        ),
        "65:A:0:0:"
    );
}

fn drive_until<E: ScriptEngine>(runtime: &mut Runtime<E>, done: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut now_ms = 0.0f64;
    while Instant::now() < deadline {
        runtime.run_microtasks();
        let fired = runtime.run_timers(64, now_ms);
        let worked = runtime.pump_workers();
        if read(runtime, done) == "true" {
            return true;
        }
        if let Some(delay) = runtime.next_timer_delay() {
            now_ms += delay.max(0.0);
        }
        if fired == 0 && worked == 0 {
            if !runtime.has_worker_work() && runtime.next_timer_delay().is_none() {
                return read(runtime, done) == "true";
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    false
}

fn worker_has_encoding_surfaces<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime.set_script_resource_loader(scripts(&[(
        "https://encoding.test/encoding-worker.js",
        "var decoder=new TextDecoder('latin1');var bytes=new Uint8Array([0,128,0]);var view=new Uint8Array(bytes.buffer,1,1);var dest=new Uint8Array(4);var encoded=new TextEncoder().encodeInto('A😀B',dest);postMessage({encoding:decoder.encoding,text:decoder.decode(view),read:encoded.read,written:encoded.written});",
    )]));
    runtime
        .eval("globalThis.encodingResult=null;var worker=new Worker('encoding-worker.js');worker.onmessage=function(event){encodingResult=event.data;};worker.onerror=function(event){encodingResult={error:event.message};};")
        .expect("start worker");
    assert!(drive_until(&mut runtime, "String(encodingResult !== null)"));
    let result = read(&mut runtime, "JSON.stringify(encodingResult)");
    assert_eq!(
        read(&mut runtime, "typeof encodingResult.error"),
        "undefined",
        "worker must load and execute the Encoding fixture: {result}"
    );
    assert_eq!(
        read(&mut runtime, "encodingResult.encoding"),
        "windows-1252"
    );
    assert_eq!(read(&mut runtime, "encodingResult.text"), "€");
    assert_eq!(read(&mut runtime, "String(encodingResult.read)"), "1");
    assert_eq!(read(&mut runtime, "String(encodingResult.written)"), "1");
}

fn dropped_decoder_finalizer_is_delivered_by_normal_pump<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval("globalThis.finalized=[];globalThis.decoderRegistry=new FinalizationRegistry(function(value){finalized.push(value);});(function(){var decoder=new TextDecoder();decoderRegistry.register(decoder,'decoder-collected');})();")
        .expect("register decoder finalizer");
    for _ in 0..4 {
        runtime.collect_garbage();
        runtime.run_event_loop(16).expect("pump finalization jobs");
        if read(
            &mut runtime,
            "String(finalized.indexOf('decoder-collected')>=0)",
        ) == "true"
        {
            return;
        }
    }
    assert_eq!(
        read(
            &mut runtime,
            "String(finalized.indexOf('decoder-collected')>=0)"
        ),
        "true",
        "dropping a decoder should eventually deliver its private cleanup job"
    );
}

macro_rules! both_engines {
    ($($body:ident => ($boa:ident, $vano:ident)),* $(,)?) => {
        $(
            #[test]
            fn $boa() { $body::<script_engine_boa::BoaEngine>(); }

            #[cfg(target_pointer_width = "64")]
            #[test]
            fn $vano() { $body::<script_engine_nova::NovaEngine>(); }
        )*
    };
}

both_engines! {
    decode_labels_and_view_offsets => (labels_and_offsets_on_boa, labels_and_offsets_on_vano),
    invalid_and_replacement_labels_throw_range_error => (label_errors_on_boa, label_errors_on_vano),
    fatal_decode_throws_type_error => (fatal_decode_on_boa, fatal_decode_on_vano),
    fatal_stream_replays_valid_tail_after_exception => (fatal_stream_tail_on_boa, fatal_stream_tail_on_vano),
    iso_2022_jp_fatal_stream_replays_escape_tail => (iso_2022_fatal_tail_on_boa, iso_2022_fatal_tail_on_vano),
    bom_modes_follow_ignore_bom => (bom_modes_on_boa, bom_modes_on_vano),
    streaming_decode_flushes_and_resets => (streaming_decode_on_boa, streaming_decode_on_vano),
    empty_stream_chunks_preserve_legacy_multibyte_leads => (empty_stream_legacy_on_boa, empty_stream_legacy_on_vano),
    encode_into_counts_utf16_and_keeps_character_boundaries => (encode_into_on_boa, encode_into_on_vano),
    encoder_replaces_lone_surrogates => (lone_surrogate_on_boa, lone_surrogate_on_vano),
    encoding_interfaces_enforce_brands_and_constructor_calls => (encoding_brands_on_boa, encoding_brands_on_vano),
    borrowed_decoder_methods_and_getters_share_agent_brands => (cross_realm_encoding_on_boa, cross_realm_encoding_on_vano),
    borrowed_encoder_methods_accept_foreign_instances_and_views => (cross_realm_encoder_on_boa, cross_realm_encoder_on_vano),
    retained_iframe_decoder_keeps_stream_state_after_removal => (retained_frame_decoder_on_boa, retained_frame_decoder_on_vano),
    captured_encoding_internals_survive_author_intrinsic_replacement => (captured_encoding_internals_on_boa, captured_encoding_internals_on_vano),
    encoder_and_decoder_private_arrays_do_not_use_author_array_hooks => (private_array_hooks_on_boa, private_array_hooks_on_vano),
    text_encoding_raw_protocol_survives_delimiters_and_intrinsic_poison => (raw_encoding_protocol_on_boa, raw_encoding_protocol_on_vano),
    worker_has_encoding_surfaces => (worker_encoding_on_boa, worker_encoding_on_vano),
    dropped_decoder_finalizer_is_delivered_by_normal_pump => (decoder_finalizer_on_boa, decoder_finalizer_on_vano),
}

#[cfg(target_pointer_width = "64")]
#[test]
fn shared_views_sab_buffers_offsets_and_encode_into_on_vano() {
    shared_views_cover_sab_buffers_offsets_and_encode_into::<script_engine_nova::NovaEngine>();
}
