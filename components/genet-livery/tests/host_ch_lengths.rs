// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The caller-owned text/font ledger entry used by Cambium's OwnedLayout.

use genet_livery::{
    Device, InteractionStates, StylePlane, StyleSet, TextSystem, ViewportSizes,
    layout_with_text_system, resolve_styles,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::values::{FontFeatureSettings, Length, LengthPercentage, Size};

const AHEM: &[u8] = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf");

struct HostFixture {
    dom: StaticDocument,
    styles: StylePlane<<StaticDocument as LayoutDom>::NodeId>,
    viewport_width: f32,
}

impl HostFixture {
    fn new(font_size: f32, viewport_width: f32) -> Self {
        let dom = StaticDocument::parse(
            "<html><body><div class=bare></div><div class=mixed></div>\
         <div class=container><div class=child></div></div>\
         <div class=zeros>000000000000000000000000000000000000000000000000000000000000000000000000</div>\
         </body></html>",
        );
        let css = format!(
            "html, body {{ margin: 0; }}\
         div {{ font-family: host-ch; font-size: {font_size}px; }}\
         .bare {{ width: 72ch; }}\
         .mixed {{ width: calc(72ch + 32px + 1vw); }}\
         .container {{ width: 72ch; container-type: inline-size; }}\
         .child {{ width: 50cqi; }}\
         .zeros {{ display: inline-block; white-space: nowrap; }}"
        );
        let device = Device::screen(viewport_width, 600.0);
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[&css]),
            &device,
            &InteractionStates::default(),
        );
        Self {
            dom,
            styles,
            viewport_width,
        }
    }

    fn widths(&self, text: &mut TextSystem) -> Vec<f32> {
        let Self {
            dom,
            styles,
            viewport_width,
        } = self;
        let viewport_width = *viewport_width;
        let bare = dom.first_with_class(dom.document(), "bare").unwrap();
        let authored_width = styles.get(bare).unwrap().width;
        assert_eq!(
            authored_width,
            Size::Value(LengthPercentage::Length(Length::ch(72.0)))
        );
        let (_, fragments) = layout_with_text_system(
            dom,
            styles,
            viewport_width,
            600.0,
            ViewportSizes::uniform(viewport_width, 600.0),
            text,
            &Default::default(),
        )
        .expect("host-owned layout");
        assert_eq!(
            styles.get(bare).unwrap().width,
            authored_width,
            "layout must leave the caller's unresolved cascade available for future font ledgers"
        );
        ["bare", "mixed", "container", "child", "zeros"]
            .map(|class| {
                let node = dom.first_with_class(dom.document(), class).unwrap();
                fragments.get(node).unwrap().width
            })
            .to_vec()
    }
}

fn assert_widths(actual: &[f32], expected: &[f32]) {
    for (actual, expected) in actual.iter().zip(expected) {
        assert!((actual - expected).abs() <= 0.5, "{actual} != {expected}");
    }
}

#[test]
fn host_layout_resolves_ch_and_mixed_calc_from_registered_zero_advance() {
    let mut text = TextSystem::new();
    text.register_font_face_bytes(AHEM.to_vec(), "host-ch", &FontFeatureSettings::Normal);
    // Ahem's zero is one em wide; the old host path used half an em for bare
    // ch and dropped the ch contribution from calc entirely.
    assert_widths(
        &HostFixture::new(20.0, 2000.0).widths(&mut text),
        &[1440.0, 1492.0, 1440.0, 720.0, 1440.0],
    );
}

#[test]
fn host_layout_refreshes_ch_after_font_registration_font_size_and_viewport_changes() {
    let mut text = TextSystem::new();
    let fixture = HostFixture::new(20.0, 2000.0);
    let _ = fixture.widths(&mut text);
    text.register_font_face_bytes(AHEM.to_vec(), "host-ch", &FontFeatureSettings::Normal);
    assert_widths(
        &fixture.widths(&mut text),
        &[1440.0, 1492.0, 1440.0, 720.0, 1440.0],
    );
    assert_widths(
        &HostFixture::new(12.0, 1800.0).widths(&mut text),
        &[864.0, 914.0, 864.0, 432.0, 864.0],
    );
}
