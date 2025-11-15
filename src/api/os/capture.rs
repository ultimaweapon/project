use crate::App;
use std::borrow::Cow;
use std::process::{Command, Stdio};
use tsuki::context::{Args, Context, Ret};
use tsuki::{FromStr, Value};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    // Get options.
    let prog = cx.arg(1);
    let opts = if let Some(s) = prog.as_str(true) {
        Options {
            prog: s
                .as_str()
                .ok_or_else(|| prog.error("expect UTF-8 string"))?
                .into(),
            from: From::default(),
        }
    } else if let Some(t) = prog.as_table() {
        // From.
        let from = match t.get_str_key("from") {
            Value::Nil => From::default(),
            Value::Str(s) => s
                .as_str()
                .ok_or_else(|| prog.error("expect UTF-8 string on 'from'"))?
                .parse()
                .map_err(|e| prog.error(e))?,
            v => {
                let ty = cx.type_name(v);

                return Err(prog.error(format!("expect string on 'from', got {ty}")));
            }
        };

        // Program.
        let prog = match t.get(1) {
            Value::Str(s) => s
                .as_str()
                .ok_or_else(|| prog.error("expect UTF-8 string at index 1"))?
                .to_owned()
                .into(),
            v => {
                let ty = cx.type_name(v);

                return Err(prog.error(format!("expect string at index 1, got {ty}")));
            }
        };

        Options { prog, from }
    } else {
        return Err(prog.invalid_type("string or table"));
    };

    // Get arguments.
    let mut cmd = Command::new(opts.prog.as_ref());

    for i in 2..=cx.args() {
        // Get argument.
        let arg = cx.arg(i);
        let val = match arg.to_nilable_str(true)? {
            Some(v) => v,
            None => continue,
        };

        // Check if UTF-8.
        let val = val
            .as_str()
            .ok_or_else(|| arg.error("expect UTF-8 string"))?;

        cmd.arg(val);
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
        .map_err(|e| erdp::wrap(format!("failed to run '{}'", opts.prog), e))?;

    if !r.status.success() {
        return Err(format!("'{}' exited with an error ({})", opts.prog, r.status).into());
    }

    // Set result.
    match opts.from {
        From::Stdout => {
            trim(&mut r.stdout);

            cx.push_bytes(r.stdout)?;
        }
        From::Stderr => {
            trim(&mut r.stderr);

            cx.push_bytes(r.stderr)?;
        }
        From::Both => {
            trim(&mut r.stdout);
            trim(&mut r.stderr);

            // Create result table.
            let t = cx.create_table();
            let o = cx.create_bytes(r.stdout);
            let e = cx.create_bytes(r.stderr);

            t.set_str_key("stdout", o);
            t.set_str_key("stderr", e);

            cx.push(t)?;
        }
    }

    Ok(cx.into())
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

#[derive(Default, Clone, Copy, FromStr)]
enum From {
    #[default]
    Stdout,
    Stderr,
    Both,
}
