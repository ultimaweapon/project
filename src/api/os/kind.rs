use zl::{Context, Error, Frame, NonYieldable};

pub fn entry(cx: &mut Context<NonYieldable>) -> Result<(), Error> {
    let v = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        todo!()
    };

    cx.push_str(v);
    Ok(())
}
