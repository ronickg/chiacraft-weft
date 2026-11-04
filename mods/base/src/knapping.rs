//! The knapping screen (`notes/FIRST_HOURS.md`; Ronald, 11-05: "Knapping screen, then book").
//! Sneaking and striking a flint nodule on the ground with a rock opens it: a 5 × 5 nodule with a
//! head's outline on it (`vitals::first::KNAPPED`: an axe head, a pick head, a shovel blade). Each
//! square outside the outline is struck away with a click; a click inside breaks the nodule into
//! a flake. When the last square outside is gone the head comes free, into the bag.
//!
//! The server keeps each player's nodule (where it is, the head picked, what's struck away) and
//! sends it to the client, whose screen shows it; the client sends its clicks.

use std::sync::Mutex;

use crafting::{code, grades, skills};
use vitals::first::{self as f, KNAPPED};
use weft_sdk::*;

use crate::first::{FLINT, LOOSE_FLINT, SHARP_FLAKE};
use crate::parts::{PART, PartOf, SKILLS};

value! {
    /// A player's nodule on the knapping screen: the head picked (`KNAPPED`), the squares struck
    /// away (a bit each, row by row), and where it lies; at the end, the square that broke it
    /// (`broken`, 1 + the square) or the head freed (`done`), shown a moment before it closes.
    pub struct Knap { pub shape: u32, pub removed: u32, pub x: i32, pub y: i32, pub z: i32, pub broken: u32, pub done: u32 }
}
static OPEN: Channel<Knap> = Channel::named("knap_open");
static UPDATE: Channel<Knap> = Channel::named("knap_update");
static CLOSE: Channel<u32> = Channel::named("knap_close");
/// Client to server: a square struck (0-24), or a head picked (`PICK` + its index).
static HIT: Channel<u32> = Channel::named("knap_hit");
const PICK: u32 = 100;
pub static SCREEN: Screen = Screen::named("knapping");

static STRIKE: Sound = Sound::named("block.stone.hit");
static CRACK: Sound = Sound::named("block.stone.break");

/// Each player's nodule, on the server.
static NODULES: Mutex<Vec<(u64, Knap)>> = Mutex::new(Vec::new());
/// The local player's, on the client.
static VIEW: Mutex<Option<Knap>> = Mutex::new(None);
/// When the client closes a screen that ended (broken or a head freed): shown this long first.
static CLOSE_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
const END_SHOWN: std::time::Duration = std::time::Duration::from_millis(500);

/// The local player's nodule as the screen shows it (UI's `screen`).
pub fn view() -> Option<Knap> {
    *VIEW.lock().unwrap()
}

/// How far a player may be from their nodule and still knap it, blocks.
const REACH: f64 = 6.0;

pub fn register(r: &mut Registry) {
    r.packet(&HIT, |p, v| hit(&p.world(), p.player(), v));
    r.client_packet(&OPEN, |c, v| {
        *VIEW.lock().unwrap() = Some(v);
        let opened = c.open_screen(&SCREEN);
        weft_sdk::info!("knapping: screen opened {opened}");
    });
    r.client_packet(&UPDATE, |c, v| {
        *VIEW.lock().unwrap() = Some(v);
        c.rebuild_screen();
    });
    // A nodule that broke or freed its head stays on screen a moment (UI redo); one taken away
    // closes at once.
    r.client_packet(&CLOSE, |c, _| {
        if VIEW
            .lock()
            .unwrap()
            .is_some_and(|v| v.broken != 0 || v.done != 0)
        {
            *CLOSE_AT.lock().unwrap() = Some(std::time::Instant::now() + END_SHOWN);
        } else {
            *VIEW.lock().unwrap() = None;
            c.close_screen();
        }
    });
    r.client_frame(|c| {
        let mut at = CLOSE_AT.lock().unwrap();
        if at.is_some_and(|t| std::time::Instant::now() >= t) {
            *at = None;
            *VIEW.lock().unwrap() = None;
            c.close_screen();
        }
    });
    // A nodule at the feet and its screen, without striking (for checks and looks; UI redo).
    r.command("knap", "[head:word]", |cmd| {
        let w = cmd.world();
        let player = cmd.player();
        let Some(e) = w.entity(player) else { return };
        let feet = e.block_pos();
        let shape = cmd
            .text("head")
            .and_then(|h| {
                KNAPPED
                    .iter()
                    .position(|k| k.kind == h || k.shape == h || (h == "pick" && k.kind == "pickaxe"))
            })
            .unwrap_or(0) as u32;
        if w.get(feet) == 0 {
            w.set(feet, LOOSE_FLINT.id());
        }
        open(&w, player, feet);
        if shape != 0
            && let Some((_, k)) = NODULES
                .lock()
                .unwrap()
                .iter_mut()
                .find(|(p, _)| *p == player)
        {
            k.shape = shape;
            w.send(player, &UPDATE, k);
        }
    })
    .permission(permission::GAME_MASTER);
    r.screen(&SCREEN, screen);
}

/// `player` starts (or goes on with) knapping the nodule at `pos`.
pub fn open(w: &World, player: u64, pos: [i32; 3]) {
    let mut nodules = NODULES.lock().unwrap();
    let knap = match nodules.iter().find(|(p, _)| *p == player) {
        Some(&(_, k)) if [k.x, k.y, k.z] == pos => k,
        _ => Knap { shape: 0, removed: 0, x: pos[0], y: pos[1], z: pos[2], broken: 0, done: 0 },
    };
    nodules.retain(|(p, _)| *p != player);
    nodules.push((player, knap));
    weft_sdk::info!("knapping: {player} opens the nodule at {pos:?}, head {}", knap.shape);
    w.send(player, &OPEN, &knap);
}

/// A click from `player`'s screen.
fn hit(w: &World, player: u64, v: u32) {
    let mut nodules = NODULES.lock().unwrap();
    let Some(i) = nodules.iter().position(|(p, _)| *p == player) else { return };
    let mut k = nodules[i].1;
    let pos = [k.x, k.y, k.z];
    let at = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.2, pos[2] as f64 + 0.5];
    let near = w.entity(player).is_some_and(|e| {
        let d = [e.pos[0] - at[0], e.pos[1] - at[1], e.pos[2] - at[2]];
        d.iter().map(|x| x * x).sum::<f64>() <= REACH * REACH
    });
    // The nodule gone (picked up, broken) or left behind: the screen closes.
    if !near || !LOOSE_FLINT.is(w.get(pos)) {
        nodules.remove(i);
        w.send(player, &CLOSE, &0);
        return;
    }
    if v >= PICK {
        // A head is picked before the first strike only.
        if k.removed == 0 && ((v - PICK) as usize) < KNAPPED.len() {
            k.shape = v - PICK;
            nodules[i].1 = k;
            w.send(player, &UPDATE, &k);
        }
        return;
    }
    let square = v as usize;
    if square >= f::KNAP_SIZE * f::KNAP_SIZE || k.removed & (1 << square) != 0 {
        return;
    }
    let head = &KNAPPED[k.shape as usize % KNAPPED.len()];
    if head.keeps(square) {
        // Struck inside the outline: the nodule breaks, a flake is all that's left.
        w.set(pos, 0);
        w.spawn_item(at, ItemStack::new(SHARP_FLAKE.id(), 1));
        w.play_sound(&CRACK, at, 1.0);
        nodules.remove(i);
        k.broken = square as u32 + 1;
        w.send(player, &UPDATE, &k);
        w.send(player, &CLOSE, &0);
        return;
    }
    k.removed |= 1 << square;
    w.play_sound(&STRIKE, at, 1.0);
    if k.removed & head.away() == head.away() {
        w.set(pos, 0);
        let left = w
            .inventory(player)
            .insert(knapped_head(w, player, head.shape, true, Some(unit(w))));
        if !left.is_empty() {
            w.spawn_item(at, left);
        }
        w.play_sound(&CRACK, at, 1.0);
        nodules.remove(i);
        k.done = 1;
        w.send(player, &UPDATE, &k);
        w.send(player, &CLOSE, &0);
    } else {
        nodules[i].1 = k;
        w.send(player, &UPDATE, &k);
    }
}

fn unit(w: &World) -> f32 {
    (w.random() >> 40) as f32 / (1u64 << 24) as f32
}

/// A flint head of part shape `shape` made by `player`: its grade from flint, their knapping skill
/// and no station; work by hand scores `BY_HAND_BONUS` over work from memory (the book). With a
/// roll, luck and the skill's rise as any made part; without (a grid's preview), neither.
pub fn knapped_head(w: &World, player: u64, shape: &str, by_hand: bool, roll: Option<f32>) -> ItemStack {
    let craft = skills::index(crafting::materials::Making::Knap);
    let xp = w.component(player, &SKILLS).map_or(0, |s| s.xp[craft]);
    let memory = grades::work_from_memory(skills::level(xp));
    let work = if by_hand { (memory + f::BY_HAND_BONUS).min(grades::WORK_MAX) } else { memory };
    // 0.5 is no luck either way (`grades::LUCK`).
    let grade = grades::roll(grades::Points { material: f::HEAD_MATERIAL_POINTS, work, station: 0 }, roll.unwrap_or(0.5));
    if roll.is_some()
        && let Some(s) = w.component_mut(player, &SKILLS)
    {
        s.xp[craft] = s.xp[craft].saturating_add(skills::xp_for(grade));
    }
    let made_day = (w.day_time() / vitals::calendar::DAY_TICKS) as u32;
    ItemStack::new(Item::generated(format!("part_{shape}")).id(), 1)
        .with(&PART, PartOf { material: code("flint"), grade: grade as u32, worn: 0, made_day, repairs: 0 })
}

/// The screen: the heads to pick, the nodule's squares (the outline's lit), a line on how.
fn screen(ui: &mut Ui) {
    let Some(k) = *VIEW.lock().unwrap() else { return };
    let head = &KNAPPED[k.shape as usize % KNAPPED.len()];
    crate::window::window(ui, S::col().pad(8.0).gap(5.0), &[], |ui| {
        ui.text_styled(S::col().class("title"), "knapping.base.title", &[]);
        ui.row(S::row().gap(2.0), |ui| {
            for (i, h) in KNAPPED.iter().enumerate() {
                let picked = i as u32 == k.shape;
                let s = S::default()
                    .h(13.0)
                    .class(if picked { "button" } else { "button_secondary" })
                    .selected(picked)
                    .disabled(k.removed != 0 && !picked);
                ui.button(&format!("item.base.part_{}", h.shape), s, move |c| {
                    c.client().send(&HIT, &(PICK + i as u32));
                });
            }
        });
        ui.column(S::col().gap(1.0).center(), |ui| {
            for row in 0..f::KNAP_SIZE {
                ui.row(S::row().gap(1.0), |ui| {
                    for col in 0..f::KNAP_SIZE {
                        let i = row * f::KNAP_SIZE + col;
                        let class = if head.keeps(i) { "slot_highlight" } else { "slot" };
                        let s = S::default().size(18.0, 18.0).class(class);
                        if k.removed & (1 << i) != 0 {
                            // Struck away: an empty space.
                            ui.column(S::col().size(18.0, 18.0), |_| {});
                        } else {
                            ui.button_with(
                                s,
                                move |c| {
                                    c.client().send(&HIT, &(i as u32));
                                },
                                |ui| ui.item(FLINT.id(), 1, S::default()),
                            );
                        }
                    }
                });
            }
        });
        ui.text_styled(S::col().class("dim").max_w(150.0), "knapping.base.how", &[]);
    });
}
