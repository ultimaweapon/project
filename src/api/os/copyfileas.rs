use super::CopyMode;
use crate::App;
use tokio::fs::File;
use tsuki::context::{Args, Context, Ret};

pub async fn entry(
    cx: Context<'_, App, Args>,
) -> Result<Context<'_, App, Ret>, Box<dyn std::error::Error>> {
    // Get arguments.
    let src = cx.arg(1);
    let src = src
        .to_str()?
        .as_utf8()
        .ok_or_else(|| src.error("expect UTF-8 string"))?;
    let dst = cx.arg(2);
    let dst = dst
        .to_str()?
        .as_utf8()
        .ok_or_else(|| dst.error("expect UTF-8 string"))?;
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
                .map_err(|e| erdp::wrap(format!("failed to open {src}"), e))?;
            let mut to = File::create(dst)
                .await
                .map_err(|e| erdp::wrap(format!("failed to open {dst}"), e))?;

            tokio::io::copy(&mut from, &mut to)
                .await
                .map_err(|e| erdp::wrap(format!("failed to copy {src} to {dst}"), e))?
        }
        CopyMode::All => match tokio::fs::copy(src, dst).await {
            Ok(v) => v,
            Err(e) => return Err(erdp::wrap(format!("failed to copy {src} to {dst}"), e).into()),
        },
    };

    // File larger than i64::MAX should not be possible...
    cx.push(i64::try_from(r).unwrap())?;

    Ok(cx.into())
}
