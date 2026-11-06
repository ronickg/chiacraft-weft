//! The client's options as the engine reads them: typed views of its `core:` settings
//! (`config::engine_configs`), which live in `config/core.txt` like every mod's. Setting one
//! saves it; the client applies what changed by itself (`Running::apply_settings`).

use std::path::{Path, PathBuf};

use crate::config;
use crate::input::{Binding, Input, Slots};

#[derive(Clone)]
pub struct Settings {
    pub run_dir: PathBuf,
    /// An automated run: it saves nothing and never takes the real mouse.
    pub read_only: bool,
}

/// The 3D world's share of the window's pixels each way: the setting, or on auto (0) two
/// thirds on a high-density screen (`density`, physical pixels per point, 2 on Retina), where
/// shading every pixel cost several times what it bought, and all of them elsewhere.
/// How a world drawn smaller than the window is scaled up (`core:upscaler`): 0 bilinear, 1 FSR 1's
/// EASU, 2 Snapdragon GSR 1. For the shaders as `scene.upscale.x`.
pub fn upscaler() -> u32 {
    number("core:upscaler") as u32
}

pub fn render_scale(density: f64) -> f32 {
    let set = number("core:render_scale") as f32;
    if set > 0.0 {
        return set.clamp(0.5, 1.0);
    }
    if density >= 1.5 { 0.67 } else { 1.0 }
}

/// The largest GUI scale a window shows whole (vanilla's 320x240 per step); auto uses it.
/// The GUI pixels every screen needs to fit whole (chiacraft: the widest and tallest window, the
/// animal screen at about 385 x 300 with its tab, and a margin round it). The automatic scale,
/// and the most the setting goes to, keeps at least this much room, so nothing is cut off on a
/// small or high-density screen (a 3456 x 2234 MacBook: 6, where 320 x 240 gave 9).
pub const GUI_MIN: (u32, u32) = (400, 320);

/// The largest GUI scale at which a window of `window` physical pixels still holds `GUI_MIN`: an
/// even number, and never under 2 (a window too small for 2 at `GUI_MIN` scrolls its screens,
/// `App::open`, rather than drop to 1, where the text came out twice its boxes' size: Playtest B9 at
/// 760 x 600 and 640 x 360). A UI pixel (a frame's, an item's, a
/// letter's) is half a GUI pixel, so only at the even scales does it land on whole screen pixels:
/// at 3 a frame pixel is 1.5, and at 1 it is half a screen pixel, so the text, which is never drawn
/// smaller than a screen pixel a font pixel, came out twice the size its boxes were laid out for
/// (Playtest B9: the animal screen at GUI scale 1, its buttons' labels over each other; Researcher,
/// round 13; Minecraft keeps to even scales for the same reason with its half-size font).
pub fn max_gui_scale(window: (u32, u32)) -> u32 {
    let min = if crate::touch::enabled() { GUI_MIN_TOUCH } else { GUI_MIN };
    even_scale((window.0 / min.0).min(window.1 / min.1)).max(2)
}

/// `GUI_MIN` on a touch screen: a phone held sideways (2400 x 1080) gets scale 4, whose GUI
/// pixels a finger can hit, rather than 2; screens taller than its 270 GUI pixels scroll.
pub const GUI_MIN_TOUCH: (u32, u32) = (400, 240);

/// `s` rounded down to a scale the UI pixel sits whole on: 2, 4, 6... (1 below 2).
pub fn even_scale(s: u32) -> u32 {
    if s <= 1 { 1 } else { s - s % 2 }
}

/// The scale the GUI is drawn at for the setting `setting` (0: auto) in a window that allows up to
/// `max`: auto is `max`; a setting is kept even, at least 2 (a 1 or a 3 saved before scales were
/// kept even shows at 2), and no more than `max`.
pub fn applied_gui_scale(setting: u32, max: u32) -> u32 {
    if setting == 0 { max } else { even_scale(setting.max(2)).min(max) }
}

/// The GUI scale setting's next step: auto (0), then the even scales up to `max`, then auto.
pub fn next_gui_scale(now: u32, max: u32) -> u32 {
    let next = match now {
        0 => 2,
        n => even_scale(n.max(2)) + 2,
    };
    if next > max.max(2) { 0 } else { next }
}

#[cfg(test)]
mod gui_scale_tests {
    use super::*;

    #[test]
    fn scales_are_even() {
        assert_eq!(max_gui_scale((1280, 720)), 2);
        assert_eq!(max_gui_scale((1920, 1080)), 2);
        assert_eq!(max_gui_scale((2560, 1440)), 4);
        assert_eq!(max_gui_scale((3456, 2234)), 6);
        // too small for 2 at `GUI_MIN`: still 2, its screens scroll
        assert_eq!(max_gui_scale((640, 480)), 2);
        assert_eq!(max_gui_scale((640, 360)), 2);
        assert_eq!(even_scale(3), 2);
        assert_eq!(even_scale(7), 6);
    }

    #[test]
    fn a_setting_of_one_shows_at_two() {
        assert_eq!(applied_gui_scale(1, 2), 2);
        assert_eq!(applied_gui_scale(3, 6), 2);
        assert_eq!(applied_gui_scale(0, 6), 6);
        assert_eq!(applied_gui_scale(8, 6), 6);
        // a window too small for 2 at `GUI_MIN` still gets 2
        assert_eq!(applied_gui_scale(1, max_gui_scale((760, 600))), 2);
        assert_eq!(applied_gui_scale(0, max_gui_scale((640, 360))), 2);
    }

    #[test]
    fn the_setting_steps_through_even_scales() {
        let mut s = 0;
        let mut seen = vec![];
        for _ in 0..5 {
            s = next_gui_scale(s, 6);
            seen.push(s);
        }
        assert_eq!(seen, vec![2, 4, 6, 0, 2]);
        assert_eq!(next_gui_scale(0, 2), 2);
        assert_eq!(next_gui_scale(2, 2), 0);
    }
}

fn id(name: &str) -> u32 {
    config::by_name(name).unwrap_or(u32::MAX)
}

fn number(name: &str) -> f64 {
    config::get(id(name))
}

/// How the chat is shown, as its settings have it.
#[derive(Clone, Copy, PartialEq)]
pub struct ChatLook {
    pub width: f32,
    /// The most it takes up open, and shown over the game.
    pub open_height: f32,
    pub closed_height: f32,
    /// Of the dark behind it, 0..255.
    pub dim: u8,
}

/// Which chat lines a player wants.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    All,
    System,
    Hidden,
}

pub fn chat_visibility() -> ChatVisibility {
    match number("core:chat_visibility") as u32 {
        0 => ChatVisibility::All,
        1 => ChatVisibility::System,
        _ => ChatVisibility::Hidden,
    }
}

pub fn chat_look() -> ChatLook {
    ChatLook {
        width: number("core:chat_width") as f32,
        open_height: number("core:chat_height") as f32,
        closed_height: number("core:chat_height_closed") as f32,
        dim: (number("core:chat_opacity") * 2.55) as u8,
    }
}

fn list(name: &str) -> Vec<String> {
    config::text(id(name))
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(String::from)
        .collect()
}

impl Settings {
    pub fn new(run_dir: &Path) -> Settings {
        Settings { run_dir: run_dir.to_path_buf(), read_only: false }
    }

    pub fn language(&self) -> String {
        config::text(id("core:language"))
    }
    /// 0 = automatic.
    /// Frames a second at most, 0 for no cap.
    pub fn fps_cap(&self) -> u32 {
        number("core:fps_cap") as u32
    }

    pub fn gui_scale(&self) -> u32 {
        number("core:gui_scale") as u32
    }
    pub fn view_distance(&self) -> i32 {
        number("core:view_distance") as i32
    }
    pub fn vsync(&self) -> bool {
        number("core:vsync") != 0.0
    }
    pub fn fov(&self) -> f32 {
        number("core:fov") as f32
    }
    pub fn sensitivity(&self) -> f32 {
        number("core:sensitivity") as f32
    }
    /// 0 moody .. 1 bright.
    pub fn brightness(&self) -> f32 {
        number("core:brightness") as f32
    }
    /// Bit per debug overlay group shown (F3 + number).
    pub fn debug_groups(&self) -> u32 {
        number("core:debug_groups") as u32
    }
    /// Mouse up looks down instead.
    pub fn invert_y(&self) -> bool {
        number("core:invert_y") != 0.0
    }
    /// The sneak key switches sneaking on and off instead of being held.
    /// The sprint key switches sprinting on and off instead of being held.
    pub fn name(&self) -> String {
        config::text(id("core:name"))
    }
    pub fn server_port(&self) -> u16 {
        number("core:server_port") as u16
    }
    /// Every connection through the relay only (`core:net_relay_only`, a test setting).
    pub fn net_relay_only(&self) -> bool {
        number("core:net_relay_only") != 0.0
    }
    /// Extra iroh relay URLs; empty uses the public defaults.
    pub fn relays(&self) -> Vec<String> {
        list("core:relays")
    }
    /// The Chia network for DIDs (chiacraft): `off`, `testnet11` or `mainnet`.
    pub fn chain(&self) -> String {
        config::text(id("core:chain")).trim().to_lowercase()
    }
    /// Own full nodes (`host:port`) instead of the DNS introducers.
    pub fn chain_peers(&self) -> Vec<String> {
        list("core:chain_peers")
    }
    /// What listing a world locks, in mojos (phase 5).
    pub fn listing_deposit(&self) -> u64 {
        number("core:listing_deposit").max(1.0) as u64
    }

    /// The fee a listing pays, in mojos (`core:listing_fee`).
    pub fn listing_fee(&self) -> u64 {
        number("core:listing_fee").max(0.0) as u64
    }
    /// DID sponsors' addresses (iroh endpoint ids), tried in order.
    pub fn sponsors(&self) -> Vec<String> {
        list("core:sponsors")
    }
    /// Enabled resource packs, highest priority first.
    pub fn resource_packs(&self) -> Vec<String> {
        list("core:resource_packs")
    }
    pub fn shader_pack(&self) -> Option<String> {
        Some(config::text(id("core:shader_pack"))).filter(|p| !p.is_empty())
    }

    pub fn occlusion_culling(&self) -> bool {
        number("core:occlusion_culling") != 0.0
    }

    /// Screen effects' half-size parts (`post::HOOK_LOW`), live.
    pub fn post_low(&self) -> bool {
        number("core:post_low") != 0.0
    }

    /// Levels of the HDR world's bloom pyramid (`core:bloom`: 0 none, 1 four, 2 six), live.
    pub fn bloom_levels(&self) -> u32 {
        match number("core:bloom") as i32 {
            i32::MIN..=0 => 0,
            1 => 4,
            _ => 6,
        }
    }

    /// The soft clouds' shadow on the land with the HDR world (`client::cloud_shadow`), live.
    pub fn cloud_shadows(&self) -> bool {
        number("core:cloud_shadows") != 0.0
    }

    /// The HDR world (`post::world_format`), read once at start.
    pub fn hdr_world(&self) -> bool {
        number("core:hdr_world") != 0.0
    }

    /// Temporal anti-aliasing (`core:aa` 1 or 2, `post::set_taa`), read once at start.
    pub fn aa(&self) -> bool {
        number("core:aa") >= 1.0
    }

    /// Whether the world's pass draws the motion image too (`core:aa` 1; 2 is from the camera
    /// alone, `post::set_motion`), read once at start.
    pub fn aa_motion(&self) -> bool {
        (number("core:aa") as i32) == 1
    }

    /// DLSS, the backend's temporal upscaler (`core:dlss`, `weft_render::UpscaleMode`), live; off
    /// unless the backend has one (the `streamline` build on Windows).
    pub fn dlss(&self) -> weft_render::UpscaleMode {
        weft_render::UpscaleMode::from_number(number("core:dlss").max(0.0) as u32)
    }

    /// The stand-in upscaler for checks (`core:upscale_standin`), read at start.
    pub fn upscale_standin(&self) -> bool {
        number("core:upscale_standin") != 0.0
    }

    /// Ray Reconstruction with DLSS (`core:dlss_rr`), live.
    pub fn dlss_rr(&self) -> bool {
        number("core:dlss_rr") != 0.0
    }

    /// DLSS's model for `mode` (`core:dlss_model_<mode>`, 0 its default), live.
    pub fn dlss_model(&self, mode: weft_render::UpscaleMode) -> u32 {
        number(&format!("core:dlss_model_{}", mode.name())).max(0.0) as u32
    }

    /// Ray Reconstruction's model (`core:dlss_rr_model`), live.
    pub fn dlss_rr_model(&self) -> u32 {
        number("core:dlss_rr_model").max(0.0) as u32
    }

    /// DLSS 5 over the upscaled picture (`core:dlss5`), live.
    pub fn dlss5(&self) -> bool {
        number("core:dlss5") != 0.0
    }

    /// DLSS 5's strength (`core:dlss5_intensity`), live.
    pub fn dlss5_intensity(&self) -> f32 {
        (number("core:dlss5_intensity") as f32).clamp(0.0, 1.0)
    }

    /// The denoiser's check on today's frame (`core:pt_denoise_test`), live.
    pub fn pt_denoise_test(&self) -> bool {
        number("core:pt_denoise_test") != 0.0
    }

    /// How the path tracer's light is denoised (`core:pt_denoiser`: 0 raw, 1 ours), live.
    pub fn pt_denoiser(&self) -> u32 {
        (number("core:pt_denoiser") as u32).min(crate::denoise::SVGF)
    }

    /// Temporal anti-aliasing's least share of each new frame (`core:taa_blend`), live.
    pub fn taa_blend(&self) -> f32 {
        (number("core:taa_blend") as f32).clamp(0.03, 0.5)
    }

    /// The mist as a volume for packs that have one (`client::froxels`), live.
    pub fn froxels(&self) -> bool {
        number("core:froxels") != 0.0
    }

    /// Pixels a mist volume cell covers each way: 8 (`core:froxels` 1), 4 (2, ultra).
    pub fn froxel_cell(&self) -> u32 {
        if number("core:froxels") >= 2.0 { 4 } else { 8 }
    }

    /// Soft clouds for packs that draw them (`client::soft_clouds`).
    pub fn soft_clouds(&self) -> bool {
        number("core:soft_clouds") != 0.0
    }

    pub fn show_avatars(&self) -> bool {
        number("core:show_avatars") != 0.0
    }

    pub fn first_person_model(&self) -> bool {
        number("core:first_person_model") != 0.0
    }

    pub fn set(&self, name: &str, v: f64) {
        config::set(id(&format!("core:{name}")), v);
    }
    pub fn set_text(&self, name: &str, v: &str) {
        config::set_text(id(&format!("core:{name}")), v);
    }
    pub fn set_resource_packs(&self, packs: &[String]) {
        self.set_text("resource_packs", &packs.join(","));
    }

    pub fn saves_dir(&self) -> PathBuf {
        self.run_dir.join("saves")
    }

    /// Every key as its `core:key.<action>` setting says.
    pub fn apply_binds(&self, input: &mut Input) {
        let actions: Vec<String> = input.binds.iter().map(|b| b.action.clone()).collect();
        for action in actions {
            match Binding::parse_slots(&config::text(id(&format!("core:key.{action}")))) {
                Some(k) => input.rebind(&action, k),
                None => log::warn!("key binding {action}: unknown key {}", config::text(id(&format!("core:key.{action}")))),
            }
        }
    }

    pub fn set_bind(&self, action: &str, keys: &Slots) {
        config::set_text(id(&format!("core:key.{action}")), &Binding::slots_id(keys));
    }
}

/// Brings settings files written by older versions up to date.
pub fn migrate(run_dir: &Path) {
    let config = crate::paths::config(run_dir);
    merge_settings(&config);
    for dir in [config, run_dir.join("test-config")] {
        migrate_bindings(&dir.join("core.txt"));
    }
}

/// Sneak and sprint switched by two settings of their own become toggled bindings, and a sprint
/// or fly key saved when they had one binding gets the double tap the engine now gives them.;
/// TODO remove in 26.11.3
fn migrate_bindings(path: &Path) {
    let Ok(src) = std::fs::read_to_string(path) else { return };
    let mut pairs: Vec<(String, String)> = crate::kv::pairs(&src)
        .map(|(_, k, v)| (k.to_string(), v.to_string()))
        .collect();
    let before = pairs.clone();
    let mut toggled = Vec::new();
    pairs.retain(|(k, v)| match k.strip_prefix("toggle_") {
        Some(action @ ("sneak" | "sprint")) => {
            if matches!(v.as_str(), "true" | "1") {
                toggled.push(format!("key.core:{action}"));
            }
            false
        }
        _ => true,
    });
    for (k, v) in &mut pairs {
        match (k.as_str(), v.as_str()) {
            ("key.core:sprint", "ControlLeft") => *v = "ControlLeft, double KeyW".into(),
            ("key.core:fly", "KeyF") => *v = "KeyF, double Space".into(),
            _ => {}
        }
        if toggled.contains(k) && !v.starts_with("toggle ") {
            *v = format!("toggle {v}");
        }
    }
    if pairs != before {
        let out: String = pairs
            .iter()
            .map(|(k, v)| {
                format!(
                    "{k} = {v}
"
                )
            })
            .collect();
        let _ = std::fs::write(path, out);
    }
}

/// The old `config/settings.txt` joins `config/core.txt` once (lists were one line per entry).
fn merge_settings(config: &Path) {
    let (old, new) = (config.join("settings.txt"), config.join("core.txt"));
    let Ok(src) = std::fs::read_to_string(&old) else { return };
    let mut out = std::fs::read_to_string(&new).unwrap_or_default();
    let (mut packs, mut relays) = (Vec::new(), Vec::new());
    for (_, k, v) in crate::kv::pairs(&src) {
        match k {
            "resource_pack" => packs.push(v),
            "relay" => relays.push(v),
            _ => out.push_str(&format!(
                "{k} = {v}
"
            )),
        }
    }
    out.push_str(&format!(
        "resource_packs = {}
relays = {}
",
        packs.join(","),
        relays.join(",")
    ));
    if std::fs::write(&new, out).is_ok() {
        let _ = std::fs::remove_file(&old);
    }
}

pub fn system_language() -> String {
    let lang = std::env::var("LANG").unwrap_or_default().to_lowercase();
    if lang.starts_with("ru") || sys_locale_ru() { "ru_ru".into() } else { "en_us".into() }
}

#[cfg(windows)]
fn sys_locale_ru() -> bool {
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }
    // Primary language id 0x19 is Russian.
    unsafe { GetUserDefaultUILanguage() & 0x3FF == 0x19 }
}

#[cfg(not(windows))]
fn sys_locale_ru() -> bool {
    false
}
