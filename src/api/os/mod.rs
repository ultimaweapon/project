use crate::script::Engine;

pub fn register(en: &mut Engine) {
    // os.kind().
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
