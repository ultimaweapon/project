use lua54::Engine;

use self::new::new;

mod new;

pub fn register(en: &mut Engine) {
    // new
    en.push_table(0, 1);
    en.push_fn(new);
    unsafe { en.set_field(-2, c"new") };

    unsafe { en.set_global(c"Buffer") };
}
