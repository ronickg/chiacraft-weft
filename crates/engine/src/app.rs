pub mod api;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::Vec2;
use weft_abi as abi;
use weft_render::{Backend, BackendDesc, Cmd, CommandList, Format, Target, TextureDesc, TextureId};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

use crate::assets::Assets;
use crate::client::Client;
use crate::client::textures::GameTextures;
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
use crate::ui::{Kind, Ui};

pub type BackendFactory = fn(&BackendDesc) -> Result<Box<dyn Backend>, weft_render::Error>;

#[derive(Clone, Debug)]
pub struct Options {
    pub run_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub mod_dirs: Vec<PathBuf>,
    /// Folders of data packages (chiacraft), earlier ones winning for a package found twice.
    pub package_dirs: Vec<PathBuf>,
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
}

pub enum Request {
    Open(Box<dyn Screen>),
    /// A screen by name: a mod's replacement if there is one, else the engine's.
    Screen(String),
    /// A mod's button was clicked: its callback (`UiArgs`) runs with the whole client API, like
    /// a frame hook, after the input is handled.
    ModCall(weft_abi::ModId, weft_abi::Callback),
    /// A mod's input or slider changed: its `on_change` with the text or value.
    ModChange(weft_abi::ModId, weft_abi::Callback, String, f32),
    /// A key for a mod's screen (`on_key`), by its settings name.
    ModKey(weft_abi::ModId, weft_abi::Callback, String),
    Close,
    Quit,
    /// Starts the game anew the way it was started, then leaves: what mods are on takes effect.
    Restart,
    Rebuild,
    Rebind(String, crate::input::Slots),
    ResetBinds,
    StartWorld(WorldMeta),
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

pub(crate) struct Running {
    gpu: Box<dyn Backend>,
    window: Arc<Window>,
    depth: (TextureId, u32, u32),
    list: CommandList,
    /// The world's pass, recorded before the sun's shadow map that goes ahead of it in `list`.
    world_list: CommandList,
    input: Input,
    pub(crate) settings: Settings,
    lang: Lang,
    languages: Vec<(String, String)>,
    look: Look,
    assets: Assets,
    textures: GameTextures,
    ui: UiRenderer,
    post: crate::post::Post,
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
    /// F3 held and used for a combination since it went down (then releasing it does not toggle).
    f3_combo: Option<bool>,
    /// F1: the world alone, until a screen opens or F1 again.
    hud_off: bool,
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
    /// Where 3D previews go, and whose: the player (0), the open menu's own entity (1), or an
    /// entity kind alone (2 + its index).
    previews: Vec<([f32; 4], u64)>,
    /// Texture holding the rendered inventory figure, drawn by the UI.
    preview_tex: Option<u32>,
    shot_path: Option<PathBuf>,
    /// Image id the pending screenshot is stored under for mods.
    shot_image: Option<u64>,
    /// Keys mods hold down, released at the given time.
    mod_held: Vec<(Key, Instant)>,
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
    /// The on-screen controls of a touch screen (`touch::enabled`).
    touch: crate::touch::Touch,
    /// A world to start once the loading screen has been on the screen this many frames
    /// (`begin_world`).
    pending_world: Option<(WorldMeta, u32)>,
}

struct App {
    factory: BackendFactory,
    opts: Options,
    run: Option<Running>,
    error: Option<String>,
    /// The window has no surface (Android, in the background): no frames until it's back.
    suspended: bool,
}

pub fn run(factory: BackendFactory, opts: Options) -> Result<(), String> {
    run_on(EventLoop::new().map_err(|e| e.to_string())?, factory, opts)
}

/// Android: the event loop of the activity `android_main` was given (crates/android).
#[cfg(target_os = "android")]
pub fn run_android(app: winit::platform::android::activity::AndroidApp, factory: BackendFactory, opts: Options) -> Result<(), String> {
    use winit::platform::android::EventLoopBuilderExtAndroid;
    let event_loop = EventLoop::builder()
        .with_android_app(app)
        .build()
        .map_err(|e| e.to_string())?;
    run_on(event_loop, factory, opts)
}

/// Android destroyed the activity (seen in winit's log, `log.rs`): the game saves and ends, which
/// Android's main thread waits for.
#[cfg(target_os = "android")]
pub(crate) static ANDROID_DESTROYED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn run_on(event_loop: EventLoop<()>, factory: BackendFactory, opts: Options) -> Result<(), String> {
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { factory, opts, run: None, error: None, suspended: false };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    match app.error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// The engine's own assets: next to the executable in a distribution, else (a build run from the
/// source tree, or `--dev`) the source tree's.
fn engine_assets(dev: bool) -> PathBuf {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let shipped = crate::paths::engine_assets()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("assets").join("core")))
        });
    match shipped {
        Some(s) if !dev && s.is_dir() => s,
        _ => source,
    }
}

/// The settings `Running::apply_settings` last put into effect.
#[derive(Default, PartialEq)]
struct Applied {
    vsync: bool,
    view: i32,
    language: String,
    gui_scale: u32,
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
        let mods = modloader::load(&self.opts.mod_dirs, &self.opts.package_dirs, &self.opts.cache_dir, &off)?;
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
        crate::settings::migrate(&self.opts.run_dir);
        crate::config::load(&self.opts.run_dir);
        if self.opts.no_vsync {
            crate::config::override_for_run("core:vsync", "false");
        }
        if let Some(v) = self.opts.view {
            crate::config::override_for_run("core:view_distance", &v.to_string());
        }
        // `--config "core:lod_distance=0,core:shader_pack="`: settings for this run only, as the
        // file writes them (benchmarks varying one thing at a time).
        if let Some(list) = api::launch_options().get("config") {
            for (name, value) in list.split(',').filter_map(|kv| kv.split_once('=')) {
                crate::config::override_for_run(name.trim(), value.trim());
            }
        }
        let mut settings = Settings::new(&self.opts.run_dir);
        log::info!("player name '{}' (setting core:name)", settings.name());
        // Links between worlds, and the inbox for them, open while the game runs (chiacraft).
        crate::links::init(&self.opts.run_dir, &settings.saves_dir());
        crate::friends::init(&self.opts.run_dir, &settings.saves_dir());
        crate::saved_worlds::set_settings(&settings);
        if chiacraft_account::current().is_some() {
            net::p2p::start_account(&settings);
            crate::chiacraft::chain::configure(&settings.run_dir, &settings.chain(), settings.chain_peers());
            // The account's first address on that chain, public, in a file: where to send it test
            // coins (a listing's deposit, phase 5).
            if let (Some(s), Ok(net)) = (chiacraft_account::current(), crate::chiacraft::chain::network(&settings.chain()))
                && let Some(ph) = s.first_puzzle_hash()
            {
                let address = chiacraft_wallet::chain::address(chiacraft_wallet::chain::Bytes32::from(ph), net);
                let _ = std::fs::write(s.home.join(format!("address-{}.txt", net.name())), format!("{address}\n"));
            }
        }
        let icon = crate::assets::decode_png_file(&engine_assets(self.opts.dev).join("icon.png"))
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
        let mut gpu = (self.factory)(&BackendDesc {
            window: &*window,
            width: size.width,
            height: size.height,
            vsync: settings.vsync(),
            debug: self.opts.gpu_debug,
        })
        .map_err(|e| e.to_string())?;
        // The pipelines the driver compiled last time, before any is made (saved at exit).
        let cache = self.opts.cache_dir.join("pipelines.bin");
        if let Ok(data) = std::fs::read(&cache) {
            gpu.load_pipeline_cache(&data);
            log::info!("pipeline cache: {} KB from the last start", data.len() / 1024);
        }
        let _ = PIPELINE_CACHE.set(cache);

        let mut roots: Vec<(String, PathBuf)> = vec![("core".into(), engine_assets(self.opts.dev))];
        // Code mods' and data packages' assets alike.
        roots.extend(
            crate::content::content()
                .mods
                .iter()
                .map(|m| (m.id.clone(), m.assets.clone())),
        );
        let packs_now = (settings.resource_packs(), settings.shader_pack());
        let packs = crate::packs::enabled(&self.opts.run_dir, &settings.resource_packs());
        crate::packs::set_shader_pack(
            settings
                .shader_pack()
                .as_ref()
                .map(|p| crate::packs::shader_dir(&self.opts.run_dir).join(p)),
        );
        let assets = Assets::new(roots, packs);
        crate::assets::set_current(&assets);
        let lang = Lang::load(&assets, &settings.language(), self.opts.dev);
        let look = Look::new(&assets, &mod_ids(), &lang);
        let textures = GameTextures::new(&mut *gpu, content(), &assets);
        let ui = UiRenderer::new(&mut *gpu, &assets)?;
        // The HDR world's format, before any pipeline drawing into the world is made.
        crate::post::set_hdr(settings.hdr_world());
        // And whether they draw the motion image too, for temporal anti-aliasing.
        crate::post::set_motion(settings.aa());
        let post = crate::post::Post::new(&mut *gpu, &assets);
        let mut input = Input::default();
        for (action, key) in &content().game.keys {
            match crate::input::Binding::parse_slots(key) {
                Some(k) => input.register(action, k),
                None => log::warn!("key binding {action}: unknown key {key}"),
            }
        }
        settings.apply_binds(&mut input);
        settings.read_only = self.opts.automated;
        crate::shaders::pin_time(self.opts.automated && !recording());
        let depth = make_depth(&mut *gpu, size.width, size.height);
        let gpu_name = gpu.device_name().to_string();
        // What the GPU is, for mods (`core:gpu_*`): as found, whatever a file says.
        for (name, v) in [("core:gpu_class", gpu.device_class() as f64), ("core:gpu_memory_mb", gpu.device_memory_mb() as f64)] {
            if let Some(id) = crate::config::by_name(name) {
                crate::config::put(id, v);
            }
        }
        if let Some(id) = crate::config::by_name("core:gpu_name") {
            crate::config::put_text(id, &gpu_name);
        }
        let mut r = Running {
            gpu,
            window,
            depth,
            list: CommandList::default(),
            world_list: CommandList::default(),
            input,
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
            f3_combo: None,
            hud_off: false,
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
            touch: crate::touch::Touch::default(),
            pending_world: None,
        };
        if let Some(world) = &self.opts.world {
            let saves_dir = r.settings.saves_dir();
            let meta = saves::list(&saves_dir)
                .into_iter()
                .find(|w| &w.name == world)
                .unwrap_or_else(|| saves::create(&saves_dir, world, self.opts.seed.unwrap_or(1)));
            r.begin_world(meta);
        } else {
            r.open(screens::engine("core:title"));
            if let Some(name) = &self.opts.screen {
                match screens::named(name) {
                    Some(s) => r.open(s),
                    None => log::warn!("--screen: no screen {name}"),
                }
            }
        }
        Ok(r)
    }
}

/// The size the 3D world is drawn at this frame: the screen's times `core:render_scale` (the
/// screen effects' last pass scales it up and the UI is drawn over it whole), or the screen's when
/// there is no world or no post pipeline to scale it with.
fn world_frame(r: &Running, info: &weft_render::FrameInfo) -> weft_render::FrameInfo {
    let s = crate::settings::render_scale(r.window.scale_factor());
    if s >= 0.999 || r.game.is_none() || !r.post.can_scale() {
        return weft_render::FrameInfo { ..*info };
    }
    let scaled = |v: u32| ((v as f32 * s).round() as u32).max(1);
    weft_render::FrameInfo { width: scaled(info.width), height: scaled(info.height), slot: info.slot }
}

fn make_depth(gpu: &mut dyn Backend, w: u32, h: u32) -> (TextureId, u32, u32) {
    let t = gpu
        .create_texture(&TextureDesc { width: w.max(1), height: h.max(1), mips: 1, format: Format::Depth32, render_target: true }, "depth");
    (t, w, h)
}

/// Where the driver's compiled pipelines are kept between starts (`Backend::pipeline_cache_data`).
static PIPELINE_CACHE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Keeps the driver's compiled pipelines for the next start: the shaders then skip most of their
/// compiling (on the Mac, MoltenVK's translation too). Written aside and moved, never half.
fn save_pipeline_cache(gpu: &dyn Backend) {
    let Some(path) = PIPELINE_CACHE.get() else { return };
    let data = gpu.pipeline_cache_data();
    if data.is_empty() {
        return;
    }
    let tmp = path.with_extension("bin.tmp");
    let saved = path
        .parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|_| std::fs::write(&tmp, &data))
        .and_then(|_| std::fs::rename(&tmp, path));
    match saved {
        Ok(()) => log::debug!("pipeline cache: {} KB saved", data.len() / 1024),
        Err(e) => log::warn!("pipeline cache {}: {e}", path.display()),
    }
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
        crate::settings::applied_gui_scale(self.settings.gui_scale(), max) as f32
    }

    fn build_cx(&self) -> BuildCx<'_> {
        BuildCx {
            lang: &self.lang,
            input: &self.input,
            settings: &self.settings,
            languages: &self.languages,
            in_game: self.game.is_some(),
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
        }
    }

    fn open(&mut self, mut s: Box<dyn Screen>) {
        if s.needs_languages() && self.languages.is_empty() {
            self.languages = Lang::available(&self.assets);
        }
        let mut ui = Ui::new();
        s.build(&mut ui, &self.build_cx());
        scroll_when_taller(&mut ui);
        self.screens.push((s, ui));
        self.quads_dirty = true;
        self.set_capture(false);
    }

    fn rebuild_all(&mut self) {
        let mut screens = std::mem::take(&mut self.screens);
        for (s, ui) in &mut screens {
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
            s.build(ui, &self.build_cx());
            scroll_when_taller(ui);
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
        self.hud.invalidate();
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
        self.assets
            .set_packs(crate::packs::enabled(&run_dir, &self.settings.resource_packs()));
        crate::packs::set_shader_pack(
            self.settings
                .shader_pack()
                .as_ref()
                .map(|p| crate::packs::shader_dir(&run_dir).join(p)),
        );
        self.assets.rescan();
        crate::assets::set_current(&self.assets);
        self.hud.load_layout(&run_dir);
        self.ui.reload(&mut *self.gpu, &self.assets);
        self.post.reload(&mut *self.gpu, &self.assets);
        self.lang = Lang::load(&self.assets, &self.settings.language(), self.lang.dev);
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
        for (action, key) in &content().game.keys {
            if let Some(k) = crate::input::Binding::parse_slots(key) {
                self.input.register(action, k);
            }
        }
        self.settings.apply_binds(&mut self.input);
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
    fn enter_remote(&mut self, conn: net::ClientConn) {
        let key = net::p2p::player_key(&self.settings.run_dir);
        match Client::new(
            &mut *self.gpu,
            &self.assets,
            &self.textures,
            conn,
            &self.settings.name(),
            &key,
            self.settings.view_distance(),
            std::fs::read(self.settings.run_dir.join("skins").join("skin.png")).ok(),
            read_avatar(&self.settings.run_dir),
        ) {
            Ok(mut client) => {
                client.sent = load_chat_history(&self.settings.run_dir);
                self.game = Some(Game { client, server: None, world: String::new(), identity: None });
                self.screens.clear();
                self.hud.invalidate();
                self.set_capture(true);
            }
            Err(e) => log::error!("{e}"),
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

    /// Starts a world after the loading screen has been drawn: starting builds every pipeline and
    /// waits for the world, the frame thread busy all the while, and that frame stays on the
    /// screen meanwhile instead of black or a frozen menu. Scripted runs start at once, as before.
    fn begin_world(&mut self, meta: WorldMeta) {
        if self.settings.read_only {
            self.start_world(meta);
            return;
        }
        self.screens.clear();
        self.open(Box::new(screens::Loading));
        self.pending_world = Some((meta, 0));
    }

    fn start_world(&mut self, mut meta: WorldMeta) {
        log::info!("opening world '{}' (seed {})", meta.name, meta.seed);
        if meta.format != saves::SAVE_FORMAT {
            log::error!("world '{}' was saved in format {}, this build reads {}", meta.name, meta.format, saves::SAVE_FORMAT);
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
        let storage = saves::Storage::open(meta.clone(), content());
        let server = server::spawn(
            ServerConfig {
                seed: meta.seed,
                view_radius: self.settings.view_distance(),
                dedicated: false,
                whitelist: None,
                identity: identity.clone(),
            },
            Some(storage),
        );
        self.play_local(server, meta.name.clone(), identity);
    }

    /// Plays in the world `server` runs in this process.
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

    fn play_local(&mut self, server: ServerHandle, world: String, identity: Option<Arc<crate::world_address::WorldIdentity>>) {
        crate::admission::set_local_world(server.dir.clone(), identity.clone());
        let (client_conn, server_conn) = net::local_pair();
        let _ = server.connect.send(server_conn);
        let key = net::p2p::player_key(&self.settings.run_dir);
        match Client::new(
            &mut *self.gpu,
            &self.assets,
            &self.textures,
            client_conn,
            &self.settings.name(),
            &key,
            self.settings.view_distance(),
            std::fs::read(self.settings.run_dir.join("skins").join("skin.png")).ok(),
            read_avatar(&self.settings.run_dir),
        ) {
            Ok(mut client) => {
                client.sent = load_chat_history(&self.settings.run_dir);
                // The synced settings the player changes here are this world's.
                crate::config::playing_in(Some(server.configs.clone()));
                self.game = Some(Game { client, server: Some(server), world, identity });
                self.host_if_listed();
                self.screens.clear();
                self.hud.invalidate();
                self.quads_dirty = true;
                self.set_capture(true);
                // The world's pipelines are built: kept now, as Android ends an app without the
                // exit that saves them (a phone compiled the shader pack's for 38 s every start).
                if cfg!(target_os = "android") {
                    save_pipeline_cache(&*self.gpu);
                }
            }
            Err(e) => {
                log::error!("{e}");
                // Something to look at instead of an empty window; the server stops with `server`.
                self.screens.clear();
                self.open(Box::new(screens::Message { title: "world.open_failed".into(), args: vec![Arg::Text(e.to_string())] }));
            }
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
        self.play_local(home.server, home.world, home.identity);
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
        if self.screens.last().is_some_and(|(s, _)| s.shows_menu()) {
            if let Some(g) = &mut self.game {
                g.client.close_menu();
            }
        }
        self.screens.pop();
        self.screens_gone();
    }

    pub(crate) fn handle(&mut self, reqs: Vec<Request>) {
        for r in reqs {
            match r {
                Request::Open(s) => self.open(s),
                Request::Close => self.close_top(),
                Request::Quit => self.quit = true,
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
                Request::ModCall(owner, cb) => {
                    let mut v = self.frame_view();
                    let mut a = weft_abi::UiArgs { ui: std::ptr::null_mut(), client: v.raw() };
                    crate::host::invoke(owner, cb, &mut a);
                }
                Request::ModChange(owner, cb, text, value) => {
                    let mut v = self.frame_view();
                    let mut a = weft_abi::UiChangeArgs { client: v.raw(), text: weft_abi::Str::new(&text), value };
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
                    self.settings.set_bind(&action, &keys);
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
                        self.settings.set_bind(&a, &k);
                    }
                    self.rebuild_all();
                }
                Request::StartWorld(meta) => {
                    let mut missing = meta.missing_mods(content());
                    missing.extend(modloader::for_world(&meta).1);
                    if missing.is_empty() {
                        self.begin_world(meta);
                    } else {
                        self.open(Box::new(saves::MissingMods { world: meta, missing }));
                    }
                }
                Request::StartWorldAnyway(meta) => self.begin_world(meta),
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
        if self.input.scroll != 0.0 && ui.scroll_at(cursor, self.input.scroll) {
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
        let menu_screen = self.screens.last().is_some_and(|(s, _)| s.shows_menu());
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
            use crate::server::menu::{CLONE, GATHER, HOTBAR_SWAP};
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
                        // Double click: the rest of the stack just picked up joins it (the
                        // server knows what is on the cursor even before we do).
                        Some(s) if double && !shift => {
                            g.client
                                .send(net::ToServer::Click { slot: s, button: GATHER, shift: false });
                            g.client.last_click = None;
                        }
                        Some(s) if holding && !shift && g.client.drag.is_none() => g.client.drag = Some((button, vec![s])),
                        Some(s) => g
                            .client
                            .send(net::ToServer::Click { slot: s, button, shift }),
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
            // Number keys over a slot swap it with that hotbar slot, unless typing a search.
            if let Some(s) = slot.filter(|_| {
                !ui.focused
                    .is_some_and(|f| matches!(ui.nodes[f].kind, Kind::Input { .. }))
            }) {
                for (n, action) in crate::input::HOTBAR.into_iter().enumerate() {
                    if self.input.pressed(action) {
                        g.client
                            .send(net::ToServer::Click { slot: s, button: HOTBAR_SWAP + n as u8, shift: false });
                    }
                }
            }
            if let Some((button, slots)) = &mut g.client.drag {
                if self.input.held_button(*button) {
                    if let Some(s) = slot.filter(|s| !slots.contains(s)) {
                        slots.push(s);
                        self.quads_dirty = true;
                    }
                } else {
                    let (button, slots) = g.client.drag.take().unwrap();
                    if slots.len() == 1 {
                        g.client
                            .send(net::ToServer::Click { slot: slots[0], button, shift: false });
                    } else {
                        g.client.send(net::ToServer::Drag { slots, button });
                    }
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
            }
            // Tooltip with the hovered item's name.
            if let Some(h) = hovered {
                if let Kind::Custom { tag, data } = ui.nodes[h].kind {
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
                    let tip = if !stack.is_empty() {
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
        let mut slider: Option<(std::rc::Rc<dyn Fn(&mut Ctx, f32)>, f32)> = None;
        let mut text_changed: Option<(std::rc::Rc<dyn Fn(&mut Ctx, &str)>, String)> = None;
        let Some((_, ui)) = self.screens.last_mut() else { return UiCallbacks::default() };
        if self.input.pressed_button(0) {
            ui.pressed = hovered;
            ui.focused = hovered.filter(|&h| matches!(ui.nodes[h].kind, Kind::Input { .. }));
            ui.anchor = None;
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
                if let Kind::Slider { value, on_change, .. } = &mut ui.nodes[p].kind {
                    let v = ((cursor.x - r.x) / r.w).clamp(0.0, 1.0);
                    if (v - *value).abs() > 1e-4 {
                        *value = v;
                        slider = Some((on_change.clone(), v));
                        self.quads_dirty = true;
                    }
                }
            }
        } else if let Some(p) = ui.pressed.take() {
            if Some(p) == hovered {
                if let Kind::Button { action: Some(a), enabled: true, .. } = &ui.nodes[p].kind {
                    actions.push(a.clone());
                }
            }
            self.quads_dirty = true;
        }
        if let Some(f) = ui.focused {
            if let Kind::Input { value, caret, max, on_change } = &mut ui.nodes[f].kind {
                let outcome = crate::ui::edit::Field { value, caret, anchor: &mut ui.anchor, max: *max }.handle(&self.input);
                let (changed, moved) = (outcome.changed, outcome.moved);
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
            }
        }
        UiCallbacks { actions, slider, text_changed }
    }

    /// Recipes of (or uses for) the item under the cursor, in any screen.
    fn recipe_keys(&mut self, requests: &mut Vec<Request>) {
        let menu = self.game.is_some() && ui_shows_menu(&self.screens);
        let Some((_, ui)) = self.screens.last_mut() else { return };
        let typing = ui
            .focused
            .is_some_and(|f| matches!(ui.nodes[f].kind, Kind::Input { .. }));
        for (action, uses) in [("core:recipes", false), ("core:uses", true)] {
            if !typing && self.input.pressed(action) {
                match crate::game_screens::hovered_item(ui, self.game.as_ref().map(|g| &g.client)) {
                    Some(item) => requests.push(Request::Open(Box::new(crate::game_screens::RecipeView::item(item, uses)))),
                    // Nothing under the mouse in a menu: every recipe, by kind (the HUD shows R).
                    None if menu => {
                        let mut kinds: Vec<String> = content().game.recipe_kinds.iter().cloned().collect();
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
        let ran = !actions.is_empty() || slider.is_some() || text_changed.is_some();
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
            if let Some(&click) = content().game.sound_names.get("core:ui.click") {
                self.ui_sounds
                    .push(crate::audio::Play { sound: click, pos: None, volume: 1.0, pitch: 0.0 });
            }
        }
        for a in actions {
            a(&mut ctx);
        }
        if let Some((f, v)) = slider {
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
    fn client_hooks(&mut self, hook: u32, screen: &str) {
        let hooks: Vec<(abi::ModId, abi::Callback)> = content()
            .game
            .client_hooks
            .iter()
            .filter(|h| h.1 == hook)
            .map(|h| (h.0, h.2))
            .collect();
        for (owner, cb) in hooks {
            let mut v = self.frame_view();
            if hook == abi::client_hook::FRAME {
                let mut a = abi::ClientFrameArgs { client: v.raw() };
                crate::host::invoke(owner, cb, &mut a);
            } else {
                let mut a = abi::ClientScreenArgs { client: v.raw(), screen: abi::Str::new(screen) };
                crate::host::invoke(owner, cb, &mut a);
            }
        }
    }

    /// Every entity's cosmetics this frame: its kind's mod tick or the engine's; finished deaths
    /// go after their last call.
    fn entity_ticks(&mut self, dt: f32) {
        let Some(g) = &self.game else { return };
        let c = content();
        let all: Vec<abi::ClientEntityArgs> = g
            .client
            .entities
            .iter()
            .filter(|(_, e)| e.item.is_empty() && e.structure.is_none())
            .map(|(&id, e)| e.args(id, dt))
            .collect();
        for mut a in all {
            match c.game.client_entity_ticks.get(&a.kind) {
                Some(&(owner, cb)) => {
                    let mut v = self.frame_view();
                    a.client = v.raw();
                    crate::host::invoke(owner, cb, &mut a);
                }
                None => {
                    if let Some(g) = &mut self.game {
                        g.client.entity_effects(&a);
                    }
                }
            }
        }
        // Who only watches, every one of them; the local player is an entity to them as well.
        if !c.game.client_entity_watchers.is_empty() {
            let mine = self
                .game
                .as_ref()
                .map(|g| g.client.me.args(g.client.player_id, dt));
            let others = self.game.as_ref().map_or(Vec::new(), |g| {
                g.client
                    .entities
                    .iter()
                    .filter(|(_, e)| e.item.is_empty() && e.structure.is_none())
                    .filter(|(_, e)| c.game.client_entity_watchers.contains_key(&e.kind))
                    .map(|(&id, e)| e.args(id, dt))
                    .collect()
            });
            for mut a in others.into_iter().chain(mine) {
                for &(owner, cb) in c
                    .game
                    .client_entity_watchers
                    .get(&a.kind)
                    .into_iter()
                    .flatten()
                {
                    let mut v = self.frame_view();
                    a.client = v.raw();
                    crate::host::invoke(owner, cb, &mut a);
                }
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
        let s = self.settings.clone();
        let mut now = Applied {
            vsync: s.vsync(),
            view: s.view_distance(),
            language: s.language(),
            gui_scale: s.gui_scale(),
            packs: (s.resource_packs(), s.shader_pack()),
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
        s.apply_binds(&mut self.input);
        if relang || now.gui_scale != self.applied.gui_scale {
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
            for n in &mut ui.nodes {
                if let Some(tip) = &mut n.tooltip {
                    redraw |= text::refresh(tip, &test);
                }
                match &mut n.kind {
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
                    Kind::Icons { bind, value, shake, effects, change, .. }
                        if !effects.is_empty() || shake.is_some_and(|s| (0.0..=s).contains(value)) =>
                    {
                        let v = bind(&ctx);
                        if (v - *value).abs() > 1e-4 && *value >= 0.0 && v >= 0.0 {
                            *change = Some((*value, crate::shaders::clock(), v > *value));
                        }
                        *value = v;
                        for e in effects.iter_mut() {
                            e.value = e.bind.as_ref().map_or(1.0, |b| b(&ctx));
                        }
                        redraw = true;
                    }
                    Kind::Progress { bind, value, .. } | Kind::Icons { bind, value, .. } | Kind::BoundImage { bind, value, .. } => {
                        let v = bind(&ctx);
                        if (v - *value).abs() > 1e-4 {
                            *value = v;
                            redraw = true;
                        }
                    }
                    Kind::Slider { bind, value, .. } => {
                        let v = bind(&ctx);
                        if v.is_finite() && (v - *value).abs() > 1e-4 {
                            *value = v;
                            redraw = true;
                        }
                    }
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

/// `{if:name}`: `debug` and `dev`, else a registered condition (`Host::reg_condition`, game
/// modes by name); unknown ones are false.
fn text_condition(name: &str, client: Option<&Client>, debug: bool, dev: bool) -> bool {
    match name {
        "debug" => debug,
        "dev" => dev,
        _ => {
            // The engine's go without their namespace: `{if:creative}`.
            let names = &content().game.condition_names;
            let id = names.get(name).or_else(|| {
                (!name.contains(':'))
                    .then(|| names.get(&format!("core:{name}")))
                    .flatten()
            });
            id.is_some_and(|&id| crate::conditions::on_client(client, id))
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if let Some(r) = &mut self.run {
            // Android: back from the background, with a new surface for the same window.
            if self.suspended {
                self.suspended = false;
                el.set_control_flow(ControlFlow::Poll);
                if let Err(e) = r.gpu.surface_restored(&*r.window) {
                    log::error!("{e}");
                }
                let size = r.window.inner_size();
                r.gpu.resize(size.width, size.height);
                r.rebuild_all();
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

    /// Android sends the app to the background: its surface goes. The world pauses, as when the
    /// window loses the focus.
    fn suspended(&mut self, el: &ActiveEventLoop) {
        let Some(r) = &mut self.run else { return };
        self.suspended = true;
        el.set_control_flow(ControlFlow::Wait);
        r.touch.release_all(&mut r.input);
        r.input.release_all();
        if r.game.is_some() && r.screens.is_empty() {
            r.open(screens::engine("core:pause"));
        }
        r.set_capture(false);
        if let Some(server) = r.game.as_ref().and_then(|g| g.server.as_ref()) {
            server.save_now();
        }
        // Android ends a background app without the exit that saves it otherwise.
        save_pipeline_cache(&*r.gpu);
        r.gpu.surface_lost();
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
            WindowEvent::Focused(true) if !self.opts.automated && r.input.captured && !r.settings.read_only => {
                r.center_cursor();
                r.grab_cursor();
            }
            WindowEvent::Focused(false) if !self.opts.automated => {
                r.input.release_all();
                if r.game.is_some() && r.screens.is_empty() {
                    r.open(screens::engine("core:pause"));
                }
                r.set_capture(false);
            }
            WindowEvent::Touch(t) if crate::touch::enabled() => {
                let play = r.game.is_some() && r.screens.is_empty();
                let (w, h) = r.gpu.surface_size();
                let scale = r.gui_scale();
                let hud = &r.hud;
                let slot_at = |p: Vec2| hud.hotbar_slot_at(p);
                let mut cx = crate::touch::Cx { input: &mut r.input, play, scale, size: Vec2::new(w as f32, h as f32), slot_at: &slot_at };
                let at = Vec2::new(t.location.x as f32, t.location.y as f32);
                if r.touch.event(t.phase, t.id, at, &mut cx) && !r.input.captured {
                    r.set_capture(true);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Android's back button is Escape: out of a screen, or to the pause screen.
                // (winit has no physical key for it, only the logical BrowserBack.)
                let back =
                    cfg!(target_os = "android") && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::BrowserBack);
                let physical = if back { PhysicalKey::Code(KeyCode::Escape) } else { event.physical_key };
                if let PhysicalKey::Code(code) = physical {
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
                r.input.scroll += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
                };
            }
            WindowEvent::CursorMoved { position, .. } => {
                r.input.cursor = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::RedrawRequested if self.suspended => {}
            WindowEvent::RedrawRequested => {
                if frame(r, &self.opts) || r.quit {
                    r.leave_world_silent();
                    r.close_home();
                    el.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        #[cfg(target_os = "android")]
        if ANDROID_DESTROYED.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!("android: the activity was destroyed, saving and ending");
            if let Some(r) = &mut self.run {
                r.leave_world_silent();
                r.close_home();
            }
            _el.exit();
            return;
        }
        if let Some(r) = &self.run
            && !self.suspended
        {
            r.window.request_redraw();
        }
    }

    fn exiting(&mut self, _el: &ActiveEventLoop) {
        finish_screenshots();
        if let Some(mut r) = self.run.take() {
            r.gpu.wait_idle();
            r.leave_world_silent();
            r.close_home();
            save_pipeline_cache(&*r.gpu);
            drop(r.gpu);
        }
    }
}

/// What a widget does, run once the screen's `Ui` is let go.
#[derive(Default)]
struct UiCallbacks {
    actions: Vec<crate::ui::Action>,
    slider: Option<(std::rc::Rc<dyn Fn(&mut Ctx, f32)>, f32)>,
    text_changed: Option<(std::rc::Rc<dyn Fn(&mut Ctx, &str)>, String)>,
}

/// One frame. Returns true when the app should quit.
fn frame(r: &mut Running, opts: &Options) -> bool {
    r.retired = None;
    // The loading screen has been shown (two frames, so one was presented): now the slow start.
    if let Some((meta, shown)) = r.pending_world.take() {
        if shown >= 2 {
            r.start_world(meta);
        } else {
            r.pending_world = Some((meta, shown + 1));
        }
    }
    // The fps cap: what is left of this frame's share of a second, slept away.
    let cap = r.settings.fps_cap();
    if cap > 0 {
        let share = std::time::Duration::from_secs_f64(1.0 / cap as f64);
        if let Some(rest) = share.checked_sub(r.last.elapsed()) {
            std::thread::sleep(rest);
        }
    }
    let now = Instant::now();
    let dt = (now - r.last).as_secs_f64();
    r.last = now;
    let scale = r.gui_scale();
    start_frame(r, opts, dt);
    let mut requests = Vec::new();
    if !screen_keys_first(r, &mut requests) && r.input.pressed_key(Key::Code(KeyCode::Escape)) {
        escape(r, &mut requests);
    }
    f3_keys(r);
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
    if r.reload_requested {
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
    r.screen_input(scale, &mut requests);
    r.handle(requests);
    update_game(r, dt, now);
    r.entity_ticks(dt as f32);
    r.client_packets();
    play_sounds(r);
    client_events(r);
    world_keys(r);
    r.poll_join();
    handle_disconnect(r);
    end_input(r);
    refresh_hud(r);
    r.poll_bindings();
    let text_unit = layout_uis(r, scale);
    let t = Instant::now();
    r.prof[1] = (t.duration_since(now).as_secs_f32() * 1000.0 - r.prof[1]).max(0.0);
    let Some(info) = r.gpu.begin_frame() else { return false };
    let t2 = Instant::now();
    r.prof[2] = t2.duration_since(t).as_secs_f32() * 1000.0;
    let world = world_frame(r, &info);
    if (r.depth.1, r.depth.2) != (world.width, world.height) {
        log::info!(
            "world drawn at {}x{} of the window's {}x{} (render scale {:.2}, density {:.1})",
            world.width,
            world.height,
            info.width,
            info.height,
            crate::settings::render_scale(r.window.scale_factor()),
            r.window.scale_factor()
        );
        r.gpu.destroy_texture(r.depth.0);
        r.depth = make_depth(&mut *r.gpu, world.width, world.height);
    }
    if r.quads_dirty {
        let _s = crate::diag::span("ui quads");
        build_ui_quads(r, scale);
    }
    let s = crate::diag::span("overlays");
    draw_overlays(r, &info, scale, text_unit);
    drop(s);
    record_passes(r, &info, &world, scale);
    let capture_now = r.shot_pending;
    let t3 = Instant::now();
    r.prof[3] = t3.duration_since(t2).as_secs_f32() * 1000.0;
    crate::diag::slow_record(r.prof[3]);
    r.gpu.end_frame(&r.list);
    r.prof[4] = t3.elapsed().as_secs_f32() * 1000.0;
    report_frame(r, dt);
    if capture_now {
        save_screenshot(r);
    }
    count_fps(r);
    false
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
    r.release_mod_keys();
    r.mod_gizmos.clear();
    r.mod_screen_text.clear();
    r.client_hooks(abi::client_hook::FRAME, "");
    if opts.automated && r.screens.is_empty() && r.game.is_some() {
        // The window may not have focus while a mod drives it.
        r.input.captured = true;
    }
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
        r.open(screens::engine("core:pause"));
    }
}

type F3Action = fn(&mut Running);

/// What F3 plus a key does. A new combination is a line here.
const F3_COMBOS: &[(KeyCode, F3Action)] = &[
    (KeyCode::KeyT, |r| r.reload_requested = true),
    // Through the command, so permissions still apply.
    (KeyCode::F4, |r| {
        if let Some(g) = &mut r.game {
            let all: Vec<_> = crate::world::GameMode::all().collect();
            let next = all[(all.iter().position(|&m| m == g.client.mode).unwrap_or(0) + 1) % all.len()];
            g.client
                .send(net::ToServer::Chat(format!("/gamemode {}", next.name())));
        }
    }),
    (KeyCode::F6, |r| {
        if r.screens.is_empty() {
            r.open(screens::engine("core:debug_options"));
        }
    }),
    (KeyCode::KeyG, |r| toggle_gizmo(r, "cubes")),
    (KeyCode::KeyB, |r| toggle_gizmo(r, "hitboxes")),
];

fn toggle_gizmo(r: &mut Running, group: &str) {
    if let Some(g) = &mut r.game {
        g.client.toggle_gizmo(group);
    }
}

/// F3 alone toggles the overlay on release; F3 + a key does what `F3_COMBOS` says, F3 + a digit
/// toggles one group instead.
fn f3_keys(r: &mut Running) {
    let f3 = r
        .input
        .binding_of("core:debug")
        .is_some_and(|b| r.input.held_key(b.chord.key));
    if r.input.pressed("core:debug") {
        r.f3_combo = Some(false);
    }
    if f3 {
        for (key, action) in F3_COMBOS {
            if r.input.pressed_key(Key::Code(*key)) {
                action(r);
                r.f3_combo = Some(true);
            }
        }
        let digits = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ];
        let groups = crate::hud::DEBUG_GROUPS.len() + crate::hud::mod_debug_groups().len();
        for (i, k) in digits.into_iter().take(groups).enumerate() {
            if r.input.pressed_key(Key::Code(k)) {
                r.settings
                    .set("debug_groups", (r.settings.debug_groups() ^ (1 << i)) as f64);
                r.f3_combo = Some(true);
                r.debug = true;
                r.hud.invalidate();
            }
        }
    } else if let Some(combo) = r.f3_combo.take() {
        if !combo {
            r.debug = !r.debug;
            r.hud.invalidate();
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

fn reload_assets(r: &mut Running, _opts: &Options) {
    r.reload_assets();
}

fn window_keys(r: &mut Running) {
    if r.input.pressed("core:fullscreen") {
        let on = r.window.fullscreen().is_none();
        r.window
            .set_fullscreen(on.then_some(winit::window::Fullscreen::Borderless(None)));
        r.settings.set("fullscreen", on as u32 as f64);
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
        g.client.fov = r.settings.fov();
        g.client.brightness = r.settings.brightness();
        g.client.first_person_model = r.settings.first_person_model();
        g.client.entity_render.show_avatars = r.settings.show_avatars();
        g.client.render.occlusion.on = r.settings.occlusion_culling();
        g.client.sky.soft_wanted = r.settings.soft_clouds();
        r.post.use_low = r.settings.post_low();
        r.post.use_taa = r.settings.aa();
        r.post.taa_blend = r.settings.taa_blend();
        r.post.bloom_levels = r.settings.bloom_levels();
        // The mist volume with the HDR world only, until today's (LDR) look takes it on Ronald's
        // word (its look was never shown to him there); `core:froxels` 0 turns it off there too.
        g.client.sky.froxels_wanted = r.settings.froxels() && crate::post::hdr();
        g.client.sky.froxels.cell = r.settings.froxel_cell();
        g.client.sky.cloud_shadow_wanted = crate::post::hdr() && r.settings.cloud_shadows();
        let t = Instant::now();
        if world_input {
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
    if let Some(a) = &mut r.audio {
        let listener = r.game.as_ref().map(|g| g.client.listener());
        a.update(&r.assets, listener.as_ref());
        if let Some(g) = &mut r.game {
            for p in g.client.sounds.drain(..) {
                a.play(&r.assets, p);
            }
        }
        for p in r.ui_sounds.drain(..) {
            a.play(&r.assets, p);
        }
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
            crate::client::ClientEvent::Died => {
                r.screens.clear();
                r.open(screens::engine("core:death"));
            }
        }
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
        let f3_held = r
            .input
            .binding_of("core:debug")
            .is_some_and(|b| r.input.held_key(b.chord.key));
        if command || (r.input.pressed("core:chat") && !f3_held) {
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
                let mut kinds: Vec<String> = content().game.recipe_kinds.iter().cloned().collect();
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
    r.input.sensitivity = r.settings.sensitivity();
    r.input.invert_y = r.settings.invert_y();
    r.input.end_frame();
    let top = (
        r.screens.len(),
        r.screens
            .last()
            .map(|(s, _)| s.name().to_string())
            .unwrap_or_default(),
    );
    if top != r.screen_seen {
        let name = top.1.clone();
        r.screen_seen = top;
        r.screen_frames = 0;
        r.client_hooks(abi::client_hook::SCREEN, &name);
    } else {
        r.screen_frames = r.screen_frames.saturating_add(1);
    }
}

fn refresh_hud(r: &mut Running) {
    if let Some(g) = &r.game {
        let chat_open = r.screens.last().is_some_and(|(s, _)| s.shows_chat());
        let menu_open = !r.screens.is_empty() && !chat_open;
        if r.hud.editing()
            && !r
                .screens
                .last()
                .is_some_and(|(s, _)| s.name() == "core:hud_editor")
        {
            r.hud.set_edit(None);
        }
        if r.hud
            .update(&g.client, r.debug, r.settings.debug_groups(), chat_open, menu_open, &r.lang, &r.input)
        {
            r.quads_dirty = true;
        }
        let mut h = g.client.selected as u64;
        for s in g.client.inventory.iter().chain(
            g.client
                .menu
                .iter()
                .flat_map(|m| m.slots.iter().chain([&m.cursor])),
        ) {
            h = crate::world::mix(h ^ ((s.item as u64) << 32 | s.count as u64) ^ (s.data as u64) << 48);
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
    {
        let mut uis: Vec<&mut Ui> = Vec::new();
        if r.game.is_some() {
            uis.push(&mut r.hud.ui);
        }
        uis.extend(r.screens.iter_mut().map(|(_, u)| u));
        for ui in uis {
            if ui.dirty || ui.size != gui_size || ui.text_unit != text_unit {
                ui.text_unit = text_unit;
                ui.layout(gui_size, &r.ui.font);
                ui.dirty = false;
                r.quads_dirty = true;
            }
        }
    }
    text_unit
}

fn build_ui_quads(r: &mut Running, scale: f32) {
    let mut all = Vec::new();
    let game = crate::content::try_content().map(|c| (c, &r.textures));
    let client = r.game.as_ref().map(|g| &g.client);
    let mut previews = Vec::new();
    let preview_tex = r.preview_tex;
    let mut layers: Vec<&Ui> = Vec::new();
    let mut animated = Vec::new();
    let hud = r
        .game
        .as_ref()
        .is_some_and(|g| g.client.camera_has(weft_abi::camera_flags::HUD))
        || !r.screens.is_empty();
    if r.game.is_some() && hud && !hud_hidden() && !(r.hud_off && r.screens.is_empty()) {
        layers.push(&r.hud.ui);
    }
    // Only the top screen shows, like vanilla: the ones under it are not drawn through its
    // see-through overlay.
    layers.extend(r.screens.last().map(|(_, u)| u));
    for ui in layers {
        let look = &r.look;
        let textures = &r.textures;
        let unit = ui.text_unit;
        let mut custom = |ren: &mut UiRenderer, tag: u32, data: u64, rect: crate::ui::Rect, hovered: bool, icon: Option<u32>| {
            crate::hud::draw_custom(ren, look, textures, client, tag, data, rect, hovered, icon, scale, unit);
            if tag == crate::client::modui::TAG_ENTITY {
                let inner = crate::ui::Rect { x: rect.x + scale, y: rect.y + scale, w: rect.w - 2.0 * scale, h: rect.h - 2.0 * scale };
                previews.push(([inner.x, inner.y, inner.w, inner.h], data));
                if let Some(t) = preview_tex {
                    ren.image(inner, t, [0.0, 0.0, 1.0, 1.0], 0xFFFFFFFF);
                }
            }
        };
        let base = all.len();
        r.ui.build(&mut *r.gpu, &r.assets, ui, &r.look, scale, game, &mut custom);
        all.append(&mut r.ui.quads);
        animated.extend(r.ui.animated.drain(..).map(|mut a| {
            a.quads = a.quads.start + base..a.quads.end + base;
            a
        }));
    }
    r.ui_cached = all;
    r.ui_animated = animated;
    r.previews = previews;
}

fn draw_overlays(r: &mut Running, info: &weft_render::FrameInfo, scale: f32, text_unit: f32) {
    // The stack held by the mouse follows it every frame, outside the cached quads.
    r.ui.quads.clear();
    r.ui.quads.extend_from_slice(&r.ui_cached);
    crate::ui::render::animate(&mut r.ui.quads, &r.ui_animated);
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
    // A touch screen's controls, over play (`touch`).
    if crate::touch::enabled() && r.game.is_some() && r.screens.is_empty() && !hud_hidden() {
        let size = Vec2::new(sw as f32 / scale, sh as f32 / scale);
        let mut o = crate::ui::Ui::new();
        o.text_unit = text_unit;
        let (lang, input) = (&r.lang, &r.input);
        r.touch
            .draw(&mut o, size, scale, &|key| Msg::new(key, Vec::new()).spans(lang, Some(input)));
        o.layout(size, &r.ui.font);
        let game = crate::content::try_content().map(|c| (c, &r.textures));
        r.ui.append(&mut *r.gpu, &r.assets, &o, &r.look, scale, game, &mut |_, _, _, _, _, _| {});
    }
    if let Some((_, ui)) = r.screens.last() {
        let game = crate::content::try_content().map(|c| (c, &r.textures));
        r.ui.tooltip(&mut *r.gpu, &r.assets, ui, &r.look, scale, r.input.cursor, game);
    }
    if let Some(cursor) = r
        .game
        .as_ref()
        .and_then(|g| g.client.menu.as_ref())
        .map(|m| m.cursor)
        .filter(|s| !s.is_empty())
    {
        let p = r.input.cursor;
        let rect = crate::ui::Rect { x: p.x - 9.0 * scale, y: p.y - 9.0 * scale, w: 18.0 * scale, h: 18.0 * scale };
        crate::hud::stack_icon(&mut r.ui, &r.textures, rect, cursor, scale, text_unit);
    }
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
    if let Some(g) = &mut r.game {
        let worn = g.client.worn();
        let worn_looks = g.client.worn_looks();
        let held = [g.client.shown_held(), g.client.off_held()];
        g.client
            .entity_render
            .preload(&mut *r.gpu, &r.assets, &g.client.entities, &g.client.entity_components, &worn, &worn_looks, held);
        if g.client
            .entity_render
            .draw_icons(&mut *r.gpu, &mut r.list, info)
        {
            r.quads_dirty = true;
        }
    }
    if let Some(g) = r.game.as_mut().filter(|_| !r.screens.is_empty()) {
        if let Some(&([x, y, w, h], whose)) = r.previews.first() {
            let worn = g.client.worn();
            let worn_looks = g.client.worn_looks();
            let look = r.input.cursor - Vec2::new(x, y);
            // The menu's own entity, or a kind alone (no entity: its own texture, nothing painted).
            let subject = match whose {
                1 => g
                    .client
                    .menu
                    .as_ref()
                    .map(|m| m.owner)
                    .and_then(|id| g.client.entities.get(&id).map(|e| (id, e.kind))),
                k if k >= 2 => Some((u64::MAX, (k - 2) as u32)),
                _ => None,
            };
            // A kind alone: its skin loads with the others (none of it may be in the world).
            g.client.entity_render.preview_kind = (whose >= 2).then(|| (whose - 2) as u32);
            let s = crate::diag::span("preview");
            let tex = g.client.entity_render.draw_preview(
                &mut *r.gpu,
                &mut r.list,
                info,
                [w.round() as u32, h.round() as u32],
                look,
                &worn,
                &worn_looks,
                subject,
            );
            drop(s);
            if tex != r.preview_tex {
                r.preview_tex = tex;
                r.quads_dirty = true;
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
    let post = (r.post.on() || scaled || crate::post::hdr() || r.post.taa_on()) && r.game.is_some();
    // Gathered over frames (temporal anti-aliasing), the world is drawn a fraction of a pixel
    // aside each frame, and its pass draws the motion image too (whenever the pipelines have it).
    crate::post::next_jitter(post && r.post.taa_on(), world_info.width, world_info.height);
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
    let mut world = std::mem::take(&mut r.world_list);
    world.clear();
    if let Some(g) = &mut r.game {
        let s = crate::diag::span("client render");
        g.client
            .render(&mut *r.gpu, &mut world, &mut r.list, world_info, &r.textures);
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
        // The opaque world copied, then what is seen through it drawn over it, reading the copy.
        if g.client.behind[2] > 0.5 {
            world.push(Cmd::EndPass);
            if let Some(scene) = g.client.scene_buffer(info.slot) {
                let depth = r.gpu.texture_index(r.depth.0);
                r.post
                    .copy_behind(&mut *r.gpu, &mut world, world_info, scene, depth);
            }
            // Soft clouds traced against that copy, before the sky draws them (chiacraft).
            world.append(&mut g.client.sky.soft.passes);
            world.push(Cmd::BeginPass {
                motion: motion.map(|m| (m, None)),
                color: Some((world_target, None)),
                depth: Some((r.depth.0, None)),
            });
            world.append(&mut g.client.late_out);
            // The "translucent" part ends here: the copy, the sky, clouds, water, particles and the
            // hand (the client leaves this mark to the app when the world is copied).
            world.push(Cmd::Mark);
        }
    }
    r.list.push(Cmd::Mark);
    r.list.push(Cmd::BeginPass {
        // A depth of -1: no surface wrote its own motion there (`taa.wgsl`).
        motion: motion.map(|m| (m, Some([0.0, 0.0, -1.0, 0.0]))),
        color: Some((world_target, Some(clear))),
        depth: Some((r.depth.0, Some(0.0))),
    });
    r.list.append(&mut world);
    r.world_list = world;
    r.list.push(Cmd::EndPass);
    if let Some(g) = &mut r.game {
        g.client
            .render
            .occlusion
            .take_depth(&mut *r.gpu, &mut r.list, r.depth.0, [r.depth.1, r.depth.2]);
    }
    if post {
        let scene = r
            .game
            .as_ref()
            .and_then(|g| g.client.scene_buffer(info.slot));
        let depth = r.gpu.texture_index(r.depth.0);
        r.post.set_depth(r.depth.0);
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
                    "pos {:.1} {:.1} {:.1}, cubes {}, draws {}, faces {}k, entities {}, regions {}",
                    g.client.player.pos.x,
                    g.client.player.pos.y,
                    g.client.player.pos.z,
                    g.client.world.len(),
                    g.client.render.drawn,
                    g.client.render.faces_drawn / 1000,
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
