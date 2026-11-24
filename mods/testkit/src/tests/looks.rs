//! Particles, text effects and conditions, shaders from mods, glow masks, models.

use std::time::Duration;

use weft_sdk::*;

use super::{add, probe};
use crate::runner::{R, T};

static SPARK: Particle = Particle::named("testmod:spark");

/// Huge counts stay cheap: moved by a compute shader, drawn in one instanced draw.
pub async fn particle_crowd(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    let pos = [f[0] as f64 + 0.5, f[1] as f64 + 1.5, f[2] as f64 - 2.5];
    for _ in 0..10 {
        drop(
            t.c()
                .spawn_particles(&SPARK, pos)
                .count(10_000)
                .spread(4.0)
                .speed(1.0),
        );
    }
    t.frames(2).await?;
    let n = t.state().particles;
    let b = t.bench(0.6, "100k particles").await?;
    t.note(format!("{n} particles: {:.0} fps, frame {:.2} ms, gpu {:.2} ms", b[0], b[1], b[3]));
    t.check(n >= 99_000 && b[1] < 33.0, format!("{n} particles in {:.1} ms frames", b[1]));
    Ok(())
}

static DROP: Particle = Particle::named("testmod:drop");

/// A kind's rules run on the GPU: testmod's drops turn into red rings falling into water and are
/// gone falling on the ground.
pub async fn particle_rules(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-3, -1, -7]), add(f, [3, -1, -3]), "water")
        .await?;
    t.c().set_look(0.0, -30.0);
    t.settled().await?;
    let rain = |at: [i32; 3]| {
        let pos = [at[0] as f64 + 0.5, at[1] as f64 + 3.0, at[2] as f64 + 0.5];
        drop(
            t.c()
                .spawn_particles(&DROP, pos)
                .count(300)
                .spread_box([2.5, 0.5, 1.5]),
        );
    };
    let middle = |s: &crate::runner::Shot| s.share(0.2, 0.3, 0.8, 0.9, probe::red);
    rain(add(f, [0, 0, -5]));
    t.secs(1.0).await?;
    let rings = middle(&t.shot("rings").await?);
    t.probe("drops falling into water turn into rings", rings, rings > 0.01);
    t.secs(3.0).await?;
    t.fill(add(f, [-3, -1, -7]), add(f, [3, -1, -3]), "stone")
        .await?;
    t.settled().await?;
    rain(add(f, [0, 0, -5]));
    t.secs(1.0).await?;
    let ground = t.shot("ground").await?;
    let red = middle(&ground);
    t.probe("drops on the ground leave nothing", red, red < 0.001);
    Ok(())
}

/// A stack's own `layers` show over one another, tinted, in the hotbar: the sprite of a tool
/// made of parts.
pub async fn layered_icon(t: T) -> R {
    t.flat().await?;
    t.c().select_slot(4);
    // The first hotbar slot, under the hearts.
    let hotbar = |s: &crate::runner::Shot| s.share(0.385, 0.95, 0.415, 0.995, probe::red);
    let before = hotbar(&t.shot("plain").await?);
    t.op("layered 0").await?;
    t.frames(6).await?;
    let after = hotbar(&t.shot("layered").await?);
    t.probe("red of the tinted layer in the hotbar", after - before, after - before > 0.04);
    Ok(())
}

/// A 16x16 texture of one color.
fn write_png(path: &str, rgb: [u8; 3]) -> Result<(), String> {
    let dir = std::path::Path::new(path).parent().ok_or("no folder")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), 16, 16);
    enc.set_color(png::ColorType::Rgba);
    let px: Vec<u8> = (0..256)
        .flat_map(|_| [rgb[0], rgb[1], rgb[2], 255])
        .collect();
    enc.write_header()
        .and_then(|mut w| w.write_image_data(&px))
        .map_err(|e| e.to_string())
}

/// Reloading assets picks up a resource pack's texture and later changes to that file.
pub async fn texture_reload(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-3, 0, -3]), add(f, [3, 3, -3]), "dirt")
        .await?;
    t.c().set_look(0.0, 0.0);
    t.settled().await?;
    // Time stands still and the GUI is off between the grabs: on CI "plain" and "back" are ~30 s
    // of game apart (~3 s on a GPU), and the clock, vitals and the hour's light change meanwhile.
    t.cmd("/tick freeze").await?;
    t.press("core:hide_hud");
    t.frames(3).await?;
    let plain = t.grab().await?;
    let pack = format!("{}/resourcepacks/testkit_reload", t.c().run_dir());
    let file = format!("{pack}/base/textures/block/dirt.png");
    let reload = async |rgb: Option<[u8; 3]>| -> Result<Vec<u8>, String> {
        match rgb {
            Some(c) => write_png(&file, c)?,
            None => drop(std::fs::remove_dir_all(&pack)),
        }
        t.c().reload_assets();
        t.frames(5).await?;
        t.settled().await?;
        t.grab().await
    };
    weft_sdk::settings::RESOURCE_PACKS.set("testkit_reload");
    let red = reload(Some([200, 40, 40])).await?;
    let blue = reload(Some([40, 40, 200])).await?;
    weft_sdk::settings::RESOURCE_PACKS.set("");
    let back = reload(None).await?;
    t.press("core:hide_hud");
    t.cmd("/tick unfreeze").await?;
    t.check(T::differ(&plain, &red) > 0.2, "the pack's dirt shows after a reload");
    t.check(T::differ(&red, &blue) > 0.2, "a changed file shows after the next reload");
    t.check(T::differ(&plain, &back) < 0.01, format!("without the pack the dirt is back: {:.4}", T::differ(&plain, &back)));
    Ok(())
}

/// A resource pack's translations change only the keys it has; the rest of the same mod's file
/// stays.
pub async fn pack_lang(t: T) -> R {
    let pack = format!("{}/resourcepacks/testkit_lang", t.c().run_dir());
    let dir = format!("{pack}/core/lang");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(format!("{dir}/en_us.lang"), "menu.done = Pack Done\n").map_err(|e| e.to_string())?;
    weft_sdk::settings::RESOURCE_PACKS.set("testkit_lang");
    t.c().reload_assets();
    t.frames(2).await?;
    t.c().menu_open(&screens::SETTINGS);
    t.screen("core:settings").await?;
    t.frames(2).await?;
    t.laid_out(&["text:Pack Done", "text:View distance", "text:Language"]);
    t.c().menu_close();
    weft_sdk::settings::RESOURCE_PACKS.set("");
    drop(std::fs::remove_dir_all(&pack));
    t.c().reload_assets();
    t.frames(2).await?;
    t.c().menu_open(&screens::SETTINGS);
    t.screen("core:settings").await?;
    t.frames(2).await?;
    t.check(t.c().ui_rect("text:Pack Done").is_none(), "without the pack the key is the mod's again");
    t.laid_out(&["text:Done"]);
    t.c().menu_close();
    Ok(())
}

/// A shader pack whose files do not compile changes nothing: every engine shader, the composed
/// ones (models, particles, screen effects, UI) too, falls back to the built-in one.
pub async fn broken_shader_pack(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.op(&format!("spawn testmod:idol {} {} {}", f[0], f[1], f[2] - 3))
        .await?;
    t.c().set_look(0.0, 0.0);
    t.settled().await?;
    t.secs(0.5).await?;
    let before = t.grab().await?;
    let dir = format!("{}/shaderpacks/testkit_broken", t.c().run_dir());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for name in ["world.wgsl", "entity.wgsl", "model.wgsl", "particle.wgsl", "post.wgsl", "ui.wgsl"] {
        std::fs::write(format!("{dir}/{name}"), "this is not wgsl").map_err(|e| e.to_string())?;
    }
    weft_sdk::settings::SHADER_PACK.set("testkit_broken");
    t.c().reload_assets();
    t.frames(5).await?;
    let after = t.grab().await?;
    weft_sdk::settings::SHADER_PACK.set("");
    t.c().reload_assets();
    let _ = std::fs::remove_dir_all(&dir);
    t.frames(2).await?;
    t.check(T::differ(&before, &after) < 0.02, format!("the picture stays ({:.1}% differ)", T::differ(&before, &after) * 100.0));
    // A working pack only writes functions: the engine's own world shader with a sepia ending,
    // its structs and bindings come from the prelude.
    let dir = format!("{}/shaderpacks/testkit_sepia", t.c().run_dir());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let world = include_str!("../../../../crates/engine/assets/shaders/world.wgsl");
    // (Ours fogs as vanilla and Distant Horizons do: `fogged` and `plain_fog`, chiacraft stage 1.)
    let ending = "    return vec4(fogged(lit.rgb, plain_fog(in.dist)), lit.a);";
    t.check(world.contains(ending), "the world shader still ends its shading like this");
    // Packs are copies of it, from whenever they were made: four words a texture since Weft
    // 26.11.2 (chiacraft's pack world.wgsl reads it so too).
    t.check(world.contains("tex_info[name * 4u]"), "the texture table still reads as packs read it");
    let sepia = world.replace(
        ending,
        "    let y = dot(fogged(lit.rgb, plain_fog(in.dist)), vec3(0.3, 0.59, 0.11));
    return vec4(vec3(y * 1.2, y, y * 0.75), lit.a);",
    );
    std::fs::write(format!("{dir}/world.wgsl"), sepia).map_err(|e| e.to_string())?;
    weft_sdk::settings::SHADER_PACK.set("testkit_sepia");
    t.c().reload_assets();
    t.frames(5).await?;
    let shot = t.shot("sepia").await?;
    weft_sdk::settings::SHADER_PACK.set("");
    t.c().reload_assets();
    let _ = std::fs::remove_dir_all(&dir);
    t.frames(2).await?;
    // The grass floor turns brown: red over green over blue.
    let brown = shot.share(0.0, 0.7, 1.0, 0.9, |c| c[0] > c[1] && c[1] > c[2]);
    t.probe("a pack's sepia world shader is used", brown, brown > 0.8);
    Ok(())
}

/// The test mod's looks at night: a glow crystal's veins glow without lighting the floor next to
/// it, a wisp floats nearby, and the daydream screen effect changes the picture.
pub async fn shaders_and_glow(t: T) -> R {
    t.world(9).await?;
    t.idle().await?;
    t.op("time 18000").await?;
    let f = t.feet();
    // A level stage of its own, so terrain changes do not move the picture.
    t.fill(add(f, [-4, -1, -7]), add(f, [4, -1, 3]), "stone")
        .await?;
    t.fill(add(f, [-4, 0, -7]), add(f, [4, 5, 3]), "air")
        .await?;
    let crystal = add(f, [0, 1, -4]);
    t.fill(add(f, [-1, 0, -4]), add(f, [1, 2, -4]), "testmod:glow_crystal")
        .await?;
    // A block shader (`shaders/block/stripes.wgsl`): magenta bands, lit or not.
    let striped = add(f, [-3, 1, -4]);
    t.set(striped, "testmod:reinforced_stone").await?;
    // Off to the side and back a little: it drifts, so it stays small in the picture.
    t.op(&format!("spawn testmod:wisp {} {} {}", f[0] + 3, f[1] + 1, f[2] - 5))
        .await?;
    t.check(t.count_near("wisp", 8.0).await? == 1, "a wisp floats nearby");
    t.op("put me 0 testmod:rune 1").await?;
    t.c().select_slot(0);
    t.c().set_look(0.0, -5.0);
    t.settled().await?;
    t.secs(1.0).await?;
    let night = t.shot("night_glow").await?;
    let veins = match t.block_on_screen(crystal, [0.0, 0.0, 0.51]) {
        Some((x, y)) => night.share(x - 0.12, y - 0.15, x + 0.12, y + 0.15, |c| probe::cyan(c) && probe::lum(c) > 100.0),
        None => 0.0,
    };
    t.probe("the crystal's veins glow", veins, veins > 0.03);
    let floor = match t.on_screen([f[0] as f64 + 0.5, f[1] as f64, f[2] as f64 - 2.5]) {
        Some((x, y)) => crate::runner::lum(night.avg(x - 0.02, y - 0.02, x + 0.02, y + 0.02)),
        None => 255.0,
    };
    t.probe("the floor in front of it stays dark (glow is not light)", floor, floor < 60.0);
    let bands = match t.block_on_screen(striped, [0.0, 0.0, 0.51]) {
        Some((x, y)) => night.share(x - 0.03, y - 0.05, x + 0.03, y + 0.05, |c| c[0] > 100 && c[2] > 100 && c[1] < 60),
        None => 0.0,
    };
    t.probe("the block shader's magenta bands", bands, bands > 0.2 && bands < 0.9);
    let before = t.grab().await?;
    t.check(t.c().post_effect(&Shader::named("testmod:dream"), 1.0), "the daydream effect exists");
    t.check(
        !t.c()
            .post_effect(&Shader::named("testmod:nonexistent"), 1.0),
        "an unknown effect is refused",
    );
    t.frames(3).await?;
    t.shot("dream").await?;
    let after = t.grab().await?;
    let d = T::differ(&before, &after) as f32;
    t.probe("the daydream effect changes the picture", d, d > 0.03);
    t.c().post_effect(&Shader::named("testmod:dream"), 0.0);
    Ok(())
}

/// Models: the orb (built in code, a sphere mesh and a child bone) turns by its file's idle
/// animation; the idol (a Blockbench file with bones and mesh wings) bows when used, a server
/// triggered animation that holds its last frame.
pub async fn models(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.op(&format!("spawn testmod:orb {} {} {}", f[0], f[1] + 1, f[2] - 3))
        .await?;
    let idol = t
        .op(&format!("spawn testmod:idol {} {} {}", f[0], f[1], f[2] + 3))
        .await?;
    t.check(t.count_near("orb", 8.0).await? == 1 && t.count_near("idol", 8.0).await? == 1, "an orb and an idol stand nearby");
    t.c().set_look(0.0, 0.0);
    t.settled().await?;
    t.secs(0.5).await?;
    let a = t.grab().await?;
    t.secs(0.5).await?;
    let b = t.grab().await?;
    t.check(T::differ(&a, &b) > 0.001, "the orb's ring turns by its keyframes");
    t.look_at(add(f, [0, 0, 3]));
    t.frames(3).await?;
    t.press("core:use");
    t.secs(1.5).await?;
    // Below the horizon: clouds drift up there.
    let ground = |f: &[u8]| f[f.len() / 4 / 3 * 4..].to_vec();
    // Two grabs 0.3 s apart the same (the idle, a 3 s loop, never is). The bow starts a round
    // trip after the press: on CI that was over a second, and 1.5 s on it still moved.
    // Near nothing rather than nothing: under load (--jobs 2) no two grabs in 8 s were the same.
    let (mut least, mut a) = (f64::MAX, t.grab().await?);
    let end = crate::runner::script_clock() + Duration::from_secs(8);
    while least > 0.0001 && crate::runner::script_clock() < end {
        t.secs(0.3).await?;
        let b = t.grab().await?;
        least = least.min(T::differ(&ground(&a), &ground(&b)));
        a = b;
    }
    t.check(least <= 0.0001, format!("the bow holds its last frame: {least:.5}"));
    t.shot("idol_bow").await?;
    // A mod has it play one action as another for a while.
    t.op(&format!("action {idol}")).await?;
    t.secs(0.2).await?;
    let plain = t.grab().await?;
    t.op(&format!("swap {idol} idle bow")).await?;
    t.secs(1.0).await?;
    let swapped = t.grab().await?;
    t.check(T::differ(&ground(&plain), &ground(&swapped)) > 0.0005, "idle plays as the bow");
    t.op(&format!("swap {idol} idle")).await?;
    // The test mod gives the player a wave, seen from the first person like from anywhere.
    t.c().set_look(90.0, 0.0);
    t.frames(3).await?;
    let a = t.grab().await?;
    t.op("action me wave").await?;
    // The most it moved over two seconds: the wave starts a round trip after the op, and one grab
    // half a second on missed it on a loaded PC (--jobs 2).
    let (mut moved, end) = (0.0f64, crate::runner::script_clock() + Duration::from_secs(2));
    while moved <= 0.002 && crate::runner::script_clock() < end {
        t.secs(0.1).await?;
        moved = moved.max(T::differ(&a, &t.grab().await?));
    }
    t.check(moved > 0.002, format!("the first person hand waves: {moved:.4}"));
    Ok(())
}

/// Tooltip parts behind `{on_key}` and `{if}` appear while their condition holds: a hurt player
/// sees the rune's red line, sneaking swaps "more" for its long text.
pub async fn rich_text(t: T) -> R {
    t.flat().await?;
    t.op("put me 0 testmod:rune 1").await?;
    // A fall hurts the player, which the test mod's `{if:testmod:hurt}` shows.
    let s = t.state();
    t.c().teleport([s.pos.x, s.pos.y + 6.0, s.pos.z]);
    t.until("hurt by the fall", Duration::from_secs(5), |t| t.state().health < super::max_health(t) - 0.5)
        .await?;
    t.press("core:inventory");
    t.screen("core:menu").await?;
    let Some(Rect { x, y, w, h }) = t.c().ui_rect(&t.hotbar(0)) else { return Err("no hotbar slot in the inventory".into()) };
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(4).await?;
    let rune = t.shot("rune").await?;
    let wound = |c: [u8; 3]| c[0] > 200 && (100..170).contains(&c[1]) && (100..170).contains(&c[2]);
    // The tooltip opens beside the cursor.
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    let v = rune.share(cx, cy - 0.15, cx + 0.3, cy + 0.15, wound);
    t.probe("the {if:testmod:hurt} line (light red)", v, v > 0.003);
    t.hold("core:sneak", 2.0);
    t.frames(4).await?;
    let sneak = t.shot("rune_sneak").await?;
    let d = T::differ(&rune.px, &sneak.px) as f32;
    t.probe("sneaking changes the tooltip ({on_key})", d, d > 0.002);
    t.c().input_release_all();
    t.c().menu_close();
    t.no_screen().await
}

/// The sky: the sun at noon and the moon at midnight stand overhead, stars show only at night, a
/// sunset warms the horizon on its side, and from inside water everything is its fog.
pub async fn sky(t: T) -> R {
    t.world(5).await?;
    t.op("mode creative").await?;
    // A clear sky: no rain clouds, no mist (base's `/air`) over the sunset.
    t.cmd("/weather clear").await?;
    t.cmd("/air 0").await?;
    // No clouds: one over the sun or the sunset hides what is checked (a cloud fog ending at 0).
    t.c().environment(
        &Environment::new("test_sky")
            .priority(100)
            .cloud_fog(0.0, 0.0),
    );
    // Base's sun and moon go round a path leaned 35° to the south (`mods/base/src/lib.rs`): noon
    // stands 55° up, not overhead. The look toward a body `turn` round that path (0 rising in the
    // east, a quarter at its height); yaw 0 faces -z (north).
    let toward = |turn: f64| {
        let (a, lean) = (turn * std::f64::consts::TAU, 35f64.to_radians());
        let (x, y, z) = (a.cos(), a.sin() * lean.cos(), a.sin() * lean.sin());
        ((-x).atan2(-z).to_degrees() as f32, y.asin().to_degrees() as f32)
    };
    let (yaw, pitch) = toward(0.25);
    t.c().set_look(yaw, pitch);
    // Without the GUI: a popup's letters (a level up) would count as stars.
    t.press("core:hide_hud");
    t.frames(4).await?;
    let bright = |c: probe::C| c.iter().all(|&v| v > 215);
    t.op("time 6000").await?;
    t.secs(0.6).await?;
    let noon = t.shot("noon").await?;
    t.probe("the sun at its height", noon.share(0.45, 0.40, 0.55, 0.57, bright), noon.share(0.45, 0.40, 0.55, 0.57, bright) > 0.5);
    // A star is a speck: bright with sky a little way off on every side, which a cloud isn't.
    let specks = |shot: &crate::runner::Shot| {
        let (sx, sy) = (1.0 / 1600.0, 1.0 / 900.0);
        let mut n = 0;
        for y in (10..225).map(|y| y as f32 * sy * 1.0) {
            for x in (10..1590).map(|x| x as f32 * sx) {
                let lit = |x: f32, y: f32| shot.at(x, y).iter().all(|&v| v > 215);
                if lit(x, y) && !lit(x - 6.0 * sx, y) && !lit(x + 6.0 * sx, y) && !lit(x, y - 6.0 * sy) && !lit(x, y + 6.0 * sy) {
                    n += 1;
                }
            }
        }
        n
    };
    t.check_eq(specks(&noon), 0, "no stars by day");
    // The moon lags the day: a turn every 8/7 days, full (opposite the sun) on day 0. At day 0's
    // midnight it is (18000 / 27429 + 0.5) round: a little past rising, high in the south-east.
    t.op("time 18000").await?;
    let (yaw, pitch) = toward((18000.0 / (24000.0 * 8.0 / 7.0) + 0.5f64).rem_euclid(1.0));
    t.c().set_look(yaw, pitch);
    t.secs(0.6).await?;
    let night = t.shot("midnight").await?;
    let moon = night.share(0.45, 0.42, 0.55, 0.56, |c| c.iter().all(|&v| v > 150));
    // Our moon (`moon_phases`) has dark seas: about a third of it is bright.
    t.probe("the moon where its path puts it", moon, moon > 0.2);
    let stars = night.share(0.0, 0.0, 1.0, 0.3, |c| c.iter().all(|&v| v > 60));
    t.probe("stars at night", stars, stars > 0.0005);
    // Looking at the sunset and away from it, at the equinox: the day's length follows the
    // seasons, and a new world starts in late winter, when the sun has set long before 12400.
    // `time` keeps the day, so this is day 7 at 12100, just as its 12 hour day ends.
    // Our sun still stood 15° up at 12100 (Tester 2, 11-24): on toward its setting, a quarter
    // hour at a time, until the west warms.
    let warm = |c: probe::C| c[0] > 170 && c[0] as i32 > c[2] as i32 + 50;
    t.c().set_look(90.0, 8.0);
    let mut west = None;
    for step in 0..12 {
        t.op(&format!("time {}", 180100 + step * 250)).await?;
        t.secs(0.6).await?;
        let shot = t.shot("sunset").await?;
        let found = shot.share(0.0, 0.3, 1.0, 0.5, warm) > 0.05;
        west = Some(shot);
        if found {
            break;
        }
    }
    let west = west.ok_or("no sunset shot")?;
    t.c().set_look(-90.0, 8.0);
    t.secs(0.6).await?;
    let east = t.shot("sunset_back").await?;
    let (w, e) = (west.share(0.0, 0.3, 1.0, 0.5, warm), east.share(0.0, 0.3, 1.0, 0.5, warm));
    t.note(format!("warm sky: toward the sunset {w:.3}, away {e:.3}"));
    t.check(w > e + 0.02, format!("a sunset on its side: {w:.3}, away {e:.3}"));
    t.c().clear_environment("test_sky");
    t.press("core:hide_hud");
    Ok(())
}

/// Clouds reach far past any grid around the camera, cover what is under them, stay behind what is
/// nearer, and the setting lifts them.
pub async fn clouds(t: T) -> R {
    static LIFT: Config<i32> = Config::named("core:cloud_height");
    t.world(5).await?;
    t.op("mode spectator").await?;
    t.op("time 6000").await?;
    t.cmd("/weather clear").await?;
    LIFT.set(0);
    let white = |c: probe::C| c.iter().all(|&v| v > 205);
    let s = t.state();
    // Near the horizon from well below them: what is there is thousands of blocks away.
    t.c().teleport([s.pos.x, 150.0, s.pos.z]);
    t.c().set_look(0.0, 2.0);
    t.secs(1.0).await?;
    let (x0, y0, x1, y1) = (0.0, 0.47, 1.0, 0.53);
    let far = t.shot("far").await?.share(x0, y0, x1, y1, white);
    let world_fog = t.c().environment_values().fog_end;
    settings::CLOUD_DISTANCE.set(64);
    t.frames(3).await?;
    let limited = t
        .shot("distance_limited")
        .await?
        .share(x0, y0, x1, y1, white);
    t.probe("cloud distance removes distant clouds", limited, limited < far - 0.02);
    t.check_eq(t.c().environment_values().fog_end, world_fog, "cloud distance leaves world fog independent");
    t.c().environment(
        &Environment::new("test_cloud_range")
            .priority(100)
            .cloud_fog(3072.0, 6144.0),
    );
    t.frames(3).await?;
    let overridden = t
        .shot("distance_override")
        .await?
        .share(x0, y0, x1, y1, white);
    t.probe("environment source overrides the cloud distance setting", overridden, overridden > limited + 0.02);
    t.c().clear_environment("test_cloud_range");
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().cloud_fog_end, 64.0, "removing the source restores the slider value");
    settings::CLOUD_DISTANCE.set(6144);
    // Lifted out of reach at that angle.
    LIFT.set(512);
    t.secs(0.3).await?;
    let lifted = t.shot("far_lifted").await?.share(x0, y0, x1, y1, white);
    t.note(format!("white near the horizon: {far:.3}, lifted {lifted:.3}"));
    t.check(far > lifted + 0.02, format!("clouds far away: {far:.3}, lifted out of reach {lifted:.3}"));
    // From above they cover the ground; lifted over the camera they don't.
    LIFT.set(0);
    t.c().teleport([s.pos.x, 330.0, s.pos.z]);
    t.c().set_look(0.0, -60.0);
    t.secs(1.0).await?;
    let above = t.shot("above").await?.share(0.0, 0.3, 1.0, 1.0, white);
    settings::CLOUD_DISTANCE.set(64);
    t.frames(3).await?;
    let culled = t
        .shot("layer_culled")
        .await?
        .share(0.0, 0.3, 1.0, 1.0, white);
    t.probe("a layer beyond the cloud fog sphere is culled", culled, culled < above - 0.02);
    settings::CLOUD_DISTANCE.set(6144);
    LIFT.set(512);
    t.secs(0.3).await?;
    let lifted = t
        .shot("above_lifted")
        .await?
        .share(0.0, 0.3, 1.0, 1.0, white);
    t.note(format!("white from above: {above:.3}, lifted {lifted:.3}"));
    t.check(above > lifted + 0.02, format!("over the ground: {above:.3}, lifted away {lifted:.3}"));
    // A wall nearer than they are hides them.
    LIFT.set(0);
    let at = [s.pos.x.floor() as i32, 150, s.pos.z.floor() as i32];
    t.fill(add(at, [-30, 0, -12]), add(at, [30, 70, -12]), "stone")
        .await?;
    t.c().teleport([s.pos.x, 150.0, s.pos.z]);
    t.c().set_look(0.0, 35.0);
    t.secs(1.0).await?;
    let wall = t.shot("wall").await?.share(0.2, 0.3, 0.8, 0.7, white);
    t.check(wall < 0.005, format!("clouds through a wall: {wall:.3}"));
    // Weather supplies cloud appearance to the same environment as external sources.
    // Seed 5 has a cloud in this cell. Frame its interior so sky gaps don't dominate the tint probes.
    t.c().teleport([540.0, 195.0, 620.0]);
    t.c().set_look(0.0, 89.0);
    t.secs(0.4).await?;
    let sky_lum = |shot: &crate::runner::Shot| crate::runner::lum(shot.avg(0.1, 0.05, 0.9, 0.8));
    let clear = sky_lum(&t.shot("clear_clouds").await?);
    t.c().say("/weather rain");
    t.until("rain cloud tint", Duration::from_secs(8), |t| t.c().environment_values().cloud_gray > 0.9)
        .await?;
    let rain = sky_lum(&t.shot("rain_clouds").await?);
    t.c().say("/weather thunder");
    t.until("thunder cloud tint", Duration::from_secs(8), |t| t.c().environment_values().cloud_dark > 0.9)
        .await?;
    let thunder = sky_lum(&t.shot("thunder_clouds").await?);
    t.probe("rain grays clouds", clear - rain, clear > rain + 15.0);
    t.probe("thunder darkens clouds", rain - thunder, rain > thunder + 15.0);
    t.c().environment(
        &Environment::new("test_clouds")
            .priority(100)
            .cloud_color(0xE03020)
            .fade(0.3, 0.3),
    );
    t.secs(0.4).await?;
    let red = t.shot("override_clouds").await?;
    let red_clouds = red.share(0.05, 0.05, 0.95, 0.8, probe::red);
    t.probe("cloud tint can replace thunder independently", red_clouds, red_clouds > 0.05);
    let values = t.c().environment_values();
    t.check(
        values.cloud_gray == 0.0 && values.cloud_dark == 0.0 && values.fog_gray > 0.1,
        "cloud override leaves other weather contributions intact",
    );
    t.c().clear_environment("test_clouds");
    t.secs(0.4).await?;
    let restored = sky_lum(&t.shot("restored_clouds").await?);
    t.probe("removing the source restores weather tint", (restored - thunder).abs(), (restored - thunder).abs() < 10.0);
    // Both cloud sliders use the normal video settings page.
    t.c().menu_open(&Menu::named("core:video"));
    t.screen("core:video").await?;
    t.frames(3).await?;
    t.laid_out(&["text:Clouds elevation", "text:Cloud render distance"]);
    t.shot("settings").await?;
    t.c().menu_close();
    t.no_screen().await?;
    Ok(())
}

/// The attack indicator is a HUD element like the others: under the crosshair while a swing
/// charges, filling, gone when charged, and the crosshair stays in the middle of the window.
pub async fn attack_indicator(t: T) -> R {
    t.flat().await?;
    t.cmd("/gear sword wrought_iron slot=0").await?;
    t.c().select_slot(0);
    // A swing that hits nothing spends the charge; looking down, nothing bright is behind the bar.
    t.op("attr_mod me core:block_reach no_reach core:add -10 0")
        .await?;
    t.c().set_look(0.0, -40.0);
    t.secs(1.0).await?;
    let (x0, y0, x1, y1) = (0.49, 0.505, 0.51, 0.54);
    // The bar's fill is the kit's amber (warm and bright), not white: red high, green middling,
    // blue low; the ground behind it (grass, dirt) is none of that.
    let fill = |c: probe::C| c[0] > 170 && c[1] > 80 && c[2] < 110;
    let rest = t.shot("charged").await?;
    t.check(rest.share(x0, y0, x1, y1, fill) < 0.01, "nothing under the crosshair when charged");
    let center = rest.share(0.497, 0.494, 0.503, 0.506, |c| c.iter().all(|&v| v > 150));
    t.probe("the crosshair in the middle of the window", center, center > 0.1);
    t.press("core:attack");
    t.secs(0.15).await?;
    let early = t.shot("charging").await?.share(x0, y0, x1, y1, fill);
    t.secs(0.2).await?;
    let later = t.shot("charging_later").await?.share(x0, y0, x1, y1, fill);
    t.note(format!("indicator filled: {early:.3}, later {later:.3}"));
    t.check(later > 0.02, format!("the indicator shows while charging: {later:.3}"));
    t.check(later > early, format!("and fills: {early:.3}, then {later:.3}"));
    t.secs(0.6).await?;
    t.check(t.shot("charged_again").await?.share(x0, y0, x1, y1, fill) < 0.01, "and is gone once charged");
    t.op("attr_rm me core:block_reach no_reach").await?;
    Ok(())
}

/// Where the ground starts on two columns of the picture; a tilted view has them differ.
fn tilt(shot: &crate::runner::Shot) -> f32 {
    let horizon = |x: f32| {
        (0..450)
            .map(|i| 0.1 + i as f32 / 450.0 * 0.8)
            .find(|&y| probe::grass(shot.at(x, y)))
            .unwrap_or(0.9)
    };
    horizon(0.1) - horizon(0.9)
}

/// Third person shakes like first: from a hit, and from walking.
pub async fn third_person_shakes(t: T) -> R {
    t.flat().await?;
    t.c().set_look(0.0, 0.0);
    t.press("core:perspective");
    t.secs(0.5).await?;
    let still = tilt(&t.shot("still").await?);
    t.check(still.abs() < 0.003, format!("level standing still: {still}"));
    t.hold("core:forward", 2.0);
    t.secs(0.3).await?;
    let mut range = (f32::MAX, f32::MIN);
    for i in 0..10 {
        let v = tilt(&t.shot(&format!("walking{i}")).await?);
        range = (range.0.min(v), range.1.max(v));
    }
    t.note(format!("tilt walking: {:.4} to {:.4}", range.0, range.1));
    t.check(range.1 - range.0 > 0.001, format!("walking sways the view: {range:?}"));
    t.secs(1.0).await?;
    let s = t.state().pos;
    t.c().teleport([s.x, s.y + 8.0, s.z]);
    t.until("hurt by the fall", Duration::from_secs(5), |t| t.state().health < super::max_health(t) - 0.5)
        .await?;
    let mut most = 0.0f32;
    for i in 0..4 {
        most = most.max(tilt(&t.shot(&format!("hurt{i}")).await?).abs());
    }
    t.note(format!("tilt when hit: {most:.4}"));
    t.check(most > 0.01, format!("a hit jolts the view: {most}"));
    t.press("core:perspective");
    t.press("core:perspective");
    Ok(())
}

/// A broken block's pieces fly out to the sides, past where it stood, also when the client hears
/// of the dust before the block is gone.
pub async fn block_dust_scatters(t: T) -> R {
    t.flat().await?;
    let b = add(t.feet(), [3, 3, 0]);
    t.set(b, "stone").await?;
    t.until("the block on the client", Duration::from_secs(2), |t| t.c().block(b).is_some_and(|s| s != 0))
        .await?;
    t.look_at(b);
    t.secs(0.3).await?;
    // Darker than the sky and any cloud.
    let piece = |c: probe::C| c.iter().all(|&v| v < 170);
    let sides = |s: &crate::runner::Shot| [s.share(0.30, 0.3, 0.42, 0.7, piece), s.share(0.58, 0.3, 0.70, 0.7, piece)];
    let before = sides(&t.shot("before").await?);
    t.op(&format!("break {} {} {}", b[0], b[1], b[2])).await?;
    t.until("the block gone on the client", Duration::from_secs(2), |t| t.c().block(b) == Some(0))
        .await?;
    // The most seen each frame for a quarter second: on CI a frame or two outlived the pieces
    // (half a second to a second), and one shot 0.25 s on found none.
    let end = crate::runner::script_clock() + Duration::from_secs_f32(0.25);
    let mut after = [0f32; 2];
    let shot = loop {
        let shot = t.shot("after").await?;
        let s = sides(&shot);
        after = [after[0].max(s[0]), after[1].max(s[1])];
        if crate::runner::script_clock() >= end {
            break shot;
        }
    };
    t.note(format!("pieces at the sides: {before:?} then {after:?}"));
    t.check(after[0] > before[0] + 0.0005, format!("pieces on the left: {before:?} then {after:?}"));
    t.check(after[1] > before[1] + 0.0005, format!("pieces on the right: {before:?} then {after:?}"));
    // None stay black from having started where the block was, with no light in it.
    let black = shot.share(0.3, 0.2, 0.7, 0.8, |c| c.iter().all(|&v| v < 30));
    t.check(black < 0.0002, format!("black pieces: {black}"));
    Ok(())
}

/// Translucent blocks show far to near inside a cube as well, from any side: the same two blocks
/// look the same within one cube as with a cube's edge between them, where the cube order sees to it.
pub async fn translucent_order(t: T) -> R {
    t.flat().await?;
    t.op("time 6000").await?;
    t.op("mode spectator").await?;
    let f = t.feet();
    let (bx, y, bz) = (f[0].div_euclid(16) * 16, f[1] + 3, f[2].div_euclid(16) * 16);
    for side in [1, -1] {
        let mut seen = Vec::new();
        for (name, low) in [("one_cube", 6), ("two_cubes", 15)] {
            let (a, b) = ([bx + low, y, bz + low], [bx + low + 1, y, bz + low + 1]);
            t.set(a, "base:slime_block").await?;
            t.set(b, "base:slime_block").await?;
            let near = if side > 0 { b } else { a };
            let middle = [(bx + low) as f64 + 1.0, y as f64 + 0.5, (bz + low) as f64 + 1.0];
            let s = side as f64;
            t.c()
                .teleport([middle[0] + s * 3.0, middle[1] + 0.5, middle[2] + s * 2.0]);
            t.frames(10).await?;
            t.look_at(near);
            t.secs(0.4).await?;
            let shot = t
                .shot(&format!("{name}_{}", if side > 0 { "east" } else { "west" }))
                .await?;
            // The near block's top, with the far one's side behind it.
            let on = t
                .on_screen([near[0] as f64 + 0.5, y as f64 + 1.0, near[2] as f64 + 0.5])
                .unwrap_or((0.5, 0.5));
            seen.push(shot.avg(on.0 - 0.01, on.1 - 0.01, on.0 + 0.01, on.1 + 0.01));
            t.set(a, "air").await?;
            t.set(b, "air").await?;
            t.frames(3).await?;
        }
        let off = (0..3)
            .map(|i| (seen[0][i] - seen[1][i]).abs())
            .fold(0.0, f32::max);
        t.probe(&format!("how far apart the two are, side {side}"), off, off < 6.0);
    }
    Ok(())
}

/// Light adds up and bends toward full: by day a lamp's color still shows on what is around it,
/// over the sun's.
pub async fn light_by_day(t: T) -> R {
    t.flat().await?;
    t.op("time 6000").await?;
    t.cmd("/weather clear").await?;
    let f = t.feet();
    t.op("mode spectator").await?;
    t.fill(add(f, [-9, 0, -8]), add(f, [9, 0, -2]), "base:snow")
        .await?;
    t.c()
        .teleport([f[0] as f64 + 0.5, f[1] as f64 + 7.0, f[2] as f64 - 2.0]);
    t.c().set_look(0.0, -60.0);
    // No HUD: on CI the test runs past base's first hint, whose box covers the probed snow.
    t.press("core:hide_hud");
    t.secs(0.4).await?;
    let spot = t
        .on_screen([f[0] as f64 + 0.5, f[1] as f64 + 1.0, f[2] as f64 - 5.5])
        .ok_or("not in view")?;
    let at = |s: &crate::runner::Shot| s.avg(spot.0 - 0.01, spot.1 - 0.01, spot.0 + 0.01, spot.1 + 0.01);
    let plain = at(&t.shot("plain").await?);
    t.set(add(f, [0, 1, -5]), "base:blue_lamp").await?;
    t.frames(4).await?;
    // Beside the lamp, on the snow.
    let spot2 = t
        .on_screen([f[0] as f64 + 1.5, f[1] as f64 + 1.0, f[2] as f64 - 4.5])
        .ok_or("not in view")?;
    let beside = |s: &crate::runner::Shot| s.avg(spot2.0 - 0.01, spot2.1 - 0.01, spot2.0 + 0.01, spot2.1 + 0.01);
    t.secs(0.4).await?;
    let lit = beside(&t.shot("lamp").await?);
    t.note(format!("snow {plain:?}, beside a blue lamp {lit:?}"));
    t.probe("how much bluer the snow is than red, by a blue lamp at noon", lit[2] - lit[0], lit[2] - lit[0] > 12.0);
    t.press("core:hide_hud");
    Ok(())
}

/// Any method on any block, here plants and box models connecting up and down, and a tile that
/// is a model file with its picture an unwrap, on a plain cube.
pub async fn ctm_any_model(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-9, 0, -7]), add(f, [9, 6, -7]), "snow")
        .await?;
    t.fill(add(f, [-9, 0, 7]), add(f, [9, 6, 7]), "snow")
        .await?;
    for (x, block) in [(-4, "testkit:varied_plant"), (1, "testkit:varied_box")] {
        for y in 0..3 {
            t.set(add(f, [x, y, -5]), block).await?;
        }
        t.set(add(f, [x + 2, 0, -5]), block).await?;
    }
    // A pack's model of a tinted block, its own textures on it: only what the block tints is.
    let grass = add(f, [-1, 0, -5]);
    t.set(grass, "base:grass").await?;
    let row: Vec<i32> = (-5..=5).collect();
    for &x in &row {
        t.set(add(f, [x, 0, 4]), "testkit:varied_cube").await?;
    }
    let pack = format!("{}/resourcepacks/testkit_ctm", t.c().run_dir());
    let write = |file: &str, text: &str| -> R {
        let path = format!("{pack}/testkit/{file}");
        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| e.to_string())
    };
    let whole = |f: &str| format!(r#"{{"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{{f}}}}}]}}"#);
    let base_model = format!("{pack}/base/models/block/grass.json");
    std::fs::create_dir_all(std::path::Path::new(&base_model).parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&base_model, whole(r#""north":{},"south":{},"east":{},"west":{},"up":{},"down":{}"#)).map_err(|e| e.to_string())?;
    // A texture's rules sit in its own folder (`textures/<name>/<last>.ctm`, Weft 310cf6d88),
    // and a tile without a namespace is in that folder, so these name theirs in full. This test
    // still wrote the old flat path and short names there: a Weft bug.
    write(
        "textures/block/varied/varied.ctm",
        "method = vertical\ntiles = testkit:block/colors/red testkit:block/colors/green testkit:block/colors/blue testkit:block/colors/yellow\n",
    )?;
    write(
        "textures/block/varied_cube/varied_cube.ctm",
        "method = random\ntiles = testkit:block/shapes/thin <default>\nhides_neighbors = false\n",
    )?;
    let face = r##"{"texture":"#unwrap","uv":[0.25,0.25,1.25,1.25]}"##;
    write(
        "models/block/shapes/thin.json",
        &format!(
            r##"{{"textures":{{"unwrap":"block/unwrap"}},"elements":[{{"from":[6,0,6],"to":[10,16,10],"faces":{{"north":{face},"south":{face},"east":{face},"west":{face},"up":{face},"down":{face}}}}}]}}"##
        ),
    )?;
    weft_sdk::settings::RESOURCE_PACKS.set("testkit_ctm");
    t.c().reload_assets();
    t.frames(5).await?;
    let out = async {
        t.look_at(add(f, [0, 1, -5]));
        t.settled().await?;
        let s = t.shot("connected").await?;
        let at = |s: &crate::runner::Shot, p: [f64; 3], test: fn(probe::C) -> bool| {
            t.on_screen(p)
                .map_or(0.0, |(x, y)| s.share(x - 0.004, y - 0.004, x + 0.004, y + 0.004, test))
        };
        let red: fn(probe::C) -> bool = |c| c[0] > 120 && c[1] < 70 && c[2] < 70;
        let green: fn(probe::C) -> bool = |c| c[1] > 120 && c[0] < 70 && c[2] < 70;
        let blue: fn(probe::C) -> bool = |c| c[2] > 120 && c[0] < 70 && c[1] < 90;
        let yellow: fn(probe::C) -> bool = |c| c[0] > 120 && c[1] > 110 && c[2] < 70;
        for (x, depth, what) in [(-4, 0.5, "plant"), (1, 0.13, "box model")] {
            let o = |x: i32, y: i32| [(f[0] + x) as f64 + 0.5, (f[1] + y) as f64 + 0.5, (f[2] - 5) as f64 + 1.0 - depth];
            t.check(at(&s, o(x, 0), red) > 0.5, format!("{what}: red at the bottom"));
            t.check(at(&s, o(x, 1), green) > 0.5, format!("{what}: green in the middle"));
            t.check(at(&s, o(x, 2), blue) > 0.5, format!("{what}: blue on top"));
            t.check(at(&s, o(x + 2, 0), yellow) > 0.5, format!("{what}: yellow alone"));
        }

        let side = |h: f64| [grass[0] as f64 + 0.5, grass[1] as f64 + h, grass[2] as f64 + 1.0];
        let brown: fn(probe::C) -> bool = |c| c[0] > c[1] && c[1] > c[2];
        let greenish: fn(probe::C) -> bool = |c| c[1] > c[0] && c[1] > c[2];
        t.check(at(&s, side(0.3), brown) > 0.5, "a model's dirt side isn't tinted");
        t.check(at(&s, side(0.97), greenish) > 0.3, "and it has its tinted overlay");

        t.look_at(add(f, [0, 0, 4]));
        t.settled().await?;
        let s = t.shot("models").await?;
        let (mut thin, mut plain, mut stray, mut ground, mut dark) = (0, 0, 0, 0, 0);
        let orange: fn(probe::C) -> bool = |c| c[0] > 120 && c[1] > 50 && c[1] < 130 && c[2] < 60;
        let grass: fn(probe::C) -> bool = |c| c[1] > c[0] && c[1] > c[2] && c[1] > 40;
        for &x in &row {
            let middle = [(f[0] + x) as f64 + 0.5, f[1] as f64 + 0.5, (f[2] + 4) as f64 + 0.375];
            if at(&s, middle, blue) > 0.5 {
                thin += 1;
                // All of a face is the island, right up to its corners (further out a cube may
                // stand in front of it).
                for (a, b) in [(0.41, 0.08), (0.59, 0.08), (0.41, 0.92), (0.59, 0.92)]
                    .into_iter()
                    .filter(|_| x.abs() <= 3)
                {
                    if at(&s, [(f[0] + x) as f64 + a, f[1] as f64 + b, (f[2] + 4) as f64 + 0.375], blue) < 0.5 {
                        stray += 1;
                    }
                }
                // A side is lit, though the block's own cell and maybe the next one have no light.
                if x != 0 {
                    let side = if x < 0 { 0.625 } else { 0.375 };
                    if t.on_screen([(f[0] + x) as f64 + side, f[1] as f64 + 0.5, (f[2] + 4) as f64 + 0.5])
                        .is_some_and(|(sx, sy)| crate::runner::lum(s.avg(sx - 0.002, sy - 0.004, sx + 0.002, sy + 0.004)) < 25.0)
                    {
                        dark += 1;
                    }
                }
                // Beside the column, the floor its cube no longer hides.
                let side = if x <= 0 { 0.12 } else { 0.88 };
                if at(&s, [(f[0] + x) as f64 + side, f[1] as f64 + 0.01, (f[2] + 4) as f64 + 0.5], grass) > 0.5 {
                    ground += 1;
                }
            } else if at(&s, middle, orange) > 0.5 {
                plain += 1;
            } else if at(&s, middle, red) > 0.1 {
                stray += 1;
            }
        }
        t.note(format!("thin {thin}, plain {plain}, off the island {stray}, floor seen by {ground}, dark sides {dark}"));
        t.check(dark == 0, format!("a model's sides are lit: {dark} are dark"));
        t.check(thin > 0 && plain > 0, format!("the rule picks the model and the plain cube: {thin} and {plain}"));
        t.check(stray == 0, format!("the model's texels are the island on its unwrap: {stray} show around it"));
        t.check(ground > 0, format!("hides_neighbors = false: the floor shows beside a model: {ground} of {thin}"));
        Ok(())
    }
    .await;
    weft_sdk::settings::RESOURCE_PACKS.set("");
    let _ = std::fs::remove_dir_all(&pack);
    t.c().reload_assets();
    t.frames(2).await?;
    out
}

// Restored from dev b5815f09 (Weft v26.11.2 removed them): see notes/crew/upstream/lost-tests.md

/// Texture edges at a distance, where mip levels mix texels: no light line along the edge of a
/// snowy grass block over dirt; a 64x64 texture shows its own colors among 16x16 ones.
pub async fn texture_seams(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-8, 0, -12]), add(f, [8, 0, -12]), "dirt")
        .await?;
    t.fill(add(f, [-8, 1, -12]), add(f, [8, 1, -12]), "snowy_grass")
        .await?;
    t.fill(add(f, [-8, 2, -12]), add(f, [8, 2, -12]), "dirt")
        .await?;
    t.fill(add(f, [-8, 3, -12]), add(f, [8, 3, -12]), "grass")
        .await?;
    t.fill(add(f, [-8, 0, -9]), add(f, [8, 0, -9]), "tall_grass")
        .await?;
    // 64x64 next to 16x16 textures.
    let mosaic = add(f, [1, 0, -5]);
    t.fill(mosaic, add(mosaic, [1, 1, 0]), "testmod:mosaic")
        .await?;
    t.c().set_look(35.0, 8.0);
    t.settled().await?;
    let shot = t.shot("seams").await?;
    // Just below and above the edge between the dirt and the snowy grass on top of it, on the
    // face the camera sees.
    let mut light = 0;
    let mut seen = 0;
    for x in -7..=7 {
        for dy in [0.96, 1.04] {
            if let Some((sx, sy)) = t.on_screen([(f[0] + x) as f64 + 0.5, f[1] as f64 + dy, (f[2] - 11) as f64 + 0.01]) {
                if (0.0..1.0).contains(&sx) && (0.0..1.0).contains(&sy) {
                    seen += 1;
                    light += (probe::lum(shot.at(sx, sy)) > 170.0) as u32;
                }
            }
        }
    }
    t.check(seen > 6, format!("the wall's edge is in view ({seen} points)"));
    t.probe("light points along the snowy grass edge", light as f32, light == 0);
    // The middle of the 2x2 mosaic face the camera sees.
    let p = t
        .on_screen([(mosaic[0] + 1) as f64, (mosaic[1] + 1) as f64, mosaic[2] as f64 + 1.01])
        .unwrap_or((0.5, 0.5));
    let colors = shot.share(p.0 - 0.03, p.1 - 0.05, p.0 + 0.03, p.1 + 0.05, |c| {
        let (hi, lo) = (*c.iter().max().unwrap() as i32, *c.iter().min().unwrap() as i32);
        hi - lo > 80
    });
    t.probe("the mosaic's saturated colors", colors, colors > 0.3);
    Ok(())
}

/// A random `.ctm` varies plants and box models too: red and blue tiles of a green texture, each
/// row in front of a white wall.
pub async fn ctm_off_the_cube(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-9, 0, -7]), add(f, [9, 5, -7]), "snow")
        .await?;
    for x in -5..=5 {
        t.set(add(f, [x, 0, -5]), "testkit:varied_plant").await?;
        t.set(add(f, [x, 3, -5]), "testkit:varied_box").await?;
    }
    t.look_at(add(f, [0, 1, -5]));
    t.secs(0.5).await?;
    let s = t.shot("rows").await?;
    let row = |p: [f64; 3]| t.on_screen(p).map_or(0.5, |at| at.1);
    let (plants, boxes) =
        (row([f[0] as f64, f[1] as f64 + 0.5, f[2] as f64 - 4.5]), row([f[0] as f64, f[1] as f64 + 3.5, f[2] as f64 - 4.5]));
    let colors = |y: f32| [s.share(0.1, y - 0.03, 0.9, y + 0.03, probe::red), s.share(0.1, y - 0.03, 0.9, y + 0.03, probe::blue)];
    let (p, b) = (colors(plants), colors(boxes));
    t.note(format!("red and blue, plants {p:?}, boxes {b:?}"));
    t.check(p[0] > 0.01 && p[1] > 0.01, format!("plants of both tiles: {p:?}"));
    t.check(b[0] > 0.01 && b[1] > 0.01, format!("boxes of both tiles: {b:?}"));
    Ok(())
}

/// An item shaped by a model file (the test mod's totem: a stone base, a crystal on it) shows its
/// boxes in the hand and in the hotbar slot.
pub async fn item_model_file(t: T) -> R {
    t.flat().await?;
    t.op("put me 0 testmod:totem 1").await?;
    t.c().select_slot(0);
    t.c().set_look(0.0, 20.0);
    t.settled().await?;
    // The hand comes up with it.
    t.secs(0.8).await?;
    let shot = t.shot("totem").await?;
    let crystal = |c: probe::C| probe::cyan(c) || (c[2] > 120 && c[0] < 90);
    let hand = shot.share(0.55, 0.55, 1.0, 0.94, crystal);
    t.probe("the crystal in the hand", hand, hand > 0.01);
    // The first hotbar slot of the HUD, at the bottom middle (gui scale 2, 1600x900).
    let slot = shot.share(0.388, 0.95, 0.412, 0.995, crystal);
    t.probe("the crystal in the hotbar slot", slot, slot > 0.015);
    Ok(())
}

/// A block shaped by a Blockbench Java model file: the test mod's pedestal is a plate, a crystal
/// column and a top, so against the sky the sides of the column show sky, the column itself not.
pub async fn block_model_file(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    let p = add(f, [0, 2, -3]);
    t.set(p, "testmod:pedestal").await?;
    t.op("put me 0 testmod:pedestal 1").await?;
    t.c().set_look(0.0, 15.0);
    t.settled().await?;
    let shot = t.shot("pedestal").await?;
    let at = |x: f64| {
        t.on_screen([p[0] as f64 + x, p[1] as f64 + 0.35, p[2] as f64 + 0.5])
            .map(|(sx, sy)| crate::runner::lum(shot.avg(sx - 0.003, sy - 0.003, sx + 0.003, sy + 0.003)))
    };
    let (gap, column) = (at(0.18).unwrap_or(0.0), at(0.5).unwrap_or(255.0));
    t.probe("sky beside the column (the file's shape, not a cube)", gap, gap > 180.0);
    t.probe("the crystal column itself", column, column < 150.0);
    // A resource pack's model of the same block redraws it, here as a whole cube.
    let pack = format!("{}/resourcepacks/testkit_shapes", t.c().run_dir());
    let file = format!("{pack}/testmod/models/block/pedestal.json");
    std::fs::create_dir_all(std::path::Path::new(&file).parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(
        &file,
        r#"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"north":{},"south":{},"east":{},"west":{},"up":{},"down":{}}}]}"#,
    )
    .map_err(|e| e.to_string())?;
    weft_sdk::settings::RESOURCE_PACKS.set("testkit_shapes");
    t.c().reload_assets();
    t.frames(5).await?;
    t.settled().await?;
    let cube = t.shot("pedestal_pack").await?;
    weft_sdk::settings::RESOURCE_PACKS.set("");
    let _ = std::fs::remove_dir_all(&pack);
    t.c().reload_assets();
    t.frames(2).await?;
    let filled = t
        .on_screen([p[0] as f64 + 0.18, p[1] as f64 + 0.35, p[2] as f64 + 0.5])
        .map(|(sx, sy)| crate::runner::lum(cube.avg(sx - 0.003, sy - 0.003, sx + 0.003, sy + 0.003)))
        .unwrap_or(255.0);
    t.probe("a pack's model draws a cube there", filled, filled < 150.0);
    // Its item with it: in the hotbar's first slot the cube covers more than the pedestal did.
    let covered =
        |s: &crate::runner::Shot| s.share(0.392, 0.955, 0.408, 0.99, |c| c[0].abs_diff(c[1]) > 12 || c[1].abs_diff(c[2]) > 12 || c[0] < 90);
    let (before, after) = (covered(&shot), covered(&cube));
    t.probe("the item takes the pack's model, how much more of its slot is covered", after - before, after > before + 0.02);
    Ok(())
}

/// From high above, where the view's cubes don't reach the ground, it is land all the way under
/// the camera, not a hole of sky.
pub async fn distant_terrain_from_above(t: T) -> R {
    static VIEW: Config<i32> = Config::named("core:view_distance");
    static LOD: Config<i32> = Config::named("core:lod_distance");
    let before = (VIEW.get(), LOD.get());
    VIEW.set(8);
    LOD.set(256);
    let out = async {
        t.world(5).await?;
        t.idle().await?;
        t.op("mode spectator").await?;
        let s = t.state().pos;
        t.c().teleport([s.x, 1024.0, s.z]);
        t.c().set_look(0.0, -89.0);
        t.until("distant terrain made", Duration::from_secs(20), |t| t.state().lod_tiles > 50 && t.state().lod_pending == 0)
            .await
            .map_err(|e| {
                format!("{e} (at y {:.0}: {} tiles shown, {} pending)", t.state().pos.y, t.state().lod_tiles, t.state().lod_pending)
            })?;
        t.secs(0.5).await?;
        let shot = t.shot("down").await?;
        // The sky's pale blue; a cloud may be in the way, which is white.
        let sky = shot.share(0.44, 0.42, 0.56, 0.58, |c| c[2] > 200 && c[0] > 120 && c[0] < 215);
        t.note(format!("sky under the camera: {sky:.3}"));
        t.check(sky < 0.05, format!("a hole in the land: {sky:.3}"));
        Ok(())
    }
    .await;
    VIEW.set(before.0);
    LOD.set(before.1);
    out
}

/// Sources share independent parameters, stable priority and lifetime rules across local code,
/// synchronized entity data and content presets. Camera and body sample different positions.
pub async fn environment_sources(t: T) -> R {
    use weft_sdk::environment::{blend, observer};
    t.flat().await?;
    t.op("mode spectator").await?;
    t.cmd("/weather clear").await?;
    t.c().set_look(0.0, -25.0);
    t.frames(3).await?;
    let initial = t.c().environment_values();
    let mut base = Environment::new("test_base")
        .priority(10)
        .fog(0x203040, 2.0, 40.0)
        .acoustics(800.0, 0.2, 0.8);
    base.values.sunlight = 0.5;
    t.check(t.c().environment(&base), "local source accepted");
    let mut addition = Environment::new("test_add").priority(20).blend(blend::ADD);
    addition.values.fog_end = 10.0;
    t.c().environment(&addition);
    let mut mul = Environment::new("test_mul")
        .priority(30)
        .blend(blend::MULTIPLY);
    mul.values.sunlight = 0.5;
    t.c().environment(&mul);
    t.frames(3).await?;
    let values = t.c().environment_values();
    t.check_eq(
        (values.fog_start, values.fog_end, values.sunlight, values.lowpass),
        (2.0, 50.0, 0.25, 800.0),
        "ordered operations leave unrelated parameters alone",
    );
    // Updating a named source replaces it, even when insertion order differs from priority.
    addition.values.fog_end = 20.0;
    t.c().environment(&addition);
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().fog_end, 60.0, "one contribution per named source");
    for name in ["test_base", "test_add", "test_mul"] {
        t.c().clear_environment(name);
    }
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().lowpass, initial.lowpass, "clearing restores the underlying environment");

    let fade = Environment::new("test_fade")
        .priority(50)
        .fade(1.0, 1.0)
        .sunlight(0.0);
    t.c().environment(&fade);
    t.secs(0.25).await?;
    let partial = t.c().environment_values().sunlight;
    t.check(partial > 0.1 && partial < 0.95, format!("fade begins smoothly: {partial}"));
    // A frame hook may refresh the source every frame without restarting its transition.
    for _ in 0..80 {
        t.c().environment(&fade);
        t.frames(1).await?;
    }
    t.secs(1.1).await?;
    t.check_eq(t.c().environment_values().sunlight, 0.0, "a refreshed source reaches its target");
    t.c().clear_environment("test_fade");
    t.secs(0.25).await?;
    let partial = t.c().environment_values().sunlight;
    t.check(partial > 0.05 && partial < 0.9, format!("removed source fades out: {partial}"));
    t.secs(1.0).await?;

    let f = t.feet();
    let eye = t.state().camera_pos;
    let center = [eye.x, eye.y, eye.z];
    let cell = [eye.x.floor() as i32, eye.y.floor() as i32, eye.z.floor() as i32];
    t.set(cell, "stone").await?;
    let camera = Environment::new("test_point")
        .priority(50)
        .around(center, 1.0, 2.0)
        .when("block(base:stone)")
        .acoustics(600.0, 0.0, 1.0);
    t.c().environment(&camera);
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().lowpass, 600.0, "condition reads the sampled block");
    t.set(cell, "air").await?;
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().lowpass, initial.lowpass, "condition failure removes its contribution");
    let camera = camera.when("");
    t.c().environment(&camera);
    t.op("camera testmod:orbit").await?;
    t.secs(0.8).await?;
    t.check_eq(t.c().environment_values().lowpass, initial.lowpass, "detached camera leaves the zone");
    let body = camera.clone().observer(observer::BODY);
    t.c().environment(&body);
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().lowpass, 600.0, "body source stays at the player's eyes");
    t.c().clear_environment("test_point");
    t.op("camera core:first_person").await?;
    t.set(cell, "air").await?;
    t.secs(0.8).await?;

    let orb = t
        .op(&format!("spawn testmod:orb {} {} {}", f[0], f[1], f[2] - 2))
        .await?;
    t.until("entity-kind zone reaches observer", Duration::from_secs(5), |t| (t.c().environment_values().lowpass - 4000.0).abs() < 1.0)
        .await?;
    t.op(&format!("remove {orb}")).await?;
    t.until("despawn removes entity-kind source", Duration::from_secs(5), |t| t.c().environment_values().lowpass == initial.lowpass)
        .await?;

    let plain = t.shot("sunlight").await?;
    let screen = Environment::new("test_screen")
        .priority(5)
        .overlay("base:block/stone", 0.4);
    t.c().environment(&screen);
    t.frames(3).await?;
    let textured = t.grab().await?;
    t.check(T::differ(&plain.px, &textured) > 0.03, "overlay contributes through a source, resolving a tiled block texture");
    t.c().environment(
        &Environment::new("test_shader")
            .priority(6)
            .post(&Shader::named("testmod:dream"), 1.0),
    );
    t.frames(3).await?;
    let filtered = t.grab().await?;
    t.check(T::differ(&textured, &filtered) > 0.01, "post shader contributes independently of the overlay");
    t.c().environment(
        &Environment::new("test_clear_screen")
            .priority(100)
            .replace_post()
            .replace_overlays(),
    );
    t.frames(3).await?;
    let restored = t.grab().await?;
    t.check(T::differ(&plain.px, &restored) < 0.01, "higher empty replacement suppresses all preceding screen effects");
    t.c().clear_environment("test_clear_screen");
    t.c().clear_environment("test_screen");
    t.c().clear_environment("test_shader");
    t.frames(3).await?;
    t.cmd("/eclipse on").await?;
    t.until("server source reaches client", Duration::from_secs(5), |t| t.c().environment_values().sunlight == 0.0)
        .await?;
    let dark = t.shot("eclipse").await?;
    t.check_eq(t.c().environment_values().fog_end, 48.0, "server source controls fog independently");
    t.check_eq(t.c().environment_values().cloud_fog_end, 48.0, "server source controls cloud fog independently");
    t.c().environment(
        &Environment::new("test_cloud_range")
            .priority(200)
            .cloud_fog(initial.cloud_fog_start, initial.cloud_fog_end),
    );
    t.frames(3).await?;
    let cloud_lighting = t.shot("eclipse_cloud_light").await?;
    let daylight = crate::runner::lum(plain.avg(0.2, 0.6, 0.8, 0.85));
    let eclipse = crate::runner::lum(dark.avg(0.2, 0.6, 0.8, 0.85));
    let white_cloud = |c: probe::C| c[0] > 150 && c[0].abs_diff(c[1]) < 15 && c[0].abs_diff(c[2]) < 15;
    let bright_clouds = plain.share(0.05, 0.02, 0.95, 0.2, white_cloud);
    let eclipsed_clouds = cloud_lighting.share(0.05, 0.02, 0.95, 0.2, white_cloud);
    t.probe("daylight illuminates clouds", bright_clouds, bright_clouds > 0.01);
    t.probe("clouds lose sunlight together with the world", eclipsed_clouds, eclipsed_clouds < 0.005);
    t.c().clear_environment("test_cloud_range");
    t.frames(3).await?;
    t.probe("solar illumination can be extinguished at noon", daylight - eclipse, daylight - eclipse > 30.0);
    t.set(add(f, [0, 0, -3]), "base:torch").await?;
    t.secs(0.5).await?;
    let lamp = t.shot("eclipse_lamp").await?;
    t.check(T::differ(&dark.px, &lamp.px) > 0.01, "block lighting remains visible without sunlight");
    t.set(add(f, [0, 0, -3]), "air").await?;
    t.secs(0.5).await?;
    let mut unlit = Environment::new("test_unlit").priority(200).sunlight(0.0);
    unlit.values.ambient = 0.0;
    t.c().environment(&unlit);
    t.frames(3).await?;
    t.check_eq(t.c().environment_values().ambient, 0.0, "ambient minimum is externally replaceable");
    let black = t.shot("no_ambient").await?;
    let dark_floor = crate::runner::lum(black.avg(0.4, 0.65, 0.6, 0.8));
    t.probe("no hidden minimum remains in the renderer", dark_floor, dark_floor < 0.5);
    t.check_eq(t.c().environment_values().lowpass, initial.lowpass, "lighting override leaves acoustics alone");
    t.c().clear_environment("test_unlit");
    t.cmd("/eclipse off").await?;
    t.until("server removal restores underlying light", Duration::from_secs(5), |t| t.c().environment_values().sunlight == 1.0)
        .await?;
    t.check_eq(t.c().environment_values().fog_end, initial.fog_end, "server removal restores fog");
    t.check_eq(t.c().environment_values().cloud_fog_end, initial.cloud_fog_end, "server removal restores cloud distance");
    Ok(())
}
