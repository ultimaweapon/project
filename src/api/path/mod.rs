use super::join_path;
use crate::App;
use std::path::{Path, PathBuf};
use tsuki::context::{Args, Context, Ret};
use tsuki::{Lua, Module, Ref, Table, fp};

/// Implementation of [Module] for `path` API.
pub struct PathModule;

impl Module<App> for PathModule {
    const NAME: &str = "path";

    type Inst<'a> = Ref<'a, Table<App>>;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let m = lua.create_table();

        m.set_str_key("basename", fp!(basename));
        m.set_str_key("dirname", fp!(dirname));
        m.set_str_key("join", fp!(join));

        Ok(m)
    }
}

fn basename(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let path = cx.arg(1);
    let path = path
        .to_str()?
        .as_utf8()
        .ok_or_else(|| path.error("expect UTF-8 string"))?;
    let path = Path::new(path);

    if let Some(v) = path.file_name() {
        // We requires argument to be UTF-8 so this will never fails.
        cx.push_str(v.to_str().unwrap())?;
    }

    Ok(cx.into())
}

fn dirname(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
    let path = cx.arg(1);
    let path = path
        .to_str()?
        .as_utf8()
        .ok_or_else(|| path.error("expect UTF-8 string"))?;
    let path = Path::new(path);

    if let Some(v) = path.parent() {
        // We requires argument to be UTF-8 so this will never fails.
        cx.push_str(v.to_str().unwrap())?;
    }

    Ok(cx.into())
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
