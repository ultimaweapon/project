use crate::App;
use tsuki::FromStr;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    // Get string and mode.
    let str = cx.arg(1);
    let str = str
        .to_str()?
        .as_utf8()
        .ok_or_else(|| str.error("expect UTF-8 string"))?;
    let mode = cx.arg(2);
    let mode = match mode.to_nilable_str(false)? {
        Some(v) => v
            .as_utf8()
            .ok_or_else(|| mode.error("expect UTF-8 string"))?
            .parse()
            .map_err(|e| mode.error(e))?,
        None => Mode::default(),
    };

    // Capitalize.
    let mut r = String::with_capacity(str.len());

    match mode {
        Mode::First => 'b: {
            let mut iter = str.chars();
            let first = match iter.next() {
                Some(v) => v,
                None => break 'b,
            };

            for c in first.to_uppercase() {
                r.push(c);
            }

            r.push_str(iter.as_str());
        }
    }

    cx.push_str(r)?;

    Ok(cx.into())
}

#[derive(Default, FromStr)]
enum Mode {
    #[default]
    First,
}
