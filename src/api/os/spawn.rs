use std::borrow::Cow;
use std::cell::RefCell;
use std::panic::AssertUnwindSafe;
use std::process::{Child, Command, Stdio};
use tokio::io::AsyncRead;
use tokio::process::ChildStdout;
use zl::{Context, Error, Frame, FromOption, PositiveInt, Value, class};

/// Implementation of `os.spawn`.
pub fn entry(cx: &mut Context) -> Result<(), Error> {
    // Get options.
    let opts = if let Some(prog) = cx.try_str(PositiveInt::ONE) {
        Options {
            prog: Cow::Borrowed(prog),
            cwd: None,
            stdout: Stream::Inherit,
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

        // CWD.
        let key = c"cwd";
        let cwd = match t.get(key) {
            Value::Nil(_) => None,
            Value::String(mut v) => v
                .to_str()
                .map_err(|e| Error::arg_table_from_std(PositiveInt::ONE, key, e))?
                .to_owned()
                .into(),
            v => return Err(Error::arg_table_type(PositiveInt::ONE, key, "string", v)),
        };

        // STDOUT.
        let key = c"stdout";
        let stdout = match t.get(key) {
            Value::Nil(_) => Stream::Inherit,
            Value::String(mut v) => v
                .to_option()
                .map_err(|e| Error::arg_table(PositiveInt::ONE, key, e))?,
            v => return Err(Error::arg_table_type(PositiveInt::ONE, key, "string", v)),
        };

        Options { prog, cwd, stdout }
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
        .map_err(|e| Error::with_source(format!("failed to spawn '{}'", opts.prog), e))?;
    let stdout = prog.stdout.take();
    let mut prog = cx.push_ud(Process(AssertUnwindSafe(RefCell::new(Some(prog)))));

    // Set stdout.
    if let Some(v) = stdout {
        let v = match ChildStdout::from_std(v) {
            Ok(v) => OutputStream(AssertUnwindSafe(RefCell::new(Box::new(v)))),
            Err(e) => {
                return Err(Error::with_source(
                    "failed to convert stdout to asynchronous",
                    e,
                ));
            }
        };

        Process::set_stdout(&mut prog, v);
    }

    Ok(())
}

/// Class of the value that returned from `os.spawn`.
pub struct Process(AssertUnwindSafe<RefCell<Option<Child>>>);

#[class]
impl Process {
    const STDOUT: OutputStream = _;

    #[close]
    fn kill(cx: &mut Context) -> Result<(), Error> {
        // Check if already killed.
        let prog = cx.to_ud::<Self>(PositiveInt::ONE).into_ud();
        let mut prog = match prog.0.borrow_mut().take() {
            Some(v) => v,
            None => return Ok(()),
        };

        // Kill.
        let id = prog.id();

        prog.kill()
            .map_err(|e| Error::with_source(format!("failed to kill {id}"), e))?;
        prog.wait()
            .map_err(|e| Error::with_source(format!("failed to wait {id}"), e))?;

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
pub struct OutputStream(AssertUnwindSafe<RefCell<Box<dyn AsyncRead + Unpin>>>);

#[class]
impl OutputStream {
    fn read(_: &mut Context) -> Result<(), Error> {
        todo!()
    }
}

/// First argument of `os.spawn`.
struct Options<'a> {
    prog: Cow<'a, str>,
    cwd: Option<String>,
    stdout: Stream,
}

/// Option of standard stream for `os.spawn`.
#[derive(FromOption)]
enum Stream {
    Null,
    Inherit,
    Pipe,
}
