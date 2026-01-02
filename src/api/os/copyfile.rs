use super::CopyMode;
use crate::App;
use std::path::Path;
use tokio::fs::File;
use tsuki::context::{Args, Context, Ret};

pub async fn entry(
    cx: Context<'_, App, Args>,
) -> Result<Context<'_, App, Ret>, Box<dyn std::error::Error>> {
    // Get file path.
    let src = cx.arg(1);
    let (src, name) = match src.to_str()?.as_utf8() {
        Some(v) => {
            let p = Path::new(v);
            let n = p
                .file_name()
                .ok_or_else(|| src.error("path does not refer to a file"))?;

            (p, n)
        }
        None => return Err(src.error("expect UTF-8 string")),
    };

    // Get destination and mode.
    let dst = cx.arg(2);
    let dst = dst
        .to_str()?
        .as_utf8()
        .map(Path::new)
        .ok_or_else(|| dst.error("expect UTF-8 string"))?
        .join(name);
    let mode = cx.arg(3);
    let mode = match mode.to_nilable_str(false)? {
        Some(v) => v
            .as_utf8()
            .ok_or_else(|| mode.error("expect UTF-8 string"))?
            .parse()
            .map_err(|e| mode.error(e))?,
        None => CopyMode::default(),
    };

    // Copy.
    let r = match mode {
        CopyMode::Content => {
            let mut from = File::open(src)
                .await
                .map_err(|e| erdp::wrap(format!("failed to open {}", src.display()), e))?;
            let mut to = File::create(&dst)
                .await
                .map_err(|e| erdp::wrap(format!("failed to open {}", dst.display()), e))?;

            tokio::io::copy(&mut from, &mut to).await.map_err(|e| {
                erdp::wrap(
                    format!("failed to copy {} to {}", src.display(), dst.display()),
                    e,
                )
            })?
        }
        CopyMode::All => match tokio::fs::copy(src, &dst).await {
            Ok(v) => v,
            Err(e) => {
                return Err(Box::new(erdp::wrap(
                    format!("failed to copy {} to {}", src.display(), dst.display()),
                    e,
                )));
            }
        },
    };

    // File larger than i64::MAX should not be possible...
    cx.push(i64::try_from(r).unwrap())?;

    Ok(cx.into())
}
