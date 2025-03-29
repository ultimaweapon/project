use zl::{Context, Error, Frame, Lua, UserData, class};

pub fn register(lua: &mut Lua) {
    // Url class.
    let mut g = lua.set_global(Url::name());
    let mut t = g.push_table(0, 1);

    t.set(c"new").push_fn(Url::new);
}

/// Implementation of `Url` class.
struct Url(url::Url);

impl Url {
    fn new(cx: &mut Context) -> Result<(), Error> {
        let url = url::Url::parse(cx.to_str(2)).map_err(|e| Error::arg_from_std(2, e))?;

        cx.push_ud(Self(url));

        Ok(())
    }
}

#[class]
impl Url {
    fn path(&self, cx: &mut Context) -> Result<(), Error> {
        cx.push_str(self.0.path());
        Ok(())
    }
}
