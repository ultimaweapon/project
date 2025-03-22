use std::ffi::c_int;

use lua54::Engine;

pub fn entry(en: &mut Engine) -> c_int {
    let v = if cfg!(target_arch = "x86_64") {
        c"x86_64"
    } else if cfg!(target_arch = "aarch64") {
        c"aarch64"
    } else {
        todo!()
    };

    en.push_string(v);
    1
}
