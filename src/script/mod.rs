use std::path::PathBuf;
use std::process::ExitCode;

use clap::ArgMatches;
use erdp::ErrorDisplay;

use self::engine::{Engine, EngineError};

mod engine;

pub fn run(script: &PathBuf, _: &ArgMatches) -> ExitCode {
    // Load script.
    let mut engine = Engine::new();

    match engine.load(script) {
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

    ExitCode::SUCCESS
}
