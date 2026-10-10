/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Platform AccessKit bridge for a genet-on-winit host.
//!
//! A host builds its accessibility tree however it likes (Genet's
//! [`genet_render::accesskit_tree`] projects a laid-out DOM; a host may stitch
//! several such subtrees) and hands the resulting [`TreeUpdate`] to
//! [`AccessKitBridge::update`]. The bridge owns the per-platform adapter
//! (`accesskit_windows` / `accesskit_macos` / `accesskit_unix`), pushes the tree
//! to the OS a11y API, and queues screen-reader [`ActionRequest`]s for the host to
//! drain and route back through its own activation paths. It is deliberately
//! host-agnostic: it knows about a winit [`Window`], a `TreeUpdate`, and a wake
//! callback, nothing about any particular host's panes or verbs.
//!
//! # Window lifecycle
//!
//! On Windows, [`AccessKitBridge::install`] creates a native subclassing
//! adapter and must run before the window is shown for the first time. Build an
//! initial tree while the window is hidden, install it, and then reveal the
//! window. Calling only [`AccessKitBridge::new`] is not sufficient, and hidden
//! winit windows may not receive the redraw that a deferred install expects.
//!
//! [`genet_render::accesskit_tree`]: https://docs.rs/genet-render

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(all(unix, not(target_os = "macos")))]
use accesskit::DeactivationHandler;
use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, NodeId as AccessNodeId,
    TreeUpdate,
};

/// Whether the OS-level AccessKit adapter is live for this window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeStatus {
    Unavailable,
    Installed,
}

/// A screen reader's request to act on a node: the [`Action`], target node id,
/// and optional action data. The host maps the id back to its own element and
/// routes the action through its activation path.
#[derive(Clone, Debug, PartialEq)]
pub struct A11yActionRequest {
    pub action: Action,
    pub target_tree: accesskit::TreeId,
    pub target_node: AccessNodeId,
    pub data: Option<ActionData>,
}

type ActionSink = Arc<dyn Fn(A11yActionRequest) + Send + Sync>;

/// A complete publication, with a safe host-only activation entry point.
#[derive(Default)]
struct A11yPublication {
    initial: Option<TreeUpdate>,
    ordered: Vec<TreeUpdate>,
    expected_epoch: Option<u64>,
}

impl A11yPublication {
    fn initial(&self) -> Option<TreeUpdate> {
        self.initial.clone()
    }
    fn replay(&self) -> Vec<TreeUpdate> {
        self.ordered.clone()
    }

    fn replace(&mut self, initial: TreeUpdate, ordered: Vec<TreeUpdate>) -> Result<(), String> {
        let Some(tree) = initial.tree.as_ref() else {
            return Err("activation requires a complete host root tree".into());
        };
        if initial.tree_id != accesskit::TreeId::ROOT
            || initial.focus != tree.root
            || initial
                .nodes
                .iter()
                .any(|(_, node)| node.tree_id().is_some())
            || !initial
                .nodes
                .iter()
                .any(|(id, node)| *id == tree.root && node.tree_id().is_none())
        {
            return Err("activation must focus the host root without entering a guest tree".into());
        }
        if ordered.is_empty() || ordered[0].tree_id != accesskit::TreeId::ROOT {
            return Err("ordered publication must begin with its host tree".into());
        }
        self.initial = Some(initial);
        self.ordered = ordered;
        self.expected_epoch = None;
        Ok(())
    }

    // Single-update callers own only the host tree. They cannot publish guest
    // references without the complete ordered guest installation protocol.
    fn replace_host(&mut self, mut update: TreeUpdate) -> bool {
        if update.tree_id != accesskit::TreeId::ROOT {
            return false;
        }
        let Some(root) = update.tree.as_ref().map(|tree| tree.root) else {
            return false;
        };
        for (_, node) in &mut update.nodes {
            node.clear_tree_id();
        }
        let mut initial = update.clone();
        initial.focus = root;
        self.replace(initial, vec![update]).is_ok()
    }
}

fn advance_epoch(epoch: &AtomicU64) {
    epoch
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
            value.checked_add(1)
        })
        .expect("accessibility activation epoch exhausted");
}

struct Activation {
    epoch: Arc<AtomicU64>,
    publication: Arc<Mutex<A11yPublication>>,
    replay: Arc<std::sync::atomic::AtomicBool>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl ActivationHandler for Activation {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        advance_epoch(&self.epoch);
        let initial = self
            .publication
            .lock()
            .ok()
            .and_then(|state| state.initial());
        self.replay
            .store(true, std::sync::atomic::Ordering::Release);
        (self.wake)();
        initial
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
struct Deactivation {
    epoch: Arc<AtomicU64>,
    replay: Arc<std::sync::atomic::AtomicBool>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

#[cfg(all(unix, not(target_os = "macos")))]
impl DeactivationHandler for Deactivation {
    fn deactivate_accessibility(&mut self) {
        advance_epoch(&self.epoch);
        self.replay
            .store(true, std::sync::atomic::Ordering::Release);
        (self.wake)();
    }
}

struct QueuedActions {
    actions: Arc<Mutex<VecDeque<A11yActionRequest>>>,
    wake: Arc<dyn Fn() + Send + Sync>,
    sink: Option<ActionSink>,
}

impl QueuedActions {
    fn with_sink(
        actions: Arc<Mutex<VecDeque<A11yActionRequest>>>,
        wake: Arc<dyn Fn() + Send + Sync>,
        sink: ActionSink,
    ) -> Self {
        Self {
            actions,
            wake,
            sink: Some(sink),
        }
    }
}

impl ActionHandler for QueuedActions {
    fn do_action(&mut self, request: ActionRequest) {
        let request = A11yActionRequest {
            action: request.action,
            target_tree: request.target_tree,
            target_node: request.target_node,
            data: request.data,
        };
        if let Some(sink) = &self.sink {
            sink(request);
        } else if let Ok(mut actions) = self.actions.lock() {
            actions.push_back(request);
        }
        (self.wake)();
    }
}

struct BridgeState {
    epoch: Arc<AtomicU64>,
    publication: Arc<Mutex<A11yPublication>>,
    replay: Arc<std::sync::atomic::AtomicBool>,
    delivered: bool,
    actions: Arc<Mutex<VecDeque<A11yActionRequest>>>,
    wake: Arc<dyn Fn() + Send + Sync>,
    sink: Option<ActionSink>,
}

impl BridgeState {
    fn new(wake: Arc<dyn Fn() + Send + Sync>, sink: Option<ActionSink>) -> Self {
        Self {
            epoch: Arc::new(AtomicU64::new(0)),
            publication: Arc::new(Mutex::new(A11yPublication::default())),
            replay: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            delivered: false,
            actions: Arc::new(Mutex::new(VecDeque::new())),
            wake,
            sink,
        }
    }
    fn activation(&self) -> Activation {
        Activation {
            epoch: self.epoch.clone(),
            publication: self.publication.clone(),
            replay: self.replay.clone(),
            wake: self.wake.clone(),
        }
    }
    fn action_handler(&self) -> QueuedActions {
        match &self.sink {
            Some(sink) => {
                QueuedActions::with_sink(self.actions.clone(), self.wake.clone(), sink.clone())
            },
            None => QueuedActions {
                actions: self.actions.clone(),
                wake: self.wake.clone(),
                sink: None,
            },
        }
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    fn deactivation(&self) -> Deactivation {
        Deactivation {
            epoch: self.epoch.clone(),
            replay: self.replay.clone(),
            wake: self.wake.clone(),
        }
    }
    fn replace_host(&mut self, update: TreeUpdate) -> bool {
        let accepted = self
            .publication
            .lock()
            .is_ok_and(|mut state| state.replace_host(update));
        if accepted {
            self.delivered = false;
            self.replay
                .store(true, std::sync::atomic::Ordering::Release);
        }
        accepted
    }
    fn replace_batch(
        &mut self,
        initial: TreeUpdate,
        ordered: Vec<TreeUpdate>,
    ) -> Result<(), String> {
        self.publication
            .lock()
            .map_err(|e| e.to_string())?
            .replace(initial, ordered)?;
        self.delivered = false;
        self.replay
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }
    fn activation_epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }
    fn replace_batch_at(
        &mut self,
        expected_epoch: u64,
        initial: TreeUpdate,
        ordered: Vec<TreeUpdate>,
    ) -> Result<(), String> {
        if self.activation_epoch() != expected_epoch {
            return Err("activation epoch changed".into());
        }
        {
            let mut publication = self.publication.lock().map_err(|e| e.to_string())?;
            // Activation can begin while replacement waits for publication custody.
            if self.activation_epoch() != expected_epoch {
                return Err("activation epoch changed".into());
            }
            publication.replace(initial, ordered)?;
            publication.expected_epoch = Some(expected_epoch);
        }
        self.delivered = false;
        self.replay.store(true, Ordering::Release);
        if self.activation_epoch() != expected_epoch {
            return Err("activation epoch changed".into());
        }
        Ok(())
    }
    // The adapter calls the supplier from INSIDE update_if_active. Some
    // adapters activate synchronously before calling that closure; checking
    // only before update_if_active would hand them a stale guest-tree delta.
    fn flush(&mut self, mut deliver: impl FnMut(&mut dyn FnMut() -> TreeUpdate) -> bool) -> bool {
        let delivery_epoch = self.activation_epoch();
        let Some((Some(safe_root), updates, expected_epoch)) = self
            .publication
            .lock()
            .ok()
            .map(|p| (p.initial(), p.replay(), p.expected_epoch))
        else {
            return false;
        };
        if expected_epoch.is_some_and(|expected| expected != delivery_epoch) {
            self.delivered = false;
            return false;
        }
        if !self.replay.swap(false, Ordering::AcqRel) {
            return self.delivered;
        }
        self.delivered = false;
        if updates.is_empty() {
            return false;
        }
        for update in updates {
            if self.activation_epoch() != delivery_epoch {
                self.replay.store(true, Ordering::Release);
                return false;
            }
            let mut supplied = false;
            let mut current = false;
            let mut supplier = || {
                supplied = true;
                current = self.activation_epoch() == delivery_epoch;
                if current {
                    update.clone()
                } else {
                    safe_root.clone()
                }
            };
            let accepted = deliver(&mut supplier);
            if !accepted
                || !supplied
                || !current
                || self.activation_epoch() != delivery_epoch
                || self.replay.load(Ordering::Acquire)
            {
                self.replay.store(true, Ordering::Release);
                return false;
            }
        }
        self.delivered = true;
        true
    }
    fn drain_actions(&mut self) -> Vec<A11yActionRequest> {
        self.actions
            .lock()
            .map(|mut actions| actions.drain(..).collect())
            .unwrap_or_default()
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use accesskit::TreeUpdate;
    use accesskit_windows::{HWND, SubclassingAdapter};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::window::Window;

    use super::{A11yActionRequest, BridgeStatus};
    use std::sync::Arc;

    pub struct AccessKitBridge {
        adapter: Option<SubclassingAdapter>,
        state: super::BridgeState,
    }

    impl AccessKitBridge {
        pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
            Self {
                adapter: None,
                state: super::BridgeState::new(Arc::new(wake), None),
            }
        }

        /// Capture action admission synchronously before the wake callback.
        pub fn with_action_handler(
            wake: impl Fn() + Send + Sync + 'static,
            handler: impl Fn(A11yActionRequest) + Send + Sync + 'static,
        ) -> Self {
            let mut bridge = Self::new(wake);
            bridge.state.sink = Some(std::sync::Arc::new(handler));
            bridge
        }

        pub fn status(&self) -> BridgeStatus {
            if self.adapter.is_some() {
                BridgeStatus::Installed
            } else {
                BridgeStatus::Unavailable
            }
        }

        pub fn install(&mut self, window: &Window, initial: TreeUpdate) -> Result<(), String> {
            if !self.state.replace_host(initial) {
                return Err("installation requires a complete host root tree".into());
            }
            if self.adapter.is_some() {
                return Ok(());
            }
            let hwnd = match window
                .window_handle()
                .map_err(|err| err.to_string())?
                .as_raw()
            {
                RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as *mut _),
                RawWindowHandle::WinRt(_) => {
                    return Err("WinRT window handles are not supported".to_string());
                },
                _ => return Err("window is not backed by a Win32 HWND".to_string()),
            };
            let adapter =
                SubclassingAdapter::new(hwnd, self.state.activation(), self.state.action_handler());
            self.adapter = Some(adapter);
            Ok(())
        }

        pub fn update(&mut self, update: TreeUpdate) {
            if self.state.replace_host(update) {
                self.flush_pending();
            }
        }

        /// Retain the complete ordered publication. The initial snapshot must
        /// focus the host root; guest trees are installed by the ordered replay.
        /// The whole ordered vector must remain valid starting from that safe
        /// initial root after any adapter reset. Epoch-dependent steady-state
        /// deltas must use [`Self::update_batch_at`] instead.
        /// Returns true only after the complete batch was served without reset.
        pub fn update_batch(
            &mut self,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            self.state.replace_batch(initial, ordered)?;
            Ok(self.flush_pending())
        }

        /// Epoch of the current native activation lifecycle. Compose a fresh
        /// host publication whenever this value changes.
        pub fn activation_epoch(&self) -> u64 {
            self.state.activation_epoch()
        }

        /// Publish updates composed for one activation lifecycle. Unlike
        /// update_batch, these updates may depend on that epoch's guest trees.
        /// False asks the host to recompose before publishing action admission.
        pub fn update_batch_at(
            &mut self,
            expected_epoch: u64,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            if self.activation_epoch() != expected_epoch {
                return Ok(false);
            }
            if let Err(error) = self
                .state
                .replace_batch_at(expected_epoch, initial, ordered)
            {
                if self.activation_epoch() != expected_epoch {
                    return Ok(false);
                }
                return Err(error);
            }
            Ok(self.flush_pending())
        }

        /// Complete a retained publication after activation. False means the
        /// host must keep semantic action admission unpublished and retry later.
        pub fn flush_pending(&mut self) -> bool {
            let adapter = &mut self.adapter;
            self.state.flush(|supply| {
                let mut sent = false;
                if let Some(adapter) = adapter.as_mut() {
                    if let Some(events) = adapter.update_if_active(|| {
                        sent = true;
                        supply()
                    }) {
                        events.raise();
                    }
                }
                sent
            })
        }

        /// Native Windows geometry queries read ClientToScreen on demand.
        pub fn update_window_bounds(&mut self, _window: &Window) {}

        pub fn update_window_focus(&mut self, _focused: bool) {}

        pub fn drain_actions(&mut self) -> Vec<A11yActionRequest> {
            self.flush_pending();
            self.state.drain_actions()
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use accesskit::TreeUpdate;
    use accesskit_macos::SubclassingAdapter;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winit::window::Window;

    use super::{A11yActionRequest, BridgeStatus};
    use std::sync::Arc;

    pub struct AccessKitBridge {
        adapter: Option<SubclassingAdapter>,
        state: super::BridgeState,
    }

    impl AccessKitBridge {
        pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
            Self {
                adapter: None,
                state: super::BridgeState::new(Arc::new(wake), None),
            }
        }

        /// Capture action admission synchronously before the wake callback.
        pub fn with_action_handler(
            wake: impl Fn() + Send + Sync + 'static,
            handler: impl Fn(A11yActionRequest) + Send + Sync + 'static,
        ) -> Self {
            let mut bridge = Self::new(wake);
            bridge.state.sink = Some(std::sync::Arc::new(handler));
            bridge
        }

        pub fn status(&self) -> BridgeStatus {
            if self.adapter.is_some() {
                BridgeStatus::Installed
            } else {
                BridgeStatus::Unavailable
            }
        }

        pub fn install(&mut self, window: &Window, initial: TreeUpdate) -> Result<(), String> {
            if !self.state.replace_host(initial) {
                return Err("installation requires a complete host root tree".into());
            }
            if self.adapter.is_some() {
                return Ok(());
            }
            let ns_view = match window
                .window_handle()
                .map_err(|err| err.to_string())?
                .as_raw()
            {
                RawWindowHandle::AppKit(handle) => handle.ns_view.as_ptr(),
                _ => return Err("window is not backed by an AppKit NSView".to_string()),
            };
            let adapter = unsafe {
                // SAFETY: `ns_view` comes from winit's live raw AppKit window handle
                // during `resumed`, while the window is owned by the application.
                SubclassingAdapter::new(
                    ns_view,
                    self.state.activation(),
                    self.state.action_handler(),
                )
            };
            self.adapter = Some(adapter);
            Ok(())
        }

        pub fn update(&mut self, update: TreeUpdate) {
            if self.state.replace_host(update) {
                self.flush_pending();
            }
        }

        /// Retain the complete ordered publication. The initial snapshot must
        /// focus the host root; guest trees are installed by the ordered replay.
        /// The whole ordered vector must remain valid starting from that safe
        /// initial root after any adapter reset. Epoch-dependent steady-state
        /// deltas must use [`Self::update_batch_at`] instead.
        /// Returns true only after the complete batch was served without reset.
        pub fn update_batch(
            &mut self,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            self.state.replace_batch(initial, ordered)?;
            Ok(self.flush_pending())
        }

        /// Epoch of the current native activation lifecycle. Compose a fresh
        /// host publication whenever this value changes.
        pub fn activation_epoch(&self) -> u64 {
            self.state.activation_epoch()
        }

        /// Publish updates composed for one activation lifecycle. Unlike
        /// update_batch, these updates may depend on that epoch's guest trees.
        /// False asks the host to recompose before publishing action admission.
        pub fn update_batch_at(
            &mut self,
            expected_epoch: u64,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            if self.activation_epoch() != expected_epoch {
                return Ok(false);
            }
            if let Err(error) = self
                .state
                .replace_batch_at(expected_epoch, initial, ordered)
            {
                if self.activation_epoch() != expected_epoch {
                    return Ok(false);
                }
                return Err(error);
            }
            Ok(self.flush_pending())
        }

        /// Complete a retained publication after activation. False means the
        /// host must keep semantic action admission unpublished and retry later.
        pub fn flush_pending(&mut self) -> bool {
            let adapter = &mut self.adapter;
            self.state.flush(|supply| {
                let mut sent = false;
                if let Some(adapter) = adapter.as_mut() {
                    if let Some(events) = adapter.update_if_active(|| {
                        sent = true;
                        supply()
                    }) {
                        events.raise();
                    }
                }
                sent
            })
        }

        /// Native macOS geometry queries read current view and screen transforms.
        pub fn update_window_bounds(&mut self, _window: &Window) {}

        pub fn update_window_focus(&mut self, focused: bool) {
            if let Some(adapter) = self.adapter.as_mut() {
                if let Some(events) = adapter.update_view_focus_state(focused) {
                    events.raise();
                }
            }
        }

        pub fn drain_actions(&mut self) -> Vec<A11yActionRequest> {
            self.flush_pending();
            self.state.drain_actions()
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use accesskit::{Rect, TreeUpdate};
    use accesskit_unix::Adapter;
    use winit::window::Window;

    use super::{A11yActionRequest, BridgeStatus};
    use std::sync::Arc;

    pub struct AccessKitBridge {
        adapter: Option<Adapter>,
        state: super::BridgeState,
    }

    impl AccessKitBridge {
        pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
            Self {
                adapter: None,
                state: super::BridgeState::new(Arc::new(wake), None),
            }
        }

        /// Capture action admission synchronously before the wake callback.
        pub fn with_action_handler(
            wake: impl Fn() + Send + Sync + 'static,
            handler: impl Fn(A11yActionRequest) + Send + Sync + 'static,
        ) -> Self {
            let mut bridge = Self::new(wake);
            bridge.state.sink = Some(std::sync::Arc::new(handler));
            bridge
        }

        pub fn status(&self) -> BridgeStatus {
            if self.adapter.is_some() {
                BridgeStatus::Installed
            } else {
                BridgeStatus::Unavailable
            }
        }

        pub fn install(&mut self, window: &Window, initial: TreeUpdate) -> Result<(), String> {
            if !self.state.replace_host(initial) {
                return Err("installation requires a complete host root tree".into());
            }
            if self.adapter.is_none() {
                let mut adapter = Adapter::new(
                    self.state.activation(),
                    self.state.action_handler(),
                    self.state.deactivation(),
                );
                set_root_window_bounds(&mut adapter, window);
                self.adapter = Some(adapter);
            }
            Ok(())
        }

        pub fn update(&mut self, update: TreeUpdate) {
            if self.state.replace_host(update) {
                self.flush_pending();
            }
        }

        /// Retain the complete ordered publication. The initial snapshot must
        /// focus the host root; guest trees are installed by the ordered replay.
        /// The whole ordered vector must remain valid starting from that safe
        /// initial root after any adapter reset. Epoch-dependent steady-state
        /// deltas must use [`Self::update_batch_at`] instead.
        /// Returns true only after the complete batch was served without reset.
        pub fn update_batch(
            &mut self,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            self.state.replace_batch(initial, ordered)?;
            Ok(self.flush_pending())
        }

        /// Epoch of the current native activation lifecycle. Compose a fresh
        /// host publication whenever this value changes.
        pub fn activation_epoch(&self) -> u64 {
            self.state.activation_epoch()
        }

        /// Publish updates composed for one activation lifecycle. Unlike
        /// update_batch, these updates may depend on that epoch's guest trees.
        /// False asks the host to recompose before publishing action admission.
        pub fn update_batch_at(
            &mut self,
            expected_epoch: u64,
            initial: TreeUpdate,
            ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            if self.activation_epoch() != expected_epoch {
                return Ok(false);
            }
            if let Err(error) = self
                .state
                .replace_batch_at(expected_epoch, initial, ordered)
            {
                if self.activation_epoch() != expected_epoch {
                    return Ok(false);
                }
                return Err(error);
            }
            Ok(self.flush_pending())
        }

        /// Complete a retained publication after activation. False means the
        /// host must keep semantic action admission unpublished and retry later.
        pub fn flush_pending(&mut self) -> bool {
            let adapter = &mut self.adapter;
            self.state.flush(|supply| {
                let mut sent = false;
                if let Some(adapter) = adapter.as_mut() {
                    adapter.update_if_active(|| {
                        sent = true;
                        supply()
                    });
                }
                sent
            })
        }

        pub fn update_window_bounds(&mut self, window: &Window) {
            if let Some(adapter) = self.adapter.as_mut() {
                set_root_window_bounds(adapter, window);
            }
        }

        pub fn update_window_focus(&mut self, focused: bool) {
            if let Some(adapter) = self.adapter.as_mut() {
                adapter.update_window_focus_state(focused);
            }
        }

        pub fn drain_actions(&mut self) -> Vec<A11yActionRequest> {
            self.flush_pending();
            self.state.drain_actions()
        }
    }

    fn set_root_window_bounds(adapter: &mut Adapter, window: &Window) {
        let outer_size = window.outer_size();
        let inner_size = window.inner_size();
        let outer = window.outer_position().ok();
        let inner = window.inner_position().ok();
        let outer_x = outer.map_or(0.0, |position| position.x as f64);
        let outer_y = outer.map_or(0.0, |position| position.y as f64);
        let inner_x = inner.map_or(outer_x, |position| position.x as f64);
        let inner_y = inner.map_or(outer_y, |position| position.y as f64);
        adapter.set_root_window_bounds(
            Rect::new(
                outer_x,
                outer_y,
                outer_x + outer_size.width as f64,
                outer_y + outer_size.height as f64,
            ),
            Rect::new(
                inner_x,
                inner_y,
                inner_x + inner_size.width as f64,
                inner_y + inner_size.height as f64,
            ),
        );
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
mod imp {
    use accesskit::TreeUpdate;
    use winit::window::Window;

    use super::{A11yActionRequest, BridgeStatus};

    pub struct AccessKitBridge;

    impl AccessKitBridge {
        pub fn new(_wake: impl Fn() + Send + Sync + 'static) -> Self {
            Self
        }

        pub fn status(&self) -> BridgeStatus {
            BridgeStatus::Unavailable
        }

        pub fn install(&mut self, _window: &Window, _initial: TreeUpdate) -> Result<(), String> {
            Err("AccessKit OS bridge is not wired for this platform".to_string())
        }

        pub fn with_action_handler(
            _wake: impl Fn() + Send + Sync + 'static,
            _handler: impl Fn(A11yActionRequest) + Send + Sync + 'static,
        ) -> Self {
            Self
        }

        pub fn update(&mut self, _update: TreeUpdate) {}

        pub fn update_batch(
            &mut self,
            _initial: TreeUpdate,
            _ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            Ok(false)
        }

        pub fn activation_epoch(&self) -> u64 {
            0
        }
        pub fn update_batch_at(
            &mut self,
            _expected_epoch: u64,
            _initial: TreeUpdate,
            _ordered: Vec<TreeUpdate>,
        ) -> Result<bool, String> {
            Ok(false)
        }
        pub fn update_window_bounds(&mut self, _window: &Window) {}

        pub fn flush_pending(&mut self) -> bool {
            false
        }

        pub fn update_window_focus(&mut self, _focused: bool) {}

        pub fn drain_actions(&mut self) -> Vec<A11yActionRequest> {
            Vec::new()
        }
    }
}

pub use imp::AccessKitBridge;

#[cfg(test)]
mod tests {
    use super::{A11yActionRequest, QueuedActions};
    use accesskit::{Action, ActionData, ActionHandler, ActionRequest, NodeId, TreeId};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn queued_actions_retain_value_data() {
        let actions = Arc::new(Mutex::new(VecDeque::<A11yActionRequest>::new()));
        let mut queued = QueuedActions {
            actions: Arc::clone(&actions),
            wake: Arc::new(|| {}),
            sink: None,
        };
        queued.do_action(ActionRequest {
            action: Action::SetValue,
            target_tree: TreeId(accesskit::Uuid::from_u128(77)),
            target_node: NodeId(11),
            data: Some(ActionData::Value("C:/next.html".into())),
        });
        let request = actions
            .lock()
            .expect("queued action lock")
            .pop_front()
            .unwrap();
        assert_eq!(request.action, Action::SetValue);
        assert_eq!(request.target_node, NodeId(11));
        assert_eq!(request.target_tree, TreeId(accesskit::Uuid::from_u128(77)));
        assert_eq!(request.data, Some(ActionData::Value("C:/next.html".into())));
    }

    #[test]
    fn custom_action_sink_captures_tree_identity_before_waking() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let actions = Arc::new(Mutex::new(VecDeque::<A11yActionRequest>::new()));
        let delivered = Arc::new(Mutex::new(Vec::new()));
        let woke = Arc::new(AtomicUsize::new(0));
        let output = delivered.clone();
        let on_wake = woke.clone();
        let mut queued = QueuedActions::with_sink(
            actions.clone(),
            Arc::new(move || {
                on_wake.fetch_add(1, Ordering::Relaxed);
            }),
            Arc::new(move |request| {
                output.lock().unwrap().push(request);
            }),
        );
        let tree = TreeId(accesskit::Uuid::from_u128(88));
        queued.do_action(ActionRequest {
            action: Action::Focus,
            target_tree: tree,
            target_node: NodeId(0),
            data: None,
        });
        assert!(
            actions.lock().unwrap().is_empty(),
            "custom custody does not duplicate default queue delivery"
        );
        assert_eq!(woke.load(Ordering::Relaxed), 1);
        assert_eq!(delivered.lock().unwrap()[0].target_tree, tree);
        assert_eq!(delivered.lock().unwrap()[0].target_node, NodeId(0));
    }

    #[test]
    fn batch_completion_requires_every_boundary_and_no_activation_reset() {
        use accesskit::{Node, Role, Tree, TreeUpdate};
        use std::sync::atomic::Ordering;
        let root = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut state = super::BridgeState::new(Arc::new(|| {}), None);
        state
            .replace_batch(root.clone(), vec![root.clone(), root.clone()])
            .unwrap();
        let mut calls = 0;
        assert!(!state.flush(|supply| {
            let _ = supply();
            calls += 1;
            calls != 2
        }));
        assert_eq!(calls, 2);
        assert!(state.replay.load(Ordering::Acquire));
        let replay = state.replay.clone();
        assert!(!state.flush(|supply| {
            let _ = supply();
            replay.store(true, Ordering::Release);
            true
        }));
        calls = 0;
        assert!(state.flush(|supply| {
            let _ = supply();
            calls += 1;
            true
        }));
        assert_eq!(calls, 2, "a reset replays the entire publication");
        assert!(state.flush(|_| panic!("completed publication is not duplicated")));
    }

    #[test]
    fn single_host_publication_cannot_activate_a_missing_guest() {
        use accesskit::{Node, Role, Tree, TreeUpdate};
        let guest_id = TreeId(accesskit::Uuid::from_u128(77));
        let mut root = Node::new(Role::Window);
        root.set_children(vec![NodeId(1)]);
        let mut graft = Node::new(Role::GenericContainer);
        graft.set_tree_id(guest_id);
        let host = TreeUpdate {
            nodes: vec![(NodeId(0), root), (NodeId(1), graft)],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(1),
        };
        let mut publication = super::A11yPublication::default();
        assert!(publication.replace_host(host));
        let initial = publication.initial().unwrap();
        assert!(
            initial
                .nodes
                .iter()
                .all(|(_, node)| node.tree_id().is_none())
        );
        let _ = accesskit_consumer::Tree::new(initial, true);
        assert!(!publication.replace_host(TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: guest_id,
            focus: NodeId(0),
        }));
        assert_eq!(publication.initial().unwrap().tree_id, TreeId::ROOT);
    }

    #[test]
    fn publication_replays_safe_root_before_guest_focus_after_activation() {
        use accesskit::{Node, Role, Tree, TreeUpdate};
        use accesskit_consumer::{Node as ConsumerNode, TreeChangeHandler};
        struct Changes;
        impl TreeChangeHandler for Changes {
            fn node_added(&mut self, _: &ConsumerNode<'_>) {}
            fn node_updated(&mut self, _: &ConsumerNode<'_>, _: &ConsumerNode<'_>) {}
            fn node_removed(&mut self, _: &ConsumerNode<'_>) {}
            fn focus_moved(&mut self, _: Option<&ConsumerNode<'_>>, _: Option<&ConsumerNode<'_>>) {}
        }
        let guest_id = TreeId(accesskit::Uuid::from_u128(77));
        let mut root = Node::new(Role::Window);
        root.set_children(vec![NodeId(1)]);
        let mut graft = Node::new(Role::GenericContainer);
        graft.set_tree_id(guest_id);
        let host = TreeUpdate {
            nodes: vec![(NodeId(0), root), (NodeId(1), graft)],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let guest = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: guest_id,
            focus: NodeId(0),
        };
        let focused = TreeUpdate {
            focus: NodeId(1),
            ..host.clone()
        };
        let mut publication = super::A11yPublication::default();
        assert!(
            publication
                .replace(focused.clone(), vec![focused.clone()])
                .is_err(),
            "an activation snapshot cannot focus an uninstalled guest"
        );
        let safe_root = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        publication
            .replace(safe_root.clone(), vec![host.clone(), guest, focused])
            .unwrap();
        let initial = publication.initial().unwrap();
        assert_eq!(initial.tree_id, TreeId::ROOT);
        assert_eq!(initial.focus, NodeId(0));
        let mut consumer = accesskit_consumer::Tree::new(initial, true);
        for update in publication.replay() {
            consumer.update_and_process_changes(update, &mut Changes);
        }
        // Activation can happen after any prefix of a previous delivery. Each
        // fresh consumer must be initialized safely and accept every boundary.
        for prefix in 0..=publication.replay().len() {
            let mut partial = accesskit_consumer::Tree::new(publication.initial().unwrap(), true);
            for update in publication.replay().into_iter().take(prefix) {
                partial.update_and_process_changes(update, &mut Changes);
            }
            let mut fresh = accesskit_consumer::Tree::new(publication.initial().unwrap(), true);
            for update in publication.replay() {
                fresh.update_and_process_changes(update, &mut Changes);
            }
        }
        // Replacement deliberately reuses NodeId(0) in a different guest tree.
        let replacement_id = TreeId(accesskit::Uuid::from_u128(88));
        let mut replacement_host = host.clone();
        replacement_host.nodes[1].1.set_tree_id(replacement_id);
        let replacement_guest = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Document))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: replacement_id,
            focus: NodeId(0),
        };
        let replacement_focused = TreeUpdate {
            focus: NodeId(1),
            ..replacement_host.clone()
        };
        publication
            .replace(
                safe_root,
                vec![replacement_host, replacement_guest, replacement_focused],
            )
            .unwrap();
        for update in publication.replay() {
            consumer.update_and_process_changes(update, &mut Changes);
        }
        for prefix in 0..=publication.replay().len() {
            let mut fresh = accesskit_consumer::Tree::new(publication.initial().unwrap(), true);
            for update in publication.replay().into_iter().take(prefix) {
                fresh.update_and_process_changes(update, &mut Changes);
            }
        }
        // Removing the graft must also retire it from future activation replay.
        let root_only = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        publication
            .replace(root_only.clone(), vec![root_only])
            .unwrap();
        for update in publication.replay() {
            consumer.update_and_process_changes(update, &mut Changes);
        }
        let mut reactivated = accesskit_consumer::Tree::new(publication.initial().unwrap(), true);
        for update in publication.replay() {
            assert_eq!(update.tree_id, TreeId::ROOT);
            reactivated.update_and_process_changes(update, &mut Changes);
        }
    }
    #[test]
    fn activation_rejects_an_uninstalled_child_graft_even_with_host_root_focus() {
        use accesskit::{Node, Role, Tree, TreeUpdate};
        let safe = TreeUpdate {
            nodes: vec![(NodeId(0), Node::new(Role::Window))],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        };
        let mut publication = super::A11yPublication::default();
        publication
            .replace(safe.clone(), vec![safe.clone()])
            .unwrap();
        let mut root = Node::new(Role::Window);
        root.set_children(vec![NodeId(1)]);
        let mut graft = Node::new(Role::GenericContainer);
        graft.set_tree_id(TreeId(accesskit::Uuid::from_u128(99)));
        let unsafe_initial = TreeUpdate {
            nodes: vec![(NodeId(0), root), (NodeId(1), graft)],
            ..safe
        };
        assert!(
            publication
                .replace(unsafe_initial.clone(), vec![unsafe_initial])
                .is_err(),
            "root focus does not make missing guest references safe activation data"
        );
        let retained = publication.initial().unwrap();
        assert_eq!(
            retained.nodes.len(),
            1,
            "refusal keeps the prior valid publication"
        );
        let _ = accesskit_consumer::Tree::new(retained, true);
    }

    fn safe_epoch_root() -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes: vec![(NodeId(0), accesskit::Node::new(accesskit::Role::Window))],
            tree: Some(accesskit::Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        }
    }

    #[test]
    fn activation_inside_adapter_before_update_supplier_substitutes_safe_root() {
        use accesskit::ActivationHandler;
        let mut state = super::BridgeState::new(Arc::new(|| {}), None);
        let safe = safe_epoch_root();
        let mut changed = safe.clone();
        changed.nodes[0].1.set_label("stale batch");
        let epoch = state.activation_epoch();
        state
            .replace_batch_at(epoch, safe.clone(), vec![changed])
            .unwrap();
        let mut activation = state.activation();
        let mut supplied = None;
        assert!(!state.flush(|supply| {
            activation.request_initial_tree();
            supplied = Some(supply());
            true
        }));
        assert_eq!(supplied.unwrap().nodes[0].1.label(), None);
        assert!(state.activation_epoch() > epoch);
    }

    #[test]
    fn epoch_bound_batch_stops_between_entries_and_refuses_later_flush() {
        use accesskit::ActivationHandler;
        let mut state = super::BridgeState::new(Arc::new(|| {}), None);
        let safe = safe_epoch_root();
        let epoch = state.activation_epoch();
        state
            .replace_batch_at(epoch, safe.clone(), vec![safe.clone(), safe])
            .unwrap();
        let mut activation = state.activation();
        let mut count = 0;
        assert!(!state.flush(|supply| {
            let _ = supply();
            count += 1;
            activation.request_initial_tree();
            true
        }));
        assert_eq!(count, 1);
        assert!(!state.flush(|_| panic!("stale retained batch must never reach an adapter")));
    }

    #[test]
    fn steady_epoch_delivers_only_ordered_updates_without_resetting_focus() {
        let mut state = super::BridgeState::new(Arc::new(|| {}), None);
        let safe = safe_epoch_root();
        let mut host = safe.clone();
        host.nodes[0].1.set_children(vec![NodeId(1)]);
        host.nodes
            .push((NodeId(1), accesskit::Node::new(accesskit::Role::Button)));
        host.focus = NodeId(1);
        state
            .replace_batch_at(state.activation_epoch(), safe, vec![host])
            .unwrap();
        let mut focus = Vec::new();
        assert!(state.flush(|supply| {
            focus.push(supply().focus);
            true
        }));
        assert_eq!(focus, vec![NodeId(1)]);
        assert!(state.flush(|_| panic!("completed publication is not duplicated")));
    }

    #[test]
    fn epoch_bound_replacement_refuses_an_already_stale_host_observation() {
        use accesskit::ActivationHandler;
        let mut state = super::BridgeState::new(Arc::new(|| {}), None);
        let safe = safe_epoch_root();
        let epoch = state.activation_epoch();
        state.activation().request_initial_tree();
        assert!(
            state
                .replace_batch_at(epoch, safe.clone(), vec![safe])
                .is_err()
        );
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn deactivation_retires_the_epoch_even_before_a_later_reactivation() {
        use accesskit::{ActivationHandler, DeactivationHandler};
        let state = super::BridgeState::new(Arc::new(|| {}), None);
        let first = state.activation_epoch();
        state.activation().request_initial_tree();
        let active = state.activation_epoch();
        assert!(active > first);
        state.deactivation().deactivate_accessibility();
        assert!(state.activation_epoch() > active);
    }
}
