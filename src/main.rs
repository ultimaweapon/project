#![allow(clippy::await_holding_refcell_ref)] // We are single-threaded.

use self::manifest::{ArgName, ArgType, CommandArg, Project, ScriptPath};
use clap::{Arg, ArgAction, ArgMatches, Command};
use erdp::ErrorDisplay;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Termination};
use tokio::task::LocalSet;
use zl::{Async, AsyncThread, ChunkType, Frame, Lua};

mod api;
mod manifest;

fn main() -> Exit {
    // Enable UTF-8 for CRT on Windows.
    #[cfg(windows)]
    if unsafe { libc::setlocale(libc::LC_ALL, c".UTF8".as_ptr()).is_null() } {
        return Exit::SetLocale;
    }

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
    // Register standard libraries that does not require special handling.
    let mut lua = match Lua::new(None) {
        Some(v) => v,
        None => return Exit::CreateLua,
    };

    lua.require_base();
    lua.require_coroutine(true);
    lua.require_io(true);
    lua.require_math(true);
    lua.require_string(true);
    lua.require_table(true);
    lua.require_utf8(true);

    // Register "os" library.
    let mut t = lua.require_os(true);

    t.set(c"exit").push_nil();
    t.set(c"setlocale").push_nil();

    self::api::os::register(t);

    // Register other APIs.
    self::api::args::register(&mut lua, defs, args);
    self::api::buffer::register(&mut lua);
    self::api::url::register(&mut lua);

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
    let lua = lua.into_async();

    local.block_on(&tokio, exec_script(lua.spawn(), script))
}

async fn exec_script(mut td: AsyncThread, script: ScriptPath) -> Exit {
    // Load script.
    let chunk = match td.load_file(&script, ChunkType::Text) {
        Ok(Ok(v)) => v,
        Ok(Err(mut e)) => return Exit::LoadScript(e.to_c_str().to_string_lossy().into_owned()),
        Err(e) => return Exit::ReadScript(script, e),
    };

    // Run the script.
    let mut chunk = chunk.into_async();
    let mut r = loop {
        match chunk.resume().await {
            Ok(Async::Yield(_)) => (),
            Ok(Async::Finish(v)) => break v,
            Err(mut e) => return Exit::RunScript(e.to_c_str().to_string_lossy().into_owned()),
        }
    };

    // Get result.
    let r = if r.is_empty() {
        return Exit::ScriptResult(0);
    } else if let Some(v) = r.to_int(1) {
        v
    } else {
        return Exit::InvalidResult(r.to_type(1).into());
    };

    match r {
        0..=99 => r.try_into().map(Exit::ScriptResult).unwrap(),
        v => Exit::ResultOurOfRange(v),
    }
}

/// Action of a command.
enum CommandAction {
    Script(ScriptPath, FxHashMap<ArgName, CommandArg>),
}

/// Exit code of Project.
#[repr(u8)]
enum Exit {
    ScriptResult(u8),
    RunScript(String) = 100,
    OpenProject(PathBuf, std::io::Error) = 102, // 101 is Rust panic.
    LoadProject(PathBuf, serde_yaml::Error) = 103,
    NoCommandAction(String) = 104,
    ReadScript(ScriptPath, std::io::Error) = 105,
    LoadScript(String) = 106,
    InvalidResult(&'static str) = 107,
    ResultOurOfRange(i64) = 108,
    SetupTokio(std::io::Error) = 109,
    CreateLua = 110,
    #[cfg(windows)]
    SetLocale = 111,
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
                eprintln!("Failed to read {}: {}.", p, e.display())
            }
            Self::LoadScript(v) => eprintln!("{v}"),
            Self::InvalidResult(v) => {
                eprintln!("expect script to return an integer, got {v}")
            }
            Self::ResultOurOfRange(v) => {
                eprintln!("expect script to return either nil or integer between 0 - 99, got {v}")
            }
            Self::SetupTokio(e) => eprintln!("Failed to setup Tokio: {}.", e.display()),
            Self::CreateLua => eprintln!("Failed to create lua_State."),
            #[cfg(windows)]
            Self::SetLocale => eprintln!("Failed to enable UTF-8 locale on CRT."),
        }

        code.into()
    }
}
