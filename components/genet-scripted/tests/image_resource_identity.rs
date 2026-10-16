#![cfg(all(feature = "livery", feature = "image-decode"))]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use base64::Engine as _;
use genet_document_resources::{ResourceFetcher, ResourceKind, ResourceLimits, ResourceResponse};
use genet_livery::{Device, LiveryPaintList};
use genet_scripted::LiveryCssom;
use genet_static_dom::StaticDocument;
use paint_list_api::{ImageKey, PaintCmd, PaintList};
use script_engine_boa::BoaEngine;
use script_runtime_api::Runtime;

const PAGE: &str = "https://example.test/docs/index.html";

// Opaque 2x3 RGBA PNGs. Their different pixels make a URL identity collision
// visible at the paint output, including the replaced-image path.
const RED: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAYAAAC56t6BAAAAEUlEQVR4nGP4z8DwH4QZMBgAoXkL9U3EmgcAAAAASUVORK5CYII=";
const BLUE: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAYAAAC56t6BAAAAEElEQVR4nGNgYPj/H4LRGQCVhQv15UfOvQAAAABJRU5ErkJggg==";
const GREEN: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAADCAYAAAC56t6BAAAADklEQVR4nGNg+A+FGAwAm38L9SgUsAQAAAAASUVORK5CYII=";

#[derive(Clone, Default)]
struct FixtureFetcher(Rc<RefCell<HashMap<String, ResourceResponse>>>);

impl FixtureFetcher {
    fn respond(&self, requested: &str, final_url: &str, bytes: impl Into<Vec<u8>>) {
        self.0.borrow_mut().insert(
            requested.to_owned(),
            ResourceResponse::new(final_url, bytes.into()),
        );
    }

    fn png(&self, url: &str, encoded: &str) {
        self.respond(
            url,
            url,
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .expect("fixture PNG"),
        );
    }
}

impl ResourceFetcher for FixtureFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.0.borrow().get(url).cloned()
    }
}

fn runtime(html: &str, fetcher: FixtureFetcher) -> (Runtime<BoaEngine>, LiveryCssom) {
    let mut runtime = Runtime::<BoaEngine>::new().expect("Boa runtime");
    runtime.set_base_url(PAGE).expect("document URL");
    runtime.load_dom(&StaticDocument::parse(html));
    let cssom = LiveryCssom::install_live(
        &mut runtime,
        fetcher,
        PAGE,
        ResourceLimits::default(),
        Device::screen(100.0, 50.0),
    );
    (runtime, cssom)
}

fn painted_image_at(
    frame: &LiveryPaintList,
    left: f32,
) -> ([u8; 4], (u32, u32), ImageKey, (f32, f32)) {
    let commands = frame
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawImage(image) if image.placement.bounds.min.x == left => Some(image),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(commands.len(), 1, "one image should paint at x={left}");
    let resource = frame
        .images()
        .iter()
        .find(|resource| resource.key == commands[0].image_key)
        .expect("paint command has decoded pixels");
    (
        resource.data[..4].try_into().expect("first RGBA pixel"),
        (resource.width, resource.height),
        commands[0].image_key,
        (
            commands[0].placement.bounds.size().width,
            commands[0].placement.bounds.size().height,
        ),
    )
}

fn assert_image_urls(cssom: &LiveryCssom, expected: &[&str]) {
    let resources = cssom.resource_set().expect("live resource ledger");
    let mut actual = resources
        .resources
        .iter()
        .filter(|resource| {
            resource.kind == ResourceKind::Image && resource.resolved_url.ends_with(".png")
        })
        .map(|resource| resource.resolved_url.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

#[test]
fn linked_and_redirected_imported_sheets_keep_equal_authored_urls_distinct() {
    let fetcher = FixtureFetcher::default();
    fetcher.respond(
        "https://example.test/docs/a.css",
        "https://assets.test/a/theme.css",
        b"html { --image:url(icon.png) } #a { position:absolute; left:0; top:0; width:10px; height:10px; background-image:url(icon.png); background-repeat:no-repeat }".to_vec(),
    );
    fetcher.respond(
        "https://example.test/docs/b.css",
        "https://assets.test/b/root.css",
        b"@import url(child.css);".to_vec(),
    );
    fetcher.respond(
        "https://assets.test/b/child.css",
        "https://cdn.test/b/imported.css",
        b"#b { position:absolute; left:20px; top:0; width:10px; height:10px; background-image:url(icon.png); background-image:var(--image); background-repeat:no-repeat }".to_vec(),
    );
    fetcher.png("https://assets.test/a/icon.png", RED);
    fetcher.png("https://cdn.test/b/icon.png", BLUE);
    fetcher.respond(
        "https://example.test/docs/c.css",
        "https://assets.test/c/root.css",
        b"@import url(child.css);".to_vec(),
    );
    fetcher.respond(
        "https://assets.test/c/child.css",
        "https://cdn.test/c/imported.css",
        b"#b { position:absolute; left:20px; top:0; width:10px; height:10px; background-image:url(icon.png); background-image:var(--image); background-repeat:no-repeat }".to_vec(),
    );
    fetcher.png("https://cdn.test/c/icon.png", GREEN);

    let (mut runtime, cssom) = runtime(
        "<html><head><link rel=stylesheet href=a.css><link id=second rel=stylesheet href=b.css></head>\
         <body><div id=a></div><div id=b></div></body></html>",
        fetcher,
    );
    runtime
        .eval(
            "console.log(getComputedStyle(document.getElementById('a')).backgroundImage);\
             console.log(getComputedStyle(document.getElementById('b')).backgroundImage);\
             console.log(document.styleSheets[0].cssRules[0].cssText);\
             console.log(document.styleSheets[1].cssRules[0].styleSheet.cssRules[0].cssText);",
        )
        .expect("computed images and authored CSSOM rules");
    {
        let console = runtime.host().borrow().console.clone();
        assert!(
            console[0].contains("https://assets.test/a/icon.png"),
            "{console:?}"
        );
        assert!(
            console[1].contains("https://cdn.test/b/icon.png"),
            "{console:?}"
        );
        assert!(console[2].contains("url(icon.png)"), "{console:?}");
        assert!(console[3].contains("url(icon.png)"), "{console:?}");
    }
    assert_image_urls(
        &cssom,
        &[
            "https://assets.test/a/icon.png",
            "https://cdn.test/b/icon.png",
        ],
    );
    let frame = cssom.frame(&mut runtime, 100, 50).expect("initial paint");
    let a = painted_image_at(&frame, 0.0);
    let b = painted_image_at(&frame, 20.0);
    assert_eq!(a.0, [255, 0, 0, 255]);
    assert_eq!(b.0, [0, 0, 255, 255]);
    assert_ne!(
        a.2, b.2,
        "different stylesheet bases need different paint keys"
    );

    runtime
        .eval(
            "document.getElementById('second').setAttribute('href', 'c.css');\
             console.log(getComputedStyle(document.getElementById('b')).backgroundImage);\
             console.log(document.styleSheets[1].cssRules[0].styleSheet.cssRules[0].cssText);",
        )
        .expect("replace redirected import source");
    {
        let console = runtime.host().borrow().console.clone();
        assert!(
            console[4].contains("https://cdn.test/c/icon.png"),
            "{console:?}"
        );
        assert!(console[5].contains("url(icon.png)"), "{console:?}");
    }
    assert_image_urls(
        &cssom,
        &[
            "https://assets.test/a/icon.png",
            "https://cdn.test/c/icon.png",
        ],
    );
    let frame = cssom
        .frame(&mut runtime, 100, 50)
        .expect("replacement paint");
    assert_eq!(painted_image_at(&frame, 0.0).0, [255, 0, 0, 255]);
    assert_eq!(painted_image_at(&frame, 20.0).0, [0, 255, 0, 255]);
}

#[test]
fn document_base_change_updates_inline_background_and_img_intrinsic_paint() {
    let fetcher = FixtureFetcher::default();
    fetcher.png("https://example.test/docs/icon.png", RED);
    fetcher.png("https://example.test/other/icon.png", BLUE);
    let (mut runtime, cssom) = runtime(
        "<html><head><base href='/docs/'></head><body>\
         <div id=tile style='position:absolute;left:0;top:0;width:10px;height:10px;background-image:url(icon.png);background-repeat:no-repeat'></div>\
         <img id=poster src=icon.png style='position:absolute;left:40px;top:0'>\
         </body></html>",
        fetcher,
    );
    runtime
        .eval(
            "console.log(getComputedStyle(document.getElementById('tile')).backgroundImage);\
             console.log(document.getElementById('tile').style.backgroundImage);\
             console.log(document.getElementById('poster').getAttribute('src'));",
        )
        .expect("document-relative image identities");
    {
        let console = runtime.host().borrow().console.clone();
        assert!(
            console[0].contains("https://example.test/docs/icon.png"),
            "{console:?}"
        );
        assert!(console[1].contains("icon.png") && !console[1].contains("https:"));
        assert_eq!(console[2], "icon.png");
    }
    assert_image_urls(&cssom, &["https://example.test/docs/icon.png"]);
    let frame = cssom.frame(&mut runtime, 100, 50).expect("initial paint");
    assert_eq!(painted_image_at(&frame, 0.0).0, [255, 0, 0, 255]);
    let poster = painted_image_at(&frame, 40.0);
    assert_eq!((poster.0, poster.1), ([255, 0, 0, 255], (2, 3)));
    assert_eq!(poster.3, (2.0, 3.0), "intrinsic img placement");

    runtime
        .eval(
            "document.querySelector('base').setAttribute('href', '/other/');\
             console.log(getComputedStyle(document.getElementById('tile')).backgroundImage);\
             console.log(document.getElementById('tile').style.backgroundImage);\
             console.log(document.getElementById('poster').getAttribute('src'));",
        )
        .expect("document base mutation");
    {
        let console = runtime.host().borrow().console.clone();
        assert!(
            console[3].contains("https://example.test/other/icon.png"),
            "{console:?}"
        );
        assert!(console[4].contains("icon.png") && !console[4].contains("https:"));
        assert_eq!(console[5], "icon.png");
    }
    assert_image_urls(&cssom, &["https://example.test/other/icon.png"]);
    let frame = cssom.frame(&mut runtime, 100, 50).expect("new base paint");
    assert_eq!(painted_image_at(&frame, 0.0).0, [0, 0, 255, 255]);
    let poster = painted_image_at(&frame, 40.0);
    assert_eq!((poster.0, poster.1), ([0, 0, 255, 255], (2, 3)));
    assert_eq!(
        poster.3,
        (2.0, 3.0),
        "intrinsic img placement after base change"
    );
}
