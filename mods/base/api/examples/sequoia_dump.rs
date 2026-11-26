//! Prints giant sequoias grown from branches (`tree_recipes::SEQUOIA`, `YOUNG_SEQUOIA`) as JSON
//! for the shapes sheet (`mods/base/source/trees/sequoia_sheet.py`):
//! `{"grown": [[[x, y, z, part], ...], ...], "young": [...]}` (part 0 leaves; 1, 2, 3 wood along
//! x, y, z), `count` trees each from seed 0.
//!
//!     cargo run -p base_api --example sequoia_dump -- [count] > sequoias.json

use base_api::branchy::{Part, Recipe, blocks};
use base_api::tree_recipes::{SEQUOIA, YOUNG_SEQUOIA};

fn trees(r: &Recipe, count: u64) -> String {
    let t: Vec<String> = (0..count)
        .map(|seed| {
            let b: Vec<String> = blocks(r, seed)
                .iter()
                .map(|&(x, y, z, p)| {
                    let k = match p {
                        Part::Leaves => 0,
                        Part::Log { axis } => 1 + axis,
                    };
                    format!("[{x},{y},{z},{k}]")
                })
                .collect();
            format!("[{}]", b.join(","))
        })
        .collect();
    format!("[{}]", t.join(","))
}

fn main() {
    let count: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);
    println!("{{\"grown\":{},\"young\":{}}}", trees(&SEQUOIA, count), trees(&YOUNG_SEQUOIA, 2));
}
