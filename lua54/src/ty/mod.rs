use crate::ffi::lua54_typename;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::ptr::null_mut;

/// Type of Lua value.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    None = -1,
    Nil,
    Boolean,
    LightUserData,
    Number,
    String,
    Table,
    Function,
    UserData,
    Thread,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str((*self).into())
    }
}

impl From<Type> for &'static str {
    fn from(value: Type) -> Self {
        // SAFETY: Lua does not use L.
        let v = unsafe { lua54_typename(null_mut(), value) };
        let v = unsafe { CStr::from_ptr(v) };

        // SAFETY: All type name returned from lua54_typename are UTF-8 and has static storage.
        unsafe { std::str::from_utf8_unchecked(v.to_bytes()) }
    }
}
