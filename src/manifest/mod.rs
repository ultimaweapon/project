pub use self::arg::*;
pub use self::script::*;

use rustc_hash::FxHashMap;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;

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

/// Non-empty string with white spaces trimmed.
pub struct TrimmedNonEmpty(String);

impl<'de> Deserialize<'de> for TrimmedNonEmpty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Cow::<str>::deserialize(deserializer)?;
        let val = val.trim();

        if val.is_empty() {
            return Err(Error::custom("value cannot be empty"));
        }

        Ok(Self(val.into()))
    }
}
