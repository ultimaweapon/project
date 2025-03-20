use crate::script::Engine;

pub fn register(en: &mut Engine) {
    // os.arch()
    en.push_fn(move |en| {
        let v = if cfg!(target_arch = "x86_64") {
            c"x86_64"
        } else if cfg!(target_arch = "aarch64") {
            c"aarch64"
        } else {
            todo!()
        };

        en.push_string(v);
        1
    });

    unsafe { en.set_field(-2, c"arch") };

    // os.kind()
    en.push_fn(move |en| {
        let v = if cfg!(target_os = "windows") {
            c"windows"
        } else if cfg!(target_os = "macos") {
            c"macos"
        } else if cfg!(target_os = "linux") {
            c"linux"
        } else {
            todo!()
        };

        en.push_string(v);
        1
    });

    unsafe { en.set_field(-2, c"kind") };
}
