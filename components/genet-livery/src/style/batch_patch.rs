// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Patching a resolved style plane for one scroll batch: only the elements
//! the style session restyled, plus those entering and leaving, are resolved
//! and compared. Every other entry is taken as unchanged, which is sound only
//! when the restyled set is complete.

use super::*;

/// One element's entries in a plane. Every plane map is keyed by element.
#[derive(Clone)]
struct Entry {
    values: ComputedValues,
    custom: Option<CustomProperties>,
    inline_diagnostics: Option<Vec<DeclarationError>>,
    hint_diagnostics: Option<Vec<PresentationalHintDiagnostic>>,
    legacy_alignment: Option<LegacyDescendantAlignment>,
    explicit_inheritance: bool,
}

type GeneratedKey<Id> = (Id, buckram::PseudoElement);

/// How a kept element's entries changed.
enum Change {
    None,
    /// Only `background-color`, which paint reads from the plane.
    Paint,
    /// Only the insets of an absolute or fixed element.
    Insets,
}

/// The resolved entries a batch puts into a plane. Swapping it in leaves the
/// replaced entries behind in it, so swapping again restores the plane.
pub(crate) struct StylePatch<Id> {
    entries: Vec<(Id, Option<Entry>)>,
    generated: Vec<(GeneratedKey<Id>, Option<(ComputedValues, String)>)>,
    /// The sole absolute or fixed element whose insets changed.
    pub(crate) moved: Option<Id>,
}

impl<Id: Copy + Eq + Hash> StylePlane<Id> {
    /// Resolve `cascade`'s entries for `restyled`, `entered` and `left` as a
    /// whole-plane resolve would, and classify them against this resolved
    /// plane. A kept element may change only `background-color`, or only its
    /// insets if it is the one absolute or fixed element doing so; anything
    /// else refuses, as does a container-relative length on a resolved
    /// element (its value would depend on container layout).
    #[expect(
        clippy::too_many_arguments,
        reason = "both planes, the batch's node sets and the resolve inputs"
    )]
    pub(crate) fn scroll_batch_patch<D: LayoutDom<NodeId = Id>>(
        &self,
        cascade: &Self,
        restyled: &HashSet<Id>,
        entered: &HashSet<Id>,
        left: &HashSet<Id>,
        dom: &D,
        text: &mut crate::TextSystem,
        viewport: ViewportSizes,
    ) -> Option<StylePatch<Id>> {
        if self.color_context != cascade.color_context {
            return None;
        }
        let kept = |id: &Id| !entered.contains(id) && !left.contains(id);
        let nodes = restyled
            .iter()
            .chain(entered)
            .chain(left)
            .copied()
            .collect::<HashSet<_>>();
        let mut moved = None;
        let mut entries = Vec::new();
        for id in &nodes {
            let new = match cascade.values.get(id) {
                Some(values) => Some(cascade.resolved_entry(*id, values, text, viewport)?),
                None => None,
            };
            let old = self.values.get(id);
            match (old, &new) {
                (None, None) => continue,
                _ if !kept(id) => {},
                // A dead descendant of a leaving root.
                (Some(_), None) if !dom.is_live(*id) => {},
                (Some(old), Some(new)) => match self.classify(*id, old, new)? {
                    // Nothing changed: no entry to swap.
                    Change::None => continue,
                    Change::Paint => {},
                    Change::Insets => {
                        if moved.replace(*id).is_some() {
                            return None;
                        }
                    },
                },
                _ => return None,
            }
            entries.push((*id, new));
        }

        let mut previous_generated = self
            .generated
            .iter()
            .filter(|((owner, _), _)| nodes.contains(owner))
            .collect::<HashMap<_, _>>();
        let mut generated = Vec::new();
        for (key, (style, content)) in &cascade.generated {
            if !nodes.contains(&key.0) {
                continue;
            }
            let mut style = style.clone();
            resolve_ch(&mut style, text, viewport);
            let entry = (style, content.clone());
            let previous = previous_generated.remove(key);
            if kept(&key.0) {
                if previous != Some(&entry) {
                    return None;
                }
                continue;
            }
            generated.push((*key, Some(entry)));
        }
        for (key, _) in previous_generated {
            if kept(&key.0) && dom.is_live(key.0) {
                return None;
            }
            generated.push((*key, None));
        }
        Some(StylePatch {
            entries,
            generated,
            moved,
        })
    }

    /// Swap `patch`'s entries with this plane's. Swapping the same patch a
    /// second time puts the plane back as it was.
    pub(crate) fn swap_patch(&mut self, patch: &mut StylePatch<Id>) {
        self.invalidate_used_colors();
        for (id, entry) in &mut patch.entries {
            let previous = self.take_entry(*id);
            if let Some(entry) = entry.take() {
                self.put_entry(*id, entry);
            }
            *entry = previous;
        }
        for (key, entry) in &mut patch.generated {
            let previous = self.generated.remove(key);
            if let Some(entry) = entry.take() {
                self.generated.insert(*key, entry);
            }
            *entry = previous;
        }
    }

    /// What changed for a kept element, or `None` when layout could read it.
    fn classify(&self, id: Id, old: &ComputedValues, new: &Entry) -> Option<Change> {
        if self.custom.get(&id) != new.custom.as_ref()
            || self.inline_diagnostics.get(&id) != new.inline_diagnostics.as_ref()
            || self.presentational_hint_diagnostics.get(&id) != new.hint_diagnostics.as_ref()
            || self.legacy_descendant_alignment.get(&id) != new.legacy_alignment.as_ref()
            || self.explicit_inheritance.contains(&id) != new.explicit_inheritance
        {
            return None;
        }
        let current = &new.values;
        if current == old {
            return Some(Change::None);
        }
        let mut normalized = old.clone();
        normalized.background_color = current.background_color.clone();
        if normalized == *current {
            return Some(Change::Paint);
        }
        normalized.top = current.top;
        normalized.right = current.right;
        normalized.bottom = current.bottom;
        normalized.left = current.left;
        (normalized == *current && matches!(current.position, Position::Absolute | Position::Fixed))
            .then_some(Change::Insets)
    }

    /// This cascade plane's entries for `id`, with `ch`, `cap` and container
    /// fallback lengths resolved as `resolve_container_relative_styles_with_text`
    /// resolves them. Both depend on the element's own values only.
    fn resolved_entry(
        &self,
        id: Id,
        values: &ComputedValues,
        text: &mut crate::TextSystem,
        viewport: ViewportSizes,
    ) -> Option<Entry> {
        let mut values = values.clone();
        resolve_ch(&mut values, text, viewport);
        let fallback = livery::values::RelativeLengthEnvironment::container_fallback(viewport)
            .with_vertical_writing(values.writing_mode.is_vertical());
        if resolve_relative_lengths(&mut values, fallback) {
            return None;
        }
        Some(Entry {
            values,
            custom: self.custom.get(&id).cloned(),
            inline_diagnostics: self.inline_diagnostics.get(&id).cloned(),
            hint_diagnostics: self.presentational_hint_diagnostics.get(&id).cloned(),
            legacy_alignment: self.legacy_descendant_alignment.get(&id).copied(),
            explicit_inheritance: self.explicit_inheritance.contains(&id),
        })
    }

    fn take_entry(&mut self, id: Id) -> Option<Entry> {
        let values = self.values.remove(&id);
        let entry = Entry {
            values: values?,
            custom: self.custom.remove(&id),
            inline_diagnostics: self.inline_diagnostics.remove(&id),
            hint_diagnostics: self.presentational_hint_diagnostics.remove(&id),
            legacy_alignment: self.legacy_descendant_alignment.remove(&id),
            explicit_inheritance: self.explicit_inheritance.remove(&id),
        };
        Some(entry)
    }

    fn put_entry(&mut self, id: Id, entry: Entry) {
        self.values.insert(id, entry.values);
        if let Some(custom) = entry.custom {
            self.custom.insert(id, custom);
        }
        if let Some(diagnostics) = entry.inline_diagnostics {
            self.inline_diagnostics.insert(id, diagnostics);
        }
        if let Some(diagnostics) = entry.hint_diagnostics {
            self.presentational_hint_diagnostics.insert(id, diagnostics);
        }
        if let Some(alignment) = entry.legacy_alignment {
            self.legacy_descendant_alignment.insert(id, alignment);
        }
        if entry.explicit_inheritance {
            self.explicit_inheritance.insert(id);
        }
    }
}

/// The `ch` and `cap` pass of `StylePlane::resolve_ch_lengths`, for one value.
pub(super) fn resolve_ch(
    values: &mut ComputedValues,
    text: &mut crate::TextSystem,
    viewport: ViewportSizes,
) {
    let environment = livery::values::RelativeLengthEnvironment::viewport(viewport)
        .with_vertical_writing(values.writing_mode.is_vertical())
        .with_ch_advance(text.ch_advance(values))
        .with_cap_height(text.cap_height(values));
    let _ = resolve_relative_lengths(values, environment);
}
