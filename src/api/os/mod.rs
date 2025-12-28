use crate::App;
use tsuki::{Lua, Module, Ref, Table, fp};

mod capture;
mod createdir;
mod removedir;
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

        // Set arch.
        let arch = lua.create_str(if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            todo!()
        });

        m.set_str_key("arch", arch);

        // Set kind.
        let kind = lua.create_str(if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            todo!()
        });

        m.set_str_key("kind", kind);

        // Set functions.
        m.set_str_key("capture", fp!(self::capture::entry));
        m.set_str_key("createdir", fp!(self::createdir::entry));
        m.set_str_key("removedir", fp!(self::removedir::entry));
        m.set_str_key("run", fp!(self::run::entry));
        m.set_str_key("spawn", fp!(self::spawn::entry));

        Ok(m)
    }
}
