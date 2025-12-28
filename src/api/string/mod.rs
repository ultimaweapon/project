use crate::App;
use tsuki::builtin::StrLib;
use tsuki::{Lua, Module, Ref, Table, fp};

mod capitalize;

/// Implementation of [Module] for `string` API.
pub struct StringModule;

impl Module<App> for StringModule {
    const NAME: &str = "string";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = StrLib.open(lua)?;

        m.set_str_key("capitalize", fp!(self::capitalize::entry));

        Ok(m)
    }
}
