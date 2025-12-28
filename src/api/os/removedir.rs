use crate::App;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let path = cx.arg(1);
    let path = path
        .to_str()?
        .as_utf8()
        .ok_or_else(|| path.error("expect UTF-8 string"))?;

    std::fs::remove_dir_all(path)?;

    Ok(cx.into())
}
