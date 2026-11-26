//! 3D plants, a prototype (Ronald, 11-05: "let's try model like grass or like sugarcane crops in 3D"): a
//! resource pack (`mods/base/packs/plants_3d/`, built by `source/plants3d/build.py`) that redraws tall grass,
//! sugar cane, wheat, carrots and beans as small box models through `.ctm` model tiles (docs/ctm.md). Only
//! the drawing changes, so it is a pack the player turns on or off in Settings > Resource Packs.
//!
//! The client writes it next to the game (`resourcepacks/plants_3d/`) after the join, as the shader pack is
//! written (`shaderpack.rs`), and reloads the assets when the pack is on and its files changed (the first
//! run that has it turned on finds the folder only then). `--resource-pack <id>` turns a pack on for a
//! scripted run (`--resource-pack plants_3d`), as `--shader-pack` does for shader packs.

use std::path::Path;
use std::sync::Once;

use weft_sdk::settings::RESOURCE_PACKS;
use weft_sdk::*;

pub const ID: &str = "plants_3d";

pub fn register(r: &mut Registry) {
    static ONCE: Once = Once::new();
    r.client_frame(|c| {
        if c.state().flags & weft_sdk::client::client_flags::JOINED != 0 {
            ONCE.call_once(|| install(c));
        }
    });
}

fn install(c: &Client) {
    let dir = Path::new(&c.run_dir()).join("resourcepacks").join(ID);
    let mut files_changed = false;
    for (rel, bytes) in crate::plants3d_files::FILES {
        let path = dir.join(rel);
        if std::fs::read(&path).ok().as_deref() == Some(*bytes) {
            continue;
        }
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                weft_sdk::warn!("3d plants pack: {}: {e}", parent.display());
                return;
            }
        }
        match std::fs::write(&path, bytes) {
            Ok(()) => files_changed = true,
            Err(e) => weft_sdk::warn!("3d plants pack: {}: {e}", path.display()),
        }
    }
    let on = |packs: &str| packs.split(',').any(|p| p.trim() == ID);
    let mut changed = files_changed && on(&RESOURCE_PACKS.get());
    if let Some(pack) = weft_sdk::client::launch_option("resource-pack") {
        if RESOURCE_PACKS.get() != pack {
            RESOURCE_PACKS.set(&pack);
            changed = true;
        }
    }
    if changed {
        c.reload_assets();
    }
}
