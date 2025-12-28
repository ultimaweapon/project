use super::join_path;
use crate::App;
use std::io::ErrorKind;
use std::path::PathBuf;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let res = cx.create_table();
    let mut path = PathBuf::new();

    join_path(&cx, |i, c| {
        path.push(c);

        // Create a directory.
        let r = match std::fs::create_dir(&path) {
            Ok(_) => true,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => false,
            Err(e) => {
                return Err(Box::new(erdp::wrap(
                    format!("failed to create {}", path.display()),
                    e,
                )));
            }
        };

        res.set(i as i64, r)?;

        Ok(())
    })?;

    cx.push(res)?;

    Ok(cx.into())
}
