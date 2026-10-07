// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Fetch scheme dispatch must not consume a body the selected scheme ignores.
use std::cell::RefCell;
use std::rc::Rc;

use script_engine_api::ScriptEngine;
use script_runtime_api::{FetchHandler, FetchOutcome, FetchRequest, Runtime};

struct DataFetch {
    requests: Rc<RefCell<Vec<(String, Option<Vec<u8>>)>>>,
}

impl FetchHandler for DataFetch {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        self.requests
            .borrow_mut()
            .push((request.url.clone(), request.body));
        FetchOutcome {
            network_error: false,
            status: 200,
            status_text: "OK".into(),
            response_type: "basic".into(),
            url: request.url,
            redirected: false,
            headers: vec![("content-type".into(), "text/plain;charset=utf-8".into())],
            body: b"test".to_vec(),
        }
    }
}

fn control<E: ScriptEngine>(body: &str) -> Vec<(String, Option<Vec<u8>>)> {
    let requests = Rc::new(RefCell::new(Vec::new()));
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.set_fetch_handler(Box::new(DataFetch {
        requests: requests.clone(),
    }));
    runtime
        .eval(&format!(
            r#"var fetchResult='pending';
            function check(value,message){{if(!value)throw new Error(message);}}
            (async function(){{{body}}})().then(
                ()=>fetchResult='ok',e=>fetchResult='error:'+e.name+':'+e.message);
            "#
        ))
        .expect("control evaluates");
    runtime.run_microtasks();
    let result = runtime.eval("fetchResult").expect("control result");
    assert_eq!(runtime.value_to_string(&result).expect("stringify"), "ok");
    let recorded = requests.borrow().clone();
    recorded
}

const NEVER_CLOSING_DATA_UPLOADS: &str = r#"
    let directPulls=0,directCancels=0;
    const directBody=new ReadableStream({
        pull(){directPulls++;},cancel(){directCancels++;}
    },{highWaterMark:0});
    const directResponse=await fetch('data:a/a;charset=utf-8,test',{
        method:'POST',body:directBody,duplex:'half'
    });
    check(await directResponse.text()==='test','data URL response resolves without waiting for upload EOF');
    check(!directBody.locked && directPulls===0 && directCancels===0,
        'direct data URL fetch leaves the never-closing source untouched');

    let requestPulls=0,requestCancels=0;
    const requestBody=new ReadableStream({
        pull(){requestPulls++;},cancel(){requestCancels++;}
    },{highWaterMark:0});
    const request=new Request('data:a/a;charset=utf-8,test',{
        method:'POST',body:requestBody,duplex:'half'
    });
    const requestStream=request.body;
    check(requestStream===requestBody && !request.bodyUsed && !requestStream.locked,
        'new Request retains its init body without consuming it');
    const requestFetch=fetch(request);
    check(request.bodyUsed && requestStream.locked,
        'fetch(Request) creates its required proxy and disturbs/locks the input body');
    check(requestPulls<=1,
        'proxy setup may start one pending read but does not repeatedly drain the never-closing source');
    check(requestCancels===0,
        'Request proxy setup and data dispatch do not cancel the source');
    const requestResponse=await requestFetch;
    check(await requestResponse.text()==='test','Request-object data URL response resolves');
    check(request.bodyUsed && requestStream.locked,
        'Request input remains disturbed and locked after fetch dispatch');
    check(requestPulls<=1 && requestCancels===0,
        'data dispatch resolves without draining or canceling the open Request body proxy');

    await fetch('data:a/a;charset=utf-8,test',{method:'POST'});
    await fetch('data:a/a;charset=utf-8,test',{method:'POST',body:''});
"#;

#[test]
fn data_url_fetch_skips_never_closing_stream_upload_but_keeps_body_presence() {
    let requests = control::<script_engine_boa::BoaEngine>(NEVER_CLOSING_DATA_UPLOADS);
    assert_eq!(requests.len(), 4);
    for (url, _) in &requests {
        assert!(url.starts_with("data:"));
    }
    assert_eq!(requests[0].1, Some(Vec::new()));
    assert_eq!(requests[1].1, Some(Vec::new()));
    assert_eq!(requests[2].1, None);
    assert_eq!(requests[3].1, Some(Vec::new()));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn vano_data_url_fetch_skips_never_closing_stream_upload_but_keeps_body_presence() {
    let requests = control::<script_engine_nova::NovaEngine>(NEVER_CLOSING_DATA_UPLOADS);
    assert_eq!(requests.len(), 4);
    for (url, _) in &requests {
        assert!(url.starts_with("data:"));
    }
    assert_eq!(requests[0].1, Some(Vec::new()));
    assert_eq!(requests[1].1, Some(Vec::new()));
    assert_eq!(requests[2].1, None);
    assert_eq!(requests[3].1, Some(Vec::new()));
}
