use super::{Table, TableIndex};
use crate::ffi::{engine_pop, lua_State};
use crate::{Frame, Function, Nil, Type, Value};
use std::ffi::c_int;

/// Encapsulates a borrowed table in the stack.
///
/// This kind of table either come from function argument or results.
pub struct BorrowedTable<'a, P: Frame> {
    parent: &'a mut P,
    index: c_int,
}

impl<'a, P: Frame> BorrowedTable<'a, P> {
    /// # Safety
    /// `index` must be a table.
    pub(crate) unsafe fn new(parent: &'a mut P, index: c_int) -> Self {
        Self { parent, index }
    }

    pub fn get<I: TableIndex>(&mut self, index: I) -> Value<Self> {
        match unsafe { index.get(self.parent.state(), self.index) } {
            Type::None => unreachable!(),
            Type::Nil => Value::Nil(unsafe { Nil::new(self) }),
            Type::Boolean => todo!(),
            Type::LightUserData => todo!(),
            Type::Number => todo!(),
            Type::String => Value::String(unsafe { crate::String::new(self) }),
            Type::Table => Value::Table(unsafe { Table::new(self) }),
            Type::Function => Value::Function(unsafe { Function::new(self) }),
            Type::UserData => todo!(),
            Type::Thread => todo!(),
        }
    }
}

impl<'a, P: Frame> Frame for BorrowedTable<'a, P> {
    fn state(&self) -> *mut lua_State {
        self.parent.state()
    }

    unsafe fn release_items(&mut self, n: c_int) {
        unsafe { engine_pop(self.state(), n) };
    }
}
