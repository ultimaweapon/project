use std::borrow::Cow;
use std::ffi::c_int;
use std::process::Command;

use lua54::{Engine, LuaError, Type};

pub fn entry(en: &mut Engine) -> c_int {
    // Get options.
    let opts = if let Some(v) = unsafe { en.to_string(1) } {
        let prog = match v.to_str() {
            Ok(v) => v.into(),
            Err(_) => en.arg_error(1, c"not UTF-8 string"),
        };

        Options { prog }
    } else if unsafe { en.is_table(1) } {
        let prog = match unsafe { en.get_index(1, 1) } {
            Type::String => match unsafe { en.pop_string().unwrap().into_string() } {
                Ok(v) => v.into(),
                Err(_) => en.arg_error(1, c"not UTF-8 string at table index #1"),
            },
            _ => en.arg_error(1, c"expect string at table index #1"),
        };

        Options { prog }
    } else {
        en.arg_invalid_type(1, c"string or table")
    };

    // Get arguments.
    let mut cmd = Command::new(opts.prog.as_ref());

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
        Err(e) => en.error(LuaError::with_source(
            format!("failed to run '{}'", opts.prog),
            &e,
        )),
    };

    if !status.success() {
        let m = format!("'{}' exited with an error ({})", opts.prog, status);
        en.error(LuaError::new(m));
    }

    0
}

struct Options<'a> {
    prog: Cow<'a, str>,
}
