// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Regressions found after the headed Realms acceptance.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

#[derive(Clone, Default)]
struct Resources(Rc<RefCell<Vec<String>>>);

impl ScriptResourceLoader for Resources {
    fn load(&self, url: &str) -> Option<String> {
        self.0.borrow_mut().push(url.to_owned());
        let path = url.strip_prefix("https://realm.test/")?;
        match path {
            "next/page.html" => Some(concat!(
                "<body><script>var order = ['inline'];</script>",
                "<script id=external src='first.js'></script>",
                "<p id=after>after</p>",
                "<script src='missing.js'>throw new Error('external fallback ran');</script>",
                "<script type='text/plain' src='inert.js'></script>",
                "<script>order.push('after');</script></body>"
            ).into()),
            "next/first.js" => Some(concat!(
                "if (document.getElementById('after')) throw new Error('parser ran ahead');",
                "if (document.currentScript.id !== 'external') throw new Error('currentScript');",
                "order.push('external');",
                "document.write('<script src=\"nested.js\"><\\/script>');",
                "if (document.currentScript.id !== 'external') throw new Error('nested currentScript');"
            ).into()),
            "next/nested.js" => Some("order.push('nested');".into()),
            _ => None,
        }
    }
}

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) -> String {
    let value = rt.eval(source).expect(source);
    rt.value_to_string(&value).unwrap()
}

fn navigation_fetches_external_classic_scripts<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    let resources = Resources::default();
    rt.set_script_resource_loader(Box::new(resources.clone()));
    rt.set_base_url("https://realm.test/start.html").unwrap();
    rt.parse_document_interleaved("<body>start</body>", &NoScriptLoader);
    rt.run_event_loop(30).unwrap();
    for _ in 0..2 {
        let previous = rt.top_realm();
        rt.eval("location.href = 'https://realm.test/next/page.html';")
            .unwrap();
        rt.run_event_loop(60).unwrap();
        assert_ne!(previous, rt.top_realm());
        assert_eq!(
            read(&mut rt, "order.join(',')"),
            "inline,external,nested,after"
        );
        assert_eq!(read(&mut rt, "String(document.currentScript)"), "null");
        assert_eq!(read(&mut rt, "document.readyState"), "complete");
        rt.collect_garbage();
    }
    assert_eq!(
        *resources.0.borrow(),
        [
            "https://realm.test/next/page.html",
            "https://realm.test/next/first.js",
            "https://realm.test/next/nested.js",
            "https://realm.test/next/missing.js",
            "https://realm.test/next/page.html",
            "https://realm.test/next/first.js",
            "https://realm.test/next/nested.js",
            "https://realm.test/next/missing.js",
        ]
    );
}

fn removed_location_keeps_document_url_separate<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://realm.test/page.html?query#fragment")
        .unwrap();
    rt.parse_document_interleaved(
        "<body><iframe id=f srcdoc='<p>child</p>'></iframe></body>",
        &NoScriptLoader,
    );
    rt.run_event_loop(40).unwrap();
    rt.eval(concat!(
        "var f = document.getElementById('f'), win = f.contentWindow;",
        "var doc = win.document, loc = win.location, oldURL = doc.URL; f.remove();"
    ))
    .unwrap();
    let top = rt.top_realm();
    for after_teardown in [false, true] {
        if after_teardown {
            rt.run_event_loop(40).unwrap();
        }
        assert_eq!(
            read(
                &mut rt,
                "[loc.href, loc.protocol, loc.host, loc.hostname, loc.port, loc.pathname, loc.search, loc.hash, loc.origin].join('|')"
            ),
            "about:blank|about:||||blank|||null"
        );
        assert_eq!(
            read(
                &mut rt,
                "String(win.document === doc && doc.URL === oldURL && win.location === loc)"
            ),
            "true"
        );
        rt.eval("loc.href='http://:'; loc.assign('https://realm.test/away'); loc.replace('about:blank'); loc.reload();").unwrap();
        rt.run_event_loop(40).unwrap();
        assert_eq!(rt.top_realm(), top);
        assert_eq!(read(&mut rt, "loc.href"), "about:blank");
        assert_eq!(
            read(&mut rt, "location.href"),
            "https://realm.test/page.html?query#fragment"
        );
    }
}

fn child_navigation_fetches_external_classic_scripts<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_script_resource_loader(Box::new(Resources::default()));
    rt.set_base_url("https://realm.test/start.html").unwrap();
    rt.parse_document_interleaved(
        "<body><iframe id=f src='next/page.html'></iframe></body>",
        &NoScriptLoader,
    );
    rt.run_event_loop(60).unwrap();
    for _ in 0..2 {
        assert_eq!(
            read(
                &mut rt,
                "document.getElementById('f').contentWindow.order.join(',')"
            ),
            "inline,external,nested,after"
        );
        rt.collect_garbage();
        rt.eval("document.getElementById('f').src = 'next/page.html';")
            .unwrap();
        rt.run_event_loop(60).unwrap();
    }
}

struct DropProbe(Rc<Cell<usize>>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
impl ScriptResourceLoader for DropProbe {
    fn load(&self, _: &str) -> Option<String> {
        None
    }
}

fn runtime_drop_releases_discarded_host_resources<E: ScriptEngine>() {
    let dropped = Rc::new(Cell::new(0));
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://realm.test/").unwrap();
    rt.set_script_resource_loader(Box::new(DropProbe(dropped.clone())));
    rt.parse_document_interleaved("<body><iframe id=f></iframe></body>", &NoScriptLoader);
    rt.run_event_loop(40).unwrap();
    let child = rt.frame_realms(rt.top_realm())[0].1;
    let old_host = rt.host_in_realm(child).unwrap();
    rt.eval("var held = document.getElementById('f').contentWindow; document.getElementById('f').remove();").unwrap();
    rt.run_event_loop(40).unwrap();
    assert!(rt.host_in_realm(child).is_err());
    assert!(old_host.borrow().script_loader.is_some());
    drop(rt);
    assert_eq!(
        dropped.get(),
        1,
        "a discarded realm kept host resources past Runtime::drop"
    );
    assert!(old_host.borrow().script_loader.is_none());
}

macro_rules! backend {
    ($name:ident, $engine:ty) => {
        mod $name {
            #[test]
            fn external_classic_navigation() {
                super::navigation_fetches_external_classic_scripts::<$engine>();
            }
            #[test]
            fn child_external_classic_navigation() {
                super::child_navigation_fetches_external_classic_scripts::<$engine>();
            }
            #[test]
            fn discarded_location() {
                super::removed_location_keeps_document_url_separate::<$engine>();
            }
            #[test]
            fn discarded_host_drop() {
                super::runtime_drop_releases_discarded_host_resources::<$engine>();
            }
        }
    };
}
backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
