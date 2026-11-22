// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The accessibility contract's serialized form across additive fields.

use document_session_api::{
    DocumentA11yCount, DocumentA11yProjection, DocumentA11yRole, DocumentA11yTable,
};

/// A projection serialized before the `table` field existed.
const OLD_PROJECTION: &str = r#"{
    "revision": 3,
    "support": { "capability": "Partial", "limitations": ["older engine"] },
    "root": 1,
    "nodes": [{
        "id": 1, "parent": null, "children": [], "role": "Cell",
        "name": "a", "value": null, "numeric_value": null,
        "numeric_minimum": null, "numeric_maximum": null, "bounds": null,
        "state": {
            "disabled": false, "hidden": false, "selected": null, "expanded": null,
            "checked": null, "toggled": null, "focused": false, "editable": false,
            "multiline": false, "read_only": false, "required": false, "live": null,
            "orientation": null, "has_popup": null
        },
        "actions": []
    }]
}"#;

#[test]
fn old_projections_default_the_table_field() {
    let projection: DocumentA11yProjection =
        serde_json::from_str(OLD_PROJECTION).expect("old projection deserializes");
    let node = &projection.nodes()[0];
    assert_eq!(node.role, DocumentA11yRole::Cell);
    assert_eq!(node.description, None);
    assert_eq!(node.table, DocumentA11yTable::default());
}

#[test]
fn table_properties_round_trip() {
    let mut projection: DocumentA11yProjection =
        serde_json::from_str(OLD_PROJECTION).expect("old projection deserializes");
    let json = serde_json::to_string(&projection).expect("serializes");
    let mut node = projection.nodes()[0].clone();
    node.role = DocumentA11yRole::GridCell;
    node.table.row_count = Some(DocumentA11yCount::Unknown);
    node.table.column_index = Some(2);
    projection = DocumentA11yProjection::new(
        projection.revision(),
        projection.support().clone(),
        projection.root(),
        vec![node.clone()],
    );
    let again: DocumentA11yProjection =
        serde_json::from_str(&serde_json::to_string(&projection).expect("serializes"))
            .expect("round trip");
    assert_eq!(again.nodes()[0], node);
    assert!(
        json.contains("\"table\""),
        "new projections carry the field"
    );
}
