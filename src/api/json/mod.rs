use crate::App;
use tsuki::{Lua, Module, Ref, Table, fp};

mod parse;

/// Implementation of [Module] for `json` API.
pub struct JsonModule;

impl Module<App> for JsonModule {
    const NAME: &str = "json";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.create_table();

        m.set_str_key("parse", fp!(self::parse::entry));

        Ok(m)
    }
}
