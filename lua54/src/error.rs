use std::error::Error;
use std::ffi::CString;
use std::fmt::Write;

/// Represents an error when Lua function that defined on Rust side fails.
pub struct LuaError<'a, M> {
    msg: M,
    src: Option<&'a dyn Error>,
}

impl<'a, M> LuaError<'a, M>
where
    M: Into<String>,
{
    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn new(msg: M) -> Self {
        Self { msg, src: None }
    }

    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn with_source(msg: M, src: &'a dyn Error) -> Self {
        Self {
            msg,
            src: Some(src),
        }
    }

    pub(super) fn to_lua(self) -> CString {
        let mut msg = self.msg.into();
        let mut src = self.src;

        while let Some(e) = src {
            write!(msg, " -> {e}").unwrap();
            src = e.source();
        }

        CString::new(msg).unwrap()
    }
}
