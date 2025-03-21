pub use self::engine::*;
pub use self::error::*;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ArgMatches;
use erdp::ErrorDisplay;

mod engine;
mod error;

pub fn run(script: &PathBuf, _: &ArgMatches) -> ExitCode {
    // Register "os" library.
    let mut en = Engine::new();

    en.require_os();

    // Remove "exit" and "setlocale".
    en.push_nil();
    unsafe { en.set_field(-2, c"exit") };
    en.push_nil();
    unsafe { en.set_field(-2, c"setlocale") };

    // Register "os" APIs.
    crate::api::os::register(&mut en);
    unsafe { en.pop() };

    // Load script.
    match en.load(script) {
        Ok(_) => (),
        Err(EngineError::LoadFile(v)) => {
            eprintln!("{v}");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("Failed to load {}: {}.", script.display(), e.display());
            return ExitCode::FAILURE;
        }
    }

    // Run the script.
    match en.run() {
        Ok(v) => v,
        Err(EngineError::RunScript(v)) => {
            eprintln!("{v}");
            return ExitCode::FAILURE;
        }
        Err(e) => {
            eprintln!("Failed to run {}: {}.", script.display(), e.display());
            return ExitCode::FAILURE;
        }
    }
}
