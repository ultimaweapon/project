use crate::App;
use memchr::memchr;
use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::ChildStdout;
use tsuki::context::{Args, Context, Ret};
use tsuki::{FromStr, Value, class};

/// Implementation of `os.spawn`.
pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    // Get options.
    let arg = cx.arg(1);
    let opts = if let Some(prog) = arg.as_str(true) {
        let prog = prog
            .as_utf8()
            .ok_or_else(|| arg.error("expect UTF-8 string"))?
            .into();

        Options {
            prog,
            cwd: None,
            stdout: Stream::Inherit,
        }
    } else if let Some(t) = arg.as_table() {
        // Program.
        let prog = match t.get(1) {
            Value::Str(v) => v
                .as_utf8()
                .ok_or_else(|| arg.error("expect UTF-8 string at index 1"))?
                .to_owned()
                .into(),
            v => {
                return Err(arg.error(format!("expect string at index 1, got {}", cx.type_name(v))));
            }
        };

        // CWD.
        let cwd = match t.get_str_key("cwd") {
            Value::Nil => None,
            Value::Str(v) => v
                .as_utf8()
                .ok_or_else(|| arg.error("expect UTF-8 string on 'cwd'"))?
                .to_owned()
                .into(),
            v => return Err(arg.error(format!("expect string on 'cwd', got {}", cx.type_name(v)))),
        };

        // STDOUT.
        let stdout = match t.get_str_key("stdout") {
            Value::Nil => Stream::Inherit,
            Value::Str(v) => {
                let v = v
                    .as_utf8()
                    .ok_or_else(|| arg.error("expect UTF-8 string on 'stdout'"))?;

                v.parse()
                    .map_err(|_| arg.error(format!("unknown option '{v}' on 'stdout'")))?
            }
            v => {
                return Err(arg.error(format!(
                    "expect string on 'stdout', got {}",
                    cx.type_name(v)
                )));
            }
        };

        Options { prog, cwd, stdout }
    } else {
        return Err(arg.invalid_type("string or table"));
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
            .as_utf8()
            .ok_or_else(|| arg.error("expect UTF-8 string"))?;

        cmd.arg(val);
    }

    if let Some(v) = opts.cwd {
        cmd.current_dir(v);
    }

    // Setup streams.
    cmd.stdin(Stdio::null());
    cmd.stderr(Stdio::inherit());

    match opts.stdout {
        Stream::Null => cmd.stdout(Stdio::null()),
        Stream::Inherit => cmd.stdout(Stdio::inherit()),
        Stream::Pipe => cmd.stdout(Stdio::piped()),
    };

    // Spawn.
    let mut prog = cmd
        .spawn()
        .map_err(|e| erdp::wrap(format!("failed to spawn '{}'", opts.prog), e))?;
    let stdout = prog.stdout.take();
    let prog = cx.create_ud(Process(RefCell::new(Some(prog))));

    // Set stdout.
    if let Some(v) = stdout {
        let v = match ChildStdout::from_std(v) {
            Ok(v) => cx.create_ud(OutputStream(RefCell::new(OutputState {
                rdr: Some(Box::pin(v)),
                buf: Vec::new(),
            }))),
            Err(e) => return Err(erdp::wrap("failed to convert stdout to asynchronous", e).into()),
        };

        prog.set("stdout", v);
    }

    cx.push(prog)?;

    Ok(cx.into())
}

/// Class of the value that returned from `os.spawn`.
pub struct Process(RefCell<Option<Child>>);

#[class(associated_data = App)]
impl Process {
    #[close(hidden)]
    fn kill(&self, _: &Context<App, Args>) -> Result<(), Box<dyn std::error::Error>> {
        // Check if already killed.
        let mut prog = match self.0.borrow_mut().take() {
            Some(v) => v,
            None => return Ok(()),
        };

        // Kill.
        let id = prog.id();

        prog.kill()
            .map_err(|e| erdp::wrap(format!("failed to kill {id}"), e))?;
        prog.wait()
            .map_err(|e| erdp::wrap(format!("failed to wait {id}"), e))?;

        Ok(())
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let mut prog = match self.0.get_mut().take() {
            Some(v) => v,
            None => return,
        };

        prog.kill().unwrap();
        prog.wait().unwrap();
    }
}

/// Class of `stdout` property of the value returned from `os.spawn`.
pub struct OutputStream(RefCell<OutputState>);

#[class(associated_data = App)]
impl OutputStream {
    async fn read(&self, cx: &Context<'_, App, Args>) -> Result<(), Box<dyn std::error::Error>> {
        // Lock stream.
        let mut st = self
            .0
            .try_borrow_mut()
            .map_err(|_| "concurrent read is not supported")?;
        let st = st.deref_mut();
        let rdr = match &mut st.rdr {
            Some(v) => v,
            None => return Ok(()),
        };

        // Read.
        let buf = &mut st.buf;

        if cx.args() == 1 {
            // Fill the buffer until LF or EOF.
            let mut end = loop {
                if let Some(i) = memchr(b'\n', buf) {
                    break i;
                }

                if rdr.read_buf(buf).await? == 0 {
                    st.rdr = None;

                    if buf.is_empty() {
                        return Ok(());
                    }

                    break buf.len();
                }
            };

            cx.push_bytes(&buf[..end])?;

            // Remove pushed data.
            if end < buf.len() {
                end += 1;
            }

            buf.drain(..end);
        } else {
            return Err("non-default format currently not supported".into());
        }

        Ok(())
    }
}

struct OutputState {
    rdr: Option<Pin<Box<dyn AsyncRead>>>,
    buf: Vec<u8>,
}

/// First argument of `os.spawn`.
struct Options<'a> {
    prog: Cow<'a, str>,
    cwd: Option<String>,
    stdout: Stream,
}

/// Option of standard stream for `os.spawn`.
#[derive(FromStr)]
enum Stream {
    Null,
    Inherit,
    Pipe,
}
