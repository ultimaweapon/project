use crate::ffi::{
    engine_argerror, engine_checkstack, engine_createtable, engine_error, engine_gettop,
    engine_load, engine_newuserdatauv, engine_pushcclosure, engine_pushnil, engine_pushstring,
    engine_require_os, engine_setfield, engine_setmetatable, engine_touserdata,
    engine_upvalueindex, lua_State, lua54_typeerror,
};
use crate::{Error, ErrorKind, FuncState, Function, GlobalSetter, Nil, Table};
use std::ffi::{CStr, c_int};
use std::panic::UnwindSafe;
use std::path::Path;

/// Frame in a Lua stack.
///
/// Most methods in this trait can raise a C++ exception. When calling outside Lua runtime it will
/// cause the process to terminate the same as Rust panic. Inside Lua runtime it will report as Lua
/// error. Usually you don't need to worry about this **unless** you use any type that does not
/// implement a proper RAII.
pub trait Frame: Sized {
    fn require_os(&mut self) -> Table<Self> {
        // SAFETY: 3 is maximum stack size used by engine_require_os.
        unsafe { engine_checkstack(self.state(), 3) };
        unsafe { engine_require_os(self.state()) };

        unsafe { Table::new(self) }
    }

    fn set_global<N: AsRef<CStr>>(&mut self, name: N) -> GlobalSetter<Self, N> {
        GlobalSetter::new(self, name)
    }

    /// This method will load the whole content of `file` into memory before passing to Lua.
    fn load_file(
        &mut self,
        file: impl AsRef<Path>,
    ) -> Result<Result<Function<Self>, crate::String<Self>>, std::io::Error> {
        // SAFETY: engine_load return either error object or a chunk.
        unsafe { engine_checkstack(self.state(), 1) };

        // Read file.
        let file = file.as_ref();
        let data = std::fs::read(file)?;

        // Get chunk name.
        let file = file.to_string_lossy();
        let mut name = String::with_capacity(1 + file.len() + 1);

        name.push('@');
        name.push_str(&file);
        name.push('\0');

        // Load.
        let name = name.as_ptr().cast();
        let r = match unsafe { engine_load(self.state(), name, data.as_ptr().cast(), data.len()) } {
            true => Ok(unsafe { Function::new(self) }),
            false => Err(unsafe { crate::String::new(self) }),
        };

        Ok(r)
    }

    fn push_nil(&mut self) -> Nil<Self> {
        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { engine_pushnil(self.state()) };

        unsafe { Nil::new(self) }
    }

    fn push_string(&mut self, s: impl AsRef<CStr>) -> crate::String<Self> {
        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { engine_pushstring(self.state(), s.as_ref().as_ptr()) };

        unsafe { crate::String::new(self) }
    }

    /// See [`FuncState`] for how to return the values to Lua.
    fn push_fn<F>(&mut self, f: F) -> Function<Self>
    where
        F: FnMut(&mut FuncState) -> Result<(), Error> + UnwindSafe + 'static,
    {
        // SAFETY: 3 is maximum items we pushed here.
        unsafe { engine_checkstack(self.state(), 3) };

        if align_of::<F>() <= align_of::<*mut ()>() {
            // Move Rust function to Lua user data.
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<F>(), 0) };

            unsafe { ptr.cast::<F>().write(f) };

            // Set finalizer.
            if std::mem::needs_drop::<F>() {
                unsafe { engine_createtable(self.state(), 0, 1) };
                unsafe { engine_pushcclosure(self.state(), finalizer::<F>, 0) };
                unsafe { engine_setfield(self.state(), -2, c"__gc".as_ptr()) };
                unsafe { engine_setmetatable(self.state(), -1) };
            }

            // Push invoker.
            unsafe { engine_pushcclosure(self.state(), invoker::<F>, 1) };
        } else {
            // Move Rust function to Lua user data.
            let ptr = unsafe { engine_newuserdatauv(self.state(), size_of::<Box<F>>(), 0) };

            unsafe { ptr.cast::<Box<F>>().write(f.into()) };

            // Set finalizer.
            unsafe { engine_createtable(self.state(), 0, 1) };
            unsafe { engine_pushcclosure(self.state(), finalizer::<Box<F>>, 0) };
            unsafe { engine_setfield(self.state(), -2, c"__gc".as_ptr()) };
            unsafe { engine_setmetatable(self.state(), -1) };

            // Push invoker.
            unsafe { engine_pushcclosure(self.state(), invoker::<Box<F>>, 1) };
        }

        unsafe { Function::new(self) }
    }

    fn push_table(&mut self, narr: u16, nrec: u16) -> Table<Self> {
        unsafe { engine_checkstack(self.state(), 1) };
        unsafe { engine_createtable(self.state(), narr.into(), nrec.into()) };

        unsafe { Table::new(self) }
    }

    /// Returns a `lua_State` this frame belong to.
    ///
    /// This is a low-level method. Using the returned `lua_State` incorrectly will violate safety
    /// guarantee of this crate. This does not mark as `unsafe` because invoke this method is safe
    /// but using the returned pointer required unsafe code.
    fn state(&self) -> *mut lua_State;

    /// # Safety
    /// `n` must be greater than zero and `n` items on the stack must be owned by the caller.
    unsafe fn release_items(&mut self, n: c_int);
}

unsafe extern "C-unwind" fn invoker<F>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int
where
    F: FnMut(&mut FuncState) -> Result<(), Error> + UnwindSafe + 'static,
{
    // Setup FuncState.
    let args = unsafe { engine_gettop(L) };
    let mut lua = unsafe { FuncState::new(L, args) };

    // Invoke.
    let cb = unsafe { engine_upvalueindex(1) };
    let cb = unsafe { engine_touserdata(L, cb).cast::<F>() };
    let e = match unsafe { (*cb)(&mut lua) } {
        Ok(_) => return lua.into_results(),
        Err(e) => ErrorKind::from(e),
    };

    // Raise error.
    let (n, e) = match e {
        // SAFETY: n only used to format the message.
        ErrorKind::Arg(n, e) => unsafe { engine_argerror(L, n, e.as_ptr().cast()) },
        ErrorKind::ArgType(n, e) => (n, e),
        ErrorKind::Other(e) => unsafe { engine_error(L, e.as_ptr().cast()) },
    };

    if n <= args {
        // SAFETY: n is positive.
        unsafe { lua54_typeerror(L, n, e.as_ptr().cast()) };
    } else {
        // lua54_typeerror require index to be valid so we need to emulate its behavior in this
        // case.
        lua.arg_out_of_bound(n, &e[..(e.len() - 1)]);
    }
}

unsafe extern "C-unwind" fn finalizer<F>(#[allow(non_snake_case)] L: *mut lua_State) -> c_int {
    let ptr = unsafe { engine_touserdata(L, 1).cast::<F>() };
    unsafe { std::ptr::drop_in_place(ptr) };
    0
}
