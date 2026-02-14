use super::TrimmedNonEmpty;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::borrow::{Borrow, Cow};
use std::ops::Deref;

/// Name of command argument.
#[derive(PartialEq, Eq, Hash)]
pub struct ArgName(String);

impl<'a> Deserialize<'a> for ArgName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let val = Cow::<str>::deserialize(deserializer)?;

        if val == "help" {
            return Err(Error::custom("reserved name"));
        }

        Ok(Self(val.into_owned()))
    }
}

impl Deref for ArgName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ArgName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for ArgName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

/// Command argument definition.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CommandArg {
    pub description: String,
    pub long: Option<String>,
    pub short: Option<char>,
    #[serde(rename = "type")]
    pub ty: ArgType,
    pub allowed_values: Option<AllowedValues>,
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

/// Allowed values for command argument.
pub struct AllowedValues(Vec<TrimmedNonEmpty>);

impl AllowedValues {
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter().map(|v| &v.0)
    }
}

impl<'de> Deserialize<'de> for AllowedValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Vec::<TrimmedNonEmpty>::deserialize(deserializer)?;

        if val.is_empty() {
            return Err(Error::custom("sequence cannot be empty"));
        }

        Ok(Self(val))
    }
}
