/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The description's title check and the name-from-content gate.

use std::collections::HashSet;

use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{LayoutDom, LayoutDomMut, NodeKind};

use super::{Names, attr, flat};

const CORPUS: &[&str] = &[
    "<button title=Tip>Save</button><button title=Tip></button><button>Plain</button>",
    "<input type=submit value=V title=Tip><input type=reset title=Tip><input type=image>",
    "<div role=dialog title=Tip><h2>H</h2></div><div role=dialog title=Tip><p>x</p></div>",
    "<abbr title=Tip>WPT</abbr><abbr title=''>WPT</abbr><span title=Tip>Text</span>",
    "<span id=a>Label</span><input aria-labelledby=a title=Tip><input title=Tip>",
    "<label for=f>Field</label><input id=f title=Tip><img alt=Alt title=Tip><img title=Tip>",
    "<table title=Tip><caption>Cap</caption><tr><td title=Cell>1</td></tr></table>",
    "<div role=group title=Tip aria-label=Own>x</div><a href=# title=Tip>Go</a><p title=T>P</p>",
];

fn elements(dom: &ScriptedDom) -> Vec<genet_scripted_dom::NodeId> {
    let mut out = Vec::new();
    let mut pending = vec![dom.document()];
    while let Some(node) = pending.pop() {
        if dom.kind(node) == NodeKind::Element {
            out.push(node);
        }
        pending.extend(dom.dom_children(node));
    }
    out
}

/// The description as computed before the title check moved first.
fn description_with_recompute<D: LayoutDom>(names: &Names<D>, node: D::NodeId) -> Option<String> {
    if !names.references(node, "aria-describedby").is_empty()
        || attr(names.dom, node, "aria-description").is_some()
    {
        return names.description(node);
    }
    if names
        .compute(node, false, true, false, false, &mut HashSet::new())
        .is_some()
    {
        return attr(names.dom, node, "title").map(flat);
    }
    None
}

#[test]
fn description_is_unchanged_by_checking_title_first() {
    let mut compared = 0;
    for html in CORPUS {
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(dom.document(), html);
        let names = Names::new(&dom);
        for node in elements(&dom) {
            assert_eq!(
                names.description(node),
                description_with_recompute(&names, node),
                "{html}"
            );
            compared += 1;
        }
    }
    assert!(compared > 25, "the corpus reached {compared} elements");
}

fn root_name(html: &str) -> Option<String> {
    let mut dom = ScriptedDom::new();
    dom.set_inner_html(dom.document(), html);
    let names = Names::new(&dom);
    let node = names
        .element_by_id(dom.document(), "target")
        .expect("target");
    names.name(node)
}

#[test]
fn a_generic_is_named_only_through_its_direct_text() {
    // Direct text keeps main's whole content traversal (D65, interim).
    for (html, name) in [
        ("<p id=target>text</p>", "text"),
        ("<div id=target>Hello <b>world</b></div>", "Hello world"),
        ("<a id=target>text</a>", "text"),
        ("<abbr id=target>x</abbr>", "x"),
    ] {
        assert_eq!(root_name(html).as_deref(), Some(name), "{html}");
    }
    // Element children only, or whitespace-only text: no name.
    assert_eq!(root_name("<div id=target><span>x</span></div>"), None);
    assert_eq!(root_name("<div id=target> <span>x</span> </div>"), None);
    assert_eq!(
        root_name("<div id=target><div>a</div><div>b</div></div>"),
        None
    );
    // Title still names a generic that takes no content.
    assert_eq!(
        root_name("<div id=target title=Tip><span>x</span></div>").as_deref(),
        Some("Tip")
    );
    // Name-from-contents roles take content without direct text.
    assert_eq!(
        root_name("<div role=row id=target><span>Row</span></div>").as_deref(),
        Some("Row")
    );
    assert_eq!(
        root_name("<div role=columnheader id=target><b>Col</b></div>").as_deref(),
        Some("Col")
    );
    // The denylist is unchanged: direct text does not name a group.
    assert_eq!(root_name("<div role=group id=target>Text</div>"), None);
}

#[test]
fn generated_text_counts_as_direct_text() {
    // Provisional, matching main: a generic whose only text is generated.
    let mut dom = ScriptedDom::new();
    dom.set_inner_html(dom.document(), "<div id=target><span>x</span></div>");
    let generated = |_| ("Before ".to_owned(), String::new());
    let names = Names::new(&dom).with_generated(&generated);
    let node = names
        .element_by_id(dom.document(), "target")
        .expect("target");
    assert!(
        names
            .name(node)
            .is_some_and(|name| name.starts_with("Before"))
    );
}
