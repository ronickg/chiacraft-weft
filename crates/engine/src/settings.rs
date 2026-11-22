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

/// How a world drawn smaller than the window is scaled up (`core:upscaler`): 0 bilinear, 1 FSR 1's
/// EASU, 2 Snapdragon GSR 1. For the shaders as `scene.upscale.x`.
pub fn upscaler() -> u32 {
    number("core:upscaler") as u32
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
    let min = if touch_controls() { GUI_MIN_TOUCH } else { GUI_MIN };
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

    pub fn saves_dir(&self) -> PathBuf {
        self.run_dir.join("saves")
    }
}

pub fn language() -> String {
    config::text(id("core:language"))
}
/// 0 = automatic.
pub fn gui_scale() -> u32 {
    number("core:gui_scale") as u32
}
/// The part of the screen's size the world is drawn at.
pub fn render_scale() -> f32 {
    number("core:render_scale") as f32
}
pub fn view_distance() -> i32 {
    number("core:view_distance") as i32
}
pub fn vsync() -> bool {
    number("core:vsync") != 0.0
}
pub fn fps_limit() -> u32 {
    number("core:fps_limit") as u32
}
/// The chosen renderer's name (`core:renderer`).
pub fn renderer() -> String {
    let id = id("core:renderer");
    crate::content::content()
        .game
        .configs
        .get(id as usize)
        .map(|d| d.show(config::get(id)))
        .unwrap_or_default()
}
pub fn fov() -> f32 {
    number("core:fov") as f32
}
pub fn fov_effects() -> f32 {
    number("core:fov_effects") as f32
}
pub fn camera_bob() -> bool {
    number("core:camera_bob") != 0.0
}
pub fn sensitivity() -> f32 {
    number("core:sensitivity") as f32
}
/// 0 moody .. 1 bright.
pub fn brightness() -> f32 {
    number("core:brightness") as f32
}
/// Bit per debug overlay group shown (F3 + number).
pub fn debug_groups() -> u32 {
    number("core:debug_groups") as u32
}
pub fn advanced_tooltips() -> bool {
    number("core:advanced_tooltips") != 0.0
}
/// Mouse up looks down instead.
pub fn invert_y() -> bool {
    number("core:invert_y") != 0.0
}
/// The HUD's buttons and steering for fingers.
pub fn touch_controls() -> bool {
    number("core:touch_controls") != 0.0
}

pub fn auto_jump() -> bool {
    number("core:auto_jump") != 0.0
}
pub fn name() -> String {
    config::text(id("core:name"))
}
pub fn server_port() -> u16 {
    number("core:server_port") as u16
}
/// Extra iroh relay URLs; empty uses the public defaults.
pub fn relays() -> Vec<String> {
    list("core:relays")
}
/// Enabled resource packs, highest priority first.
pub fn resource_packs() -> Vec<String> {
    list("core:resource_packs")
}
pub fn shader_pack() -> Option<String> {
    Some(config::text(id("core:shader_pack"))).filter(|p| !p.is_empty())
}

pub fn occlusion_culling() -> bool {
    number("core:occlusion_culling") != 0.0
}
pub fn frustum_culling() -> bool {
    number("core:frustum_culling") != 0.0
}

pub fn slim_skin() -> bool {
    number("core:skin_model") != 0.0
}

pub fn show_avatars() -> bool {
    number("core:show_avatars") != 0.0
}

pub fn first_person_model() -> bool {
    number("core:first_person_model") != 0.0
}

/// Every connection through the relay only (`core:net_relay_only`, a test setting).
pub fn net_relay_only() -> bool {
    number("core:net_relay_only") != 0.0
}

/// The Chia network for DIDs (chiacraft): `off`, `testnet11` or `mainnet`.
pub fn chain() -> String {
    config::text(id("core:chain")).trim().to_lowercase()
}

/// Own full nodes (`host:port`) instead of the DNS introducers.
pub fn chain_peers() -> Vec<String> {
    list("core:chain_peers")
}

/// What listing a world locks, in mojos (phase 5).
pub fn listing_deposit() -> u64 {
    number("core:listing_deposit").max(1.0) as u64
}

/// The fee a listing pays, in mojos (`core:listing_fee`).
pub fn listing_fee() -> u64 {
    number("core:listing_fee").max(0.0) as u64
}

/// DID sponsors' addresses (iroh endpoint ids), tried in order.
pub fn sponsors() -> Vec<String> {
    list("core:sponsors")
}

/// Screen effects' half-size parts (`post::HOOK_LOW`), live.
pub fn post_low() -> bool {
    number("core:post_low") != 0.0
}

/// Levels of the HDR world's bloom pyramid (`core:bloom`: 0 none, 1 four, 2 six), live.
pub fn bloom_levels() -> u32 {
    match number("core:bloom") as i32 {
        i32::MIN..=0 => 0,
        1 => 4,
        _ => 6,
    }
}

/// The soft clouds' shadow on the land with the HDR world (`client::cloud_shadow`), live.
pub fn cloud_shadows() -> bool {
    number("core:cloud_shadows") != 0.0
}

/// The HDR world (`post::world_format`), read once at start.
pub fn hdr_world() -> bool {
    number("core:hdr_world") != 0.0
}

/// Temporal anti-aliasing (`core:aa` 1 or 2, `post::set_taa`), read once at start.
pub fn aa() -> bool {
    number("core:aa") >= 1.0
}

/// Whether the world's pass draws the motion image too (`core:aa` 1; 2 is from the camera
/// alone, `post::set_motion`), read once at start.
pub fn aa_motion() -> bool {
    (number("core:aa") as i32) == 1
}

/// DLSS, the backend's temporal upscaler (`core:dlss`, `weft_render::UpscaleMode`), live; off
/// unless the backend has one (the `streamline` build on Windows).
pub fn dlss() -> weft_render::UpscaleMode {
    weft_render::UpscaleMode::from_number(number("core:dlss").max(0.0) as u32)
}

/// The stand-in upscaler for checks (`core:upscale_standin`), read at start.
pub fn upscale_standin() -> bool {
    number("core:upscale_standin") != 0.0
}

/// Ray Reconstruction with DLSS (`core:dlss_rr`), live.
pub fn dlss_rr() -> bool {
    number("core:dlss_rr") != 0.0
}

/// DLSS's model for `mode` (`core:dlss_model_<mode>`, 0 its default), live.
pub fn dlss_model(mode: weft_render::UpscaleMode) -> u32 {
    number(&format!("core:dlss_model_{}", mode.name())).max(0.0) as u32
}

/// Ray Reconstruction's model (`core:dlss_rr_model`), live.
pub fn dlss_rr_model() -> u32 {
    number("core:dlss_rr_model").max(0.0) as u32
}

/// DLSS 5 over the upscaled picture (`core:dlss5`), live.
pub fn dlss5() -> bool {
    number("core:dlss5") != 0.0
}

/// DLSS 5's strength (`core:dlss5_intensity`), live.
pub fn dlss5_intensity() -> f32 {
    (number("core:dlss5_intensity") as f32).clamp(0.0, 1.0)
}

/// The denoiser's check on today's frame (`core:pt_denoise_test`), live.
pub fn pt_denoise_test() -> bool {
    number("core:pt_denoise_test") != 0.0
}

/// The denoiser's tuning (`core:pt_dn_phi`, `core:pt_dn_steps`), live.
pub fn pt_dn_debug() -> u32 {
    (number("core:pt_dn_debug") as u32).min(6)
}

pub fn pt_dn_tuning() -> (f32, u32) {
    ((number("core:pt_dn_phi") as f32).clamp(0.25, 16.0), (number("core:pt_dn_steps") as u32).clamp(1, 5))
}

/// How the path tracer's light is denoised (`core:pt_denoiser`: 0 raw, 1 ours), live.
pub fn pt_denoiser() -> u32 {
    (number("core:pt_denoiser") as u32).min(crate::denoise::SVGF)
}

/// Temporal anti-aliasing's least share of each new frame (`core:taa_blend`), live.
pub fn taa_blend() -> f32 {
    (number("core:taa_blend") as f32).clamp(0.03, 0.5)
}

/// The mist as a volume for packs that have one (`client::froxels`), live.
pub fn froxels() -> bool {
    number("core:froxels") != 0.0
}

/// Pixels a mist volume cell covers each way: 8 (`core:froxels` 1), 4 (2, ultra).
pub fn froxel_cell() -> u32 {
    if number("core:froxels") >= 2.0 { 4 } else { 8 }
}

pub fn set(name: &str, v: f64) {
    config::set(id(&format!("core:{name}")), v);
}
pub fn set_text(name: &str, v: &str) {
    config::set_text(id(&format!("core:{name}")), v);
}
pub fn set_resource_packs(packs: &[String]) {
    set_text("resource_packs", &packs.join(","));
}

/// Every key as its `core:key.<action>` setting says.
pub fn apply_binds(input: &mut Input) {
    let actions: Vec<String> = input.binds.iter().map(|b| b.action.clone()).collect();
    for action in actions {
        match Binding::parse_slots(&config::text(id(&format!("core:key.{action}")))) {
            Some(k) => input.rebind(&action, k),
            None => log::warn!("key binding {action}: unknown key {}", config::text(id(&format!("core:key.{action}")))),
        }
    }
}

pub fn set_bind(action: &str, keys: &Slots) {
    config::set_text(id(&format!("core:key.{action}")), &Binding::slots_id(keys));
}

/// The system's language as a lang code (`ru_ru`), English when the system doesn't say.
pub fn system_language() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
        .or_else(os_language)
        .and_then(|locale| lang_code(&locale))
        .unwrap_or_else(|| crate::ui::lang::FALLBACK.into())
}

/// `ru_RU.UTF-8` and `ru-RU` are `ru_ru`; `C`, `POSIX` and a language without a region are none.
fn lang_code(locale: &str) -> Option<String> {
    let code = locale
        .split(['.', '@'])
        .next()?
        .replace('-', "_")
        .to_lowercase();
    let (lang, region) = code.split_once('_')?;
    (!lang.is_empty() && !region.is_empty()).then_some(code)
}

#[cfg(windows)]
fn os_language() -> Option<String> {
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
        fn LCIDToLocaleName(lcid: u32, name: *mut u16, len: i32, flags: u32) -> i32;
    }
    const LOCALE_NAME_MAX_LENGTH: usize = 85;
    let mut name = [0u16; LOCALE_NAME_MAX_LENGTH];
    // Characters written with the closing zero, 0 when the language has no name.
    let written = unsafe { LCIDToLocaleName(GetUserDefaultUILanguage().into(), name.as_mut_ptr(), name.len() as i32, 0) };
    (written > 1).then(|| String::from_utf16_lossy(&name[..written as usize - 1]))
}

#[cfg(not(windows))]
fn os_language() -> Option<String> {
    None
}
