/// Type of Lua value.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Type {
    None = -1,
    Nil,
    Boolean,
    LightUserData,
    Number,
    String,
    Table,
    Function,
    Userdata,
    Thread,
}
