use std::borrow::Cow;
use std::process::{Command, Stdio};
use zl::{Context, Error, NonYieldable, PositiveInt, Value};

pub fn entry(cx: &mut Context<NonYieldable>) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(prog) = cx.try_str(PositiveInt::ONE) {
        Options {
            prog: Cow::Borrowed(prog),
        }
    } else if let Some(mut v) = cx.try_table(PositiveInt::ONE) {
        let key = 1;
        let prog = match v.get(key) {
            Value::String(mut s) => match s.to_str() {
                Ok(v) => Cow::Owned(v.into()),
                Err(e) => return Err(Error::arg_table_from_std(PositiveInt::ONE, key, e)),
            },
            v => return Err(Error::arg_table_type(PositiveInt::ONE, key, "string", v)),
        };

        Options { prog }
    } else {
        return Err(Error::arg_type(PositiveInt::ONE, c"string or table"));
    };

    // Get arguments.
    let mut cmd = Command::new(opts.prog.as_ref());

    for i in 2..=cx.args() {
        if !cx.is_nil(i) {
            cmd.arg(cx.to_str(PositiveInt::new(i).unwrap()));
        }
    }

    cmd.stdin(Stdio::null());

    // Run.
    let status = cmd
        .status()
        .map_err(|e| Error::with_source(format!("failed to run '{}'", opts.prog), e))?;

    if !status.success() {
        return Err(Error::other(format!(
            "'{}' exited with an error ({})",
            opts.prog, status
        )));
    }

    Ok(())
}

struct Options<'a> {
    prog: Cow<'a, str>,
}
