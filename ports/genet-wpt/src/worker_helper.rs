/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The native helper that hosts dedicated workers for the testharness lane.
//!
//! The runtime's default `WorkerPolicy::RequireCancellation` admits a worker
//! only through a killable helper process, so every `Runtime` the lane builds
//! gets a launcher when the helper resolves. Resolution happens in each
//! process (the `testharness-one` children inherit the environment and share
//! this executable's directory), so nothing is passed between them.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use script_engine_api::ScriptEngine;
use script_runtime_api::{WorkerBackend, WorkerProcessLauncher};

const HELPER_ENV: &str = "GENET_WORKER_HELPER";
const HELPER_NAME: &str = "genet-worker-process";
/// The largest outbound queue the launcher accepts. The helper exits when it
/// overflows, and a worker running a testharness file bursts a few messages
/// per subtest, so the default of 32 killed workers with more than a dozen.
const MAX_QUEUED_MESSAGES: usize = 1024;

/// `GENET_WORKER_HELPER` when it names an existing absolute path, else the
/// helper beside `exe`.
fn resolve(env: Option<PathBuf>, exe: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = env.filter(|path| path.is_absolute() && path.is_file()) {
        return Some(path);
    }
    let sibling = exe?
        .parent()?
        .join(format!("{HELPER_NAME}{}", std::env::consts::EXE_SUFFIX));
    sibling.is_file().then_some(sibling)
}

fn helper() -> Option<&'static Path> {
    static HELPER: OnceLock<Option<PathBuf>> = OnceLock::new();
    HELPER
        .get_or_init(|| {
            resolve(
                std::env::var_os(HELPER_ENV).map(PathBuf::from),
                std::env::current_exe().ok().as_deref(),
            )
        })
        .as_deref()
}

/// One stderr line when no helper resolves; worker variants then error with
/// the runtime's `NotSupportedError`.
pub(crate) fn announce_if_missing() {
    if helper().is_none() {
        eprintln!(
            "genet-wpt: no worker helper found; dedicated-worker variants will error. \
             Set {HELPER_ENV} to the absolute path of {HELPER_NAME}, or place it beside \
             this executable (cargo build -p genet-worker)."
        );
    }
}

/// The launcher for engine `E`, or `None` when no helper resolves or the
/// engine has no native worker backend.
pub(crate) fn launcher<E: ScriptEngine>() -> Option<WorkerProcessLauncher> {
    let backend = match E::native_worker_backend()? {
        "boa" => WorkerBackend::Boa,
        "nova" => WorkerBackend::Nova,
        _ => return None,
    };
    let mut launcher = WorkerProcessLauncher::new(helper()?.to_path_buf(), backend);
    launcher.max_queued_messages = MAX_QUEUED_MESSAGES;
    Some(launcher)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, b"").unwrap();
        path
    }

    fn scratch(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genet-wpt-helper-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn env_wins_over_the_sibling() {
        let dir = scratch("env");
        let exe = touch(&dir, "genet-wpt");
        touch(
            &dir,
            &format!("{HELPER_NAME}{}", std::env::consts::EXE_SUFFIX),
        );
        let chosen = touch(&dir, "chosen-helper");
        assert_eq!(resolve(Some(chosen.clone()), Some(&exe)), Some(chosen));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_relative_or_missing_env_falls_back_to_the_sibling() {
        let dir = scratch("sibling");
        let exe = touch(&dir, "genet-wpt");
        let sibling = touch(
            &dir,
            &format!("{HELPER_NAME}{}", std::env::consts::EXE_SUFFIX),
        );
        assert_eq!(
            resolve(Some("relative/helper".into()), Some(&exe)),
            Some(sibling.clone())
        );
        assert_eq!(
            resolve(Some(dir.join("gone")), Some(&exe)),
            Some(sibling.clone())
        );
        assert_eq!(resolve(None, Some(&exe)), Some(sibling));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn nothing_resolves_without_either() {
        let dir = scratch("none");
        let exe = touch(&dir, "genet-wpt");
        assert_eq!(resolve(None, Some(&exe)), None);
        assert_eq!(resolve(None, None), None);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
