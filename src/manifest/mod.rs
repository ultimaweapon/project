pub use self::cmd::*;

use rustc_hash::FxHashMap;
use serde::Deserialize;

mod cmd;

/// Contains data deserialized from `Project.yml`.
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Project {
    pub commands: FxHashMap<String, Command>,
}
