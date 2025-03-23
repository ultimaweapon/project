pub use self::ret::*;

use crate::Frame;
use crate::ffi::{engine_checkstack, engine_pcall};

mod ret;

/// Encapsulates a callable object in a frame.
pub struct Function<'a, P: Frame>(Option<&'a mut P>);

impl<'a, P: Frame> Function<'a, P> {
    /// # Safety
    /// Top of the stack must be a callable object.
    pub(crate) unsafe fn new(p: &'a mut P) -> Self {
        Self(Some(p))
    }

    pub fn call<R: FuncRet<'a, P>>(mut self) -> Result<R, crate::String<'a, P>> {
        // Ensure stack for results. We can't take out the parent here since engine_checkstack can
        // throw a C++ exception.
        if R::N > 0 {
            unsafe { engine_checkstack(self.0.as_ref().unwrap().state(), R::N) };
        }

        // Call.
        let p = self.0.take().unwrap();

        match unsafe { engine_pcall(p.state(), 0, R::N, 0) } {
            true => Ok(unsafe { R::new(p) }),
            false => Err(unsafe { crate::String::new(p) }),
        }
    }
}

impl<'a, P: Frame> Drop for Function<'a, P> {
    fn drop(&mut self) {
        if let Some(p) = self.0.take() {
            unsafe { p.release_items(1) };
        }
    }
}
