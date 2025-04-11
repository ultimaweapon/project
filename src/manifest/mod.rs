pub use self::cmd::*;
pub use self::script::*;

use rustc_hash::FxHashMap;
use serde::Deserialize;

mod cmd;
mod script;

/// Contains data deserialized from `Project.yml`.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Project {
    pub commands: FxHashMap<String, Command>,
}
