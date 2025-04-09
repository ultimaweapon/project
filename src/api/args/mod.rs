use crate::manifest::{ArgType, CommandArg};
use clap::ArgMatches;
use rustc_hash::FxHashMap;
use std::panic::AssertUnwindSafe;
use zl::{Context, Error, Frame, Lua, class};

pub fn register(lua: &mut Lua, defs: FxHashMap<String, CommandArg>, args: ArgMatches) {
    assert!(lua.register_ud::<Args>());

    lua.set_global(c"args").push_ud(Args {
        defs,
        vals: AssertUnwindSafe(args),
    });
}

/// Class of the global variable `args`.
struct Args {
    defs: FxHashMap<String, CommandArg>,
    vals: AssertUnwindSafe<ArgMatches>,
}

#[class]
impl Args {
    fn get(&self, cx: &mut Context) -> Result<(), Error> {
        let name = cx.to_str(2);
        let def = match self.defs.get(name) {
            Some(v) => v,
            None => {
                cx.push_nil();
                return Ok(());
            }
        };

        match def.ty {
            ArgType::Bool => drop(cx.push_bool(self.vals.get_flag(name))),
            ArgType::String => match self.vals.get_one::<String>(name) {
                Some(v) => drop(cx.push_str(v)),
                None => drop(cx.push_nil()),
            },
        }

        Ok(())
    }
}
