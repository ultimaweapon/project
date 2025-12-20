use crate::App;
use crate::manifest::{ArgName, ArgType, CommandArg};
use clap::ArgMatches;
use rustc_hash::FxHashMap;
use tsuki::context::{Context, Ret};
use tsuki::{Lua, Module, Nil, Ref, UserData, fp};

/// Implementation of [Module] for global variabla `args`.
pub struct ArgsModule {
    pub defs: FxHashMap<ArgName, CommandArg>,
    pub args: ArgMatches,
}

impl Module<App> for ArgsModule {
    const NAME: &str = "args";

    type Inst<'a>
        = Ref<'a, UserData<App, Args>>
    where
        App: 'a;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        // Setup metatable.
        let mt = lua.create_table();

        mt.set_str_key("__index", fp!(Args::get));

        lua.register_metatable::<Args>(&mt);

        Ok(lua.create_ud(Args {
            defs: self.defs,
            vals: self.args,
        }))
    }
}

/// Class of the global variable `args`.
pub struct Args {
    defs: FxHashMap<ArgName, CommandArg>,
    vals: ArgMatches,
}

impl Args {
    fn get(
        cx: Context<App, tsuki::context::Args>,
    ) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
        let args = cx.arg(1).get_ud::<Self>()?.value();
        let name = cx
            .arg(2)
            .get_str()?
            .as_utf8()
            .ok_or("expect UTF-8 string")?;
        let def = match args.defs.get(name) {
            Some(v) => v,
            None => {
                cx.push(Nil)?;
                return Ok(cx.into());
            }
        };

        match def.ty {
            ArgType::Bool => cx.push(args.vals.get_flag(name))?,
            ArgType::String => match args.vals.get_one::<String>(name) {
                Some(v) => cx.push_str(v.as_str())?,
                None => cx.push(Nil)?,
            },
        }

        Ok(cx.into())
    }
}
