// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Real loader, parser-time metadata and owning-Document encoding receipts.
use layout_dom_api::LayoutDom;
use script_engine_api::ScriptEngine;
use script_runtime_api::{FetchOutcome, NoScriptLoader, Runtime, ScriptResourceLoader};

struct Documents;
impl ScriptResourceLoader for Documents {
    fn load(&self, url: &str) -> Option<String> {
        match url.rsplit('/').next()? {
            "unicode" => Some("<meta charset=shift_jis><p id=v>é</p><script>window.seen=document.characterSet</script>".into()),
            _ => None,
        }
    }
    fn load_document(&self, url: &str) -> Option<FetchOutcome> {
        let (header, bytes): (&str, &[u8]) = match url.rsplit('/').next()? {
            "sj" => ("text/html;charset=shift_jis", b"<form id=f><input id=v name=q value='\x82\xA0'></form><script>window.seen=document.characterSet+'|'+document.getElementById('v').value</script>"),
            "kr" => ("text/html;charset=euc-kr", b"<input id=v value='\xB0\xA1'><script>window.seen=document.characterSet+'|'+document.getElementById('v').value</script>"),
            "meta" => ("text/html", b"<meta charset=shift_jis><input id=v value='\x82\xA0'><script>window.seen=document.characterSet+'|'+document.getElementById('v').value</script>"),
            "bom" => ("text/html;charset=windows-1252", b"\xEF\xBB\xBF<p id=v>\xC3\xA9</p><script>window.seen=document.characterSet+'|'+document.getElementById('v').textContent</script>"),
            "fallback" => ("text/html", b"<input id=v value='\xE9'><script>window.seen=document.characterSet+'|'+document.getElementById('v').value</script>"),
            "no-content" => return Some(FetchOutcome { network_error: false, status: 204, status_text: String::new(), response_type: "basic".into(), url: url.into(), redirected: false, headers: vec![], body: vec![] }),
            _ => return None,
        };
        Some(FetchOutcome {
            network_error: false,
            status: 200,
            status_text: "OK".into(),
            response_type: "basic".into(),
            url: url.replace("/meta", "/final-meta"),
            redirected: url.ends_with("/meta"),
            headers: vec![("Content-Type".into(), header.into())],
            body: bytes.to_vec(),
        })
    }
}
fn read<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) -> String {
    let value = rt.eval(source).expect("evaluate projection");
    rt.value_to_string(&value).expect("string projection")
}
fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.set_base_url("https://encoding.test/start").unwrap();
    rt.set_script_resource_loader(Box::new(Documents));
    rt.parse_document_interleaved("<body><p id=original>kept</p></body>", &NoScriptLoader);
    rt
}
fn navigate<E: ScriptEngine>(rt: &mut Runtime<E>, page: &str) {
    rt.eval(&format!("location.href='https://encoding.test/{page}'"))
        .unwrap();
    rt.run_event_loop(100).unwrap();
}
// RED-only missing caller configuration adapter. GREEN changes this adapter
// alone to invoke the additive runtime fallback setter; fixture values remain.
fn configure_windows1252<E: ScriptEngine>(rt: &mut Runtime<E>) {
    rt.set_html_decode_options(script_runtime_api::HtmlDecodeOptions {
        fallback: genet_scripted_dom::form_encoding::Encoding::for_label(b"windows-1252").unwrap(),
        ..Default::default()
    });
}
fn document_confidence<E: ScriptEngine>(rt: &Runtime<E>) -> Option<&'static str> {
    let h = rt.host().borrow();
    h.dom
        .document_encoding(h.dom.document())
        .map(|e| e.confidence.as_str())
}

fn initial_rich_iframe<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var frame=document.createElement('iframe'); frame.src='https://encoding.test/sj'; document.body.appendChild(frame)").unwrap();
    rt.run_event_loop(100).unwrap();
    assert_eq!(read(&mut rt, "frame.contentWindow.seen"), "Shift_JIS|あ");
    assert_eq!(
        read(&mut rt, "frame.contentDocument.characterSet"),
        "Shift_JIS"
    );
}
fn navigation_meta_and_final_url<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    navigate(&mut rt, "meta");
    assert_eq!(
        read(&mut rt, "document.getElementById('v').value"),
        "あ",
        "real navigation meta decoding"
    );
    assert_eq!(read(&mut rt, "seen"), "Shift_JIS|あ");
    assert_eq!(
        read(&mut rt, "location.href"),
        "https://encoding.test/final-meta"
    );
    navigate(&mut rt, "kr");
    assert_eq!(
        read(&mut rt, "seen"),
        "EUC-KR|가",
        "replacement metadata before authored script"
    );
    navigate(&mut rt, "bom");
    assert_eq!(
        read(&mut rt, "seen"),
        "UTF-8|é",
        "actual BOM-selected encoding"
    );
}
fn unicode_secondary_defaults<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    navigate(&mut rt, "sj");
    assert_eq!(read(&mut rt, "document.characterSet"), "Shift_JIS");
    assert_eq!(
        read(
            &mut rt,
            "new DOMParser().parseFromString('<meta charset=euc-kr><p>é</p>','text/html').characterSet"
        ),
        "UTF-8"
    );
    assert_eq!(
        read(
            &mut rt,
            "new DOMParser().parseFromString('<?xml version=\"1.0\" encoding=\"euc-kr\"?><p/>','text/xml').characterSet"
        ),
        "UTF-8"
    );
    assert_eq!(
        read(
            &mut rt,
            "document.implementation.createHTMLDocument('é').characterSet"
        ),
        "UTF-8"
    );
    assert_eq!(read(&mut rt, "new Document().characterSet"), "UTF-8");
    navigate(&mut rt, "unicode");
    assert_eq!(
        read(&mut rt, "seen+'|'+document.getElementById('v').textContent"),
        "UTF-8|é"
    );
}
fn blank_srcdoc_and_aliases<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    navigate(&mut rt, "sj");
    rt.eval("var blank=document.createElement('iframe'); document.body.appendChild(blank); var src=document.createElement('iframe'); src.srcdoc='<meta charset=euc-kr><p id=x>é</p><script>window.seen=document.characterSet</script>'; document.body.appendChild(src)").unwrap();
    rt.run_event_loop(100).unwrap();
    assert_eq!(read(&mut rt, "blank.contentDocument.characterSet"), "UTF-8");
    assert_eq!(
        read(
            &mut rt,
            "src.contentWindow.seen+'|'+src.contentDocument.getElementById('x').textContent"
        ),
        "UTF-8|é"
    );
    assert_eq!(
        read(
            &mut rt,
            "document.characterSet+'|'+document.charset+'|'+document.inputEncoding"
        ),
        "Shift_JIS|Shift_JIS|Shift_JIS"
    );
}
fn adoption_observes_destination_document<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var child=document.createElement('iframe'); child.src='https://encoding.test/sj'; document.body.appendChild(child)").unwrap();
    rt.run_event_loop(100).unwrap();
    assert_eq!(
        read(&mut rt, "child.contentDocument.characterSet"),
        "Shift_JIS"
    );
    rt.eval("var source=child.contentDocument; var form=source.getElementById('f'); var before=form.ownerDocument.characterSet; document.adoptNode(form); document.body.appendChild(form); child.remove()").unwrap();
    assert_eq!(
        read(
            &mut rt,
            "before+'|'+form.ownerDocument.characterSet+'|'+form.querySelector('input').value"
        ),
        "Shift_JIS|UTF-8|あ"
    );
    assert_eq!(
        read(&mut rt, "source.characterSet"),
        "Shift_JIS",
        "retained detached source receiver keeps its metadata"
    );
}
fn clone_keeps_source_encoding<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    navigate(&mut rt, "sj");
    assert_eq!(read(&mut rt, "document.characterSet"), "Shift_JIS");
    assert_eq!(
        read(&mut rt, "document.cloneNode(false).characterSet"),
        "Shift_JIS"
    );
    assert_eq!(
        read(&mut rt, "document.cloneNode(true).characterSet"),
        "Shift_JIS"
    );
}
fn document_open_keeps_encoding_with_irrelevant_confidence<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    navigate(&mut rt, "sj");
    assert_eq!(read(&mut rt, "document.characterSet"), "Shift_JIS");
    rt.eval(
        "document.open();document.write('<meta charset=euc-kr><p id=x>é</p>');document.close()",
    )
    .unwrap();
    assert_eq!(
        read(
            &mut rt,
            "document.characterSet+'|'+document.getElementById('x').textContent"
        ),
        "Shift_JIS|é"
    );
    assert_eq!(document_confidence(&rt), Some("irrelevant"));
}
fn configured_fallback_reaches_navigation_and_child<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    configure_windows1252(&mut rt);
    navigate(&mut rt, "fallback");
    assert_eq!(read(&mut rt, "document.getElementById('v').value"), "é");
    assert_eq!(read(&mut rt, "seen"), "windows-1252|é");
    rt.eval("var child=document.createElement('iframe'); child.src='https://encoding.test/fallback';document.body.appendChild(child)").unwrap();
    rt.run_event_loop(100).unwrap();
    assert_eq!(read(&mut rt, "child.contentWindow.seen"), "windows-1252|é");
}
macro_rules! cases {
    ($module:ident,$engine:ty) => {
        mod $module {
            use super::*;
            #[test]
            fn phase_c_document_encoding_initial_rich_iframe() {
                initial_rich_iframe::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_navigation_meta_and_final_url() {
                navigation_meta_and_final_url::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_unicode_secondary_defaults() {
                unicode_secondary_defaults::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_blank_srcdoc_and_aliases() {
                blank_srcdoc_and_aliases::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_adoption_destination() {
                adoption_observes_destination_document::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_clone_keeps_source() {
                clone_keeps_source_encoding::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_document_open() {
                document_open_keeps_encoding_with_irrelevant_confidence::<$engine>();
            }
            #[test]
            fn phase_c_document_encoding_configured_fallback() {
                configured_fallback_reaches_navigation_and_child::<$engine>();
            }
        }
    };
}
cases!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
cases!(nova, script_engine_nova::NovaEngine);
