// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The scripted lane as an inker session engine: a page's DOM after
//! its scripts ran, wrapped for the session registry.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use document_session_api::DocumentCapabilities;
use document_session_api::session_engine::{
    DocumentClip, DocumentSession, SessionClick, SessionEngine, SessionError, SessionLink,
    SessionPendingWork, SessionScrollKey, SessionSpawnRequest, SessionTextTarget,
};
use netrender::Scene;

use super::*;

#[cfg(feature = "scripted")]
type ScriptedOptionsFactory =
    dyn Fn(&str) -> Result<genet_scripted::ScriptedDocumentOptions, String> + Send + Sync;

/// Map the host-neutral scroll-key vocabulary onto the owned scripted lane.
#[cfg(feature = "scripted")]
pub(crate) fn scripted_scroll_key(key: SessionScrollKey) -> genet_scripted::ScrollKey {
    match key {
        SessionScrollKey::LineUp => genet_scripted::ScrollKey::Up,
        SessionScrollKey::LineDown => genet_scripted::ScrollKey::Down,
        SessionScrollKey::PageUp => genet_scripted::ScrollKey::PageUp,
        SessionScrollKey::PageDown => genet_scripted::ScrollKey::PageDown,
        SessionScrollKey::Home => genet_scripted::ScrollKey::Home,
        SessionScrollKey::End => genet_scripted::ScrollKey::End,
    }
}

/// Session engine for the scripted lane, generic over the JS engine `E` (the
/// per-engine monomorphization genet-scripted already uses: the host
/// registers `ScriptedSessionEngine::<BoaEngine, _>` under `genet.scripted`
/// and, on 64-bit targets with the `scripted-nova` feature,
/// `ScriptedSessionEngine::<NovaEngine, _>` under `genet.scripted.nova`).
/// Holds the shell's fetcher for external `<script src>` resolution.
#[cfg(feature = "scripted")]
pub struct ScriptedSessionEngine<E, Fetch> {
    engine_id: String,
    fetcher: Fetch,
    wake: genet_scripted::ScriptWake,
    generation: AtomicU64,
    options_factory: Option<Box<ScriptedOptionsFactory>>,
    _engine: std::marker::PhantomData<fn() -> E>,
}

#[cfg(feature = "scripted")]
impl<E, Fetch> ScriptedSessionEngine<E, Fetch> {
    pub fn new(engine_id: impl Into<String>, fetcher: Fetch) -> Self {
        Self::new_with_wake(engine_id, fetcher, genet_scripted::ScriptWake::new())
    }

    pub fn new_with_wake(
        engine_id: impl Into<String>,
        fetcher: Fetch,
        wake: genet_scripted::ScriptWake,
    ) -> Self {
        Self {
            engine_id: engine_id.into(),
            fetcher,
            wake,
            generation: AtomicU64::new(0),
            options_factory: None,
            _engine: std::marker::PhantomData,
        }
    }

    /// Build fresh document-local capabilities before every navigation's first
    /// authored script. Transport and child-realm routing stay with the
    /// document's retained resource bridge.
    pub fn with_options_factory(
        mut self,
        factory: impl Fn(&str) -> Result<genet_scripted::ScriptedDocumentOptions, String>
        + Send
        + Sync
        + 'static,
    ) -> Self {
        self.options_factory = Some(Box::new(factory));
        self
    }
}

#[cfg(feature = "scripted")]
impl<E, Fetch> SessionEngine<Scene> for ScriptedSessionEngine<E, Fetch>
where
    E: script_engine_api::ScriptEngine + 'static,
    Fetch: genet_scripted::ResourceFetcher + Clone + Send + Sync + 'static,
{
    fn engine_id(&self) -> &str {
        &self.engine_id
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<Scene>>, SessionError> {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        let navigation = genet_livery::NavigationFragment::parse(&request.address);
        let options = match &self.options_factory {
            Some(factory) => {
                factory(&navigation.script_visible_url).map_err(SessionError::SpawnFailed)?
            },
            None => genet_scripted::ScriptedDocumentOptions::default(),
        };
        let doc = match &request.body {
            Some(body) => {
                genet_scripted::LiveryScriptedDocument::<E>::from_body_with_options_and_wake_generation(
                    body,
                    self.fetcher.clone(),
                    &request.address,
                    self.wake.clone(),
                    generation,
                    options,
                )
            },
            None => genet_scripted::LiveryScriptedDocument::<E>::load_with_options_and_wake_generation(
                self.fetcher.clone(),
                &request.address,
                self.wake.clone(),
                generation,
                options,
            ),
        }
        .map_err(SessionError::SpawnFailed)?;
        let mut session = ScriptedDocumentSession {
            doc,
            address: navigation.script_visible_url,
            pressed_target: None,
            pointer_active: false,
            pending_key_text: None,
            external_textures: Vec::new(),
            collection_stats: (0, 0),
            a11y_revision: std::cell::Cell::new(0),
            a11y_cache: std::cell::RefCell::new(None),
            validation_alert_id: std::cell::Cell::new(None),
        };
        if request.hidden {
            session.doc.set_hidden(true);
        }
        Ok(Box::new(session))
    }
}

/// The scripted document as a session. Public so a host with richer
/// construction seams (per-spawn fetchers, cookie jars) builds the document
/// itself and wraps it; the engine above is the simple-seam path.
#[cfg(feature = "scripted")]
pub struct ScriptedDocumentSession<E: script_engine_api::ScriptEngine> {
    doc: genet_scripted::LiveryScriptedDocument<E>,
    address: String,
    pressed_target: Option<genet_scripted_dom::NodeId>,
    pointer_active: bool,
    /// Whether the Text item paired with the last keydown may insert text.
    pending_key_text: Option<bool>,
    /// Cumulative `(reflectors_unpinned, nodes_collected)` from normal frame
    /// pumps. Reading this never triggers another collection.
    collection_stats: (usize, usize),
    external_textures: Vec<document_session_api::SessionExternalTextureDraw>,
    /// Monotonic revision for the published accessibility projection, and the
    /// last projection published under it. Same discipline as the Livery
    /// session: the revision advances only when the content actually changed,
    /// so a host may compare revisions to decide whether to republish.
    a11y_revision: std::cell::Cell<u64>,
    a11y_cache: std::cell::RefCell<Option<document_session_api::DocumentA11yProjection>>,
    /// Identity belongs to the native notice, independently of arena node IDs.
    validation_alert_id: std::cell::Cell<Option<(u64, document_session_api::DocumentA11yNodeId)>>,
}

#[cfg(feature = "scripted")]
impl<E: script_engine_api::ScriptEngine + 'static> ScriptedDocumentSession<E> {
    pub fn new(doc: genet_scripted::LiveryScriptedDocument<E>) -> Self {
        Self::new_at(doc, "about:blank")
    }

    pub fn new_at(
        doc: genet_scripted::LiveryScriptedDocument<E>,
        address: impl Into<String>,
    ) -> Self {
        Self {
            doc,
            address: address.into(),
            pressed_target: None,
            pointer_active: false,
            pending_key_text: None,
            collection_stats: (0, 0),
            external_textures: Vec::new(),
            a11y_revision: std::cell::Cell::new(0),
            a11y_cache: std::cell::RefCell::new(None),
            validation_alert_id: std::cell::Cell::new(None),
        }
    }

    /// The neutral accessibility projection of the **top** browsing context's
    /// document, built off the retained layout of the last rendered frame.
    ///
    /// `Click` is advertised only for a node whose retained fragment actually
    /// intersects the presented viewport, which is the same on-screen rule the
    /// static Livery session reaches through `accessible_pointer_target`. A
    /// receipt that states its window in physical pixels on a scaled display
    /// therefore sees exactly the links a person could click.
    ///
    /// A composited child frame keeps its own arena and its own retained
    /// layout; this projection covers the top document alone, so a node that
    /// has been adopted away into a child is absent here rather than reparented.
    fn unrevisioned_accessibility_projection(
        &self,
    ) -> Option<document_session_api::DocumentA11yProjection> {
        use document_session_api::{DocumentA11yAction, DocumentA11yProjection};

        let projection = self.doc.with_retained_styled_frame_and_dom(
            |dom, styles, fragments, scroll, viewport| {
                let projection = genet_render::document_a11y_projection_with_style(
                    dom,
                    fragments,
                    None,
                    0,
                    Some(&std::collections::HashMap::new()),
                    &genet_render::A11yStyleQueries {
                        generated: &|node| genet_livery::rendered_generated_text(dom, styles, node),
                        rendered: &|node| genet_livery::rendered_visible(dom, styles, node),
                    },
                );
                let (scroll_x, scroll_y) = scroll;
                let (view_w, view_h) = (viewport.0 as f32, viewport.1 as f32);
                let nodes = projection
                    .nodes()
                    .iter()
                    .cloned()
                    .map(|mut node| {
                        let on_screen = node.bounds.as_ref().is_some_and(|bounds| {
                            let x = bounds.x - scroll_x;
                            let y = bounds.y - scroll_y;
                            bounds.width > 0.0
                                && bounds.height > 0.0
                                && x < view_w
                                && y < view_h
                                && x + bounds.width > 0.0
                                && y + bounds.height > 0.0
                        });
                        if node.state.disabled || node.state.hidden || !on_screen {
                            node.actions
                                .retain(|action| *action != DocumentA11yAction::Click);
                        }
                        if let Some(bounds) = node.bounds.as_mut() {
                            bounds.x -= scroll_x;
                            bounds.y -= scroll_y;
                        }
                        node
                    })
                    .collect();
                DocumentA11yProjection::new(
                    0,
                    projection.support().clone(),
                    projection.root(),
                    nodes,
                )
            },
        )?;
        Some(self.with_validation_alert(projection))
    }

    fn with_validation_alert(
        &self,
        projection: document_session_api::DocumentA11yProjection,
    ) -> document_session_api::DocumentA11yProjection {
        super::with_validation_alert(projection, self.doc.validation_notice().map(|notice|
            (notice.sequence, notice.message.as_str(), notice.bounds)), &self.validation_alert_id)
    }

    fn current_accessibility_projection(
        &self,
    ) -> Option<document_session_api::DocumentA11yProjection> {
        let fresh = self.unrevisioned_accessibility_projection()?;
        let unchanged = self.a11y_cache.borrow().as_ref().is_some_and(|cached| {
            cached.root() == fresh.root()
                && cached.support() == fresh.support()
                && cached.nodes() == fresh.nodes()
        });
        if unchanged {
            return self.a11y_cache.borrow().clone();
        }
        let revision = self.a11y_revision.get().saturating_add(1).max(1);
        let current = document_session_api::DocumentA11yProjection::new(
            revision,
            fresh.support().clone(),
            fresh.root(),
            fresh.nodes().to_vec(),
        );
        self.a11y_revision.set(revision);
        *self.a11y_cache.borrow_mut() = Some(current.clone());
        Some(current)
    }
}

#[cfg(feature = "scripted")]
impl<E: script_engine_api::ScriptEngine + 'static> DocumentSession<Scene>
    for ScriptedDocumentSession<E>
{
    fn current_address(&self) -> Option<String> { Some(self.doc.current_document_address()) }
    fn document_capabilities(&self) -> DocumentCapabilities {
        retained_document_capabilities("scripted sessions do not expose document find")
    }

    fn frame(&mut self, width: u32, height: u32) -> Scene {
        let frame = self.doc.frame_with_external_textures(width, height);
        self.external_textures = frame
            .external_textures
            .into_iter()
            .map(|draw| document_session_api::SessionExternalTextureDraw {
                texture_key: draw.texture_key,
                dest_rect: draw.dest_rect,
                opacity: draw.opacity,
                scene_op_boundary: draw.scene_op_boundary,
            })
            .collect();
        frame.scene
    }
    fn external_texture_draws(&self) -> &[document_session_api::SessionExternalTextureDraw] {
        &self.external_textures
    }
    fn scroll_by(&mut self, dx: f32, dy: f32) -> bool {
        self.doc.scroll_by(dx, dy)
    }
    fn scroll_for_key(&mut self, key: SessionScrollKey) -> bool {
        self.doc.scroll_for_key(scripted_scroll_key(key))
    }
    fn click_at(&mut self, x: f32, y: f32) -> SessionClick {
        match self.doc.click_at_result(x, y) {
            genet_scripted::ScriptedClick::Miss => SessionClick::Miss,
            genet_scripted::ScriptedClick::Handled => SessionClick::Handled,
            genet_scripted::ScriptedClick::Navigate(target) => SessionClick::Navigate(target),
        }
    }
    fn pointer_down(&mut self, x: f32, y: f32) -> SessionClick {
        self.pending_key_text = None;
        let pressed_target = self.doc.click_target_at(x, y);
        if let Some(node) = pressed_target { let _ = self.doc.focus_host_node(node); }
        self.pressed_target = pressed_target;
        self.pointer_active = self.doc.begin_text_selection(x, y) || pressed_target.is_some();
        if self.pointer_active {
            SessionClick::Handled
        } else {
            SessionClick::Miss
        }
    }
    fn pointer_move(&mut self, x: f32, y: f32) -> bool {
        self.doc.extend_text_selection(x, y)
    }
    fn pointer_up(&mut self, x: f32, y: f32) -> SessionClick {
        if !std::mem::replace(&mut self.pointer_active, false) {
            self.pressed_target = None;
            return SessionClick::Miss;
        }
        let pressed_target = self.pressed_target.take();
        if self.doc.finish_text_selection(x, y) {
            SessionClick::Handled
        } else if pressed_target.is_some() && self.doc.click_target_at(x, y) == pressed_target {
            self.click_at(x, y)
        } else {
            // The press was captured; releasing outside consumes it without
            // activating a different target or leaking the gesture to a host.
            SessionClick::Handled
        }
    }
    fn focus_input(&mut self, focused: bool) {
        if !focused { self.doc.clear_host_focus(); self.pending_key_text = None; }
        if !focused && self.pointer_active {
            self.pointer_active = false;
            self.pressed_target = None;
            let _ = self.doc.cancel_text_selection();
        }
    }
    fn focus_move(&mut self, direction: document_session_api::SessionFocusDirection) -> bool {
        self.pending_key_text = None;
        self.doc.move_host_focus(direction == document_session_api::SessionFocusDirection::Backward)
    }
    fn cancel_input(&mut self) -> bool {
        self.pending_key_text = None;
        let had_pointer = std::mem::replace(&mut self.pointer_active, false)
            || self.pressed_target.take().is_some();
        self.pressed_target = None;
        let dismissed = self.doc.dismiss_validation_notice();
        let composition = self.doc.cancel_host_preedit();
        self.doc.cancel_text_selection() || had_pointer || dismissed || composition
    }
    fn key_input(
        &mut self,
        key: document_session_api::SessionKey,
        state: document_session_api::SessionButtonState,
        modifiers: document_session_api::SessionModifiers,
        repeat: bool,
    ) -> document_session_api::SessionEffect {
        use document_session_api::{SessionButtonState, SessionEffect, SessionKey};
        let name = match &key {
            SessionKey::Character(value) => value.as_str(), SessionKey::Enter => "Enter",
            SessionKey::Space => " ", SessionKey::Escape => "Escape", SessionKey::Backspace => "Backspace",
            SessionKey::Delete => "Delete", SessionKey::ArrowLeft => "ArrowLeft", SessionKey::ArrowRight => "ArrowRight",
            SessionKey::ArrowUp => "ArrowUp", SessionKey::ArrowDown => "ArrowDown", SessionKey::Home => "Home",
            SessionKey::End => "End", SessionKey::PageUp => "PageUp", SessionKey::PageDown => "PageDown",
            SessionKey::Tab => "Tab", SessionKey::Unidentified => "Unidentified",
        };
        let had_focus = self.doc.host_has_focus();
        if state == SessionButtonState::Released { self.pending_key_text = None; }
        let cancelled = self.doc.dispatch_host_key(name, state == SessionButtonState::Pressed, repeat,
            modifiers.alt, modifiers.control, modifiers.meta, modifiers.shift);
        if state == SessionButtonState::Released {
            self.doc.finish_host_key();
            return if cancelled { SessionEffect::Cancelled } else { SessionEffect::Ignored };
        }
        if cancelled {
            self.pending_key_text = matches!(&key,SessionKey::Character(_) | SessionKey::Space).then_some(false);
            self.doc.finish_host_key();
            return SessionEffect::Cancelled;
        }
        let unmodified = !modifiers.alt && !modifiers.control && !modifiers.meta;
        self.pending_key_text = match &key {
            SessionKey::Character(_) | SessionKey::Space => Some(!self.doc.host_has_preedit()),
            _ => None,
        };
        let escape = matches!(&key, SessionKey::Escape);
        let handled = match key {
            SessionKey::Tab => self.doc.move_host_focus(modifiers.shift),
            SessionKey::Escape => self.cancel_input(),
            SessionKey::Enter if unmodified && self.doc.host_focused_is_textarea() =>
                self.doc.host_text_input("insert", "\n", "insertLineBreak"),
            SessionKey::Enter => had_focus,
            SessionKey::Character(_) | SessionKey::Space => false,
            SessionKey::Backspace if unmodified => self.doc.host_text_input("backward", "", "deleteContentBackward"),
            SessionKey::Delete if unmodified => self.doc.host_text_input("forward", "", "deleteContentForward"),
            SessionKey::ArrowLeft if unmodified => self.doc.host_text_input("left", "", ""),
            SessionKey::ArrowRight if unmodified => self.doc.host_text_input("right", "", ""),
            SessionKey::Home if unmodified => self.doc.host_text_input("home", "", ""),
            SessionKey::End if unmodified => self.doc.host_text_input("end", "", ""),
            _ => false,
        };
        self.doc.finish_host_key();
        if handled && escape { SessionEffect::Cancelled }
        else if handled { SessionEffect::Handled } else { SessionEffect::Ignored }
    }

    fn keyboard_text_input(&mut self, text: &str) -> bool {
        if self.pending_key_text.take() == Some(false) { return false; }
        self.text_input(text)
    }

    fn text_input(&mut self, text: &str) -> bool {
        !text.is_empty() && self.doc.host_text_input("insert", text, "insertText")
    }

    fn ime_input(&mut self, ime: document_session_api::SessionIme) -> bool {
        use document_session_api::SessionIme;
        self.pending_key_text = None;
        match ime {
            SessionIme::Enabled => self.doc.host_editable_focus(),
            SessionIme::Preedit { text, .. } => {
                let phase = if self.doc.host_has_preedit() { "compositionupdate" } else { "compositionstart" };
                self.doc.host_composition(phase, &text)
            },
            SessionIme::Commit(text) => {
                let composing = self.doc.host_has_preedit();
                let accepted = !text.is_empty() && self.doc.host_text_input("insert", &text,
                    if composing { "insertCompositionText" } else { "insertText" });
                if composing { let _ = self.doc.host_composition("compositionend", &text); }
                accepted
            },
            SessionIme::Disabled => {
                if self.doc.host_has_preedit() { self.doc.host_composition("compositionend", "") } else { false }
            },
        }
    }

    fn editable_focus(&self) -> bool { self.doc.host_editable_focus() }

    fn text_target(&self, text: &str) -> Option<SessionTextTarget> {
        let (anchor, focus) = self.doc.text_target(text)?;
        Some(SessionTextTarget { anchor, focus })
    }
    fn links(&self) -> Vec<SessionLink> {
        self.doc
            .links()
            .into_iter()
            .map(|(url, rect)| SessionLink { url, rect })
            .collect()
    }
    fn pump(&mut self, now_ms: f64) {
        let (unpinned, collected) = self.doc.pump(now_ms);
        self.collection_stats.0 += unpinned;
        self.collection_stats.1 += collected;
    }
    fn collection_stats(&self) -> (usize, usize) {
        self.collection_stats
    }
    fn pending_work(&mut self) -> SessionPendingWork {
        // The session's deadline field is also the wake for a queued rAF;
        // `next_wake_delay` returns zero for the next visible rendering turn.
        let wake = self
            .doc
            .next_wake_delay()
            .and_then(|delay| delay.is_finite().then_some(delay.max(0.0)))
            .map(|delay| Duration::from_secs_f64(delay / 1000.0));
        let external = self.doc.has_external_work();
        SessionPendingWork {
            timer: wake,
            microtasks: false,
            external,
        }
    }
    fn settled(&mut self) -> bool {
        self.pending_work().is_settled()
    }
    fn set_hidden(&mut self, hidden: bool) {
        self.doc.set_hidden(hidden);
    }
    fn supports_freeze_resume(&self) -> bool {
        true
    }
    fn freeze(&mut self) {
        self.doc.freeze();
    }
    fn resume(&mut self) {
        self.doc.resume();
    }
    fn inspect(&self) -> Option<document_session_api::ContentReport> {
        Some(self.doc.with_dom(content_report))
    }

    /// The scripted lane publishes a projection of the top document. Action
    /// dispatch and click-target revalidation stay unimplemented here: they
    /// need a live pointer target for a scripted document, which is the
    /// Livery-session seam (`accessible_pointer_target`) and not this lane's.
    fn accessibility_projection(&self) -> Option<document_session_api::DocumentA11yProjection> {
        self.current_accessibility_projection()
    }
    fn clip(&self) -> Option<DocumentClip> {
        let selection = self.doc.text_selection();
        let address = self.doc.current_document_address();
        let resolution_base = self.doc.document_resolution_base().unwrap_or_else(|| address.clone());
        self.doc.with_dom(|dom| match selection {
            Some(selection) => {
                let selected_links = links_for_source_nodes(dom, &selection.source_nodes);
                semantic_clip_from_selection_with_links(
                    &address,
                    dom,
                    ClipSelection {
                        range: ClipRange {
                            anchor_node: selection.range.anchor_node,
                            anchor_offset: selection.range.anchor_offset,
                            focus_node: selection.range.focus_node,
                            focus_offset: selection.range.focus_offset,
                        },
                        text: selection.text,
                    },
                    selected_links,
                )
            },
            None => semantic_clip_from_dom(&address, dom).map(|mut clip| {
                for href in &mut clip.links {
                    *href = genet_scripted::resolve_href(&resolution_base, href);
                }
                clip
            }),
        })
    }
    /// Observation extras (extract, dom_snapshot, dispatch_event, dom stats)
    /// stay on the concrete type until the observation contract lands
    /// (session-engines plan phase 3 rescope); hosts reach them here.
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(feature = "scripted")]
impl<E: script_engine_api::ScriptEngine> ScriptedDocumentSession<E> {
    /// The concrete document, for observation downcasts (phase 3 rescope:
    /// extract / dom_snapshot / dispatch_event stay concrete until the
    /// observation contract lands).
    pub fn document_mut(&mut self) -> &mut genet_scripted::LiveryScriptedDocument<E> {
        &mut self.doc
    }
}

#[cfg(all(test, feature = "scripted"))]
mod animation_frame_pending_tests {
    use super::*;

    fn raf_only_session_wakes<E: script_engine_api::ScriptEngine + 'static>() {
        let doc = genet_scripted::LiveryScriptedDocument::<E>::parse(
            "<body><script>requestAnimationFrame(function() {\
               document.body.setAttribute('data-frame', 'done');\
             });</script></body>",
        )
        .expect("scripted document");
        let mut session = ScriptedDocumentSession::new(doc);
        let pending = session.pending_work();
        assert_eq!(pending.timer, Some(Duration::ZERO));
        assert!(!pending.external);
        assert!(!pending.is_idle(), "rAF wakes the session host");
        assert!(session.settled(), "a future rendering turn does not hold a completion receipt open");
        session.pump(16.0);
        assert!(
            session
                .document_mut()
                .dom_snapshot()
                .contains("data-frame=\"done\"")
        );
        assert!(session.pending_work().is_settled());
    }

    fn host_lifecycle_controls_scripted_session<E: script_engine_api::ScriptEngine + 'static>() {
        let doc = genet_scripted::LiveryScriptedDocument::<E>::parse(
            "<body><script>\
             document.addEventListener('visibilitychange', () => console.log('visibility'));\
             document.addEventListener('freeze', () => console.log('freeze'));\
             document.addEventListener('resume', () => console.log('resume'));\
             setInterval(() => console.log('tick'), 16);\
             </script></body>",
        ).expect("scripted document");
        let mut host_session: Box<dyn DocumentSession<Scene>> =
            Box::new(ScriptedDocumentSession::new(doc));
        assert!(host_session.supports_freeze_resume());
        host_session.set_hidden(true);
        assert_eq!(host_session.pending_work().timer, Some(Duration::ZERO));
        host_session.pump(0.0);
        assert_eq!(host_session.pending_work().timer, Some(Duration::from_secs(1)));
        host_session.freeze();
        assert!(host_session.pending_work().is_idle());
        host_session.pump(1000.0);
        let live = host_session.as_any().downcast_mut::<ScriptedDocumentSession<E>>().unwrap();
        assert!(live.document_mut().is_hidden());
        assert!(live.document_mut().is_frozen());
        assert_eq!(live.document_mut().console(), vec!["visibility", "freeze"]);
        host_session.resume();
        assert_eq!(host_session.pending_work().timer, Some(Duration::ZERO));
        host_session.pump(1000.0);
        assert_eq!(host_session.pending_work().timer, Some(Duration::from_secs(1)));
        host_session.set_hidden(false);
        host_session.pump(1016.0);
        let live = host_session.as_any().downcast_mut::<ScriptedDocumentSession<E>>().unwrap();
        assert!(!live.document_mut().is_hidden());
        assert!(!live.document_mut().is_frozen());
        assert_eq!(live.document_mut().console(), vec!["visibility", "freeze", "resume", "visibility", "tick"]);
    }

    #[test]
    fn raf_only_session_wakes_on_boa() {
        raf_only_session_wakes::<script_engine_boa::BoaEngine>();
    }

    #[test]
    fn host_lifecycle_controls_scripted_session_on_boa() {
        host_lifecycle_controls_scripted_session::<script_engine_boa::BoaEngine>();
    }

    #[cfg(all(target_pointer_width = "64", feature = "scripted-nova"))]
    #[test]
    fn raf_only_session_wakes_on_nova() {
        raf_only_session_wakes::<script_engine_nova::NovaEngine>();
    }

    #[cfg(all(target_pointer_width = "64", feature = "scripted-nova"))]
    #[test]
    fn host_lifecycle_controls_scripted_session_on_nova() {
        host_lifecycle_controls_scripted_session::<script_engine_nova::NovaEngine>();
    }
}
