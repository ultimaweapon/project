use std::borrow::Cow;
use std::process::{Command, Stdio};
use zl::{Context, Error, Frame, FromOption, PositiveInt, Value};

pub fn entry(cx: &mut Context) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(prog) = cx.try_str(PositiveInt::ONE) {
        Options {
            prog: Cow::Borrowed(prog),
            from: From::default(),
        }
    } else if let Some(mut t) = cx.try_table(PositiveInt::ONE) {
        // Program.
        let key = 1;
        let prog = match t.get(key) {
            Value::String(mut s) => s
                .to_str()
                .map_err(|e| Error::arg_table_from_std(PositiveInt::ONE, key, e))
                .map(|v| Cow::Owned(v.into()))?,
            v => return Err(Error::arg_table_type(PositiveInt::ONE, key, "string", v)),
        };

        // From.
        let key = c"from";
        let from = match t.get(key) {
            Value::Nil(_) => From::default(),
            Value::String(mut v) => v
                .to_option()
                .map_err(|e| Error::arg_table(PositiveInt::ONE, key, e))?,
            v => return Err(Error::arg_table_type(PositiveInt::ONE, key, "string", v)),
        };

        Options { prog, from }
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

    // Setup streams.
    cmd.stdin(Stdio::null());

    match opts.from {
        From::Stdout => {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::inherit());
        }
        From::Stderr => {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::piped());
        }
        From::Both => (), // Default to both for Command::output().
    }

    // Run.
    let mut r = cmd
        .output()
        .map_err(|e| Error::with_source(format!("failed to run '{}'", opts.prog), e))?;

    if !r.status.success() {
        return Err(Error::other(format!(
            "'{}' exited with an error ({})",
            opts.prog, r.status
        )));
    }

    // Set result.
    match opts.from {
        From::Stdout => {
            trim(&mut r.stdout);
            cx.push_str(r.stdout);
        }
        From::Stderr => {
            trim(&mut r.stderr);
            cx.push_str(r.stderr);
        }
        From::Both => {
            let mut t = cx.push_table(0, 2);

            trim(&mut r.stdout);
            trim(&mut r.stderr);

            t.set(c"stdout").push_str(r.stdout);
            t.set(c"stderr").push_str(r.stderr);
        }
    }

    Ok(())
}

fn trim(v: &mut Vec<u8>) {
    if v.last().is_some_and(|&b| b == b'\n') {
        v.pop();
    }

    if v.last().is_some_and(|&b| b == b'\r') {
        v.pop();
    }
}

struct Options<'a> {
    prog: Cow<'a, str>,
    from: From,
}

#[derive(Default, Clone, Copy, FromOption)]
enum From {
    #[default]
    Stdout,
    Stderr,
    Both,
}
