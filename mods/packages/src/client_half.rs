//! A package's sandboxed client half (chiacraft plan, phase 6, slice 4; `docs/sandboxed-mods.md`,
//! "Client halves"): `code/client.wasm`, run by `chiacraft-sandbox` in each player's own game. Its
//! HUD elements, screens, keys, packets and frame hooks become this mod's through Weft's SDK, in
//! the package's slot. A HUD element or screen asks the client half for its parts and replays
//! them into Weft's `Ui`, so it draws with the game's own kit; bars, icons and text bindings ask
//! it for their values every frame, and buttons call it back.

use std::sync::{Arc, Mutex};

use chiacraft_sandbox::client::{ClientGame, ClientHook, ClientInstance, ClientMod, Part, PartKind, Shared};
use chiacraft_sandbox::{self as sandbox, Arg as WArg};
use weft_sdk::abi::{self, Str};
use weft_sdk::*;

/// A thing by its full name, as the SDK's handle.
type Named<T> = (String, &'static T);

/// The client half of one package in one content build: its instance, and the channels and
/// screens it names, as the SDK's handles.
pub(crate) struct Half {
    pub(crate) inst: Shared,
    channels: Mutex<Vec<Named<Channel<Vec<u8>>>>>,
    screens: Vec<(String, &'static Screen)>,
    /// A personal mod (slice 5): its own lines, worded here from its `assets/lang` (it is no
    /// content, so the game's language files don't have them), and no channels or components.
    pub(crate) personal: Option<crate::personal::Words>,
}

impl Half {
    /// A personal mod's client half.
    pub(crate) fn personal(m: Arc<ClientMod>, words: crate::personal::Words) -> Half {
        Half {
            inst: Arc::new(Mutex::new(ClientInstance::new(m))),
            channels: Mutex::new(Vec::new()),
            screens: Vec::new(),
            personal: Some(words),
        }
    }
}

impl Half {
    fn channel(&self, name: &str) -> &'static Channel<Vec<u8>> {
        let mut channels = self.channels.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, c)) = channels.iter().find(|(n, _)| n == name) {
            return c;
        }
        let c = Channel::generated(name.to_string());
        channels.push((name.to_string(), c));
        c
    }
}

/// Loads `dir/code/client.wasm`, if the package has one, and registers what it registers.
pub fn load(r: &mut Registry, package: &str, dir: &std::path::Path) -> Result<(), String> {
    let path = dir.join(chiacraft_packages::identity::CLIENT_CODE);
    if !path.is_file() {
        return Ok(());
    }
    let shown = chiacraft_packages::identity::CLIENT_CODE;
    let wasm = std::fs::read(&path).map_err(|e| format!("{shown}: {e}"))?;
    let m = Arc::new(ClientMod::load(package, &wasm).map_err(|e| format!("{shown}: {e}"))?);
    let screens: Vec<(String, &'static Screen)> = m
        .hooks()
        .iter()
        .filter_map(|h| match h {
            ClientHook::Screen { name, .. } => Some((name.clone(), Screen::generated(name.clone()))),
            _ => None,
        })
        .collect();
    let half = Arc::new(Half {
        inst: Arc::new(Mutex::new(ClientInstance::new(m.clone()))),
        channels: Mutex::new(Vec::new()),
        screens,
        personal: None,
    });
    for (i, hook) in m.hooks().iter().enumerate() {
        let cb = i as u32;
        let h = half.clone();
        match hook {
            ClientHook::Hud { name, anchor, order } => {
                r.hud_element(HudElement::generated(name.clone()), anchor, *order, move |ui| build(ui, &h, cb));
            }
            ClientHook::Screen { name, pauses } => {
                let screen = half
                    .screens
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, s)| *s)
                    .unwrap_or_else(|| Screen::generated(name.clone()));
                let b = r.screen(screen, move |ui| build(ui, &h, cb));
                if *pauses {
                    drop(b.pauses());
                }
            }
            ClientHook::Key { name, default } => {
                // A key's binding is a setting: the engine carries settings over by name when
                // the content switches (`config::content_switched`), so a package may add one.
                r.key(Key::generated(name.clone()), default)
                    .on_press(move |c| {
                        let g = game(*c, true, &h);
                        lock(&h).key(g, cb)
                    });
            }
            ClientHook::Packet(channel) => {
                let ch = half.channel(channel);
                r.client_packet(ch, move |c, data| {
                    if data.len() <= sandbox::PACKET_BYTES {
                        let g = game(*c, true, &h);
                        lock(&h).packet(g, cb, &data)
                    }
                });
            }
            ClientHook::Frame => {
                let last = Mutex::new(std::time::Instant::now());
                r.client_frame(move |c| {
                    let seconds = {
                        let mut l = last.lock().unwrap_or_else(|e| e.into_inner());
                        let s = l.elapsed().as_secs_f32();
                        *l = std::time::Instant::now();
                        s
                    };
                    let g = game(*c, true, &h);
                    lock(&h).frame(g, cb, seconds)
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn lock(h: &Half) -> std::sync::MutexGuard<'_, ClientInstance> {
    h.inst.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn game(c: Client, act: bool, h: &Arc<Half>) -> Box<dyn ClientGame> {
    Box::new(PlayerGame { c, act, half: h.clone() })
}

// ------------------------------------------------------------------ parts into Weft's Ui

/// Class names a client half uses, kept for the game's life (the SDK's styles hold `'static`
/// names); at most a few hundred, from the mods' own code.
fn class(name: &str) -> &'static str {
    static CLASSES: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
    let mut classes = CLASSES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(c) = classes.iter().find(|c| **c == name) {
        return c;
    }
    if classes.len() >= 1024 {
        return "";
    }
    let c: &'static str = Box::leak(name.to_string().into_boxed_str());
    classes.push(c);
    c
}

fn style(p: &Part) -> S {
    let s = &p.style;
    let mut out = if s.row { S::row() } else { S::col() };
    out.width = s.width;
    out.height = s.height;
    out.gap = s.gap;
    out.pad = s.pad;
    out.grow = s.grow;
    out.align = s.align.min(3);
    out.justify = s.justify.min(3);
    out.scale = s.scale.clamp(0.25, 8.0);
    // Only the flags a part may set itself.
    out.flags = s.flags & (abi::ui_flags::NO_WRAP | abi::ui_flags::MONO | abi::ui_flags::DISABLED | abi::ui_flags::SELECTED);
    out.class = class(&s.class);
    out.at = s.at.map(|a| [a.0, a.1, a.2, a.3]);
    out
}

/// A line's key and arguments as the UI takes them: a personal mod's worded here, as one text.
fn worded(h: &Half, key: &str, a: &[WArg]) -> (String, Vec<WArg>) {
    match h.personal.as_ref().and_then(|w| w.line(key, a)) {
        Some(text) => ("packages.line".to_string(), vec![WArg::Text(text)]),
        None => (key.to_string(), a.to_vec()),
    }
}

/// A part's arguments as the SDK's (texts borrowed from the part).
fn args(a: &[WArg]) -> Vec<Arg<'_>> {
    a.iter()
        .map(|a| match a {
            WArg::Text(s) => Arg::Text(s),
            WArg::Key(s) => Arg::Key(s),
            WArg::Int(v) => Arg::Int(*v),
            WArg::Float((v, d)) => Arg::Float(*v, *d),
            WArg::Item(v) => Arg::Item(*v),
            WArg::Name(s) => Arg::Name(s),
        })
        .collect()
}

pub(crate) fn build(ui: &mut Ui, h: &Arc<Half>, cb: u32) {
    let view = ui.client();
    let parts = lock(h).build(game(view.client(), false, h), cb);
    let mut i = 0;
    while i < parts.len() {
        i = replay(ui, &parts, i, h);
    }
}

/// Puts part `i` and its children into `ui`; the index after them.
fn replay(ui: &mut Ui, parts: &[Part], i: usize, h: &Arc<Half>) -> usize {
    let p = &parts[i];
    let s = style(p);
    let children = |ui: &mut Ui| {
        let mut next = i + 1;
        for _ in 0..p.children {
            next = replay(ui, parts, next, h);
        }
    };
    let mut after = i + 1;
    for _ in 0..p.children {
        after = sandbox_skip(parts, after);
    }
    let bind = p.bind;
    match p.kind {
        PartKind::Container if p.style.row => ui.row(s, children),
        PartKind::Container => ui.column(s, children),
        PartKind::Panel => ui.panel(class(&p.class), s, children),
        PartKind::Text => {
            let (key, a) = worded(h, &p.text, &p.args);
            ui.text_styled(s, &key, &args(&a))
        }
        PartKind::TextBind => {
            let h = h.clone();
            ui.text_bind_styled(s, move |view, out| {
                let (key, a) = lock(&h).text(game(view.client(), false, &h), bind);
                let (key, a) = worded(&h, &key, &a);
                out.set(&key, &args(&a));
            });
        }
        PartKind::Progress => {
            let h = h.clone();
            ui.progress_bind_dir(class(&p.class), s, p.count.min(3), move |view| lock(&h).value(game(view.client(), false, &h), bind));
        }
        PartKind::Icons => {
            let h = h.clone();
            ui.icons(class(&p.class), p.count.min(64), move |view| lock(&h).value(game(view.client(), false, &h), bind));
        }
        PartKind::Image => ui.image(class(&p.class), p.size.0.clamp(0.0, 1024.0), p.size.1.clamp(0.0, 1024.0)),
        PartKind::Item => {
            if !item_name(p.item).is_empty() {
                ui.item(p.item, p.count.min(9999), s)
            }
        }
        PartKind::Spacer => ui.spacer(),
        PartKind::Button => {
            let (key, a) = worded(h, &p.text, &p.args);
            let h = h.clone();
            ui.button_args(&key, &args(&a), s, move |view| lock(&h).click(game(view.client(), true, &h), bind));
        }
    }
    after
}

fn sandbox_skip(parts: &[Part], i: usize) -> usize {
    let mut next = i + 1;
    for _ in 0..parts.get(i).map_or(0, |p| p.children) {
        next = sandbox_skip(parts, next);
    }
    next
}

// ------------------------------------------------------------------ the player's game

/// The player's game during one call into a client half.
struct PlayerGame {
    c: Client,
    /// A key, button, packet or frame: may act. A binding: may only read.
    act: bool,
    half: Arc<Half>,
}

// SAFETY: made for one callback on the thread running it and dropped before it returns.
unsafe impl Send for PlayerGame {}

fn find(kind: u32, name: &str) -> Option<u32> {
    let id = unsafe { (host().find)(kind, Str::new(name), std::ptr::null_mut()) };
    (id != u32::MAX).then_some(id)
}

impl ClientGame for PlayerGame {
    fn me(&mut self) -> Option<([f64; 3], f32, f32)> {
        self.c.view().me().map(|e| (e.pos, e.health, e.max_health))
    }
    fn day_time(&mut self) -> u64 {
        self.c.view().day_time()
    }
    fn raining(&mut self) -> bool {
        self.c.view().raining()
    }
    fn creative(&mut self) -> bool {
        self.c.view().creative()
    }
    fn component(&mut self, name: &str) -> Option<Vec<u8>> {
        // A personal mod has no components of its own in any world.
        if self.half.personal.is_some() {
            return None;
        }
        // The player's own value, of the size its bucket has (`sandboxed::Comp`).
        crate::sandboxed::client_component(self.c, name)
    }
    fn inventory_slot(&mut self, slot: u32) -> sandbox::ItemStack {
        let s = self.c.view().inventory_slot(slot.min(255));
        sandbox::ItemStack { item: s.item, count: s.count, data: s.data }
    }
    fn screen(&mut self) -> String {
        self.c.screen()
    }
    fn can_act(&self) -> bool {
        self.act
    }
    fn send(&mut self, channel: &str, data: &[u8]) {
        // A personal mod has no server half to send to.
        if self.half.personal.is_none() {
            self.c.send(self.half.channel(channel), &data.to_vec());
        }
    }
    fn open_screen(&mut self, name: &str) {
        if self.half.personal.is_some() {
            crate::personal::open_screen(self.c, &self.half, name);
        } else if let Some((_, s)) = self.half.screens.iter().find(|(n, _)| n == name) {
            self.c.open_screen(s);
        }
    }
    fn close_screen(&mut self) {
        self.c.close_screen();
    }
    fn show(&mut self, key: &str, args: &[WArg]) {
        let (key, args) = worded(&self.half, key, args);
        crate::show_line(&self.c, crate::sandboxed::line_text(&key, &args));
    }

    fn block_name(&mut self, state: u32) -> String {
        block_name(state).to_string()
    }
    fn block_state(&mut self, name: &str) -> Option<u32> {
        find(abi::entry_kind::BLOCK, name)
    }
    fn prop(&mut self, state: u32, prop: &str) -> Option<String> {
        if block_name(state).is_empty() {
            return None;
        }
        state_prop(state, prop).map(str::to_string)
    }
    fn with_prop(&mut self, state: u32, prop: &str, value: &str) -> Option<u32> {
        if block_name(state).is_empty() {
            return None;
        }
        state_with(state, prop, value)
    }
    fn item_name(&mut self, id: u32) -> String {
        item_name(id).to_string()
    }
    fn item_id(&mut self, name: &str) -> Option<u32> {
        find(abi::entry_kind::ITEM, name)
    }
    fn entity_kind(&mut self, id: u32) -> String {
        name_of(abi::entry_kind::ENTITY, id).to_string()
    }
    fn entity_kind_id(&mut self, name: &str) -> Option<u32> {
        find(abi::entry_kind::ENTITY, name)
    }
}
