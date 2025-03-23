use super::{Table, TableIndex};
use crate::Frame;
use crate::ffi::{engine_pop, lua_State, lua54_replace};
use std::ffi::c_int;

/// Provides [`Frame`] implementation to set a table value.
///
/// Only the first value is used if you push a multiple values (the rest will be discarded) and only
/// the value from the recent push will be used.
pub struct TableSetter<'a, 'b, P, I>
where
    P: Frame,
    I: TableIndex,
{
    table: &'a mut Table<'b, P>,
    index: I,
    has_value: bool,
}

impl<'a, 'b, P, I> TableSetter<'a, 'b, P, I>
where
    P: Frame,
    I: TableIndex,
{
    /// # Safety
    /// Top of the stack must be a table.
    pub(super) unsafe fn new(table: &'a mut Table<'b, P>, index: I) -> Self {
        Self {
            table,
            index,
            has_value: false,
        }
    }
}

impl<'a, 'b, P, I> Drop for TableSetter<'a, 'b, P, I>
where
    P: Frame,
    I: TableIndex,
{
    fn drop(&mut self) {
        if self.has_value {
            unsafe { self.index.set(self.table.0.state(), -2) };
        }
    }
}

impl<'a, 'b, P, I> Frame for TableSetter<'a, 'b, P, I>
where
    P: Frame,
    I: TableIndex,
{
    fn state(&self) -> *mut lua_State {
        self.table.0.state()
    }

    unsafe fn release_items(&mut self, n: c_int) {
        // All checks here should be optimized away in most cases since this method and new() should
        // be automatically inlined.
        let excess = n - 1;

        if excess > 0 {
            unsafe { engine_pop(self.state(), excess) };
        }

        if self.has_value {
            unsafe { lua54_replace(self.state(), -2) };
        }

        self.has_value = true;
    }
}
