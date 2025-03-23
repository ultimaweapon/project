use crate::Frame;
use crate::ffi::engine_tostring;
use std::ffi::CStr;

/// Encapsulates Lua string in a frame.
pub struct String<'a, P: Frame>(&'a mut P);

impl<'a, P: Frame> String<'a, P> {
    /// # Safety
    /// Top of the stack must be a string.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }

    pub fn get(&self) -> &CStr {
        unsafe { CStr::from_ptr(engine_tostring(self.0.state(), -1)) }
    }
}

impl<'a, P: Frame> Drop for String<'a, P> {
    fn drop(&mut self) {
        unsafe { self.0.release_items(1) };
    }
}
