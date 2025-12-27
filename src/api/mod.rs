pub use self::args::ArgsModule;
pub use self::json::JsonModule;
pub use self::os::OsModule;
pub use self::url::UrlModule;

use tsuki::{Lua, Module, Table, fp};

mod args;
mod json;
mod os;
mod url;

/// Implementation of [Module] for global APIs.
pub struct GlobalModule;

impl<A> Module<A> for GlobalModule {
    const NAME: &str = "_G";

    type Inst<'a>
        = &'a Table<A>
    where
        A: 'a;

    fn open(self, lua: &Lua<A>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.global();

        m.set_str_key("assert", fp!(tsuki::builtin::base::assert));
        m.set_str_key("error", fp!(tsuki::builtin::base::error));
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
