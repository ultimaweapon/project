use crate::Type;
use std::ffi::{c_char, c_int};

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct lua_State([u8; 0]);

unsafe extern "C-unwind" {
    pub safe fn lua54_newstate() -> *mut lua_State;
    pub fn lua54_close(L: *mut lua_State);
    pub fn engine_require_os(L: *mut lua_State);
    pub fn engine_load(
        L: *mut lua_State,
        name: *const c_char,
        script: *const c_char,
        len: usize,
    ) -> bool;
    pub fn engine_pcall(L: *mut lua_State, nargs: c_int, nresults: c_int, msgh: c_int) -> bool;
    pub fn engine_checkstack(L: *mut lua_State, n: c_int);
    pub fn engine_pushnil(L: *mut lua_State);
    pub fn engine_pushstring(L: *mut lua_State, s: *const c_char) -> *const c_char;
    pub fn engine_pushcclosure(
        L: *mut lua_State,
        fp: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int,
        n: c_int,
    );
    pub fn engine_gettop(L: *mut lua_State) -> c_int;
    pub fn engine_checkstring(L: *mut lua_State, arg: c_int) -> *const c_char;
    pub fn lua54_typeerror(L: *mut lua_State, arg: c_int, tname: *const c_char) -> !;
    pub fn engine_argerror(L: *mut lua_State, arg: c_int, extramsg: *const c_char) -> !;
    pub fn engine_isnil(L: *mut lua_State, index: c_int) -> bool;
    pub fn lua54_istable(L: *mut lua_State, index: c_int) -> bool;
    pub fn engine_tointegerx(L: *mut lua_State, index: c_int, isnum: *mut c_int) -> i64;
    pub fn engine_tostring(L: *mut lua_State, index: c_int) -> *const c_char;
    pub fn engine_touserdata(L: *mut lua_State, index: c_int) -> *mut u8;
    pub fn lua54_type(L: *mut lua_State, index: c_int) -> Type;
    pub fn lua54_typename(L: *mut lua_State, tp: Type) -> *const c_char;
    pub fn engine_createtable(L: *mut lua_State, narr: c_int, nrec: c_int);
    pub fn lua54_geti(L: *mut lua_State, index: c_int, i: i64) -> Type;
    pub fn lua54_seti(L: *mut lua_State, index: c_int, n: i64);
    pub fn lua54_getfield(L: *mut lua_State, index: c_int, k: *const c_char) -> Type;
    pub fn engine_setfield(L: *mut lua_State, index: c_int, k: *const c_char);
    pub fn engine_newuserdatauv(L: *mut lua_State, size: usize, nuvalue: c_int) -> *mut u8;
    pub fn engine_setmetatable(L: *mut lua_State, index: c_int);
    pub fn engine_upvalueindex(i: c_int) -> c_int;
    pub fn lua54_setglobal(L: *mut lua_State, name: *const c_char);
    pub fn lua54_replace(L: *mut lua_State, index: c_int);
    pub fn engine_pop(L: *mut lua_State, n: c_int);
    pub fn engine_error(L: *mut lua_State, msg: *const c_char) -> !;
}
