use super::*;
use document_session_api::session_engine::{HostMutation, HostMutationOp, SessionInput};

const HTML: &str = r#"<!doctype html><style>
body { margin:0; font:20px monospace; }
input,textarea { display:block; margin:24px 0 0 40px; width:240px; height:40px; font:20px monospace; }
</style><div id="owner"><input id="entry" aria-label="editor" value="a😀b" inputmode="email" autocomplete="username"></div>
<textarea aria-label="empty"></textarea><textarea aria-label="multiline">line one
second 😀 line</textarea><input aria-label="password" type="password" value="secret-marker-value">
<input aria-label="readonly" readonly value="unchanged"><div style="height:800px">tail</div>"#;

fn focus(session: &mut dyn DocumentSession<Scene>, label: &str) {
    let projection = session.accessibility_projection().unwrap();
    let field = projection
        .nodes()
        .iter()
        .find(|n| n.name.as_deref() == Some(label))
        .unwrap();
    let request = DocumentA11yActionRequest {
        revision: projection.revision(),
        target: field.id,
        action: DocumentA11yAction::Focus,
        data: None,
    };
    assert!(session.dispatch_accessibility_action(&request));
}

#[test]
fn livery_text_input_geometry_tracks_selection_scroll_zoom_and_invalidation() {
    let mut session = LiverySessionEngine::new(NoFetch)
        .spawn(
            &SessionSpawnRequest::new("https://fixture.test/editor")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(400, 260),
        )
        .unwrap();
    assert!(session.text_input_state().is_none());
    session.try_frame(400, 260).unwrap();
    focus(&mut *session, "editor");
    session.try_frame(400, 260).unwrap();
    session.input(SessionInput::Key {
        key: SessionKey::End,
        state: SessionButtonState::Pressed,
        modifiers: Default::default(),
        repeat: false,
    });
    session.try_frame(400, 260).unwrap();
    let original = session.text_input_state().unwrap();
    assert_eq!(original.element_kind, "input");
    assert_eq!(original.input_type, "text");
    assert_eq!(original.input_mode, "email");
    assert_eq!(original.autocomplete, "username");
    assert_eq!(original.selection_utf16, (4, 4));
    assert!(original.caret.x > 40.0 && original.caret.y >= 24.0);
    for _ in 0..2 {
        session.input(SessionInput::Key {
            key: SessionKey::ArrowLeft,
            state: SessionButtonState::Pressed,
            modifiers: Default::default(),
            repeat: false,
        });
    }
    session.try_frame(400, 260).unwrap();
    let moved = session.text_input_state().unwrap();
    assert_eq!(
        moved.selection_utf16,
        (1, 1),
        "UTF-16 units, not UTF-8 bytes"
    );
    assert!(moved.caret.x < original.caret.x);
    session.input(SessionInput::Ime(SessionIme::Preedit {
        text: "仮".into(),
        selection: Some((0, 3)),
    }));
    session.try_frame(400, 260).unwrap();
    assert_eq!(session.text_input_state().unwrap().selection_utf16, (1, 1));
    session.input(SessionInput::Ime(SessionIme::Commit("é".into())));
    assert!(
        session.text_input_state().is_none(),
        "pending text layout cannot expose old caret geometry"
    );
    session.try_frame(400, 260).unwrap();
    let committed = session.text_input_state().unwrap();
    assert_eq!(committed.selection_utf16, (2, 2));
    session.scroll_by(0.0, 12.0);
    session.try_frame(400, 260).unwrap();
    let scrolled = session.text_input_state().unwrap();
    assert!((scrolled.caret.y - (committed.caret.y - 12.0)).abs() < 0.1);
    session.set_page_zoom(2.0).unwrap();
    assert!(
        session.text_input_state().is_none(),
        "zoom must await its own layout"
    );
    session.try_frame(400, 260).unwrap();
    let zoomed = session.text_input_state().unwrap();
    assert!((zoomed.caret.height - scrolled.caret.height * 2.0).abs() < 1.0);
    session.try_frame(640, 400).unwrap();
    assert!(session.text_input_state().is_some());
    session
        .apply_host_mutations(&[HostMutation::new(
            "entry",
            HostMutationOp::SetAttribute {
                name: "readonly".into(),
                value: String::new(),
            },
        )])
        .unwrap();
    assert!(session.text_input_state().is_none());
    session.try_frame(640, 400).unwrap();
    assert!(session.text_input_state().is_none());
}

#[test]
fn livery_text_input_geometry_handles_empty_password_blur_and_detach() {
    let mut session = LiverySessionEngine::new(NoFetch)
        .spawn(
            &SessionSpawnRequest::new("https://fixture.test/editor")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(640, 600),
        )
        .unwrap();
    session.try_frame(640, 600).unwrap();
    focus(&mut *session, "empty");
    session.try_frame(640, 600).unwrap();
    let empty = session
        .text_input_state()
        .expect("empty textarea has a native caret");
    assert!(empty.multiline);
    assert_eq!(empty.selection_utf16, (0, 0));
    assert!(empty.caret.width > 0.0 && empty.caret.height > 0.0);
    focus(&mut *session, "multiline");
    session.try_frame(640, 600).unwrap();
    let multiline = session.text_input_state().expect("nonempty multiline control has shaped geometry");
    assert!(multiline.multiline);
    assert!(multiline.caret.height > 0.0);
    focus(&mut *session, "password");
    session.try_frame(640, 600).unwrap();
    let password = session.text_input_state().unwrap();
    assert!(password.password);
    assert!(!format!("{password:?}").contains("secret-marker-value"));
    session.input(SessionInput::Focus(false));
    assert!(session.text_input_state().is_none());
    session.try_frame(640, 600).unwrap();
    focus(&mut *session, "editor");
    session.try_frame(640, 600).unwrap();
    assert!(session.text_input_state().is_some());
    session
        .apply_host_mutations(&[HostMutation::new("owner", HostMutationOp::RemoveLastChild)])
        .unwrap();
    assert!(session.text_input_state().is_none());
    session.try_frame(640, 600).unwrap();
    assert!(session.text_input_state().is_none());
}

#[cfg(feature = "scripted")]
#[test]
fn scripted_text_input_geometry_uses_live_utf16_selection_and_rejects_stale_layout() {
    let engine =
        ScriptedSessionEngine::<script_engine_boa::BoaEngine, _>::new("genet.scripted", NoFetch);
    let mut session = engine
        .spawn(
            &SessionSpawnRequest::new("https://fixture.test/editor")
                .with_body(HTML)
                .with_content_type("text/html")
                .with_viewport(640, 600),
        )
        .unwrap();
    session.try_frame(640, 600).unwrap();
    assert!(session.focus_move(SessionFocusDirection::Forward));
    session.try_frame(640, 600).unwrap();
    let before = session.text_input_state().unwrap();
    session.input(SessionInput::Text("x".into()));
    assert!(session.text_input_state().is_none());
    session.try_frame(640, 600).unwrap();
    let after = session.text_input_state().unwrap();
    assert_eq!(after.selection_utf16.0, before.selection_utf16.0 + 1);
    assert!(after.caret.x > before.caret.x);
    session.input(SessionInput::Focus(false));
    assert!(session.text_input_state().is_none());
}
