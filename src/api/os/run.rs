use std::ffi::c_int;
use std::process::Command;

use crate::script::{Engine, LuaError};

pub fn entry(en: &mut Engine) -> c_int {
    // Get program.
    let prog = match en.arg_string(1).to_str() {
        Ok(v) => v,
        Err(_) => en.arg_error(1, c"not UTF-8 string"),
    };

    // Get arguments.
    let mut cmd = Command::new(prog);

    for i in 2..=en.top() {
        // Skip nil.
        if unsafe { en.is_nil(i) } {
            continue;
        }

        // Push arguments.
        match en.arg_string(i).to_str() {
            Ok(v) => cmd.arg(v),
            Err(_) => en.arg_error(i, c"not UTF-8 string"),
        };
    }

    // Run.
    let status = match cmd.status() {
        Ok(v) => v,
        Err(e) => en.error(LuaError::with_source(format!("failed to run '{prog}'"), &e)),
    };

    if !status.success() {
        let m = format!("'{prog}' exited with an error ({status})");
        en.error(LuaError::new(m));
    }

    0
}
