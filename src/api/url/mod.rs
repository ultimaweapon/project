use zl::{Context, Error, Frame, Lua, NonYieldable, class};

pub fn register(lua: &mut Lua) {
    assert!(lua.register_ud::<Url>());
}

/// Implementation of `Url` class.
struct Url(url::Url);

#[class]
impl Url {
    #[class]
    fn new(cx: &mut Context<NonYieldable>) -> Result<(), Error> {
        let url = url::Url::parse(cx.to_str(2)).map_err(|e| Error::arg_from_std(2, e))?;

        cx.push_ud(Self(url));

        Ok(())
    }

    fn path(&self, cx: &mut Context<NonYieldable>) -> Result<(), Error> {
        cx.push_str(self.0.path());
        Ok(())
    }
}
