// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Stream serialization rejects by private brand across clone and message paths.
use script_engine_api::ScriptEngine;
use script_runtime_api::{HostState, Runtime};

fn control<E: ScriptEngine>(body: &str, child: bool) {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    if child {
        let realm = runtime
            .create_child_realm(HostState::default())
            .expect("child realm");
        let global = runtime
            .engine_mut()
            .realm_global(realm)
            .expect("child global");
        let top = runtime.top_realm();
        runtime
            .engine_mut()
            .set_global_in_realm(top, "child", &global)
            .expect("share child global");
    }
    let script = format!(
        r#"(function(){{
        function check(value,message){{if(!value)throw new Error(message);}}
        function rejects(action){{
            let error;
            try{{action();}}catch(reason){{error=reason;}}
            check(error && error.name==='DataCloneError','stream must reject serialization');
        }}
        {body}
        return true;
        }})()"#
    );
    let result = runtime.eval(&script).expect("clone control");
    assert_eq!(runtime.value_to_string(&result).expect("result"), "true");
}

const PRIVATE_STREAM_BRANDS: &str = r#"
    const channel=new MessageChannel();
    for(const stream of [new ReadableStream(),new WritableStream(),new TransformStream()]){
        let calls=0;
        Object.defineProperty(stream,Symbol.toStringTag,{get(){calls++;throw new Error('tag');}});
        Object.defineProperty(stream,'constructor',{get(){calls++;throw new Error('constructor');}});
        Object.defineProperty(stream,'value',{enumerable:true,get(){calls++;throw new Error('value');}});
        Object.setPrototypeOf(stream,Object.prototype);
        rejects(()=>structuredClone(stream));
        rejects(()=>__scSerializeRecord({nested:stream}));
        rejects(()=>__scSerialize(stream));
        rejects(()=>channel.port1.postMessage(stream));
        check(calls===0,'serialization rejects stream before author getters');
    }
    check(!('__finishStreamsClone' in globalThis),'one-shot clone capture is removed');
    channel.port1.close();channel.port2.close();
"#;

const ORDINARY_LOOKALIKES: &str = r#"
    for(const ctor of [ReadableStream,WritableStream,TransformStream]){
        const ordinary=Object.create(ctor.prototype);ordinary.value=42;
        const copy=structuredClone(ordinary);
        const recordCopy=__scDeserializeRecord(__scSerializeRecord(ordinary));
        check(copy.value===42 && recordCopy.value===42,'stream prototype alone is not a stream brand');
        check(Object.getPrototypeOf(copy)===Object.prototype,'ordinary clone keeps ordinary prototype');
    }
"#;

const CROSS_REALM_BRANDS: &str = r#"
    for(const pair of [[ReadableStream,child.ReadableStream],
                      [WritableStream,child.WritableStream],
                      [TransformStream,child.TransformStream]]){
        const local=new pair[0](),foreign=new pair[1]();
        rejects(()=>structuredClone(foreign));
        rejects(()=>__scSerializeRecord(foreign));
        rejects(()=>child.structuredClone(local));
        rejects(()=>child.__scSerializeRecord(local));
    }
"#;

macro_rules! cases {
    ($backend:ty) => {
        #[test]
        fn serialization_rejects_private_stream_brands_before_author_getters() {
            control::<$backend>(PRIVATE_STREAM_BRANDS, false);
        }
        #[test]
        fn ordinary_objects_with_stream_prototypes_remain_cloneable() {
            control::<$backend>(ORDINARY_LOOKALIKES, false);
        }
        #[test]
        fn serialization_rejects_streams_from_sibling_realms() {
            control::<$backend>(CROSS_REALM_BRANDS, true);
        }
    };
}

mod boa {
    use super::*;
    cases!(script_engine_boa::BoaEngine);
}

#[cfg(target_pointer_width = "64")]
mod vano {
    use super::*;
    cases!(script_engine_nova::NovaEngine);
}
