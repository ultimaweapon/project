use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Termination};

use clap::{Arg, ArgAction, ArgMatches, Command};
use erdp::ErrorDisplay;
use lua54::Engine;
use rustc_hash::FxHashMap;

use self::manifest::{ArgType, Project};

mod api;
mod manifest;

fn main() -> Exit {
    // Open Project.yml.
    let path = Path::new("Project.yml");
    let manifest = match File::open(path) {
        Ok(v) => v,
        Err(e) => return Exit::OpenProject(path.into(), e),
    };

    // Load Project.yml.
    let manifest: Project = match serde_yaml::from_reader(manifest) {
        Ok(v) => v,
        Err(e) => return Exit::LoadProject(path.into(), e),
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
            return Exit::NoCommandAction(name);
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

    match actions.remove(cmd).unwrap() {
        CommandAction::Script(script) => run_script(script, args),
    }
}

fn run_script(script: PathBuf, _: &ArgMatches) -> Exit {
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

    // Register "buffer" APIs.
    crate::api::buffer::register(&mut en);

    // Load script.
    match en.load(&script) {
        Ok(true) => (),
        Ok(false) => return Exit::LoadScript(unsafe { en.pop_string_lossy().unwrap() }),
        Err(e) => return Exit::ReadScript(script, e),
    }

    // Run the script.
    if !unsafe { en.run(0, 1, 0) } {
        return Exit::RunScript(unsafe { en.pop_string_lossy().unwrap() });
    }

    // Get result.
    let r = if let Some(v) = unsafe { en.pop_int() } {
        v
    } else if let Some(_) = unsafe { en.pop_nil() } {
        return Exit::ScriptResult(0);
    } else {
        return Exit::InvalidResult(unsafe { en.pop_type() });
    };

    match r {
        0..=99 => r.try_into().map(Exit::ScriptResult).unwrap(),
        v => Exit::ResultOurOfRange(v),
    }
}

/// Action of a command.
enum CommandAction {
    Script(PathBuf),
}

/// Exit code of Project.
#[repr(u8)]
enum Exit {
    ScriptResult(u8),
    RunScript(String) = 100,
    OpenProject(PathBuf, std::io::Error) = 102, // 101 is Rust panic.
    LoadProject(PathBuf, serde_yaml::Error) = 103,
    NoCommandAction(String) = 104,
    ReadScript(PathBuf, std::io::Error) = 105,
    LoadScript(String) = 106,
    InvalidResult(&'static str) = 107,
    ResultOurOfRange(i64) = 108,
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        // SAFETY: This is safe since Exit marked with `repr(u8)`. See
        // https://doc.rust-lang.org/std/mem/fn.discriminant.html for more details.
        let mut code = unsafe { (&self as *const Self as *const u8).read() };

        match self {
            Self::ScriptResult(v) => code = v,
            Self::RunScript(v) => eprintln!("{v}"),
            Self::OpenProject(p, e) => {
                eprintln!("Failed to open {}: {}.", p.display(), e.display())
            }
            Self::LoadProject(p, e) => {
                eprintln!("Failed to load {}: {}.", p.display(), e.display())
            }
            Self::NoCommandAction(n) => eprintln!("No action is configured for command '{n}'."),
            Self::ReadScript(p, e) => {
                eprintln!("Failed to read {}: {}.", p.display(), e.display())
            }
            Self::LoadScript(v) => eprintln!("{v}"),
            Self::InvalidResult(v) => {
                eprintln!("expect script to return either nil or integer, got {v}")
            }
            Self::ResultOurOfRange(v) => {
                eprintln!("expect script to return either nil or integer between 0 - 99, got {v}")
            }
        }

        code.into()
    }
}
