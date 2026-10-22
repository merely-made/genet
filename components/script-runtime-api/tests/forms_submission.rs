// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Semantic submission/default-action fixtures. Test-local missing-capability
//! adapters avoid compiler/ReferenceError-only RED and never supply expectations.
use script_engine_api::ScriptEngineLive;
use script_runtime_api::{
    DocumentEncoding, EncodingConfidence, FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader,
    Runtime,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
struct Requests(Rc<RefCell<Vec<FetchRequest>>>);
impl FetchHandler for Requests {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        let url = request.url.clone();
        self.0.borrow_mut().push(request);
        FetchOutcome {
            network_error: false,
            status: 204,
            status_text: "No Content".into(),
            response_type: "basic".into(),
            url,
            redirected: false,
            headers: vec![],
            body: vec![],
        }
    }
}
fn run<E: ScriptEngineLive>(rt: &mut Runtime<E>, source: &str) {
    if let Err(error) = rt.eval(source) {
        panic!("{}\n{source}", rt.describe_error(&error));
    }
}
fn read<E: ScriptEngineLive>(rt: &mut Runtime<E>, source: &str) -> String {
    let v = rt.eval(source).expect("projection");
    rt.value_to_string(&v).unwrap()
}
fn runtime<E: ScriptEngineLive>() -> (Runtime<E>, Requests) {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.set_base_url("https://forms.test/source").unwrap();
    let requests = Requests::default();
    rt.set_fetch_handler(Box::new(requests.clone()));
    rt.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    run(&mut rt, HELPERS);
    (rt, requests)
}
const HELPERS: &str = r#"
var fixtureError=Error, missingCapabilities=[];
function check(value,message){if(!value)throw new fixtureError(message);}
function errorName(fn){try{fn();return 'none';}catch(e){return e.name;}}
function makeForm(){var f=document.createElement('form');f.id='f';f.action='/send?old=1#frag';f.innerHTML='<input name=q value="a b"><button id=s name=mode value=go>go</button>';document.body.appendChild(f);return f;}
// RED-only missing-method adapter. Does not emulate submission or validation.
function fixtureRequestSubmit(f,s){if(typeof f.requestSubmit==='function')return arguments.length>1?f.requestSubmit(s):f.requestSubmit();missingCapabilities.push('requestSubmit');}
// RED-only missing-method adapter. Does not synthesize an alternative algorithm.
function fixtureSubmit(f){if(typeof f.submit==='function')return f.submit();missingCapabilities.push('submit');}
function fixtureClick(x){if(typeof x.click==='function')return x.click();missingCapabilities.push('HTMLElement.click');}
function observe(f){globalThis.events=[];globalThis.lastSubmit=null;globalThis.lastData=null;f.addEventListener('submit',function(e){events.push('submit');lastSubmit=e;});f.addEventListener('formdata',function(e){events.push('formdata');lastData=e;});}
"#;
fn request<E: ScriptEngineLive>(rt: &mut Runtime<E>, rec: &Requests) -> FetchRequest {
    rt.run_event_loop(100).unwrap();
    let entries = rec.0.borrow();
    assert_eq!(
        entries.len(),
        1,
        "one actual request, missing-capability adapters cannot fabricate it"
    );
    let request = &entries[0];
    FetchRequest {
        method: request.method.clone(),
        url: request.url.clone(),
        headers: request.headers.clone(),
        body: request.body.clone(),
        cache: request.cache.clone(),
        redirect: request.redirect.clone(),
        mode: request.mode.clone(),
        referrer: request.referrer.clone(),
        referrer_policy: request.referrer_policy.clone(),
        credentials: request.credentials.clone(),
        integrity: request.integrity.clone(),
        upgrade_insecure_requests: request.upgrade_insecure_requests,
        block_all_mixed_content: request.block_all_mixed_content,
    }
}
fn header<'a>(request: &'a FetchRequest, name: &str) -> Option<&'a str> {
    request
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
// RED-only host adapter routes the existing actual event bridge, which has no
// form activation. GREEN may change only this named adapter to the new host API.
fn host_click_adapter<E: ScriptEngineLive>(
    rt: &mut Runtime<E>,
    id: u64,
    _position: Option<(u32, u32)>,
) -> bool {
    rt.dispatch_form_activation(id, _position).expect("private host form activation")
}
// RED-only host adapter: current string-only bridge does not carry key/modifiers.
fn host_key_adapter<E: ScriptEngineLive>(
    rt: &mut Runtime<E>,
    id: u64,
    _key: &str,
    _modifiers: [bool; 4],
) -> bool {
    rt.dispatch_form_keydown(id, _key, script_runtime_api::FormKeyModifiers { alt: _modifiers[0], ctrl: _modifiers[1], meta: _modifiers[2], shift: _modifiers[3] }).expect("private host rich key")
}
fn node_id<E: ScriptEngineLive>(rt: &mut Runtime<E>, id: &str) -> u64 {
    let v = rt
        .eval(&format!("document.getElementById('{id}').__ref"))
        .unwrap();
    rt.engine_mut()
        .reflector_data(&v)
        .expect("genuine native reflector")
}

fn event_constructors<E: ScriptEngineLive>() {
    let (mut rt, _) = runtime::<E>();
    run(
        &mut rt,
        r#"check(typeof SubmitEvent==='function' && typeof FormDataEvent==='function','missing capability: submission event constructors');
      check(errorName(function(){new SubmitEvent();})==='TypeError','SubmitEvent required type');
      var s=new SubmitEvent('submit');check(s.submitter===null && !s.isTrusted && !s.bubbles && !s.cancelable,'SubmitEvent defaults');
      var f=makeForm(),b=f.querySelector('button'),s2=new SubmitEvent('submit',{submitter:b,bubbles:true,cancelable:true});check(s2.submitter===b && s2 instanceof Event && s2.bubbles && s2.cancelable,'SubmitEvent init');
      s2.submitter=null;check(s2.submitter===b && Object.getPrototypeOf(s2)===SubmitEvent.prototype,'readonly submitter/prototype');
      check(errorName(function(){new SubmitEvent('x',{submitter:{}});})==='TypeError','submitter conversion');
      check(errorName(function(){new FormDataEvent('formdata');})==='TypeError','required FormDataEvent dictionary');
      check(errorName(function(){new FormDataEvent('x',{});})==='TypeError','required formData member');
      var fd=new FormData(),d=new FormDataEvent('formdata',{formData:fd,bubbles:true});check(d.formData===fd && d.bubbles && !d.cancelable && !d.isTrusted,'FormDataEvent init');d.formData=null;check(d.formData===fd && Object.getPrototypeOf(d)===FormDataEvent.prototype && d instanceof Event,'readonly formData/prototype');"#,
    );
}
fn submit_bypass<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.querySelector('input').required=true;f.querySelector('input').value='';observe(f);var invalid=0;f.addEventListener('invalid',function(){invalid++;},true);fixtureSubmit(f);check(invalid===0 && events.join(',')==='formdata','submit bypasses validation/submit event');"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(r.url, "https://forms.test/send?q=#frag");
}
fn request_validation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.querySelector('input').required=true;f.querySelector('input').value='';observe(f);var invalid=0;f.addEventListener('invalid',function(e){invalid++;e.preventDefault();},true);fixtureRequestSubmit(f);check(invalid===1 && events.length===0,'interactive invalid still prevents request');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn request_submitter<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);fixtureRequestSubmit(f,b);check(events.join(',')==='submit,formdata' && lastSubmit.submitter===b,'chosen submitter events');check(lastSubmit.isTrusted && lastSubmit.bubbles && lastSubmit.cancelable && lastData.isTrusted && lastData.bubbles && !lastData.cancelable,'UA event flags');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=a+b&mode=go#frag"
    );
}
fn request_null_submitter<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);fixtureRequestSubmit(f,null);check(lastSubmit!==null && lastSubmit.submitter===null,'null submitter');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=a+b#frag"
    );
}
fn request_receiver_errors<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),method=f.requestSubmit;check(typeof method==='function','missing capability: native requestSubmit receiver/errors');check(errorName(function(){method.call({});})==='TypeError','genuine form receiver');check(errorName(function(){method.call(f,{});})==='TypeError','submitter conversion');var div=document.createElement('div');check(errorName(function(){method.call(f,div);})==='TypeError','non-submit element');var other=document.createElement('form'),b=document.createElement('button');other.appendChild(b);document.body.appendChild(other);check(errorName(function(){method.call(f,b);})==='NotFoundError','genuine foreign form owner');var auto=document.createElement('button');auto.setAttribute('command','');f.appendChild(auto);check(errorName(function(){method.call(f,auto);})==='TypeError','shared native Auto predicate');observe(f);auto.type='SuBmIt';auto.name='mode';auto.value='explicit';method.call(f,auto);check(lastSubmit!==null && lastSubmit.submitter===auto,'explicit Submit exception positive control');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=a+b&mode=explicit#frag"
    );
}
fn submitter_target_override<E: ScriptEngineLive>() {
    struct Reply(Requests);
    impl FetchHandler for Reply {
        fn fetch(&self, r: FetchRequest) -> FetchOutcome {
            let url = r.url.clone();
            self.0.0.borrow_mut().push(r);
            FetchOutcome {
                network_error: false,
                status: 200,
                status_text: "OK".into(),
                response_type: "basic".into(),
                url,
                redirected: false,
                headers: vec![("content-type".into(), "text/html".into())],
                body: b"<html><body><p id=loaded>target reply</p></body></html>".to_vec(),
            }
        }
    }
    let (mut rt, rec) = runtime::<E>();
    rt.set_fetch_handler(Box::new(Reply(rec.clone())));
    run(
        &mut rt,
        "globalThis.frame=document.createElement('iframe');frame.name='chosen';frame.srcdoc='<html><body>child';document.body.appendChild(frame);",
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        "var f=makeForm();f.target='missing';var b=f.querySelector('button');b.setAttribute('formtarget','chosen');b.setAttribute('formaction','/chosen');fixtureRequestSubmit(f,b);",
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/chosen?q=a+b&mode=go"
    );
    assert_eq!(
        read(
            &mut rt,
            "document.getElementById('f')!==null && frame.contentDocument.getElementById('loaded').textContent==='target reply'"
        ),
        "true",
        "submitter target uses existing child context"
    );
}
// enctypes-helper.js: each case removes its named iframe in cleanup and the
// next case appends a fresh one under the same name and submits within the
// same task, before the removed context's teardown task runs.
fn reused_target_name<E: ScriptEngineLive>(settle_teardown: bool) {
    struct Reply(Requests);
    impl FetchHandler for Reply {
        fn fetch(&self, r: FetchRequest) -> FetchOutcome {
            let url = r.url.clone();
            self.0.0.borrow_mut().push(r);
            FetchOutcome {
                network_error: false,
                status: 200,
                status_text: "OK".into(),
                response_type: "basic".into(),
                url,
                redirected: false,
                headers: vec![("content-type".into(), "text/html".into())],
                body: b"<html><body><p id=loaded>target reply</p></body></html>".to_vec(),
            }
        }
    }
    let (mut rt, rec) = runtime::<E>();
    rt.set_fetch_handler(Box::new(Reply(rec.clone())));
    run(
        &mut rt,
        "var old=document.createElement('iframe');old.name='reused';old.srcdoc='<body>old';document.body.appendChild(old);",
    );
    rt.run_event_loop(100).unwrap();
    run(&mut rt, "document.body.removeChild(old);");
    if settle_teardown {
        rt.run_event_loop(100).unwrap();
    }
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        "globalThis.frame=document.createElement('iframe');frame.name='reused';document.body.appendChild(frame);var f=makeForm();f.target='reused';f.action='/reused';fixtureSubmit(f);",
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/reused?q=a+b"
    );
    assert_eq!(
        read(
            &mut rt,
            "frame.contentDocument.getElementById('loaded').textContent"
        ),
        "target reply",
        "the connected frame of that name is navigated"
    );
}
fn reused_target_name_before_teardown<E: ScriptEngineLive>() {
    reused_target_name::<E>(false);
}
fn reused_target_name_after_teardown<E: ScriptEngineLive>() {
    reused_target_name::<E>(true);
}
fn canceled_submit<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);f.addEventListener('submit',function(e){e.preventDefault();});fixtureRequestSubmit(f);check(events.join(',')==='submit','cancel stops formdata');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn trust_tampering_redispatch<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);var originals={Event:Event,SubmitEvent:globalThis.SubmitEvent,FormDataEvent:globalThis.FormDataEvent};function poison(){throw new fixtureError('public event constructor invoked');}
      Event=poison;SubmitEvent=poison;FormDataEvent=poison;f.dispatchEvent=poison;try{fixtureRequestSubmit(f,b);}finally{Event=originals.Event;SubmitEvent=originals.SubmitEvent;FormDataEvent=originals.FormDataEvent;delete f.dispatchEvent;}
      check(lastSubmit!==null && lastSubmit.isTrusted && lastSubmit.submitter===b,'captured trusted event factory');var old=lastSubmit;Node.prototype.dispatchEvent.call(f,old);check(!old.isTrusted,'scripted redispatch clears trust');"#,
    );
    request(&mut rt, &rec);
}
fn mutable_list_reentrancy<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);f.addEventListener('submit',function(){fixtureRequestSubmit(f);});f.addEventListener('formdata',function(e){fixtureSubmit(f);fixtureRequestSubmit(f);e.formData.set('q','edited');e.formData.append('tail','3');});fixtureRequestSubmit(f);
      check(events.join(',')==='submit,formdata','independent reentrancy flags');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=edited&tail=3#frag"
    );
}
fn remove_during_submit<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);f.addEventListener('submit',function(){f.remove();});fixtureRequestSubmit(f);check(events.join(',')==='submit','connectedness recheck');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn cancellation_cleanup<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);function cancel(e){e.preventDefault();throw new fixtureError('listener error');}f.addEventListener('submit',cancel);fixtureRequestSubmit(f);f.removeEventListener('submit',cancel);fixtureRequestSubmit(f);check(events.join(',')==='submit,submit,formdata','flag cleanup after cancellation/throw');"#,
    );
    request(&mut rt, &rec);
}
fn submitter_overrides<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');b.setAttribute('formaction','/override?old=1#frag');b.setAttribute('formmethod','post');b.setAttribute('formenctype','text/plain');b.setAttribute('formnovalidate','');f.querySelector('input').required=true;f.querySelector('input').value='';
      var shadow=document.createElement('input');shadow.name='submit';shadow.value='shadow';f.appendChild(shadow);fixtureRequestSubmit(f,b);"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.url, "https://forms.test/override?old=1#frag");
    assert_eq!(r.method, "POST");
    assert_eq!(header(&r, "content-type"), Some("text/plain"));
    assert_eq!(
        r.body.as_deref(),
        Some(&b"q=\r\nmode=go\r\nsubmit=shadow\r\n"[..])
    );
}
fn get_query_fragment<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, "var f=makeForm();fixtureSubmit(f);");
    let r = request(&mut rt, &rec);
    assert_eq!(r.url, "https://forms.test/send?q=a+b#frag");
    assert_eq!(r.method, "GET");
    assert!(r.body.is_none());
}
fn empty_get_query<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, "var f=makeForm();f.innerHTML='';fixtureSubmit(f);");
    assert_eq!(request(&mut rt, &rec).url, "https://forms.test/send?#frag");
}
fn post_plain_binary_fragment<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.action='https://forms.test/source#next';f.method='post';f.enctype='text/plain';f.innerHTML='<textarea name=q></textarea>';f.querySelector('textarea').value='a\nb';fixtureSubmit(f);"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.url, "https://forms.test/source#next");
    assert_eq!(r.method, "POST");
    assert_eq!(r.body.as_deref(), Some(&b"q=a\r\nb\r\n"[..]));
    assert_eq!(header(&r, "content-type"), Some("text/plain"));
}
fn post_url_newlines<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.method='post';f.innerHTML='<textarea name=q></textarea>';f.querySelector('textarea').value='a\nb';fixtureSubmit(f);"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "POST");
    assert_eq!(r.body.as_deref(), Some(&b"q=a%0D%0Ab"[..]));
    assert_eq!(
        header(&r, "content-type"),
        Some("application/x-www-form-urlencoded")
    );
}
fn post_multipart_binary<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.method='post';f.enctype='multipart/form-data';f.innerHTML='<input type=file name=upload><textarea name=q></textarea>';f.querySelector('textarea').value='a\nb';
      check(typeof DataTransfer==='function','missing capability: multipart selected File');var dt=new DataTransfer(),original=new File([new Uint8Array([0,128,255])],'f\r\"\n\\.bin',{type:'application/octet-stream'});dt.items.add(original);f.querySelector('input').files=dt.files;
      f.addEventListener('formdata',function(e){check(e.formData.get('upload')===original,'multipart event File identity');e.formData.append('tail','3');});fixtureSubmit(f);"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "POST");
    assert_eq!(r.url, "https://forms.test/send?old=1#frag");
    let boundary = header(&r, "content-type")
        .expect("multipart header")
        .strip_prefix("multipart/form-data; boundary=")
        .expect("multipart boundary");
    let mut expected=format!("--{boundary}\r\nContent-Disposition: form-data; name=\"upload\"; filename=\"f%0D%22%0A\\.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n").into_bytes();
    expected.extend_from_slice(&[0, 128, 255]);
    expected.extend_from_slice(format!("\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"q\"\r\n\r\na\r\nb\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"tail\"\r\n\r\n3\r\n--{boundary}--\r\n").as_bytes());
    assert_eq!(
        r.body.as_deref(),
        Some(expected.as_slice()),
        "literal multipart body with only observed boundary substituted"
    );
}
fn owner_encoding<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    rt.set_document_encoding(DocumentEncoding {
        encoding: genet_scripted_dom::form_encoding::Encoding::for_label(b"windows-1252").unwrap(),
        confidence: EncodingConfidence::Certain,
    });
    run(
        &mut rt,
        r#"var f=makeForm();f.innerHTML='<input name=q value=é><input name=_charset_ type=hidden value=default>';fixtureSubmit(f);"#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(
        r.url,
        "https://forms.test/send?q=%E9&_charset_=windows-1252#frag"
    );
    run(
        &mut rt,
        "check(new FormData(f).get('_charset_')==='UTF-8','constructor encoding independent of owner');f.setAttribute('accept-charset','invalid');fixtureSubmit(f);",
    );
    rt.run_event_loop(100).unwrap();
    assert_eq!(
        rec.0.borrow()[1].url,
        "https://forms.test/send?q=%C3%A9&_charset_=UTF-8#frag"
    );
}
fn detached_adopted_encoding<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    rt.set_document_encoding(DocumentEncoding {
        encoding: genet_scripted_dom::form_encoding::Encoding::for_label(b"shift_jis").unwrap(),
        confidence: EncodingConfidence::Certain,
    });
    run(
        &mut rt,
        r#"var detached=document.implementation.createHTMLDocument(),f=detached.createElement('form');f.action='/send';f.innerHTML='<input name=q value=あ><input name=_charset_ type=hidden>';detached.body.appendChild(f);
      check(new FormData(f).get('_charset_')==='UTF-8','detached constructor UTF8');document.body.appendChild(document.adoptNode(f));fixtureSubmit(f);"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=%82%A0&_charset_=Shift_JIS"
    );
}
fn pending_replacement<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var one=document.createElement('iframe'),two=document.createElement('iframe');one.name='one';two.name='two';one.srcdoc='<body>one';two.srcdoc='<body>two';document.body.appendChild(one);document.body.appendChild(two);"#,
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        r#"var f=makeForm(),g=makeForm();g.id='g';f.target='one';f.action='/old';fixtureSubmit(f);g.target='one';g.action='/other';fixtureSubmit(g);f.target='two';f.action='/new';fixtureSubmit(f);"#,
    );
    rt.run_event_loop(100).unwrap();
    let mut urls = rec
        .0
        .borrow()
        .iter()
        .map(|r| r.url.clone())
        .collect::<Vec<_>>();
    urls.sort();
    assert_eq!(
        urls,
        vec![
            "https://forms.test/new?q=a+b",
            "https://forms.test/other?q=a+b"
        ]
    );
}
fn deferred_fragment_replacement<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.innerHTML='';f.action='https://forms.test/source#old';fixtureSubmit(f);check(location.href==='https://forms.test/source','form fragment task deferred');f.action='/new';fixtureSubmit(f);"#,
    );
    assert_eq!(request(&mut rt, &rec).url, "https://forms.test/new?");
}
fn raw_referrer_policy<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.setAttribute('rel','noreferrer');fixtureSubmit(f);",
    );
    let r = request(&mut rt, &rec);
    assert_eq!(
        r.referrer, "https://forms.test/source",
        "raw initiator identity is retained"
    );
    assert_eq!(r.referrer_policy, "no-referrer");
}
fn click_method<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);b.addEventListener('click',function(e){check(!e.isTrusted && e instanceof PointerEvent,'scripted pointer click trust');events.push('click');fixtureClick(b);queueMicrotask(function(){events.push('microtask');});});fixtureClick(b);"#,
    );
    rt.run_microtasks();
    assert_eq!(
        read(&mut rt, "events.join(',')"),
        "click,submit,formdata,microtask"
    );
    request(&mut rt, &rec);
}
fn native_click_order<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);f.querySelector('button').addEventListener('click',function(e){check(e.isTrusted && e instanceof PointerEvent,'native pointer click trust');events.push('click');queueMicrotask(function(){events.push('microtask');});});"#,
    );
    let id = node_id(&mut rt, "s");
    assert!(host_click_adapter(&mut rt, id, None));
    assert_eq!(
        read(&mut rt, "events.join(',')"),
        "click,submit,formdata,microtask"
    );
    request(&mut rt, &rec);
}
fn native_click_cancel<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();observe(f);var clicks=0;f.querySelector('button').addEventListener('click',function(e){clicks++;e.preventDefault();});"#,
    );
    let id = node_id(&mut rt, "s");
    assert!(!host_click_adapter(&mut rt, id, None));
    assert_eq!(read(&mut rt, "clicks+'|'+events.length"), "1|0");
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn scripted_mouse_activation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);var e=new MouseEvent('click',{bubbles:true,cancelable:true});check(b.dispatchEvent(e),'uncanceled return');check(!e.isTrusted && events.join(',')==='submit,formdata','untrusted genuine MouseEvent activates');"#,
    );
    request(&mut rt, &rec);
}
fn scripted_mouse_cancel<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);var clicks=0;b.addEventListener('click',function(e){clicks++;e.preventDefault();});var e=new MouseEvent('click',{bubbles:true,cancelable:true});check(!b.dispatchEvent(e) && e.defaultPrevented && !e.isTrusted,'scripted MouseEvent cancellation/return');check(clicks===1 && events.length===0,'canceled MouseEvent no activation');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn fake_mouse_brand<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);var fake=new Event('click',{bubbles:true,cancelable:true});Object.setPrototypeOf(fake,MouseEvent.prototype);b.dispatchEvent(fake);check(events.length===0,'MouseEvent prototype cannot brand an ordinary Event');var target=new Event('click',{bubbles:true,cancelable:true});try{MouseEvent.call(target,'click',{bubbles:true,cancelable:true});}catch(e){}b.dispatchEvent(target);check(events.length===0,'authored constructor call cannot brand another Event');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn nonmouse_no_activation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');observe(f);b.dispatchEvent(new Event('click',{bubbles:true,cancelable:true}));b.dispatchEvent(new MouseEvent('other',{bubbles:true}));check(events.length===0,'ordinary Event click is not activation');"#,
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn ancestor_borrowed_activation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button'),span=document.createElement('span');b.appendChild(span);observe(f);span.dispatchEvent(new MouseEvent('click',{bubbles:false}));check(events.length===0,'nonbubbling child does not choose ancestor');check(typeof EventTarget.prototype.dispatchEvent==='function','missing capability: borrowed EventTarget node dispatch');EventTarget.prototype.dispatchEvent.call(span,new MouseEvent('click',{bubbles:true,cancelable:true}));check(events.join(',')==='submit,formdata','borrowed EventTarget routes node ancestor activation');"#,
    );
    request(&mut rt, &rec);
}
fn disabled_request_distinction<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),b=f.querySelector('button');b.disabled=true;observe(f);var clicks=0;b.addEventListener('click',function(){clicks++;});fixtureClick(b);check(clicks===0 && events.length===0,'disabled .click is suppressed');fixtureRequestSubmit(f,b);check(clicks===0 && events.join(',')==='submit,formdata' && lastSubmit.submitter===b,'disabled genuine requestSubmit bypasses click');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=a+b#frag"
    );
}
fn disabled_native_click<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm(),b=f.querySelector('button');b.disabled=true;observe(f);var clicks=0;b.addEventListener('click',function(){clicks++;});",
    );
    let id = node_id(&mut rt, "s");
    host_click_adapter(&mut rt, id, None);
    assert_eq!(
        read(&mut rt, "clicks+'|'+events.length"),
        "0|0",
        "disabled physical control suppresses queued click"
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn implicit_one_blocker<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.innerHTML='<input id=t name=q value=one>';",
    );
    let id = node_id(&mut rt, "t");
    host_key_adapter(&mut rt, id, "Enter", [false; 4]);
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=one#frag"
    );
}
fn implicit_two_blockers<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.innerHTML='<input id=t name=q value=one><input name=r value=two>';",
    );
    let id = node_id(&mut rt, "t");
    host_key_adapter(&mut rt, id, "Enter", [false; 4]);
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn implicit_default_button<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.querySelector('input').id='t';observe(f);var clicks=0;f.querySelector('button').addEventListener('click',function(){clicks++;});",
    );
    let id = node_id(&mut rt, "t");
    host_key_adapter(&mut rt, id, "Enter", [false; 4]);
    assert_eq!(
        read(
            &mut rt,
            "clicks+'|'+(lastSubmit!==null && lastSubmit.submitter===document.getElementById('s'))"
        ),
        "1|true",
        "implicit submission clicks native default submit button"
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=a+b&mode=go#frag"
    );
}
fn implicit_disabled_default<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.querySelector('input').id='t';f.querySelector('button').disabled=true;var enabled=document.createElement('button');enabled.name='second';enabled.value='go';f.appendChild(enabled);observe(f);var clicks=0;f.addEventListener('click',function(){clicks++;});",
    );
    let id = node_id(&mut rt, "t");
    host_key_adapter(&mut rt, id, "Enter", [false; 4]);
    assert_eq!(
        read(&mut rt, "clicks+'|'+events.length"),
        "0|0",
        "disabled first default blocks later enabled button"
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn implicit_key_modifiers<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.innerHTML='<input id=t name=q value=one>';observe(f);",
    );
    let id = node_id(&mut rt, "t");
    host_key_adapter(&mut rt, id, "Escape", [false; 4]);
    for modifiers in [
        [true, false, false, false],
        [false, true, false, false],
        [false, false, true, false],
        [false, false, false, true],
    ] {
        host_key_adapter(&mut rt, id, "Enter", modifiers);
    }
    assert_eq!(
        read(&mut rt, "events.length"),
        "0",
        "non-Enter and modified Enter do not submit"
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
    host_key_adapter(&mut rt, id, "Enter", [false; 4]);
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?q=one#frag",
        "unmodified Enter positive control"
    );
}
fn callback_child_activation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm();f.addEventListener('submit',function(e){e.preventDefault();globalThis.frame=document.createElement('iframe');frame.srcdoc='<html><body><form id=childForm action=https://forms.test/child><input name=q value=child><button id=childButton>go</button></form></body></html>';document.body.appendChild(frame);});fixtureRequestSubmit(f);check(typeof globalThis.frame==='object' && globalThis.frame!==null,'missing capability: requestSubmit callback-created child');"#,
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        r#"var cw=frame.contentWindow,cf=cw.document.getElementById('childForm'),cb=cw.document.getElementById('childButton'),ParentMouse=MouseEvent,seen=0;cf.addEventListener('submit',function(e){check(e.isTrusted && e.submitter===cb,'callback-created child private submit event');seen++;});var saved=cw.MouseEvent;cw.MouseEvent=function(){throw new fixtureError('public child constructor');};try{cb.dispatchEvent(new ParentMouse('click',{bubbles:true,cancelable:true}));}finally{cw.MouseEvent=saved;}check(seen===1,'parent genuine MouseEvent activates child created during callback');"#,
    );
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/child?q=child"
    );
}
fn adopted_creator_teardown_activation<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"globalThis.frame=document.createElement('iframe');frame.srcdoc='<html><body><form id=adopted action=https://forms.test/adopted><input type=file name=upload><button id=s>go</button></form></body></html>';document.body.appendChild(frame);"#,
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        r#"var cw=frame.contentWindow;check(typeof cw.DataTransfer==='function','missing capability: creator File selection');var dt=new cw.DataTransfer(),file=new cw.File([new cw.Uint8Array([0,128,255])],'same.bin');dt.items.add(file);var f=cw.document.getElementById('adopted');f.querySelector('input').files=dt.files;globalThis.held=file;document.body.appendChild(document.adoptNode(f));frame.remove();frame=null;cw=null;dt=null;file=null;f=null;"#,
    );
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    run(
        &mut rt,
        r#"var f=document.getElementById('adopted');observe(f);f.addEventListener('formdata',function(e){check(e.formData.get('upload')===held,'activation event File identity after creator retirement');});check(new FormData(f).get('upload')===held,'adopted native selected identity');"#,
    );
    let id = node_id(&mut rt, "s");
    host_click_adapter(&mut rt, id, None);
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/adopted?upload=same.bin"
    );
    run(
        &mut rt,
        "globalThis.byteResult='pending';held.arrayBuffer().then(function(b){byteResult=Array.from(new Uint8Array(b)).join(',');});",
    );
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "byteResult"), "0,128,255");
}
fn image_native_position<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.innerHTML='<input id=s type=image name=img>';",
    );
    let id = node_id(&mut rt, "s");
    host_click_adapter(&mut rt, id, Some((7, 9)));
    assert_eq!(
        request(&mut rt, &rec).url,
        "https://forms.test/send?img.x=7&img.y=9#frag"
    );
}
fn unknown_target_declines<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        "var f=makeForm();f.target='_blank';fixtureSubmit(f);f.target='missing';fixtureSubmit(f);",
    );
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
    assert_eq!(read(&mut rt, "location.href"), "https://forms.test/source");
}
macro_rules! both{($($name:ident=>$case:ident),*$(,)?)=>{$(mod $name{#[test]fn boa(){super::$case::<script_engine_boa::BoaEngine>();}#[cfg(target_pointer_width="64")]#[test]fn nova(){super::$case::<script_engine_nova::NovaEngine>();}})*};}
both! {
    phase_c_forms_submission_event_constructors=>event_constructors,
    phase_c_forms_submission_submit_bypass=>submit_bypass,
    phase_c_forms_submission_request_validation=>request_validation,
    phase_c_forms_submission_request_submitter=>request_submitter,
    phase_c_forms_submission_request_null_submitter=>request_null_submitter,
    phase_c_forms_submission_request_receiver_errors=>request_receiver_errors,
    phase_c_forms_submission_submitter_target_override=>submitter_target_override,
    phase_c_forms_submission_reused_target_name_before_teardown=>reused_target_name_before_teardown,
    phase_c_forms_submission_reused_target_name_after_teardown=>reused_target_name_after_teardown,
    phase_c_forms_submission_canceled_submit=>canceled_submit,
    phase_c_forms_submission_trust_tampering_redispatch=>trust_tampering_redispatch,
    phase_c_forms_submission_mutable_list_reentrancy=>mutable_list_reentrancy,
    phase_c_forms_submission_remove_during_submit=>remove_during_submit,
    phase_c_forms_submission_cancellation_cleanup=>cancellation_cleanup,
    phase_c_forms_submission_submitter_overrides=>submitter_overrides,
    phase_c_forms_submission_get_query_fragment=>get_query_fragment,
    phase_c_forms_submission_empty_get_query=>empty_get_query,
    phase_c_forms_submission_post_plain_fragment=>post_plain_binary_fragment,
    phase_c_forms_submission_post_url_newlines=>post_url_newlines,
    phase_c_forms_submission_post_multipart_binary=>post_multipart_binary,
    phase_c_forms_submission_owner_encoding=>owner_encoding,
    phase_c_forms_submission_detached_adopted_encoding=>detached_adopted_encoding,
    phase_c_forms_submission_pending_replacement=>pending_replacement,
    phase_c_forms_submission_deferred_fragment_replacement=>deferred_fragment_replacement,
    phase_c_forms_submission_raw_referrer_policy=>raw_referrer_policy,
    phase_c_forms_submission_click_method=>click_method,
    phase_c_forms_submission_native_click_order=>native_click_order,
    phase_c_forms_submission_native_click_cancel=>native_click_cancel,
    phase_c_forms_submission_scripted_mouse_activation=>scripted_mouse_activation,
    phase_c_forms_submission_scripted_mouse_cancel=>scripted_mouse_cancel,
    phase_c_forms_submission_fake_mouse_brand=>fake_mouse_brand,
    phase_c_forms_submission_nonmouse_no_activation=>nonmouse_no_activation,
    phase_c_forms_submission_ancestor_borrowed_activation=>ancestor_borrowed_activation,
    phase_c_forms_submission_disabled_request_distinction=>disabled_request_distinction,
    phase_c_forms_submission_disabled_native_click=>disabled_native_click,
    phase_c_forms_submission_implicit_one_blocker=>implicit_one_blocker,
    phase_c_forms_submission_implicit_two_blockers=>implicit_two_blockers,
    phase_c_forms_submission_implicit_default_button=>implicit_default_button,
    phase_c_forms_submission_implicit_disabled_default=>implicit_disabled_default,
    phase_c_forms_submission_implicit_key_modifiers=>implicit_key_modifiers,
    phase_c_forms_submission_callback_child_activation=>callback_child_activation,
    phase_c_forms_submission_adopted_creator_teardown_activation=>adopted_creator_teardown_activation,
    phase_c_forms_submission_image_native_position=>image_native_position,
    phase_c_forms_submission_unknown_target_declines=>unknown_target_declines,
}

// Separately authorized review regressions. Original Phase C scenarios above
// stay frozen; these expectations are literal and independent of native encoders.
fn fix1_submit_listener_bypass<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm();f.action='/fix1-nested';observe(f);
      var savedSubmit=HTMLFormElement.prototype.submit,savedRequest=HTMLFormElement.prototype.requestSubmit;
      var submitCalls=0,dataCalls=0,nestedValue=null;
      f.addEventListener('formdata',function(e){dataCalls++;nestedValue=e.formData.get('q');});
      f.addEventListener('submit',function(e){e.preventDefault();submitCalls++;if(submitCalls>1)return;
        f.querySelector('input').value='nested';savedSubmit.call(f);
        f.querySelector('input').value='late';savedRequest.call(f);
      });
      savedRequest.call(f);
      check(events.join(',')==='submit,formdata','nested bypass event sequence');
      check(submitCalls===1 && dataCalls===1 && nestedValue==='nested','bypass preserves outer firing guard');
    "#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(r.url, "https://forms.test/fix1-nested?q=nested");
    assert!(r.body.is_none());
}
fn fix1_native_entry_own_fields<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm();f.innerHTML='<input name=q value=plain><input name=upload type=file>';
      var selectedFile=new File([new Uint8Array([0,128,255])],'held.bin',{type:'application/octet-stream'});
      var dt=new DataTransfer();dt.items.add(selectedFile);f.querySelector('input[type=file]').files=dt.files;
      var poisonReads=0,caught=null,result=null,projectionDataEvents=[];
      f.addEventListener('formdata',function(e){projectionDataEvents.push(e);});
      var oldDescriptor=Object.getOwnPropertyDescriptor(Object.prototype,'file');
      Object.defineProperty(Object.prototype,'file',{configurable:true,get:function(){poisonReads++;throw new fixtureError('inherited file getter reached');}});
      try{result=new FormData(f);check(result.get('q')==='plain','text entry under prototype poisoning');
        check(result.get('upload')===selectedFile,'selected identity under prototype poisoning');
        check(result.get('upload').name==='held.bin' && result.get('upload').type==='application/octet-stream','selected metadata under prototype poisoning');
      }catch(e){caught=e;}finally{if(oldDescriptor)Object.defineProperty(Object.prototype,'file',oldDescriptor);else delete Object.prototype.file;}
      check(poisonReads===0,'native entry projection reads zero inherited file getters');
      check(caught===null && result.get('q')==='plain' && result.get('upload')===selectedFile,'native own-data projection succeeds');
      check(projectionDataEvents.length===1 && projectionDataEvents[0].isTrusted,'one trusted constructor formdata');
      globalThis.byteResult='pending';result.get('upload').arrayBuffer().then(function(b){byteResult=Array.from(new Uint8Array(b)).join(',');});
    "#,
    );
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "byteResult"), "0,128,255");
    rt.run_event_loop(100).unwrap();
    assert_eq!(rec.0.borrow().len(), 0);
}
fn fix1_borrowed_creator_events<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var frame=document.createElement('iframe');frame.srcdoc='<html><body><form id=child action=https://forms.test/fix1-borrowed><input name=q value=child><button id=childButton name=mode value=child-go>go</button></form></body></html>';document.body.appendChild(frame);
    "#,
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        r#"
      var cw=frame.contentWindow,cf=cw.document.getElementById('child'),cb=cw.document.getElementById('childButton');
      var ParentData=FormData,ParentSubmit=SubmitEvent,ParentDataEvent=FormDataEvent,parentRequest=HTMLFormElement.prototype.requestSubmit;
      var ChildSubmit=cw.SubmitEvent,ChildDataEvent=cw.FormDataEvent,creatorEvents=[];
      cf.addEventListener('submit',function(e){creatorEvents.push(e);});cf.addEventListener('formdata',function(e){creatorEvents.push(e);});
      function eventPoison(){throw new fixtureError('public event constructor invoked');}
      SubmitEvent=eventPoison;FormDataEvent=eventPoison;cw.SubmitEvent=eventPoison;cw.FormDataEvent=eventPoison;
      try{var parentConstructed=new ParentData(cf);parentRequest.call(cf,cb);}
      finally{SubmitEvent=ParentSubmit;FormDataEvent=ParentDataEvent;cw.SubmitEvent=ChildSubmit;cw.FormDataEvent=ChildDataEvent;}
      check(creatorEvents.map(function(e){return e.type;}).join(',')==='formdata,submit,formdata','borrowed creator event sequence');
      check(parentConstructed.get('q')==='child','borrowed constructor entries');
      check(Object.getPrototypeOf(creatorEvents[0])===ChildDataEvent.prototype && creatorEvents[0] instanceof ChildDataEvent && Object.getPrototypeOf(creatorEvents[0])!==ParentDataEvent.prototype,'borrowed constructor creator FormDataEvent prototype');
      check(Object.getPrototypeOf(creatorEvents[1])===ChildSubmit.prototype && creatorEvents[1] instanceof ChildSubmit && Object.getPrototypeOf(creatorEvents[1])!==ParentSubmit.prototype,'borrowed request creator SubmitEvent prototype');
      check(Object.getPrototypeOf(creatorEvents[2])===ChildDataEvent.prototype && creatorEvents[2] instanceof ChildDataEvent && Object.getPrototypeOf(creatorEvents[2])!==ParentDataEvent.prototype,'borrowed request creator FormDataEvent prototype');
      check(creatorEvents[0].isTrusted && creatorEvents[1].isTrusted && creatorEvents[2].isTrusted && creatorEvents[1].submitter===cb,'borrowed creator event trust/member');
      check(creatorEvents[2].formData.get('q')==='child' && creatorEvents[2].formData.get('mode')==='child-go','borrowed request event entries');
    "#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(
        r.url,
        "https://forms.test/fix1-borrowed?q=child&mode=child-go"
    );
}
fn fix1_retired_creator_events<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var frame=document.createElement('iframe');frame.srcdoc='<html><body><form id=retired action=https://forms.test/fix1-retired><input name=upload type=file><button id=fix1RetiredButton>go</button></form></body></html>';document.body.appendChild(frame);
    "#,
    );
    rt.run_event_loop(100).unwrap();
    rec.0.borrow_mut().clear();
    run(
        &mut rt,
        r#"
      var cw=frame.contentWindow,CreatorSubmit=cw.SubmitEvent,CreatorData=cw.FormDataEvent;
      var dt=new cw.DataTransfer(),heldFile=new cw.File([new cw.Uint8Array([0,128,255])],'held.bin');dt.items.add(heldFile);
      var adopted=cw.document.getElementById('retired'),input=adopted.querySelector('input'),fileList=dt.files;input.files=fileList;
      document.body.appendChild(document.adoptNode(adopted));frame.remove();frame=null;cw=null;dt=null;input=null;fileList=null;adopted=null;
    "#,
    );
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    run(
        &mut rt,
        r#"
      var retiredForm=document.getElementById('retired'),retiredEvents=[];
      retiredForm.addEventListener('submit',function(e){retiredEvents.push(e);});retiredForm.addEventListener('formdata',function(e){retiredEvents.push(e);});
    "#,
    );
    let id = node_id(&mut rt, "fix1RetiredButton");
    assert!(rt.dispatch_form_activation(id, None).unwrap());
    run(
        &mut rt,
        r#"
      check(retiredEvents.map(function(e){return e.type;}).join(',')==='submit,formdata','retired creator activation sequence');
      check(Object.getPrototypeOf(retiredEvents[0])===CreatorSubmit.prototype && retiredEvents[0] instanceof CreatorSubmit && retiredEvents[0].isTrusted,'retired creator SubmitEvent prototype/trust');
      check(Object.getPrototypeOf(retiredEvents[1])===CreatorData.prototype && retiredEvents[1] instanceof CreatorData && retiredEvents[1].isTrusted,'retired creator FormDataEvent prototype/trust');
      check(retiredEvents[1].formData.get('upload')===heldFile,'retired creator event selected identity');
      check(new FormData(retiredForm).get('upload')===heldFile,'retired creator later FormData identity');
      check(retiredEvents.map(function(e){return e.type;}).join(',')==='submit,formdata,formdata','retired creator later constructor sequence');
      check(Object.getPrototypeOf(retiredEvents[2])===CreatorData.prototype && retiredEvents[2] instanceof CreatorData && retiredEvents[2].isTrusted,'retired creator later FormDataEvent prototype/trust');
      globalThis.byteResult='pending';heldFile.arrayBuffer().then(function(b){byteResult=Array.from(new Uint8Array(b)).join(',');});
    "#,
    );
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "byteResult"), "0,128,255");
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(r.url, "https://forms.test/fix1-retired?upload=held.bin");
}
fn fix1_encoding_retained_after_formdata<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm();f.action='/fix1-encoding';f.method='post';f.enctype='application/x-www-form-urlencoded';f.acceptCharset='windows-1252';f.innerHTML='<input name=q value=é><input name=_charset_ type=hidden>';observe(f);
      var entryCharset=null,entryQ=null;
      f.addEventListener('formdata',function(e){entryCharset=e.formData.get('_charset_');entryQ=e.formData.get('q');f.acceptCharset='UTF-8';e.formData.append('tail','é');});
      f.requestSubmit();
      check(events.join(',')==='submit,formdata','retained encoding event sequence');
      check(entryCharset==='windows-1252' && entryQ==='é','encoding selected before entry construction');
      check(f.acceptCharset==='UTF-8','listener changed public acceptCharset');
    "#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "POST");
    assert_eq!(r.url, "https://forms.test/fix1-encoding");
    assert_eq!(
        header(&r, "content-type"),
        Some("application/x-www-form-urlencoded")
    );
    assert_eq!(
        r.body.as_deref(),
        Some(&b"q=%E9&_charset_=windows-1252&tail=%E9"[..])
    );
}
fn fix1_same_query_reload<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    rt.set_base_url("https://forms.test/fix1-reload?q=a+b")
        .unwrap();
    run(
        &mut rt,
        r#"
      check(location.href==='https://forms.test/fix1-reload?q=a+b','same-query actual initiating URL');
      var f=makeForm();f.action='https://forms.test/fix1-reload?q=a+b';observe(f);f.requestSubmit();
      check(events.join(',')==='submit,formdata','same-query submission events');
    "#,
    );
    assert_eq!(rec.0.borrow().len(), 0, "reload remains queued");
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(r.url, "https://forms.test/fix1-reload?q=a+b");
    assert!(r.body.is_none());
}
fn fix1_real_fragment_deferred<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    rt.set_base_url("https://forms.test/fix1-fragment?q=a+b")
        .unwrap();
    run(
        &mut rt,
        r#"
      var retainedMarker=document.createElement('div');retainedMarker.id='fix1Marker';document.body.appendChild(retainedMarker);
      var f=makeForm();f.action='https://forms.test/fix1-fragment?q=a+b#next';observe(f);var hashes=0;
      window.addEventListener('hashchange',function(){hashes++;events.push('hashchange');});f.requestSubmit();
      check(events.join(',')==='submit,formdata' && location.href==='https://forms.test/fix1-fragment?q=a+b','real fragment action deferred');
    "#,
    );
    assert_eq!(rec.0.borrow().len(), 0);
    rt.run_event_loop(100).unwrap();
    assert_eq!(
        rec.0.borrow().len(),
        0,
        "fragment navigation does not fetch"
    );
    run(
        &mut rt,
        r#"
      check(location.href==='https://forms.test/fix1-fragment?q=a+b#next','real fragment location applied');
      check(document.getElementById('fix1Marker')===retainedMarker,'real fragment preserves document marker');
      check(hashes===1 && events.join(',')==='submit,formdata,hashchange','one deferred real fragment hashchange');
    "#,
    );
}
fn fix1_event_composed<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm(),b=f.querySelector('button'),fd=new FormData(),SavedSubmit=SubmitEvent,SavedData=FormDataEvent;
      var s=new SavedSubmit('x',{submitter:b,bubbles:true,cancelable:true,composed:true});
      var d=new SavedData('x',{formData:fd,bubbles:true,cancelable:true,composed:true});
      check(s.bubbles && s.cancelable && s.composed && !s.isTrusted && s.submitter===b,'SubmitEvent inherited EventInit');
      check(d.bubbles && d.cancelable && d.composed && !d.isTrusted && d.formData===fd,'FormDataEvent inherited EventInit');
      check(Object.getPrototypeOf(s)===SavedSubmit.prototype && Object.getPrototypeOf(d)===SavedData.prototype,'public base event prototypes');
      check(!new SavedSubmit('x').composed && !new SavedData('x',{formData:fd}).composed,'event composed default false');
    "#,
    );
    rt.run_event_loop(100).unwrap();
    assert_eq!(rec.0.borrow().len(), 0);
}
fn fix1_event_dictionary_once<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm(),b=f.querySelector('button'),fd=new FormData(),sCounts=[0,0,0,0],dCounts=[0,0,0,0];
      var s=null,d=null,sError=null,dError=null;
      var si={get bubbles(){sCounts[0]++;return true;},get cancelable(){sCounts[1]++;return true;},get composed(){sCounts[2]++;return true;},get submitter(){sCounts[3]++;return sCounts[3]===1?b:{};}};
      var di={get bubbles(){dCounts[0]++;return true;},get cancelable(){dCounts[1]++;return true;},get composed(){dCounts[2]++;return true;},get formData(){dCounts[3]++;return dCounts[3]===1?fd:{};}};
      try{s=new SubmitEvent('x',si);}catch(e){sError=e;}try{d=new FormDataEvent('x',di);}catch(e){dError=e;}
      check(sError===null && dError===null,'genuine once-converted event dictionary members accepted');
      check(sCounts.join(',')==='1,1,1,1' && dCounts.join(',')==='1,1,1,1','each event dictionary getter read exactly once');
      check(s.submitter===b && d.formData===fd,'event stores first genuine dictionary member');
      check(s.bubbles && s.cancelable && s.composed && d.bubbles && d.cancelable && d.composed && !s.isTrusted && !d.isTrusted,'once-converted event flags');
    "#,
    );
    rt.run_event_loop(100).unwrap();
    assert_eq!(rec.0.borrow().len(), 0);
}
fn fix1_event_derived_prototypes<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm(),b=f.querySelector('button'),fd=new FormData(),SavedSubmit=SubmitEvent,SavedData=FormDataEvent,SavedEvent=Event;
      class DerivedSubmit extends SavedSubmit{};class DerivedData extends SavedData{};
      var s=new DerivedSubmit('x',{submitter:b,composed:true}),d=new DerivedData('x',{formData:fd,composed:true});
      check(Object.getPrototypeOf(s)===DerivedSubmit.prototype && s instanceof DerivedSubmit && s instanceof SavedSubmit && s instanceof SavedEvent,'derived SubmitEvent prototype');
      check(Object.getPrototypeOf(d)===DerivedData.prototype && d instanceof DerivedData && d instanceof SavedData && d instanceof SavedEvent,'derived FormDataEvent prototype');
      check(s.submitter===b && d.formData===fd && s.composed && d.composed && !s.isTrusted && !d.isTrusted,'derived event member/flags');
    "#,
    );
    rt.run_event_loop(100).unwrap();
    assert_eq!(rec.0.borrow().len(), 0);
}
both! {
    phase_c_task4_fix1_submit_listener_bypass=>fix1_submit_listener_bypass,
    phase_c_task4_fix1_native_entry_own_fields=>fix1_native_entry_own_fields,
    phase_c_task4_fix1_borrowed_creator_events=>fix1_borrowed_creator_events,
    phase_c_task4_fix1_retired_creator_events=>fix1_retired_creator_events,
    phase_c_task4_fix1_encoding_retained_after_formdata=>fix1_encoding_retained_after_formdata,
    phase_c_task4_fix1_same_query_reload=>fix1_same_query_reload,
    phase_c_task4_fix1_real_fragment_deferred=>fix1_real_fragment_deferred,
    phase_c_task4_fix1_event_composed=>fix1_event_composed,
    phase_c_task4_fix1_event_dictionary_once=>fix1_event_dictionary_once,
    phase_c_task4_fix1_event_derived_prototypes=>fix1_event_derived_prototypes,
}

fn fix1_transient_native_hooks_absent<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(
        &mut rt,
        r#"
      var f=makeForm();f.action='/fix1-hooks';observe(f);
      var fd=new FormData(f);check(fd.get('q')==='a b','genuine constructor survives private sink cleanup');
      f.requestSubmit();
      check(events.join(',')==='formdata,submit,formdata' && lastSubmit.isTrusted && lastData.isTrusted,'genuine submission survives private sink cleanup');
      check(lastData.formData.get('q')==='a b','genuine submission entry extraction survives cleanup');
      check(typeof __formFileAllocate==='undefined','transient __formFileAllocate absent');
      check(typeof __formFilesReplace==='undefined','transient __formFilesReplace absent');
      check(typeof __formFilesSnapshot==='undefined','transient __formFilesSnapshot absent');
      check(typeof __formEntrySnapshot==='undefined','transient __formEntrySnapshot absent');
      check(typeof __formDataEncode==='undefined','transient __formDataEncode absent');
      check(typeof __formControlInfo==='undefined','transient __formControlInfo absent');
      check(typeof __formSubmitterCheck==='undefined','transient __formSubmitterCheck absent');
      check(typeof __formImplicitDecision==='undefined','transient __formImplicitDecision absent');
      check(typeof __formMeta==='undefined','transient __formMeta absent');
      check(typeof __formSubmissionUserValidity==='undefined','transient submission validity hook absent');
      check(typeof __formSubmissionPlan==='undefined','transient __formSubmissionPlan absent');
      check(typeof __domRegisterHooks==='undefined','transient __domRegisterHooks absent');
    "#,
    );
    let r = request(&mut rt, &rec);
    assert_eq!(r.method, "GET");
    assert_eq!(r.url, "https://forms.test/fix1-hooks?q=a+b");
    assert!(r.body.is_none());
}
both! {
    phase_c_task4_fix1_transient_native_hooks_absent=>fix1_transient_native_hooks_absent,
}

// Final Phase C review regressions: submission validity and listener mutations.
fn review_submission_user_validity<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, r#"
      var f=makeForm(),q=f.querySelector('input'),outside=document.createElement('input');
      q.required=true;q.value='';outside.required=true;document.body.appendChild(outside);
      check(!q.matches(':user-invalid') && !outside.matches(':user-invalid'),'untouched fields start neutral');
      var invalid=0;f.addEventListener('invalid',function(e){invalid++;e.preventDefault();check(q.matches(':user-invalid'),'user-invalid is set before invalid event');},true);
      f.requestSubmit();
      check(invalid===1 && q.matches(':user-invalid'),'attempted submission marks invalid control');
      check(!outside.matches(':user-invalid'),'unassociated control remains untouched');
      f.reset();check(!q.matches(':user-invalid'),'reset clears submission user validity');
    "#);
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn review_novalidate_user_validity<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, r#"
      var f=makeForm(),q=f.querySelector('input');q.required=true;q.value='';f.noValidate=true;
      var submitted=0;f.addEventListener('submit',function(e){submitted++;e.preventDefault();check(q.matches(submitted===1?':user-invalid':':user-valid'),'submission marks user validity before submit');});
      f.requestSubmit();check(submitted===1 && q.matches(':user-invalid'),'novalidate submission transition');
      f.reset();f.noValidate=false;q.value='valid';
      f.requestSubmit();check(q.matches(':user-valid'),'valid untouched control becomes user-valid');
    "#);
    rt.run_event_loop(100).unwrap();
    assert!(rec.0.borrow().is_empty());
}
fn review_submit_bypass_user_validity<E: ScriptEngineLive>() {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, r#"
      var f=makeForm(),q=f.querySelector('input');q.required=true;q.value='';
      f.checkValidity();f.reportValidity();new FormData(f);
      check(!q.matches(':user-invalid'),'validation APIs and entry construction do not mark user validity');
      f.submit();check(!q.matches(':user-invalid'),'submit() keeps the scripted bypass');
    "#);
    assert_eq!(request(&mut rt,&rec).url,"https://forms.test/send?q=#frag");
}
fn review_mutated_submitter<E: ScriptEngineLive>(owner: bool, late: bool) {
    let (mut rt, rec) = runtime::<E>();
    run(&mut rt, if owner {"var moveOwner=true;"} else {"var moveOwner=false;"});
    run(&mut rt, if late {"var mutateLate=true;"} else {"var mutateLate=false;"});
    run(&mut rt, r#"
      var f=makeForm(),b=f.querySelector('button'),other=document.createElement('form');
      other.action='/other-owner';other.method='post';other.enctype='text/plain';
      other.innerHTML='<input name=outside value=excluded>';document.body.appendChild(other);observe(f);
      if(!moveOwner){b.setAttribute('formaction','/wrong-override');b.setAttribute('formmethod','post');b.setAttribute('formenctype','text/plain');}
      f.addEventListener(mutateLate?'formdata':'submit',function(){if(moveOwner)other.appendChild(b);else b.type='button';});
      f.requestSubmit(b);
      check(events.join(',')==='submit,formdata','established submission survives submitter mutation');
      check(lastData.formData.get('q')==='a b' && !lastData.formData.has('outside'),'entry collection stays on original form');
      check(lastData.formData.get('mode')===(!moveOwner || mutateLate?'go':null),'submitter entry follows collection timing');
    "#);
    let r=request(&mut rt,&rec);
    if owner {
        assert_eq!(r.method,"POST");
        assert_eq!(r.url,"https://forms.test/other-owner");
        assert_eq!(header(&r,"Content-Type"),Some("text/plain"));
        assert_eq!(r.body.as_deref(),Some(if late {b"q=a b\r\nmode=go\r\n".as_slice()} else {b"q=a b\r\n".as_slice()}));
    } else {
        assert_eq!(r.method,"GET");
        assert_eq!(r.url,"https://forms.test/send?q=a+b&mode=go#frag");
        assert!(r.body.is_none());
    }
}
fn review_submitter_type_during_submit<E: ScriptEngineLive>(){review_mutated_submitter::<E>(false,false);}
fn review_submitter_type_during_formdata<E: ScriptEngineLive>(){review_mutated_submitter::<E>(false,true);}
fn review_submitter_owner_during_submit<E: ScriptEngineLive>(){review_mutated_submitter::<E>(true,false);}
fn review_submitter_owner_during_formdata<E: ScriptEngineLive>(){review_mutated_submitter::<E>(true,true);}
both! {
    phase_c_review_submission_user_validity=>review_submission_user_validity,
    phase_c_review_novalidate_user_validity=>review_novalidate_user_validity,
    phase_c_review_submit_bypass_user_validity=>review_submit_bypass_user_validity,
    phase_c_review_submitter_type_during_submit=>review_submitter_type_during_submit,
    phase_c_review_submitter_type_during_formdata=>review_submitter_type_during_formdata,
    phase_c_review_submitter_owner_during_submit=>review_submitter_owner_during_submit,
    phase_c_review_submitter_owner_during_formdata=>review_submitter_owner_during_formdata,
}
