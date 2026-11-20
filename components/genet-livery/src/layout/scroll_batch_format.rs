// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Formatting one absolutely positioned root on its own, the way complete
//! layout formats it inside its parent: the scratch formatter puts it at its
//! static position, then Buckram's positioned pass places its positioned
//! descendants against it.

use super::*;

/// Format the subtree at `root_box` in a block the size of its parent's
/// content box, at that block's origin. Returns its fragments, its text and
/// its root fragment; the root itself is still at its static position, since
/// its containing block lies outside. `None` when it builds no single root.
#[expect(
    clippy::too_many_arguments,
    reason = "one formatting pass borrows the layout inputs"
)]
pub(in crate::layout) fn format_positioned_root<D>(
    dom: &D,
    styles: &StylePlane<D::NodeId>,
    boxes: &GeneratedBoxTree<D::NodeId>,
    root_box: BoxId,
    containing: (f32, f32),
    viewport: (f32, f32),
    text: &mut TextSystem,
    image_sources: &dyn crate::ImageSourceLookup,
) -> Result<Option<(FragmentTree, TextFrame<D::NodeId>, FragmentId)>, LayoutError>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    let (width, height) = containing;
    if !(width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0) {
        return Ok(None);
    }
    let atomic = AtomicLayoutPlane::default();
    let mut intrinsic_sizes = IntrinsicSizeCache::default();
    let mut state = InlineBuildState {
        dom,
        styles,
        boxes,
        atomic: &atomic,
        text_batch: text_batch::TextBatch::new(text.shaping_options().record_timings),
        tree: {
            let mut tree = AlgorithmTree::new();
            tree.set_calc_resolver(resolve_taffy_calc);
            tree
        },
        image_sources,
        table_shadow: TableShadowLedger::default(),
        pending_tables: Vec::new(),
        pending_table_handoff: None,
    };
    let parent_font_size = inherited_font_size(boxes, styles, root_box);
    let Some(built) = state.build_box(
        root_box,
        None,
        parent_font_size,
        (Some(width), Some(height)),
    )?
    else {
        return Ok(None);
    };
    let formatter_root = state.tree.new_with_children_and_block_style(
        AlgorithmKind::Block,
        BlockStyle {
            size: BlockDimensions::new(
                BlockSizeValue::Length(FlowLength::px(width)),
                BlockSizeValue::Length(FlowLength::px(height)),
            ),
            ..BlockStyle::default()
        },
        Style {
            display: Display::Block,
            size: Size {
                width: Dimension::length(width),
                height: Dimension::length(height),
            },
            ..Style::default()
        },
        &[built],
        Vec::new(),
    );
    state
        .text_batch
        .flush::<D>(text, state.atomic, &mut state.tree)?;
    text_batch::check_errors(&state.tree)?;
    state.tree.compute_layout_with_measure(
        formatter_root,
        AlgorithmSize::new(
            AlgorithmAvailableSpace::Definite(width),
            AlgorithmAvailableSpace::Definite(height),
        ),
        |known, available, _, context, line_constraints| {
            measure_inline_algorithm_node(
                text,
                dom,
                styles,
                boxes,
                &atomic,
                &mut intrinsic_sizes,
                known,
                available,
                context,
                line_constraints,
            )
        },
    );
    text_batch::check_errors(&state.tree)?;
    populate_inline_baselines(&mut state.tree);
    let mut fragments = FragmentTree::default();
    let mut text_frame = TextFrame::default();
    let table_paint = state.table_paint_plane();
    collect_inline_fragments(
        &state.tree,
        boxes,
        formatter_root,
        FragmentCursor {
            origin: Point { x: 0.0, y: 0.0 },
            containing: Fragment {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            parent: None,
        },
        &table_paint.fragments(),
        &mut FragmentOutput {
            fragments: &mut fragments,
        },
        &mut text_frame,
        styles,
    )?;
    drop(state);
    // Positioned descendants are placed against the root, which is here;
    // the root's own containing block is not, so this pass skips it.
    apply_absolute_and_fixed_positioning(
        &mut fragments,
        boxes,
        styles,
        dom,
        Some(&mut text_frame),
        image_sources,
        &HashMap::new(),
        viewport.0,
        viewport.1,
    );
    let [root] = *fragments.fragment_ids_for_box(root_box) else {
        return Ok(None);
    };
    Ok(Some((fragments, text_frame, root)))
}
