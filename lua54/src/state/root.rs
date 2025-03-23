use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_close, lua54_newstate};
use std::ffi::c_int;

/// Encapsulates a `lua_State` created by `lua_newstate`.
pub struct RootState {
    state: *mut lua_State,
}

impl RootState {
    pub fn new() -> Self {
        Self {
            state: lua54_newstate(),
        }
    }
}

impl Drop for RootState {
    fn drop(&mut self) {
        unsafe { lua54_close(self.state) };
    }
}

impl Frame for RootState {
    fn state(&self) -> *mut lua_State {
        self.state
    }

    unsafe fn release_items(&mut self, n: c_int) {
        unsafe { engine_pop(self.state, n) };
    }
}
