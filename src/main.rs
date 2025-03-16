use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};
use erdp::ErrorDisplay;
use rustc_hash::FxHashMap;

use self::manifest::{ArgType, Project};

mod manifest;
mod script;

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
    let mut actions = FxHashMap::default();

    for (name, def) in manifest.commands {
        // Get command action.
        let mut cmd = Command::new(&name).about(def.description);
        let action = if let Some(v) = def.script {
            CommandAction::Script(v.into())
        } else {
            eprintln!("No action is configured for command '{name}'.");
            return ExitCode::FAILURE;
        };

        assert!(actions.insert(name, action).is_none());

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

            if let Some(v) = def.default {
                arg = arg.default_missing_value(v).num_args(0..=1);
            }

            cmd = cmd.arg(arg);
        }

        parser = parser.subcommand(cmd);
    }

    // Execute command.
    let args = parser.get_matches();
    let (cmd, args) = args.subcommand().unwrap();

    match actions.get(cmd).unwrap() {
        CommandAction::Script(script) => self::script::run(script, args),
    }
}

/// Action of a command.
enum CommandAction {
    Script(PathBuf),
}
