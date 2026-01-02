use crate::App;
use crate::api::join_path;
use std::io::ErrorKind;
use std::path::PathBuf;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let mut path = PathBuf::new();

    join_path(&cx, |_, v| {
        path.push(v);
        Ok(())
    })?;

    if let Err(e) = std::fs::remove_dir_all(&path)
        && e.kind() != ErrorKind::NotFound
    {
        return Err(erdp::wrap(format!("failed to remove {}", path.display()), e).into());
    }

    Ok(cx.into())
}
