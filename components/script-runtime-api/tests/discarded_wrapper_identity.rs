// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use genet_scripted_dom::NodeId;
use layout_dom_api::LayoutDom;
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct Resources;
impl ScriptResourceLoader for Resources {
    fn load(&self, _: &str) -> Option<String> {
        Some("<body>replacement</body>".into())
    }
}

fn run<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) {
    if let Err(error) = rt.eval(source) {
        panic!("{source}: {}", rt.describe_error(&error));
    }
}

fn identity_after_discard<E: ScriptEngine>(navigate: bool, connected: bool) {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://identity.test/").unwrap();
    rt.set_script_resource_loader(Box::new(Resources));
    rt.parse_document_interleaved(
        "<body><iframe id=source></iframe><iframe id=destination></iframe></body>",
        &NoScriptLoader,
    );
    rt.run_event_loop(40).unwrap();
    let source_realm = rt.frame_realms(rt.top_realm())[0].1;
    run(
        &mut rt,
        r#"
        var sourceFrame = document.getElementById('source');
        var sourceWindow = sourceFrame.contentWindow;
        var destinationFrame = document.getElementById('destination');
        var destinationWindow = destinationFrame.contentWindow;
        var node = sourceWindow.document.createElement('section');
        node.id = 'kept';
        node.innerHTML = '<span>ordinary</span><template><b>template</b></template>';
        var leaf = node.firstChild;
        var openRoot = node.attachShadow({mode:'open'});
        openRoot.innerHTML = '<div></div>';
        var closedRoot = openRoot.firstChild.attachShadow({mode:'closed'});
        closedRoot.innerHTML = '<i>closed</i>';
        var contents = node.lastChild.content;
        var templateLeaf = contents.firstChild;
        var closedLeaf = closedRoot.firstChild;
        var saved = [node, leaf, openRoot, openRoot.firstChild, closedRoot,
                     closedLeaf, contents, templateLeaf];
        var prototypes = saved.map(function(n) { return Object.getPrototypeOf(n); });
        var token = {}; node.token = token;
        var events = 0; node.addEventListener('probe', function() { events++; });
        destinationWindow.document.adoptNode(node);
        function check() {
            var observed = [node, node.firstChild, node.shadowRoot,
                node.shadowRoot.firstChild, closedLeaf.getRootNode(),
                closedRoot.firstChild, node.lastChild.content, node.lastChild.content.firstChild];
            for (var i=0; i<saved.length; i++) {
                if (observed[i] !== saved[i]) throw new Error('identity '+i);
                if (Object.getPrototypeOf(observed[i]) !== prototypes[i]) throw new Error('prototype '+i);
            }
            if (node.token !== token) throw new Error('expando');
            if (openRoot.firstChild.shadowRoot !== null) throw new Error('closed exposed');
            if (leaf.ownerDocument !== node.ownerDocument || closedLeaf.ownerDocument !== node.ownerDocument)
                throw new Error('owner document');
            if (contents.ownerDocument === node.ownerDocument || templateLeaf.ownerDocument !== contents.ownerDocument)
                throw new Error('template owner');
        }
        check();
    "#,
    );
    let ids: Vec<_> = (0..8)
        .map(|i| {
            let value = rt.eval(&format!("__nodeRawId(saved[{i}].__ref)")).unwrap();
            NodeId::from_raw(rt.value_to_string(&value).unwrap().parse().unwrap())
        })
        .collect();
    if connected {
        run(
            &mut rt,
            "destinationWindow.document.body.appendChild(node);",
        );
    }
    run(
        &mut rt,
        if navigate {
            "sourceFrame.src = '/replacement.html'; sourceWindow = null;"
        } else {
            "sourceFrame.remove(); sourceWindow = null;"
        },
    );
    rt.run_event_loop(50).unwrap();
    assert!(
        rt.host_in_realm(source_realm).is_err(),
        "source registration survived"
    );
    run(&mut rt, "check();");
    for _ in 0..3 {
        rt.collect_garbage();
    }
    run(&mut rt, "check(); document.adoptNode(node); check();");
    for _ in 0..3 {
        rt.collect_garbage();
    }
    run(
        &mut rt,
        "check(); destinationWindow.document.adoptNode(node); check(); document.adoptNode(node); destinationFrame.remove(); destinationWindow = null;",
    );
    rt.run_event_loop(50).unwrap();
    for _ in 0..3 {
        rt.collect_garbage();
    }
    run(
        &mut rt,
        "check(); leaf.dispatchEvent(new Event('probe', {bubbles:true})); if(events !== 1) throw new Error('listener');",
    );
    for id in &ids {
        assert!(rt.host().borrow().dom.is_live(*id), "held node collected");
    }
    run(
        &mut rt,
        "var weak = saved.map(function(n) { return new WeakRef(n); }); node = openRoot = closedRoot = closedLeaf = contents = templateLeaf = saved = prototypes = token = null;",
    );
    for _ in 0..3 {
        rt.collect_garbage();
    }
    run(
        &mut rt,
        r#"
        for (var i=0; i<weak.length; i++) {
            if (!weak[i].deref()) throw new Error('sole descendant lost component '+i);
        }
        if (leaf.parentNode !== weak[0].deref() || leaf.parentNode.shadowRoot !== weak[2].deref() ||
            leaf.parentNode.lastChild.content !== weak[6].deref()) throw new Error('sole descendant identity');
        leaf = weak = null;
    "#,
    );
    for _ in 0..5 {
        rt.collect_garbage();
    }
    for id in ids {
        assert!(
            !rt.host().borrow().dom.is_live(id),
            "unreachable node retained: {id:?}"
        );
    }
}

macro_rules! backend {
    ($name:ident, $engine:ty) => {
        mod $name {
            #[test]
            fn template_retention_is_directed() {
                super::template_retention_is_directed::<$engine>();
            }
            #[test]
            fn removed_detached() {
                super::identity_after_discard::<$engine>(false, false);
            }
            #[test]
            fn removed_connected() {
                super::identity_after_discard::<$engine>(false, true);
            }
            #[test]
            fn navigated_detached() {
                super::identity_after_discard::<$engine>(true, false);
            }
            #[test]
            fn navigated_connected() {
                super::identity_after_discard::<$engine>(true, true);
            }
        }
    };
}

fn template_retention_is_directed<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://identity.test/").unwrap();
    rt.parse_document_interleaved("<body><iframe></iframe></body>", &NoScriptLoader);
    rt.run_event_loop(30).unwrap();
    run(
        &mut rt,
        r#"
        var frame = document.querySelector('iframe');
        var template = frame.contentDocument.createElement('template');
        template.innerHTML = '<b>retained</b>';
        var contents = template.content, weakTemplate = new WeakRef(template);
        document.adoptNode(template);
        frame.remove();
    "#,
    );
    rt.run_event_loop(30).unwrap();
    run(&mut rt, "template = null;");
    for _ in 0..4 {
        rt.collect_garbage();
    }
    run(
        &mut rt,
        "if(weakTemplate.deref() !== undefined) throw new Error('contents retained template'); if(contents.firstChild.textContent !== 'retained') throw new Error('contents lost'); contents = weakTemplate = null;",
    );
    for _ in 0..4 {
        rt.collect_garbage();
    }
}

backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
