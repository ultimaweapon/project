use crate::ffi::{engine_isnil, engine_tointegerx, lua54_type};
use crate::{Frame, Type};
use std::ffi::c_int;
use std::num::NonZero;

/// Encapsulates results of a function call.
pub unsafe trait FuncRet<'a, P: Frame> {
    const N: c_int;

    /// # Safety
    /// The owner of [`FuncRet::N`] items at the top of stack will be transferred to the returned
    /// [`FuncRet`].
    unsafe fn new(p: &'a mut P) -> Self;
}

/// Implementation of [`FuncRet`] with a fixed number of results.
pub struct FixedRet<'a, const N: u16, P: Frame>(&'a mut P);

impl<'a, const N: u16, P: Frame> FixedRet<'a, N, P> {
    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is geater than `N`.
    pub fn to_nil(&self, n: NonZero<u16>) -> Option<()> {
        unsafe { engine_isnil(self.0.state(), Self::index(n)).then_some(()) }
    }

    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is geater than `N`.
    pub fn to_int(&self, n: NonZero<u16>) -> Option<i64> {
        let mut ok = 0;
        let val = unsafe { engine_tointegerx(self.0.state(), Self::index(n), &mut ok) };

        if ok == 0 { None } else { Some(val) }
    }

    /// `n` is one-based the same as function arguments.
    ///
    /// # Panics
    /// If `n` is geater than `N`.
    pub fn to_type(&self, n: NonZero<u16>) -> Type {
        unsafe { lua54_type(self.0.state(), Self::index(n)) }
    }

    fn index(n: NonZero<u16>) -> c_int {
        -(c_int::from(N.checked_sub(n.get()).unwrap()) + 1)
    }
}

impl<'a, const N: u16, P: Frame> Drop for FixedRet<'a, N, P> {
    fn drop(&mut self) {
        if N > 0 {
            // SAFETY: This is safe because the requirement of FuncRet::new().
            unsafe { self.0.release_items(N.into()) };
        }
    }
}

unsafe impl<'a, const N: u16, P: Frame> FuncRet<'a, P> for FixedRet<'a, N, P> {
    const N: c_int = N as c_int;

    unsafe fn new(p: &'a mut P) -> Self {
        Self(p)
    }
}
