use std::ffi::{CStr, CString, c_char, c_int};
use std::num::TryFromIntError;
use std::path::Path;
use std::process::ExitCode;

use thiserror::Error;

/// Encapsulates a `lua_State`.
///
/// Any method that requires a mutable borrow of this struct indicate it is going to pop Lua stack.
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
            return unsafe { Err(EngineError::LoadFile(self.pop_string_lossy().unwrap())) };
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<ExitCode, EngineError> {
        // Run.
        if !unsafe { engine_pcall(self.0, 0, 1, 0) } {
            return unsafe { Err(EngineError::RunScript(self.pop_string_lossy().unwrap())) };
        }

        // Get result.
        if let Some(v) = unsafe { self.pop_int() } {
            u8::try_from(v)
                .map_err(|e| EngineError::ConvertResult(v, e))
                .map(ExitCode::from)
        } else if let Some(_) = unsafe { self.pop_nil() } {
            Ok(ExitCode::SUCCESS)
        } else {
            unsafe { Err(EngineError::InvalidResult(self.pop_type())) }
        }
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn pop_nil(&mut self) -> Option<()> {
        if !unsafe { engine_isnil(self.0, -1) } {
            return None;
        }

        unsafe { engine_pop(self.0, 1) };
        Some(())
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn pop_int(&mut self) -> Option<i64> {
        let v = unsafe { self.to_int(-1)? };
        unsafe { engine_pop(self.0, 1) };
        Some(v)
    }

    /// # Safety
    /// `index` must be valid.
    pub unsafe fn to_int(&self, index: c_int) -> Option<i64> {
        let mut ok = 0;
        let val = unsafe { engine_tointegerx(self.0, index, &mut ok) };

        if ok == 0 { None } else { Some(val) }
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn pop_string_lossy(&mut self) -> Option<String> {
        let v = unsafe { self.to_string(-1)?.to_string_lossy().into_owned() };

        // SAFETY: We already converted the borrowed CStr to String on the above.
        unsafe { engine_pop(self.0, 1) };

        Some(v)
    }

    /// # Safety
    /// `index` must be valid and not a key from `lua_next`.
    pub unsafe fn to_string(&self, index: c_int) -> Option<&CStr> {
        let v = unsafe { engine_tostring(self.0, index) };

        if v.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(v) })
        }
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn pop_type(&mut self) -> &'static str {
        let v = unsafe { engine_typename(self.0, -1) };

        // SAFETY: engine_typename return a pointer to static storage.
        unsafe { engine_pop(self.0, 1) };

        unsafe { CStr::from_ptr(v).to_str().unwrap() }
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

    #[error("{0}")]
    RunScript(String),

    #[error("couldn't convert script result {0} to exit code")]
    ConvertResult(i64, #[source] TryFromIntError),

    #[error("expect script to return either nil or integer, got {0}")]
    InvalidResult(&'static str),
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    fn engine_new() -> *mut lua_State;
    fn engine_free(L: *mut lua_State);
    fn engine_load(
        L: *mut lua_State,
        name: *const c_char,
        script: *const c_char,
        len: usize,
    ) -> bool;
    fn engine_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, msgh: c_int) -> bool;
    fn engine_isnil(L: *mut lua_State, index: c_int) -> bool;
    fn engine_tointegerx(L: *mut lua_State, index: c_int, isnum: *mut c_int) -> i64;
    fn engine_tostring(L: *mut lua_State, index: c_int) -> *const c_char;
    fn engine_typename(L: *mut lua_State, index: c_int) -> *const c_char;
    fn engine_pop(L: *mut lua_State, n: c_int);
}
