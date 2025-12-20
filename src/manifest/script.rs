use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::Path;

/// Path of command script.
pub struct ScriptPath(String);

impl ScriptPath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'a> Deserialize<'a> for ScriptPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let raw = Cow::<str>::deserialize(deserializer)?;
        let native = if cfg!(unix) {
            raw.into_owned()
        } else {
            let mut buf = String::with_capacity(raw.len());

            for c in raw.split('/') {
                if !buf.is_empty() {
                    buf.push('\\');
                }

                buf.push_str(c);
            }

            buf
        };

        Ok(Self(native))
    }
}

impl AsRef<str> for ScriptPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<Path> for ScriptPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl Display for ScriptPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
