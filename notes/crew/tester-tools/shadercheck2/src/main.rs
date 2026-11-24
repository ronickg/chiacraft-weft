//! Validates the chiacraft shader pack (mods/base/shaderpack) offline, composed as the engine does:
//! the shader's prelude and common.wgsl, then MIST_TOP, look.wgsl and the file, then hook dispatch.
//! Each shader that validates then goes the game's Mac path: naga -> SPIR-V (the Vulkan backend's
//! options) -> SPIRV-Cross MSL (as MoltenVK runs it) -> Apple's `metal` compiler.
//! Usage: shadercheck2 <repo> [--no-msl]. Never runs the game.
use shaders::Hook;
use spirv_cross2::compile::msl::{ArgumentBuffersTier, MslVersion};
use spirv_cross2::compile::CompilableTarget;
use spirv_cross2::targets::Msl;
use spirv_cross2::{Compiler, Module};
use weft_engine::{client, gpu_layout as g, post, shaders};
use weft_render::Shader;

/// The same SPIR-V options as `backends/vulkan` (`spirv`): SPIR-V 1.3, the prelude's samplers in set 2.
fn spirv(shader: &Shader, stage: naga::ShaderStage, entry: &str) -> Result<Vec<u32>, String> {
    let mut options = naga::back::spv::Options::default();
    options.lang_version = (1, 3);
    for binding in 1..5 {
        options.binding_map.insert(
            naga::ResourceBinding { group: 0, binding },
            naga::back::spv::BindingInfo { descriptor_set: 2, binding: binding - 1, binding_array_size: None },
        );
    }
    options.force_loop_bounding = false;
    let pipeline = naga::back::spv::PipelineOptions { shader_stage: stage, entry_point: entry.to_string() };
    naga::back::spv::write_vec(&shader.module, &shader.info, &options, Some(&pipeline)).map_err(|e| format!("SPIR-V: {e}"))
}

/// SPIR-V to MSL with SPIRV-Cross, then compiled by `xcrun metal`. Returns the first error lines.
fn metal(words: &[u32], out: &std::path::Path) -> Result<(), String> {
    let compiler = Compiler::<Msl>::new(Module::from_words(words)).map_err(|e| format!("SPIRV-Cross: {e}"))?;
    let mut options = Msl::options();
    options.version = MslVersion::new(3, 0, 0);
    options.argument_buffers = true;
    options.argument_buffers_tier = ArgumentBuffersTier::Tier2;
    let msl = compiler.compile(&options).map_err(|e| format!("SPIRV-Cross MSL: {e}"))?;
    std::fs::write(out, msl.to_string()).map_err(|e| e.to_string())?;
    let run = std::process::Command::new("xcrun")
        .args(["-sdk", "macosx", "metal", "-std=metal3.0", "-c", "-o", "/dev/null"])
        .arg(out)
        .output()
        .map_err(|e| format!("xcrun metal: {e}"))?;
    if run.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&run.stderr);
    let lines: Vec<&str> = err.lines().filter(|l| l.contains("error")).take(4).collect();
    Err(format!("metal ({}):\n       {}", out.display(), lines.join("\n       ")))
}

fn main() {
    let repo = std::path::PathBuf::from(std::env::args().nth(1).expect("repo"));
    let msl = !std::env::args().any(|a| a == "--no-msl");
    let out_dir = std::env::temp_dir().join("shadercheck2-msl");
    let _ = std::fs::create_dir_all(&out_dir);
    let rd = |p: &str| std::fs::read_to_string(repo.join(p)).unwrap_or_else(|e| panic!("{p}: {e}"));
    let common = rd("crates/engine/assets/shaders/common.wgsl");
    let look = rd("mods/base/shaderpack/look.wgsl");
    let pack = |f: &str| format!("// From vitals::air\nconst MIST_TOP: f32 = 18.0;\n{look}\n{}", rd(&format!("mods/base/shaderpack/{f}")));
    let mut bad = 0;
    let mut check = |label: &str, src: String, hooks: &[&Hook]| {
        let parts: Vec<(&Hook, &[(String, String)])> = hooks.iter().map(|h| (*h, &[][..])).collect();
        let shader = match shaders::compose(label, &src, &parts) {
            Ok((shader, _)) => shader,
            Err(e) => {
                bad += 1;
                println!("FAIL {label}: {}", e.0);
                return;
            }
        };
        if !msl {
            println!("ok   {label}");
            return;
        }
        let mut errors = Vec::new();
        for ep in shader.module.entry_points.iter() {
            let file = out_dir.join(format!("{}.{}.metal", label.replace([' ', '(', ')', ','], "_"), ep.name));
            if let Err(e) = spirv(&shader, ep.stage, &ep.name).and_then(|w| metal(&w, &file)) {
                errors.push(format!("{}: {e}", ep.name));
            }
        }
        if errors.is_empty() {
            println!("ok   {label} (naga, metal)");
        } else {
            bad += 1;
            println!("FAIL {label}: {}", errors.join("\n     "));
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
