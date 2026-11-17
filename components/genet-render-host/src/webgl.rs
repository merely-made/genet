// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native WebGL contexts backed by a document host's existing render device.
//!
//! A [`WebGlHost`] is document-host state, rather than a global GPU helper:
//! every context it mints uses the exact `wgpu::Device` and `Queue` from the
//! current `RenderCore`.  The registry retains only live default-framebuffer
//! contexts.  A handler has a stable key for its lifetime; resize replaces the
//! backing texture in-place and drop removes the key.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use script_runtime_api::WebGlHandler;
use webgl_wgpu::{WebGlCanvasDescriptor, WebGlContext};

mod handler;
use handler::NativeWebGl;
#[cfg(test)]
mod tests;

const GL_ARRAY_BUFFER: u32 = 0x8892;
const GL_COLOR_BUFFER_BIT: u32 = 0x4000;
const GL_DITHER: u32 = 0x0BD0;
const GL_ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_INVALID_ENUM: u32 = 0x0500;
const GL_INVALID_FRAMEBUFFER_OPERATION: u32 = 0x0506;
const GL_INVALID_OPERATION: u32 = 0x0502;
const GL_INVALID_VALUE: u32 = 0x0501;
const GL_NO_ERROR: u32 = 0;
const GL_SCISSOR_TEST: u32 = 0x0C11;
const GL_TRIANGLES: u32 = 0x0004;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_CONTEXT_LOST_WEBGL: u32 = 0x9242;

type Context = Arc<Mutex<WebGlContext>>;

#[derive(Clone)]
struct RegisteredContext {
    context: Context,
    content_generation: Arc<AtomicU64>,
}

type Registry = Arc<Mutex<HashMap<u64, RegisteredContext>>>;

/// Same-device WebGL factory and live default-framebuffer registry.
#[derive(Clone)]
pub struct WebGlHost {
    device: wgpu::Device,
    queue: wgpu::Queue,
    next_key: Arc<AtomicU64>,
    contexts: Registry,
}

impl WebGlHost {
    /// Build from the `RenderCore` device before a scripted document is
    /// spawned.  This must not create a second wgpu device.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            next_key: Arc::new(AtomicU64::new(1)),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a script handler and register its default framebuffer.
    pub fn create_handler(&self, width: u32, height: u32) -> Option<Box<dyn WebGlHandler>> {
        self.make_handler(width, height)
            .map(|handler| Box::new(handler) as Box<dyn WebGlHandler>)
    }

    /// Whether these handles belong to the device that owns the contexts.
    pub fn matches_device(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        &self.device == device && &self.queue == queue
    }

    fn make_handler(&self, width: u32, height: u32) -> Option<NativeWebGl> {
        let width = width.max(1);
        let height = height.max(1);
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        if width > max_dimension || height > max_dimension {
            return None;
        }
        let context = Arc::new(Mutex::new(
            WebGlContext::from_wgpu_handles(
                self.device.clone(),
                self.queue.clone(),
                WebGlCanvasDescriptor::new(width, height),
            )
            .ok()?,
        ));
        let key = self.next_key.fetch_add(1, Ordering::Relaxed);
        let content_generation = Arc::new(AtomicU64::new(1));
        self.contexts.lock().expect("WebGL registry lock").insert(
            key,
            RegisteredContext {
                context: context.clone(),
                content_generation: content_generation.clone(),
            },
        );
        Some(NativeWebGl::new(
            context,
            key,
            self.contexts.clone(),
            content_generation,
        ))
    }

    /// Context handlers retaining the host device.
    pub fn live_contexts(&self) -> usize {
        self.contexts.lock().expect("WebGL registry lock").len()
    }
}

/// The WebGL images staged into one renderer. Each renderer needs its own
/// set so two placements of a document cannot retire each other's images.
#[derive(Default)]
pub struct WebGlImages {
    staged: HashSet<netrender::ImageKey>,
}

impl WebGlImages {
    /// Number of images staged in this renderer for the current document.
    pub fn staged_count(&self) -> usize {
        self.staged.len()
    }

    /// Refresh the trusted images for this frame and retire absent images.
    ///
    /// # Errors
    ///
    /// Returns an error if the renderer and WebGL host use different devices
    /// or queues. Replace this state when replacing the renderer.
    pub fn sync(
        &mut self,
        host: &WebGlHost,
        renderer: &netrender::Renderer,
        keys: impl IntoIterator<Item = u64>,
    ) -> Result<(), &'static str> {
        let core = &renderer.wgpu_device.core;
        if !host.matches_device(&core.device, &core.queue) {
            return Err("WebGL images belong to a different render device");
        }
        let live: Vec<_> = {
            let registry = host.contexts.lock().expect("WebGL registry lock");
            keys.into_iter()
                .filter_map(|key| {
                    registry
                        .get(&key)
                        .cloned()
                        .map(|context| (netrender::external_image_key(key), context))
                })
                .collect()
        };
        let desired: HashSet<_> = live.iter().map(|(key, _)| *key).collect();
        for (image_key, registered) in &live {
            let context = registered.context.lock().expect("WebGL context lock");
            let drawing_buffer = context.texture();
            renderer.stage_external_image(
                *image_key,
                &drawing_buffer.create_view(),
                [drawing_buffer.size.0, drawing_buffer.size.1],
                netrender::SourceAlpha::Premultiplied,
                registered.content_generation.load(Ordering::Relaxed),
            );
        }
        for stale in self.staged.difference(&desired) {
            renderer.unregister_external_image(*stale);
        }
        self.staged = desired;
        Ok(())
    }
}
