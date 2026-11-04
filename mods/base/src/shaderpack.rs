//! chiacraft's look (`mods/base/shaderpack/`): a shader pack the client writes next to the game
//! (`shaderpacks/chiacraft/`) and turns on the first time, so every player has it and can still
//! pick another one or none in the video settings. Each file goes out with `look.wgsl` in front:
//! the colors of the day, the sky, the light and the grade the files share.
//!
//! It waits for the join: turning a pack on in the first frames, before the server let the player
//! in, hung the GPU (an NVIDIA Xid 109; on an Apple M3 Max the whole machine restarted), and the
//! engine now holds such a reload back too. Checked on an RTX 5090 and an M3 Max (2026-11-04).

use std::path::Path;
use std::sync::Once;

use weft_sdk::*;
use weft_sdk::settings::SHADER_PACK;

pub const ID: &str = "chiacraft";

const LOOK: &str = include_str!("../shaderpack/look.wgsl");

const FILES: [(&str, &str); 6] = [
    ("world.wgsl", include_str!("../shaderpack/world.wgsl")),
    ("sky.wgsl", include_str!("../shaderpack/sky.wgsl")),
    ("model.wgsl", include_str!("../shaderpack/model.wgsl")),
    ("entity.wgsl", include_str!("../shaderpack/entity.wgsl")),
    ("particle.wgsl", include_str!("../shaderpack/particle.wgsl")),
    ("weather.wgsl", include_str!("../shaderpack/weather.wgsl")),
];

const CFG: &str = "name = chiacraft\ndescription = Sun and shadows, sky, haze and water\n";

/// Only `--shader-pack <id>` (a script checking a look; tests reset packs when they start, so
/// it is turned on after that), `--shader-pack-after <secs>` seconds into the run (0 by
/// default, counted from the join; the engine holds back reloads before it anyway): no files
/// written, nothing turned on for players.
pub fn register_for_scripts(r: &mut Registry) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static STARTED: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    static TIMES: AtomicU32 = AtomicU32::new(0);
    r.client_frame(|c| {
        let Some(pack) = weft_sdk::client::launch_option("shader-pack") else { return };
        let after: f32 = weft_sdk::client::launch_option("shader-pack-after").and_then(|s| s.parse().ok()).unwrap_or(0.0);
        if c.state().flags & weft_sdk::client::client_flags::JOINED == 0 {
            return;
        }
        if STARTED.get_or_init(std::time::Instant::now).elapsed().as_secs_f32() < after {
            return;
        }
        // The testkit turns packs off when a test begins, which on a slow start comes after the
        // join: it is turned on again then, a few times at most.
        if SHADER_PACK.get() != pack && TIMES.fetch_add(1, Ordering::Relaxed) < 3 {
            weft_sdk::info!("shader pack {pack} on");
            SHADER_PACK.set(&pack);
            c.reload_assets();
        }
    });
}

/// The pack's light shafts (`assets/shaders/post/shafts.wgsl`, a screen effect: packs cannot
/// add passes): on while chiacraft's pack is the one chosen, off with any other or none.
pub fn register_shafts(r: &mut Registry) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ON: AtomicBool = AtomicBool::new(false);
    r.client_frame(|c| {
        // Out of a world: turned on again at the next join.
        if c.state().flags & weft_sdk::client::client_flags::JOINED == 0 {
            ON.store(false, Ordering::Relaxed);
            return;
        }
        let want = SHADER_PACK.get() == ID;
        if ON.swap(want, Ordering::Relaxed) != want {
            let known = c.post_effect("shafts", if want { 1.0 } else { 0.0 });
            weft_sdk::info!("light shafts {}{}", if want { "on" } else { "off" }, if known { "" } else { " (no such effect)" });
        }
    });
}

pub fn register(r: &mut Registry) {
    static ONCE: Once = Once::new();
    // After the join: a reload before it hung the GPU (the engine now holds one back itself).
    r.client_frame(|c| {
        if c.state().flags & weft_sdk::client::client_flags::JOINED != 0 {
            ONCE.call_once(|| install(c));
        }
    });
}

/// Brings the folder up to date with this build; the first time the game is played, turns it on
/// unless the player already chose a pack. Tests and scripts keep their settings apart
/// (`test-config/`), so they neither turn it on nor count as that first time.
fn install(c: &Client) {
    let dir = Path::new(&c.run_dir()).join("shaderpacks").join(ID);
    let offered = dir.join(".offered");
    let automated = ["test", "script"].iter().any(|o| weft_sdk::client::launch_option(o).is_some());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        weft_sdk::warn!("shader pack: {}: {e}", dir.display());
        return;
    }
    let mut changed = false;
    let files = FILES
        .iter()
        .map(|(name, src)| (*name, format!("{LOOK}\n{src}")))
        .chain(std::iter::once(("pack.cfg", CFG.to_string())));
    for (name, text) in files {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            match std::fs::write(&path, &text) {
                Ok(()) => changed = true,
                Err(e) => weft_sdk::warn!("shader pack: {}: {e}", path.display()),
            }
        }
    }
    // `--shader-pack <id>` picks one for this run (a script checking the look).
    if let Some(pack) = weft_sdk::client::launch_option("shader-pack") {
        if SHADER_PACK.get() != pack {
            SHADER_PACK.set(&pack);
            changed = true;
        }
    } else if !automated && !offered.exists() {
        if SHADER_PACK.get().is_empty() {
            SHADER_PACK.set(ID);
            changed = true;
        }
        let _ = std::fs::write(&offered, "");
    }
    if changed {
        c.reload_assets();
    }
}
