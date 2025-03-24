use zl::{Error, Frame, FuncState};

pub fn entry(lua: &mut FuncState) -> Result<(), Error> {
    let v = if cfg!(target_os = "windows") {
        c"windows"
    } else if cfg!(target_os = "macos") {
        c"macos"
    } else if cfg!(target_os = "linux") {
        c"linux"
    } else {
        todo!()
    };

    lua.push_string(v);
    Ok(())
}
