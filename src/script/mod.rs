use std::path::PathBuf;
use std::process::ExitCode;

use clap::ArgMatches;

use self::engine::Engine;

mod engine;

pub fn run(_: &PathBuf, _: &ArgMatches) -> ExitCode {
    Engine::new();
    ExitCode::SUCCESS
}
