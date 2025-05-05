use zl::{Context, Error, Frame, Lua, PositiveInt, class};

pub fn register(lua: &mut Lua) {
    lua.register_ud::<Url>();
}

/// Implementation of `Url` class.
struct Url(url::Url);

#[class(global)]
impl Url {
    #[class]
    fn new(cx: &mut Context) -> Result<(), Error> {
        let url = cx.to_str(PositiveInt::TWO);
        let url = url::Url::parse(url).map_err(|e| Error::arg_from_std(PositiveInt::TWO, e))?;

        cx.push_ud(Self(url));

        Ok(())
    }

    #[prop]
    fn path(cx: &mut Context) -> Result<(), Error> {
        let url = cx.to_ud::<Self>(PositiveInt::ONE).into_ud();

        cx.push_str(url.0.path());

        Ok(())
    }
}
