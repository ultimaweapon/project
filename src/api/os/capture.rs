use std::borrow::Cow;
use std::process::Stdio;
use tokio::process::Command;
use zl::{Context, Error, Frame, FromOption, Value, Yieldable};

pub async fn entry(cx: &mut Context<'_, Yieldable>) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(prog) = cx.try_str(1) {
        Options {
            prog: Cow::Borrowed(prog),
            from: From::default(),
        }
    } else if let Some(mut t) = cx.try_table(1) {
        // Program.
        let key = 1;
        let prog = match t.get(key) {
            Value::String(s) => s
                .to_str()
                .map_err(|e| Error::arg_table_from_std(1, key, e))
                .map(|v| Cow::Owned(v.into()))?,
            v => return Err(Error::arg_table_type(1, key, "string", v)),
        };

        // From.
        let key = c"from";
        let from = match t.get(key) {
            Value::Nil(_) => From::default(),
            Value::String(v) => v.to_option().map_err(|e| Error::arg_table(1, key, e))?,
            v => return Err(Error::arg_table_type(1, key, "string", v)),
        };

        Options { prog, from }
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
    let p = cmd
        .spawn()
        .map_err(|e| (format!("failed to run '{}'", opts.prog), e))?;
    let mut r = p
        .wait_with_output()
        .await
        .map_err(|e| (format!("failed to wait '{}'", opts.prog), e))?;

    if !r.status.success() {
        return Err(format!("'{}' exited with an error ({})", opts.prog, r.status).into());
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
