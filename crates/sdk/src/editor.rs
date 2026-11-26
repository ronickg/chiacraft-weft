//! Describing mod-defined builder methods for the content editor.

use crate::{Component, Record, Registry, WithComponents, abi};

pub use abi::editor::{BLOCK, ITEM, Method, Param};

/// A typed default supplies the value layout and initial Rust literal of a parameter.
pub fn parameter<T: abi::data::Data + std::fmt::Debug>(name: &str, ty: &str, component: &str, value: T) -> Param {
    let mut literal = format!("{value:?}");
    if let Some(at) = literal.find(" {") {
        literal.replace_range(..at, ty);
    }
    Param { name: name.into(), ty: ty.into(), schema: T::schema(), default: literal, picture: String::new(), component: component.into() }
}

impl Registry {
    /// Adds a mod's public builder method to the editor. `import` names its extension trait.
    /// Its parameters and defaults describe the actual Rust call emitted by the editor.
    pub fn editor_method(&mut self, method: Method) {
        let record = Record::generated(format!("editor/{}", method.name));
        self.record(record)
            .set(&Component::<Method>::named(abi::comp::EDITOR_METHOD), method);
    }
}
