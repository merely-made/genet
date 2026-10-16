/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The lanes as inker session engines: `SessionEngine<Scene>` /
//! `DocumentSession<Scene>` impls wrapping [`LoadedDocument`], the scripted
//! document.
//!
//! Construction seams (fetchers, themes, cookie jars) live on the engine at
//! registration time; the spawn request stays plain data (session-engines
//! plan, review-resolved 2026-07-10).

#[cfg(feature = "scripted")]
use document_session_api::{DocumentCapabilities, DocumentCapabilityStatus};

#[cfg(feature = "scripted")]
fn retained_document_capabilities(find_reason: impl Into<String>) -> DocumentCapabilities {
    DocumentCapabilities {
        find_in_page: DocumentCapabilityStatus::unsupported(find_reason),
        page_zoom: DocumentCapabilityStatus::unsupported(
            "scripted and smolweb sessions do not expose page zoom",
        ),
        page_capture: DocumentCapabilityStatus::unsupported(
            "retained sessions do not capture rendered pages",
        ),
        navigation: DocumentCapabilityStatus::Partial {
            detail: "the host owns document lineage, policy, and refetch".into(),
        },
    }
}

#[cfg(any(feature = "livery", feature = "scripted"))]
mod clip;
#[cfg(feature = "livery")]
mod frames;
#[cfg(feature = "livery")]
mod livery;
#[cfg(feature = "scripted")]
mod scripted;
#[cfg(test)]
mod tests;

#[cfg(feature = "scripted")]
pub(crate) use clip::links_for_source_nodes;
#[cfg(any(feature = "livery", feature = "scripted"))]
pub(crate) use clip::{
    ClipRange, ClipSelection, content_report, semantic_clip_from_dom,
    semantic_clip_from_selection_with_links,
};
#[cfg(all(test, feature = "livery"))]
pub(crate) use livery::EditableKind;
#[cfg(feature = "livery")]
pub use livery::{LiveryDocumentSession, LiveryResourcePreparation, LiverySessionEngine};
#[cfg(feature = "scripted")]
pub use scripted::{ScriptedDocumentSession, ScriptedSessionEngine};

/// Add the shared inert validation alert after document geometry is projected.
#[cfg(feature = "livery")]
fn with_validation_alert(
    projection: document_session_api::DocumentA11yProjection,
    notice: Option<(u64, &str, Option<[f32; 4]>)>,
    cache: &std::cell::Cell<Option<(u64, document_session_api::DocumentA11yNodeId)>>,
) -> document_session_api::DocumentA11yProjection {
    use document_session_api::{
        DocumentA11yBounds, DocumentA11yLive, DocumentA11yNode, DocumentA11yNodeId,
        DocumentA11yProjection, DocumentA11yRole, DocumentA11yState,
    };

    let Some((sequence, message, bounds)) = notice else {
        cache.set(None);
        return projection;
    };
    let id = cache
        .get()
        .filter(|(cached_sequence, id)| {
            *cached_sequence == sequence && projection.node(*id).is_none()
        })
        .map(|(_, id)| id)
        .unwrap_or_else(|| {
            // Host-only nodes cannot alias any current arena projection.
            // A repeated report receives a new ID so unchanged text can
            // still be announced as a new alert.
            let previous = cache.get().map(|(_, id)| id);
            (0..=u64::MAX)
                .rev()
                .map(DocumentA11yNodeId::new)
                .find(|id| Some(*id) != previous && projection.node(*id).is_none())
                .expect("a finite document leaves a native alert identity available")
        });
    cache.set(Some((sequence, id)));
    let root = projection.root();
    let mut nodes = projection.nodes().to_vec();
    let Some(root_node) = nodes.iter_mut().find(|node| node.id == root) else {
        return projection;
    };
    root_node.children.push(id);
    nodes.push(DocumentA11yNode {
        id,
        parent: Some(root),
        children: Vec::new(),
        role: DocumentA11yRole::Alert,
        name: Some(message.to_owned()),
        description: None,
        value: None,
        numeric_value: None,
        numeric_minimum: None,
        numeric_maximum: None,
        bounds: bounds.map(|[x, y, width, height]| DocumentA11yBounds {
            x,
            y,
            width,
            height,
        }),
        state: DocumentA11yState {
            live: Some(DocumentA11yLive::Assertive),
            ..DocumentA11yState::default()
        },
        actions: Vec::new(),
    });
    DocumentA11yProjection::new(0, projection.support().clone(), root, nodes)
}
