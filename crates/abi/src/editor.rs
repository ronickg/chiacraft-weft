//! Builder methods contributed to the content editor by mods.

pub const BLOCK: u8 = 1;
pub const ITEM: u8 = 2;

crate::data! {
    pub struct Param {
        pub name: alloc::string::String,
        pub ty: alloc::string::String,
        pub schema: alloc::string::String,
        pub default: alloc::string::String,
        pub picture: alloc::string::String,
        pub component: alloc::string::String
    }
}

crate::data! {
    pub struct Method {
        pub on: u8,
        pub name: alloc::string::String,
        pub doc: alloc::string::String,
        pub import: alloc::string::String,
        pub params: alloc::vec::Vec<Param>
    }
}
