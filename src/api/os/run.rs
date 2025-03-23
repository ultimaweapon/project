use lua54::{Error, FuncState, Value};
use std::borrow::Cow;
use std::process::Command;

pub fn entry(lua: &mut FuncState) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(v) = lua.try_string(1) {
        let prog = v.to_str().map_err(|e| Error::arg_from_std(1, e))?;

        Options {
            prog: Cow::Borrowed(prog),
        }
    } else if let Some(mut v) = lua.try_table(1) {
        let prog = match v.get(1) {
            Value::String(s) => match s.get().to_str() {
                Ok(v) => Cow::Owned(v.into()),
                Err(_) => return Err(Error::arg(1, c"not UTF-8 string at table index #1")),
            },
            _ => return Err(Error::arg(1, c"expect string at table index #1")),
        };

        Options { prog }
    } else {
        return Err(Error::ty(1, c"string or table"));
    };

    // Get arguments.
    let mut cmd = Command::new(opts.prog.as_ref());

    for i in 2..=lua.args() {
        // Skip nil.
        if lua.is_nil(i) {
            continue;
        }

        // Push arguments.
        match lua.get_string(i).to_str() {
            Ok(v) => cmd.arg(v),
            Err(e) => return Err(Error::arg_from_std(i, e)),
        };
    }

    // Run.
    let status = match cmd.status() {
        Ok(v) => v,
        Err(e) => return Err((format!("failed to run '{}'", opts.prog), e).into()),
    };

    if !status.success() {
        return Err(format!("'{}' exited with an error ({})", opts.prog, status).into());
    }

    Ok(())
}

struct Options<'a> {
    prog: Cow<'a, str>,
}
