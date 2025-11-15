use crate::App;
use tsuki::{Lua, Module, Ref, Table, fp};

mod arch;
mod capture;
mod kind;
mod run;
mod spawn;

/// Implementation of [Module] for `os` API.
pub struct OsModule;

impl Module<App> for OsModule {
    const NAME: &str = "os";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        // Register classes.
        lua.register_class::<self::spawn::Process>();
        lua.register_class::<self::spawn::OutputStream>();

        // We need to manually create the table instead of using OsLib so the linker don't keep the
        // functions we don't use.
        let m = lua.create_table();

        m.set_str_key("arch", fp!(self::arch::entry));
        m.set_str_key("capture", fp!(self::capture::entry));
        m.set_str_key("kind", fp!(self::kind::entry));
        m.set_str_key("run", fp!(self::run::entry));
        m.set_str_key("spawn", fp!(self::spawn::entry));

        Ok(m)
    }
}
