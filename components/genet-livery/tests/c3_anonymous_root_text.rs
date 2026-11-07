// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! C3 regression: text in a box that no element styled reaches emission with
//! the initial `color`, which is the contextual system color `CanvasText`.
//!
//! A host-built DOM may hang text, or several elements, straight off the
//! document. The anonymous box around that text has no parent style to
//! inherit, so layout used `ComputedValues::default()`. Text shaping resolves
//! the run color with `paint::resolve_color`, which expects a numeric color
//! ("C3 resolves every paint color before emission") and panicked on
//! `System(CanvasText)`. Graphshell web logged that message and then died with
//! `RuntimeError: unreachable` the moment its Graph controls toggled in. The
//! initial style an unstyled box takes now resolves its colors under the
//! document's own palette and scheme, as the cascade does for a root element.

use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};
use livery::{
    media::SystemPalette,
    values::{ColorScheme, SystemColor},
};
use paint_list_api::{ColorF, PaintCmd, PaintList};

fn channel(value: u8) -> f32 {
    f32::from(value) / 255.0
}

fn palette() -> SystemPalette {
    let mut palette = SystemPalette::default();
    palette.set(
        ColorScheme::Light,
        SystemColor::CanvasText,
        "#102030".parse().unwrap(),
    );
    palette.set(
        ColorScheme::Dark,
        SystemColor::CanvasText,
        "#d0e0f0".parse().unwrap(),
    );
    palette
}

fn element(dom: &mut ScriptedDom, name: &str) -> <ScriptedDom as LayoutDom>::NodeId {
    dom.create_element(QualName::new(
        None,
        Namespace::from(""),
        LocalName::from(name),
    ))
}

fn text_colors(dom: ScriptedDom, scheme: ColorScheme) -> Vec<ColorF> {
    let mut device = Device::screen(200.0, 100.0);
    device.set_system_palette(palette());
    device.set_preferred_color_scheme(scheme);
    let mut document = LiveryDocument::new(dom, StyleSet::cambium(&[""]), device);
    document
        .frame(200, 100)
        .expect("a frame whose only text sits in an anonymous root box")
        .commands()
        .iter()
        .filter_map(|command| match command {
            PaintCmd::DrawText(run) => Some(run.color),
            _ => None,
        })
        .collect()
}

fn bare_text() -> ScriptedDom {
    let mut dom = ScriptedDom::new();
    let text = dom.create_text("words");
    let root = dom.document();
    dom.append_child(root, text);
    dom
}

/// A document whose only content is a text node lays out without panicking,
/// whether or not the host paints a lone anonymous inline.
#[test]
fn text_directly_under_the_document_does_not_reach_emission_unresolved() {
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let _ = text_colors(bare_text(), scheme);
    }
}

/// Text beside unwrapped root elements sits in an anonymous box with no parent
/// style. Its foreground is the host palette's `CanvasText` for the scheme in
/// force, light or dark.
#[test]
fn text_beside_unwrapped_root_elements_takes_the_host_palette_foreground() {
    let light = ColorF::new(channel(0x10), channel(0x20), channel(0x30), 1.0);
    let dark = ColorF::new(channel(0xd0), channel(0xe0), channel(0xf0), 1.0);
    for (scheme, expected) in [(ColorScheme::Light, light), (ColorScheme::Dark, dark)] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let before = dom.create_text("before");
        let block = element(&mut dom, "div");
        let inside = dom.create_text("inside");
        let after = dom.create_text("after");
        dom.append_child(block, inside);
        for child in [before, block, after] {
            dom.append_child(root, child);
        }
        let colors = text_colors(dom, scheme);
        assert!(!colors.is_empty(), "{scheme:?} painted no text");
        assert!(
            colors.iter().all(|color| *color == expected),
            "{scheme:?}: {colors:?}"
        );
    }
}
