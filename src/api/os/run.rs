use crate::App;
use std::process::{Command, Stdio};
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    // Get options.
    let prog = cx.arg(1);
    let prog = prog
        .get_str()?
        .as_utf8()
        .ok_or_else(|| prog.error("expect UTF-8 string"))?;

    // Get arguments.
    let mut cmd = Command::new(prog);

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

    cmd.stdin(Stdio::null());

    // Run.
    let status = cmd
        .status()
        .map_err(|e| erdp::wrap(format!("failed to run '{prog}'"), e))?;

    if !status.success() {
        return Err(format!("'{prog}' exited with an error ({status})").into());
    }

    Ok(cx.into())
}
