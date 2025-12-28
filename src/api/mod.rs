pub use self::args::ArgsModule;
pub use self::json::JsonModule;
pub use self::os::OsModule;
pub use self::path::PathModule;
pub use self::url::UrlModule;

use crate::App;
use std::fmt::{Display, Formatter};
use tsuki::context::{Args, Context, Ret};
use tsuki::{Lua, Module, Table, fp};

mod args;
mod json;
mod os;
mod path;
mod url;

/// Implementation of [Module] for global APIs.
pub struct GlobalModule;

impl GlobalModule {
    fn exit(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
        let code = cx.arg(1);
        let code = code
            .to_int()?
            .try_into()
            .ok()
            .filter(|c: &u8| matches!(c, 0..=99))
            .ok_or_else(|| code.error("value out of range"))?;

        Err(Box::new(Exit(code)))
    }
}

impl Module<App> for GlobalModule {
    const NAME: &str = "_G";

    type Inst<'a> = &'a Table<App>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.global();

        m.set_str_key("assert", fp!(tsuki::builtin::base::assert));
        m.set_str_key("error", fp!(tsuki::builtin::base::error));
        m.set_str_key("exit", fp!(Self::exit));
        m.set_str_key("getmetatable", fp!(tsuki::builtin::base::getmetatable));
        m.set_str_key("load", fp!(tsuki::builtin::base::load));
        m.set_str_key("next", fp!(tsuki::builtin::base::next));
        m.set_str_key("pairs", fp!(tsuki::builtin::base::pairs));
        m.set_str_key("print", fp!(tsuki::builtin::base::print));
        m.set_str_key("rawequal", fp!(tsuki::builtin::base::rawequal));
        m.set_str_key("rawget", fp!(tsuki::builtin::base::rawget));
        m.set_str_key("rawlen", fp!(tsuki::builtin::base::rawlen));
        m.set_str_key("rawset", fp!(tsuki::builtin::base::rawset));
        m.set_str_key("select", fp!(tsuki::builtin::base::select));
        m.set_str_key("setmetatable", fp!(tsuki::builtin::base::setmetatable));
        m.set_str_key("tonumber", fp!(tsuki::builtin::base::tonumber));
        m.set_str_key("tostring", fp!(tsuki::builtin::base::tostring));
        m.set_str_key("type", fp!(tsuki::builtin::base::r#type));

        Ok(m)
    }
}

/// Encapsulates exit code to exit our process.
#[derive(Debug)]
pub struct Exit(u8);

impl Exit {
    pub fn code(&self) -> u8 {
        self.0
    }
}

impl std::error::Error for Exit {}

impl Display for Exit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

fn join_path(
    cx: &Context<App, Args>,
    mut f: impl FnMut(usize, &str) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = cx.arg(1);
    let path = path
        .to_str()?
        .as_utf8()
        .ok_or_else(|| path.error("expect UTF-8 string"))?;

    f(1, path)?;

    for i in 2..=cx.args() {
        let path = cx.arg(i);
        let path = path
            .to_str()?
            .as_utf8()
            .ok_or_else(|| path.error("expect UTF-8 string"))?;

        f(i, path)?;
    }

    Ok(())
}
