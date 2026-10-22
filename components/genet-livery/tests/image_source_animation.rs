#![cfg(feature = "image-decode")]

use genet_livery::{Device, LiveryDocument, LiveryPaintList, StyleSet};
use genet_static_dom::StaticDocument;
use paint_list_api::{PaintCmd, PaintList};

const HTML: &str = "<html><body><div class=card></div></body></html>";
const CSS: &str = r#"
    .card {
        position: absolute;
        left: 0;
        top: 0;
        width: 10px;
        height: 10px;
        background-repeat: no-repeat;
        animation: icon 1000ms linear -500ms;
        animation-play-state: paused;
    }
    @keyframes icon {
        from { background-image: url(icon.png); }
        to { background-image: url(icon.png); }
    }
"#;

fn png(color: [u8; 4]) -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(2, 3, image::Rgba(color));
    let mut bytes = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encode fixture PNG");
    bytes
}

fn painted_pixel(frame: &LiveryPaintList) -> [u8; 4] {
    let images = frame
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawImage(image) => Some(image),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(images.len(), 1, "animated background paints once");
    assert_eq!(
        images[0].placement.bounds.size(),
        paint_list_api::LayoutSize::new(2.0, 3.0)
    );
    let resource = frame
        .images()
        .iter()
        .find(|resource| resource.key == images[0].image_key)
        .expect("animation paint command has decoded bytes");
    assert_eq!((resource.width, resource.height), (2, 3));
    resource.data[..4].try_into().expect("first RGBA pixel")
}

#[test]
fn source_less_keyframe_url_follows_document_base_set_after_parse() {
    // Host-supplied CSS starts without a source URL. Its keyframes must gain
    // the document base when the host assigns one after parsing the sheet.
    let mut styles = StyleSet::cambium(&[CSS]);
    assert_eq!(styles.document_base_url(), None);
    styles.set_document_base_url(Some("https://site.test/one/page.html"));
    let mut first = LiveryDocument::new(
        StaticDocument::parse(HTML),
        styles.clone(),
        Device::screen(100.0, 50.0),
    );
    first.set_image_resource("https://site.test/one/icon.png", png([255, 0, 0, 255]));
    let frame = first.frame(100, 50).expect("first retained paint frame");
    assert_eq!(painted_pixel(&frame), [255, 0, 0, 255]);

    // The same parsed source can be used for a later navigation with a new
    // document base. A stale keyframe source would paint the old image.
    styles.set_document_base_url(Some("https://site.test/two/page.html"));
    let mut second = LiveryDocument::new(
        StaticDocument::parse(HTML),
        styles,
        Device::screen(100.0, 50.0),
    );
    second.set_image_resource("https://site.test/two/icon.png", png([0, 0, 255, 255]));
    let frame = second.frame(100, 50).expect("second retained paint frame");
    assert_eq!(painted_pixel(&frame), [0, 0, 255, 255]);
}
