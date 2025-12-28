use super::join_path;
use crate::App;
use std::path::PathBuf;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let mut path = PathBuf::new();

    join_path(&cx, |_, v| {
        path.push(v);
        Ok(())
    })?;

    std::fs::remove_dir_all(&path)
        .map_err(|e| erdp::wrap(format!("failed to remove {}", path.display()), e))?;

    Ok(cx.into())
}
