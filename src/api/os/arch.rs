use zl::{Error, Frame, FuncState};

pub fn entry(lua: &mut FuncState) -> Result<(), Error> {
    let v = if cfg!(target_arch = "x86_64") {
        c"x86_64"
    } else if cfg!(target_arch = "aarch64") {
        c"aarch64"
    } else {
        todo!()
    };

    lua.push_string(v);
    Ok(())
}
