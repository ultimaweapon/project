use zl::{Frame, Lua, Table};

mod arch;
mod capture;
mod kind;
mod run;
mod spawn;

pub fn register(mut t: Table<Lua>) {
    t.register_ud::<self::spawn::Process>();

    t.set(c"arch").push_fn(self::arch::entry);
    t.set(c"capture").push_fn(self::capture::entry);
    t.set(c"kind").push_fn(self::kind::entry);
    t.set(c"run").push_fn(self::run::entry);
    t.set(c"spawn").push_fn(self::spawn::entry);
}
