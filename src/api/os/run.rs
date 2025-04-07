use std::borrow::Cow;
use std::process::{Command, Stdio};
use zl::{Context, Error, NonYieldable, Value};

pub fn entry(cx: &mut Context<NonYieldable>) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(prog) = cx.try_str(1) {
        Options {
            prog: Cow::Borrowed(prog),
        }
    } else if let Some(mut v) = cx.try_table(1) {
        let key = 1;
        let prog = match v.get(key) {
            Value::String(s) => match s.to_str() {
                Ok(v) => Cow::Owned(v.into()),
                Err(e) => return Err(Error::arg_table_from_std(1, key, e)),
            },
            v => return Err(Error::arg_table_type(1, key, "string", v)),
        };

        Options { prog }
    } else {
        return Err(Error::arg_type(1, c"string or table"));
    };

    // Get arguments.
    let mut cmd = Command::new(opts.prog.as_ref());

    for i in 2..=cx.args() {
        if !cx.is_nil(i) {
            cmd.arg(cx.to_str(i));
        }
    }

    cmd.stdin(Stdio::null());

    // Run.
    let status = cmd
        .status()
        .map_err(|e| Error::with_source(format!("failed to run '{}'", opts.prog), e))?;

    if !status.success() {
        return Err(format!("'{}' exited with an error ({})", opts.prog, status).into());
    }

    Ok(())
}

struct Options<'a> {
    prog: Cow<'a, str>,
}
