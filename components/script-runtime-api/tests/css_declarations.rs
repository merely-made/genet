// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Complete CSS declaration lists cross the native CSS owner boundary.

use genet_static_dom::StaticDocument;
use script_engine_api::ScriptEngine;
use script_runtime_api::{
    InlineStyleHandler, InlineStyleValueResult, Runtime, StyleSheetHandler, StyleSheetMutationError,
    StyleSheetRule, StyleSheetRuleKind,
};

struct PairStyle;
impl InlineStyleHandler for PairStyle {
    fn canonicalize(&self, property: &str, value: &str) -> InlineStyleValueResult {
        if property == "pair" && value == "left right" {
            InlineStyleValueResult::Expanded(vec![
                ("pair-first".into(), "left".into()),
                ("pair-second".into(), "right".into()),
            ])
        } else {
            InlineStyleValueResult::PassThrough
        }
    }

    fn shorthand_components(&self, property: &str) -> Vec<String> {
        if property == "pair" {
            vec!["pair-first".into(), "pair-second".into()]
        } else {
            Vec::new()
        }
    }

    fn shorthands(&self) -> Vec<String> {
        vec!["pair".into()]
    }

    fn reconstruct_shorthand(&self, property: &str, components: &[(String, String)]) -> Option<String> {
        if property == "pair"
            && components.len() == 2
            && components[0].0 == "pair-first"
            && components[0].1 == "left"
            && components[1].0 == "pair-second"
            && components[1].1 == "right"
        {
            Some("left right".into())
        } else {
            None
        }
    }
}

struct RetainedRule;
impl StyleSheetHandler for RetainedRule {
    fn sheet_count(&self) -> usize { 1 }
    fn rule_count(&self, sheet: usize) -> Option<usize> { (sheet == 0).then_some(1) }
    fn insert_rule(&self, _: usize, _: &str, _: usize) -> Result<usize, StyleSheetMutationError> {
        Err(StyleSheetMutationError::IndexSize)
    }
    fn delete_rule(&self, _: usize, _: usize) -> Result<(), StyleSheetMutationError> {
        Err(StyleSheetMutationError::IndexSize)
    }
    fn rule_by_key(&self, key: &str, path: &[usize]) -> Option<StyleSheetRule> {
        if key != "0" || path != [0] { return None; }
        let style = "color: red; color: blue !important; color: green; --Tone: one; --tone: two; pair-first: keep !important; pair: left right";
        Some(StyleSheetRule {
            kind: StyleSheetRuleKind::Style,
            css_text: format!(".target {{ {style} }}"),
            selector_text: Some(".target".into()),
            style_text: Some(style.into()),
            condition_text: None,
            name: None,
            key_text: None,
            child_count: 0,
        })
    }
}

fn retained_rule_declarations<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.set_inline_style_handler(Box::new(PairStyle));
    rt.set_stylesheet_handler(Box::new(RetainedRule));
    rt.eval(r#"
        var s = document.styleSheets[0].cssRules[0].style;
        console.log([s.length, s.item(0), s.item(1), s.item(2), s.item(3), s.item(4)].join('|'));
        console.log([s.getPropertyValue('color'), s.getPropertyPriority('color'), s.getPropertyValue('--Tone'), s.getPropertyValue('--tone')].join('|'));
        console.log([s.getPropertyValue('pair-first'), s.getPropertyPriority('pair-first'), s.getPropertyValue('pair-second'), s.getPropertyValue('pair')].join('|'));
    "#).expect("retained rule script");
    assert_eq!(rt.host().borrow().console, vec![
        "5|color|--Tone|--tone|pair-first|pair-second",
        "blue|important|one|two",
        "keep|important|right|",
    ]);
}

fn declaration_lists<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.load_dom(&StaticDocument::parse("<html><body><div id='d'></div></body></html>"));
    rt.eval(r#"
        var d = document.getElementById('d');
        d.setAttribute('style', '--Theme: "a;b:c" !important; content: "x;y:z"; background-image: url("data:image/svg+xml;a:b"); c\\6flor: blue');
        var s = d.style;
        console.log([s.length, s.item(0), s.item(1), s.item(2), s.item(3)].join('|'));
        console.log([s.getPropertyValue('--Theme'), s.getPropertyPriority('--Theme'), s.getPropertyValue('--theme'), s.getPropertyValue('content'), s.color].join('|'));
        console.log(s.getPropertyValue('background-image'));
        s.cssText = '--MiX: url("data:a;b") !important; content: "v;:w"; width: 1px ! important; --semi: a\\;b';
        console.log([s.length, s.item(0), s.item(1), s.item(2), s.item(3)].join('|'));
        console.log([s.getPropertyValue('--MiX'), s.getPropertyPriority('--MiX'), s.getPropertyValue('content'), s.getPropertyPriority('width'), s.getPropertyValue('--semi')].join('|'));
        console.log(s.cssText);
        s.setProperty('--MiX', 'Case', 'IMPORTANT');
        console.log([s.getPropertyValue('--MiX'), s.getPropertyPriority('--MiX'), s.item(0)].join('|'));
        s.setProperty('width', '2px', 'nonsense');
        console.log(s.getPropertyValue('width'));
    "#).expect("declaration script");
    assert_eq!(rt.host().borrow().console, vec![
        "4|--Theme|content|background-image|color",
        "\"a;b:c\"|important||\"x;y:z\"|blue",
        "url(\"data:image/svg+xml;a:b\")",
        "4|--MiX|content|width|--semi",
        "url(\"data:a;b\")|important|\"v;:w\"|important|a\\;b",
        "--MiX: url(\"data:a;b\") !important; content: \"v;:w\"; width: 1px !important; --semi: a\\;b;",
        "Case|important|--MiX",
        "1px",
    ]);
}

fn malformed_records_are_atomic<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.load_dom(&StaticDocument::parse("<html><body><div id='d' style='color: red'></div></body></html>"));
    rt.set_inline_style_handler(Box::new(PairStyle));

    // Reinstall only the production declaration section inside lexical native
    // hook parameters. Nova's installed global functions are non-writable; this
    // gives both backends the same actual decoder without altering either VM.
    let bootstrap = include_str!("../dom/bootstrap.js");
    let start = bootstrap.find("  function cssKebab(s)").expect("style section start");
    let end = bootstrap[start..].find("  // window.getComputedStyle(el)").expect("style section end") + start;
    let mut script = String::from(r#"
      (function() {
        var nativeParse = __inlineStyleDeclarations;
        var nativeValue = __inlineStyleValue;
        var nativeComponents = __inlineStyleShorthandComponents;
        var nativeShorthands = __inlineStyleShorthands;
        var nativeShorthandValue = __inlineStyleShorthandValue;
        var mode = '', badRecord = '', hits = 0, valueCalls = 0;
        function parseHook(text) {
          if (mode === 'parse') { hits++; return badRecord; }
          return nativeParse(text);
        }
        function valueHook(name, value) {
          if ((mode === 'value' || mode === 'valid-invalid') && ++valueCalls === 2) {
            hits++; return badRecord;
          }
          return nativeValue(name, value);
        }
        function componentsHook(name) {
          if (mode === 'components' && name === 'pair') { hits++; return 'pair-first\\q'; }
          return nativeComponents(name);
        }
        function shorthandsHook() {
          if (mode === 'shorthands') { hits++; return 'pair\\q'; }
          return nativeShorthands();
        }
        function shorthandValueHook(name, values) {
          if (mode === 'reconstruct' && name === 'pair') { hits++; return 'left\\q'; }
          return nativeShorthandValue(name, values);
        }
        (function(__inlineStyleDeclarations, __inlineStyleValue,
                  __inlineStyleShorthandComponents, __inlineStyleShorthands,
                  __inlineStyleShorthandValue) {
    "#);
    script.push_str(&bootstrap[start..end]);
    script.push_str(r#"
        })(parseHook, valueHook, componentsHook, shorthandsHook, shorthandValueHook);
        var d = document.getElementById('d'), s = d.style;
        var original = d.getAttribute('style');
        var bad = ['color\\q\tred\t', 'color\tred', 'color\tred\turgent'];
        for (var i = 0; i < bad.length; i++) {
          mode = 'parse'; badRecord = bad[i]; hits = 0;
          var threw = false;
          try { s.cssText = 'color: blue'; } catch (e) { threw = e instanceof TypeError; }
          mode = '';
          console.log([threw, hits > 0, d.getAttribute('style') === original, s.color === 'red'].join('|'));
        }
        var badValues = ['canonical\nbad\\q', 'unknown\nvalue'];
        for (var j = 0; j < badValues.length; j++) {
          mode = 'value'; badRecord = badValues[j]; valueCalls = 0; hits = 0;
          var threw = false;
          try { s.cssText = 'color: red; width: 2px'; } catch (e) { threw = e instanceof TypeError; }
          mode = '';
          console.log([threw, hits === 1, d.getAttribute('style') === original, s.color === 'red'].join('|'));
        }
        mode = 'components'; hits = 0; threw = false;
        try { s.cssText = 'color: blue; pair: left right'; } catch (e) { threw = e instanceof TypeError; }
        mode = '';
        console.log([threw, hits > 0, d.getAttribute('style') === original, s.color === 'red'].join('|'));
        mode = 'shorthands'; hits = 0; threw = false;
        try { s.cssText = 'color: blue'; } catch (e) { threw = e instanceof TypeError; }
        mode = '';
        console.log([threw, hits > 0, d.getAttribute('style') === original, s.color === 'red'].join('|'));
        mode = 'reconstruct'; hits = 0; threw = false;
        try { s.cssText = 'pair: left right'; } catch (e) { threw = e instanceof TypeError; }
        mode = '';
        console.log([threw, hits > 0, d.getAttribute('style') === original, s.color === 'red'].join('|'));
        mode = 'valid-invalid'; badRecord = 'invalid\n'; valueCalls = 0; hits = 0;
        s.cssText = 'color: blue; width: 2px';
        mode = '';
        console.log([hits === 1, d.getAttribute('style') === 'color: blue;', s.getPropertyValue('width') === '', s.color === 'blue'].join('|'));
      })();
    "#);
    rt.eval(&script).expect("malformed record script");
    assert_eq!(rt.host().borrow().console, vec!["true|true|true|true"; 9]);
}

#[test]
fn declaration_lists_on_boa() {
    declaration_lists::<script_engine_boa::BoaEngine>();
    retained_rule_declarations::<script_engine_boa::BoaEngine>();
    malformed_records_are_atomic::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn declaration_lists_on_nova() {
    declaration_lists::<script_engine_nova::NovaEngine>();
    retained_rule_declarations::<script_engine_nova::NovaEngine>();
    malformed_records_are_atomic::<script_engine_nova::NovaEngine>();
}
