//! The animals mod's logic that needs no game world: the species table, genes and breeding,
//! coats and builds from genes, scars, and evolution's generations. The mod (`mods/animals`)
//! builds on it in the game; the website's simulator (`mods/animals/web`, compiled to WASM) runs
//! the very same code, so what is tested there is what the game does.

use std::sync::Mutex;

pub mod balance;
pub mod biology;
pub mod bodies;
pub mod coats;
pub mod evolution;
pub mod hides;
pub mod journal;
pub mod life;
pub mod meat;
pub mod needs;
pub mod sheet;
pub mod sky;
pub mod species;
pub mod voices;
pub mod web;

static PROBLEMS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Something wrong in the data (a gene file that doesn't parse): kept for whoever runs this to
/// report (`problems`).
pub(crate) fn problem(text: String) {
    PROBLEMS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(text);
}

/// The problems found so far, taken (each reported once).
pub fn problems() -> Vec<String> {
    std::mem::take(&mut *PROBLEMS.lock().unwrap_or_else(|e| e.into_inner()))
}

#[cfg(feature = "sdk")]
unsafe impl weft_sdk::Value for coats::Genome {
    // As `weft_sdk::value!` writes it, so saves and other mods read it alike.
    fn schema() -> String {
        format!(
            "{{seed:{},species:{},breed:{},female:{},genes:{},core:{},version:{},stage:{},scars:{},temper:{}}}",
            <u32 as weft_sdk::Value>::schema(),
            <u8 as weft_sdk::Value>::schema(),
            <u8 as weft_sdk::Value>::schema(),
            <u8 as weft_sdk::Value>::schema(),
            <[u8; 32] as weft_sdk::Value>::schema(),
            <[u8; 32] as weft_sdk::Value>::schema(),
            <u8 as weft_sdk::Value>::schema(),
            <u8 as weft_sdk::Value>::schema(),
            <[u8; 8] as weft_sdk::Value>::schema(),
            <[u8; 2] as weft_sdk::Value>::schema()
        )
    }
}
