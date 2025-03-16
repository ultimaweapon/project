use rustc_hash::FxHashMap;
use serde::Deserialize;

/// Project command.
#[derive(Deserialize)]
pub struct Command {
    pub description: String,
    #[serde(default)]
    pub args: FxHashMap<String, CommandArg>,
}

/// Command argument definition.
#[derive(Deserialize)]
pub struct CommandArg {
    pub description: String,
    pub long: Option<String>,
    pub short: Option<char>,
    #[serde(rename = "type")]
    pub ty: ArgType,
    pub placeholder: Option<String>,
    pub default: Option<String>,
}

/// Type of command argument.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArgType {
    Bool,
    String,
}
