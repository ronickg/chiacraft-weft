//! A chiacraft mod's side of the engine's bridge (`crates/engine/src/chiacraft/bridge.rs`, which
//! lists the questions): what our own mods may ask beyond Weft's mod interface, as text.
//!
//! A mod says `chiacraft_bridge::install!();` once, at its crate root, and then asks with
//! [`ask`]. The engine hands the bridge over when it loads the mod; until then, and in a game
//! without it, every answer is empty.

use std::sync::OnceLock;

use weft_abi::Str;

/// The engine's `bridge::query`.
pub type Query = unsafe extern "C" fn(query: Str, arg: Str) -> Str;

static BRIDGE: OnceLock<Query> = OnceLock::new();

/// Asks the engine `query` about `arg`; empty when it has no answer or no bridge.
pub fn ask(query: &str, arg: &str) -> String {
    BRIDGE
        .get()
        // SAFETY: the engine's query, called with live strings; its answer is copied at once.
        .map(|f| unsafe { f(Str::new(query), Str::new(arg)).as_str().to_string() })
        .unwrap_or_default()
}

#[doc(hidden)]
pub fn __set(q: Query) {
    let _ = BRIDGE.set(q);
}

/// Exports `chiacraft_bridge`, which the engine calls with its query when it loads the mod. It
/// must be written in the mod itself: an export from a dependency can be dropped by the linker.
#[macro_export]
macro_rules! install {
    () => {
        /// The engine hands the mod its bridge when the mod loads (`chiacraft_bridge::ask`).
        ///
        /// # Safety
        /// `q` must be the engine's `bridge::query`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn chiacraft_bridge(q: $crate::Query) {
            $crate::__set(q);
        }
    };
}
