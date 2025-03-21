use std::ffi::c_int;

use crate::script::Engine;

pub fn entry(en: &mut Engine) -> c_int {
    let v = if cfg!(target_os = "windows") {
        c"windows"
    } else if cfg!(target_os = "macos") {
        c"macos"
    } else if cfg!(target_os = "linux") {
        c"linux"
    } else {
        todo!()
    };

    en.push_string(v);
    1
}
