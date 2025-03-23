use std::borrow::Cow;
use std::ffi::{CStr, c_int};
use std::fmt::Write;

/// Represents an error when Lua function that defined on Rust side fails.
///
/// The message will be silently truncated if its contain a NUL byte.
pub struct Error(ErrorKind);

impl Error {
    /// # Panics
    /// If `arg` is zero or negative.
    pub fn ty(arg: c_int, expect: impl Into<ErrorMsg>) -> Self {
        assert!(arg > 0);

        Self(ErrorKind::ArgType(arg, expect.into().into()))
    }

    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg_from_std(arg: c_int, e: impl std::error::Error) -> Self {
        let mut msg = e.to_string();
        let mut src = e.source();

        while let Some(e) = src {
            write!(msg, " -> {e}").unwrap();
            src = e.source();
        }

        Self::arg(arg, msg)
    }

    /// # Panics
    /// If `arg` is zero or negative.
    pub fn arg(arg: c_int, msg: impl Into<ErrorMsg>) -> Self {
        assert!(arg > 0);

        Self(ErrorKind::Arg(arg, msg.into().into()))
    }

    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn other(msg: impl Into<ErrorMsg>) -> Self {
        Self(ErrorKind::Other(msg.into().into()))
    }

    /// `msg` are typically concise lowercase sentences without trailing punctuation (e.g. `failed
    /// to open 'foo'`).
    pub fn with_source(msg: impl Into<String>, src: impl std::error::Error) -> Self {
        let mut msg = msg.into();
        let mut src: Option<&dyn std::error::Error> = Some(&src);

        while let Some(e) = src {
            write!(msg, " -> {e}").unwrap();
            src = e.source();
        }

        Self::other(msg)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::other(value)
    }
}

impl<E: std::error::Error> From<(String, E)> for Error {
    fn from(value: (String, E)) -> Self {
        Self::with_source(value.0, value.1)
    }
}

/// Encapsulates a message for [`Error`].
pub enum ErrorMsg {
    Static(&'static CStr),
    String(String),
}

impl From<&'static CStr> for ErrorMsg {
    fn from(value: &'static CStr) -> Self {
        Self::Static(value)
    }
}

impl From<String> for ErrorMsg {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<ErrorMsg> for Cow<'static, [u8]> {
    fn from(value: ErrorMsg) -> Self {
        let mut v = match value {
            ErrorMsg::Static(v) => return Cow::Borrowed(v.to_bytes_with_nul()),
            ErrorMsg::String(v) => v.into_bytes(),
        };

        v.push(0);

        Cow::Owned(v)
    }
}

/// Kind of [`Error`].
pub(crate) enum ErrorKind {
    /// # Safety
    /// Second value must null-terminated.
    Arg(c_int, Cow<'static, [u8]>),
    /// # Safety
    /// - First value must be positive.
    /// - Second value must null-terminated.
    ArgType(c_int, Cow<'static, [u8]>),
    /// # Safety
    /// The value must null-terminated.
    Other(Cow<'static, [u8]>),
}

impl From<Error> for ErrorKind {
    fn from(value: Error) -> Self {
        value.0
    }
}
