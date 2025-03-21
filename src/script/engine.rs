use std::ffi::{CStr, CString, c_char, c_int};
use std::mem::ManuallyDrop;
use std::num::TryFromIntError;
use std::ops::DerefMut;
use std::path::Path;
use std::process::ExitCode;

use thiserror::Error;

use super::LuaError;

/// Encapsulates a `lua_State`.
///
/// Any method that requires a mutable borrow of this struct indicate it is going to pop Lua stack.
pub struct Engine(*mut lua_State);

impl Engine {
    pub fn new() -> Self {
        Self(unsafe { engine_new() })
    }

    pub fn require_os(&self) {
        // SAFETY: 3 is maximum stack size used by engine_require_os.
        unsafe { engine_checkstack(self.0, 3) };
        unsafe { engine_require_os(self.0) };
    }

    pub fn load(&mut self, file: impl AsRef<Path>) -> Result<(), EngineError> {
        unsafe { engine_checkstack(self.0, 1) };

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
        unsafe { engine_checkstack(self.0, 1) };

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

    pub fn push_nil(&self) {
        unsafe { engine_checkstack(self.0, 1) };
        unsafe { engine_pushnil(self.0) };
    }

    pub fn push_string(&self, s: impl AsRef<CStr>) {
        unsafe { engine_checkstack(self.0, 1) };
        unsafe { engine_pushstring(self.0, s.as_ref().as_ptr()) };
    }

    /// # Panics
    /// If alignment of `f` greater than pointer size.
    pub fn push_fn<F: FnMut(&mut Self) -> c_int + 'static>(&self, f: F) {
        assert!(align_of::<F>() <= align_of::<*mut ()>());

        // SAFETY: 3 is maximum items we pushed here.
        unsafe { engine_checkstack(self.0, 3) };

        // Move Rust function to Lua user data.
        let ptr = unsafe { engine_newuserdatauv(self.0, size_of::<F>(), 1) };

        unsafe { ptr.cast::<F>().write(f) };

        // Set finalizer.
        unsafe extern "C-unwind" fn finalizer<F>(
            #[allow(non_snake_case)] L: *mut lua_State,
        ) -> c_int {
            let ptr = unsafe { engine_touserdata(L, 1).cast::<F>() };
            unsafe { std::ptr::drop_in_place(ptr) };
            0
        }

        if std::mem::needs_drop::<F>() {
            unsafe { engine_createtable(self.0, 0, 1) };
            unsafe { engine_pushcclosure(self.0, finalizer::<F>, 0) };
            unsafe { engine_setfield(self.0, -2, c"__gc".as_ptr()) };
            unsafe { engine_setmetatable(self.0, -1) };
        }

        // Push invoker.
        unsafe extern "C-unwind" fn invoker<F: FnMut(&mut Engine) -> c_int + 'static>(
            #[allow(non_snake_case)] L: *mut lua_State,
        ) -> c_int {
            let cb = unsafe { engine_upvalueindex(1) };
            let cb = unsafe { engine_touserdata(L, cb).cast::<F>() };

            unsafe { (*cb)(ManuallyDrop::new(Engine(L)).deref_mut()) }
        }

        unsafe { engine_pushcclosure(self.0, invoker::<F>, 1) };
    }

    pub fn top(&self) -> c_int {
        unsafe { engine_gettop(self.0) }
    }

    pub fn arg_string(&self, n: c_int) -> &CStr {
        let v = unsafe { engine_checkstring(self.0, n) };

        unsafe { CStr::from_ptr(v) }
    }

    pub fn arg_error(&self, n: c_int, m: impl AsRef<CStr>) -> ! {
        unsafe { engine_argerror(self.0, n, m.as_ref().as_ptr()) };
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
    /// `index` must be valid.
    pub unsafe fn is_nil(&self, index: c_int) -> bool {
        unsafe { engine_isnil(self.0, index) }
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
        let v = unsafe { CStr::from_ptr(v) };

        // SAFETY: engine_typename return a pointer to static storage.
        unsafe { engine_pop(self.0, 1) };

        // SAFETY: All type name in Lua are UTF-8.
        unsafe { std::str::from_utf8_unchecked(v.to_bytes()) }
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn pop(&mut self) {
        unsafe { engine_pop(self.0, 1) };
    }

    /// # Safety
    /// - `table` must be valid.
    /// - Top of Lua stack must have a value for this field.
    pub unsafe fn set_field(&mut self, table: c_int, key: impl AsRef<CStr>) {
        unsafe { engine_setfield(self.0, table, key.as_ref().as_ptr()) };
    }

    pub fn error<M>(&self, e: LuaError<M>) -> !
    where
        M: Into<String>,
    {
        unsafe { engine_error(self.0, e.to_lua().as_ptr()) };
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
pub struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    fn engine_new() -> *mut lua_State;
    fn engine_free(L: *mut lua_State);
    fn engine_require_os(L: *mut lua_State);
    fn engine_load(
        L: *mut lua_State,
        name: *const c_char,
        script: *const c_char,
        len: usize,
    ) -> bool;
    fn engine_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, msgh: c_int) -> bool;
    fn engine_checkstack(L: *mut lua_State, n: c_int);
    fn engine_pushnil(L: *mut lua_State);
    fn engine_pushstring(L: *mut lua_State, s: *const c_char) -> *const c_char;
    fn engine_pushcclosure(
        L: *mut lua_State,
        fp: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int,
        n: c_int,
    );
    fn engine_gettop(L: *mut lua_State) -> c_int;
    fn engine_checkstring(L: *mut lua_State, arg: c_int) -> *const c_char;
    fn engine_argerror(L: *mut lua_State, arg: c_int, extramsg: *const c_char) -> !;
    fn engine_isnil(L: *mut lua_State, index: c_int) -> bool;
    fn engine_tointegerx(L: *mut lua_State, index: c_int, isnum: *mut c_int) -> i64;
    fn engine_tostring(L: *mut lua_State, index: c_int) -> *const c_char;
    fn engine_touserdata(L: *mut lua_State, index: c_int) -> *mut u8;
    fn engine_typename(L: *mut lua_State, index: c_int) -> *const c_char;
    fn engine_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    fn engine_setfield(L: *mut lua_State, index: c_int, k: *const c_char);
    fn engine_newuserdatauv(L: *mut lua_State, size: usize, nuvalue: c_int) -> *mut u8;
    fn engine_setmetatable(L: *mut lua_State, index: c_int);
    fn engine_upvalueindex(i: c_int) -> c_int;
    fn engine_pop(L: *mut lua_State, n: c_int);
    fn engine_error(L: *mut lua_State, msg: *const c_char) -> !;
}
