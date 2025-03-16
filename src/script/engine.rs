/// Encapsulates a `lua_State`.
pub struct Engine(*mut lua_State);

impl Engine {
    pub fn new() -> Self {
        Self(unsafe { engine_new() })
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { engine_free(self.0) };
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    fn engine_new() -> *mut lua_State;
    fn engine_free(L: *mut lua_State);
}
