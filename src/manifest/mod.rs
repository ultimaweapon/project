pub use self::arg::*;
pub use self::script::*;

use rustc_hash::FxHashMap;
use serde::Deserialize;

mod arg;
mod script;

/// Contains data deserialized from `Project.yml`.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Project {
    pub commands: FxHashMap<String, Command>,
}

/// Project command.
#[derive(Deserialize)]
pub struct Command {
    pub description: String,
    #[serde(default)]
    pub args: FxHashMap<ArgName, CommandArg>,
    pub script: Option<ScriptPath>,
}
