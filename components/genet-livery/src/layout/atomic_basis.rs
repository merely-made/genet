//! Atomic inline boxes against their real containing block (Buckram K7,
//! `docs/2026-09-24_buckram_k7_atomic_inline_basis_execution_plan.md`).
//!
//! The atomic pre-pass formats every atomic inline root before the main
//! layout has placed any containing block. Its first pass formats each root
//! for its contribution. After the main layout, this module finds the roots
//! whose used size depends on the containing block they actually got, so the
//! pre-pass can format them again against it.

use super::*;

/// The containing block an atomic root is formatted against: its content
/// width, and its content height when that height is definite.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::layout) struct AtomicBasis {
    pub(in crate::layout) width: f32,
    pub(in crate::layout) height: Option<f32>,
}

/// Which atomic pre-pass is running.
pub(in crate::layout) enum AtomicPass<'a> {
    /// Every root formatted for its contribution.
    Contribution,
    /// The roots in `bases` formatted against their real containing block,
    /// each keeping its contribution-pass width in `contributions` for
    /// intrinsic queries. Every other root is formatted as in the first pass.
    Basis {
        bases: &'a HashMap<BoxId, AtomicBasis>,
        contributions: &'a HashMap<BoxId, f32>,
    },
}

impl AtomicLayoutPlane {
    pub(in crate::layout) fn roots(&self) -> impl Iterator<Item = BoxId> + '_ {
        self.subtrees.iter().map(|subtree| subtree.root)
    }
}

/// Whether `style` holds a percentage that its containing block resolves:
/// in a size, in padding or a margin on any side, or in a relative inset.
pub(in crate::layout) fn has_containing_percentage(style: &ComputedValues) -> bool {
    let size = |size: CssSize| matches!(size, CssSize::Value(value) | CssSize::FitContent(value) if value.has_percentage());
    let margin = |margin: Margin| matches!(margin, Margin::Value(value) if value.has_percentage());
    let inset = |inset: Inset| matches!(inset, Inset::Value(value) if value.has_percentage());
    [
        style.width,
        style.min_width,
        style.max_width,
        style.height,
        style.min_height,
        style.max_height,
    ]
    .into_iter()
    .any(size)
        || [
            style.padding_top,
            style.padding_right,
            style.padding_bottom,
            style.padding_left,
        ]
        .into_iter()
        .any(|padding| padding.0.has_percentage())
        || [
            style.margin_top,
            style.margin_right,
            style.margin_bottom,
            style.margin_left,
        ]
        .into_iter()
        .any(margin)
        || (style.position == CssPosition::Relative
            && [style.top, style.right, style.bottom, style.left]
                .into_iter()
                .any(inset))
}

/// CSS Sizing 3 section 5.2.1, for an atomic root's contribution: a
/// percentage preferred or max size acts as its initial value, and a
/// percentage in a min size, padding or a margin resolves against zero.
pub(in crate::layout) fn contribution_style(style: &mut ComputedValues) {
    let percentage = |size: CssSize| matches!(size, CssSize::Value(value) | CssSize::FitContent(value) if value.has_percentage());
    for size in [&mut style.width, &mut style.height] {
        if percentage(*size) {
            *size = CssSize::Auto;
        }
    }
    for size in [&mut style.max_width, &mut style.max_height] {
        if percentage(*size) {
            *size = CssSize::None;
        }
    }
    for size in [&mut style.min_width, &mut style.min_height] {
        if let CssSize::Value(value) = *size {
            *size = CssSize::Value(against_zero(value));
        }
    }
    for padding in [
        &mut style.padding_top,
        &mut style.padding_right,
        &mut style.padding_bottom,
        &mut style.padding_left,
    ] {
        padding.0 = against_zero(padding.0);
    }
    for margin in [
        &mut style.margin_top,
        &mut style.margin_right,
        &mut style.margin_bottom,
        &mut style.margin_left,
    ] {
        if let Margin::Value(value) = *margin {
            *margin = Margin::Value(against_zero(value));
        }
    }
}

/// `value` with its percentage resolved against zero.
fn against_zero(value: CssLengthPercentage) -> CssLengthPercentage {
    match value {
        CssLengthPercentage::Percentage(_) => CssLengthPercentage::Zero,
        CssLengthPercentage::Calc(mut calc) => {
            calc.percentage = 0.0;
            CssLengthPercentage::Calc(calc)
        },
        other => other,
    }
}

/// The atomic roots whose used size depends on the containing block the main
/// layout gave them, each with that block. A root that is not replaced is
/// sensitive when its style holds a containing-block percentage, or when it
/// is auto-width and its contribution-pass width reached the smaller of the
/// pre-pass's available size and the real one: in a narrower block or a
/// wider one, it would have come out another width.
pub(in crate::layout) fn basis_sensitive_roots<Id>(
    styles: &StylePlane<Id>,
    layout: &LiveryLayout<Id>,
    atomic: &AtomicLayoutPlane,
    viewport: (f32, f32),
) -> HashMap<BoxId, AtomicBasis>
where
    Id: Copy + Eq + Hash,
{
    let boxes = layout.boxes();
    let mut bases = HashMap::new();
    for root in atomic.roots() {
        let BoxOrigin::Element(node) = boxes[root].origin else {
            continue;
        };
        let Some(style) = styles.get(node) else {
            continue;
        };
        let Some(basis) = containing_basis(styles, layout, root, viewport) else {
            continue;
        };
        // A replaced root keeps its natural-size path: formatted against a
        // definite width it would stretch to it (see `layout_atomic_subtrees`).
        if boxes[root].replaced {
            continue;
        }
        let sensitive = has_containing_percentage(style)
            || (style.width == CssSize::Auto
                && (basis.width - viewport.0).abs() > 0.5
                && atomic.get(root).is_some_and(|fragment| {
                    let outer = fragment.width + horizontal_margins(style, basis.width);
                    outer >= viewport.0 - 0.5 || outer > basis.width + 0.5
                }));
        if sensitive {
            bases.insert(root, basis);
        }
    }
    bases
}

/// The content box of the block that contains `root` in `layout`.
pub(in crate::layout) fn containing_basis<Id>(
    styles: &StylePlane<Id>,
    layout: &LiveryLayout<Id>,
    root: BoxId,
    viewport: (f32, f32),
) -> Option<AtomicBasis>
where
    Id: Copy + Eq + Hash,
{
    let boxes = layout.boxes();
    let ContainingBlock::Box(block) = boxes[root].containing_block else {
        return Some(AtomicBasis {
            width: viewport.0,
            height: Some(viewport.1),
        });
    };
    let tree = layout.fragments();
    let fragment = tree
        .fragment_ids_for_box(block)
        .first()
        .and_then(|id| tree.get(*id))?;
    let style = match boxes[block].origin {
        BoxOrigin::Element(node) => styles.get(node),
        _ => None,
    };
    let (width, height) = style.map_or_else(
        || {
            let rect = fragment.physical_rect();
            (rect.width, rect.height)
        },
        |style| content_box_size(style, fragment),
    );
    let definite = style.is_some_and(
        |style| matches!(style.height, CssSize::Value(value) if !value.has_percentage()),
    );
    Some(AtomicBasis {
        width,
        height: definite.then_some(height),
    })
}

/// The shrink-to-fit border-box width (CSS 2.1 section 10.3.9) of an
/// auto-width atomic root that Buckram does not admit to its intrinsic
/// shrink-to-fit, such as one with percentage padding or a percentage child,
/// which would otherwise fill `basis_width`. `None` when the root is sized
/// some other way. Taffy's intrinsic widths leave percentage padding out, so
/// its share of `basis_width` is added back.
pub(in crate::layout) fn fallback_shrink_to_fit_width<D>(
    state: &mut BuildState<'_, D>,
    root: AlgorithmNodeId,
    node: D::NodeId,
    basis_width: f32,
) -> Option<f32>
where
    D: LayoutDom,
    D::NodeId: Copy + Eq + Hash,
{
    if state.tree.uses_intrinsic_shrink_to_fit(root) || is_replaced_element(state.dom, node) {
        return None;
    }
    let mut style = state.styles.get(node)?.clone();
    if state.contribution_root == Some(node) {
        contribution_style(&mut style);
    }
    if style.width != CssSize::Auto
        || matches!(style.display, CssDisplay::Table | CssDisplay::InlineTable)
    {
        return None;
    }
    let min = state.measure_intrinsic_width(root, AlgorithmAvailableSpace::MinContent);
    let max = state.measure_intrinsic_width(root, AlgorithmAvailableSpace::MaxContent);
    let em = crate::paint::used_font_size(&style);
    let percentage_padding: f32 = [style.padding_left.0, style.padding_right.0]
        .into_iter()
        .map(|padding| {
            taffy_style::length_percentage_px(padding, em, basis_width)
                - taffy_style::length_percentage_px(padding, em, 0.0)
        })
        .sum();
    let available = (basis_width - horizontal_margins(&style, basis_width)).max(0.0);
    Some(
        available
            .min(max + percentage_padding)
            .max(min + percentage_padding),
    )
}

fn horizontal_margins(style: &ComputedValues, basis: f32) -> f32 {
    let em = crate::paint::used_font_size(style);
    [style.margin_left, style.margin_right]
        .into_iter()
        .map(|margin| match margin {
            Margin::Value(value) => taffy_style::length_percentage_px(value, em, basis),
            _ => 0.0,
        })
        .sum()
}

/// After the basis pass, every sensitive root must still sit in the
/// containing block it was formatted against: its contribution kept that
/// block's size. A mismatch means the two passes disagree, which is loud.
pub(in crate::layout) fn debug_assert_bases_held<Id>(
    styles: &StylePlane<Id>,
    layout: &LiveryLayout<Id>,
    bases: &HashMap<BoxId, AtomicBasis>,
    viewport: (f32, f32),
) where
    Id: Copy + Eq + Hash,
{
    if cfg!(debug_assertions) {
        for (root, basis) in bases {
            let held = containing_basis(styles, layout, *root, viewport);
            debug_assert!(
                held.is_some_and(|held| (held.width - basis.width).abs() <= 0.5
                    && match (held.height, basis.height) {
                        (Some(held), Some(basis)) => (held - basis).abs() <= 0.5,
                        (held, basis) => held.is_none() == basis.is_none(),
                    }),
                "atomic root {root:?} was formatted against {basis:?} but sits in {held:?}",
            );
        }
    }
}

#[cfg(test)]
thread_local! {
    static BASIS_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many basis passes this thread's layouts have run.
#[cfg(test)]
pub(crate) fn basis_passes() -> usize {
    BASIS_PASSES.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::layout) fn note_basis_pass() {
    BASIS_PASSES.with(|passes| passes.set(passes.get() + 1));
}
