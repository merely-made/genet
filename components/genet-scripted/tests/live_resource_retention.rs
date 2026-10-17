#![cfg(feature = "livery")]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use genet_document_resources::{ResourceFetcher, ResourceKind, ResourceLimits, ResourceResponse};
use genet_livery::Device;
use genet_scripted::LiveryCssom;
use genet_static_dom::StaticDocument;
use script_engine_boa::BoaEngine;
use script_runtime_api::Runtime;

const PAGE: &str = "https://example.test/docs/index.html";

#[derive(Clone, Default)]
struct CountingFetcher {
    calls: Rc<RefCell<Vec<String>>>,
    responses: Rc<RefCell<HashMap<String, Option<Vec<u8>>>>>,
}

impl CountingFetcher {
    fn add(&self, url: &str, body: impl Into<Vec<u8>>) {
        self.responses
            .borrow_mut()
            .insert(url.to_owned(), Some(body.into()));
    }

    fn miss(&self, url: &str) {
        self.responses.borrow_mut().insert(url.to_owned(), None);
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ResourceFetcher for CountingFetcher {
    fn fetch_request(
        &self,
        _request: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        None
    }

    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.calls.borrow_mut().push(url.to_owned());
        self.responses.borrow().get(url).cloned().flatten()
    }
}

#[derive(Clone, Default)]
struct RedirectingFetcher {
    calls: Rc<RefCell<Vec<String>>>,
    responses: Rc<RefCell<HashMap<String, Option<ResourceResponse>>>>,
}

impl RedirectingFetcher {
    fn respond(&self, requested: &str, final_url: &str, body: impl Into<Vec<u8>>) {
        self.responses.borrow_mut().insert(
            requested.to_owned(),
            Some(ResourceResponse::new(final_url, body.into())),
        );
    }

    fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ResourceFetcher for RedirectingFetcher {
    fn fetch_request(
        &self,
        _request: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        None
    }

    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.calls.borrow_mut().push(url.to_owned());
        self.responses.borrow().get(url).cloned().flatten()
    }
}

fn device() -> Device {
    Device::screen(800.0, 600.0)
}

fn count(calls: &[String], url: &str) -> usize {
    calls.iter().filter(|called| called.as_str() == url).count()
}

#[test]
fn unrelated_dom_work_reuses_successes_and_negative_results() {
    let fetcher = CountingFetcher::default();
    fetcher.add(
        "https://example.test/docs/theme.css",
        b"@import url('child.css'); .card { color: green; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/child.css",
        b"#card { color: blue; }".to_vec(),
    );
    fetcher.miss("https://example.test/docs/missing.png");
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><link rel=stylesheet href=theme.css><style>.card { background-image: url(missing.png) }</style></head>\
         <body><div id=card class=card>before</div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );

    let initial_calls = fetcher.calls();
    assert_eq!(
        count(&initial_calls, "https://example.test/docs/theme.css"),
        1
    );
    assert_eq!(
        count(&initial_calls, "https://example.test/docs/child.css"),
        1
    );
    assert_eq!(
        count(&initial_calls, "https://example.test/docs/missing.png"),
        1
    );
    assert_eq!(cssom.resource_set().unwrap().resources.len(), 0);

    runtime
        .eval(
            "var card = document.getElementById('card');\
             console.log(getComputedStyle(card).color);\
             card.firstChild.textContent = 'after';\
             card.className = 'card changed';\
             console.log(getComputedStyle(card).color);\
             console.log(getComputedStyle(card).color);",
        )
        .expect("unrelated DOM mutations and style reads");
    runtime
        .eval(
            "var transient = document.createElement('div');\
             document.body.appendChild(transient);\
             document.body.removeChild(transient);\
             transient = null;",
        )
        .expect("append and remove transient node");
    let _ = runtime.collect_garbage();
    cssom.frame(&mut runtime, 800, 600).expect("paint frame");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 0, 255)", "rgb(0, 0, 255)", "rgb(0, 0, 255)"]
    );
    cssom
        .frame(&mut runtime, 800, 600)
        .expect("cached paint frame");
    assert_eq!(
        fetcher.calls(),
        initial_calls,
        "unrelated work reuses fetched and missing dependencies"
    );
    assert_eq!(
        count(&fetcher.calls(), "https://example.test/docs/missing.png"),
        1
    );
}

#[test]
fn relevant_url_changes_fetch_new_dependencies_and_update_the_ledger() {
    let fetcher = CountingFetcher::default();
    fetcher.add(
        "https://example.test/docs/theme.css",
        b"@import url('child.css'); .card { color: green; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/child.css",
        b"#card { color: blue; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/next.css",
        b"@import url('next-child.css'); .card { color: orange; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/next-child.css",
        b"#card { color: purple; }".to_vec(),
    );
    fetcher.add("https://example.test/docs/one.png", vec![1]);
    fetcher.add("https://example.test/docs/two.png", vec![2]);
    fetcher.add(
        "https://cdn.example.test/assets/theme.css",
        b"#card { color: red; }".to_vec(),
    );
    fetcher.add("https://cdn.example.test/assets/poster.png", vec![3]);
    fetcher.add("https://cdn.example.test/assets/three.png", vec![4]);

    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><base href='/docs/'><link id=theme rel=stylesheet href=theme.css>\
         <style id=asset>.card { background-image: url(one.png) }</style></head>\
         <body><img id=poster src=one.png><div id=card class=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );

    runtime
        .eval(
            "var card = document.getElementById('card');\
             console.log(getComputedStyle(card).color);\
             document.getElementById('theme').setAttribute('href', 'next.css');\
             console.log(getComputedStyle(card).color);\
             document.getElementById('poster').setAttribute('src', 'two.png');\
             document.getElementById('asset').textContent = '.card { background-image: url(two.png) }';\
             console.log(getComputedStyle(card).color);\
             document.querySelector('base').setAttribute('href', 'https://cdn.example.test/assets/');\
             document.getElementById('theme').setAttribute('href', 'theme.css');\
             document.getElementById('poster').setAttribute('src', 'poster.png');\
             document.getElementById('asset').textContent = '.card { background-image: url(three.png) }';\
             console.log(getComputedStyle(card).color);",
        )
        .expect("stylesheet, image, inline stylesheet, and base URL changes");

    assert_eq!(
        runtime.host().borrow().console,
        vec![
            "rgb(0, 0, 255)",
            "rgb(128, 0, 128)",
            "rgb(128, 0, 128)",
            "rgb(255, 0, 0)"
        ]
    );
    let calls = fetcher.calls();
    for url in [
        "https://example.test/docs/theme.css",
        "https://example.test/docs/child.css",
        "https://example.test/docs/next.css",
        "https://example.test/docs/next-child.css",
        "https://example.test/docs/one.png",
        "https://example.test/docs/two.png",
        "https://cdn.example.test/assets/theme.css",
        "https://cdn.example.test/assets/poster.png",
        "https://cdn.example.test/assets/three.png",
    ] {
        assert_eq!(count(&calls, url), 1, "expected one acquisition for {url}");
    }
    let resources = cssom.resource_set().expect("resource ledger");
    assert!(resources.resources.iter().any(|resource| {
        resource.kind == ResourceKind::Image
            && resource.resolved_url == "https://cdn.example.test/assets/three.png"
    }));
    assert!(!resources.resources.iter().any(|resource| {
        resource.resolved_url == "https://example.test/docs/one.png"
            || resource.resolved_url == "https://example.test/docs/two.png"
    }));
}

#[test]
fn removed_owner_is_evicted_and_same_url_reintroduction_refetches() {
    let fetcher = CountingFetcher::default();
    fetcher.add(
        "https://example.test/docs/theme.css",
        b".card { color: green; }".to_vec(),
    );
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><link id=theme rel=stylesheet href=theme.css></head>\
         <body><div id=card class=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );
    runtime
        .eval("console.log(getComputedStyle(document.getElementById('card')).color);")
        .expect("initial computed style");
    assert_eq!(
        count(&fetcher.calls(), "https://example.test/docs/theme.css"),
        1
    );

    runtime
        .eval(
            "document.head.removeChild(document.getElementById('theme'));\
             document.styleSheets.length;",
        )
        .expect("remove original stylesheet owner");
    assert!(cssom.resource_set().unwrap().stylesheets.is_empty());
    fetcher.add(
        "https://example.test/docs/theme.css",
        b".card { color: orange; }".to_vec(),
    );
    runtime
        .eval(
            "var link = document.createElement('link');\
             link.setAttribute('id', 'theme2'); link.setAttribute('rel', 'stylesheet'); link.setAttribute('href', 'theme.css');\
             document.head.appendChild(link);\
             console.log(getComputedStyle(document.getElementById('card')).color);",
        )
        .expect("reintroduce same URL under a new owner");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 128, 0)", "rgb(255, 165, 0)"]
    );
    assert_eq!(
        count(&fetcher.calls(), "https://example.test/docs/theme.css"),
        2
    );
    assert_eq!(
        cssom.resource_set().unwrap().stylesheets[0].text,
        ".card { color: orange; }"
    );

    let mut replacement = Runtime::<BoaEngine>::new().expect("replacement runtime");
    replacement.set_base_url(PAGE).unwrap();
    replacement.load_dom(&StaticDocument::parse(
        "<html><head><link rel=stylesheet href=theme.css></head>\
         <body><div id=card class=card>new arena</div></body></html>",
    ));
    fetcher.add(
        "https://example.test/docs/theme.css",
        b".card { color: red; }".to_vec(),
    );
    cssom
        .frame(&mut replacement, 800, 600)
        .expect("replacement arena frame");
    assert_eq!(
        count(&fetcher.calls(), "https://example.test/docs/theme.css"),
        3
    );
    assert_eq!(
        cssom.resource_set().unwrap().stylesheets[0].text,
        ".card { color: red; }",
        "a fresh DOM arena gets fresh bytes even at the same URL"
    );
}

#[test]
fn reordering_existing_inline_sheets_updates_cascade_without_fetching() {
    let fetcher = CountingFetcher::default();
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><style id=first>.card { color: red; }</style>\
         <style id=second>.card { color: blue; }</style></head>\
         <body><div id=card class=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );

    runtime
        .eval(
            "var card = document.getElementById('card');\
             console.log(getComputedStyle(card).color);\
             document.head.appendChild(document.getElementById('first'));\
             console.log(getComputedStyle(card).color);",
        )
        .expect("reorder two retained inline stylesheet owners");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 0, 255)", "rgb(255, 0, 0)"]
    );
    assert!(fetcher.calls().is_empty());
    assert_eq!(
        cssom
            .resource_set()
            .unwrap()
            .stylesheets
            .iter()
            .map(|sheet| sheet.owner_node)
            .collect::<Vec<_>>()
            .len(),
        2
    );
}

#[test]
fn replacing_redirected_root_reacquires_nested_imports_at_the_same_final_urls() {
    let fetcher = RedirectingFetcher::default();
    let requested_root = "https://example.test/docs/theme.css";
    let root_v1 = "https://cdn.example.test/styles/v1/root.css";
    let child_v1_request = "https://cdn.example.test/styles/v1/nested/child.css";
    let child_v1_final = "https://assets.example.test/sheets/v1/child.css";
    let leaf_v1_request = "https://assets.example.test/sheets/v1/leaf.css";
    let leaf_v1_final = "https://assets.example.test/sheets/v1/leaf-final.css";
    fetcher.respond(
        requested_root,
        root_v1,
        b"@import url('nested/child.css'); .card { color: green; }".to_vec(),
    );
    fetcher.respond(
        child_v1_request,
        child_v1_final,
        b"@import url('leaf.css'); .card { color: green; }".to_vec(),
    );
    fetcher.respond(
        leaf_v1_request,
        leaf_v1_final,
        b"#card { color: blue; }".to_vec(),
    );

    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><link id=theme rel=stylesheet href=theme.css></head>\
         <body><div id=card class=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );
    runtime
        .eval("console.log(getComputedStyle(document.getElementById('card')).color);")
        .expect("initial redirected import cascade");
    assert_eq!(runtime.host().borrow().console, vec!["rgb(0, 0, 255)"]);

    // Replace the stylesheet owner within one JS turn, before CSSOM can resolve
    // an intermediate DOM. Every requested and final URL remains the same, but
    // the host now has fresh bytes for both imported sheets.
    runtime
        .eval(
            "var head = document.head;\
             var old = document.getElementById('theme');\
             var replacement = document.createElement('link');\
             replacement.setAttribute('id', 'replacement'); replacement.setAttribute('rel', 'stylesheet'); replacement.setAttribute('href', 'theme.css');\
             head.removeChild(old); head.appendChild(replacement);",
        )
        .expect("replace root owner at the same URL in one DOM turn");

    fetcher.respond(
        requested_root,
        root_v1,
        b"@import url('nested/child.css'); .card { color: orange; }".to_vec(),
    );
    fetcher.respond(
        child_v1_request,
        child_v1_final,
        b"@import url('leaf.css'); .card { color: orange; }".to_vec(),
    );
    fetcher.respond(
        leaf_v1_request,
        leaf_v1_final,
        b"#card { color: purple; }".to_vec(),
    );
    runtime
        .eval("console.log(getComputedStyle(document.getElementById('card')).color);")
        .expect("refresh redirected root and nested import");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 0, 255)", "rgb(128, 0, 128)"]
    );
    let calls = fetcher.calls();
    assert_eq!(count(&calls, requested_root), 2);
    assert_eq!(count(&calls, child_v1_request), 2);
    assert_eq!(count(&calls, leaf_v1_request), 2);
    let resources = cssom.resource_set().expect("redirected resource graph");
    let root = resources
        .stylesheets
        .iter()
        .find(|sheet| sheet.owner == genet_document_resources::StylesheetOwner::Linked)
        .expect("linked root stylesheet");
    assert_eq!(root.requested_url.as_deref(), Some(requested_root));
    assert_eq!(root.source_url.as_deref(), Some(root_v1));
    let imported = resources
        .stylesheets
        .iter()
        .find(|sheet| {
            sheet.owner == genet_document_resources::StylesheetOwner::Imported
                && sheet.requested_url.as_deref() == Some(child_v1_request)
        })
        .expect("child imported sheet");
    assert_eq!(imported.source_url.as_deref(), Some(child_v1_final));
    assert!(imported.text.contains("color: orange"));
    let leaf = resources
        .stylesheets
        .iter()
        .find(|sheet| {
            sheet.owner == genet_document_resources::StylesheetOwner::Imported
                && sheet.requested_url.as_deref() == Some(leaf_v1_request)
        })
        .expect("grandchild imported sheet");
    assert_eq!(leaf.source_url.as_deref(), Some(leaf_v1_final));
    assert!(leaf.text.contains("color: purple"));
}

fn assert_same_link_owner_reinserted_refetches_dependencies(replace_parent_text: bool) {
    let fetcher = CountingFetcher::default();
    fetcher.add(
        "https://example.test/docs/theme.css",
        b"@import url('child.css'); .card { color: green; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/child.css",
        b"#card { color: blue; }".to_vec(),
    );
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head><link id=theme rel=stylesheet href=theme.css></head>\
         <body><div id=card class=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );
    runtime
        .eval("console.log(getComputedStyle(document.getElementById('card')).color);")
        .expect("initial cascade");

    fetcher.add(
        "https://example.test/docs/theme.css",
        b"@import url('child.css'); .card { color: orange; }".to_vec(),
    );
    fetcher.add(
        "https://example.test/docs/child.css",
        b"#card { color: purple; }".to_vec(),
    );
    let mutation = if replace_parent_text {
        "head.textContent = ''; head.appendChild(link);"
    } else {
        "head.removeChild(link); head.appendChild(link);"
    };
    runtime
        .eval(&format!(
            "var head = document.head; var link = document.getElementById('theme'); {mutation}"
        ))
        .expect("detach and reinsert same link owner in one DOM turn");
    runtime
        .eval("console.log(getComputedStyle(document.getElementById('card')).color);")
        .expect("reconcile changed resource responses");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 0, 255)", "rgb(128, 0, 128)"]
    );
    let calls = fetcher.calls();
    assert_eq!(count(&calls, "https://example.test/docs/theme.css"), 2);
    assert_eq!(count(&calls, "https://example.test/docs/child.css"), 2);
    let resources = cssom.resource_set().unwrap();
    let linked = resources
        .stylesheets
        .iter()
        .find(|sheet| sheet.owner == genet_document_resources::StylesheetOwner::Linked)
        .expect("retained link owner");
    assert!(linked.text.contains("color: orange"));
}

#[test]
fn same_link_owner_removed_and_reinserted_in_one_turn_refetches_dependencies() {
    assert_same_link_owner_reinserted_refetches_dependencies(false);
}

#[test]
fn same_link_owner_reinserted_after_parent_text_replacement_refetches_dependencies() {
    assert_same_link_owner_reinserted_refetches_dependencies(true);
}

#[test]
fn uppercase_html_style_created_with_namespace_is_retained_and_edited() {
    let fetcher = CountingFetcher::default();
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head></head><body><div id=card></div></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );
    runtime
        .eval(
            "var card = document.getElementById('card');\
             console.log(getComputedStyle(card).color);\
             var style = document.createElementNS('http://www.w3.org/1999/xhtml', 'STYLE');\
             style.textContent = '#card { color: red; }'; document.head.appendChild(style);\
             console.log(getComputedStyle(card).color);\
             style.textContent = '#card { color: blue; }';\
             console.log(getComputedStyle(card).color);",
        )
        .expect("uppercase HTML style insertion and edit");

    assert_eq!(
        runtime.host().borrow().console,
        vec!["rgb(0, 0, 0)", "rgb(255, 0, 0)", "rgb(0, 0, 255)"]
    );
    assert!(fetcher.calls().is_empty());
    let stylesheets = cssom.resource_set().unwrap().stylesheets;
    assert_eq!(stylesheets.len(), 1);
    assert!(stylesheets[0].text.contains("color: blue"));
}

#[test]
fn moving_an_image_into_and_out_of_shadow_dom_updates_resource_ledger() {
    let fetcher = CountingFetcher::default();
    fetcher.add("https://example.test/docs/poster.png", vec![1]);
    let mut runtime = Runtime::<BoaEngine>::new().expect("runtime");
    runtime.set_base_url(PAGE).unwrap();
    runtime.load_dom(&StaticDocument::parse(
        "<html><head></head><body><div id=host></div><img id=poster src=poster.png></body></html>",
    ));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher.clone(),
        PAGE,
        ResourceLimits::default(),
        device(),
    );
    let image_url = "https://example.test/docs/poster.png";
    assert_eq!(count(&fetcher.calls(), image_url), 1);
    assert_eq!(cssom.resource_set().unwrap().resources.len(), 1);

    runtime
        .eval(
            "var image = document.getElementById('poster');\
             var shadow = document.getElementById('host').attachShadow({mode: 'open'});\
             shadow.appendChild(image); document.styleSheets.length;",
        )
        .expect("move light-DOM image into shadow root");
    assert!(cssom.resource_set().unwrap().resources.is_empty());
    assert_eq!(count(&fetcher.calls(), image_url), 1);

    runtime
        .eval("document.body.appendChild(image); document.styleSheets.length;")
        .expect("move image back to light DOM");
    assert_eq!(count(&fetcher.calls(), image_url), 2);
    let resources = cssom.resource_set().unwrap();
    assert_eq!(resources.resources.len(), 1);
    assert_eq!(resources.resources[0].resolved_url, image_url);
}
