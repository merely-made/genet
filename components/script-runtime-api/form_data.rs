// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Genuine VM extraction followed by native immutable file snapshots.
use crate::{
    OwnerResolvedCx as _,
    dom::{adoption, reflect_pinned},
};
use genet_scripted_dom::{FormEntry, FormEntryValue, FormFile, FormFileBindingIdentity};
use script_engine_api::{CallCx, NativeFn, ScriptEngine};
use std::sync::Arc;

fn private_realm<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> script_engine_api::RealmId {
    let realm = cx.current_realm();
    let agent = adoption::current_host(cx).and_then(|host| host.borrow().agent.upgrade());
    let Some(agent) = agent else {
        return realm;
    };
    let agent = agent.borrow();
    // Retained interface methods can execute in a retired creation realm.
    // The same-agent brands survive there; use a live retained extractor.
    if agent.dom_adoption.hook(realm).is_some() {
        realm
    } else {
        agent.frames.top_realm()
    }
}

pub(crate) fn private_call<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    kind: &str,
    value: E::Value,
) -> Result<E::Value, E::Error> {
    let kind = cx.make_string(kind)?;
    let realm = private_realm::<E>(cx);
    adoption::call_hook::<E>(cx, realm, "formPrivate", vec![kind, value])
}
pub(crate) fn type_error<E: ScriptEngine>(cx: &mut E::CallCx<'_>, message: &str) -> E::Error {
    let message = match cx.make_string(message) {
        Ok(value) => value,
        Err(error) => return error,
    };
    match private_call::<E>(cx, "reject", message) {
        Err(error) => error,
        Ok(_) => cx.error("Private form TypeError route is unavailable"),
    }
}
fn extract<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    kind: &str,
    value: E::Value,
) -> Result<String, E::Error> {
    let wire = private_call::<E>(cx, kind, value)?;
    cx.value_to_string(&wire)
}

// This cursor accepts only the private writer's ordered File/list/entry schemas.
// It is not an author-facing JSON parser. Every record must consume its full wire.
struct FormWire<'a> {
    rest: &'a str,
}
impl<'a> FormWire<'a> {
    fn new(wire: &'a str) -> Self {
        Self { rest: wire }
    }
    fn take(&mut self, literal: &str) -> Option<()> {
        self.rest = self.rest.strip_prefix(literal)?;
        Some(())
    }
    fn hex(&mut self) -> Option<u16> {
        let digits = self.rest.get(..4)?;
        if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let value = u16::from_str_radix(digits, 16).ok()?;
        self.rest = self.rest.get(4..)?;
        Some(value)
    }
    fn string(&mut self) -> Option<String> {
        self.take("\"")?;
        let mut out = String::new();
        loop {
            let c = self.rest.chars().next()?;
            self.rest = &self.rest[c.len_utf8()..];
            match c {
                '"' => return Some(out),
                '\\' => {
                    let escape = self.rest.as_bytes().first().copied()?;
                    self.rest = self.rest.get(1..)?;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.hex()?;
                            let scalar = if (0xd800..=0xdbff).contains(&first) {
                                self.take("\\u")?;
                                let second = self.hex()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return None;
                                }
                                0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(second)
                                    - 0xdc00
                            } else {
                                u32::from(first)
                            };
                            out.push(char::from_u32(scalar)?);
                        },
                        _ => return None,
                    }
                },
                c if c < '\u{20}' => return None,
                c => out.push(c),
            }
        }
    }
    fn bytes(&mut self) -> Option<Vec<u8>> {
        self.take("[")?;
        let mut bytes = Vec::new();
        if self.rest.starts_with(']') {
            self.take("]")?;
            return Some(bytes);
        }
        loop {
            let count = self.rest.bytes().take_while(u8::is_ascii_digit).count();
            let digits = self.rest.get(..count)?;
            if digits.is_empty() || (digits.len() > 1 && digits.starts_with('0')) {
                return None;
            }
            bytes.push(digits.parse::<u8>().ok()?);
            self.rest = self.rest.get(count..)?;
            if self.rest.starts_with(']') {
                self.take("]")?;
                return Some(bytes);
            }
            self.take(",")?;
        }
    }
    fn file(&mut self) -> Option<FormFile> {
        self.take("{\"name\":")?;
        let name = self.string()?;
        self.take(",\"type\":")?;
        let mime = self.string()?;
        self.take(",\"bytes\":")?;
        let bytes = Arc::from(self.bytes()?);
        self.take(",\"token\":")?;
        let identity = if self.rest.starts_with("null") {
            self.take("null")?;
            None
        } else {
            Some(FormFileBindingIdentity::decode(&self.string()?).ok()?)
        };
        self.take("}")?;
        match identity {
            None => FormFile::new(name, mime, bytes).ok(),
            Some(identity) => Some(FormFile::from_binding_identity(identity, name, mime, bytes)),
        }
    }
    fn files(&mut self) -> Option<Vec<FormFile>> {
        self.take("[")?;
        let mut files = Vec::new();
        if !self.rest.starts_with(']') {
            loop {
                files.push(self.file()?);
                if self.rest.starts_with(']') {
                    break;
                }
                self.take(",")?;
            }
        }
        self.take("]")?;
        self.rest.is_empty().then_some(files)
    }
    fn entries(&mut self) -> Option<Vec<FormEntry>> {
        self.take("[")?;
        let mut entries = Vec::new();
        if !self.rest.starts_with(']') {
            loop {
                self.take("{\"name\":")?;
                let name = self.string()?;
                let value = if self.rest.starts_with(",\"text\":") {
                    self.take(",\"text\":")?;
                    FormEntryValue::Text(self.string()?)
                } else {
                    self.take(",\"file\":")?;
                    FormEntryValue::File {
                        file: self.file()?,
                        source_input: None,
                    }
                };
                self.take("}")?;
                entries.push(FormEntry { name, value });
                if self.rest.starts_with(']') {
                    break;
                }
                self.take(",")?;
            }
        }
        self.take("]")?;
        self.rest.is_empty().then_some(entries)
    }
}
fn file_from_wire(wire: &str) -> Option<FormFile> {
    let mut cursor = FormWire::new(wire);
    let file = cursor.file()?;
    cursor.rest.is_empty().then_some(file)
}
pub(crate) fn entries<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    value: E::Value,
) -> Result<Vec<FormEntry>, E::Error> {
    let wire = extract::<E>(cx, "fd", value)?;
    FormWire::new(&wire)
        .entries()
        .ok_or_else(|| type_error::<E>(cx, "Invalid File identity"))
}
fn bytes_json(bytes: &[u8]) -> String {
    let mut wire = String::from("[");
    for (i, byte) in bytes.iter().enumerate() {
        if i != 0 {
            wire.push(',');
        }
        wire.push_str(&byte.to_string());
    }
    wire.push(']');
    wire
}
fn file_json(file: &FormFile) -> String {
    format!(
        "{{\"name\":{},\"type\":{},\"bytes\":{},\"token\":{}}}",
        crate::js_str(&file.name),
        crate::js_str(&file.mime_type),
        bytes_json(&file.bytes),
        crate::js_str(&file.binding_identity().encode())
    )
}
pub(crate) struct FormFileAllocate;
impl<E: ScriptEngine> NativeFn<E> for FormFileAllocate {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let record = extract::<E>(cx, "file", value)?;
        let file =
            file_from_wire(&record).ok_or_else(|| type_error::<E>(cx, "Invalid File identity"))?;
        cx.make_string(&file.binding_identity().encode())
    }
}
pub(crate) struct FormFilesReplace;
impl<E: ScriptEngine> NativeFn<E> for FormFilesReplace {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let input = cx.arg(0);
        let actual = cx.arg(1);
        let wire = extract::<E>(cx, "list", actual)?;
        let files = FormWire::new(&wire)
            .files()
            .ok_or_else(|| type_error::<E>(cx, "Invalid File identity"))?;
        let node = cx
            .owned_node(&input)?
            .ok_or_else(|| type_error::<E>(cx, "Invalid input"))?;
        let changed = node.with_dom(|dom| dom.set_selected_files(node.id(), files));
        if changed {
            let kind = cx.make_string("commitSelection")?;
            let realm = private_realm::<E>(cx);
            let actual = cx.arg(1);
            adoption::call_hook::<E>(cx, realm, "formPrivate", vec![kind, input, actual])?;
        }
        cx.make_string(if changed { "true" } else { "false" })
    }
}
pub(crate) struct FormFilesSnapshot;
impl<E: ScriptEngine> NativeFn<E> for FormFilesSnapshot {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let input = cx.arg(0);
        let Some(node) = cx.owned_node(&input)? else {
            return Ok(cx.make_null());
        };
        let Some(files) = node.with_dom(|dom| dom.selected_files(node.id())) else {
            return Ok(cx.make_null());
        };
        let records: Vec<_> = files.iter().map(file_json).collect();
        cx.make_string(&format!("[{}]", records.join(",")))
    }
}
pub(crate) struct FormEntrySnapshot;
impl<E: ScriptEngine> NativeFn<E> for FormEntrySnapshot {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let form = cx.arg(0);
        let submitter = cx.arg(1);
        let encoding = cx.arg(2);
        let x = cx.arg(3);
        let y = cx.arg(4);
        let accumulator = cx.arg(5);
        let node = cx
            .owned_node(&form)?
            .ok_or_else(|| type_error::<E>(cx, "Invalid form"))?;
        let submitter = cx.owned_node(&submitter)?.map(|node| node.id());
        let encoding = cx.value_to_string(&encoding)?;
        let position = (
            cx.value_to_string(&x)?.parse().unwrap_or(0),
            cx.value_to_string(&y)?.parse().unwrap_or(0),
        );
        let entries = node
            .with_dom(|dom| dom.form_submission_entries(node.id(), submitter, &encoding, position))
            .map_err(|_| type_error::<E>(cx, "Invalid submitter"))?;
        for entry in entries {
            let (record, input) = match entry.value {
                FormEntryValue::Text(text) => (
                    format!(
                        "{{\"name\":{},\"text\":{}}}",
                        crate::js_str(&entry.name),
                        crate::js_str(&text)
                    ),
                    None,
                ),
                FormEntryValue::File { file, source_input } => (
                    format!(
                        "{{\"name\":{},\"file\":{}}}",
                        crate::js_str(&entry.name),
                        file_json(&file)
                    ),
                    source_input,
                ),
            };
            let wire = cx.make_string(&record)?;
            let input = if let Some(input) = input {
                reflect_pinned::<E>(cx, input.raw())?
            } else {
                cx.make_null()
            };
            let this = cx.undefined();
            E::call_from_call(cx, &accumulator, &this, &[wire, input])?;
        }
        Ok(cx.undefined())
    }
}
pub(crate) struct FormDataEncode;
impl<E: ScriptEngine> NativeFn<E> for FormDataEncode {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let label = cx.arg(1);
        let kind = cx.arg(2);
        let entries = entries::<E>(cx, value)?;
        let label = cx.value_to_string(&label)?;
        let kind = cx.value_to_string(&kind)?;
        let encoding = genet_scripted_dom::form_encoding::Encoding::for_label(label.as_bytes())
            .unwrap_or(genet_scripted_dom::form_encoding::UTF_8)
            .output_encoding();
        let body = encoded(&entries, encoding, &kind);
        cx.make_string(&format!(
            "{{\"bytes\":{},\"type\":{}}}",
            bytes_json(&body.0),
            crate::js_str(&body.1)
        ))
    }
}
pub(crate) fn encoded(
    entries: &[FormEntry],
    encoding: &'static genet_scripted_dom::form_encoding::Encoding,
    kind: &str,
) -> (Vec<u8>, String) {
    use genet_scripted_dom::form_encoding::*;
    match kind {
        "text/plain" => (encode_text_plain(entries, encoding), "text/plain".into()),
        "multipart/form-data" => {
            let body = encode_multipart_auto(entries, encoding).expect("automatic safe boundary");
            (body.bytes, body.content_type)
        },
        _ => (
            encode_urlencoded(entries, encoding),
            "application/x-www-form-urlencoded".into(),
        ),
    }
}
#[cfg(test)]
struct CorruptFileToken;
#[cfg(test)]
impl<E: ScriptEngine> NativeFn<E> for CorruptFileToken {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let token = cx.arg(1);
        let kind = cx.make_string("testToken")?;
        let realm = cx.current_realm();
        adoption::call_hook::<E>(cx, realm, "formPrivate", vec![kind, value, token])
    }
}
pub(crate) fn install<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<FormFileAllocate>("__formFileAllocate", 1)?;
    engine.set_function::<FormFilesReplace>("__formFilesReplace", 2)?;
    engine.set_function::<FormFilesSnapshot>("__formFilesSnapshot", 1)?;
    engine.set_function::<FormEntrySnapshot>("__formEntrySnapshot", 6)?;
    engine.set_function::<FormDataEncode>("__formDataEncode", 3)?;
    crate::form_submission::install(engine)?;
    engine.eval(include_str!("form_data/bootstrap.js"))?;
    Ok(())
}
#[cfg(test)]
mod phase_c_native_boundary_fixtures {
    use crate::{NoScriptLoader, Runtime};
    use genet_scripted_dom::FormFile;
    use script_engine_api::{CallCx, NativeFn, ScriptEngine, ScriptEngineLive};
    use std::sync::Arc;
    struct MissingAllocate;
    impl<E: ScriptEngine> NativeFn<E> for MissingAllocate {
        fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
            cx.make_string("MissingCapability: genuine File extractor/allocation")
        }
    }
    struct MissingReplace;
    impl<E: ScriptEngine> NativeFn<E> for MissingReplace {
        fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
            cx.make_string("MissingCapability: genuine FileList extractor/replacement")
        }
    }
    struct MissingEncode;
    impl<E: ScriptEngine> NativeFn<E> for MissingEncode {
        fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
            cx.make_string("MissingCapability: genuine FormData extractor/encoder")
        }
    }
    // RED-only missing-installer adapter. GREEN may route these test aliases
    // to actual production NativeFn types; scenario/helper bodies stay frozen.
    fn install_native_adapters<E: ScriptEngineLive>(rt: &mut Runtime<E>) {
        let realm = rt.top_realm();
        rt.engine_mut()
            .set_function_in_realm::<super::FormFileAllocate>(realm, "__phaseCNativeFileAllocate", 1)
            .unwrap();
        rt.engine_mut()
            .set_function_in_realm::<super::FormFilesReplace>(realm, "__phaseCNativeFilesReplace", 2)
            .unwrap();
        rt.engine_mut()
            .set_function_in_realm::<super::FormDataEncode>(realm, "__phaseCNativeFormDataEncode", 3)
            .unwrap();
        rt.engine_mut()
            .set_function_in_realm::<crate::dom::adoption::RegisterHooks>(
                realm,
                "__phaseCTryReplaceHook",
                1,
            )
            .unwrap();
    }
    // Test-only private-slot corruption injection is absent at RED. GREEN must
    // target private fixture wiring, never publish a production authority API.
    fn corrupt_token_adapter<E:ScriptEngineLive>(rt:&mut Runtime<E>,token:&str)->bool{
        let realm=rt.top_realm();
        rt.engine_mut().set_function_in_realm::<super::CorruptFileToken>(realm,"__phaseCCorruptFileToken",2).unwrap();
        let source=format!("__phaseCCorruptFileToken(original,{});delete globalThis.__phaseCCorruptFileToken;",crate::js_str(token));
        rt.eval(&source).is_ok()
    }

    fn run<E: ScriptEngineLive>(rt: &mut Runtime<E>, source: &str) {
        if let Err(error) = rt.eval(source) {
            panic!("{}\n{source}", rt.describe_error(&error));
        }
    }
    fn read<E: ScriptEngineLive>(rt: &mut Runtime<E>, source: &str) -> String {
        let value = rt.eval(source).unwrap();
        rt.value_to_string(&value).unwrap()
    }
    fn fixture<E: ScriptEngineLive>() -> Runtime<E> {
        let mut rt = Runtime::<E>::new().unwrap();
        rt.parse_document_interleaved(
            "<body><form id=f><input id=upload type=file name=upload></form></body>",
            &NoScriptLoader,
        );
        install_native_adapters(&mut rt);
        run(
            &mut rt,
            "var input=document.getElementById('upload'),original=new File([new Uint8Array([0,128,255])],'same.bin',{type:'application/octet-stream'});function status(fn){try{return fn();}catch(e){return e.name;}}",
        );
        rt
    }
    fn seed<E: ScriptEngineLive>(rt: &mut Runtime<E>) -> genet_scripted_dom::FormFileId {
        let value = rt.eval("input.__ref").unwrap();
        let raw = rt.engine_mut().reflector_data(&value).unwrap();
        let file = FormFile::new(
            "prior.bin".into(),
            "application/octet-stream".into(),
            Arc::from([0, 128, 255]),
        )
        .unwrap();
        let id = file.id;
        let top = rt.top_realm();
        let owner = rt.host_in_realm(top).unwrap();
        assert!(
            owner
                .borrow_mut()
                .dom
                .set_selected_files(genet_scripted_dom::NodeId::from_raw(raw), vec![file])
        );
        id
    }
    fn unchanged<E: ScriptEngineLive>(rt: &mut Runtime<E>, id: genet_scripted_dom::FormFileId) {
        let value = rt.eval("input.__ref").unwrap();
        let raw = rt.engine_mut().reflector_data(&value).unwrap();
        let top = rt.top_realm();
        let owner = rt.host_in_realm(top).unwrap();
        let snapshot = owner
            .borrow()
            .dom
            .selected_files(genet_scripted_dom::NodeId::from_raw(raw))
            .unwrap();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].id, id);
        assert_eq!(&*snapshot[0].bytes, &[0, 128, 255]);
    }
    fn genuine_file_positive<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        let token = read(
            &mut rt,
            "status(function(){return __phaseCNativeFileAllocate(original);})",
        );
        assert!(
            token.starts_with("genet-form-file-v1:"),
            "genuine File positive control: {token}"
        );
        assert_eq!(
            read(&mut rt, "__phaseCNativeFileAllocate(original)"),
            token,
            "same genuine File keeps allocation"
        );
    }
    fn fake_file_negative<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        assert_eq!(
            read(
                &mut rt,
                "status(function(){return __phaseCNativeFileAllocate({_b:new Uint8Array([0,128,255]),name:'same.bin',type:'application/octet-stream',id:'1'});})"
            ),
            "TypeError",
            "missing installer is not authoritative rejection"
        );
    }
    fn genuine_list_positive<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        run(
            &mut rt,
            "if(typeof DataTransfer!=='function')throw new Error('missing capability: genuine FileList positive control');var dt=new DataTransfer();dt.items.add(original);",
        );
        assert_eq!(
            read(
                &mut rt,
                "status(function(){return __phaseCNativeFilesReplace(input.__ref,dt.files);})"
            ),
            "true",
            "genuine FileList accepted by actual native callable"
        );
        run(
            &mut rt,
            "if(input.files[0]!==original || new FormData(document.getElementById('f')).get('upload')!==original)throw new Error('native genuine replacement identity');",
        );
    }
    fn fake_list_negative_atomic<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        let prior = seed(&mut rt);
        for expression in [
            "{length:1,0:original}",
            "'{\"files\":[{\"id\":\"1\",\"name\":\"fake\",\"bytes\":[9]}]}'",
            "Object.create(globalThis.FileList?FileList.prototype:null)",
            "1",
        ] {
            assert_eq!(
                read(
                    &mut rt,
                    &format!(
                        "status(function(){{return __phaseCNativeFilesReplace(input.__ref,{expression});}})"
                    )
                ),
                "TypeError",
                "direct raw/fake list must reject before mutation"
            );
            unchanged(&mut rt, prior);
        }
    }
    fn genuine_fd_positive<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        run(
            &mut rt,
            "var fd=new FormData();fd.append('upload',original);",
        );
        let result = read(
            &mut rt,
            "status(function(){return __phaseCNativeFormDataEncode(fd,'UTF-8','multipart/form-data');})",
        );
        assert!(
            result.contains("multipart/form-data; boundary="),
            "genuine FormData native encoder positive: {result}"
        );
    }
    fn fake_fd_negative<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        for expression in [
            "{_l:[['upload',original]]}",
            "Object.create(FormData.prototype)",
            "'{\"entries\":[[\"x\",\"forged\"]]}'",
        ] {
            assert_eq!(
                read(
                    &mut rt,
                    &format!(
                        "status(function(){{return __phaseCNativeFormDataEncode({expression},'UTF-8','multipart/form-data');}})"
                    )
                ),
                "TypeError",
                "genuine extraction at direct encoder boundary"
            );
        }
    }
    fn re_registration_cannot_replace_extractor<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        run(
            &mut rt,
            "var intercepted=0;__phaseCTryReplaceHook(function(){intercepted++;return 'forged';});",
        );
        let token = read(&mut rt, "__phaseCNativeFileAllocate(original)");
        assert!(
            token.starts_with("genet-form-file-v1:"),
            "positive extractor remains installed: {token}"
        );
        assert_eq!(read(&mut rt, "intercepted"), "0");
    }
    fn invalid_token_atomic<E: ScriptEngineLive>() {
        let mut rt = fixture::<E>();
        let prior = seed(&mut rt);
        run(
            &mut rt,
            "if(typeof DataTransfer!=='function')throw new Error('missing capability: invalid-token genuine FileList');var dt=new DataTransfer();dt.items.add(original);",
        );
        assert!(
            corrupt_token_adapter(&mut rt, "genet-form-file-v1:ffffffffffffffff"),
            "missing capability: private test-only token fault injection"
        );
        assert_eq!(
            read(
                &mut rt,
                "status(function(){return __phaseCNativeFilesReplace(input.__ref,dt.files);})"
            ),
            "TypeError",
            "malformed private identity rejects at genuine-value native mutation boundary"
        );
        unchanged(&mut rt, prior);
    }
    macro_rules! both{($($name:ident=>$case:ident),*$(,)?)=>{$(mod $name{#[test]fn boa(){super::$case::<script_engine_boa::BoaEngine>();}#[cfg(target_pointer_width="64")]#[test]fn nova(){super::$case::<script_engine_nova::NovaEngine>();}})*};}
    both! {
        phase_c_form_native_boundary_genuine_file=>genuine_file_positive,
        phase_c_form_native_boundary_fake_file=>fake_file_negative,
        phase_c_form_native_boundary_genuine_list=>genuine_list_positive,
        phase_c_form_native_boundary_fake_list_atomic=>fake_list_negative_atomic,
        phase_c_form_native_boundary_genuine_fd=>genuine_fd_positive,
        phase_c_form_native_boundary_fake_fd=>fake_fd_negative,
        phase_c_form_native_boundary_re_registration=>re_registration_cannot_replace_extractor,
        phase_c_form_native_boundary_invalid_token_atomic=>invalid_token_atomic,
    }
}
