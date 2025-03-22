use lua54::Engine;

mod arch;
mod kind;
mod run;

pub fn register(en: &mut Engine) {
    // os.arch()
    en.push_fn(self::arch::entry);
    unsafe { en.set_field(-2, c"arch") };

    // os.kind()
    en.push_fn(self::kind::entry);
    unsafe { en.set_field(-2, c"kind") };

    // os.run()
    en.push_fn(self::run::entry);
    unsafe { en.set_field(-2, c"run") };
}
