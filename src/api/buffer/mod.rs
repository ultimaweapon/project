use self::new::new;
use zl::{Frame, Lua};

mod new;

pub fn register(lua: &mut Lua) {
    // Buffer
    let mut g = lua.set_global(c"Buffer");
    let mut t = g.push_table(0, 1);

    t.set(c"new").push_fn(new);
}
