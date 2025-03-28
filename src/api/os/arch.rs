use zl::{Context, Error, Frame};

pub fn entry(cx: &mut Context) -> Result<(), Error> {
    let v = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        todo!()
    };

    cx.push_str(v);
    Ok(())
}
