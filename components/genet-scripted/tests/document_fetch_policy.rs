#![cfg(feature = "livery")]

use std::sync::{Arc, Mutex};

use genet_host_api::{
    DocumentFetchKind, DocumentFetchResult, ResourceFetchResponse, ResourceFetcher,
    ResourceRequest, ResourceResponse,
};
use genet_scripted::LiveryScriptedDocument;
use script_engine_api::ScriptEngine;

const HTML: &str = r#"<body><iframe id=parser src="child"></iframe><script>
var loadedChildren = [];
var dynamic = document.createElement('iframe');
dynamic.src = 'dynamic';
dynamic.referrerPolicy = 'no-referrer';
document.body.appendChild(dynamic);
</script></body>"#;

#[derive(Clone, Default)]
struct Host {
    documents: Arc<Mutex<Vec<(DocumentFetchKind, ResourceRequest)>>>,
    legacy: Arc<Mutex<Vec<String>>>,
    fail_top: bool,
    fail_iframe: bool,
}

fn response(url: &str, bytes: &[u8]) -> ResourceFetchResponse {
    ResourceFetchResponse {
        final_url: url.into(),
        content_type: Some("text/html; charset=utf-8".into()),
        bytes: bytes.to_vec(),
        status: 200,
        status_text: "OK".into(),
        headers: vec![
            ("Referrer-Policy".into(), "unsafe-url".into()),
            (
                "Content-Security-Policy".into(),
                "upgrade-insecure-requests; block-all-mixed-content".into(),
            ),
        ],
        response_type: "basic".into(),
        redirected: false,
    }
}

impl ResourceFetcher for Host {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|r| r.bytes)
    }
    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        Some(response(&request.url, b"<p>legacy child</p>"))
    }
    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.legacy.lock().unwrap().push(url.into());
        Some(ResourceResponse::new(url, HTML.as_bytes().to_vec()).with_content_type("text/html"))
    }
    fn fetch_document(
        &self,
        request: &ResourceRequest,
        kind: DocumentFetchKind,
    ) -> DocumentFetchResult {
        self.documents.lock().unwrap().push((kind, request.clone()));
        if (kind == DocumentFetchKind::TopLevel && self.fail_top)
            || (kind == DocumentFetchKind::Iframe
                && (self.fail_iframe || request.url.ends_with("/blocked")))
        {
            return DocumentFetchResult::Failed;
        }
        let response = match kind {
            DocumentFetchKind::TopLevel => response("https://site.test/dir/page", HTML.as_bytes()),
            DocumentFetchKind::Iframe => response(
                &format!("{}?final", request.url),
                b"<script>parent.loadedChildren.push(location.href)</script>",
            ),
        };
        DocumentFetchResult::Response(response)
    }
}

fn product_document_policy<E: ScriptEngine>() {
    let host = Host::default();
    let mut doc =
        LiveryScriptedDocument::<E>::load(host.clone(), "https://site.test/start#focus").unwrap();
    for n in 0..3 {
        doc.pump(n as f64 * 16.0);
    }
    doc.evaluate(
        "console.log(loadedChildren.sort().join('|')); getComputedStyle(document.body).color;",
    )
    .unwrap();
    let requests = host.documents.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].0, DocumentFetchKind::TopLevel);
    assert!(requests[0].1.is_plain_get());
    for (path, policy) in [("child", "unsafe-url"), ("dynamic", "no-referrer")] {
        let (_, request) = requests
            .iter()
            .find(|(kind, r)| {
                *kind == DocumentFetchKind::Iframe
                    && r.url == format!("https://site.test/dir/{path}")
            })
            .unwrap();
        assert_eq!(request.mode, "navigate");
        assert_eq!(request.credentials, "include");
        assert_eq!(request.referrer, "https://site.test/dir/page#focus");
        assert_eq!(request.referrer_policy, policy);
        assert!(request.upgrade_insecure_requests && request.block_all_mixed_content);
    }
    assert!(
        doc.console()
            .iter()
            .any(|line| line
                == "https://site.test/dir/child?final|https://site.test/dir/dynamic?final"),
        "{:?}",
        doc.console()
    );
    assert!(
        host.legacy.lock().unwrap().is_empty(),
        "URL-only acquisition bypassed policy"
    );
}

#[test]
fn product_document_policy_boa() {
    product_document_policy::<script_engine_boa::BoaEngine>();
}
#[cfg(feature = "scripted-nova")]
#[test]
fn product_document_policy_vano() {
    product_document_policy::<script_engine_nova::NovaEngine>();
}

fn post_insertion_iframe_policy<E: ScriptEngine>() {
    let host = Host::default();
    let mut doc =
        LiveryScriptedDocument::<E>::load(host.clone(), "https://site.test/start#focus").unwrap();
    doc.pump(0.0);
    host.documents.lock().unwrap().clear();
    doc.evaluate(
        r#"var later = document.createElement('iframe');
        document.body.appendChild(later);
        var previousWindow = later.contentWindow;
        later.setAttribute('referrerpolicy', 'origin');
        later.setAttribute('src', 'superseded');
        later.setAttribute('src', 'later');"#,
    )
    .unwrap();
    for n in 1..5 {
        doc.pump(n as f64 * 16.0);
    }

    let requests = host.documents.lock().unwrap();
    assert_eq!(
        requests.len(),
        1,
        "the final src uses exactly one typed fetch"
    );
    let (kind, request) = &requests[0];
    assert_eq!(*kind, DocumentFetchKind::Iframe);
    assert_eq!(request.url, "https://site.test/dir/later");
    assert_eq!(request.mode, "navigate");
    assert_eq!(request.credentials, "include");
    assert_eq!(request.referrer, "https://site.test/dir/page#focus");
    assert_eq!(request.referrer_policy, "origin");
    assert!(request.upgrade_insecure_requests && request.block_all_mixed_content);
    drop(requests);

    doc.evaluate(
        "console.log((previousWindow === later.contentWindow) + ' ' + later.contentWindow.location.href)",
    )
    .unwrap();
    assert!(
        doc.console()
            .iter()
            .any(|line| line == "true https://site.test/dir/later?final"),
        "{:?}",
        doc.console()
    );
    assert!(host.legacy.lock().unwrap().is_empty());
}

#[test]
fn post_insertion_iframe_policy_boa() {
    post_insertion_iframe_policy::<script_engine_boa::BoaEngine>();
}

#[cfg(feature = "scripted-nova")]
#[test]
fn post_insertion_iframe_policy_vano() {
    post_insertion_iframe_policy::<script_engine_nova::NovaEngine>();
}

#[test]
fn post_insertion_iframe_drops_older_src_when_latest_fails() {
    let host = Host::default();
    let mut doc = LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
        host.clone(),
        "https://site.test/start",
    )
    .unwrap();
    doc.pump(0.0);
    host.documents.lock().unwrap().clear();
    doc.evaluate(
        r#"var later = document.createElement('iframe');
        document.body.appendChild(later);
        later.src = 'older';
        later.src = 'blocked';"#,
    )
    .unwrap();
    for n in 1..4 {
        doc.pump(n as f64 * 16.0);
    }

    let requests = host.documents.lock().unwrap();
    assert_eq!(requests.len(), 1, "the older src cannot resume after failure");
    assert_eq!(requests[0].0, DocumentFetchKind::Iframe);
    assert_eq!(requests[0].1.url, "https://site.test/dir/blocked");
    drop(requests);
    assert!(host.legacy.lock().unwrap().is_empty());
    doc.evaluate("console.log(later.contentWindow.location.href)")
        .unwrap();
    assert!(doc.console().iter().any(|line| line == "about:blank"));
}

#[test]
fn post_insertion_iframe_respects_empty_srcdoc() {
    let host = Host::default();
    let mut doc = LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
        host.clone(),
        "https://site.test/start",
    )
    .unwrap();
    doc.evaluate(
        r#"var inlineFrame = document.createElement('iframe');
        inlineFrame.setAttribute('srcdoc', '');
        document.body.appendChild(inlineFrame);
        inlineFrame.setAttribute('src', 'ignored');"#,
    )
    .unwrap();
    for n in 0..4 {
        doc.pump(n as f64 * 16.0);
    }
    doc.evaluate("console.log(inlineFrame.contentWindow.location.href)")
        .unwrap();
    assert!(doc.console().iter().any(|line| line == "about:srcdoc"));
    assert_eq!(host.documents.lock().unwrap().len(), 3);
    assert!(host.legacy.lock().unwrap().is_empty());
}

#[test]
fn post_insertion_iframe_preserves_opaque_sandbox_origin() {
    let host = Host::default();
    let mut doc = LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
        host.clone(),
        "https://site.test/start",
    )
    .unwrap();
    doc.pump(0.0);
    host.documents.lock().unwrap().clear();
    doc.evaluate(
        r#"var sandboxed = document.createElement('iframe');
        sandboxed.setAttribute('sandbox', 'allow-scripts');
        document.body.appendChild(sandboxed);
        sandboxed.src = 'sandboxed';"#,
    )
    .unwrap();
    for n in 1..4 {
        doc.pump(n as f64 * 16.0);
    }
    doc.evaluate("console.log(sandboxed.contentDocument === null)")
        .unwrap();
    assert!(doc.console().iter().any(|line| line == "true"));
    let requests = host.documents.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].1.url, "https://site.test/dir/sandboxed");
}

#[test]
fn failed_top_document_does_not_retry_legacy_loader() {
    let host = Host {
        fail_top: true,
        ..Host::default()
    };
    assert!(
        LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
            host.clone(),
            "https://site.test/start"
        )
        .is_err()
    );
    assert!(host.legacy.lock().unwrap().is_empty());
}

#[test]
fn closed_bridge_claims_document_failure_without_host_call() {
    let host = Host::default();
    let bridge =
        genet_scripted::ScriptResourceBridge::new(host.clone(), genet_scripted::ScriptWake::new());
    bridge.close();
    assert!(matches!(
        ResourceFetcher::fetch_document(
            &bridge,
            &ResourceRequest::plain_get("https://site.test/"),
            DocumentFetchKind::TopLevel
        ),
        DocumentFetchResult::Failed
    ));
    assert!(host.documents.lock().unwrap().is_empty());
}

#[test]
fn failed_iframe_does_not_retry_legacy_loader() {
    let host = Host {
        fail_iframe: true,
        ..Host::default()
    };
    let mut doc = LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
        host.clone(),
        "https://site.test/start",
    )
    .unwrap();
    doc.pump(0.0);
    assert_eq!(host.documents.lock().unwrap().len(), 3);
    assert!(host.legacy.lock().unwrap().is_empty());
    doc.evaluate("console.log(loadedChildren.length)").unwrap();
    assert!(doc.console().iter().any(|line| line == "0"));
}

#[test]
fn legacy_document_host_keeps_its_url_loader() {
    struct Legacy {
        urls: Arc<Mutex<Vec<String>>>,
        requests: Arc<Mutex<Vec<ResourceRequest>>>,
    }
    impl ResourceFetcher for Legacy {
        fn max_concurrent_fetches(&self) -> usize {
            0
        }
        fn fetch(&self, url: &str) -> Option<Vec<u8>> {
            self.urls.lock().unwrap().push(url.into());
            Some(b"<p>legacy document</p><iframe src=child></iframe>".to_vec())
        }
        fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
            self.requests.lock().unwrap().push(request.clone());
            Some(response(
                &request.url,
                b"<script>parent.legacyChild = true</script>",
            ))
        }
    }
    let urls = Arc::new(Mutex::new(Vec::new()));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let host = Legacy {
        urls: urls.clone(),
        requests: requests.clone(),
    };
    let mut doc = LiveryScriptedDocument::<script_engine_boa::BoaEngine>::load(
        host,
        "https://site.test/legacy",
    )
    .unwrap();
    doc.pump(0.0);
    doc.evaluate("console.log(legacyChild); getComputedStyle(document.body).color;")
        .unwrap();
    assert!(doc.dom_snapshot().contains("legacy document"));
    assert!(doc.console().iter().any(|line| line == "true"));
    assert_eq!(
        urls.lock().unwrap().as_slice(),
        ["https://site.test/legacy"]
    );
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].url, "https://site.test/child");
    assert!(requests[0].is_plain_get());
}
