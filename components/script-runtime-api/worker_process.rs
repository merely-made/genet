/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A killable native boundary around one top-level dedicated-worker tree.
//! Descendant Workers use the ordinary in-process thread transport inside the
//! helper. The supplied helper starts no OS subprocesses, so reaping this one
//! process reclaims every nested worker too. Hosts must choose a trusted helper
//! executable rather than a wrapper that forks its own process tree.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use script_engine_api::ScriptEngine;
use serde::{Deserialize, Serialize};

use crate::worker::{FromWorker, ToWorker, WorkerBoot, WorkerPolicy, worker_main};

const PROTOCOL_VERSION: u32 = 2;

/// Concrete engine built into the selected helper executable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerBackend {
    Boa,
    Nova,
}

impl WorkerBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Boa => "boa",
            Self::Nova => "nova",
        }
    }
}

/// Host-provided native worker helper. The executable path is explicit; the
/// runtime never searches the current directory or guesses a sibling binary.
#[derive(Clone, Debug)]
pub struct WorkerProcessLauncher {
    pub executable: PathBuf,
    pub backend: WorkerBackend,
    pub resource_timeout: Duration,
    pub startup_timeout: Duration,
    pub shutdown_grace: Duration,
    pub max_frame_bytes: usize,
    pub max_queued_messages: usize,
}

impl WorkerProcessLauncher {
    pub fn new(executable: PathBuf, backend: WorkerBackend) -> Self {
        Self {
            executable,
            backend,
            resource_timeout: Duration::from_secs(30),
            startup_timeout: Duration::from_secs(5),
            shutdown_grace: Duration::from_millis(500),
            max_frame_bytes: 16 * 1024 * 1024,
            max_queued_messages: 32,
        }
    }

    pub(crate) fn valid(&self) -> bool {
        self.executable.is_absolute()
            && self.max_frame_bytes > 0
            && self.max_frame_bytes <= u32::MAX as usize
            && (1..=1024).contains(&self.max_queued_messages)
    }
}

#[derive(Serialize, Deserialize)]
struct Hello {
    version: u32,
    backend: WorkerBackend,
    url: String,
    name: String,
    /// The owner's secure context; absent from an older peer's hello.
    #[serde(default = "secure_by_default")]
    secure_context: bool,
    resource_timeout_ms: u64,
    max_frame_bytes: usize,
    max_queued_messages: usize,
    /// The parent's shared-clock anchor: the system monotonic source is
    /// system-wide, so the child's readings line up with the parent's.
    clock_epoch_offset_ms: f64,
    clock_skipped_ms: f64,
}

fn secure_by_default() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
struct HelloAck {
    version: u32,
    backend: WorkerBackend,
}

struct LimitedFrame {
    bytes: Vec<u8>,
    max: usize,
}

impl Write for LimitedFrame {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.max.saturating_sub(self.bytes.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "worker frame too large",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn write_frame<T: Serialize>(out: &mut impl Write, value: &T, max: usize) -> io::Result<()> {
    let mut body = LimitedFrame {
        bytes: Vec::new(),
        max,
    };
    serde_json::to_writer(&mut body, value).map_err(io::Error::other)?;
    out.write_all(&(body.bytes.len() as u32).to_be_bytes())?;
    out.write_all(&body.bytes)?;
    out.flush()
}

fn read_frame<T: for<'de> Deserialize<'de>>(
    input: &mut impl Read,
    max: usize,
) -> io::Result<Option<T>> {
    let mut len = [0; 4];
    if input.read(&mut len[..1])? == 0 {
        return Ok(None);
    }
    input.read_exact(&mut len[1..])?;
    let len = u32::from_be_bytes(len) as usize;
    if len > max {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worker frame too large",
        ));
    }
    let mut body = vec![0; len];
    input.read_exact(&mut body)?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(io::Error::other)
}

/// The parent retains this until terminate or runtime teardown. Bridge threads
/// own only pipes and channels; after a kill they cannot keep the child alive.
pub(crate) struct WorkerProcessHandle {
    child: Child,
    writer: Option<JoinHandle<()>>,
    reader: Option<JoinHandle<()>>,
    grace: Duration,
    stopping: Arc<AtomicBool>,
    outcome: Option<ProcessExit>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcessExit {
    Graceful,
    Killed,
    Crashed,
    Failed,
}

impl WorkerProcessHandle {
    pub(crate) fn spawn(
        boot: WorkerBoot,
        launcher: &WorkerProcessLauncher,
        failed: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        if !launcher.valid() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid worker process launcher",
            ));
        }
        let mut child = Command::new(&launcher.executable)
            .arg("--engine")
            .arg(launcher.backend.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child.stdin.take().expect("piped stdin");
        let mut stdout = child.stdout.take().expect("piped stdout");
        let hello = Hello {
            version: PROTOCOL_VERSION,
            backend: launcher.backend,
            url: boot.url,
            name: boot.name,
            secure_context: boot.secure_context,
            resource_timeout_ms: launcher.resource_timeout.as_millis().min(u64::MAX as u128) as u64,
            max_frame_bytes: launcher.max_frame_bytes,
            max_queued_messages: launcher.max_queued_messages,
            clock_epoch_offset_ms: boot.clock.anchor().0,
            clock_skipped_ms: boot.clock.anchor().1,
        };
        let (ack_tx, ack_rx) = sync_channel(1);
        let max = launcher.max_frame_bytes;
        let expected_backend = launcher.backend;
        let done = Arc::new(AtomicBool::new(false));
        let stopping = Arc::new(AtomicBool::new(false));
        let reader_done = done.clone();
        let reader_stopping = stopping.clone();
        let reader_failed = failed.clone();
        let writer_failed = failed;
        let writer_wake = boot.wake.clone();
        let reader = match std::thread::Builder::new()
            .name("genet-worker-ipc-reader".into())
            .spawn(move || {
                let ack = read_frame::<HelloAck>(&mut stdout, 64 * 1024).and_then(|ack| {
                    let ack = ack.ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "missing worker hello acknowledgement",
                        )
                    })?;
                    if ack.version != PROTOCOL_VERSION || ack.backend != expected_backend {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "worker protocol or backend mismatch",
                        ));
                    }
                    Ok(())
                });
                let accepted = ack.is_ok();
                let _ = ack_tx.send(ack);
                if !accepted {
                    reader_done.store(true, Ordering::Release);
                    return;
                }
                let mut closed = false;
                loop {
                    match read_frame::<FromWorker>(&mut stdout, max) {
                        Ok(Some(message)) => {
                            closed = matches!(message, FromWorker::Closed);
                            if boot.tx.send(message).is_err() {
                                break;
                            }
                            if let Some(wake) = boot.wake.as_ref() {
                                wake();
                            }
                            if closed {
                                break;
                            }
                        },
                        Ok(None) | Err(_) => break,
                    }
                }
                if !closed && !reader_stopping.load(Ordering::Acquire) {
                    reader_failed.store(true, Ordering::Release);
                }
                reader_done.store(true, Ordering::Release);
                if let Some(wake) = boot.wake {
                    wake();
                }
            }) {
            Ok(reader) => reader,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            },
        };
        let max = launcher.max_frame_bytes;
        let writer = match std::thread::Builder::new()
            .name("genet-worker-ipc-writer".into())
            .spawn(move || {
                if write_frame(&mut stdin, &hello, max).is_err() {
                    return;
                }
                while !done.load(Ordering::Acquire) {
                    match boot.rx.recv_timeout(Duration::from_millis(10)) {
                        Ok(message) => {
                            let terminate = matches!(message, ToWorker::Terminate);
                            if write_frame(&mut stdin, &message, max).is_err() {
                                writer_failed.store(true, Ordering::Release);
                                if let Some(wake) = writer_wake.as_ref() {
                                    wake();
                                }
                                break;
                            }
                            if terminate {
                                break;
                            }
                        },
                        Err(RecvTimeoutError::Timeout) => {},
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            }) {
            Ok(writer) => writer,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                return Err(error);
            },
        };
        match ack_rx.recv_timeout(launcher.startup_timeout) {
            Ok(Ok(())) => {},
            Ok(Err(error)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                let _ = writer.join();
                return Err(error);
            },
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                let _ = writer.join();
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "worker helper startup timeout",
                ));
            },
        }
        Ok(Self {
            child,
            writer: Some(writer),
            reader: Some(reader),
            grace: launcher.shutdown_grace,
            stopping,
            outcome: None,
        })
    }

    pub(crate) fn shutdown(&mut self, immediate: bool) -> ProcessExit {
        if let Some(outcome) = self.outcome {
            return outcome;
        }
        self.stopping.store(true, Ordering::Release);
        let started = Instant::now();
        if !immediate {
            while started.elapsed() < self.grace {
                match self.child.try_wait() {
                    Ok(Some(status)) => {
                        self.join_bridges();
                        let outcome = if status.success() {
                            ProcessExit::Graceful
                        } else {
                            ProcessExit::Crashed
                        };
                        self.outcome = Some(outcome);
                        return outcome;
                    },
                    Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                    Err(_) => break,
                }
            }
        }
        let outcome = match self.child.kill() {
            Ok(()) => match self.child.wait() {
                Ok(_) => ProcessExit::Killed,
                Err(_) => ProcessExit::Failed,
            },
            Err(_) => match self.child.try_wait() {
                Ok(Some(status)) if status.success() => ProcessExit::Graceful,
                Ok(Some(_)) => ProcessExit::Crashed,
                _ => ProcessExit::Failed,
            },
        };
        if outcome == ProcessExit::Failed {
            // A failed kill/reap may leave a live child holding both pipe ends.
            // Never block teardown by joining bridge threads in that case.
            self.writer.take();
            self.reader.take();
        } else {
            self.join_bridges();
        }
        self.outcome = Some(outcome);
        outcome
    }

    fn join_bridges(&mut self) {
        if let Some(writer) = self.writer.take() {
            let _ = writer.join();
        }
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Drop for WorkerProcessHandle {
    fn drop(&mut self) {
        if self.outcome.is_none() {
            let _ = self.shutdown(true);
        }
    }
}

/// Called by the concrete helper executable. The pipe reader is also the
/// parent's lifetime signal: EOF exits the process even if the VM is spinning.
pub fn run_worker_process<E: ScriptEngine>(backend: WorkerBackend) -> io::Result<()> {
    let mut stdin = io::stdin();
    let hello: Hello = read_frame(&mut stdin, 64 * 1024)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing worker hello"))?;
    if hello.version != PROTOCOL_VERSION
        || hello.backend != backend
        || hello.max_frame_bytes == 0
        || hello.max_frame_bytes > u32::MAX as usize
        || !(1..=1024).contains(&hello.max_queued_messages)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "worker protocol or backend mismatch",
        ));
    }
    write_frame(
        &mut io::stdout(),
        &HelloAck {
            version: PROTOCOL_VERSION,
            backend,
        },
        64 * 1024,
    )?;
    let (to_tx, to_rx) = std::sync::mpsc::sync_channel::<ToWorker>(hello.max_queued_messages);
    let (from_tx, from_rx) = std::sync::mpsc::sync_channel::<FromWorker>(hello.max_queued_messages);
    let max = hello.max_frame_bytes;
    let reader = std::thread::Builder::new()
        .name("genet-worker-stdin".into())
        .spawn(move || {
            loop {
                match read_frame::<ToWorker>(&mut stdin, max) {
                    Ok(Some(message)) => {
                        if to_tx.try_send(message).is_err() {
                            std::process::exit(2);
                        }
                    },
                    Ok(None) => std::process::exit(0),
                    Err(_) => std::process::exit(2),
                }
            }
        })?;
    let writer = std::thread::Builder::new()
        .name("genet-worker-stdout".into())
        .spawn(move || {
            let mut stdout = io::stdout();
            for message in from_rx {
                if write_frame(&mut stdout, &message, max).is_err() {
                    break;
                }
            }
        })?;
    worker_main::<E>(WorkerBoot {
        url: hello.url,
        name: hello.name,
        secure_context: hello.secure_context,
        rx: to_rx,
        tx: crate::worker::WorkerSender::Process {
            tx: from_tx,
            failed: Arc::new(AtomicBool::new(false)),
            child_side: true,
        },
        wake: None,
        // The process is the kill boundary. Nested workers must stay in it.
        policy: WorkerPolicy::TrustedCooperative {
            resource_timeout: Duration::from_millis(hello.resource_timeout_ms),
            shutdown_grace: Duration::ZERO,
        },
        clock: crate::SharedClock::from_anchor(hello.clock_epoch_offset_ms, hello.clock_skipped_ms),
    });
    let _ = writer.join();
    // stdin may still be open while the worker voluntarily closes. The process
    // is exiting, so do not block waiting for the reader.
    drop(reader);
    Ok(())
}
