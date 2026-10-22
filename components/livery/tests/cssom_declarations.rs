// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use livery::cascade::parse_cssom_declarations;

#[test]
fn whole_list_keeps_order_identifiers_and_priority() {
    let declarations = parse_cssom_declarations(
        r#"--MiX: "a;!important:b"; c\6flor: blue !important; --semi: a\;b; content: "x:y;z"/*comment*/"#,
    );
    assert_eq!(declarations.len(), 4);
    assert_eq!(declarations[0].name, "--MiX");
    assert_eq!(declarations[0].value, "\"a;!important:b\"");
    assert!(!declarations[0].important);
    assert_eq!(declarations[1].name, "color");
    assert_eq!(declarations[1].value, "blue");
    assert!(declarations[1].important);
    assert_eq!(declarations[2].value, r"a\;b");
    assert_eq!(declarations[3].value, r#""x:y;z""#);
}
