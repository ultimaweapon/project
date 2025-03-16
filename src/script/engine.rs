use std::ffi::{CStr, CString, c_char, c_int};
use std::path::Path;

use thiserror::Error;

/// Encapsulates a `lua_State`.
///
/// Any method that requires a mutable borrow of this struct indicate it is going to change the
/// depth of Lua stack.
pub struct Engine(*mut lua_State);

impl Engine {
    pub fn new() -> Self {
        Self(unsafe { engine_new() })
    }

    pub fn load(&mut self, file: impl AsRef<Path>) -> Result<(), EngineError> {
        // Load.
        let file = file.as_ref();
        let script = std::fs::read_to_string(file).map_err(EngineError::ReadFile)?;
        let name = CString::new(format!("@{}", file.to_string_lossy())).unwrap();

        if !unsafe { engine_load(self.0, name.as_ptr(), script.as_ptr().cast(), script.len()) } {
            return Err(EngineError::LoadFile(self.pop_string_lossy().unwrap()));
        }

        Ok(())
    }

    pub fn pop_string_lossy(&mut self) -> Option<String> {
        let v = unsafe { self.to_string(-1)?.to_string_lossy().into_owned() };

        // SAFETY: We already converted the borrowed CStr to String on the above.
        unsafe { engine_pop(self.0, 1) };

        Some(v)
    }

    /// # Safety
    /// `index` must be valid and not a key from `lua_next`.
    pub unsafe fn to_string(&self, index: c_int) -> Option<&CStr> {
        let v = unsafe { engine_to_string(self.0, index) };

        if v.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(v) })
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { engine_free(self.0) };
    }
}

/// Represents an error when [`Engine`] fails.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("couldn't read the file")]
    ReadFile(#[source] std::io::Error),

    #[error("{0}")]
    LoadFile(String),
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    fn engine_new() -> *mut lua_State;
    fn engine_free(L: *mut lua_State);
    fn engine_pop(L: *mut lua_State, n: c_int);
    fn engine_load(
        L: *mut lua_State,
        name: *const c_char,
        script: *const c_char,
        len: usize,
    ) -> bool;
    fn engine_to_string(L: *mut lua_State, index: c_int) -> *const c_char;
}
