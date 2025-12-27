use crate::App;
use serde_json::Deserializer;
use tsuki::context::{Args, Context, Ret};

pub fn entry(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let data = cx.arg(1).to_str()?;
    let mut deserializer = Deserializer::from_slice(data.as_bytes());
    let value = cx.deserialize_value(&mut deserializer)?;

    deserializer.end()?;

    cx.push(value)?;

    Ok(cx.into())
}
