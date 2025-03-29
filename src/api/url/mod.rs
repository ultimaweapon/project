use zl::{Context, Error, Frame, Lua, class};

pub fn register(lua: &mut Lua) {
    // Url
    let mut g = lua.set_global(c"Url");
    let mut t = g.push_table(0, 1);

    t.set(c"new").push_fn(Url::new);
}

struct Url(url::Url);

impl Url {
    fn new(cx: &mut Context) -> Result<(), Error> {
        Ok(())
    }
}

#[class]
impl Url {}
