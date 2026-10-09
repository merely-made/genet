// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Independent FormData/File identity scenarios. RED adapters are explicitly
//! named and are the only functions permitted to change when routing GREEN.
use script_engine_api::ScriptEngine;
use script_runtime_api::{HostState, NoScriptLoader, Runtime};

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.set_base_url("https://forms.test/source").unwrap();
    rt.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    run(&mut rt, FIXTURE_HELPERS);
    rt
}
fn run<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) {
    if let Err(error) = rt.eval(source) {
        panic!(
            "fixture evaluation: {}\n{source}",
            rt.describe_error(&error)
        );
    }
}
fn read<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) -> String {
    let value = rt.eval(source).expect("fixture projection");
    rt.value_to_string(&value).unwrap()
}
fn bytes<E: ScriptEngine>(rt: &mut Runtime<E>, expression: &str) {
    run(
        rt,
        &format!(
            "globalThis.byteResult='pending'; ({expression}).arrayBuffer().then(function(b){{ byteResult=Array.from(new Uint8Array(b)).join(','); }});"
        ),
    );
    rt.run_microtasks();
    assert_eq!(
        read(rt, "byteResult"),
        "0,128,255",
        "independent binary bytes"
    );
}
const FIXTURE_HELPERS: &str = r#"
var fixtureError = Error;
function check(value, message) { if (!value) throw new fixtureError(message); }
function errorName(fn) { try { fn(); return 'none'; } catch (e) { return e.name; } }
function makeForm() {
  var f=document.createElement('form'); f.id='f';
  f.innerHTML='<input id=upload type=file name=upload><input name=a value=1>';
  document.body.appendChild(f); return f;
}
function file() { return new File([new Uint8Array([0,128,255])], 'same.bin',
  {type:'application/octet-stream',lastModified:4294967297}); }
function dataTransfer() {
  check(typeof globalThis.DataTransfer==='function',
    'missing capability: DataTransfer constructor (semantic assertion)');
  return new DataTransfer();
}
function selected(f, files) {
  var dt=dataTransfer(); for(var i=0;i<files.length;i++) dt.items.add(files[i]);
  var input=f.querySelector('input[type=file]'); input.files=dt.files; return input;
}
function pairs(fd) { return JSON.stringify(Array.from(fd.entries())); }
"#;

fn empty_receiver<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"check(pairs(new FormData())==='[]','empty constructor');
      check(errorName(function(){FormData.prototype.get.call({_l:[['x','forged']]},'x');})==='TypeError','genuine receiver required');"#,
    );
}
fn native_entries<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"document.body.innerHTML='<input name=x value=external form=f><form id=f><input id=current name=x value=default><input name=checked type=checkbox checked><input name=off type=checkbox><input name=no disabled><button id=s name=mode value=go>go</button><button name=other value=no>no</button></form>';
      var f=document.getElementById('f'); document.getElementById('current').value='current';
      check(pairs(new FormData(f,document.getElementById('s'))) === '[["x","external"],["x","current"],["checked","on"],["mode","go"]]','native successful entries');"#,
    );
}
fn constructor_validation<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(), other=document.createElement('form'), b=document.createElement('button'); other.appendChild(b); document.body.appendChild(other);
      check(errorName(function(){new FormData({localName:'form'});})==='TypeError','fake form');
      check(errorName(function(){new FormData(f,document.createElement('div'));})==='TypeError','nonbutton submitter');
      check(errorName(function(){new FormData(f,b);})==='NotFoundError','foreign owner');
      var auto=document.createElement('button'); auto.setAttribute('command',''); f.appendChild(auto);
      check(errorName(function(){new FormData(f,auto);})==='TypeError','native Auto rule');
      auto.type='SuBmIt'; auto.name='action'; auto.value='open';
      check(new FormData(f,auto).get('action')==='open','explicit native Submit exception');"#,
    );
}
fn set_order<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var fd=new FormData(); fd.append('a','1'); fd.append('b','2'); fd.append('a','3'); fd.append('c','4'); fd.set('a','9');
      check(pairs(fd)==='[["a","9"],["b","2"],["c","4"]]','replace first duplicate in place');"#,
    );
}
fn live_iterators<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var fd=new FormData(); fd.append('a','1'); fd.append('b','2'); var it=fd.entries();
      check(it.next().value[0]==='a','initial iterator'); fd.append('c','3'); fd.delete('b');
      var next=it.next(); check(!next.done && next.value[0]==='c' && next.value[1]==='3','live remaining iterator'); check(it.next().done,'iterator completed');
      var visits=[], once=false; fd.forEach(function(v,k){visits.push(k+v);if(!once){once=true;fd.append('tail','4');}});
      check(visits.join(',')==='a1,c3,tail4','live forEach');"#,
    );
}
fn blob_entry_matrix<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(), blob=new Blob([new Uint8Array([0,128,255])],{type:'application/octet-stream'}), fd=new FormData();
      fd.append('file',original); fd.append('blob',blob); fd.append('renamed',original,'new.bin'); fd.set('namedBlob',blob,'blob.bin');
      check(fd.get('file')===original,'unrenamed original File identity');
      check(fd.get('blob') instanceof File && fd.get('blob')!==blob && fd.get('blob').name==='blob','Blob becomes File');
      check(fd.get('renamed')!==original && fd.get('renamed').name==='new.bin','filename creates File');
      check(fd.get('namedBlob') instanceof File && fd.get('namedBlob').name==='blob.bin','set Blob wrapper');
      check(original.lastModified===4294967297,'lastModified is not int32');"#,
    );
    for name in ["file", "blob", "renamed", "namedBlob"] {
        bytes(&mut rt, &format!("fd.get('{name}')"));
    }
}
fn usv_coercion<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var fd=new FormData(), calls=0; fd.append('\uD800',{toString:function(){calls++;return '\uD800';}});
      check(fd.get('\uFFFD')==='\uFFFD' && calls===1,'USVString and legitimate coercion');
      check(errorName(function(){fd.append(Symbol('x'),'v');})==='TypeError','Symbol name conversion');
      var saved=String; String=function(){return 'poison';}; try {fd.append('real','value');} finally {String=saved;}
      check(fd.get('real')==='value','intrinsic String conversion');"#,
    );
}
fn filelist_getter<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(), x=f.querySelector('input[type=file]'), text=f.querySelector('input[name=a]');
      check(x.files!==undefined && x.files!==null,'missing capability: selected FileList');
      check(x.files===x.files && x.files.length===0,'unchanged getter identity'); check(text.files===null,'non-file getter');
      var before=x.files; x.files=null; check(x.files===before,'null assignment ignored');
      check(before.item(0)===null && before[0]===undefined,'empty item');
      check(errorName(function(){new FileList();})==='TypeError','illegal constructor');
      check(errorName(function(){x.files={length:1,0:file()};})==='TypeError','genuine list setter');"#,
    );
}
fn datatransfer_live<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var dt=dataTransfer(), list=dt.files, a=file(), b=new File([new Uint8Array([0,128,255])],'second.bin');
      dt.items.add(a); dt.items.add(b); check(dt.files===list && list.length===2 && list[0]===list[0],'live list');
      var first=dt.items[0].getAsFile(), again=dt.items[0].getAsFile(); check(first!==a && first!==again && first.name==='same.bin','getAsFile creates fresh File');
      dt.items.remove(0); check(list.length===1 && list[0].name==='second.bin','live remove'); dt.clearData(); check(list.length===1,'clearData keeps files'); dt.items.clear(); check(list.length===0 && dt.files===list,'live clear');"#,
    );
    bytes(&mut rt, "first");
    bytes(&mut rt, "again");
}
fn assignment_snapshot<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(), dt=dataTransfer(), original=file();dt.items.add(original);var x=f.querySelector('input[type=file]');x.files=dt.files;
      var old=new FormData(f), oldList=x.files; check(old.get('upload')===original && x.files[0]===original && oldList.item(0)===original,'selected original');
      dt.items.clear();check(x.files.length===1,'assignment snapshot independent drag list');
      x.value='';check(x.files.length===0 && old.get('upload')===original,'empty value preserves prior FD');
      selected(f,[original]);f.reset();check(x.files.length===0 && old.get('upload')===original,'reset preserves prior FD');
      selected(f,[original]);x.type='text';x.type='file';check(x.files.length===0 && old.get('upload')===original,'type transition clears selection');"#,
    );
    bytes(&mut rt, "old.get('upload')");
}
fn reset_reassignment<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(), original=file(), x=selected(f,[original]);
      check(errorName(function(){x.value='forbidden';})==='InvalidStateError' && x.files[0]===original,'nonempty value is atomic');
      f.reset(); selected(f,[original]);check(x.files[0]===original && new FormData(f).get('upload')===original,'held File survives reset/reassignment');"#,
    );
    bytes(&mut rt, "original");
}
fn distinct_file_projection<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(), clone=structuredClone(original), fd=new FormData();fd.append('x',original,'wrapped.bin');var wrapped=fd.get('x'), f=makeForm();
      check(original!==clone && original!==wrapped && clone!==wrapped,'three new-object identities');
      var x=selected(f,[original,clone,wrapped,original]);var values=new FormData(f).getAll('upload');
      check(values.length===4 && values[0]===original && values[1]===clone && values[2]===wrapped && values[3]===original,'native projection cannot conflate copies or duplicates');
      check(x.files[0]===original && x.files[1]===clone && x.files[2]===wrapped,'selected projection identity');"#,
    );
    bytes(&mut rt, "clone");
    bytes(&mut rt, "wrapped");
}
fn child<E: ScriptEngine>(rt: &mut Runtime<E>) {
    let realm = rt.create_child_realm(HostState::default()).unwrap();
    let value = rt.engine_mut().realm_global(realm).unwrap();
    let top = rt.top_realm();
    rt.engine_mut()
        .set_global_in_realm(top, "child", &value)
        .unwrap();
}
fn cross_realm_brands<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    child(&mut rt);
    run(
        &mut rt,
        r#"var original=new child.File([new child.Uint8Array([0,128,255])],'same.bin',{type:'application/octet-stream'}), fd=new child.FormData();
      FormData.prototype.append.call(fd,'x',original);check(FormData.prototype.get.call(fd,'x')===original,'borrowed genuine FD/File');
      var saved=child.File;child.File=function(){throw new Error('poison');};try {FormData.prototype.append.call(fd,'y',original);}finally{child.File=saved;}
      check(fd.get('y')===original,'constructor is not File authority');
      check(errorName(function(){FormData.prototype.append.call(Object.create(child.FormData.prototype),'x','y');})==='TypeError','prototype cannot forge FD');"#,
    );
    bytes(&mut rt, "fd.get('x')");
}
fn held_file_gc<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(), x=selected(f,[file()]), list=x.files;globalThis.held=list[0]; f=null;x=null;list=null;"#,
    );
    rt.collect_garbage();
    run(
        &mut rt,
        r#"var f=document.getElementById('f'), x=f.querySelector('input[type=file]');check(x.files[0]===held && new FormData(f).get('upload')===held,'held File only, reacquired live input');f=null;x=null;"#,
    );
    bytes(&mut rt, "held");
}
fn prior_fd_file_gc<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),x=selected(f,[file()]),fd=new FormData(f);globalThis.held=fd.get('upload');f=null;x=null;fd=null;"#,
    );
    rt.collect_garbage();
    run(
        &mut rt,
        r#"var f=document.getElementById('f');check(f.querySelector('input[type=file]').files[0]===held && new FormData(f).get('upload')===held,'prior FD File only retains selected identity');f=null;"#,
    );
    bytes(&mut rt, "held");
}
fn adoption_teardown_gc<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"globalThis.frame=document.createElement('iframe');frame.srcdoc='<html><body><form id=f><input name=upload type=file></form></body></html>';document.body.appendChild(frame);"#,
    );
    rt.run_event_loop(100).unwrap();
    run(
        &mut rt,
        r#"var cw=frame.contentWindow;check(typeof cw.DataTransfer==='function','missing capability: child DataTransfer');var dt=new cw.DataTransfer(), original=new cw.File([new cw.Uint8Array([0,128,255])],'same.bin');dt.items.add(original);
      var f=cw.document.getElementById('f'),x=f.querySelector('input');x.files=dt.files;globalThis.held=new cw.FormData(f).get('upload');check(held===original,'creator File');
      document.body.appendChild(document.adoptNode(f));frame.remove();frame=null;cw=null;dt=null;original=null;x=null;f=null;"#,
    );
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    run(
        &mut rt,
        r#"var f=document.getElementById('f'),x=f.querySelector('input');check(x.files[0]===held && new FormData(f).get('upload')===held,'adopted File after creator retirement');
      var seen=null;f.addEventListener('formdata',function(e){seen=e.formData.get('upload');});new FormData(f);check(seen===held,'formdata File after teardown');seen=null;x=null;f=null;"#,
    );
    bytes(&mut rt, "held");
}
fn shared_edits<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),saved;f.addEventListener('formdata',function(e){saved=e.formData;e.formData.delete('upload');e.formData.set('a','edited');e.formData.append('tail','3');});var fd=new FormData(f);
      check(pairs(fd)==='[["a","edited"],["tail","3"]]','final mutable event list');saved.set('a','late');check(fd.get('a')==='edited','completed constructor snapshots final list');"#,
    );
}
fn recursive_cleanup<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var f=makeForm(),nested=[],events=0;function listener(){events++;nested.push(errorName(function(){new FormData(f);}));throw new Error('listener error');}
      f.addEventListener('formdata',listener);var a=new FormData(f);f.removeEventListener('formdata',listener);var b=new FormData(f);
      check(events===1 && nested.join(',')==='InvalidStateError','recursive construction guard');check(a.get('a')==='1' && b.get('a')==='1','throw released guard');"#,
    );
}
fn fake_tags_shadowing<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(),fd=new FormData();fd.append('x',original);original._b=new Uint8Array([9]);original.name='forged';original.type='bad/type';original.id='1';fd._l=[['x','forged']];
      check(fd.get('x')===original && original.name==='same.bin' && original.type==='application/octet-stream','public fields do not rewrite authority');
      var fake=Object.create(Blob.prototype);fake._b=new Uint8Array([9]);fake.size=1;fd.append('fake',fake);
      check(!(fd.get('fake') instanceof File),'prototype fake is not Blob overload');"#,
    );
    bytes(&mut rt, "fd.get('x')");
}
fn captured_intrinsics<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(),fd=new FormData(),saved={get:WeakMap.prototype.get,set:WeakMap.prototype.set,push:Array.prototype.push,parse:JSON.parse,String:String,U8:Uint8Array},caught=null;
      function poison(){throw new fixtureError('private intrinsic interception');}
      try {WeakMap.prototype.get=poison;WeakMap.prototype.set=poison;Array.prototype.push=poison;JSON.parse=poison;String=poison;Uint8Array=poison;fd.append('x',original);fd.set('a','1');}
      catch(e){caught=e;}finally{WeakMap.prototype.get=saved.get;WeakMap.prototype.set=saved.set;Array.prototype.push=saved.push;JSON.parse=saved.parse;String=saved.String;Uint8Array=saved.U8;}
      check(caught===null,'private operations use captured intrinsics');check(fd.get('x')===original && fd.get('a')==='1','captured mutation values');"#,
    );
    bytes(&mut rt, "original");
}
fn private_getter_extraction<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(),fd=new FormData(),f=makeForm(),dt=dataTransfer(),calls=0;dt.items.add(original);function poison(){calls++;throw new fixtureError('public metadata getter used as authority');}Object.defineProperties(original,{name:{configurable:true,get:poison},type:{configurable:true,get:poison},size:{configurable:true,get:poison},lastModified:{configurable:true,get:poison},_b:{configurable:true,get:poison}});Object.defineProperty(fd,'_l',{configurable:true,get:poison});var caught=null;try{fd.append('upload',original);f.querySelector('input[type=file]').files=dt.files;globalThis.projected=new FormData(f).get('upload');globalThis.copy=structuredClone(original);}catch(e){caught=e;}finally{delete original.name;delete original.type;delete original.size;delete original.lastModified;delete original._b;delete fd._l;}check(caught===null && calls===0,'private extraction ignores public getters');check(projected===original && fd.get('upload')===original && copy.name==='same.bin' && copy.lastModified===4294967297,'private getter snapshot identities/metadata');"#,
    );
    bytes(&mut rt, "copy");
    bytes(&mut rt, "projected");
}
fn private_tojson_species<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(),fd=new FormData(),f=makeForm(),x=f.querySelector('input[type=file]'),dt=dataTransfer();dt.items.add(original);var ab=ArrayBuffer,typed=Object.getPrototypeOf(Uint8Array),saved={o:Object.getOwnPropertyDescriptor(Object.prototype,'toJSON'),a:Object.getOwnPropertyDescriptor(Array.prototype,'toJSON'),ab:Object.getOwnPropertyDescriptor(ab,Symbol.species),ty:Object.getOwnPropertyDescriptor(typed,Symbol.species)},calls=0,caught=null;
      function poison(){calls++;throw new fixtureError('private toJSON/species interception');}
      try {Object.defineProperty(Object.prototype,'toJSON',{configurable:true,value:poison});Object.defineProperty(Array.prototype,'toJSON',{configurable:true,value:poison});Object.defineProperty(ab,Symbol.species,{configurable:true,get:poison});Object.defineProperty(typed,Symbol.species,{configurable:true,get:poison});fd.append('x',original);x.files=dt.files;check(new FormData(f).get('upload')===original,'private native wire identity');globalThis.copy=structuredClone(original);globalThis.sliced=original.slice(0,3);globalThis.outward=original.arrayBuffer();}
      catch(e){caught=e;}finally{function restore(o,k,d){if(d)Object.defineProperty(o,k,d);else delete o[k];}restore(Object.prototype,'toJSON',saved.o);restore(Array.prototype,'toJSON',saved.a);restore(ab,Symbol.species,saved.ab);restore(typed,Symbol.species,saved.ty);}
      check(caught===null && calls===0,'manual private wire/direct copies bypass inherited hooks');"#,
    );
    bytes(&mut rt, "copy");
    bytes(&mut rt, "sliced");
    run(
        &mut rt,
        "globalThis.outwardBytes='pending';outward.then(function(b){outwardBytes=Array.from(new Uint8Array(b)).join(',');});",
    );
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "outwardBytes"), "0,128,255");
}
fn clone_graph<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    run(
        &mut rt,
        r#"var original=file(),copy=structuredClone([original,original]);check(copy[0]!==original && copy[0]===copy[1],'File graph identity');check(copy[0].name==='same.bin' && copy[0].type==='application/octet-stream' && copy[0].lastModified===4294967297,'clone private metadata');
      var f=makeForm(),x=selected(f,[original]),list=structuredClone(x.files);check(list!==x.files && list.length===1 && list[0]!==original,'FileList serialization');"#,
    );
    bytes(&mut rt, "copy[0]");
    bytes(&mut rt, "list[0]");
}
fn persistent_cross_realm<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    child(&mut rt);
    run(
        &mut rt,
        r#"var original=file();original._b=new Uint8Array([9]);Object.defineProperty(original,'name',{value:'shadow'});Object.defineProperty(original,'type',{value:'shadow/type'});Object.defineProperty(original,'lastModified',{value:9});
      globalThis.copy=child.__scDeserializeRecord(__scSerializeRecord([original,original]));check(copy[0]!==original && copy[0]===copy[1],'cross-realm graph');check(copy[0].name==='same.bin' && copy[0].type==='application/octet-stream' && copy[0].lastModified===4294967297,'persistent private metadata');"#,
    );
    bytes(&mut rt, "copy[0]");
}

macro_rules! both {
    ($($name:ident => $case:ident),* $(,)?) => {$(
        mod $name {
            #[test] fn boa(){super::$case::<script_engine_boa::BoaEngine>();}
            #[cfg(target_pointer_width="64")]
            #[test] fn nova(){super::$case::<script_engine_nova::NovaEngine>();}
        }
    )*};
}
both! {
    phase_c_form_data_empty_receiver => empty_receiver,
    phase_c_form_data_native_entries => native_entries,
    phase_c_form_data_constructor_validation => constructor_validation,
    phase_c_form_data_set_order => set_order,
    phase_c_form_data_live_iterators => live_iterators,
    phase_c_form_data_blob_entry_matrix => blob_entry_matrix,
    phase_c_form_data_usv_coercion => usv_coercion,
    phase_c_form_data_filelist_getter => filelist_getter,
    phase_c_form_data_datatransfer_live => datatransfer_live,
    phase_c_form_data_assignment_snapshot => assignment_snapshot,
    phase_c_form_data_reset_reassignment => reset_reassignment,
    phase_c_form_data_distinct_file_projection => distinct_file_projection,
    phase_c_form_data_cross_realm_brands => cross_realm_brands,
    phase_c_form_data_held_file_gc => held_file_gc,
    phase_c_form_data_prior_fd_file_gc => prior_fd_file_gc,
    phase_c_form_data_adoption_teardown_gc => adoption_teardown_gc,
    phase_c_form_data_shared_edits => shared_edits,
    phase_c_form_data_recursive_cleanup => recursive_cleanup,
    phase_c_form_data_fake_tags_shadowing => fake_tags_shadowing,
    phase_c_form_data_captured_intrinsics => captured_intrinsics,
    phase_c_form_data_private_getter_extraction => private_getter_extraction,
    phase_c_form_data_private_tojson_species => private_tojson_species,
    phase_c_form_data_clone_graph => clone_graph,
    phase_c_form_data_persistent_cross_realm => persistent_cross_realm,
}

// Only Nova implements ScriptEngineSnapshot. This is a separate honest engine
// capability fixture, not mislabeled as a both-engine snapshot qualification.
#[cfg(all(target_pointer_width = "64", not(target_arch = "wasm32")))]
#[test]
fn phase_c_form_data_snapshot_private_hooks_nova() {
    let mut template = runtime::<script_engine_nova::NovaEngine>();
    let mut rt = template.snapshot_clone().expect("Nova snapshot");
    drop(template);
    rt.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    run(
        &mut rt,
        r#"var f=makeForm(),original=file(),x=selected(f,[original]);check(new FormData(f).get('upload')===original,'snapshot restored private form operations');check(typeof __formsDOM==='undefined' && typeof __formsPrivate==='undefined' && typeof __domRegisterHooks==='undefined','transient authority absent');"#,
    );
    bytes(&mut rt, "new FormData(f).get('upload')");
}
