// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashSet;

use super::*;

/// A hyperlink in DOM preorder. Identity is local to the supplied document
/// representation. Equal hrefs can name different occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOccurrence {
    /// Zero-based order among the anchors with href attributes.
    pub document_order: usize,
    pub link: Link,
    /// Canonical Fleece text selectors, absent for links without an anchorable
    /// text range such as image-only links.
    pub anchor: Option<TextAnchor>,
}

/// Extract every hyperlink occurrence against the same text stream as
/// [`extract_text`]. URL resolution and source identity belong to the caller.
pub fn extract_link_occurrences<D: LayoutDom>(dom: &D) -> Vec<LinkOccurrence> {
    let index = FleeceTextIndex::build(dom);
    let mut occurrences = Vec::new();
    visit_link_nodes(dom, dom.document(), &mut |node, href| {
        occurrences.push(LinkOccurrence {
            document_order: occurrences.len(),
            link: Link {
                href,
                text: text_of(dom, node),
                rel: attr(dom, node, "rel"),
            },
            anchor: anchor_for_node(dom, node, &index, ExtractionOptions::default()),
        });
    });
    occurrences
}

/// Find the link occurrences that enclose text nodes in a host selection.
/// The result uses the same document-local order as [`LinkOccurrence`].
pub fn link_occurrence_orders_for_sources<D: LayoutDom>(
    dom: &D,
    sources: &[D::NodeId],
) -> Vec<usize> {
    let selected: HashSet<_> = sources
        .iter()
        .filter_map(|source| {
            std::iter::successors(Some(*source), |node| dom.parent(*node)).find(|node| {
                local_name(dom, *node) == Some("a") && attr(dom, *node, "href").is_some()
            })
        })
        .collect();
    if selected.is_empty() {
        return Vec::new();
    }
    let mut orders = Vec::new();
    let mut order = 0;
    visit_link_nodes(dom, dom.document(), &mut |node, _| {
        if selected.contains(&node) {
            orders.push(order);
        }
        order += 1;
    });
    orders
}

pub(super) fn visit_link_nodes<D: LayoutDom>(
    dom: &D,
    node: D::NodeId,
    visitor: &mut impl FnMut(D::NodeId, String),
) {
    if local_name(dom, node) == Some("a")
        && let Some(href) = attr(dom, node, "href")
    {
        visitor(node, href);
    }
    for child in dom.dom_children(node) {
        visit_link_nodes(dom, child, visitor);
    }
}

#[cfg(test)]
mod tests {
    use genet_static_dom::StaticDocument;

    use super::*;

    #[test]
    fn equal_hrefs_get_distinct_anchors_and_selection_orders() {
        let dom = StaticDocument::parse(concat!(
            "<p>First <a href='../shared' rel='next'>Cafe\u{301}</a> on this page.</p>",
            "<p>Second <a href='../shared'>Cafe\u{301}</a> elsewhere.</p>",
            "<p><a href='../shared'><img alt='Map'></a></p>",
        ));
        let occurrences = extract_link_occurrences(&dom);
        assert_eq!(occurrences.len(), 3);
        assert_eq!(occurrences[0].link.href, "../shared");
        assert_eq!(occurrences[0].link.rel.as_deref(), Some("next"));
        assert_eq!(occurrences[1].document_order, 1);
        assert!(occurrences[2].anchor.is_none());
        let first = occurrences[0].anchor.as_ref().unwrap();
        let second = occurrences[1].anchor.as_ref().unwrap();
        assert_eq!(first.quote.exact, second.quote.exact);
        assert_ne!(first.position, second.position);
        assert_eq!(
            resolve_anchor(&extract_text(&dom), second),
            [second.position]
        );

        let mut nodes = Vec::new();
        visit_link_nodes(&dom, dom.document(), &mut |node, _| nodes.push(node));
        let selected = dom.dom_children(nodes[1]).next().unwrap();
        assert_eq!(link_occurrence_orders_for_sources(&dom, &[selected]), [1]);
    }
}
