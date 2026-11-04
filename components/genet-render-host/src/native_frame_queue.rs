// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Condvar, Mutex, mpsc};

use super::{NativeFrames, RasterizeFailure, State, unavailable};

type Work = Box<dyn FnOnce() + Send>;
#[derive(Default)]
pub(super) struct Worker {
    state: Mutex<WorkerState>,
    closed: Condvar,
}
#[derive(Default)]
enum WorkerState {
    #[default]
    Unstarted,
    Running(mpsc::Sender<Work>, std::thread::JoinHandle<()>),
    Closing(std::thread::ThreadId),
    Closed,
}
impl Worker {
    pub(super) fn schedule(&self, work: Work) -> Result<(), String> {
        let mut state = self.state.lock().expect("native worker poisoned");
        if matches!(*state, WorkerState::Unstarted) {
            let (sender, receiver) = mpsc::channel::<Work>();
            let handle = std::thread::Builder::new()
                .name("genet-frame-completion".into())
                .spawn(move || {
                    for work in receiver {
                        work();
                    }
                })
                .map_err(|error| format!("native completion worker could not start: {error}"))?;
            *state = WorkerState::Running(sender, handle);
        }
        match &*state {
            WorkerState::Running(sender, _) => sender
                .send(work)
                .map_err(|_| "native completion worker stopped".into()),
            _ => Err("native completion worker is closed".into()),
        }
    }
    pub(super) fn is_current(&self) -> bool {
        let state = self.state.lock().expect("native worker poisoned");
        let current = std::thread::current().id();
        match &*state {
            WorkerState::Running(_, handle) => handle.thread().id() == current,
            WorkerState::Closing(id) => *id == current,
            _ => false,
        }
    }
    pub(super) fn close(&self) -> Result<(), String> {
        let mut state = self.state.lock().expect("native worker poisoned");
        loop {
            match &*state {
                WorkerState::Closed => return Ok(()),
                WorkerState::Closing(id) if *id == std::thread::current().id() => {
                    return Err("native completion worker cannot join itself".into());
                },
                WorkerState::Closing(_) => {
                    state = self.closed.wait(state).expect("native worker poisoned")
                },
                WorkerState::Running(_, handle)
                    if handle.thread().id() == std::thread::current().id() =>
                {
                    return Err("native completion worker cannot join itself".into());
                },
                _ => break,
            }
        }
        let old = std::mem::replace(&mut *state, WorkerState::Closed);
        if let WorkerState::Running(sender, handle) = old {
            *state = WorkerState::Closing(handle.thread().id());
            drop(sender);
            drop(state);
            let result = handle
                .join()
                .map_err(|_| "native completion worker panicked".to_string());
            *self.state.lock().expect("native worker poisoned") = WorkerState::Closed;
            self.closed.notify_all();
            result
        } else {
            self.closed.notify_all();
            Ok(())
        }
    }
}

#[derive(Clone)]
pub(crate) struct NativeFrameShutdown {
    pub(super) state: Arc<Mutex<State>>,
    pub(super) worker: Arc<Worker>,
    pub(super) changed: Arc<Condvar>,
}
impl NativeFrameShutdown {
    pub(crate) fn close(&self) -> Result<(), String> {
        if self.worker.is_current() {
            return Err("native completion worker cannot join itself".into());
        }
        {
            let mut state = self.state.lock().expect("native frames poisoned");
            loop {
                match *state {
                    State::Submitting | State::Legacy => {
                        return Err("native frame submission is still active".into());
                    },
                    State::Publishing(id, _) if id == std::thread::current().id() => {
                        return Err("native frame publisher cannot close itself".into());
                    },
                    State::Publishing(id, _) => {
                        *state = State::Publishing(id, true);
                        state = self.changed.wait(state).expect("native frames poisoned");
                    },
                    _ => break,
                }
            }
            *state = State::Closed;
        }
        self.worker.close()
    }
}

pub(super) struct Publication<'a>(&'a NativeFrames);
impl Drop for Publication<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().expect("native frames poisoned");
        if let State::Publishing(_, closing) = *state {
            *state = if closing {
                State::Closed
            } else {
                State::Pending
            };
            self.0.changed.notify_all();
        }
    }
}

impl NativeFrames {
    pub(super) fn begin_publication(&self) -> Result<Publication<'_>, RasterizeFailure> {
        let mut state = self.state.lock().expect("native frames poisoned");
        if !matches!(*state, State::Pending) {
            return Err(unavailable("native frame publication unavailable"));
        }
        *state = State::Publishing(std::thread::current().id(), false);
        Ok(Publication(self))
    }
}
