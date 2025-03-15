use std::fs::File;
use std::path::Path;
use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};
use erdp::ErrorDisplay;

use self::manifest::{ArgType, Project};

mod manifest;

fn main() -> ExitCode {
    // Open Project.yml.
    let path = Path::new("Project.yml");
    let manifest = match File::open(path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open {}: {}.", path.display(), e.display());
            return ExitCode::FAILURE;
        }
    };

    // Load Project.yml.
    let manifest: Project = match serde_yaml::from_reader(manifest) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to load {}: {}.", path.display(), e.display());
            return ExitCode::FAILURE;
        }
    };

    // Build arguments parser.
    let mut parser = Command::new("Project")
        .about("Run a command defined in Project.yml")
        .subcommand_required(true);

    for (name, def) in manifest.commands {
        let mut cmd = Command::new(name).about(def.description);

        // Add command arguments.
        for (id, def) in def.args {
            let mut arg = Arg::new(&id)
                .help(def.description)
                .value_name(def.placeholder.unwrap_or_else(|| id.to_uppercase()));

            match def.ty {
                ArgType::Bool => arg = arg.action(ArgAction::SetTrue),
                ArgType::String => (),
            }

            if let Some(v) = def.long {
                arg = arg.long(v);
            }

            if let Some(v) = def.short {
                arg = arg.short(v);
            }

            cmd = cmd.arg(arg);
        }

        parser = parser.subcommand(cmd);
    }

    // Parse arguments.
    parser.get_matches();

    ExitCode::SUCCESS
}
