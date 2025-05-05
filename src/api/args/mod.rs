use crate::manifest::{ArgName, ArgType, CommandArg};
use clap::ArgMatches;
use rustc_hash::FxHashMap;
use std::any::type_name;
use std::ffi::{CStr, CString};
use std::panic::AssertUnwindSafe;
use std::sync::LazyLock;
use zl::{Context, Error, Frame, Lua, PositiveInt, Table, UserType};

pub fn register(lua: &mut Lua, defs: FxHashMap<ArgName, CommandArg>, args: ArgMatches) {
    lua.register_ud::<Args>();

    lua.set_global(c"args").push_ud(Args {
        defs,
        vals: AssertUnwindSafe(args),
    });
}

/// Class of the global variable `args`.
struct Args {
    defs: FxHashMap<ArgName, CommandArg>,
    vals: AssertUnwindSafe<ArgMatches>,
}

impl Args {
    fn get(&self, cx: &mut Context) -> Result<(), Error> {
        let name = cx.to_str(PositiveInt::TWO);
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

impl UserType for Args {
    fn name() -> &'static CStr {
        static NAME: LazyLock<CString> =
            LazyLock::new(|| CString::new(type_name::<Args>()).unwrap());

        NAME.as_c_str()
    }

    fn setup<P: Frame>(meta: &mut Table<P>) {
        meta.set(c"__index")
            .push_fn(|cx| cx.to_ud::<Self>(PositiveInt::ONE).into_ud().get(cx));
    }
}
