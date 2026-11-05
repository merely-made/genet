// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Condvar, Mutex, mpsc};

use super::{Job, NativeFrames, RasterizeFailure, State, availability, unavailable};

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
    pub(super) state: Arc<Mutex<availability::Gate>>,
    pub(super) worker: Arc<Worker>,
    pub(super) changed: Arc<Condvar>,
}
impl NativeFrameShutdown {
    pub(crate) fn close(&self) -> Result<(), String> {
        if self.worker.is_current() {
            return Err("native completion worker cannot join itself".into());
        }
        let waiters = {
            let mut state = self.state.lock().expect("native frames poisoned");
            loop {
                match state.phase {
                    State::Submitting | State::Legacy => {
                        return Err("native frame submission is still active".into());
                    },
                    State::Publishing(id, _) if id == std::thread::current().id() => {
                        return Err("native frame publisher cannot close itself".into());
                    },
                    State::Publishing(id, _) => {
                        state.phase = State::Publishing(id, true);
                        state = self.changed.wait(state).expect("native frames poisoned");
                    },
                    _ => break,
                }
            }
            state.close()
        };
        for waiter in waiters {
            waiter.notify();
        }
        self.worker.close()
    }
}

pub(super) struct Publication<'a>(&'a NativeFrames);
impl Drop for Publication<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state.lock().expect("native frames poisoned");
        let waiters = if let State::Publishing(_, closing) = state.phase {
            let waiters = if closing {
                state.close()
            } else {
                state.phase = State::Pending;
                Vec::new()
            };
            self.0.changed.notify_all();
            waiters
        } else {
            Vec::new()
        };
        drop(state);
        for waiter in waiters {
            waiter.notify();
        }
    }
}

impl NativeFrames {
    pub(super) fn schedule(&self, job: Job) {
        let job = Arc::new(Mutex::new(Some(job)));
        let worker_job = job.clone();
        if let Err(reason) = self.worker.schedule(Box::new(move || {
            worker_job.lock().unwrap().take().unwrap().run();
        })) {
            job.lock().unwrap().take().unwrap().refuse(&reason);
        }
    }
    pub(super) fn close(&self) -> Result<(), String> {
        self.shutdown_handle().close()
    }
    pub(super) fn shutdown_handle(&self) -> NativeFrameShutdown {
        NativeFrameShutdown {
            state: self.state.clone(),
            worker: self.worker.clone(),
            changed: self.changed.clone(),
        }
    }
    pub(super) fn begin_publication(&self) -> Result<Publication<'_>, RasterizeFailure> {
        let mut state = self.state.lock().expect("native frames poisoned");
        if !matches!(state.phase, State::Pending) {
            return Err(unavailable("native frame publication unavailable"));
        }
        state.phase = State::Publishing(std::thread::current().id(), false);
        Ok(Publication(self))
    }
}
