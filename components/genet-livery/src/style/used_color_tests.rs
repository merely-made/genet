// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
struct CountedId(usize, Arc<AtomicUsize>);

impl Clone for CountedId {
    fn clone(&self) -> Self {
        self.1.fetch_add(1, Ordering::Relaxed);
        Self(self.0, self.1.clone())
    }
}
impl PartialEq for CountedId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for CountedId {}
impl Hash for CountedId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[test]
fn repeated_used_color_paint_does_not_clone_the_retained_plane() {
    let clones = Arc::new(AtomicUsize::new(0));
    let mut plane = StylePlane::default();
    plane
        .values
        .insert(CountedId(1, clones.clone()), ComputedValues::default());
    let first = plane.with_used_colors();
    assert_eq!(first.len(), 1);
    assert!(
        clones.swap(0, Ordering::Relaxed) > 0,
        "first paint creates its numeric view"
    );
    let second = plane.with_used_colors();
    assert_eq!(second.len(), 1);
    assert_eq!(
        clones.load(Ordering::Relaxed),
        0,
        "unchanged paint must borrow the existing numeric view"
    );
}

fn colored_plane() -> StylePlane<usize> {
    let mut plane = StylePlane::default();
    let mut values = ComputedValues::default();
    values.color = "#2468ac".parse().unwrap();
    values.background_color = ComputedColor::CURRENT_COLOR;
    plane.values.insert(1, values.clone());
    plane.values.insert(2, values.clone());
    plane.generated.insert(
        (1, buckram::PseudoElement::Before),
        (values, "before".into()),
    );
    plane
}

#[test]
fn used_view_preserves_contextual_source_and_shared_clones_refresh_independently() {
    let plane = colored_plane();
    let preserved = plane.clone();
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().background_color,
        "#2468ac".parse::<ComputedColor>().unwrap()
    );
    assert_eq!(
        plane.get(1).unwrap().background_color,
        ComputedColor::CURRENT_COLOR
    );
    assert_eq!(
        plane, preserved,
        "derived view is outside semantic equality"
    );
    let mut changed = plane.clone();
    assert!(
        std::ptr::eq(plane.with_used_colors(), changed.with_used_colors()),
        "an unchanged clone shares immutable paint data"
    );
    changed.get_mut(1).unwrap().color = "#ab6521".parse().unwrap();
    assert_eq!(
        changed.with_used_colors().get(1).unwrap().background_color,
        "#ab6521".parse::<ComputedColor>().unwrap()
    );
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().background_color,
        "#2468ac".parse::<ComputedColor>().unwrap()
    );
}

#[test]
fn removal_and_retention_drop_cached_elements_and_generated_content() {
    let mut plane = colored_plane();
    plane.with_used_colors();
    // Keep the old allocation alive: an address-only check could accidentally
    // pass if a discarded view was rebuilt at the allocator's same address.
    let view = plane.used_colors.get().unwrap().clone();
    plane.retain(|_| true);
    plane.with_used_colors();
    assert!(
        Arc::ptr_eq(&view, plane.used_colors.get().unwrap()),
        "no-op liveness checks retain the view"
    );
    assert_eq!(
        plane
            .with_used_colors()
            .generated_text(1, buckram::PseudoElement::Before),
        Some("before")
    );
    plane.remove(1);
    assert!(plane.with_used_colors().get(1).is_none());
    assert_eq!(
        plane
            .with_used_colors()
            .generated_text(1, buckram::PseudoElement::Before),
        None
    );
    plane.retain(|_| false);
    assert!(plane.with_used_colors().is_empty());
}

#[test]
fn relative_and_ch_length_resolution_refresh_non_color_paint_fields() {
    let mut plane = colored_plane();
    plane.get_mut(1).unwrap().width = "10vw".parse().unwrap();
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().width,
        "10vw".parse().unwrap()
    );
    let viewport = Device::screen(800.0, 600.0).viewport_sizes;
    assert!(plane.resolve_relative_lengths(
        1,
        livery::values::RelativeLengthEnvironment::viewport(viewport)
    ));
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().width,
        "80px".parse().unwrap()
    );
    plane.get_mut(1).unwrap().width = "2ch".parse().unwrap();
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().width,
        "2ch".parse().unwrap()
    );
    plane.resolve_ch_lengths(&mut crate::TextSystem::new(), viewport);
    assert_ne!(plane.get(1).unwrap().width, "2ch".parse().unwrap());
    assert_eq!(
        plane.with_used_colors().get(1).unwrap().width,
        plane.get(1).unwrap().width
    );
}

fn target<D: LayoutDom>(dom: &D, plane: &StylePlane<D::NodeId>) -> D::NodeId
where
    D::NodeId: Copy,
{
    *plane
        .values
        .keys()
        .find(|&&id| {
            dom.attribute(id, &Namespace::default(), &LocalName::from("id")) == Some("target")
        })
        .unwrap()
}

#[test]
fn incremental_cascade_refreshes_colors_and_generated_styles() {
    let dom = genet_static_dom::StaticDocument::parse("<div id=target>content</div>");
    let before = StyleSet::cambium(&[
        "#target { color: #2468ac; background-color:currentcolor } #target::before { content:'first'; color: #2468ac }",
    ]);
    let after = StyleSet::cambium(&[
        "#target { color: #ab6521; background-color:currentcolor } #target::before { content:'next'; color:#ab6521 }",
    ]);
    let device = Device::screen(320.0, 200.0);
    let states = InteractionStates::default();
    let mut plane = resolve_styles(&dom, &before, &device, &states);
    let id = target(&dom, &plane);
    assert_eq!(
        plane
            .with_used_colors()
            .generated_text(id, buckram::PseudoElement::Before),
        Some("first")
    );
    let tree = SelectorTree::new(&dom, &states);
    resolve_subtree_with_containers(
        &tree,
        &after,
        &device,
        dom.document(),
        None,
        None,
        TreeCounts::Deferred,
        &mut plane,
        &PresentationalHints::from_html_dom(&dom),
        None,
        &tree_scopes(&dom, &after),
    );
    let used = plane.with_used_colors();
    assert_eq!(
        used.get(id).unwrap().background_color,
        "#ab6521".parse::<ComputedColor>().unwrap()
    );
    assert_eq!(
        used.generated_text(id, buckram::PseudoElement::Before),
        Some("next")
    );
    assert_eq!(
        used.generated_style(id, buckram::PseudoElement::Before)
            .unwrap()
            .color,
        "#ab6521".parse::<ComputedColor>().unwrap()
    );
    assert_eq!(
        plane.get(id).unwrap().background_color,
        ComputedColor::CURRENT_COLOR
    );
}

#[test]
fn counter_resolution_refreshes_cached_generated_strings() {
    let dom = genet_static_dom::StaticDocument::parse("<div id=target></div>");
    let sheet = StyleSet::cambium(&[
        "#target { counter-reset:n 7 } #target::before { content:counter(n) }",
    ]);
    let mut plane = resolve_styles(
        &dom,
        &sheet,
        &Device::screen(320.0, 200.0),
        &InteractionStates::default(),
    );
    let id = target(&dom, &plane);
    plane
        .generated
        .get_mut(&(id, buckram::PseudoElement::Before))
        .unwrap()
        .1 = "pending counter pass".into();
    assert_eq!(
        plane
            .with_used_colors()
            .generated_text(id, buckram::PseudoElement::Before),
        Some("pending counter pass")
    );
    resolve_counters(&dom, &mut plane);
    assert_eq!(
        plane
            .with_used_colors()
            .generated_text(id, buckram::PseudoElement::Before),
        Some("7")
    );
}

#[test]
fn device_palette_and_scheme_replacement_refresh_used_context() {
    use livery::{media::SystemPalette, values::ColorScheme};
    let dom = genet_static_dom::StaticDocument::parse("<div id=target></div>");
    let sheet = StyleSet::cambium(&[
        "#target { color-scheme:light dark; color:CanvasText; background-color:currentcolor }",
    ]);
    let states = InteractionStates::default();
    let mut device = Device::screen(320.0, 200.0);
    let mut palette = SystemPalette::default();
    palette.set(
        ColorScheme::Light,
        SystemColor::CanvasText,
        "#2468ac".parse().unwrap(),
    );
    palette.set(
        ColorScheme::Dark,
        SystemColor::CanvasText,
        "#ab6521".parse().unwrap(),
    );
    device.set_system_palette(palette);
    device.set_preferred_color_scheme(ColorScheme::Light);
    let mut retained = crate::IncrementalStyle::new();
    retained.update(&dom, &sheet, &device, &states, &[]);
    let id = target(&dom, retained.styles());
    assert_eq!(
        retained
            .styles()
            .with_used_colors()
            .get(id)
            .unwrap()
            .background_color,
        "#2468ac".parse::<ComputedColor>().unwrap()
    );
    device.set_preferred_color_scheme(ColorScheme::Dark);
    assert!(
        retained
            .update(&dom, &sheet, &device, &states, &[])
            .device_invalidated
    );
    assert_eq!(
        retained
            .styles()
            .with_used_colors()
            .get(id)
            .unwrap()
            .background_color,
        "#ab6521".parse::<ComputedColor>().unwrap()
    );
    palette.set(
        ColorScheme::Dark,
        SystemColor::CanvasText,
        "#19af72".parse().unwrap(),
    );
    device.set_system_palette(palette);
    assert!(
        retained
            .update(&dom, &sheet, &device, &states, &[])
            .device_invalidated
    );
    assert_eq!(
        retained
            .styles()
            .with_used_colors()
            .get(id)
            .unwrap()
            .background_color,
        "#19af72".parse::<ComputedColor>().unwrap()
    );
}
