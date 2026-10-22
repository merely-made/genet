/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_runtime_api::{WorkerBackend, run_worker_process};

fn main() {
    let mut args = std::env::args().skip(1);
    let result = match (args.next().as_deref(), args.next().as_deref(), args.next()) {
        (Some("--engine"), Some("boa"), None) => {
            run_worker_process::<script_engine_boa::BoaEngine>(WorkerBackend::Boa)
        },
        #[cfg(target_pointer_width = "64")]
        (Some("--engine"), Some("nova"), None) => {
            run_worker_process::<script_engine_nova::NovaEngine>(WorkerBackend::Nova)
        },
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "expected --engine boa|nova",
        )),
    };
    if let Err(error) = result {
        eprintln!("worker helper: {error}");
        std::process::exit(1);
    }
}
