pub mod api;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use weft_abi::entry_kind as ek;

use glam::Vec2;
use weft_abi as abi;
use weft_render::{Backend, BackendDesc, Cmd, CommandList, Format, Target, TextureDesc, TextureId};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, NamedKey, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::assets::Assets;
use crate::client::textures::GameTextures;
use crate::client::{Client, DragPreview};
use crate::content::content;
use crate::hud::Hud;
use crate::input::{Input, Key};
use crate::modloader::{self, LoadedMod};
use crate::net;
use crate::saves::{self, WorldMeta};
use crate::screens::{self, BuildCx, Screen};
use crate::server::{self, ServerConfig, ServerHandle};
use crate::settings::Settings;
use crate::ui::lang::Lang;
use crate::ui::look::Look;
use crate::ui::render::UiRenderer;
use crate::ui::text::{self, Arg, Msg};
use crate::ui::{Kind, Style, Ui};

pub type BackendFactory = fn(&BackendDesc) -> Result<Box<dyn Backend>, weft_render::Error>;

/// The renderers the game was built with, by the name `core:renderer` chooses them by.
pub type Renderers = &'static [(&'static str, BackendFactory)];

#[derive(Clone, Debug)]
pub struct Options {
    pub run_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub mod_dirs: Vec<PathBuf>,
    /// Folders of data packages (chiacraft), earlier ones winning for a package found twice.
    pub package_dirs: Vec<PathBuf>,
    /// Mods off for this run only (`--off a,b`, from a launcher).
    pub mods_off: Vec<String>,
    pub gpu_debug: bool,
    /// Automation: skip the menu and play this world (created if missing).
    pub world: Option<String>,
    pub seed: Option<u64>,
    /// Automation: open this built-in screen.
    pub screen: Option<String>,
    pub size: (u32, u32),
    pub dev: bool,
    /// A mod drives the game (tests, scripts): no pause on focus loss, settings not saved.
    pub automated: bool,
    /// Benchmarks: force vsync off / a view distance without touching saved settings.
    pub no_vsync: bool,
    pub view: Option<i32>,
    /// `core:renderer` for this run only.
    pub renderer: Option<String>,
}

pub enum Request {
    Open(Box<dyn Screen>),
    /// A screen by name: a mod's replacement if there is one, else the engine's.
    Screen(String),
    /// A mod's button was clicked: its callback (`UiArgs`) runs with the whole client API, like
    /// a frame hook, after the input is handled.
    ModCall(weft_abi::ModId, weft_abi::Callback, u32),
    /// A mod's input or slider changed: its `on_change` with the text or value.
    ModChange(weft_abi::ModId, weft_abi::Callback, String, f32, u32),
    /// A key for a mod's screen (`on_key`), by its settings name.
    ModKey(weft_abi::ModId, weft_abi::Callback, String),
    Close,
    Quit,
    /// Starts the game anew the way it was started, then leaves: what mods are on takes effect.
    Restart,
    Rebuild,
    Sound(crate::audio::Play),
    Rebind(String, crate::input::Slots),
    ResetBinds,
    ResetHudLayout,
    SetDebug(bool),
    StartWorld(WorldMeta),
    /// Into a world from its picture in the list, `picture` (a GUI texture) growing from `from`
    /// (GUI pixels) to the whole window first.
    EnterWorld {
        world: WorldMeta,
        picture: String,
        from: crate::ui::Rect,
    },
    /// Without asking about mods that are missing.
    StartWorldAnyway(WorldMeta),
    LeaveWorld,
    Host,
    Join(String),
    /// Back to your own world from a visit (chiacraft).
    ReturnHome,
    /// Asks the world at this address to link with the world in play, with a note (chiacraft).
    AskLink(String, String),
    /// Makes an invite to link with the world in play (chiacraft, `links::invite`).
    MakeInvite,
    /// Stops the join in progress (chiacraft).
    CancelJoin,
    /// The join sheet's answer: download the world's packages; true also trusts their creators.
    JoinConsent(bool),
    Respawn,
    SendChat(String),
}

/// A world's picture growing from its place in the list to the whole window while the world
/// loads under the list, then faded into the world, playable from then on: going into the world
/// as into the picture.
struct Entering {
    picture: String,
    /// Where it was, GUI pixels.
    from: crate::ui::Rect,
    started: Instant,
    expanded: bool,
    /// When the prepared view started to replace the picture.
    shown: Option<Instant>,
}

struct OpenedWorld {
    storage: saves::Storage,
    skin: Option<net::Skin>,
    avatar: Option<net::AvatarData>,
    chat_history: Vec<String>,
    /// The world's own key (chiacraft), made before the world was read.
    identity: Option<Arc<crate::world_address::WorldIdentity>>,
}

enum OpeningWorld {
    Reading(crossbeam_channel::Receiver<OpenedWorld>),
    Streaming {
        conn: net::ClientConn,
        server: ServerHandle,
        dir: PathBuf,
        view: i32,
        skin: Option<net::Skin>,
        chat_history: Vec<String>,
        /// The world's name and its own key (chiacraft), for `Game`.
        world: String,
        identity: Option<Arc<crate::world_address::WorldIdentity>>,
    },
}

/// Cubes about the player whose ground has to be drawn for the world to be there to see.
const NEAR: i32 = 2;

/// Seconds the HUD takes to come in when the world is there.
const HUD_FADE: f32 = 0.5;

/// Seconds the picture takes to fill the window, and to fade into the world.
const ENTER_GROW: f32 = 0.35;
const ENTER_FADE: f32 = 0.3;
/// A slow or unusual spawn can finish streaming once play has started.
const ENTER_LOAD_LIMIT: Duration = Duration::from_millis(1500);

#[derive(Clone, Copy)]
enum PreviewKind {
    /// chiacraft: whose figure, the player (0), the open menu's own entity (1), or an entity kind
    /// alone (2 + its index).
    Entity(u64),
    Block(u64),
}

#[derive(Clone, Copy)]
struct ModelPreview {
    rect: crate::ui::Rect,
    kind: PreviewKind,
}

/// What UI actions and bindings can see and change.
pub struct Ctx<'a> {
    pub settings: &'a mut Settings,
    pub client: Option<&'a mut Client>,
    pub input: &'a Input,
    pub requests: &'a mut Vec<Request>,
    pub stats: &'a Stats,
    pub debug: bool,
    /// The top screen and its UI, for what its buttons and keys do to it.
    pub top: Option<Top<'a>>,
}

pub struct Top<'a> {
    /// None while the screen itself handles a key.
    pub screen: Option<&'a mut dyn Screen>,
    pub ui: &'a mut Ui,
}

impl Ctx<'_> {
    /// The text of the top screen's input `id`.
    pub fn input_text(&self, id: &str) -> Option<String> {
        let ui = &self.top.as_ref()?.ui;
        ui.find(id).and_then(|n| match &ui.nodes[n].kind {
            Kind::Input { value, .. } => Some(value.clone()),
            _ => None,
        })
    }

    /// Replaces the text of the top screen's input `id` (completion, history).
    pub fn set_input(&mut self, id: &str, text: String) {
        let Some(top) = &mut self.top else { return };
        if let Some(n) = top.ui.find(id)
            && let Kind::Input { value, caret, .. } = &mut top.ui.nodes[n].kind
        {
            *caret = text.chars().count();
            *value = text;
            top.ui.dirty = true;
        }
    }

    /// The top screen as what it is, for its own buttons.
    pub fn screen<S: Screen>(&mut self) -> Option<&mut S> {
        let s: &mut dyn Screen = self.top.as_mut()?.screen.as_deref_mut()?;
        (s as &mut dyn std::any::Any).downcast_mut::<S>()
    }
}

/// Numbers the debug overlay shows, refreshed every frame.
#[derive(Default, Clone)]
pub struct Stats {
    pub fps: f32,
    pub frame_ms: f32,
    /// CPU parts of the last frame (`bench::CPU_PARTS`) and GPU passes (`bench::GPU_PARTS`).
    pub cpu: [f32; 5],
    pub gpu: Vec<f32>,
    pub server_ms: f32,
    pub phases: [f32; 6],
    pub regions: u32,
    /// Cubes generating or waiting for insertion, and cubes not yet sent to a player.
    pub load: (u64, u64),
    pub gpu_name: String,
    pub cpu_name: String,
    /// Physical memory of the machine, bytes.
    pub memory_total: u64,
    pub threads: usize,
    pub window: (u32, u32),
    pub gui_scale: f32,
}

pub struct Game {
    pub client: Client,
    pub server: Option<ServerHandle>,
    /// The world's name (empty for a world joined over the network).
    pub world: String,
    /// The world's own key and certificate (chiacraft), for hosting it; none for a world joined
    /// over the network or without an account.
    pub identity: Option<Arc<crate::world_address::WorldIdentity>>,
}

/// A trip through a portal (chiacraft): whether the player has left the world they set out from.
pub(crate) struct Travel {
    left: bool,
}

/// Your own world, kept running while you visit others because players are in it (it is
/// hosted; chiacraft). It keeps its own content and settings (`server::Scope`).
pub(crate) struct Home {
    server: ServerHandle,
    world: String,
    identity: Option<Arc<crate::world_address::WorldIdentity>>,
}

/// Commands the engine runs on the client that types them: name, usage, handler. One table for
/// completion, `/help` and running them.
pub(crate) const CLIENT_COMMANDS: &[(&str, &str, fn(&mut Running, &crate::commands::Args) -> Vec<Msg>)] = &[
    // Logging is per process.
    ("log", "[filters:text]", |_, a| crate::commands::log_command(a.text("filters").unwrap_or(""))),
    // The shown gizmo groups belong to this client.
    ("gizmo", "[group:word(#core:gizmos)]", |r, a| {
        r.game
            .as_mut()
            .map_or(Vec::new(), |g| g.client.gizmo_command(a.text("group").unwrap_or("")))
    }),
    ("music", "", |r, _| match &mut r.audio {
        Some(a) => {
            a.skip_music();
            vec![Msg::new("command.music.next", Vec::new())]
        }
        None => vec![Msg::new("command.music.no_device", Vec::new())],
    }),
    // What this client hears now, most important first (also in the log, for scripts).
    ("sounds", "[which:word]", |r, args| match &r.audio {
        Some(a) if args.text("which") == Some("recent") => {
            // The last sounds started, newest first (also in the log): one-shots that already ended.
            let mut out = Vec::new();
            for (name, volume, looped) in a.recent() {
                log::info!("sound recent: {name} volume {volume:.2}{}", if looped { " loop" } else { "" });
                let key = if looped { "command.sounds.loop" } else { "command.sounds.one" };
                out.push(Msg::new(key, vec![Arg::Text(name), Arg::Float(volume as f64, 2)]));
            }
            if out.is_empty() {
                out.push(Msg::new("command.sounds.none_recent", Vec::new()));
            }
            out
        }
        Some(a) => {
            let (playing, held) = a.counts();
            log::info!("sounds: {playing} playing, {held} held, {} played, {} dropped", a.played, a.dropped);
            let mut out =
                vec![Msg::new("command.sounds.count", vec![Arg::Int(playing as i64), Arg::Int(held as i64), Arg::Int(a.dropped as i64)])];
            for (name, volume, looped, waiting) in a.describe() {
                log::info!(
                    "sound: {name} volume {volume:.2}{}{}",
                    if looped { " loop" } else { "" },
                    if waiting { " reading" } else { "" }
                );
                let key = match (looped, waiting) {
                    (_, true) => "command.sounds.reading",
                    (true, _) => "command.sounds.loop",
                    _ => "command.sounds.one",
                };
                out.push(Msg::new(key, vec![Arg::Text(name), Arg::Float(volume as f64, 2)]));
            }
            out
        }
        None => vec![Msg::new("command.music.no_device", Vec::new())],
    }),
];

/// The sizes last asked of the upscaler: mode, Ray Reconstruction, screen, world.
type UpscaleSize = (weft_render::UpscaleMode, bool, (u32, u32), (u32, u32));

/// What goes to the backend's upscaler this frame (`upscale_frame`): the settings, less what the
/// upscaler doesn't run.
#[derive(Clone, Copy, Debug, PartialEq)]
struct UpscaleChoice {
    mode: weft_render::UpscaleMode,
    reconstruct: bool,
    model: u32,
    rr_model: u32,
    neural: bool,
}

pub(crate) struct Running {
    gpu: Box<dyn Backend>,
    cache_dir: PathBuf,
    /// Which of the game's renderers `gpu` is.
    renderer: &'static str,
    /// The backend's temporal upscaler (DLSS through Streamline on Windows), as it said at start;
    /// the mode this frame goes to it with (`core:dlss`), and the sizes last asked of it
    /// (mode, screen, world).
    upscaler: weft_render::UpscalerInfo,
    upscale_now: Option<UpscaleChoice>,
    upscale_logged: Option<UpscaleChoice>,
    upscale_size: Option<UpscaleSize>,
    window: Arc<Window>,
    depth: (TextureId, u32, u32),
    list: CommandList,
    /// The button clicked last and when, for double clicks.
    last_button: Option<(String, Instant)>,
    /// What goes in the world's pass this frame, while it is being made (chiacraft: recorded
    /// before the sun's shadow map that goes ahead of it in `list`).
    world_list: CommandList,
    /// What is drawn over the world after its depth was taken (`Client::render`).
    after_list: CommandList,
    input: Input,
    touches: crate::touch::Touches,
    /// The phone's keyboard is up (`soft_keyboard`).
    keyboard: bool,
    pub(crate) settings: Settings,
    lang: Lang,
    languages: Vec<(String, String)>,
    look: Look,
    assets: Assets,
    textures: GameTextures,
    ui: UiRenderer,
    post: crate::post::Post,
    /// The screen shaders the local player's effects turned on (`EntityEffectDef::screen`).
    audio: Option<crate::audio::Audio>,
    screens: Vec<(Box<dyn Screen>, Ui)>,
    hud: Hud,
    game: Option<Game>,
    /// Joining a friend's world, until it is let in (chiacraft: packages are downloaded first).
    joining: Option<net::p2p::Joining>,
    /// The packages the join sheet asks about.
    join_offer: Option<Vec<net::screens::OfferRow>>,
    /// Your world, running on while you visit another.
    home: Option<Home>,
    /// The address of the join in progress, saved once it lets the player in.
    join_address: Option<String>,
    /// `links::GENERATION` last shown.
    links_seen: u64,
    /// A trip through a portal in progress (chiacraft): the player stays in the world they are in
    /// until the destination is reached and its packages are downloaded.
    travel: Option<Travel>,
    /// The address of the world joined over the network that is in play.
    remote_address: Option<String>,
    /// Where to go back to if the world in play goes away: the world joined over the network that
    /// the player came from through a portal (their own world is `home`).
    came_from: Option<String>,
    /// The game just left, kept to the end of the frame: a mod's view of the client that asked to
    /// leave still points into it.
    retired: Option<Game>,
    last: Instant,
    shot_pending: bool,
    fps: f32,
    frame_ms: f32,
    fps_timer: Instant,
    fps_frames: u32,
    quads_dirty: bool,
    debug: bool,
    quit: bool,
    mods: Vec<LoadedMod>,
    client_hash: u64,
    /// Texture generation the UI quads were built with (icons drawn blank until loaded).
    ui_textures: u64,
    pub(crate) stats: Stats,
    /// F1: the world alone, until a screen opens or F1 again.
    hud_off: bool,
    /// The folder of the world played on this machine, for its picture in the list.
    world_dir: Option<PathBuf>,
    /// The world's picture being taken this frame, before the pause screen opens: where it goes,
    /// and `hud_off` as it was.
    preview_shot: Option<(PathBuf, bool)>,
    /// Going into a world from its picture in the list (`Request::EnterWorld`).
    entering: Option<Entering>,
    /// The save being opened, then its server streaming the spawn while the preview grows.
    opening_world: Option<OpeningWorld>,
    /// When the player entered the world and the HUD started fading in.
    hud_since: Option<Instant>,
    /// The HUD's quads among the cached ones.
    hud_quads: std::ops::Range<usize>,
    reload_requested: bool,
    /// Since when a requested reload has been held back while the world opens.
    reload_held: Option<Instant>,
    /// The settings in effect, and the config changes seen (`apply_settings`).
    applied: Applied,
    /// Packs changed while their screen was open (`apply_settings`).
    packs_pending: bool,
    config_seen: u64,
    /// CPU milliseconds of the last frame, see `bench::CPU_PARTS`.
    prof: [f32; 5],
    bench: Option<crate::bench::Bench>,
    diag: crate::diag::Reporter,
    /// UI quads as of the last rebuild; per-frame parts are added on top.
    ui_cached: Vec<crate::ui::render::Quad>,
    ui_animated: Vec<crate::ui::render::Animated>,
    /// Where UI asked for the player model, physical pixels.
    /// Where 3D previews go, and whose (chiacraft: `PreviewKind::Entity`).
    previews: Vec<ModelPreview>,
    /// Texture holding the rendered inventory figure, drawn by the UI.
    preview_tex: Option<u32>,
    menu_entities: Option<crate::client::entity_render::EntityRenderer>,
    menu_preview_failed: bool,
    shot_path: Option<PathBuf>,
    /// Image id the pending screenshot is stored under for mods.
    shot_image: Option<u64>,
    /// Keys mods hold down, released at the given time.
    mod_held: Vec<(Key, Instant, u64)>,
    /// What mods draw this frame (`Host::client_draw`).
    mod_gizmos: Vec<crate::net::GizmoShape>,
    /// Sounds of the UI and of mods' client code, played after this frame's world sounds.
    ui_sounds: Vec<crate::audio::Play>,
    /// Mods' screen text: where (window fractions), what, colour, size, the width it wraps to.
    mod_screen_text: Vec<(f32, f32, Msg, u32, f32, f32)>,
    frame_no: u64,
    frame_dt: f32,
    /// The top screen as of the last frame (stack depth, name) and frames since it changed.
    screen_seen: (usize, String),
    screen_frames: u32,
    /// Finished benches, the last one's fps, avg, p99, gpu and server ms, and its report line.
    benches: u32,
    bench_last: [f32; 5],
    bench_line: String,
}

struct App {
    renderers: Renderers,
    opts: Options,
    run: Option<Running>,
    error: Option<String>,
}

pub fn run(renderers: Renderers, opts: Options) -> Result<(), String> {
    run_on(EventLoop::new().map_err(|e| e.to_string())?, renderers, opts)
}

/// `run` in an Android app's activity.
#[cfg(target_os = "android")]
pub fn run_android(app: winit::platform::android::activity::AndroidApp, renderers: Renderers, opts: Options) -> Result<(), String> {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    crate::clipboard::init(app.clone());
    run_on(
        EventLoop::builder()
            .with_android_app(app)
            .build()
            .map_err(|e| e.to_string())?,
        renderers,
        opts,
    )
}

fn run_on(event_loop: EventLoop<()>, renderers: Renderers, opts: Options) -> Result<(), String> {
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { renderers, opts, run: None, error: None };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    match app.error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// The engine's own assets: those a distribution ships, else (a build run from the source tree, or
/// `--dev`) the source tree's.
pub fn engine_assets(dev: bool) -> PathBuf {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let shipped = crate::paths::shipped().map(|d| d.join("assets").join("core"));
    match shipped {
        Some(s) if !dev && s.is_dir() => s,
        _ => source,
    }
}

/// The settings `Running::apply_settings` last put into effect.
#[derive(Default, PartialEq)]
struct Applied {
    vsync: bool,
    frame_interval: Option<Duration>,
    view: i32,
    language: String,
    gui_scale: u32,
    advanced_tooltips: bool,
    /// Resource and shader packs: put into effect once their screens are closed, however.
    packs: (Vec<String>, Option<String>),
}

/// Mod ids in load order (where a bare UI class looks for its texture, the last first).
fn mod_ids() -> Vec<String> {
    let mut mods: Vec<&crate::content::ModMeta> = content().mods.iter().collect();
    mods.sort_by_key(|m| m.order);
    mods.iter().map(|m| m.id.clone()).collect()
}

/// `--hide-hud` (or `--record nohud`): the world without the HUD, for screenshots and footage.
fn hud_hidden() -> bool {
    let o = api::launch_options();
    o.contains_key("hide-hud") || o.get("record").is_some_and(|v| v == "nohud")
}

/// `--record`: a script run on screen for filming (full screen, animated time, no cursor).
fn recording() -> bool {
    api::launch_options().contains_key("record")
}

impl App {
    fn start(&mut self, el: &ActiveEventLoop) -> Result<Running, String> {
        // Mods first: the engine's settings are `core:` configs like theirs.
        let off = modloader::off_file(&self.opts.run_dir, self.opts.automated);
        let core_assets = engine_assets(self.opts.dev);
        // chiacraft: our data packages' folders (`package_dirs`, phase 2) after the mods' folders.
        let mods =
            modloader::load(&self.opts.mod_dirs, &self.opts.package_dirs, &self.opts.cache_dir, &off, &self.opts.mods_off, &core_assets)?;
        modloader::tidy_store(&self.opts.run_dir.join("saves"));
        crate::config::set_automated(self.opts.automated);
        // The account, before anything needs the network key: made quietly on the first start.
        // Automated runs keep theirs in the run directory's file store, never the OS keystore.
        if self.opts.automated {
            chiacraft_account::use_file_store();
        }
        if let Err(e) = chiacraft_account::open(&self.opts.run_dir) {
            log::error!("{e}; this run plays without an account, keyed by name");
        }
        crate::config::load(&self.opts.run_dir);
        if self.opts.no_vsync {
            crate::config::override_for_run("core:vsync", "false");
        }
        if let Some(v) = self.opts.view {
            crate::config::override_for_run("core:view_distance", &v.to_string());
        }
        if let Some(r) = &self.opts.renderer {
            crate::config::override_for_run("core:renderer", r);
        }
        // `--config "core:lod_distance=0,core:shader_pack="`: settings for this run only, as the
        // file writes them (benchmarks varying one thing at a time).
        if let Some(list) = api::launch_options().get("config") {
            for (name, value) in list.split(',').filter_map(|kv| kv.split_once('=')) {
                crate::config::override_for_run(name.trim(), value.trim());
            }
        }
        let mut settings = Settings::new(&self.opts.run_dir);
        log::info!("player name '{}' (setting core:name)", crate::settings::name());
        // Links between worlds, and the inbox for them, open while the game runs (chiacraft).
        crate::links::init(&self.opts.run_dir, &settings.saves_dir());
        crate::friends::init(&self.opts.run_dir, &settings.saves_dir());
        crate::saved_worlds::set_settings(&settings);
        if chiacraft_account::current().is_some() {
            net::p2p::start_account(&settings);
            crate::chiacraft::chain::configure(&settings.run_dir, &crate::settings::chain(), crate::settings::chain_peers());
            // The account's first address on that chain, public, in a file: where to send it test
            // coins (a listing's deposit, phase 5).
            if let (Some(s), Ok(net)) = (chiacraft_account::current(), crate::chiacraft::chain::network(&crate::settings::chain()))
                && let Some(ph) = s.first_puzzle_hash()
            {
                let address = chiacraft_wallet::chain::address(chiacraft_wallet::chain::Bytes32::from(ph), net);
                let _ = std::fs::write(s.home.join(format!("address-{}.txt", net.name())), format!("{address}\n"));
            }
        }
        let icon = crate::assets::decode_png_file(&core_assets.join("icon.png"))
            .and_then(|(w, h, px)| winit::window::Icon::from_rgba(px, w, h).ok());
        let mut attrs = Window::default_attributes()
            .with_title("Weft")
            .with_window_icon(icon)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.opts.size.0, self.opts.size.1));
        // Automated runs work in the background: no focus taken, under other windows, off the
        // desktop (still rendered at full size, so screenshots are the same).
        if recording() {
            // Fixed size and place, for a screen recorder to crop to.
            attrs = attrs
                .with_decorations(false)
                .with_window_level(winit::window::WindowLevel::AlwaysOnTop)
                .with_inner_size(winit::dpi::PhysicalSize::new(2560, 1360))
                .with_position(winit::dpi::PhysicalPosition::new(0, 30));
        } else if self.opts.automated {
            attrs = attrs
                .with_active(false)
                .with_window_level(winit::window::WindowLevel::AlwaysOnBottom)
                .with_position(winit::dpi::PhysicalPosition::new(-32000, -32000));
        }
        let window = Arc::new(el.create_window(attrs).map_err(|e| e.to_string())?);
        let size = window.inner_size();
        let desc = BackendDesc {
            window: &*window,
            width: size.width,
            height: size.height,
            vsync: crate::settings::vsync(),
            debug: self.opts.gpu_debug,
            upscale_standin: crate::settings::upscale_standin(),
            cache: Some(&self.opts.cache_dir),
        };
        // The chosen renderer, and the others if it can't start on this machine.
        let mut chosen = crate::settings::renderer();
        if chosen == "vulkan" && self.opts.renderer.is_none() && !self.opts.automated && crate::diag::vulkan_startup_faulted() {
            log::warn!("Vulkan crashed during startup on a previous run; trying wgpu first");
            chosen = "wgpu".into();
        }
        let mut order: Vec<_> = self.renderers.iter().collect();
        order.sort_by_key(|r| r.0 != chosen);
        let mut failed = Vec::new();
        let mut gpu = None;
        for &(name, factory) in order {
            crate::diag::renderer_starting(name == "vulkan" && !self.opts.automated);
            log::info!("starting renderer {name}");
            match factory(&desc) {
                Ok(g) => {
                    gpu = Some((g, name));
                    break;
                }
                Err(e) => {
                    crate::diag::renderer_starting(false);
                    log::warn!("renderer {name}: {e}");
                    failed.push(format!("{name}: {e}"));
                }
            }
        }
        let (mut gpu, renderer) = gpu.ok_or_else(|| failed.join("; "))?;

        let mut roots: Vec<(String, PathBuf)> = vec![("core".into(), core_assets)];
        // The content's mods, data packages among them (chiacraft): the code mods alone left a
        // package's lang and textures out until the content was built again.
        roots.extend(
            content()
                .mods
                .iter()
                .map(|m| (m.id.clone(), m.assets.clone())),
        );
        let packs_now = (crate::settings::resource_packs(), crate::settings::shader_pack());
        let packs = crate::packs::enabled(&self.opts.run_dir, &crate::settings::resource_packs());
        crate::packs::set_shader_pack(
            crate::settings::shader_pack()
                .as_ref()
                .map(|p| crate::packs::shader_dir(&self.opts.run_dir).join(p)),
        );
        let assets = Assets::new(roots, packs);
        crate::assets::set_current(&assets);
        let lang = Lang::load(&assets, &crate::settings::language(), self.opts.dev);
        let look = Look::new(&assets, &mod_ids(), &lang);
        let textures = GameTextures::new(&mut *gpu, content(), &assets);
        crate::client::terrain_gpu::GpuTerrain::precompile(&mut *gpu, &assets);
        let ui = UiRenderer::new(&mut *gpu, &assets)?;
        // The HDR world's format, before any pipeline drawing into the world is made.
        crate::post::set_hdr(crate::settings::hdr_world());
        // And whether they draw the motion image too, for temporal anti-aliasing.
        // The world draws the motion image for temporal anti-aliasing with it (`core:aa` 1, not 2,
        // from the camera alone) and for an upscaler (DLSS): with one, always, so `core:dlss` can be
        // switched on while playing.
        crate::post::set_motion(crate::settings::aa_motion() || gpu.upscaler().modes != 0);
        crate::post::set_taa(crate::settings::aa());
        let post = crate::post::Post::new(&mut *gpu, &assets);
        let mut input = Input::default();
        for k in &content().game.keys {
            input.register(&k.name, k.default, k.contexts);
            input.binds.last_mut().unwrap().while_held = k.while_held.clone();
        }
        crate::settings::apply_binds(&mut input);
        settings.read_only = self.opts.automated;
        crate::shaders::pin_time(self.opts.automated && !recording());
        let depth = make_depth(&mut *gpu, size.width, size.height);
        let gpu_name = format!("{} ({})", gpu.device_name(), gpu.name());
        // What the GPU is, for mods (`core:gpu_*`): as found, whatever a file says.
        for (name, v) in [
            ("core:gpu_class", gpu.device_class() as f64),
            ("core:gpu_memory_mb", gpu.device_memory_mb() as f64),
            ("core:gpu_ray_query", gpu.ray_query() as u8 as f64),
        ] {
            if let Some(id) = crate::config::by_name(name) {
                crate::config::put(id, v);
            }
        }
        if let Some(id) = crate::config::by_name("core:gpu_name") {
            crate::config::put_text(id, gpu.device_name());
        }
        let upscaler = gpu.upscaler();
        // What it runs, for the Video menu's DLSS tab (`core:gpu_dlss*`), as found.
        for (name, v) in [
            ("core:gpu_dlss_modes", upscaler.modes as f64),
            ("core:gpu_dlss_models", weft_render::UpscalerInfo::model_bits(&upscaler.models) as f64),
            ("core:gpu_dlss_rr", upscaler.reconstruct as u8 as f64),
            ("core:gpu_dlss_rr_models", weft_render::UpscalerInfo::model_bits(&upscaler.rr_models) as f64),
            ("core:gpu_dlss5", upscaler.neural as u8 as f64),
            ("core:gpu_frame_gen", upscaler.frame_gen as f64),
        ] {
            if let Some(id) = crate::config::by_name(name) {
                crate::config::put(id, v);
            }
        }
        if let Some(id) = crate::config::by_name("core:gpu_dlss") {
            crate::config::put_text(id, &upscaler.name);
        }
        if !upscaler.name.is_empty() {
            log::info!("upscaler: {} (core:dlss picks the mode)", upscaler.name);
        }
        let mut r = Running {
            upscaler,
            upscale_now: None,
            upscale_logged: None,
            upscale_size: None,
            gpu,
            cache_dir: self.opts.cache_dir.clone(),
            renderer,
            window,
            depth,
            list: CommandList::default(),
            last_button: None,
            world_list: CommandList::default(),
            after_list: CommandList::default(),
            input,
            touches: Default::default(),
            keyboard: false,
            settings,
            lang,
            languages: Vec::new(),
            look,
            assets,
            textures,
            ui,
            post,
            audio: crate::audio::Audio::new().map(|mut a| {
                a.muted = self.opts.automated;
                a
            }),
            screens: Vec::new(),
            hud: {
                let mut h = Hud::default();
                h.load_layout(&self.opts.run_dir);
                crate::touch::load(&self.opts.run_dir);
                h
            },
            game: None,
            joining: None,
            join_offer: None,
            home: None,
            join_address: None,
            links_seen: 0,
            travel: None,
            remote_address: None,
            came_from: None,
            retired: None,
            last: Instant::now(),
            shot_pending: false,
            fps: 0.0,
            frame_ms: 0.0,
            fps_timer: Instant::now(),
            fps_frames: 0,
            quads_dirty: true,
            debug: false,
            quit: false,
            mods,
            client_hash: 0,
            ui_textures: 0,
            stats: Stats {
                gpu_name: gpu_name.clone(),
                cpu_name: crate::diag::cpu_name(),
                memory_total: crate::diag::total_memory(),
                threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
                ..Default::default()
            },
            hud_off: false,
            world_dir: None,
            preview_shot: None,
            entering: None,
            opening_world: None,
            hud_since: None,
            hud_quads: 0..0,
            reload_requested: false,
            reload_held: None,
            packs_pending: false,
            applied: Applied { packs: packs_now, ..Default::default() },
            config_seen: u64::MAX,
            prof: [0.0; 5],
            bench: None,
            diag: Default::default(),
            ui_cached: Vec::new(),
            ui_animated: Vec::new(),
            previews: Vec::new(),
            preview_tex: None,
            menu_entities: None,
            menu_preview_failed: false,
            shot_path: None,
            shot_image: None,
            mod_held: Vec::new(),
            mod_gizmos: Vec::new(),
            ui_sounds: Vec::new(),
            mod_screen_text: Vec::new(),
            frame_no: 0,
            frame_dt: 0.0,
            screen_seen: (0, String::new()),
            screen_frames: 0,
            benches: 0,
            bench_last: [0.0; 5],
            bench_line: String::new(),
        };
        if let Some(world) = &self.opts.world {
            let saves_dir = r.settings.saves_dir();
            let meta = saves::list(&saves_dir)
                .into_iter()
                .find(|w| &w.name == world)
                .unwrap_or_else(|| saves::create(&saves_dir, world, self.opts.seed.unwrap_or(1)));
            r.start_world(meta);
        } else {
            r.open(screens::engine("core:title"));
            if let Some(name) = &self.opts.screen {
                match screens::named(name) {
                    Some(s) => r.open(s),
                    None => log::warn!("--screen: no screen {name}"),
                }
            } else if !crate::modloader::newly_disabled().is_empty() {
                r.open(screens::engine("core:disabled_mods"));
            }
        }
        Ok(r)
    }
}

/// The frame the world is drawn in: the screen's, or smaller (`core:render_scale`) to be
/// stretched over it.
fn world_frame(r: &Running, info: &weft_render::FrameInfo) -> weft_render::FrameInfo {
    let scale = if r.game.is_some() && r.post.stretches() { crate::settings::render_scale().clamp(0.25, 1.0) } else { 1.0 };
    let size = |n: u32| ((n as f32 * scale).round() as u32).max(1);
    weft_render::FrameInfo { width: size(info.width), height: size(info.height), ..*info }
}

/// With DLSS on (`core:dlss`, a mode the backend runs): the size it wants
/// the world drawn at, and this frame goes to it (`Running::upscale_now`, with Ray Reconstruction,
/// the models and DLSS 5 as set, where it runs them). None: the world as always.
fn upscale_frame(r: &mut Running, info: &weft_render::FrameInfo) -> Option<weft_render::FrameInfo> {
    use weft_render::UpscaleMode;
    r.upscale_now = None;
    let mode = crate::settings::dlss();
    if mode == UpscaleMode::Off || r.game.is_none() || !r.post.can_upscale() {
        return None;
    }
    // Each thing asked for that doesn't run here, said once.
    let say = |what: &str, why: &str| {
        static SAID: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        let mut said = SAID.lock().unwrap();
        if !said.iter().any(|s| s == what) {
            log::warn!("{what} is off: {why}");
            said.push(what.to_string());
        }
    };
    if !r.upscaler.has(mode) {
        say(
            &format!("core:dlss {} ({})", mode as u32, mode.name()),
            "the renderer has no upscaler for it (DLSS: the streamline build on Windows, with its files)",
        );
        return None;
    }
    let reconstruct = crate::settings::dlss_rr() && {
        if !r.upscaler.reconstruct {
            say("core:dlss_rr", "the upscaler has no Ray Reconstruction here");
        }
        r.upscaler.reconstruct
    };
    let neural = crate::settings::dlss5() && {
        if !r.upscaler.neural {
            say("core:dlss5", "DLSS 5 needs NVIDIA's neural rendering plugin, not in the public Streamline SDK");
        }
        r.upscaler.neural
    };
    // A model the upscaler doesn't list is its default.
    let known = |id: u32, list: &[weft_render::UpscaleModel]| if list.iter().any(|m| m.id == id) { id } else { 0 };
    let choice = UpscaleChoice {
        mode,
        reconstruct,
        model: known(crate::settings::dlss_model(mode), &r.upscaler.models),
        rr_model: known(crate::settings::dlss_rr_model(), &r.upscaler.rr_models),
        neural,
    };
    let screen = (info.width, info.height);
    let render = match r.upscale_size {
        Some((m, rr, s, w)) if m == mode && rr == reconstruct && s == screen => w,
        _ => {
            let w = r.gpu.upscale_render_size(mode, reconstruct, screen);
            log::info!(
                "upscaler {}{}: world {}x{} for the screen's {}x{}",
                mode.name(),
                if reconstruct { " with ray reconstruction" } else { "" },
                w.0,
                w.1,
                screen.0,
                screen.1
            );
            r.upscale_size = Some((mode, reconstruct, screen, w));
            w
        }
    };
    // Logged when it changes (`upscale_now` is cleared each frame, so the last one is kept apart).
    if r.upscale_logged != Some(choice) {
        log::debug!("upscaler: {choice:?}");
        r.upscale_logged = Some(choice);
    }
    r.upscale_now = Some(choice);
    // DLSS's advice: 8 jitter points times the scale squared.
    let ratio = (screen.0 as f32 / render.0.max(1) as f32).max(1.0);
    crate::post::set_jitter_phases((8.0 * ratio * ratio).round() as u32);
    Some(weft_render::FrameInfo { width: render.0, height: render.1, ..*info })
}

fn make_depth(gpu: &mut dyn Backend, w: u32, h: u32) -> (TextureId, u32, u32) {
    let t = gpu
        .create_texture(&TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: Format::Depth32, render_target: true }, "depth");
    (t, w, h)
}

/// A screen taller than the window scrolls (its root, with the wheel and a bar), as in a small
/// window at GUI scale 2 (Playtest B9: at 640 x 360 the animal screen's tabs and bottom were cut
/// off); it stays centred while it fits.
fn scroll_when_taller(ui: &mut Ui) {
    ui.nodes[0].style.scroll = true;
}

impl Running {
    fn gui_scale(&self) -> f32 {
        let max = crate::settings::max_gui_scale(self.gpu.surface_size());
        crate::settings::applied_gui_scale(crate::settings::gui_scale(), max) as f32
    }

    fn build_cx(&self) -> BuildCx<'_> {
        BuildCx {
            lang: &self.lang,
            input: &self.input,
            settings: &self.settings,
            languages: &self.languages,
            in_game: self.game.is_some(),
            debug: self.debug,
            remote: self.game.as_ref().is_some_and(|g| g.server.is_none()),
            home: self.home.as_ref().map(|h| h.world.as_str()),
            mods: self.mods.len(),
            client: self.game.as_ref().map(|g| &g.client),
            size: {
                let (w, h) = self.gpu.surface_size();
                glam::Vec2::new(w as f32, h as f32) / self.gui_scale()
            },
            look: &self.look,
            assets: &self.assets,
            renderer: self.renderer,
        }
    }

    fn open(&mut self, mut s: Box<dyn Screen>) {
        if s.needs_languages() && self.languages.is_empty() {
            self.languages = Lang::available(&self.assets);
        }
        let mut ui = Ui::new();
        let depth = self.screens.len() + 1;
        self.build_screen(&mut *s, &mut ui, depth);
        self.screens.push((s, ui));
        self.quads_dirty = true;
        self.set_capture(false);
    }

    /// A screen's tree, with a fallback close icon when it has no control to leave it.
    fn build_screen(&self, s: &mut dyn Screen, ui: &mut Ui, depth: usize) {
        s.build(ui, &self.build_cx());
        scroll_when_taller(ui);
        if s.escape_closes() && !ui.has_close && (depth > 1 || self.game.is_some()) {
            ui.container(Style::default().at_from([1.0, 0.0], -4.0, 4.0, [1.0, 0.0]), |ui| screens::close_button(ui, 16.0));
        }
    }

    fn rebuild_all(&mut self) {
        self.rebuild_screens(|_| true);
        self.hud.invalidate();
    }

    fn set_debug(&mut self, on: bool) {
        if self.debug != on {
            self.debug = on;
            self.hud.invalidate();
            self.rebuild_screens(|ui| ui.find("hud_tools").is_some());
        }
    }

    fn rebuild_music_screens(&mut self) {
        if self.screens.iter().any(|(_, ui)| ui.read_music) {
            self.rebuild_screens(|ui| ui.read_music);
        }
    }

    fn rebuild_screens(&mut self, rebuild: impl Fn(&Ui) -> bool) {
        let mut screens = std::mem::take(&mut self.screens);
        for (depth, (s, ui)) in screens.iter_mut().enumerate() {
            if !rebuild(ui) {
                continue;
            }
            let (hover, focus) = (ui.hovered, ui.focused);
            let focus_id = focus
                .and_then(|f| ui.nodes.get(f))
                .and_then(|n| n.id.clone());
            let inputs: Vec<(String, String, usize)> = ui
                .nodes
                .iter()
                .filter_map(|n| match (&n.id, &n.kind) {
                    (Some(id), Kind::Input { value, caret, .. }) => Some((id.clone(), value.clone(), *caret)),
                    _ => None,
                })
                .collect();
            ui.clear();
            self.build_screen(&mut **s, ui, depth + 1);
            // Typed text survives rebuilds (language switch, resize).
            for (id, v, c) in inputs {
                if let Some(n) = ui.find(&id) {
                    if let Kind::Input { value, caret, .. } = &mut ui.nodes[n].kind {
                        *value = v;
                        *caret = c;
                    }
                }
            }
            ui.hovered = hover.filter(|&h| h < ui.nodes.len());
            // Focus follows the same input (by id), or whatever the screen focused itself.
            if let Some(id) = focus_id {
                ui.focused = ui.find(&id).or(ui.focused);
            } else if ui.focused.is_none() {
                ui.focused = focus.filter(|&h| h < ui.nodes.len());
            }
        }
        self.screens = screens;
        self.quads_dirty = true;
    }

    fn set_capture(&mut self, on: bool) {
        let on = on && self.screens.is_empty() && self.game.is_some();
        // Automated runs (read-only settings) never take the real mouse: they run in the background.
        if recording() {
            self.window.set_cursor_visible(false);
        } else if self.settings.read_only {
        } else if on {
            // Into the middle first, like vanilla: a cursor left by the edge (or outside, after an
            // alt-tab) is no cursor kept in, where Windows only confines it.
            self.center_cursor();
            self.grab_cursor();
            self.window.set_cursor_visible(false);
        } else {
            let _ = self.window.set_cursor_grab(CursorGrabMode::None);
            // A screen opening from play starts with the pointer in the middle, as vanilla's.
            if self.input.captured {
                self.center_cursor();
            }
            self.window.set_cursor_visible(true);
        }
        self.input.captured = on;
    }

    /// Reads every asset again (resource packs, textures, sounds, languages, the look) and
    /// redraws.
    fn reload_assets(&mut self) {
        let run_dir = self.settings.run_dir.clone();
        self.menu_preview_failed = false;
        if let Some(mut entities) = self.menu_entities.take() {
            entities.destroy(&mut *self.gpu);
            self.preview_tex = None;
        }
        self.assets
            .set_packs(crate::packs::enabled(&run_dir, &crate::settings::resource_packs()));
        crate::packs::set_shader_pack(
            crate::settings::shader_pack()
                .as_ref()
                .map(|p| crate::packs::shader_dir(&run_dir).join(p)),
        );
        self.assets.rescan();
        crate::assets::set_current(&self.assets);
        self.hud.load_layout(&run_dir);
        self.ui.reload(&mut *self.gpu, &self.assets);
        self.post.reload(&mut *self.gpu, &self.assets);
        self.lang = Lang::load(&self.assets, &crate::settings::language(), self.lang.dev);
        self.look = Look::new(&self.assets, &mod_ids(), &self.lang);
        self.textures
            .reload(&mut *self.gpu, content(), &self.assets);
        if let Some(a) = &mut self.audio {
            a.reload();
        }
        if let Some(g) = &mut self.game {
            g.client
                .render
                .reload_shaders(&mut *self.gpu, &self.textures, &self.assets);
            g.client.render.set_textures(&mut *self.gpu, &self.textures);
            g.client.entity_render.reload(&mut *self.gpu, &self.assets);
            // Slots may show pictures that are gone now.
            self.quads_dirty = true;
            g.client.particles.reload(&mut *self.gpu, &self.assets);
            g.client.wires.reload(&mut *self.gpu, &self.assets);
            g.client.weather_render.reload(&mut *self.gpu, &self.assets);
            g.client.wake_render.reload(&mut *self.gpu, &self.assets);
            g.client.sky.reload(&mut *self.gpu, &self.assets);
            g.client.world.remesh_all();
            g.client
                .entities
                .values_mut()
                .filter_map(|e| e.structure.as_mut())
                .for_each(|s| s.remesh_all());
        }
        self.rebuild_all();
        log::info!("assets reloaded");
    }

    /// After the content was built for another set of data packages (no world loaded): their
    /// assets, and everything drawn from the content, again.
    fn content_changed(&mut self) {
        let mut roots: Vec<(String, PathBuf)> = vec![("core".into(), engine_assets(self.lang.dev))];
        roots.extend(
            content()
                .mods
                .iter()
                .map(|m| (m.id.clone(), m.assets.clone())),
        );
        self.assets.set_mods(roots);
        self.reload_assets();
        // Keys the new content brings (a package's client half, chiacraft phase 6): known to the
        // input as the start's are, with the player's binding if they set one.
        for k in &content().game.keys {
            self.input.register(&k.name, k.default, k.contexts);
            self.input.binds.last_mut().unwrap().while_held = k.while_held.clone();
        }
        crate::settings::apply_binds(&mut self.input);
    }

    /// The join in progress: answers what it reports, until the game is let in or it fails.
    fn poll_join(&mut self) {
        use net::p2p::{JoinAnswer, JoinEvent};
        let Some(j) = &self.joining else { return };
        let events: Vec<JoinEvent> = j.events.try_iter().collect();
        for ev in events {
            let answer = |app: &Self, a: JoinAnswer| {
                if let Some(j) = &app.joining {
                    let _ = j.answers.send(a);
                }
            };
            match ev {
                JoinEvent::Offer(all, missing) => {
                    let trust = crate::package_trust::Trust::load(&self.settings.run_dir);
                    let rows: Vec<net::screens::OfferRow> = missing
                        .iter()
                        .map(|o| {
                            let (creator, problem) = match (!o.4.is_empty()).then(|| crate::package_trust::verify(&o.4, &o.2)) {
                                None => (None, None),
                                Some(Ok(c)) => (Some(c), None),
                                Some(Err(e)) => (None, Some(e)),
                            };
                            let trusted = trust.trusts(&o.2, creator.as_ref());
                            net::screens::OfferRow {
                                id: o.0.clone(),
                                version: o.1.clone(),
                                hash: o.2.clone(),
                                size: o.3,
                                creator,
                                problem,
                                trusted,
                                api: o.5.clone(),
                            }
                        })
                        .collect();
                    let bytes: u64 = missing.iter().map(|o| o.3).sum();
                    if rows.iter().all(|r| r.trusted) {
                        log::info!(
                            "the world plays with {} data packages; downloading {} ({} KB), all trusted",
                            all.len(),
                            missing.len(),
                            bytes.div_ceil(1024)
                        );
                        net::screens::set_join_status(
                            "multiplayer.downloading",
                            &[missing.len().to_string(), "0".into(), bytes.div_ceil(1024).to_string()],
                        );
                        answer(self, JoinAnswer::Accept);
                    } else {
                        log::info!(
                            "the world plays with {} data packages; asking before downloading {} ({} KB)",
                            all.len(),
                            missing.len(),
                            bytes.div_ceil(1024)
                        );
                        self.join_offer = Some(rows.clone());
                        self.screens.clear();
                        self.open(Box::new(net::screens::JoinSheet { rows }));
                    }
                }
                JoinEvent::Progress(done, total) => {
                    // The package count stays as the offer set it.
                    let count = net::screens::JOIN_STATUS
                        .lock()
                        .1
                        .first()
                        .cloned()
                        .unwrap_or_default();
                    net::screens::set_join_status(
                        "multiplayer.downloading",
                        &[count, done.div_ceil(1024).to_string(), total.div_ceil(1024).to_string()],
                    );
                }
                JoinEvent::Packages(hashes) => {
                    // A trip leaves the world it set out from only now: the destination answered
                    // and its packages are here, and loading them changes what the game shows.
                    if let Some(t) = self.travel.as_mut().filter(|t| !t.left) {
                        t.left = true;
                        self.leave_for_another();
                    }
                    net::screens::set_join_status("multiplayer.loading_packages", &[]);
                    let set: Result<Vec<modloader::PackageRef>, String> = hashes
                        .iter()
                        .map(|h| {
                            chiacraft_packages::identity::Hash::from_hex(h)
                                .ok_or_else(|| "a bad package hash".to_string())
                                .and_then(modloader::stored_package)
                        })
                        .collect();
                    match set.and_then(|s| modloader::use_packages(&s)) {
                        Ok(changed) => {
                            if changed {
                                self.content_changed();
                            }
                            answer(self, JoinAnswer::Ready(net::p2p::content_hash()));
                        }
                        Err(e) => answer(self, JoinAnswer::Cancel(e)),
                    }
                }
                JoinEvent::Admitted(conn) => {
                    self.joining = None;
                    self.travel = None;
                    if let Some(a) = self.join_address.take() {
                        crate::saved_worlds::remember(&self.settings.run_dir, &a);
                        // The address without a travel ticket: joining it again goes in as usual.
                        self.remote_address = crate::world_address::parse(&a)
                            .map(|p| crate::world_address::address(&p.key, &p.name, Some(&p.relay), &p.addrs));
                    }
                    self.enter_remote(conn);
                    return;
                }
                JoinEvent::Failed(e) => {
                    log::warn!("joining failed: {e}");
                    self.joining = None;
                    self.join_offer = None;
                    // A trip that never left: the player is still where they were.
                    if self.travel.take().is_some_and(|t| !t.left) {
                        self.screens.clear();
                        self.open(Box::new(screens::Message { title: "travel.failed".into(), args: vec![Arg::Text(e)] }));
                        return;
                    }
                    self.return_home();
                    self.screens.clear();
                    // A world's refusal at the door (chiacraft, `admission`) in this player's language.
                    let why = crate::admission::refusal_from_text(&e).map_or(Arg::Text(e), Arg::Msg);
                    self.open(Box::new(screens::Message { title: "multiplayer.disconnected".into(), args: vec![why] }));
                    return;
                }
            }
        }
    }

    /// Plays in a friend's world over `conn`.
    fn enter_remote(&mut self, mut conn: net::ClientConn) {
        let key = net::p2p::player_key(&self.settings.run_dir);
        let view = crate::settings::view_distance();
        let skin = read_skin(&self.settings.run_dir);
        let avatar = read_avatar(&self.settings.run_dir);
        Client::send_join(&mut conn, crate::settings::name(), key, view, skin.as_ref(), avatar);
        match Client::new(&mut *self.gpu, &self.assets, &self.textures, &self.cache_dir, conn, view, skin) {
            Ok(mut client) => {
                client.sent = load_chat_history(&self.settings.run_dir);
                self.game = Some(Game { client, server: None, world: String::new(), identity: None });
                self.screens.clear();
                self.hud.invalidate();
                self.open(Box::new(screens::LoadingWorld::Streaming));
            }
            Err(e) => log::error!("{e}"),
        }
    }

    /// A finger on a touch screen (`touch`).
    fn touch(&mut self, t: winit::event::Touch) {
        // Off, a computer's touch screen is the system's mouse (Windows makes one of it); a phone
        // has only fingers, which then still work the menus, the setting among them.
        let controls = crate::settings::touch_controls();
        if !controls && !cfg!(target_os = "android") {
            return;
        }
        let world = controls && self.game.is_some() && self.screens.is_empty();
        if world && !self.input.captured && t.phase == winit::event::TouchPhase::Started {
            self.set_capture(true);
        }
        let at = Vec2::new(t.location.x as f32, t.location.y as f32);
        let (scale, aim) = (self.gui_scale(), self.aim());
        let hud = &self.hud;
        let stick = hud.element_rect(crate::touch::STICK);
        let element = |p| {
            hud.element_at(p)
                .and_then(|e| hud.element_rect(&e).map(|r| (e, r)))
        };
        self.touches
            .finger(&mut self.input, t.id, t.phase, at, scale, world, aim, &element, stick);
    }

    /// What the crosshair is on, for what a finger on the view does.
    fn aim(&self) -> crate::touch::Aim {
        let Some(g) = &self.game else { return Default::default() };
        let held = [g.client.shown_held(), g.client.off_held()];
        crate::touch::Aim {
            entity: g.client.target_entity.is_some(),
            block: g.client.target.is_some(),
            usable: g
                .client
                .target
                .is_some_and(|h| content().block_of(h.state).usable()),
            holding_item: held
                .iter()
                .any(|s| crate::content::use_ticks(content(), s) > 0),
        }
    }

    fn grab_cursor(&self) {
        let _ = self
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));
    }

    fn center_cursor(&self) {
        let size = self.window.inner_size();
        let middle = winit::dpi::PhysicalPosition::new(size.width as f64 / 2.0, size.height as f64 / 2.0);
        let _ = self.window.set_cursor_position(middle);
    }

    fn start_world(&mut self, mut meta: WorldMeta) {
        if self.opening_world.is_some() {
            return;
        }
        log::info!("opening world '{}' (seed {})", meta.name, meta.seed);
        if meta.format != saves::SAVE_FORMAT {
            log::error!("world '{}' was saved in format {}, this build reads {}", meta.name, meta.format, saves::SAVE_FORMAT);
            self.entering = None;
            self.screens.clear();
            self.open(Box::new(screens::Message { title: "world.open_failed".into(), args: vec![Arg::Key("world.old_format".into())] }));
            return;
        }
        // The world's data packages (chiacraft): content is built again for another set.
        let (packages, missing) = modloader::for_world(&meta);
        match modloader::use_packages(&packages) {
            Ok(true) => self.content_changed(),
            Ok(false) => {}
            Err(e) => {
                log::error!("{e}");
                self.screens.clear();
                self.open(Box::new(screens::Message { title: "world.open_failed".into(), args: vec![Arg::Text(e)] }));
                return;
            }
        }
        for m in meta.missing_mods(content()).into_iter().chain(missing) {
            log::warn!("the world was played with {m}, which is not loaded");
        }
        // The world's own key (chiacraft): made now for a world that has none yet, and saved
        // with it before the server takes over the world's info.
        let identity = match crate::world_address::for_world(&mut meta) {
            Ok(id) => Some(Arc::new(id)),
            Err(e) => {
                log::warn!("'{}' has no address of its own: {e}", meta.name);
                None
            }
        };
        // chiacraft, test only: `--import-mc` loads a Minecraft world into a new world first.
        crate::chiacraft::mc_import::maybe_import(&mut meta, content());
        if self.gpu.compiling() && self.entering.is_none() {
            log::info!("waiting for shaders to compile");
            self.screens.clear();
            self.open(Box::new(screens::LoadingWorld::Compiling));
        }
        let run_dir = self.settings.run_dir.clone();
        let (send, opened) = crossbeam_channel::bounded(1);
        crate::background().spawn(move || {
            let storage = saves::Storage::open(meta, content());
            let world = OpenedWorld {
                storage,
                skin: read_skin(&run_dir),
                avatar: read_avatar(&run_dir),
                chat_history: load_chat_history(&run_dir),
                identity,
            };
            let _ = send.send(world);
        });
        self.opening_world = Some(OpeningWorld::Reading(opened));
    }

    fn stream_world(&mut self, world: OpenedWorld) {
        let OpenedWorld { storage, skin, avatar, chat_history, identity } = world;
        let seed = storage.meta.seed;
        let name = storage.meta.name.clone();
        let view = crate::settings::view_distance();
        let preload_view = if self.entering.is_some() || self.gpu.compiling() { view.min(NEAR + 2) } else { view };
        let server = server::spawn(
            ServerConfig { seed, view_radius: preload_view, dedicated: false, whitelist: None, identity: identity.clone() },
            Some(storage),
        );
        self.play_local(server, name, identity, preload_view, skin, avatar, chat_history);
    }

    /// A listed world is there to be found: while it is open it is hosted under its own key, the
    /// one its listing names, without its owner opening it to others on Multiplayer. Otherwise
    /// every finder saw it offline (finder3, 11-04).
    fn host_if_listed(&mut self) {
        if net::p2p::HOSTING.lock().is_some() {
            return;
        }
        if let Some(Game { server: Some(s), identity: Some(id), world, .. }) = &self.game
            && s.dir
                .as_deref()
                .is_some_and(|d| crate::chiacraft::worlds::listing::load(d).is_some())
        {
            log::info!("'{world}' is listed: hosting it under its own key {}", id.key());
            net::host(s, id.clone(), world, &self.settings);
        }
    }

    /// Plays in the world `server` runs in this process: joins it now, and `finish_world_open`
    /// makes the client once the view can be drawn.
    #[allow(clippy::too_many_arguments)]
    fn play_local(
        &mut self,
        server: ServerHandle,
        world: String,
        identity: Option<Arc<crate::world_address::WorldIdentity>>,
        preload_view: i32,
        skin: Option<net::Skin>,
        avatar: Option<net::AvatarData>,
        chat_history: Vec<String>,
    ) {
        crate::admission::set_local_world(server.dir.clone(), identity.clone());
        let dir = server.dir.clone().unwrap_or_default();
        let view = crate::settings::view_distance();
        let (mut conn, server_conn) = net::local_pair();
        let _ = server.connect.send(server_conn);
        let key = net::p2p::player_key(&self.settings.run_dir);
        Client::send_join(&mut conn, crate::settings::name(), key, preload_view, skin.as_ref(), avatar);
        self.opening_world = Some(OpeningWorld::Streaming { conn, server, dir, view, skin, chat_history, world, identity });
    }

    fn finish_world_open(&mut self) {
        if let Some(OpeningWorld::Reading(opened)) = &self.opening_world {
            match opened.try_recv() {
                Ok(world) => self.stream_world(world),
                Err(crossbeam_channel::TryRecvError::Empty) => return,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.opening_world = None;
                    self.world_open_failed("world loader stopped".into());
                    return;
                }
            }
        }
        // The server can stream ahead while the preview grows. GPU resources
        // still belong to the render thread, behind the full-screen preview.
        if self.entering.as_ref().is_some_and(|e| !e.expanded) || self.gpu.compiling() {
            return;
        }
        let Some(OpeningWorld::Streaming { mut conn, server, dir, view, skin, chat_history, world, identity }) = self.opening_world.take()
        else {
            return;
        };
        conn.send(net::ToServer::View(view as u32));
        match Client::new(&mut *self.gpu, &self.assets, &self.textures, &self.cache_dir, conn, view, skin) {
            Ok(mut client) => {
                client.sent = chat_history;
                self.world_dir = Some(dir);
                // The synced settings the player changes here are this world's.
                crate::config::playing_in(Some(server.configs.clone()));
                self.game = Some(Game { client, server: Some(server), world, identity });
                self.host_if_listed();
                self.hud.invalidate();
                self.quads_dirty = true;
                if self.entering.is_none() {
                    self.screens.clear();
                    self.open(Box::new(screens::LoadingWorld::Streaming));
                }
            }
            Err(e) => self.world_open_failed(e),
        }
    }

    fn world_open_failed(&mut self, reason: String) {
        log::error!("{reason}");
        self.entering = None;
        self.screens.clear();
        self.open(Box::new(screens::Message { title: "world.open_failed".into(), args: vec![Arg::Text(reason)] }));
    }

    /// Opens the pause screen over play. In a world of this machine the frame first goes without
    /// the HUD and is kept as the world's picture in the list: the last it was seen.
    fn pause(&mut self) {
        match self.world_dir.clone() {
            Some(dir) if !self.shot_pending && self.preview_shot.is_none() => {
                self.preview_shot = Some((dir.join(saves::PREVIEW), self.hud_off));
                self.hud_off = true;
                self.quads_dirty = true;
                self.gpu.request_capture();
                self.shot_pending = true;
            }
            Some(_) => {}
            None => self.open(screens::engine("core:pause")),
        }
    }

    /// Leaves the world in play to join another. Your own world runs on while others play in
    /// it (it is hosted): it becomes `home`, to come back to. Otherwise it is saved and closed.
    fn leave_for_another(&mut self) {
        let hosted = matches!(&self.game, Some(Game { server: Some(s), .. }) if s.is_hosting());
        if !hosted {
            self.leave_world_silent();
            return;
        }
        let Some(mut g) = self.game.take() else { return };
        // Away from it: the visited world's rules stay out of it.
        crate::config::playing_in(None);
        g.client.leave();
        self.gpu.wait_idle();
        g.client.destroy(&mut *self.gpu);
        if let Some(server) = g.server.take() {
            // It may have been paused by the menu this was chosen from.
            server.set_paused(false);
            log::info!("'{}' keeps running while you are away", g.world);
            self.home = Some(Home { server, world: g.world, identity: g.identity });
        }
        self.set_capture(false);
    }

    /// Back to your own world, left running while you visited another.
    fn return_home(&mut self) {
        let Some(home) = self.home.take() else { return };
        self.leave_world_silent();
        if let Some(j) = self.joining.take() {
            let _ = j
                .answers
                .send(net::p2p::JoinAnswer::Cancel("went home".into()));
        }
        self.join_offer = None;
        log::info!("back to '{}'", home.world);
        if modloader::use_built(home.server.content) {
            self.content_changed();
        }
        let view = crate::settings::view_distance();
        let (skin, avatar) = (read_skin(&self.settings.run_dir), read_avatar(&self.settings.run_dir));
        let chat_history = load_chat_history(&self.settings.run_dir);
        self.play_local(home.server, home.world, home.identity, view, skin, avatar, chat_history);
    }

    /// Stops your own world if it was left running.
    fn close_home(&mut self) {
        if let Some(mut h) = self.home.take() {
            h.server.stop();
            net::p2p::stop_hosting();
        }
    }

    fn leave_world(&mut self) {
        self.leave_world_silent();
        self.close_home();
        // A finished play session: the title screen asks for the words after the first one.
        if !crate::config::automated() {
            if let Some(s) = chiacraft_account::current() {
                s.session_ended();
            }
        }
        self.screens.clear();
        self.open(screens::engine("core:title"));
    }

    fn leave_world_silent(&mut self) {
        self.opening_world = None;
        self.entering = None;
        self.world_dir = None;
        if let Some(mut g) = self.game.take() {
            crate::config::playing_in(None);
            if g.server.is_none() {
                // A remote server's synced settings were in force; back to our own.
                crate::config::left_server(&self.settings.run_dir);
                self.remote_address = None;
            }
            g.client.leave();
            self.gpu.wait_idle();
            g.client.destroy(&mut *self.gpu);
            if let Some(mut s) = g.server.take() {
                s.stop();
                net::p2p::stop_hosting();
            }
            self.retired = Some(g);
        }
        self.set_capture(false);
    }

    /// Screens just went: with none left it is back to the game, otherwise those left look again.
    /// The one place that decides that.
    fn screens_gone(&mut self) {
        self.quads_dirty = true;
        if self.screens.is_empty() {
            self.set_capture(true);
        } else {
            self.rebuild_all();
        }
    }

    /// Closes the screens `drop` picks.
    pub(crate) fn close_screens(&mut self, drop: impl Fn(&dyn Screen) -> bool) {
        self.screens.retain(|(s, _)| !drop(s.as_ref()));
        self.screens_gone();
    }

    /// Closes the top screen, and the menu it shows if it does.
    pub(crate) fn close_top(&mut self) {
        let mut requests = Vec::new();
        if let Some((screen, ui)) = self.screens.last_mut()
            && let Some(action) = screen.close_action()
        {
            let close = action(&mut Ctx {
                settings: &mut self.settings,
                client: self.game.as_mut().map(|g| &mut g.client),
                input: &self.input,
                requests: &mut requests,
                stats: &self.stats,
                debug: self.debug,
                top: Some(Top { screen: Some(&mut **screen), ui }),
            });
            if !close {
                self.handle(requests);
                return;
            }
        }
        if self.screens.last().is_some_and(|(s, _)| s.shows_menu()) {
            if let Some(g) = &mut self.game {
                g.client.close_menu();
            }
        }
        self.screens.pop();
        self.screens_gone();
        self.handle(requests);
    }

    pub(crate) fn handle(&mut self, reqs: Vec<Request>) {
        for r in reqs {
            match r {
                Request::Open(s) => self.open(s),
                Request::Close => self.close_top(),
                Request::Quit => self.quit = true,
                // A phone's app can't start itself again: it closes, to be opened again.
                Request::Restart if cfg!(target_os = "android") => self.quit = true,
                Request::Restart => {
                    match crate::paths::own_program().and_then(|exe| {
                        std::process::Command::new(exe)
                            .args(std::env::args().skip(1))
                            .spawn()
                    }) {
                        Ok(_) => self.quit = true,
                        Err(e) => log::error!("restarting: {e}"),
                    }
                }
                Request::Rebuild => self.rebuild_all(),
                Request::ResetHudLayout => {
                    self.hud.reset_all();
                    crate::touch::reset_sizes();
                }
                Request::SetDebug(on) => self.set_debug(on),
                Request::Sound(play) => self.ui_sounds.push(play),
                Request::ModCall(owner, cb, data) => {
                    let mut v = self.frame_view();
                    let mut a = weft_abi::UiArgs { ui: std::ptr::null_mut(), client: v.raw(), data };
                    crate::host::invoke(owner, cb, &mut a);
                }
                Request::ModChange(owner, cb, text, value, handle) => {
                    let mut v = self.frame_view();
                    let mut a = weft_abi::UiChangeArgs { client: v.raw(), text: weft_abi::Str::new(&text), value, handle };
                    crate::host::invoke(owner, cb, &mut a);
                }
                Request::ModKey(owner, cb, key) => {
                    let mut v = self.frame_view();
                    let mut a = weft_abi::ScreenKeyArgs { client: v.raw(), key: weft_abi::Str::new(&key) };
                    crate::host::invoke(owner, cb, &mut a);
                }
                Request::Screen(name) => match screens::named(&name) {
                    Some(s) => self.open(s),
                    None => log::warn!("no screen {name}"),
                },
                Request::Rebind(action, keys) => {
                    self.input.rebind(&action, keys);
                    crate::settings::set_bind(&action, &keys);
                }
                Request::ResetBinds => {
                    let defaults: Vec<(String, crate::input::Slots)> = self
                        .input
                        .binds
                        .iter()
                        .map(|b| (b.action.clone(), b.default))
                        .collect();
                    for (a, k) in defaults {
                        self.input.rebind(&a, k);
                        crate::settings::set_bind(&a, &k);
                    }
                    self.rebuild_all();
                }
                Request::StartWorld(meta) => {
                    let mut missing = meta.missing_mods(content());
                    missing.extend(modloader::for_world(&meta).1);
                    if missing.is_empty() {
                        self.start_world(meta);
                    } else {
                        self.open(Box::new(saves::MissingMods { world: meta, missing }));
                    }
                }
                Request::EnterWorld { world, picture, from } => {
                    let mut missing = world.missing_mods(content());
                    missing.extend(modloader::for_world(&world).1);
                    if self.entering.is_some() || self.game.is_some() || self.opening_world.is_some() {
                    } else if missing.is_empty() {
                        self.entering = Some(Entering { picture, from, started: Instant::now(), expanded: false, shown: None });
                        self.start_world(world);
                    } else {
                        self.open(Box::new(saves::MissingMods { world, missing }));
                    }
                }
                Request::StartWorldAnyway(meta) => self.start_world(meta),
                Request::LeaveWorld => self.leave_world(),
                Request::Host => {
                    match &self.game {
                        Some(Game { server: Some(s), identity: Some(id), world, .. }) => net::host(s, id.clone(), world, &self.settings),
                        Some(Game { server: Some(_), identity: None, .. }) => {
                            *net::p2p::HOSTING.lock() = Some(net::p2p::Hosting {
                                ticket: None,
                                error: Some("this world has no address: no account is open".into()),
                            });
                        }
                        _ => {}
                    }
                    self.rebuild_all();
                }
                Request::Respawn => {
                    if let Some(g) = &mut self.game {
                        g.client.send(net::ToServer::Respawn);
                        g.client.dead = false;
                    }
                    self.close_screens(|s| s.closes_on_respawn());
                }
                Request::SendChat(text) => {
                    if self.game.is_some() && !self.client_command(&text) && !text.trim().is_empty() {
                        if let Some(g) = &mut self.game {
                            g.client.send(net::ToServer::Chat(text));
                        }
                    }
                }
                Request::Join(ticket) => {
                    // Your own world, left running: going there is going home, not joining it.
                    let parsed = crate::world_address::parse(&ticket);
                    let key = parsed.as_ref().map(|p| p.key.clone());
                    if key.is_some()
                        && key
                            == self
                                .home
                                .as_ref()
                                .and_then(|h| h.identity.as_ref())
                                .map(|i| i.key())
                    {
                        // Through a portal: placed at home's portal for that link.
                        if let Some(t) = parsed.and_then(|p| crate::travel::Ticket::parse(&p.ticket)) {
                            crate::travel::expect(&net::p2p::player_key(&self.settings.run_dir), &t.link);
                        }
                        self.return_home();
                        continue;
                    }
                    // Through a portal: the player stays where they are until the trip is ready.
                    let portal = self.game.is_some() && crate::world_address::parse(&ticket).is_some_and(|p| !p.ticket.is_empty());
                    if portal {
                        self.came_from = self
                            .game
                            .as_ref()
                            .is_some_and(|g| g.server.is_none())
                            .then(|| self.remote_address.clone())
                            .flatten();
                        self.travel = Some(Travel { left: false });
                    } else {
                        self.came_from = None;
                        self.travel = None;
                        self.leave_for_another();
                    }
                    self.join_address = Some(ticket.clone());
                    self.joining = None;
                    match net::start_join(&ticket, &self.settings) {
                        Ok(j) => {
                            net::screens::set_join_status("multiplayer.connecting", &[]);
                            self.joining = Some(j);
                            if !portal {
                                self.screens.clear();
                                self.open(Box::new(net::screens::Joining));
                            }
                        }
                        Err(e) => {
                            self.open(Box::new(screens::Message { title: "multiplayer.disconnected".into(), args: vec![Arg::Text(e)] }))
                        }
                    }
                }
                Request::JoinConsent(trust_creators) => {
                    if let (Some(rows), Some(j)) = (self.join_offer.take(), &self.joining) {
                        let mut trust = crate::package_trust::Trust::load(&self.settings.run_dir);
                        for r in &rows {
                            trust.add_package(&r.hash);
                            if let (true, Some(c)) = (trust_creators, &r.creator) {
                                trust.add_creator(c);
                            }
                        }
                        trust.save();
                        let bytes: u64 = rows.iter().map(|r| r.size).sum();
                        net::screens::set_join_status(
                            "multiplayer.downloading",
                            &[rows.len().to_string(), "0".into(), bytes.div_ceil(1024).to_string()],
                        );
                        let _ = j.answers.send(net::p2p::JoinAnswer::Accept);
                        self.screens.clear();
                        self.open(Box::new(net::screens::Joining));
                    }
                }
                Request::ReturnHome => self.return_home(),
                Request::MakeInvite => {
                    if let Some(Game { server: Some(_), identity: Some(id), world, .. }) = &self.game {
                        let invite = crate::links::invite(id, world);
                        log::info!("invite made for '{world}': {invite}");
                        *crate::links::LAST_INVITE.lock() = invite;
                    }
                    self.rebuild_all();
                }
                Request::AskLink(address, note) => {
                    let result = match &self.game {
                        Some(Game { server: Some(_), identity: Some(id), world, .. }) => {
                            net::p2p::account_key(&self.settings).and_then(|k| crate::links::request(&k, id, world, &address, &note))
                        }
                        _ => Err("open one of your worlds to link it".into()),
                    };
                    if let Err(e) = result {
                        self.open(Box::new(screens::Message { title: "links.failed".into(), args: vec![Arg::Text(e)] }));
                    } else {
                        self.rebuild_all();
                    }
                }
                Request::CancelJoin => {
                    self.join_offer = None;
                    if let Some(j) = self.joining.take() {
                        let _ = j
                            .answers
                            .send(net::p2p::JoinAnswer::Cancel("cancelled".into()));
                    }
                    if self.home.is_some() {
                        self.return_home();
                        continue;
                    }
                    self.screens.clear();
                    self.open(screens::engine("core:title"));
                }
            }
        }
    }

    /// Mouse and keyboard for the top screen.
    fn screen_input(&mut self, scale: f32, requests: &mut Vec<Request>) {
        let cursor = self.input.cursor / scale;
        let menu_screen = self.screens.last().is_some_and(|(s, _)| s.shows_menu());
        let Some((screen, ui)) = self.screens.last_mut() else { return };
        let mut ctx = Ctx {
            settings: &mut self.settings,
            client: self.game.as_mut().map(|g| &mut g.client),
            input: &self.input,
            requests: &mut *requests,
            stats: &self.stats,
            debug: self.debug,
            top: None,
        };
        if screen.pointer(cursor, ui, &mut ctx, &mut self.hud) {
            self.quads_dirty = true;
            return;
        }
        // The wheel over a slot moves items (`menu_input`), not the panel behind it.
        let over_slot = menu_screen
            && ui
                .hit(cursor)
                .is_some_and(|h| matches!(ui.nodes[h].kind, Kind::Custom { tag: crate::client::modui::TAG_SLOT, .. }));
        if self.input.scroll() != 0.0 && !over_slot && ui.scroll_at(cursor, self.input.scroll()) {
            self.quads_dirty = true;
        }
        // A scroll bar takes the mouse before anything under it.
        let on_bar = if self.input.pressed_button(0) {
            ui.bar_press(cursor)
        } else if self.input.held_button(0) {
            ui.bar_drag(cursor)
        } else {
            ui.bar_release();
            false
        };
        if on_bar {
            self.quads_dirty = true;
            return;
        }
        let hovered = ui.hit(cursor);
        if hovered != ui.hovered {
            ui.hovered = hovered;
            self.quads_dirty = true;
        }
        if menu_screen {
            self.menu_input(hovered, cursor);
        }
        let Some((_, ui)) = self.screens.last_mut() else { return };
        if self.input.pressed_button(0) && menu_screen {
            let arrow = hovered.is_some_and(|h| ui.nodes[h].recipes);
            let kinds = self
                .game
                .as_ref()
                .and_then(|g| g.client.menu.as_ref())
                .and_then(|m| content().game.menus.get(m.kind as usize))
                .map(|d| d.recipes.clone())
                .unwrap_or_default();
            if arrow && !kinds.is_empty() {
                requests
                    .push(Request::Open(Box::new(crate::game_screens::RecipeView::new(crate::game_screens::RecipeQuery::Kinds(kinds)))));
                return;
            }
        }
        let callbacks = self.widgets_input(hovered, cursor);
        self.recipe_keys(requests);
        self.run_callbacks(callbacks, requests);
    }

    /// Slots: clicks, drags, sweeps, number keys, tooltips.
    fn menu_input(&mut self, hovered: Option<usize>, cursor: Vec2) {
        let Some((_, ui)) = self.screens.last_mut() else { return };
        let slot = hovered.and_then(|h| match ui.nodes[h].kind {
            Kind::Custom { tag: crate::client::modui::TAG_SLOT, data } => Some(data as u32),
            _ => None,
        });
        if let Some(g) = &mut self.game {
            let shift = self.input.held_key(Key::Code(KeyCode::ShiftLeft)) || self.input.held_key(Key::Code(KeyCode::ShiftRight));
            // Pressing with a stack on the cursor starts a drag; releasing over one slot is
            // a plain click, over several it spreads the stack.
            let holding = g.client.menu.as_ref().is_some_and(|m| !m.cursor.is_empty());
            use crate::server::menu::{
                CLONE, GATHER, HOTBAR_SWAP, OFF_HAND_SWAP, PICK_SAME, THROW, THROW_ALL, TO_TRASH, WHEEL_DOWN, WHEEL_UP,
            };
            // A release and a new press can both land between two frames.
            if g.client
                .drag
                .as_ref()
                .is_some_and(|(button, _)| !self.input.held_button(*button) || self.input.pressed_button(*button))
            {
                let (button, slots) = g.client.drag.take().unwrap();
                if slots.len() == 1 {
                    g.client
                        .send(net::ToServer::Click { slot: slots[0], button, shift: false });
                } else {
                    g.client.send(net::ToServer::Drag { slots, button });
                }
                self.quads_dirty = true;
            }
            let typing = ui
                .focused
                .is_some_and(|f| matches!(ui.nodes[f].kind, Kind::Input { .. }));
            // The wheel over a slot (Mouse Tweaks'): a notch up brings an item in from the other
            // side, down sends one there; with shift a stack.
            if let Some(s) = slot.filter(|_| self.input.scroll() != 0.0) {
                let button = if self.input.scroll() > 0.0 { WHEEL_UP } else { WHEEL_DOWN };
                for _ in 0..(self.input.scroll().abs().round() as u32).max(1) {
                    g.client
                        .send(net::ToServer::Click { slot: s, button, shift });
                }
            }
            for button in [0u8, 1] {
                if self.input.pressed_button(button) {
                    let inside = ui
                        .nodes
                        .iter()
                        .any(|n| matches!(n.kind, Kind::Container { class: Some(_) }) && n.rect.contains(cursor) && n.rect.w < ui.size.x);
                    // Vanilla: the same slot and button again within a quarter second, any other
                    // click in between breaks it.
                    let double = button == 0
                        && g.client
                            .last_click
                            .is_some_and(|(l, b, t)| Some(l) == slot && b == button && t.elapsed().as_secs_f32() < 0.25);
                    g.client.last_click = slot.map(|s| (s, button, Instant::now()));
                    if shift && button == 0 {
                        g.client.sweep.extend(slot);
                    }
                    match slot {
                        // The server tries to gather or applies a normal click with its cursor state.
                        Some(s) if double && !shift => {
                            g.client
                                .send(net::ToServer::Click { slot: s, button: GATHER, shift: false });
                            g.client.last_click = None;
                        }
                        Some(s) if holding && !shift && self.input.held_button(button) && g.client.drag.is_none() => {
                            g.client.drag = Some((button, vec![s]));
                            self.quads_dirty = true;
                        }
                        Some(s) => {
                            g.client
                                .send(net::ToServer::Click { slot: s, button, shift });
                            if button == 0 && !shift {
                                g.client.picking = vec![s];
                            }
                        }
                        None if !inside => g
                            .client
                            .send(net::ToServer::Click { slot: u32::MAX, button, shift: false }),
                        None => {}
                    }
                }
            }
            if let Some(s) = slot.filter(|_| self.input.pressed_button(2)) {
                g.client
                    .send(net::ToServer::Click { slot: s, button: CLONE, shift: false });
            }
            // Shift with the left button held: every slot swept over is shift-clicked.
            if shift && self.input.held_button(0) && !holding {
                if let Some(s) = slot.filter(|s| !g.client.sweep.contains(s)) {
                    g.client.sweep.push(s);
                    g.client
                        .send(net::ToServer::Click { slot: s, button: 0, shift: true });
                }
            } else if !self.input.held_button(0) {
                g.client.sweep.clear();
            }
            // Picked a stack up and still holding the button: slots swept over give theirs.
            if !self.input.held_button(0) {
                g.client.picking.clear();
            } else if let Some(s) = slot.filter(|s| !g.client.picking.is_empty() && !g.client.picking.contains(s)) {
                g.client.picking.push(s);
                g.client
                    .send(net::ToServer::Click { slot: s, button: PICK_SAME, shift: false });
            }
            // Number keys over a slot swap it with that hotbar slot, the drop keys throw from
            // it, unless typing a search.
            if let Some(s) = slot.filter(|_| !typing) {
                for (n, action) in crate::input::HOTBAR.into_iter().enumerate() {
                    if self.input.pressed(action) {
                        g.client
                            .send(net::ToServer::Click { slot: s, button: HOTBAR_SWAP + n as u8, shift: false });
                    }
                }
                let throw = if self.input.pressed("core:drop_all") {
                    Some(THROW_ALL)
                } else if self.input.pressed("core:drop") {
                    Some(THROW)
                } else if self.input.pressed("core:trash") {
                    Some(TO_TRASH)
                } else if self.input.pressed("core:swap_hands") {
                    Some(OFF_HAND_SWAP)
                } else {
                    None
                };
                if let Some(button) = throw.filter(|_| !holding) {
                    g.client
                        .send(net::ToServer::Click { slot: s, button, shift: false });
                }
            }
            if let Some((button, slots)) = &mut g.client.drag {
                // A right-button drag leaves one more each time it comes back onto a slot.
                let again = |s: &u32| if *button == 1 { slots.last() != Some(s) } else { !slots.contains(s) };
                if let Some(s) = slot.filter(again) {
                    slots.push(s);
                    self.quads_dirty = true;
                }
            }
            // Creative entries put a full stack on the cursor.
            let creative_item = hovered.and_then(|h| match ui.nodes[h].kind {
                Kind::Custom { tag: crate::client::modui::TAG_CREATIVE, data } => Some(data as u32),
                _ => None,
            });
            if let Some(item) = creative_item {
                for button in [0u8, 1] {
                    if self.input.pressed_button(button) {
                        g.client.send(net::ToServer::CreativeTake { item, button });
                    }
                }
                // A stack, like a slot.
                if self.input.pressed_button(2) {
                    g.client
                        .send(net::ToServer::CreativeTake { item, button: 0 });
                }
                // Number keys: a stack of it in that hotbar slot.
                for (n, action) in crate::input::HOTBAR.into_iter().enumerate() {
                    if !typing && self.input.pressed(action) {
                        g.client
                            .send(net::ToServer::CreativeTake { item, button: HOTBAR_SWAP + n as u8 });
                    }
                }
            }
            // Tooltip with the hovered item's name; other custom nodes (an effect's icon) have their own.
            if let Some(h) = hovered {
                if let Kind::Custom { tag: tag @ (crate::client::modui::TAG_SLOT | crate::client::modui::TAG_CREATIVE), data } =
                    ui.nodes[h].kind
                {
                    let stack = if tag == crate::client::modui::TAG_CREATIVE {
                        weft_abi::ItemStack::new(data as u32, 1)
                    } else {
                        g.client
                            .menu
                            .as_ref()
                            .and_then(|m| m.slots.get(data as usize))
                            .copied()
                            .unwrap_or_default()
                    };
                    // An empty slot shows its group's hint, if it has one.
                    let hint = || {
                        let c = content();
                        let def = c.game.menus.get(g.client.menu.as_ref()?.kind as usize)?;
                        let key = def
                            .group_of(&c.game.player_layout, data as u32)?
                            .0
                            .tooltip
                            .clone()?;
                        Some(crate::ui::text::rich(self.lang.get(&key), &self.lang, Some(&self.input)))
                    };
                    let tip = if g.client.drag.is_some() {
                        None
                    } else if !stack.is_empty() {
                        Some(crate::hud::item_tooltip(&self.lang, &self.input, stack))
                    } else if tag == crate::client::modui::TAG_SLOT {
                        hint()
                    } else {
                        None
                    };
                    if ui.nodes[h].tooltip != tip {
                        ui.nodes[h].tooltip = tip;
                        self.quads_dirty = true;
                    }
                }
            }
        }
    }

    /// Buttons, sliders and text fields under the mouse. What they do runs once `ui` is let go.
    fn widgets_input(&mut self, hovered: Option<usize>, cursor: Vec2) -> UiCallbacks {
        let mut actions: Vec<crate::ui::Action> = Vec::new();
        let mut slider: Vec<(std::rc::Rc<dyn Fn(&mut Ctx, f32)>, f32)> = Vec::new();
        let mut text_changed: Option<(std::rc::Rc<dyn Fn(&mut Ctx, &str)>, String)> = None;
        let Some((_, ui)) = self.screens.last_mut() else { return UiCallbacks::default() };
        if self.input.pressed_button(0) {
            ui.pressed = hovered;
            ui.focused = hovered.filter(|&h| {
                matches!(ui.nodes[h].kind, Kind::Input { .. } | Kind::Slider(crate::ui::slider::Slider { enabled: true, .. }))
            });
            if let Some(h) = hovered {
                let r = ui.nodes[h].rect;
                if let Kind::Slider(s) = &mut ui.nodes[h].kind {
                    s.begin(crate::ui::slider::Slider::value_at(cursor.x - r.x, r.w));
                }
            }
            ui.anchor = None;
            if let Some(f) = ui.focused
                && let Some(at) = self.ui.caret_at(ui, f, cursor)
                && let Kind::Input { caret, area, .. } = &mut ui.nodes[f].kind
            {
                *caret = at;
                if let Some(area) = area {
                    area.follow_caret.set(true);
                }
                self.quads_dirty = true;
            }
            if let Some(h) = hovered {
                if let Kind::Text(t) = &ui.nodes[h].kind {
                    let r = ui.nodes[h].rect;
                    let unit = ui.text_unit * t.scale;
                    let p = (cursor - Vec2::new(r.x, r.y)) / unit;
                    let target = t
                        .layout
                        .links
                        .iter()
                        .find(|(_, b)| p.x >= b[0] && p.x <= b[2] && p.y >= b[1] && p.y <= b[3])
                        .map(|l| l.0.clone());
                    match target.as_deref().map(|t| t.strip_prefix("open:").ok_or(t)) {
                        // A file or folder, opened with what the system opens it with.
                        Some(Ok(path)) => crate::packs::open_path(std::path::Path::new(path)),
                        Some(Err(t)) if t.starts_with("suggest:") => {
                            if let Some(n) = ui.find("chat") {
                                if let Kind::Input { value, caret, max, on_change, .. } = &mut ui.nodes[n].kind {
                                    *value = t["suggest:".len()..].chars().take(*max as usize).collect();
                                    *caret = value.chars().count();
                                    text_changed = on_change.as_ref().map(|cb| (cb.clone(), value.clone()));
                                    ui.focused = Some(n);
                                    ui.dirty = true;
                                }
                            }
                        }
                        Some(Err(t)) => {
                            if let Some(n) = ui.find(t) {
                                ui.focused = Some(n);
                            }
                        }
                        None => {}
                    }
                }
            }
            self.quads_dirty = true;
        }
        if self.input.held_button(0) {
            if let Some(p) = ui.pressed {
                let r = ui.nodes[p].rect;
                if let Kind::Slider(s) = &mut ui.nodes[p].kind
                    && s.enabled
                    && let Some((change, value)) = s.drag(crate::ui::slider::Slider::value_at(cursor.x - r.x, r.w))
                {
                    slider.extend(change.map(|f| (f, value)));
                    self.quads_dirty = true;
                }
            }
        } else if let Some(p) = ui.pressed.take() {
            if let Kind::Slider(s) = &ui.nodes[p].kind
                && s.enabled
                && let Some(t) = s.thumbs.get(s.active)
            {
                slider.extend(t.release.clone().map(|f| (f, t.value)));
            }
            if Some(p) == hovered {
                if let Kind::Button { action, on_double, enabled: true, .. } = &ui.nodes[p].kind {
                    actions.extend(action.clone());
                    // The same button again soon, built anew between or not (a list that
                    // shows what is selected): its id, or its place.
                    let which = ui.nodes[p].id.clone().unwrap_or_else(|| format!("#{p}"));
                    let again = self
                        .last_button
                        .as_ref()
                        .is_some_and(|(w, t)| *w == which && t.elapsed() < crate::ui::DOUBLE_CLICK);
                    if again {
                        actions.extend(on_double.clone());
                        self.last_button = None;
                    } else {
                        self.last_button = Some((which, Instant::now()));
                    }
                }
            }
            self.quads_dirty = true;
        }
        if let Some(f) = ui.focused {
            if let Kind::Input { value, caret, max, on_change, area } = &mut ui.nodes[f].kind {
                let lines = area.is_some();
                let outcome = crate::ui::edit::Field { value, caret, anchor: &mut ui.anchor, max: *max, lines }.handle(&self.input);
                let (changed, moved) = (outcome.changed, outcome.moved);
                if changed || moved {
                    if let Some(area) = area {
                        area.follow_caret.set(true);
                    }
                }
                if moved {
                    self.quads_dirty = true;
                }
                if changed {
                    ui.dirty = true;
                    self.quads_dirty = true;
                    if let Some(cb) = on_change {
                        text_changed = Some((cb.clone(), value.clone()));
                    }
                }
            } else if let Kind::Slider(s) = &mut ui.nodes[f].kind
                && s.enabled
                && ui.pressed != Some(f)
            {
                if self.input.pressed_key(Key::Code(KeyCode::ArrowUp)) && s.active > 0 {
                    s.active -= 1;
                    self.quads_dirty = true;
                }
                if self.input.pressed_key(Key::Code(KeyCode::ArrowDown)) && s.active + 1 < s.thumbs.len() {
                    s.active += 1;
                    self.quads_dirty = true;
                }
                let by: f32 = [(KeyCode::ArrowLeft, -1.0), (KeyCode::ArrowRight, 1.0)]
                    .into_iter()
                    .filter(|(k, _)| self.input.pressed_or_repeated(Key::Code(*k)))
                    .map(|(_, by)| by)
                    .sum();
                if by != 0.0
                    && let Some((change, value)) = s.nudge(by)
                {
                    slider.extend(change.map(|f| (f, value)));
                    slider.extend(s.thumbs[s.active].release.clone().map(|f| (f, value)));
                    self.quads_dirty = true;
                }
            }
        }
        UiCallbacks { actions, slider, text_changed }
    }

    /// Recipes of (or uses for) the item under the cursor, in any screen.
    fn recipe_keys(&mut self, requests: &mut Vec<Request>) {
        let menu = self.game.is_some() && ui_shows_menu(&self.screens);
        let Some((screen, ui)) = self.screens.last_mut() else { return };
        let typing = ui
            .focused
            .is_some_and(|f| matches!(ui.nodes[f].kind, Kind::Input { .. }));
        for (action, uses) in [("core:recipes", false), ("core:uses", true)] {
            let right_click = uses && matches!(screen.name(), "core:recipes" | "core:tag") && self.input.pressed_button(1);
            if !typing && (self.input.pressed(action) || right_click) {
                match crate::game_screens::hovered_item(ui, self.game.as_ref().map(|g| &g.client)) {
                    Some(item) => requests.push(Request::Open(Box::new(crate::game_screens::RecipeView::item(item, uses)))),
                    // Nothing under the mouse in a menu: every recipe, by kind (the HUD shows R).
                    None if menu && !right_click => {
                        let mut kinds: Vec<String> = content()
                            .game
                            .catalog
                            .ids(weft_abi::entry_kind::RECIPE_KIND)
                            .keys()
                            .cloned()
                            .collect();
                        kinds.sort();
                        requests.push(Request::Open(Box::new(crate::game_screens::RecipeView::new(
                            crate::game_screens::RecipeQuery::Kinds(kinds),
                        ))));
                    }
                    None => {}
                }
            }
        }
    }

    fn run_callbacks(&mut self, callbacks: UiCallbacks, requests: &mut Vec<Request>) {
        let UiCallbacks { actions, slider, text_changed } = callbacks;
        let ran = !actions.is_empty() || !slider.is_empty() || text_changed.is_some();
        let mut ctx = Ctx {
            settings: &mut self.settings,
            client: self.game.as_mut().map(|g| &mut g.client),
            input: &self.input,
            requests,
            stats: &self.stats,
            debug: self.debug,
            top: self
                .screens
                .last_mut()
                .map(|(s, ui)| Top { screen: Some(&mut **s), ui }),
        };
        if !actions.is_empty() {
            if let Some(click) = content().game.catalog.find(ek::SOUND, "core:ui.click") {
                self.ui_sounds
                    .push(crate::audio::Play { sound: click, pos: None, volume: 1.0, pitch: 0.0 });
            }
        }
        for a in actions {
            a(&mut ctx);
        }
        for (f, v) in slider {
            f(&mut ctx, v);
        }
        if let Some((f, v)) = text_changed {
            f(&mut ctx, &v);
        }
        if ran {
            self.quads_dirty = true;
        }
    }

    /// Runs the mods' client hooks of one kind with a view that may drive the client.
    fn client_frames(&mut self) {
        for &(owner, cb) in &content().game.client_frames {
            let mut v = self.frame_view();
            let mut a = abi::ClientFrameArgs { client: v.raw() };
            crate::host::invoke(owner, cb, &mut a);
        }
    }

    /// Every entity's cosmetics this frame: the engine's (`EntityKind::effects`), and mods'
    /// `client_entity_frame`, the local player's too; finished deaths go after their last call.
    fn entity_ticks(&mut self, dt: f32) {
        let Some(g) = &self.game else { return };
        if !g.client.joined {
            return;
        }
        let c = content();
        let mine = g.client.me.args(g.client.player_id, dt);
        let all: Vec<abi::ClientEntityArgs> = g
            .client
            .entities
            .iter()
            .filter(|(_, e)| e.item.is_empty() && e.structure.is_none())
            .map(|(&id, e)| e.args(id, dt))
            .collect();
        for mut a in all.into_iter().chain([mine]) {
            if let Some(g) = &mut self.game {
                g.client.entity_effects(&a);
            }
            for &(owner, cb) in &c.game.entities[a.kind as usize].frames {
                let mut v = self.frame_view();
                a.client = v.raw();
                crate::host::invoke(owner, cb, &mut a);
            }
        }
        if let Some(g) = &mut self.game {
            g.client
                .entities
                .retain(|_, e| !e.dying.is_some_and(|d| d >= 1.0));
        }
    }

    /// Puts what changed in the settings into effect, whoever changed them: a screen, a mod, a
    /// command or a test.
    fn apply_settings(&mut self) {
        use crate::settings as s;
        let fps_limit = s::fps_limit();
        let mut now = Applied {
            vsync: s::vsync(),
            frame_interval: (fps_limit > 0).then(|| Duration::from_secs_f64(1.0 / f64::from(fps_limit))),
            view: s::view_distance(),
            language: s::language(),
            gui_scale: s::gui_scale(),
            advanced_tooltips: s::advanced_tooltips(),
            packs: (s::resource_packs(), s::shader_pack()),
        };
        if now.vsync != self.applied.vsync {
            self.gpu.set_vsync(now.vsync);
        }
        if now.view != self.applied.view {
            if let Some(g) = &mut self.game {
                g.client.set_view_distance(now.view);
            }
        }
        let relang = now.language != self.applied.language;
        if relang {
            self.lang = Lang::load(&self.assets, &now.language, self.lang.dev);
            self.look = Look::new(&self.assets, &mod_ids(), &self.lang);
        }
        s::apply_binds(&mut self.input);
        if let Some(g) = &mut self.game {
            g.client.set_skin_model(s::slim_skin());
        }
        if let Some(entities) = &mut self.menu_entities {
            entities.set_preview_model(s::slim_skin());
        }
        if relang || now.gui_scale != self.applied.gui_scale || now.advanced_tooltips != self.applied.advanced_tooltips {
            self.rebuild_all();
        }
        // Not while choosing: every click would reload everything.
        if now.packs != self.applied.packs {
            if self
                .screens
                .iter()
                .any(|(s, _)| matches!(s.name(), "core:resource_packs" | "core:shader_packs"))
            {
                now.packs = self.applied.packs.clone();
                self.packs_pending = true;
            } else {
                self.reload_requested = true;
            }
        }
        self.applied = now;
    }

    fn frame_view(&mut self) -> crate::client::modui::ClientView {
        let mut v = crate::client::modui::ClientView::new(self.game.as_ref().map(|g| &g.client));
        v.run = self as *mut Running as *mut std::ffi::c_void;
        v
    }

    /// Runs a command for code that wants the answer (`client_op::RUN_COMMAND`): a client
    /// command here and now, any other on the server. The id finds the answer.
    pub(crate) fn run_command(&mut self, line: &str) -> u32 {
        let line = if line.starts_with('/') { line.to_string() } else { format!("/{line}") };
        let Some(g) = &mut self.game else { return 0 };
        g.client.next_command += 1;
        let id = g.client.next_command;
        match self.client_command_lines(&line) {
            Some(lines) => {
                if let Some(g) = &mut self.game {
                    g.client.command_replies.insert(id, lines);
                }
            }
            None => {
                if let Some(g) = &mut self.game {
                    g.client.send(net::ToServer::Command { id, line });
                }
            }
        }
        id
    }

    /// Runs `line` here if it is a command marked to run on the client; false otherwise.
    fn client_command(&mut self, line: &str) -> bool {
        let Some(reply) = self.client_command_lines(line) else { return false };
        if let Some(g) = &mut self.game {
            for line in reply {
                g.client.chat_line(line);
            }
        }
        true
    }

    /// The answer of `line` if it is a command that runs on the client.
    fn client_command_lines(&mut self, line: &str) -> Option<Vec<Msg>> {
        let rest = line.trim().strip_prefix('/')?;
        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let def = tokens
            .first()
            .and_then(|n| content().commands.iter().find(|d| d.client && d.name == *n))?;
        let g = self.game.as_ref()?;
        let p = &g.client.player;
        let here = crate::commands::Origin { pos: p.pos, yaw: p.yaw, pitch: p.pitch };
        let players = g.client.players.clone();
        let reply = match def.parse(content(), &tokens[1..], here, &players) {
            Err(e) => vec![Msg::new(
                "command.error_usage",
                vec![Arg::Msg(e), Arg::Text(def.name.clone()), Arg::Text(def.usage_for(&tokens[1..]))],
            )],
            Ok((args, handler)) => match handler {
                None => match CLIENT_COMMANDS.iter().find(|c| c.0 == def.name) {
                    Some(c) => (c.2)(self, &args),
                    None => Vec::new(),
                },
                Some((owner, cb)) => {
                    let values = crate::commands::abi_values(&args, &[]);
                    let mut reply: Vec<Msg> = Vec::new();
                    let reply_ptr = &mut reply as *mut Vec<Msg> as *mut core::ffi::c_void;
                    let player = self.game.as_ref().map_or(0, |g| g.client.player_id);
                    let mut v = self.frame_view();
                    let mut a = abi::CommandArgs {
                        world: std::ptr::null_mut(),
                        player,
                        values: values.as_ptr(),
                        count: values.len() as u32,
                        reply: reply_ptr,
                        client: v.raw(),
                    };
                    crate::host::invoke(owner, cb, &mut a);
                    reply
                }
            },
        };
        Some(reply)
    }

    /// Mod channel messages that arrived for the client, each to its handler.
    fn client_packets(&mut self) {
        let Some(g) = self.game.as_mut() else { return };
        let packets = std::mem::take(&mut g.client.packets);
        for (channel, data) in packets {
            let Some((owner, cb)) = content()
                .game
                .packets
                .get(channel as usize)
                .and_then(|p| p.client)
            else {
                continue;
            };
            let mut v = self.frame_view();
            let mut a = abi::ClientPacketArgs { client: v.raw(), data: abi::Slice::new(&data) };
            crate::host::invoke(owner, cb, &mut a);
        }
    }

    /// Re-evaluates bound values; layout and geometry only change when a value did.
    fn poll_bindings(&mut self) {
        let mut reqs = Vec::new();
        let server = self.game.as_ref().and_then(|g| g.server.as_ref());
        let gui_scale = self.gui_scale();
        let st = &mut self.stats;
        (st.fps, st.frame_ms, st.cpu) = (self.fps, self.frame_ms, self.prof);
        st.gpu.clear();
        st.gpu.extend_from_slice(self.gpu.gpu_times());
        (st.server_ms, st.regions, st.phases) = server.map_or((0.0, 0, [0.0; 6]), |s| (s.tick_ms(), s.regions(), s.phases()));
        st.load = server.map_or((0, 0), |s| {
            (s.shared.pending.load(std::sync::atomic::Ordering::Relaxed), s.shared.unsent.load(std::sync::atomic::Ordering::Relaxed))
        });
        st.window = self.gpu.surface_size();
        st.gui_scale = gui_scale;
        let ctx = Ctx {
            settings: &mut self.settings,
            client: self.game.as_mut().map(|g| &mut g.client),
            input: &self.input,
            requests: &mut reqs,
            stats: &self.stats,
            debug: self.debug,
            top: None,
        };
        let tcx = text::Ctx::new(&self.lang, Some(ctx.input));
        // Each condition is asked once per frame however many texts use it.
        let asked: std::cell::RefCell<Vec<(String, bool)>> = Default::default();
        let dev = self.lang.dev;
        let test = |t: &text::Test| match t {
            text::Test::Keys(keys) => keys.iter().all(|k| {
                if k.contains(':') { ctx.input.held(k) } else { crate::input::Key::parse(k).is_some_and(|key| ctx.input.held_key(key)) }
            }),
            text::Test::Fixed(holds) => *holds,
            text::Test::If(name) => {
                if let Some(&(_, v)) = asked.borrow().iter().find(|(n, _)| n == name) {
                    return v;
                }
                let v = text_condition(name, ctx.client.as_deref(), ctx.debug, dev);
                asked.borrow_mut().push((name.clone(), v));
                v
            }
        };
        let mut uis: Vec<&mut Ui> = self.screens.iter_mut().map(|(_, u)| u).collect();
        uis.push(&mut self.hud.ui);
        for ui in uis {
            let mut relayout = false;
            let mut redraw = false;
            let held = ui.pressed;
            for (i, n) in ui.nodes.iter_mut().enumerate() {
                if let Some(tip) = &mut n.tooltip {
                    redraw |= text::refresh(tip, &test);
                }
                match &mut n.kind {
                    Kind::Button { selected, selected_bind: Some(bind), .. } => {
                        let next = bind(&ctx);
                        redraw |= *selected != next;
                        *selected = next;
                    }
                    Kind::Text(t) => {
                        if let Some(b) = &t.bind {
                            let v = b(&ctx);
                            if t.last.as_ref() != Some(&v) {
                                t.spans.clear();
                                text::message(&v.0, &v.1, &tcx, &mut t.spans);
                                t.last = Some(v);
                                relayout = true;
                            }
                        }
                        relayout |= text::refresh(&mut t.spans, &test);
                    }
                    // Moving icons need drawing again, whatever the value does.
                    Kind::Icons { bind, value, shake, effects, change, count, .. }
                        if !effects.is_empty() || shake.is_some_and(|s| (0.0..=s).contains(value)) =>
                    {
                        let v = bind(&ctx);
                        if (v - *value).abs() > 1e-4 && *value >= 0.0 && v >= 0.0 {
                            *change = Some((*value, crate::shaders::clock(), v > *value));
                        }
                        *value = v;
                        let rows = crate::ui::icon_rows(*count, effects).1;
                        for e in effects.iter_mut() {
                            e.value = e.bind.as_ref().map_or(1.0, |b| b(&ctx));
                        }
                        relayout |= crate::ui::icon_rows(*count, effects).1 != rows;
                        redraw = true;
                    }
                    Kind::Progress { bind, value, .. } | Kind::Icons { bind, value, .. } | Kind::BoundImage { bind, value, .. } => {
                        let v = bind(&ctx);
                        if (v - *value).abs() > 1e-4 {
                            *value = v;
                            redraw = true;
                        }
                    }
                    Kind::Slider(s) => redraw |= s.poll(&ctx, held == Some(i)),
                    Kind::Item { bind, last } => {
                        let v = bind(&ctx);
                        if v != *last {
                            *last = v;
                            redraw = true;
                        }
                    }
                    _ => {}
                }
            }
            if relayout {
                ui.dirty = true;
            }
            if relayout || redraw {
                self.quads_dirty = true;
            }
        }
    }
}

/// `{if:..}`: `debug` and `dev`, else a condition (`conditions`, `creative`, `night & !surface`)
/// about the player; a wrong one is false.
fn text_condition(text: &str, client: Option<&Client>, debug: bool, dev: bool) -> bool {
    match text {
        "debug" => debug,
        "dev" => dev,
        _ => content().condition(text).is_some_and(|e| {
            let pos = client.map_or(glam::DVec3::ZERO, |cl| cl.player.pos);
            crate::conditions::client_holds(client, &e, &crate::conditions::Ask { pos, ..Default::default() })
        }),
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if let Some(r) = &mut self.run {
            r.gpu.set_surface(Some(&*r.window));
            if let Some(a) = &mut r.audio {
                a.set_active(self.opts.automated || r.window.has_focus());
            }
            return;
        }
        match self.start(el) {
            Ok(r) => self.run = Some(r),
            Err(e) => {
                self.error = Some(e);
                el.exit();
            }
        }
    }

    /// An Android app sent to the background: its window has no surface until it comes back.
    fn suspended(&mut self, _el: &ActiveEventLoop) {
        if let Some(r) = &mut self.run {
            if let Some(a) = &mut r.audio {
                a.set_active(false);
            }
            r.input.release_all();
            r.touches = Default::default();
            // At once: `pause` waits for a frame to take the world's picture, and none comes now.
            if r.game.is_some() && r.screens.is_empty() {
                r.open(screens::engine("core:pause"));
            }
            r.gpu.set_surface(None);
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let (Some(r), DeviceEvent::MouseMotion { delta }) = (&mut self.run, event) {
            if r.input.captured {
                r.input.mouse_delta += Vec2::new(delta.0 as f32, delta.1 as f32);
            }
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(r) = &mut self.run else { return };
        match event {
            WindowEvent::CloseRequested => r.quit = true,
            WindowEvent::Resized(size) => {
                r.gpu.resize(size.width, size.height);
                r.rebuild_all();
            }
            // Windows lets go of a confined cursor while another window has the focus.
            WindowEvent::Focused(true) if !self.opts.automated => {
                if let Some(a) = &mut r.audio {
                    a.set_active(true);
                }
                if r.input.captured && !r.settings.read_only {
                    r.center_cursor();
                    r.grab_cursor();
                }
            }
            WindowEvent::Focused(false) if !self.opts.automated => {
                if let Some(a) = &mut r.audio {
                    a.set_active(false);
                }
                r.input.release_all();
                r.touches = Default::default();
                if r.game.is_some() && r.screens.is_empty() {
                    r.pause();
                }
                r.set_capture(false);
            }
            WindowEvent::Touch(t) => r.touch(t),
            WindowEvent::KeyboardInput { event, .. } => {
                // Android Back arrives as a logical key with no physical code.
                let code = match event.physical_key {
                    PhysicalKey::Code(KeyCode::BrowserBack) => Some(KeyCode::Escape),
                    PhysicalKey::Code(code) => Some(code),
                    _ if event.logical_key == winit::keyboard::Key::Named(NamedKey::BrowserBack) => Some(KeyCode::Escape),
                    _ => None,
                };
                if let Some(code) = code {
                    match event.state {
                        ElementState::Pressed if event.repeat => r.input.key_repeat(Key::Code(code)),
                        ElementState::Pressed => r.input.key_down(Key::Code(code)),
                        ElementState::Released => r.input.key_up(Key::Code(code)),
                    }
                }
                if event.state == ElementState::Pressed {
                    if let Some(t) = &event.text {
                        r.input.typed.push_str(t);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let b = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Back => 3,
                    MouseButton::Forward => 4,
                    MouseButton::Other(n) => (5 + n).min(255) as u8,
                };
                if state == ElementState::Pressed {
                    if !r.input.captured && r.screens.is_empty() && r.game.is_some() && b == 0 {
                        r.set_capture(true);
                        return;
                    }
                    r.input.key_down(Key::Mouse(b));
                } else {
                    r.input.key_up(Key::Mouse(b));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                r.input.wheel(match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
                });
            }
            // Scripted and test runs place the cursor themselves: the desktop's pointer passing
            // over the window (two `--jobs` windows side by side) moved it off what they hover.
            WindowEvent::CursorMoved { position, .. } if !self.opts.automated => {
                r.input.cursor = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                if r.applied
                    .frame_interval
                    .is_some_and(|interval| now < r.last + interval)
                    && !r.quit
                {
                    return;
                }
                let aim = r.aim();
                r.touches.tick(&mut r.input, aim);
                if frame(r, &self.opts, now) || r.quit {
                    r.leave_world_silent();
                    r.close_home();
                    el.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(r) = &self.run {
            if let Some(interval) = r.applied.frame_interval {
                let next = r.last + interval;
                if next > Instant::now() && !r.quit {
                    el.set_control_flow(ControlFlow::WaitUntil(next));
                    return;
                }
            }
            el.set_control_flow(ControlFlow::Poll);
            r.window.request_redraw();
        }
    }

    fn exiting(&mut self, _el: &ActiveEventLoop) {
        finish_screenshots();
        if let Some(mut r) = self.run.take() {
            r.gpu.wait_idle();
            r.leave_world_silent();
            r.close_home();
            drop(r.gpu);
        }
    }
}

/// What a widget does, run once the screen's `Ui` is let go.
#[derive(Default)]
struct UiCallbacks {
    actions: Vec<crate::ui::Action>,
    slider: Vec<(std::rc::Rc<dyn Fn(&mut Ctx, f32)>, f32)>,
    text_changed: Option<(std::rc::Rc<dyn Fn(&mut Ctx, &str)>, String)>,
}

/// One frame. Returns true when the app should quit.
fn frame(r: &mut Running, opts: &Options, now: Instant) -> bool {
    r.retired = None;
    let dt = (now - r.last).as_secs_f64();
    r.last = now;
    let scale = r.gui_scale();
    start_frame(r, opts, dt);
    let mut requests = Vec::new();
    let opening = r.opening_world.is_some() || r.entering.is_some();
    if !opening && !screen_keys_first(r, &mut requests) && r.input.pressed_key(Key::Code(KeyCode::Escape)) {
        escape(r, &mut requests);
    }
    debug_keys(r);
    toggle_keys(r);
    if crate::chiacraft::worlds::listing::HOST_NOW.swap(false, std::sync::atomic::Ordering::Relaxed) {
        r.host_if_listed();
    }
    if r.packs_pending
        && !r
            .screens
            .iter()
            .any(|(s, _)| matches!(s.name(), "core:resource_packs" | "core:shader_packs"))
    {
        r.packs_pending = false;
        r.apply_settings();
    }
    if r.input.pressed("core:reload") {
        r.reload_requested = true;
    }
    // Not while a world is opening: a reload before the server let the player in, and once
    // (2026-11-04) one just after while the first terrain was still being built, left the GPU on
    // work it never finished (NVIDIA: Xid 109; Apple: a hang the whole machine went down with).
    // It waits for the join and the first terrain, but never more than 15 s after the join.
    if r.reload_requested && !opening {
        let joined = r.game.as_ref().is_none_or(|g| g.client.joined);
        let settled = r.game.is_none() || r.load_state().0;
        // Counted from the join.
        let waited = joined && r.reload_held.get_or_insert_with(Instant::now).elapsed() > Duration::from_secs(15);
        if joined && (settled || waited) {
            r.reload_requested = false;
            r.reload_held = None;
            reload_assets(r, opts);
        }
    }
    window_keys(r);
    if !opening {
        r.screen_input(scale, &mut requests);
    }
    r.handle(requests);
    soft_keyboard(r);
    r.finish_world_open();
    update_game(r, dt, now);
    r.entity_ticks(dt as f32);
    r.client_packets();
    if let Some(g) = &mut r.game {
        g.client
            .update_environment(&r.textures, dt as f32, r.audio.is_some());
    }
    play_sounds(r);
    // A join in progress (chiacraft): what it reports, answered.
    r.poll_join();
    handle_disconnect(r);
    enter_world(r);
    client_events(r);
    if r.game.as_ref().is_some_and(|g| g.client.joined) {
        world_keys(r);
    }
    end_input(r);
    refresh_hud(r);
    r.poll_bindings();
    let text_unit = layout_uis(r, scale);
    let t = Instant::now();
    r.prof[1] = (t.duration_since(now).as_secs_f32() * 1000.0 - r.prof[1]).max(0.0);
    let Some(info) = r.gpu.begin_frame() else { return false };
    let t2 = Instant::now();
    r.prof[2] = t2.duration_since(t).as_secs_f32() * 1000.0;
    let world = match upscale_frame(r, &info) {
        Some(w) => w,
        None => world_frame(r, &info),
    };
    if (r.depth.1, r.depth.2) != (world.width, world.height) {
        log::info!(
            "world drawn at {}x{} of the window's {}x{} (render scale {:.2}, density {:.1})",
            world.width,
            world.height,
            info.width,
            info.height,
            crate::settings::render_scale(),
            r.window.scale_factor()
        );
        r.gpu.destroy_texture(r.depth.0);
        r.depth = make_depth(&mut *r.gpu, world.width, world.height);
    }
    // Block items in slots take the colors of the biome the player is in.
    let biome = r.game.as_ref().and_then(|g| {
        g.client
            .world
            .biome_at(g.client.player.pos.floor().as_ivec3())
    });
    if biome != r.ui.biome {
        r.ui.biome = biome;
        r.quads_dirty = true;
    }
    let drag = r.game.as_ref().and_then(|g| {
        let (button, slots) = g.client.drag.as_ref()?;
        Some(g.client.menu.as_ref()?.drag_preview(*button, slots))
    });
    if r.quads_dirty {
        let _s = crate::diag::span("ui quads");
        build_ui_quads(r, scale, drag.as_ref());
    }
    let s = crate::diag::span("overlays");
    draw_overlays(r, &info, scale, text_unit, drag.as_ref());
    drop(s);
    record_passes(r, &info, &world, scale);
    let capture_now = r.shot_pending;
    let t3 = Instant::now();
    r.prof[3] = t3.duration_since(t2).as_secs_f32() * 1000.0;
    crate::diag::slow_record(r.prof[3]);
    if r.gpu.end_frame(&r.list) {
        crate::diag::renderer_started();
    }
    r.prof[4] = t3.elapsed().as_secs_f32() * 1000.0;
    report_frame(r, dt);
    if capture_now {
        save_screenshot(r);
    }
    count_fps(r);
    false
}

/// A phone's keyboard comes up while a text field has the focus (the chat) and goes away after.
/// A computer's text comes with its keys: asking for an input method there would turn it into
/// composition events instead.
fn soft_keyboard(r: &mut Running) {
    if !cfg!(target_os = "android") {
        return;
    }
    let typing = r
        .screens
        .last()
        .and_then(|(_, ui)| ui.focused.and_then(|f| ui.nodes.get(f)))
        .map(|n| matches!(n.kind, Kind::Input { .. }))
        .unwrap_or(false);
    if typing != r.keyboard {
        r.keyboard = typing;
        r.window.set_ime_allowed(typing);
    }
}

fn start_frame(r: &mut Running, opts: &Options, dt: f64) {
    r.frame_no += 1;
    // Link requests and answers arrive in the background: the links screen shows them as they do.
    let links_now = crate::links::GENERATION.load(std::sync::atomic::Ordering::Relaxed)
        + crate::friends::GENERATION.load(std::sync::atomic::Ordering::Relaxed)
        + crate::admission::GENERATION.load(std::sync::atomic::Ordering::Relaxed)
        + crate::chiacraft::achievements::GENERATION.load(std::sync::atomic::Ordering::Relaxed)
        + crate::chiacraft::worlds::finder::GENERATION.load(std::sync::atomic::Ordering::Relaxed);
    if links_now != r.links_seen {
        r.links_seen = links_now;
        if r.screens.last().is_some_and(|(s, _)| {
            ["core:links", "core:multiplayer", "core:friends", "core:admission", "core:account", "core:achievements", "core:finder"]
                .contains(&s.name())
        }) {
            r.rebuild_all();
        }
    }
    if crate::config::generation() != r.config_seen {
        r.config_seen = crate::config::generation();
        r.apply_settings();
    }
    r.frame_dt = dt as f32;
    // Textures first asked for while drawing the last frame.
    r.textures.load_requested(&mut *r.gpu);
    // The window may not have focus while a mod drives it.
    if opts.automated && r.screens.is_empty() && r.game.is_some() {
        r.input.captured = true;
    }
    r.release_mod_keys();
    r.mod_gizmos.clear();
    r.mod_screen_text.clear();
    r.client_frames();
}

/// Keys the top screen wants first (chat: completion, history; controls: the key to bind).
/// Escape still closes it after this, unless it took Escape itself: returns whether it did.
fn screen_keys_first(r: &mut Running, requests: &mut Vec<Request>) -> bool {
    let mut escape_taken = false;
    if let Some((s, ui)) = r.screens.last_mut().filter(|(s, _)| s.keys_first()) {
        for k in r.input.pressed_list() {
            let mut ctx = Ctx {
                settings: &mut r.settings,
                client: r.game.as_mut().map(|g| &mut g.client),
                input: &r.input,
                requests: &mut *requests,
                stats: &r.stats,
                debug: r.debug,
                top: Some(Top { screen: None, ui: &mut *ui }),
            };
            if s.key(k, &mut ctx) && k == Key::Code(KeyCode::Escape) {
                escape_taken = true;
            }
        }
        for k in r.input.released_list() {
            let mut ctx = Ctx {
                settings: &mut r.settings,
                client: r.game.as_mut().map(|g| &mut g.client),
                input: &r.input,
                requests: &mut *requests,
                stats: &r.stats,
                debug: r.debug,
                top: Some(Top { screen: None, ui: &mut *ui }),
            };
            s.key_up(k, &mut ctx);
        }
        if ui.dirty {
            r.quads_dirty = true;
        }
    }
    escape_taken
}

fn escape(r: &mut Running, requests: &mut Vec<Request>) {
    if r.screens.last().is_some_and(|(s, _)| !s.escape_closes()) {
    } else if r.screens.len() > 1 || (r.game.is_some() && !r.screens.is_empty()) {
        requests.push(Request::Close);
    } else if r.game.is_some() && r.screens.is_empty() {
        r.pause();
    }
}

/// The debug overlay and what goes with it (F3 and F3 plus a key by default).
fn debug_keys(r: &mut Running) {
    if r.input.pressed("core:advanced_tooltips") {
        crate::settings::set("advanced_tooltips", (!crate::settings::advanced_tooltips()) as u8 as f64);
    }
    if r.input.pressed("core:debug") {
        r.set_debug(!r.debug);
    }
    // Through the command, so permissions still apply.
    if r.input.pressed("core:cycle_game_mode")
        && let Some(g) = &mut r.game
    {
        let all: Vec<_> = crate::world::GameMode::all().collect();
        let next = all[(all.iter().position(|&m| m == g.client.mode).unwrap_or(0) + 1) % all.len()];
        g.client
            .send(net::ToServer::Chat(format!("/gamemode {}", next.name())));
    }
    if r.input.pressed("core:debug_options") && r.screens.is_empty() {
        r.open(screens::engine("core:debug_options"));
    }
    if let Some(g) = r.game.as_mut().filter(|_| r.input.pressed("core:relight")) {
        g.client.relight();
    }
    for (key, group) in [("core:show_cubes", "cubes"), ("core:show_hitboxes", "hitboxes")] {
        if let Some(g) = r.game.as_mut().filter(|_| r.input.pressed(key)) {
            g.client.toggle_gizmo(group);
        }
    }
    let groups = crate::hud::DEBUG_GROUPS.len() + crate::hud::mod_debug_groups().len();
    for i in 0..groups.min(9) {
        if r.input.pressed(&format!("core:debug_group{}", i + 1)) {
            crate::settings::set("debug_groups", (crate::settings::debug_groups() ^ (1 << i)) as f64);
            r.set_debug(true);
        }
    }
}

fn toggle_keys(r: &mut Running) {
    if r.input.pressed("core:hide_hud") && r.screens.is_empty() {
        r.hud_off = !r.hud_off;
        r.quads_dirty = true;
    }
    // The crosshair shows only in first person (`hud::draw_custom`): redraw when the view turns.
    if r.input.pressed("core:perspective") {
        r.quads_dirty = true;
    }
    if let Some(g) = &mut r.game {
        g.client.hide_gui = r.hud_off;
    }
}

/// Upstream's reload, kept as `Running::reload_assets` (chiacraft): a change of data packages
/// reloads too, where no `Options` is at hand. Our far terrain keeps no meshes to reload, so
/// upstream's `lod.reload()` is not called.
fn reload_assets(r: &mut Running, _opts: &Options) {
    r.reload_assets();
}

fn window_keys(r: &mut Running) {
    if r.input.pressed("core:fullscreen") {
        let on = r.window.fullscreen().is_none();
        r.window
            .set_fullscreen(on.then_some(winit::window::Fullscreen::Borderless(None)));
        crate::settings::set("fullscreen", on as u32 as f64);
    }
    if r.input.pressed("core:screenshot") {
        r.gpu.request_capture();
        r.shot_pending = true;
    }
}

fn update_game(r: &mut Running, dt: f64, frame_start: Instant) {
    let paused = r.screens.iter().any(|(s, _)| s.pauses());
    let world_input = r.screens.is_empty();
    if let Some(g) = &mut r.game {
        if let Some(s) = &g.server {
            s.set_paused(paused && !s.is_hosting());
        }
        g.client.fov = crate::settings::fov();
        g.client.fov_effects = crate::settings::fov_effects();
        g.client.camera_bob = crate::settings::camera_bob();
        g.client.brightness = crate::settings::brightness();
        g.client.first_person_model = crate::settings::first_person_model();
        g.client
            .entity_render
            .set_show_avatars(crate::settings::show_avatars());
        g.client.render.occlusion.on = crate::settings::occlusion_culling();
        // chiacraft: our soft clouds, until the A/B against upstream's volumetric clouds.
        g.client.sky.soft_wanted = crate::settings::soft_clouds();
        r.post.use_low = crate::settings::post_low();
        r.post.use_taa = crate::settings::aa();
        r.post.taa_blend = crate::settings::taa_blend();
        // The path tracer's denoiser: ours, unless Ray Reconstruction denoises the frame.
        r.post.denoiser.mode = crate::settings::pt_denoiser();
        (r.post.denoiser.phi_color, r.post.denoiser.steps) = crate::settings::pt_dn_tuning();
        r.post.denoiser.debug = crate::settings::pt_dn_debug();
        r.post.denoiser.reconstruct = r.upscale_now.is_some_and(|c| c.reconstruct);
        crate::denoise::set_test(crate::settings::pt_denoise_test());
        r.post.bloom_levels = crate::settings::bloom_levels();
        // The mist volume with the HDR world only, until today's (LDR) look takes it on Ronald's
        // word (its look was never shown to him there); `core:froxels` 0 turns it off there too.
        g.client.sky.froxels_wanted = crate::settings::froxels() && crate::post::hdr();
        g.client.sky.froxels.cell = crate::settings::froxel_cell();
        g.client.sky.cloud_shadow_wanted = crate::post::hdr() && crate::settings::cloud_shadows();
        let t = Instant::now();
        if world_input && g.client.joined {
            g.client.update(&mut *r.gpu, &r.textures, &mut r.input, dt);
        } else {
            g.client
                .update(&mut *r.gpu, &r.textures, &mut Input::default(), dt);
        }
        r.prof[0] = t.elapsed().as_secs_f32() * 1000.0;
        r.prof[1] = Instant::now().duration_since(frame_start).as_secs_f32() * 1000.0;
    }
}

fn play_sounds(r: &mut Running) {
    let mut music_ended = false;
    if let Some(a) = &mut r.audio {
        let listener = r.game.as_ref().map(|g| g.client.listener());
        if let Some(g) = &mut r.game
            && let Some(listener) = &listener
        {
            let cl = &mut g.client;
            let source = cl
                .local_music
                .source()
                .and_then(|source| cl.music_source_position(source))
                .map(|pos| (pos, cl.source_filter(pos, listener.pos)));
            music_ended = cl.local_music.update(source, listener.pos);
        }
        let local_playing = r
            .game
            .as_ref()
            .is_some_and(|g| g.client.local_music.source().is_some());
        a.update(&r.assets, listener.as_ref(), local_playing);
        if let Some(g) = &mut r.game
            && let Some(listener) = listener
        {
            g.client
                .play_audio(a, &r.assets, listener.pos, r.ui_sounds.drain(..));
        } else {
            for p in r.ui_sounds.drain(..) {
                a.play(&r.assets, p);
            }
        }
    } else {
        r.ui_sounds.clear();
        if let Some(g) = &mut r.game {
            g.client.discard_audio();
        }
    }
    if music_ended {
        r.rebuild_music_screens();
    }
}

fn client_events(r: &mut Running) {
    let events = r
        .game
        .as_mut()
        .map(|g| std::mem::take(&mut g.client.events))
        .unwrap_or_default();
    for ev in events {
        match ev {
            crate::client::ClientEvent::ControllerEdit => r.open(screens::engine(crate::controller::SCREEN)),
            crate::client::ClientEvent::OpenMenu => {
                let kind = r
                    .game
                    .as_ref()
                    .and_then(|g| g.client.menu.as_ref())
                    .map_or(0, |m| m.kind);
                r.screens.retain(|(s, _)| !s.shows_menu());
                r.open(Box::new(crate::game_screens::MenuScreen::new(kind)));
            }
            crate::client::ClientEvent::CloseMenu => r.close_screens(|s| s.shows_menu()),
        }
    }
    let dead = r.game.as_ref().is_some_and(|g| g.client.dead);
    let shown = r.entering.as_ref().is_none_or(|e| e.shown.is_some());
    if dead && shown && !r.screens.iter().any(|(s, _)| s.name() == "core:death") {
        r.screens.clear();
        r.open(screens::engine("core:death"));
    }
}

/// Whether the top screen is a menu (inventory, a machine): where R with nothing hovered opens
/// the whole recipe list. Not the recipe view itself, or chat and the like, where R is typed.
fn ui_shows_menu(screens: &[(Box<dyn Screen>, Ui)]) -> bool {
    screens.last().is_some_and(|(s, _)| s.shows_menu())
}

/// Chat and inventory keys, which open and close screens around the game.
fn world_keys(r: &mut Running) {
    if r.game.is_some() && r.screens.is_empty() {
        let command = r.input.pressed("core:command");
        if command || r.input.pressed("core:chat") {
            r.open(Box::new(crate::game_screens::Chat::new(if command { "/" } else { "" })));
            r.input.typed.clear();
        }
        // The recipe keys in the game itself: what the held item is made from (or used in), or
        // with an empty hand every recipe, by kind.
        else if let Some(uses) = [("core:recipes", false), ("core:uses", true)]
            .into_iter()
            .find(|(a, _)| r.input.pressed(a))
            .map(|(_, u)| u)
        {
            let held = r.game.as_ref().map(|g| g.client.held()).unwrap_or_default();
            let view = if held.item != 0 {
                crate::game_screens::RecipeView::item(held.item, uses)
            } else {
                let mut kinds: Vec<String> = content()
                    .game
                    .catalog
                    .ids(weft_abi::entry_kind::RECIPE_KIND)
                    .keys()
                    .cloned()
                    .collect();
                kinds.sort();
                crate::game_screens::RecipeView::new(crate::game_screens::RecipeQuery::Kinds(kinds))
            };
            r.open(Box::new(view));
        }
    }
    let (top_chat, top_menu) = r
        .screens
        .last()
        .map_or((false, false), |(s, _)| (s.shows_chat(), s.shows_menu()));
    if top_chat && r.input.pressed_key(Key::Code(KeyCode::Enter)) {
        let text = r.screens.last().and_then(|(_, ui)| {
            ui.find("chat").and_then(|n| match &ui.nodes[n].kind {
                Kind::Input { value, .. } => Some(value.clone()),
                _ => None,
            })
        });
        r.close_top();
        if let Some(t) = text.filter(|t| !t.trim().is_empty()) {
            if let Some(g) = &mut r.game {
                if g.client.sent.last() != Some(&t) {
                    g.client.sent.push(t.clone());
                    let sent = &mut g.client.sent;
                    sent.drain(..sent.len().saturating_sub(CHAT_HISTORY));
                    std::fs::write(chat_history_path(&r.settings.run_dir), sent.join("\n")).ok();
                }
            }
            r.handle(vec![Request::SendChat(t)]);
        }
    }
    let typing = r.screens.last().is_some_and(|(_, ui)| {
        ui.focused
            .is_some_and(|f| matches!(ui.nodes.get(f).map(|n| &n.kind), Some(Kind::Input { .. })))
    });
    if top_menu && r.input.pressed("core:inventory") && !typing {
        if let Some(g) = &mut r.game {
            g.client.close_menu();
        }
        r.close_screens(|s| s.shows_menu());
    }
    // Into the screen's search field: the first text field whose id says it is one
    // (`creative_search`, `tag_search`, a mod's own).
    if r.input.pressed("core:focus_search") && !typing {
        if let Some((_, ui)) = r.screens.last_mut() {
            let field = (0..ui.nodes.len()).find(|&i| {
                matches!(ui.nodes[i].kind, Kind::Input { .. })
                    && ui.nodes[i]
                        .id
                        .as_deref()
                        .is_some_and(|id| id.contains("search"))
            });
            if let Some(f) = field {
                ui.focused = Some(f);
                ui.dirty = true;
                r.quads_dirty = true;
            }
        }
    }
}

fn handle_disconnect(r: &mut Running) {
    if let Some(reason) = r.game.as_ref().and_then(|g| g.client.disconnected.clone()) {
        log::info!("disconnected: {reason}");
        let refused = r.game.as_ref().and_then(|g| g.client.refused.clone());
        r.leave_world_silent();
        r.screens.clear();
        if let Some(origin) = r.came_from.take() {
            // The world reached through a portal went away: back to the one the player came from.
            log::info!("back to the world before");
            r.handle(vec![Request::Join(origin)]);
        } else if r.home.is_some() {
            // The world you visited went away: back to your own, which kept running.
            r.return_home();
        } else {
            r.open(screens::engine("core:title"));
        }
        let why = refused.map_or(Arg::Text(reason), Arg::Msg);
        r.open(Box::new(screens::Message { title: "multiplayer.disconnected".into(), args: vec![why] }));
    }
}

fn end_input(r: &mut Running) {
    r.input.sensitivity = crate::settings::sensitivity();
    r.input.invert_y = crate::settings::invert_y();
    r.input.end_frame();
    let top = (
        r.screens.len(),
        r.screens
            .last()
            .map(|(s, _)| s.name().to_string())
            .unwrap_or_default(),
    );
    if top != r.screen_seen {
        r.screen_seen = top;
        r.screen_frames = 0;
    } else {
        r.screen_frames = r.screen_frames.saturating_add(1);
    }
}

fn refresh_hud(r: &mut Running) {
    let top = r.screens.last().map(|(s, _)| s.as_ref());
    if !top.is_some_and(|s| s.name() == "core:hud_editor") {
        r.hud.set_edit(None);
    }
    if r.game.is_some() || r.hud.editing() {
        let chat_open = top.is_some_and(Screen::shows_chat);
        // Any other screen over play (chiacraft): the HUD shows what it shows under a menu.
        let menu_open = top.is_some() && !chat_open;
        r.hud.knob = r.touches.knob();
        if r.hud.update(
            r.game.as_ref().map(|g| &g.client),
            r.debug,
            crate::settings::debug_groups(),
            chat_open,
            menu_open,
            &r.lang,
            &r.input,
        ) {
            r.quads_dirty = true;
        }
    }
    if let Some(g) = &r.game {
        let mut h = g.client.selected as u64;
        for s in g.client.inventory.iter().chain(
            g.client
                .menu
                .iter()
                .flat_map(|m| m.slots.iter().chain([&m.cursor])),
        ) {
            h = crate::world::mix(h ^ ((s.item as u64) << 32 | s.count as u64) ^ (s.data as u64) << 48);
        }
        if g.client.drag.is_some()
            && let Some(menu) = &g.client.menu
        {
            for target in &menu.drag_targets {
                h = crate::world::mix(h ^ ((u64::from(target.slot) << 32) | u64::from(target.room)));
            }
        }
        if h != r.client_hash {
            r.client_hash = h;
            r.quads_dirty = true;
        }
    }
    if r.textures.generation() != r.ui_textures {
        r.ui_textures = r.textures.generation();
        r.quads_dirty = true;
    }
}

fn layout_uis(r: &mut Running, scale: f32) -> f32 {
    let surface = r.gpu.surface_size();
    let gui_size = Vec2::new(surface.0 as f32, surface.1 as f32) / scale;
    // Whole physical pixels per pixel of the font's design.
    let p = r.ui.font.pixel;
    let text_unit = (scale * 0.5 * p).round().max(1.0) / (scale * p);
    // A phone's keyboard covers the bottom of the window: screens are laid out above it, so the
    // chat line stays in sight while it is typed in. The HUD keeps the whole window.
    let screen_size = Vec2::new(gui_size.x, visible_height(r).map_or(gui_size.y, |h| (h / scale).min(gui_size.y)));
    {
        let mut uis: Vec<(&mut Ui, Vec2)> = Vec::new();
        if r.game.is_some() || r.hud.editing() {
            uis.push((&mut r.hud.ui, gui_size));
        }
        uis.extend(r.screens.iter_mut().map(|(_, u)| (u, screen_size)));
        for (ui, size) in uis {
            if ui.dirty || ui.size != size || ui.text_unit != text_unit {
                ui.text_unit = text_unit;
                ui.layout(size, &r.ui.font);
                ui.dirty = false;
                r.quads_dirty = true;
            }
        }
    }
    text_unit
}

/// Physical pixels from the window's top down to what covers it (a phone's keyboard).
fn visible_height(r: &Running) -> Option<f32> {
    #[cfg(target_os = "android")]
    {
        use winit::platform::android::WindowExtAndroid;
        let rect = r.window.content_rect();
        (rect.bottom > rect.top).then_some(rect.bottom as f32)
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = r;
        None
    }
}

fn build_ui_quads(r: &mut Running, scale: f32, drag: Option<&DragPreview>) {
    let mut all = Vec::new();
    let game = crate::content::try_content().map(|c| (c, &r.textures));
    let client = r.game.as_ref().map(|g| &g.client);
    let mut previews = Vec::new();
    let preview_tex = r.preview_tex;
    let mut layers: Vec<&Ui> = Vec::new();
    let mut animated = Vec::new();
    let mut hud_quads = 0..0;
    let hud = r
        .game
        .as_ref()
        .is_some_and(|g| g.client.camera_has(crate::tag_bits::camera::HUD))
        || !r.screens.is_empty();
    let with_hud = r.hud.editing() || (r.game.is_some() && hud && !hud_hidden() && !(r.hud_off && r.screens.is_empty()));
    if with_hud {
        layers.push(&r.hud.ui);
    }
    let first = r
        .screens
        .iter()
        .rposition(|(s, _)| !s.overlays())
        .unwrap_or(0);
    layers.extend(r.screens[first..].iter().map(|(_, ui)| ui));
    for ui in layers {
        let look = &r.look;
        let textures = &r.textures;
        let mut custom = |ren: &mut UiRenderer, tag: u32, data: u64, rect: crate::ui::Rect, hovered: bool, icon: Option<u32>| {
            crate::hud::draw_custom(ren, look, textures, client, tag, data, rect, hovered, icon, scale, drag);
            let preview = match tag {
                crate::client::modui::TAG_ENTITY => Some(ModelPreview {
                    rect: crate::ui::Rect { x: rect.x + scale, y: rect.y + scale, w: rect.w - 2.0 * scale, h: rect.h - 2.0 * scale },
                    kind: PreviewKind::Entity(data),
                }),
                crate::editor::TAG_PREVIEW if crate::editor::is_block_preview(data) => {
                    Some(ModelPreview { rect, kind: PreviewKind::Block(data) })
                }
                _ => None,
            };
            if let Some(preview) = preview {
                previews.push(preview);
                if let Some(t) = preview_tex {
                    match preview.kind {
                        PreviewKind::Entity(_) => ren.image(preview.rect, t, [0.0, 0.0, 1.0, 1.0], 0xFFFFFFFF),
                        PreviewKind::Block(_) => ren.premultiplied_image(preview.rect, t, [0.0, 0.0, 1.0, 1.0], 0xFFFFFFFF),
                    }
                }
            }
        };
        let base = all.len();
        r.ui.build(&mut *r.gpu, &r.assets, ui, &r.look, scale, game, &mut custom);
        if with_hud && std::ptr::eq(ui, &r.hud.ui) {
            hud_quads = base..base + r.ui.quads.len();
        }
        all.append(&mut r.ui.quads);
        animated.extend(r.ui.animated.drain(..).map(|mut a| {
            a.quads = a.quads.start + base..a.quads.end + base;
            a
        }));
    }
    r.ui_cached = all;
    r.ui_animated = animated;
    r.hud_quads = hud_quads;
    r.previews = previews;
}

fn draw_overlays(r: &mut Running, info: &weft_render::FrameInfo, scale: f32, text_unit: f32, drag: Option<&DragPreview>) {
    // The stack held by the mouse follows it every frame, outside the cached quads.
    r.ui.quads.clear();
    r.ui.quads.extend_from_slice(&r.ui_cached);
    crate::ui::render::animate(&mut r.ui.quads, &r.ui_animated);
    // The HUD fades in once the server has let the player in.
    let shown = r
        .hud_since
        .map_or(0.0, |t| (t.elapsed().as_secs_f32() / HUD_FADE).min(1.0));
    if shown < 1.0 && !r.hud.editing() {
        for q in
            r.ui.quads
                .get_mut(r.hud_quads.clone())
                .into_iter()
                .flatten()
        {
            q.color = (q.color & !0xFF) | ((q.color & 0xFF) as f32 * shown) as u32;
        }
    }
    if let Some(g) = &mut r.game {
        g.client.collect_gizmos();
        let cl = &mut g.client;
        for s in &r.mod_gizmos {
            crate::client::gizmo::shape(s, &mut cl.gizmo_lines, &mut cl.gizmo_fills, &mut cl.gizmo_labels);
        }
    }
    // Gizmo labels and mods' screen text: text nodes placed at points (GUI pixels), the same
    // text as anywhere else.
    let mut labels: Vec<(f32, f32, &crate::ui::text::Msg, u32, f32, f32)> = Vec::new();
    if let Some(g) = &r.game {
        let cam = g.client.render_camera();
        let view_proj = cam.proj_plain(info.width as f32 / info.height.max(1) as f32) * cam.view();
        for (pos, text, color, size) in &g.client.gizmo_labels {
            let clip = view_proj * (*pos - cam.pos).as_vec3().extend(1.0);
            if clip.w < 0.1 || (*pos - cam.pos).length_squared() > 24.0 * 24.0 {
                continue;
            }
            let (x, y) = ((clip.x / clip.w * 0.5 + 0.5) * info.width as f32, (0.5 - clip.y / clip.w * 0.5) * info.height as f32);
            labels.push((x / scale, y / scale - 4.0, text, *color, *size, 0.0));
        }
    }
    let (sw, sh) = r.gpu.surface_size();
    if !hud_hidden() && !(r.hud_off && r.screens.is_empty()) {
        labels.extend(
            r.mod_screen_text
                .iter()
                .map(|(x, y, text, color, size, wrap)| (x * sw as f32 / scale, y * sh as f32 / scale, text, *color, *size, *wrap)),
        );
    }
    if !labels.is_empty() {
        let mut o = crate::ui::Ui::new();
        o.text_unit = text_unit;
        for (x, y, text, color, size, wrap) in labels {
            // A negative width: on the HUD's plate, wrapped to the width.
            let plate = wrap < 0.0;
            let wrap = wrap.abs();
            let mut spans = text.spans(&r.lang, Some(&r.input));
            if wrap > 0.0 {
                for s in &mut spans {
                    s.align = crate::ui::text::align::CENTER;
                }
            }
            let at = crate::ui::Style::default().at(x, y, [0.5, 0.0]);
            if plate {
                o.open(crate::ui::Kind::Container { class: Some("hud_plate".into()) }, at.pad(4.0));
            }
            let id = o.text(spans, "label");
            if !plate {
                o.nodes[id].style = at;
            }
            if wrap > 0.0 {
                o.nodes[id].style.max_w = wrap;
            }
            if let crate::ui::Kind::Text(t) = &mut o.nodes[id].kind {
                // A width given: wrapped to it, the lines centered (a mod's long hint in a narrow
                // window ran over the HUD's top-left column, 11-04).
                t.wrap = wrap > 0.0;
                t.center = wrap > 0.0;
                t.color = Some(color);
                t.scale = size;
            }
            if plate {
                o.close();
            }
        }
        o.layout(Vec2::new(sw as f32 / scale, sh as f32 / scale), &r.ui.font);
        let game = crate::content::try_content().map(|c| (c, &r.textures));
        r.ui.append(&mut *r.gpu, &r.assets, &o, &r.look, scale, game, &mut |_, _, _, _, _, _| {});
    }
    if let Some((screen, ui)) = r.screens.last() {
        let game = crate::content::try_content().map(|c| (c, &r.textures));
        let cursor = r.input.cursor / scale;
        let (source, hovered) = match (screen.as_ref() as &dyn std::any::Any).downcast_ref::<crate::hud_editor::HudEditor>() {
            Some(editor) => match editor.element_tooltip(cursor, ui, &r.hud) {
                Some(node) => (&r.hud.ui, Some(node)),
                None => (ui, ui.hit(cursor)),
            },
            None => (ui, ui.hovered),
        };
        r.ui.tooltip(&mut *r.gpu, &r.assets, source, hovered, &r.look, scale, r.input.cursor, game);
    }
    if let Some(cursor) = r
        .game
        .as_ref()
        .and_then(|g| g.client.menu.as_ref())
        .map(|m| drag.map_or(m.cursor, |preview| preview.cursor))
        .filter(|s| !s.is_empty())
    {
        let p = r.input.cursor;
        let rect = crate::ui::Rect { x: p.x - 9.0 * scale, y: p.y - 9.0 * scale, w: 18.0 * scale, h: 18.0 * scale };
        crate::hud::stack_icon(&mut r.ui, &r.textures, rect, cursor, scale, text_unit);
    }
    draw_entering(r, info, scale);
}

fn read_skin(run_dir: &std::path::Path) -> Option<crate::net::Skin> {
    Some(crate::net::Skin { png: std::fs::read(run_dir.join("skins/skin.png")).ok()?, slim: crate::settings::slim_skin() })
}

/// The player's own model, if `skins/avatar` has one: a Blockbench geometry and its picture, and
/// if they like clips, what bones do by themselves and a glow mask.
fn read_avatar(run_dir: &std::path::Path) -> Option<crate::net::AvatarData> {
    let dir = run_dir.join("skins").join("avatar");
    let file = |name: &str| std::fs::read(dir.join(name)).unwrap_or_default();
    let a = crate::net::AvatarData {
        geo: file("avatar.geo.json"),
        animations: file("avatar.animation.json"),
        bones: file("avatar.bones"),
        png: file("avatar.png"),
        glow: file("avatar_e.png"),
    };
    if a.geo.is_empty() || a.png.is_empty() {
        return None;
    }
    if a.size() > crate::net::AVATAR_BYTES {
        log::warn!("avatar: {} bytes, at most {} are sent", a.size(), crate::net::AVATAR_BYTES);
        return None;
    }
    Some(a)
}

/// `info` is the screen's frame, `world` the size the 3D world is drawn at (`world_frame`).
fn record_passes(r: &mut Running, info: &weft_render::FrameInfo, world_info: &weft_render::FrameInfo, scale: f32) {
    r.list.clear();
    if r.previews.is_empty() {
        r.menu_preview_failed = false;
    }
    if (r.game.is_some() || r.previews.is_empty())
        && let Some(mut entities) = r.menu_entities.take()
    {
        entities.destroy(&mut *r.gpu);
        r.preview_tex = None;
        r.quads_dirty = true;
    }
    if let Some(g) = &mut r.game {
        g.client
            .entity_render
            .preload(&mut *r.gpu, &r.assets, &g.client.entities, &g.client.entity_components, &g.client.me);
        if g.client
            .entity_render
            .draw_icons(&mut *r.gpu, &mut r.list, info)
        {
            r.quads_dirty = true;
        }
    }
    if !r.screens.is_empty() {
        if let Some(preview) = r.previews.first().copied() {
            let crate::ui::Rect { x, y, w, h } = preview.rect;
            if r.game.is_none() && r.menu_entities.is_none() && !r.menu_preview_failed {
                match crate::client::entity_render::EntityRenderer::new(&mut *r.gpu, &r.assets) {
                    Ok(mut entities) => {
                        entities.prepare_preview(
                            &mut *r.gpu,
                            &r.assets,
                            read_skin(&r.settings.run_dir).as_ref(),
                            read_avatar(&r.settings.run_dir),
                        );
                        r.menu_entities = Some(entities);
                    }
                    Err(e) => {
                        log::error!("player preview: {e}");
                        r.menu_preview_failed = true;
                    }
                }
            }
            // Whose figure (chiacraft): the player's (0), the menu's own entity (1), or a kind
            // alone (2 + its index; no entity: its own texture, nothing painted).
            let whose = match preview.kind {
                PreviewKind::Entity(whose) => whose,
                PreviewKind::Block(_) => 0,
            };
            let subject = r.game.as_ref().and_then(|g| match whose {
                1 => g
                    .client
                    .menu
                    .as_ref()
                    .map(|m| m.owner)
                    .and_then(|id| g.client.entities.get(&id).map(|e| (id, e.kind))),
                k if k >= 2 => Some((u64::MAX, (k - 2) as u32)),
                _ => None,
            });
            let renderer = match (&mut r.game, &mut r.menu_entities) {
                (Some(g), _) => Some((&mut g.client.entity_render, g.client.me.worn.as_slice())),
                (_, Some(entities)) => Some((entities, &[][..])),
                _ => None,
            };
            if let Some((entities, worn)) = renderer {
                let size = [w.round() as u32, h.round() as u32];
                let s = crate::diag::span("preview");
                let tex = match preview.kind {
                    PreviewKind::Entity(_) => {
                        // A kind alone: its skin loads with the others (none of it may be in the world).
                        entities.preview_kind = (whose >= 2).then(|| (whose - 2) as u32);
                        entities.draw_preview(&mut *r.gpu, &mut r.list, info, size, r.input.cursor - Vec2::new(x, y), worn, subject)
                    }
                    PreviewKind::Block(index) => {
                        crate::editor::render_block_preview(entities, &mut *r.gpu, &mut r.list, info, size, &r.textures, index)
                    }
                };
                drop(s);
                if tex != r.preview_tex {
                    r.preview_tex = tex;
                    r.quads_dirty = true;
                }
            }
        }
    }
    let clear = r.game.as_ref().map_or([0.02, 0.02, 0.03, 1.0], |g| {
        let [r, g, b, _rain] = g.client.fog_color();
        [r, g, b, 1.0]
    });
    // Screen effects need the world in an image they can read, and so does a world drawn smaller
    // than the screen (`render_scale`), which their last pass scales up.
    let scaled = (world_info.width, world_info.height) != (info.width, info.height);
    // The HDR world always ends in the effects' last pass, which finishes the picture.
    let upscaling = r.upscale_now.is_some();
    crate::post::set_upscaling(upscaling);
    crate::post::set_mip_bias(world_info.width, info.width);
    // The environment's screen effects (upstream's layers), with the camera's effects on.
    r.post.active = r
        .game
        .as_ref()
        .filter(|g| g.client.camera_has(crate::tag_bits::camera::EFFECTS))
        .map_or(Vec::new(), |g| g.client.environment.post.clone());
    let post = (r.post.on() || scaled || crate::post::hdr() || r.post.taa_on() || upscaling) && r.game.is_some();
    // Gathered over frames (temporal anti-aliasing), the world is drawn a fraction of a pixel
    // aside each frame, and its pass draws the motion image too (whenever the pipelines have it).
    crate::post::next_jitter(post && (r.post.taa_on() || upscaling), world_info.width, world_info.height);
    if !upscaling {
        crate::post::set_jitter_phases(8);
    }
    let motion = r
        .game
        .is_some()
        .then(|| {
            r.post
                .motion_target(&mut *r.gpu, world_info.width, world_info.height)
        })
        .flatten();
    let world_target = if post {
        Target::Texture(
            r.post
                .scene_target(&mut *r.gpu, world_info.width, world_info.height),
        )
    } else {
        Target::Surface
    };
    if let Some(g) = &mut r.game {
        let s = crate::diag::span("terrain gpu");
        g.client
            .gpu_terrain
            .frame(&mut *r.gpu, &mut r.list, info.slot);
        drop(s);
        let s = crate::diag::span("particles");
        let cam = g.client.render_camera();
        let wind = g.client.particle_wind();
        g.client
            .particles
            .simulate(&mut *r.gpu, &mut r.list, info.slot, &r.textures, &g.client.world, &cam, wind);
        drop(s);
        let s = crate::diag::span("wires and sun");
        g.client.update_wires(&mut *r.gpu, &r.textures);
        // `core:rt_shadows` or `core:ground_map` changed: the world shader composed again, tracing
        // or not, with the ground map or not, before anything of this frame is drawn with it.
        if g.client.rt_switch(&mut *r.gpu) || g.client.render.ground_switch() {
            g.client
                .render
                .reload_shaders(&mut *r.gpu, &r.textures, &r.assets);
        }
        // The sun's light for shadow maps (packs that use them), which the world's scene samples.
        g.client.prepare_shadows(&mut *r.gpu);
        // A copy of the world for what is seen through it (`water_behind` packs, with post on).
        drop(s);
        g.client.behind = [0.0; 4];
        if post && crate::packs::water_behind() {
            if let Some([c, d]) = r
                .post
                .behind_targets(&mut *r.gpu, world_info.width, world_info.height)
            {
                g.client.behind = [c as f32, d as f32, 1.0, 0.0];
            }
        }
    }
    // GPU time parts (`bench::GPU_PARTS`): terrain made and particles simulated so far, then the
    // shadow map, then the world.
    r.list.push(Cmd::Mark);
    // What the world's draws go by has to be done before its pass, and is known only making them.
    let (mut world, mut after) = (std::mem::take(&mut r.world_list), std::mem::take(&mut r.after_list));
    world.clear();
    after.clear();
    let mut atmosphere = false;
    if let Some(g) = &mut r.game {
        // Where this frame's gathering writes its count and motion, for the effects' sharpening.
        g.client.taa_out = r.post.taa_out(world_info.width, world_info.height);
        let s = crate::diag::span("client render");
        atmosphere = g
            .client
            .render(&mut *r.gpu, &mut world, &mut after, &mut r.list, world_info, &r.textures, r.entering.is_none());
        drop(s);
        // What the pack's look works out once a frame, before anything that reads it.
        r.list.append(&mut g.client.sky.frame_look.passes);
        // The soft clouds' shadow on the land, before the world that reads it.
        r.list.append(&mut g.client.sky.cloud_shadow.passes);
        // The shadow map once the models are posed, ahead of the world's pass that samples it.
        let s = crate::diag::span("shadows");
        g.client
            .render_shadows(&mut *r.gpu, &mut r.list, world_info);
        // The mist's volume, which reads the shadow maps, before the world that reads it.
        r.list.append(&mut g.client.sky.froxels.passes);
        drop(s);
    }
    r.list.push(Cmd::Mark);
    r.list.push(Cmd::BeginPass {
        // A depth of -1: no surface wrote its own motion there (`taa.wgsl`).
        motion: motion.map(|m| (m, Some([0.0, 0.0, -1.0, 0.0]))),
        color: Some((world_target, Some(clear))),
        depth: Some((r.depth.0, Some(0.0))),
    });
    r.list.append(&mut world);
    r.list.push(Cmd::EndPass);
    // Between passes: a tiled GPU (phones) draws a pass all at once, so a mark inside one tells
    // nothing there. It ends the GPU part "world" (`bench::GPU_PARTS`); `after` ends with the
    // client's mark, which ends "translucent".
    r.list.push(Cmd::Mark);
    if let Some(g) = &mut r.game {
        // The solid world's depth: clouds and the hand would hide chunks behind them in the next frame.
        g.client
            .render
            .occlusion
            .take_depth(&mut *r.gpu, &mut r.list, r.depth.0, [r.depth.1, r.depth.2], atmosphere);
        g.client.sample_atmosphere(&mut r.list);
        // The opaque world copied for what is seen through it (chiacraft, `water_behind` packs),
        // and the soft clouds traced against that copy, before the sky draws them.
        if g.client.behind[2] > 0.5 {
            if let Some(scene) = g.client.scene_buffer(info.slot) {
                let depth = r.gpu.texture_index(r.depth.0);
                r.post
                    .copy_behind(&mut *r.gpu, &mut r.list, world_info, scene, depth);
            }
            r.list.append(&mut g.client.sky.soft.passes);
        }
    }
    // What is drawn over the solid world (`Client::render`'s `after`): the sky, clouds, water,
    // particles and the hand, with our gathering's motion image (TAA).
    if !after.cmds.is_empty() {
        r.list.push(Cmd::BeginPass {
            motion: motion.map(|m| (m, None)),
            color: Some((world_target, None)),
            depth: Some((r.depth.0, None)),
        });
        r.list.append(&mut after);
        r.list.push(Cmd::EndPass);
    }
    (r.world_list, r.after_list) = (world, after);
    // Path traced lighting over the world's image (`core:path_traced`), before the effects read it.
    if let (Some(g), Target::Texture(_)) = (&mut r.game, world_target) {
        g.client
            .path_trace(&mut *r.gpu, &mut r.list, info.slot, r.depth.0, world_target, (world_info.width, world_info.height));
    }
    if post {
        let scene = r
            .game
            .as_ref()
            .and_then(|g| g.client.scene_buffer(info.slot));
        let depth = r.gpu.texture_index(r.depth.0);
        r.post.set_depth(r.depth.0);
        // The backend's upscaler in the engine's anti-aliasing's place, with the camera the world
        // was drawn with.
        if let (Some(c), Some(cam)) = (r.upscale_now, r.game.as_ref().and_then(|g| g.client.frame_camera)) {
            r.post.set_upscale(crate::post::UpscaleFrame {
                mode: c.mode,
                reconstruct: c.reconstruct || c.neural,
                output: (info.width, info.height),
                params: weft_render::UpscaleParams {
                    intensity: crate::settings::dlss5_intensity(),
                    reconstruct: c.reconstruct as u32,
                    model: c.model,
                    rr_model: c.rr_model,
                    neural: c.neural as u32,
                    ..cam.upscale_params()
                },
            });
        }
        let _s = crate::diag::span("effects");
        r.post
            .apply(&mut *r.gpu, &mut r.list, world_info, scene, depth, scaled);
        r.list.push(Cmd::Mark);
    } else {
        r.list.push(Cmd::Mark);
        // The UI pipeline has no depth, so it cannot draw in a pass that has one.
        r.list
            .push(Cmd::BeginPass { motion: None, color: Some((Target::Surface, None)), depth: None });
    }
    let s = crate::diag::span("ui draw");
    r.ui.draw(&mut *r.gpu, &mut r.list, info, scale);
    drop(s);
    r.list.push(Cmd::Mark);
    r.list.push(Cmd::EndPass);
}

fn report_frame(r: &mut Running, dt: f64) {
    if let Some(g) = &r.game {
        r.diag.frame(&crate::diag::FrameSample {
            frame_ms: dt as f32 * 1000.0,
            cpu: &r.prof,
            gpu: r.gpu.gpu_times(),
            client_cubes: g.client.world.len(),
            client_entities: g.client.entities.len(),
            pending_meshes: if crate::log::on(crate::log::Channel::Mesh, log::Level::Debug) { g.client.pending_meshes() } else { 0 },
            server: g.server.as_ref().map(|s| &*s.shared),
        });
    }
    if let Some(b) = &mut r.bench {
        let server = r.game.as_ref().and_then(|g| g.server.as_ref());
        let (server_ms, phases) = server.map_or((0.0, [0.0; 6]), |s| (s.tick_ms(), s.phases()));
        b.sample(dt as f32 * 1000.0, &r.prof, r.gpu.gpu_times(), server_ms, phases, &r.list);
        if b.done() {
            let extra = match &r.game {
                Some(g) => format!(
                    "pos {:.1} {:.1} {:.1}, cubes {}, draws {}, faces {}k, lod {} tiles {}k quads {}k drawn, entities {}, regions {}",
                    g.client.player.pos.x,
                    g.client.player.pos.y,
                    g.client.player.pos.z,
                    g.client.world.len(),
                    g.client.render.drawn,
                    g.client.render.faces_drawn / 1000,
                    g.client.lod_stats().0,
                    g.client.lod_stats().2 / 1000,
                    g.client.render.lod_drawn / 1000,
                    g.client.entities.len(),
                    g.server.as_ref().map_or(0, |s| s.regions())
                ),
                None => String::new(),
            };
            let line = b.report(&extra);
            log::info!("{line}");
            log::info!("{}", b.pacing());
            r.bench_last = b.summary();
            r.bench_line = line;
            r.benches += 1;
            r.bench = None;
        }
    }
}

fn save_screenshot(r: &mut Running) {
    r.shot_pending = false;
    if let Some((path, hud)) = r.preview_shot.take() {
        if let Some(cap) = r.gpu.take_capture() {
            // Whole for going into the world through it, small for the list; written aside, the
            // pause screen doesn't wait for it.
            let small = path.with_file_name(saves::PREVIEW_SMALL);
            crate::background().spawn(move || {
                let (rgba, w, h) = shrink(&cap.rgba, cap.width, cap.height, PREVIEW_WIDTH);
                for (p, w, h, px) in [(&path, cap.width, cap.height, &cap.rgba), (&small, w, h, &rgba)] {
                    if let Err(e) = crate::assets::encode_png(p, w, h, px) {
                        log::warn!("world picture {}: {e}", p.display());
                    }
                }
            });
        }
        r.hud_off = hud;
        r.quads_dirty = true;
        if r.game.is_some() && r.screens.is_empty() {
            r.open(screens::engine("core:pause"));
        }
        return;
    }
    if let Some(cap) = r.gpu.take_capture() {
        // The player's own (F2) says where it went; a mod's knows.
        let own = r.shot_path.is_none();
        let path = r.shot_path.take().unwrap_or_else(|| {
            r.settings
                .run_dir
                .join("screenshots")
                .join(format!("{}.png", saves::now_secs()))
        });
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Encoded on a thread: a PNG at 3456 x 2234 took 0.4-0.6 s of the frame (perf review).
        // The chat line can say where it goes now; quitting waits for it (`finish_screenshots`).
        if let Some(g) = r.game.as_mut().filter(|_| own) {
            let open = |p: &std::path::Path, label: &str| {
                Arg::Msg(Msg::template(&format!("{{link:open:{}}}{{lang:{label}}}{{/link}}", p.display())))
            };
            let folder = path.parent().unwrap_or(&path);
            g.client.chat_line(Msg::new(
                "chat.screenshot",
                vec![open(&path, "chat.screenshot.picture"), open(folder, "chat.screenshot.folder")],
            ));
        }
        let rgba = match r.shot_image.take() {
            Some(id) => {
                api::store_image(id, cap.width, cap.height, cap.rgba.clone());
                cap.rgba
            }
            None => cap.rgba,
        };
        let (w, h) = (cap.width, cap.height);
        let job = std::thread::Builder::new()
            .name("screenshot".into())
            .spawn(move || match crate::assets::encode_png(&path, w, h, &rgba) {
                Ok(()) => log::info!("screenshot saved to {}", path.display()),
                Err(e) => log::error!("screenshot: {e}"),
            });
        match job {
            Ok(j) => SHOTS.lock().unwrap_or_else(|e| e.into_inner()).push(j),
            Err(e) => log::error!("screenshot: no thread to save it: {e}"),
        }
    }
}

/// Screenshots still being encoded and written.
static SHOTS: std::sync::Mutex<Vec<std::thread::JoinHandle<()>>> = std::sync::Mutex::new(Vec::new());

/// Waits for the screenshots still being written (before the game quits).
fn finish_screenshots() {
    let jobs = std::mem::take(&mut *SHOTS.lock().unwrap_or_else(|e| e.into_inner()));
    for j in jobs {
        let _ = j.join();
    }
}

/// Reveal the prepared scene, then ask the server to spawn the player once the
/// animation has finished. Controls wait for the server's welcome.
fn enter_world(r: &mut Running) {
    let Some(g) = &mut r.game else {
        r.hud_since = None;
        return;
    };
    let now = Instant::now();
    if g.client.joined {
        r.hud_since.get_or_insert(now);
        if g.client.preparing_since.take().is_some() {
            r.entering = None;
            r.screens.clear();
            r.quads_dirty = true;
            r.set_capture(true);
        }
        return;
    }
    let Some(preparing) = g.client.preparing_since else { return };
    let near = g.client.near_shown(NEAR);
    let lod = g.client.lod_ready_for_entry();
    let scene_ready = near && lod || now - preparing >= ENTER_LOAD_LIMIT;
    match &mut r.entering {
        Some(e) => {
            if let Some(shown) = e.shown {
                if (now - shown).as_secs_f32() >= ENTER_FADE {
                    g.client.ready_to_join();
                }
            } else if e.expanded && scene_ready {
                log::debug!(
                    target: "load",
                    "revealing world after {:.3} s (near ready: {near}, LOD ready: {lod})",
                    (now - e.started).as_secs_f64()
                );
                e.shown = Some(now);
                r.screens.clear();
                r.quads_dirty = true;
            }
        }
        None if scene_ready => g.client.ready_to_join(),
        None => {}
    }
}

/// The picture of `Entering` over everything: from its place in the list to the window, eased
/// out, then fading.
fn draw_entering(r: &mut Running, info: &weft_render::FrameInfo, scale: f32) {
    let Some(e) = &mut r.entering else { return };
    let grow = (e.started.elapsed().as_secs_f32() / ENTER_GROW).min(1.0);
    e.expanded = grow >= 1.0;
    let k = 1.0 - (1.0 - grow).powi(3);
    let alpha = 1.0
        - e.shown
            .map_or(0.0, |t| (t.elapsed().as_secs_f32() / ENTER_FADE).min(1.0));
    let (w, h) = (info.width as f32, info.height as f32);
    let f = crate::ui::Rect { x: e.from.x * scale, y: e.from.y * scale, w: e.from.w * scale, h: e.from.h * scale };
    let rect = crate::ui::Rect { x: f.x * (1.0 - k), y: f.y * (1.0 - k), w: f.w + (w - f.w) * k, h: f.h + (h - f.h) * k };
    let picture = e.picture.clone();
    if let Some((idx, ..)) = r.ui.texture(&mut *r.gpu, &r.assets, &picture) {
        r.ui.image(rect, idx, [0.0, 0.0, 1.0, 1.0], 0xFFFFFF00 | (alpha * 255.0) as u32);
    }
}

/// Pixels across a world's small picture, for the list.
const PREVIEW_WIDTH: u32 = 384;

/// `rgba` (`w` x `h`) made `to` wide, each pixel the average of those it covers.
fn shrink(rgba: &[u8], w: u32, h: u32, to: u32) -> (Vec<u8>, u32, u32) {
    if w <= to || h == 0 {
        return (rgba.to_vec(), w, h);
    }
    let th = (h as u64 * to as u64 / w as u64).max(1) as u32;
    let mut out = vec![0u8; (to * th * 4) as usize];
    for y in 0..th {
        let (y0, y1) = (y * h / th, ((y + 1) * h / th).max(y * h / th + 1));
        for x in 0..to {
            let (x0, x1) = (x * w / to, ((x + 1) * w / to).max(x * w / to + 1));
            let mut sum = [0u32; 4];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy * w + sx) * 4) as usize;
                    for c in 0..4 {
                        sum[c] += rgba[i + c] as u32;
                    }
                }
            }
            let n = (y1 - y0) * (x1 - x0);
            let o = ((y * to + x) * 4) as usize;
            for c in 0..4 {
                out[o + c] = (sum[c] / n) as u8;
            }
        }
    }
    (out, to, th)
}

fn count_fps(r: &mut Running) {
    r.fps_frames += 1;
    let el = r.fps_timer.elapsed().as_secs_f32();
    if el >= 0.5 {
        r.fps = r.fps_frames as f32 / el;
        r.frame_ms = el * 1000.0 / r.fps_frames as f32;
        r.fps_timer = Instant::now();
        r.fps_frames = 0;
    }
}

/// Lines typed into chat, kept across sessions for the arrow keys.
const CHAT_HISTORY: usize = 100;

fn chat_history_path(run_dir: &std::path::Path) -> PathBuf {
    crate::paths::config(run_dir).join("chat_history.txt")
}

fn load_chat_history(run_dir: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(chat_history_path(run_dir))
        .map(|s| {
            s.lines()
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
