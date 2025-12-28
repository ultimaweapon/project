use crate::App;
use serde_json::Deserializer;
use tsuki::context::{Args, Context, Ret};
use tsuki::{Lua, Module, Ref, Table, fp};

/// Implementation of [Module] for `json` API.
pub struct JsonModule;

impl Module<App> for JsonModule {
    const NAME: &str = "json";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.create_table();

        m.set_str_key("parse", fp!(parse));

        Ok(m)
    }
}

fn parse(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let data = cx.arg(1).to_str()?;
    let mut deserializer = Deserializer::from_slice(data.as_bytes());
    let value = cx.deserialize_value(&mut deserializer)?;

    deserializer.end()?;

    cx.push(value)?;

    Ok(cx.into())
}
