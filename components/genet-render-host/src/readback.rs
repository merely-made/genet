// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A readback owns cancellable GPU handles, never a strong RenderCore.
use crate::recovery::LossCancellation;
use crate::{DeviceLossMonitor, DeviceLossNotice, RgbaFrame};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadbackFailure {
    DeviceLost(DeviceLossNotice),
    Gpu(String),
    AlreadyCollected,
}
impl std::fmt::Display for ReadbackFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "readback refused: {self:?}")
    }
}
impl std::error::Error for ReadbackFailure {}
struct Resources {
    device: wgpu::Device,
    buffer: wgpu::Buffer,
    mapped: Receiver<Result<(), wgpu::BufferAsyncError>>,
    submission: wgpu::SubmissionIndex,
}
#[cfg(all(target_arch = "wasm32", feature = "browser-surface-readback-diagnostics"))]
impl Drop for Resources {
    fn drop(&mut self) { self.buffer.unmap(); }
}
struct State {
    resources: Option<Resources>,
    loss: Option<DeviceLossNotice>,
}
fn cancel_state(state: &Arc<Mutex<State>>, loss: &DeviceLossNotice) {
    let resources = {
        let mut state = state.lock().expect("readback state poisoned");
        if state.loss.is_none() {
            state.loss = Some(loss.clone());
        }
        state.resources.take()
    };
    drop(resources);
}
// Browser GPU handles are not Send. The WebGPU loss callback runs on their
// origin JS thread; its Send closure carries only an ID, never a GPU handle.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static LOCAL_READBACKS: std::cell::RefCell<std::collections::HashMap<u64, std::sync::Weak<Mutex<State>>>> = Default::default();
}
#[cfg(target_arch = "wasm32")]
struct LocalReadback(u64);
#[cfg(target_arch = "wasm32")]
impl Drop for LocalReadback {
    fn drop(&mut self) {
        LOCAL_READBACKS.with(|states| {
            states.borrow_mut().remove(&self.0);
        });
    }
}
pub struct PendingRgbaReadback {
    state: Arc<Mutex<State>>,
    _cancellation: LossCancellation,
    width: u32,
    height: u32,
    padded: u32,
    generation: u64,
    completed: bool,
    #[cfg(target_arch = "wasm32")]
    _local: LocalReadback,
}
impl PendingRgbaReadback {
    pub(crate) fn new(
        device: wgpu::Device,
        buffer: wgpu::Buffer,
        mapped: Receiver<Result<(), wgpu::BufferAsyncError>>,
        submission: wgpu::SubmissionIndex,
        width: u32,
        height: u32,
        padded: u32,
        monitor: &Arc<DeviceLossMonitor>,
    ) -> Self {
        let state = Arc::new(Mutex::new(State {
            resources: Some(Resources {
                device,
                buffer,
                mapped,
                submission,
            }),
            loss: None,
        }));
        let weak = Arc::downgrade(&state);
        #[cfg(not(target_arch = "wasm32"))]
        let cancellation = monitor.register_cancellation(Arc::new(move |loss| {
            if let Some(state) = weak.upgrade() {
                // Drop buffer, device and map receiver before waking hosts to
                // detach/recover; a late map callback has no receiver.
                cancel_state(&state, loss);
            }
        }));
        #[cfg(target_arch = "wasm32")]
        let (cancellation, local) = {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
            let id = NEXT
                .fetch_update(
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                    |v| v.checked_add(1),
                )
                .expect("browser readback identity exhausted");
            LOCAL_READBACKS.with(|states| {
                states.borrow_mut().insert(id, weak);
            });
            let cancellation = monitor.register_cancellation(Arc::new(move |loss| {
                let state = LOCAL_READBACKS
                    .with(|states| states.borrow().get(&id).and_then(|weak| weak.upgrade()));
                if let Some(state) = state {
                    cancel_state(&state, loss);
                }
            }));
            (cancellation, LocalReadback(id))
        };
        Self {
            state,
            _cancellation: cancellation,
            width,
            height,
            padded,
            generation: monitor.generation,
            completed: false,
            #[cfg(target_arch = "wasm32")]
            _local: local,
        }
    }
    #[cfg(all(target_arch = "wasm32", feature = "browser-surface-readback-diagnostics"))]
    pub(crate) fn cancel(&mut self) {
        let resources={ self.state.lock().expect("readback state poisoned").resources.take() };
        self.completed=true;
        drop(resources); // unmap/device drop can reenter loss outside the lock.
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    /// Pixels or a loss notice are delivered once. Repeated collection is a
    /// distinct misuse refusal, preserving the previous no-replay contract.
    /// Loss releases GPU handles even if the consumer has not polled.
    pub fn poll(&mut self) -> Option<Result<RgbaFrame, ReadbackFailure>> {
        if self.completed {
            return Some(Err(ReadbackFailure::AlreadyCollected));
        }
        let device = self
            .state
            .lock()
            .expect("readback state poisoned")
            .resources
            .as_ref()
            .map(|resources| resources.device.clone());
        if let Some(device) = device {
            let result = device.poll(wgpu::PollType::Poll);
            drop(device);
            if let Err(error) = result {
                if let Some(result) = self.collect() {
                    return Some(result);
                }
                self.completed = true;
                let resources = {
                    self.state
                        .lock()
                        .expect("readback state poisoned")
                        .resources
                        .take()
                };
                // Dropping the last device can synchronously notify loss and
                // reenter cancel_state. Never release handles under its lock.
                drop(resources);
                return Some(Err(ReadbackFailure::Gpu(format!(
                    "RGBA readback poll failed: {error}"
                ))));
            }
        }
        self.collect()
    }
    fn collect(&mut self) -> Option<Result<RgbaFrame, ReadbackFailure>> {
        let mut state = self.state.lock().expect("readback state poisoned");
        if let Some(loss) = state.loss.take() {
            self.completed = true;
            return Some(Err(ReadbackFailure::DeviceLost(loss)));
        }
        let resources = state.resources.as_ref()?;
        let mapped = match resources.mapped.try_recv() {
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(ReadbackFailure::Gpu(
                "RGBA map callback disconnected".into(),
            )),
            Ok(Err(error)) => Err(ReadbackFailure::Gpu(format!("RGBA map failed: {error}"))),
            Ok(Ok(())) => Ok(()),
        };
        self.completed = true;
        let result = mapped.and_then(|()| {
            let slice = resources.buffer.slice(..);
            let data = slice
                .get_mapped_range()
                .map_err(|error| ReadbackFailure::Gpu(format!("RGBA map failed: {error}")))?;
            let unpadded = self.width as usize * 4;
            let bytes=unpadded.checked_mul(self.height as usize)
                .ok_or_else(|| ReadbackFailure::Gpu("RGBA mapped row size overflow".into()))?;
            let mut rgba=Vec::new();
            rgba.try_reserve_exact(bytes)
                .map_err(|error|ReadbackFailure::Gpu(format!("RGBA CPU reservation failed: {error}")))?;
            for row in 0..self.height as usize {
                let start = row * self.padded as usize;
                let bytes=data.get(start..start + unpadded)
                    .ok_or_else(||ReadbackFailure::Gpu("mapped buffer is shorter than its row layout".into()))?;
                rgba.extend_from_slice(bytes);
            }
            drop(data);
            resources.buffer.unmap();
            Ok(RgbaFrame {
                width: self.width,
                height: self.height,
                rgba,
            })
        });
        let resources = state.resources.take();
        drop(state);
        drop(resources);
        Some(result)
    }
    pub(crate) fn wait_for_completion(&mut self) -> Result<RgbaFrame, ReadbackFailure> {
        let wait = self
            .state
            .lock()
            .expect("readback state poisoned")
            .resources
            .as_ref()
            .map(|resources| (resources.device.clone(), resources.submission.clone()));
        if let Some((device, submission)) = wait {
            let result = device.poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(Duration::from_secs(5)),
            });
            drop(device);
            if let Err(error) = result {
                if let Some(result) = self.collect() {
                    return result;
                }
                return Err(ReadbackFailure::Gpu(format!("RGBA wait failed: {error}")));
            }
        }
        self.collect()
            .unwrap_or_else(|| Err(ReadbackFailure::Gpu("RGBA mapping did not complete".into())))
    }
}
