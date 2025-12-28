use super::join_path;
use crate::App;
use std::path::PathBuf;
use tsuki::context::{Args, Context, Ret};
use tsuki::{Lua, Module, Ref, Table, fp};

/// Implementation of [Module] for `path` API.
pub struct PathModule;

impl Module<App> for PathModule {
    const NAME: &str = "path";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.create_table();

        m.set_str_key("join", fp!(join));

        Ok(m)
    }
}

fn join(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let mut path = PathBuf::new();

    join_path(&cx, |_, c| {
        path.push(c);
        Ok(())
    })?;

    // join_path() requires all components to be UTF-8 so this should never fails.
    cx.push_str(path.to_str().unwrap())?;

    Ok(cx.into())
}
