//! Personal mods (chiacraft plan, phase 6, slice 5; `docs/sandboxed-mods.md`, "Personal mods"):
//! a player's own client halves, from `<run dir>/personal/<package>/code/client.wasm`, never
//! downloaded and never shared. Each world's owner allows them or not with the world rule
//! `packages:personal_mods` (off by default; on Create World and in the world's settings). Where
//! it is off they don't run, and joining tells the player which of theirs are off.
//!
//! A personal mod is no content: it must not change what the world and the player's game agree
//! on (the join checks the content's hash). So it registers nothing of its own: this mod
//! registers, once and the same in every game, a HUD element per anchor, one screen and eight
//! keys (Personal Key 1 to 8, Shift+Alt+1 to 8, rebindable), and runs the personal mods' parts,
//! keys and frames through them. Their lines are worded here from their own `assets/lang`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use chiacraft_sandbox::Arg as WArg;
use chiacraft_sandbox::client::{ClientHook, ClientMod};
use weft_sdk::*;

use crate::client_half::{Half, build, game, lock};

/// The world rule: 0 off, 1 client halves.
pub static RULE: Config<u32> = Config::named("personal_mods");
static SCREEN: Screen = Screen::named("personal");
const KEYS: usize = 8;
static KEY_SLOTS: [Key; KEYS] = [
    Key::named("personal_1"),
    Key::named("personal_2"),
    Key::named("personal_3"),
    Key::named("personal_4"),
    Key::named("personal_5"),
    Key::named("personal_6"),
    Key::named("personal_7"),
    Key::named("personal_8"),
];
/// Personal mods a player may have.
const MAX: usize = 16;

/// A personal mod's lines, from its `assets/lang/<language>.lang` (English where its language
/// lacks one).
pub struct Words(HashMap<String, String>);

impl Words {
    fn load(dir: &Path) -> Words {
        let mut map = HashMap::new();
        let lang = settings::LANGUAGE.get();
        for file in ["en_us".to_string(), lang] {
            let path = dir.join("assets/lang").join(format!("{file}.lang"));
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            for line in text.lines().take(4096) {
                if let Some((k, v)) = line.split_once('=') {
                    map.insert(k.trim().to_string(), v.trim().chars().take(512).collect());
                }
            }
        }
        Words(map)
    }

    /// The line `key` worded with `args` ({0}, {1}, ...), if the mod has it.
    pub fn line(&self, key: &str, args: &[WArg]) -> Option<String> {
        let mut text = self.0.get(key)?.clone();
        for (i, a) in args.iter().enumerate() {
            let v = match a {
                WArg::Text(s) | WArg::Name(s) => s.clone(),
                WArg::Key(k) => self.0.get(k).cloned().unwrap_or_else(|| k.clone()),
                WArg::Int(v) => v.to_string(),
                WArg::Float((v, d)) => format!("{v:.*}", *d as usize),
                WArg::Item(v) => item_name(*v).to_string(),
            };
            text = text.replace(&format!("{{{i}}}"), &v);
        }
        Some(text)
    }
}

/// One personal mod: its name, its client half, what it registered.
struct Personal {
    name: String,
    half: Arc<Half>,
    hooks: Vec<ClientHook>,
}

/// The player's personal mods, loaded on the first frame of the game (there is a client then).
static MODS: OnceLock<Vec<Personal>> = OnceLock::new();
/// The personal screen showing: (mod, callback).
static SHOWING: Mutex<Option<(usize, u32)>> = Mutex::new(None);
/// The last world (by run of the rule's value) the player was told about.
static TOLD: Mutex<Option<u32>> = Mutex::new(None);

fn mods() -> &'static [Personal] {
    MODS.get().map_or(&[], Vec::as_slice)
}

/// Whether the world the player is in allows personal mods.
fn allowed() -> bool {
    RULE.get() == 1
}

fn load_all(run_dir: &Path) -> Vec<Personal> {
    let dir = run_dir.join("personal");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut dirs: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join(chiacraft_packages::identity::CLIENT_CODE).is_file())
        .collect();
    dirs.sort();
    let mut out = Vec::new();
    for d in dirs.into_iter().take(MAX) {
        let name = match chiacraft_packages::read_manifest(&d) {
            Ok(m) => m.id,
            Err(e) => {
                warn!("personal mod {}: {e}", d.display());
                continue;
            }
        };
        let loaded = std::fs::read(d.join(chiacraft_packages::identity::CLIENT_CODE))
            .map_err(|e| e.to_string())
            .and_then(|w| ClientMod::load(&name, &w));
        match loaded {
            Ok(m) => {
                let hooks = m.hooks().to_vec();
                info!("personal mod {name}: {} registrations", hooks.len());
                let half = Arc::new(Half::personal(Arc::new(m), Words::load(&d)));
                crate::client_half::track(&half);
                out.push(Personal { name, half, hooks });
            }
            Err(e) => warn!("personal mod {name}: {e}"),
        }
    }
    out
}

/// Opens one of a personal mod's screens through the shared one.
pub fn open_screen(c: Client, half: &Arc<Half>, name: &str) {
    let found = mods().iter().enumerate().find_map(|(i, p)| {
        if !Arc::ptr_eq(&p.half, half) {
            return None;
        }
        p.hooks.iter().enumerate().find_map(|(cb, h)| match h {
            ClientHook::Screen { name: n, .. } if n == name => Some((i, cb as u32)),
            _ => None,
        })
    });
    if let Some(s) = found {
        *SHOWING.lock().unwrap_or_else(|e| e.into_inner()) = Some(s);
        c.screen_open(&SCREEN);
    }
}

/// Registers the rule and the shared slots (in this mod's own `init`: the same in every game).
pub fn register(r: &mut Registry) {
    r.config(&RULE).choice(0, &["off", "client"]).world();
    for anchor in chiacraft_sandbox::client::ANCHORS {
        let element = HudElement::generated(format!("personal_{anchor}"));
        r.hud_element(element, anchor, 1000, move |ui| {
            if !allowed() {
                return;
            }
            // Each personal mod's elements at this anchor, by their order.
            let mut parts: Vec<(i32, &Personal, u32)> = Vec::new();
            for p in mods() {
                for (cb, h) in p.hooks.iter().enumerate() {
                    if let ClientHook::Hud { anchor: a, order, .. } = h
                        && a == anchor
                    {
                        parts.push((*order, p, cb as u32));
                    }
                }
            }
            parts.sort_by_key(|(o, ..)| *o);
            for (_, p, cb) in parts {
                build(ui, &p.half, cb);
            }
        });
    }
    r.screen(&SCREEN, |ui| {
        let showing = *SHOWING.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((i, cb)) = showing
            && allowed()
            && let Some(p) = mods().get(i)
        {
            build(ui, &p.half, cb);
        }
    });
    for (slot, key) in KEY_SLOTS.iter().enumerate() {
        r.key(key, &format!("shift+alt+{}", slot + 1))
            .on_press_client(move |c| {
                if !allowed() {
                    return;
                }
                // The personal mods' keys, in order, take the slots.
                let mut n = 0;
                for p in mods() {
                    for (cb, h) in p.hooks.iter().enumerate() {
                        if matches!(h, ClientHook::Key { .. }) {
                            if n == slot {
                                let g = game(*c, true, &p.half);
                                lock(&p.half).key(g, cb as u32);
                                return;
                            }
                            n += 1;
                        }
                    }
                }
            });
    }
    let last = Mutex::new(std::time::Instant::now());
    r.client_frame(move |c| {
        let mods = MODS.get_or_init(|| load_all(Path::new(&c.run_dir())));
        if mods.is_empty() {
            return;
        }
        tell(c);
        if !allowed() {
            return;
        }
        let seconds = {
            let mut l = last.lock().unwrap_or_else(|e| e.into_inner());
            let s = l.elapsed().as_secs_f32();
            *l = std::time::Instant::now();
            s
        };
        for p in mods {
            for (cb, h) in p.hooks.iter().enumerate() {
                if matches!(h, ClientHook::Frame) {
                    let g = game(*c, true, &p.half);
                    lock(&p.half).frame(g, cb as u32, seconds);
                }
            }
        }
    });
}

/// Once per world and value of the rule: in a world, tells the player which of their personal
/// mods are off there (or on, when they are).
fn tell(c: &Client) {
    if c.view().player_state().is_none() {
        // Not in a world: forget, so the next one is told.
        *TOLD.lock().unwrap_or_else(|e| e.into_inner()) = None;
        return;
    }
    let rule = RULE.get();
    let mut told = TOLD.lock().unwrap_or_else(|e| e.into_inner());
    if *told == Some(rule) {
        return;
    }
    *told = Some(rule);
    drop(told);
    let names: Vec<&str> = mods().iter().map(|p| p.name.as_str()).collect();
    let key = if rule == 1 { "packages.personal.on" } else { "packages.personal.off" };
    crate::show_line(c, crate::sandboxed::line_text(key, &[WArg::Text(names.join(", "))]));
}
