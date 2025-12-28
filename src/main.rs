#![allow(clippy::await_holding_refcell_ref)] // We are single-threaded.
#![allow(clippy::new_ret_no_self)] // We need this for Lua userdata.

use self::api::{ArgsModule, GlobalModule, JsonModule, OsModule, PathModule, UrlModule};
use self::manifest::{ArgName, ArgType, CommandArg, Project, ScriptPath};
use clap::{Arg, ArgAction, ArgMatches, Command};
use erdp::ErrorDisplay;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{ExitCode, Termination};
use std::rc::Rc;
use tokio::task::LocalSet;
use tsuki::builtin::{CoroLib, IoLib, MathLib, StrLib, TableLib, Utf8Lib};
use tsuki::{CallError, Lua, ParseError};

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
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .disable_help_subcommand(true);
    let mut actions = FxHashMap::default();

    for (name, def) in manifest.commands {
        // Add command arguments.
        let mut cmd = Command::new(&name).about(def.description);

        for (id, def) in &def.args {
            let mut arg = Arg::new(id.as_ref().to_owned())
                .help(&def.description)
                .value_name(def.placeholder.clone().unwrap_or_else(|| id.to_uppercase()));

            match def.ty {
                ArgType::Bool => arg = arg.action(ArgAction::SetTrue),
                ArgType::String => (),
            }

            if let Some(v) = &def.long {
                arg = arg.long(v);
            }

            if let Some(v) = def.short {
                arg = arg.short(v);
            }

            if let Some(v) = &def.default {
                arg = arg.default_missing_value(v).num_args(0..=1);
            }

            cmd = cmd.arg(arg);
        }

        // Get command action.
        let action = if let Some(v) = def.script {
            CommandAction::Script(v, def.args)
        } else {
            return Exit::NoCommandAction(name);
        };

        assert!(actions.insert(name, action).is_none());

        parser = parser.subcommand(cmd);
    }

    // Execute command.
    let mut args = parser.get_matches();
    let (cmd, args) = args.remove_subcommand().unwrap();

    match actions.remove(&cmd).unwrap() {
        CommandAction::Script(script, defs) => run_script(script, defs, args),
    }
}

fn run_script(script: ScriptPath, defs: FxHashMap<ArgName, CommandArg>, args: ArgMatches) -> Exit {
    // Register modules.
    let lua = Lua::new(App {});

    lua.use_module(None, true, ArgsModule { defs, args })
        .unwrap();
    lua.use_module(None, true, GlobalModule).unwrap();
    lua.use_module(None, true, CoroLib).unwrap();
    lua.use_module(None, true, IoLib).unwrap();
    lua.use_module(None, true, JsonModule).unwrap();
    lua.use_module(None, true, MathLib).unwrap();
    lua.use_module(None, true, OsModule).unwrap();
    lua.use_module(None, true, PathModule).unwrap();
    lua.use_module(None, true, StrLib).unwrap();
    lua.use_module(None, true, TableLib).unwrap();
    lua.use_module(None, true, UrlModule).unwrap();
    lua.use_module(None, true, Utf8Lib).unwrap();

    // Setup Tokio.
    let tokio = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(v) => v,
        Err(e) => return Exit::SetupTokio(e),
    };

    // Execute the script.
    let local = LocalSet::new();

    local.block_on(&tokio, exec_script(lua, script))
}

async fn exec_script(lua: Pin<Rc<Lua<App>>>, script: ScriptPath) -> Exit {
    // Read script.
    let chunk = match std::fs::read(&script) {
        Ok(v) => v,
        Err(e) => return Exit::ReadScript(script, e),
    };

    // Load script.
    let chunk = match lua.load(script.as_str(), chunk) {
        Ok(v) => v,
        Err(e) => return Exit::LoadScript(script, e),
    };

    // Run the script.
    let td = lua.create_thread();

    match td.async_call(&chunk, ()).await {
        Ok(()) => Exit::ScriptResult(0),
        Err(e) => Exit::RunScript(script, e),
    }
}

/// Associated data of [Lua].
struct App {}

/// Action of a command.
enum CommandAction {
    Script(ScriptPath, FxHashMap<ArgName, CommandArg>),
}

/// Exit code of Project.
#[repr(u8)]
enum Exit {
    ScriptResult(u8),
    RunScript(ScriptPath, Box<dyn std::error::Error>) = 100,
    OpenProject(PathBuf, std::io::Error) = 102, // 101 is Rust panic.
    LoadProject(PathBuf, serde_yaml::Error) = 103,
    NoCommandAction(String) = 104,
    ReadScript(ScriptPath, std::io::Error) = 105,
    LoadScript(ScriptPath, ParseError) = 106,
    SetupTokio(std::io::Error) = 109,
}

impl Termination for Exit {
    fn report(self) -> ExitCode {
        use std::error::Error;

        // SAFETY: This is safe since Exit marked with `repr(u8)`. See
        // https://doc.rust-lang.org/std/mem/fn.discriminant.html for more details.
        let mut code = unsafe { (&self as *const Self as *const u8).read() };

        match self {
            Self::ScriptResult(v) => code = v,
            Self::RunScript(p, e) => match e.downcast::<CallError>() {
                Ok(e) => match e.source().and_then(|e| e.downcast_ref::<self::api::Exit>()) {
                    Some(e) => code = e.code(),
                    None => {
                        let (f, l) = e.location().unwrap();

                        eprintln!("{}:{}: {}.", f, l, e.display());
                    }
                },
                Err(e) => eprintln!("Failed to run {}: {}.", p, e.display()),
            },
            Self::OpenProject(p, e) => {
                eprintln!("Failed to open {}: {}.", p.display(), e.display())
            }
            Self::LoadProject(p, e) => {
                eprintln!("Failed to load {}: {}.", p.display(), e.display())
            }
            Self::NoCommandAction(n) => eprintln!("No action is configured for command '{n}'."),
            Self::ReadScript(p, e) => {
                eprintln!("Failed to read {}: {}.", p, e.display())
            }
            Self::LoadScript(p, e) => eprintln!("{}:{}: {}.", p, e.line(), e.display()),
            Self::SetupTokio(e) => eprintln!("Failed to setup Tokio: {}.", e.display()),
        }

        code.into()
    }
}
