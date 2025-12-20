use crate::App;
use tsuki::context::{Args, Context, Ret};
use tsuki::{Lua, Module, Ref, Table, fp};

/// Implementation of [Module] for `Url` class.
pub struct UrlModule;

impl Module<App> for UrlModule {
    const NAME: &str = "Url";

    type Inst<'a>
        = Ref<'a, Table<App>>
    where
        App: 'a;

    fn open(self, lua: &Lua<App>) -> Result<Self::Inst<'_>, Box<dyn core::error::Error>> {
        let g = lua.create_table();

        g.set_str_key("new", fp!(Url::new));

        Ok(g)
    }
}

/// Implementation of `Url` class.
struct Url;

impl Url {
    fn new(cx: Context<App, Args>) -> Result<Context<App, Ret>, Box<dyn std::error::Error>> {
        // Parse URL.
        let url = cx.arg(2);
        let url = match url.to_str()?.as_utf8() {
            Some(v) => url::Url::parse(v).map_err(|e| url.error(e))?,
            None => return Err("expect UTF-8 string".into()),
        };

        // Create userdata.
        let ud = cx.create_ud(Self);

        ud.set("path", cx.create_str(url.path()));

        cx.push(ud)?;

        Ok(cx.into())
    }
}
