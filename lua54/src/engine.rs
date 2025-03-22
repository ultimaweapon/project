use std::ffi::{CStr, CString, c_int};
use std::mem::ManuallyDrop;
use std::ops::DerefMut;
use std::panic::UnwindSafe;
use std::path::Path;

use crate::ffi::{
    engine_argerror, engine_checkstack, engine_checkstring, engine_createtable, engine_error,
    engine_free, engine_gettop, engine_isnil, engine_load, engine_newuserdatauv, engine_pcall,
    engine_pop, engine_pushcclosure, engine_pushnil, engine_pushstring, engine_require_os,
    engine_setfield, engine_setmetatable, engine_tointegerx, engine_tostring, engine_touserdata,
    engine_typename, engine_upvalueindex, lua_State, lua54_getfield, lua54_geti, lua54_istable,
    lua54_newstate, lua54_setglobal, lua54_typeerror,
};
use crate::{LuaError, Type};

/// Encapsulates a `lua_State`.
///
/// Any method that requires a mutable borrow of this struct indicate it is going to pop Lua stack.
///
/// All methods in this struct may raise a C++ exception. When calling outside Lua runtime it will
/// cause the process to terminate the same as Rust panic. Inside Lua runtime it will report as Lua
/// error.
pub struct Engine(*mut lua_State);

impl Engine {
    pub fn new() -> Self {
        Self(lua54_newstate())
    }

    pub fn require_os(&self) {
        // SAFETY: 3 is maximum stack size used by engine_require_os.
        unsafe { engine_checkstack(self.0, 3) };
        unsafe { engine_require_os(self.0) };
    }

    /// This method will load the whole content of `file` into memory before passing to Lua.
    pub fn load(&mut self, file: impl AsRef<Path>) -> Result<bool, std::io::Error> {
        // SAFETY: engine_load return either error object or a chunk.
        unsafe { engine_checkstack(self.0, 1) };

        // Read file.
        let file = file.as_ref();
        let script = std::fs::read(file)?;

        // Get chunk name.
        let file = file.to_string_lossy();
        let mut name = String::with_capacity(1 + file.len() + 1);

        name.push('@');
        name.push_str(&file);
        name.push('\0');

        // Load.
        let name = name.as_ptr().cast();

        Ok(unsafe { engine_load(self.0, name, script.as_ptr().cast(), script.len()) })
    }

    pub unsafe fn run(&mut self, nargs: c_int, nresults: c_int, msgh: c_int) -> bool {
        unsafe { engine_checkstack(self.0, nresults) };
        unsafe { engine_pcall(self.0, nargs, nresults, msgh) }
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
    pub fn push_fn<F>(&self, f: F)
    where
        F: FnMut(&mut Self) -> c_int + UnwindSafe + 'static,
    {
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

    pub fn push_table(&self, narr: c_int, nrec: c_int) {
        unsafe { engine_checkstack(self.0, 1) };
        unsafe { engine_createtable(self.0, narr, nrec) };
    }

    pub fn top(&self) -> c_int {
        unsafe { engine_gettop(self.0) }
    }

    pub fn arg_string(&self, n: c_int) -> &CStr {
        let v = unsafe { engine_checkstring(self.0, n) };

        unsafe { CStr::from_ptr(v) }
    }

    pub fn arg_invalid_type(&self, arg: c_int, expect: impl AsRef<CStr>) -> ! {
        unsafe { lua54_typeerror(self.0, arg, expect.as_ref().as_ptr()) };
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

    pub unsafe fn pop_string(&mut self) -> Option<CString> {
        let v = unsafe { self.to_string(-1)?.to_owned() };

        // SAFETY: We already converted the borrowed CStr to CString on the above.
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

    pub unsafe fn is_table(&self, index: c_int) -> bool {
        unsafe { lua54_istable(self.0, index) }
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

    pub unsafe fn get_index(&self, table: c_int, index: i64) -> Type {
        unsafe { lua54_geti(self.0, table, index) }
    }

    pub unsafe fn get_field(&self, table: c_int, key: impl AsRef<CStr>) -> Type {
        unsafe { lua54_getfield(self.0, table, key.as_ref().as_ptr()) }
    }

    /// # Safety
    /// - `table` must be valid.
    /// - Top of Lua stack must have a value for this field.
    pub unsafe fn set_field(&mut self, table: c_int, key: impl AsRef<CStr>) {
        unsafe { engine_setfield(self.0, table, key.as_ref().as_ptr()) };
    }

    /// # Safety
    /// Lua stack must have at least one item.
    pub unsafe fn set_global(&self, name: impl AsRef<CStr>) {
        unsafe { lua54_setglobal(self.0, name.as_ref().as_ptr()) };
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
