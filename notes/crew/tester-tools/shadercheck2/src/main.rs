//! Validates the chiacraft shader pack (mods/base/shaderpack) offline, composed as the engine does:
//! the shader's prelude and common.wgsl, then MIST_TOP, look.wgsl and the file, then hook dispatch.
//! Usage: shadercheck2 <repo>. Never runs the game.
use weft_engine::{client, gpu_layout as g, post, shaders};
use shaders::Hook;

fn main() {
    let repo = std::path::PathBuf::from(std::env::args().nth(1).expect("repo"));
    let rd = |p: &str| std::fs::read_to_string(repo.join(p)).unwrap_or_else(|e| panic!("{p}: {e}"));
    let common = rd("crates/engine/assets/shaders/common.wgsl");
    let look = rd("mods/base/shaderpack/look.wgsl");
    let pack = |f: &str| format!("// From vitals::air\nconst MIST_TOP: f32 = 18.0;\n{look}\n{}", rd(&format!("mods/base/shaderpack/{f}")));
    let mut bad = 0;
    let mut check = |label: &str, src: String, hooks: &[&Hook]| {
        let parts: Vec<(&Hook, &[(String, String)])> = hooks.iter().map(|h| (*h, &[][..])).collect();
        match shaders::compose(label, &src, &parts) {
            Ok(_) => println!("ok   {label}"),
            Err(e) => {
                bad += 1;
                println!("FAIL {label}: {}", e.0)
            }
        }
    };
    let with = |name: &str, f: &str| format!("{}{}", g::prelude(name, &common), pack(f));
    let files: [(&str, Vec<&Hook>); 7] = [
        ("world.wgsl", vec![&client::world_render::HOOK]),
        ("sky.wgsl", vec![&client::sky::HOOK]),
        ("model.wgsl", vec![&client::entity_render::HOOK, &client::entity_render::EFFECT_HOOK]),
        ("entity.wgsl", vec![]),
        ("particle.wgsl", vec![&client::particles::HOOK]),
        ("weather.wgsl", vec![]),
        ("post.wgsl", vec![&post::HOOK, &post::HOOK_LOW]),
    ];
    for (f, hooks) in files.iter() {
        let src = with(f, f);
        check(f, src.clone(), hooks);
        if *f == "world.wgsl" {
            check("world.wgsl (no ground map)", src.replacen(g::GROUND_MAP_ON, g::GROUND_MAP_OFF, 1), hooks);
        }
        if src.contains("CALM") {
            let calm = src.replacen(g::CALM_OFF, g::CALM_ON, 1);
            check(&format!("{f} (calm)"), calm.clone(), hooks);
            if src.contains("DRY") {
                check(&format!("{f} (calm, dry)"), calm.replacen(g::DRY_OFF, g::DRY_ON, 1), hooks);
            }
        }
    }
    // The engine's own world shader (the plain look), which carries chiacraft's face flags too.
    let builtin = format!("{}{}", g::prelude("world.wgsl", &common), rd("crates/engine/assets/shaders/world.wgsl"));
    check("engine world.wgsl", builtin, &[&client::world_render::HOOK]);
    let pt = format!("{}{}{}", g::prelude("world.wgsl", &common), g::PT_DECLS, pack("pathtrace.wgsl"));
    check("pathtrace.wgsl", pt, &[]);
    std::process::exit(if bad > 0 { 1 } else { 0 });
}
