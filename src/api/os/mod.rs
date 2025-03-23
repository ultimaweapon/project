use lua54::{Frame, Table};

mod arch;
mod kind;
mod run;

pub fn register<P: Frame>(mut t: Table<P>) {
    t.set(c"arch").push_fn(self::arch::entry);
    t.set(c"kind").push_fn(self::kind::entry);
    t.set(c"run").push_fn(self::run::entry);
}
