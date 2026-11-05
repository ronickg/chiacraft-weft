//! What an animal's body shows (`notes/ANIMALS.md`, "Animations"): the animation states its
//! behaviour asks for, as the engine's actions (`World::action_play`, `action_override`), each a
//! clip by name in the species' file (`source/<model>/build.py`). A species without that clip
//! goes on as it was, so a state can be asked for before every species has it.
//!
//! - **Held** (`hold`): a state it stays in until another or `clear`: staring at what it half
//!   noticed, grazing or drinking heads down, asleep, stalking (its walk becomes a crouched creep).
//! - **Once** (`act`): a clip over whatever it does: an alarm call, a pounce, a bite, rearing, a
//!   howl, play, grooming, nuzzling a young one, a stretch on getting up.
//! - **By itself**, here: a flinch when hit (`HURT`), and idle life now and then while it stands
//!   with nothing held (`idle_life`: looking round, sniffing the ground, a scratch). Breathing and
//!   ear and tail flicks are in every idle clip; the death fall is the engine's `death`.
//!
//! Cheap: the state is kept on the animal (`HELD`, not saved) and only a change is sent; idle life
//! looks once a second and plays a clip every 15 to 40 seconds.

use weft_sdk::*;

use crate::species::SPECIES;

/// An animation state, by its clip's name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    // Held.
    Stare,
    Graze,
    Drink,
    Sleep,
    Stalk,
    // Once.
    Alarm,
    Pounce,
    Bite,
    Rear,
    Howl,
    Play,
    Groom,
    Nuzzle,
    Stretch,
    Flinch,
    // Idle life.
    Look,
    Sniff,
    Scratch,
}

impl Act {
    /// The clip's name in a species' animation file.
    pub fn clip(self) -> &'static str {
        match self {
            Act::Stare => "stare",
            Act::Graze => "graze",
            Act::Drink => "drink",
            Act::Sleep => "sleep",
            Act::Stalk => "stalk",
            Act::Alarm => "alarm",
            Act::Pounce => "pounce",
            Act::Bite => "bite",
            Act::Rear => "rear",
            Act::Howl => "howl",
            Act::Play => "play",
            Act::Groom => "groom",
            Act::Nuzzle => "nuzzle",
            Act::Stretch => "stretch",
            Act::Flinch => "flinch",
            Act::Look => "look",
            Act::Sniff => "sniff",
            Act::Scratch => "scratch",
        }
    }
    const ALL: [Act; 18] = [
        Act::Stare,
        Act::Graze,
        Act::Drink,
        Act::Sleep,
        Act::Stalk,
        Act::Alarm,
        Act::Pounce,
        Act::Bite,
        Act::Rear,
        Act::Howl,
        Act::Play,
        Act::Groom,
        Act::Nuzzle,
        Act::Stretch,
        Act::Flinch,
        Act::Look,
        Act::Sniff,
        Act::Scratch,
    ];
    fn code(self) -> u32 {
        Act::ALL
            .iter()
            .position(|&a| a == self)
            .map_or(0, |i| i as u32 + 1)
    }
    fn of(code: u32) -> Option<Act> {
        code.checked_sub(1)
            .and_then(|i| Act::ALL.get(i as usize).copied())
    }
}

/// The held state + 1 (0: none), and when idle life looks next (not saved).
static HELD: Component<[u64; 2]> = Component::named("anim_held");

/// Ticks between idle life's clips: at least, and as much again at random.
const IDLE_EVERY: u64 = 300;
/// The idle extras (a species without one of these clips goes on idling).
const IDLE_LIFE: [Act; 3] = [Act::Look, Act::Sniff, Act::Scratch];

pub fn init(r: &mut Registry) {
    r.entity_component(&HELD, [0u64; 2], 0);
    for s in &SPECIES {
        r.attach(s.kind, &HELD);
    }
    r.command("anim", "<clip:word> [mode:word(hold,once,clear)]", command)
        .permission(permission::GAME_MASTER);
    // A flinch when hit, unless the blow kills it (the engine plays `death`).
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            if h.amount > 0.0 && w.component(h.entity, &HELD).is_some() && w.entity(h.entity).is_some_and(|e| e.health > h.amount) {
                act(&w, h.entity, Act::Flinch);
            }
        }
    });
}

/// The state `id` holds now, if any.
pub fn held(w: &World, id: u64) -> Option<Act> {
    w.component(id, &HELD).and_then(|h| Act::of(h[0] as u32))
}

/// `id` holds state `a` until another or `clear`: sent only when it changes.
pub fn hold(w: &World, id: u64, a: Act) {
    let Some(h) = w.component(id, &HELD) else { return };
    if h[0] as u32 == a.code() {
        return;
    }
    end(w, id, Act::of(h[0] as u32));
    if a == Act::Stalk {
        w.action_override(id, action::WALK, a.clip());
    } else {
        w.action_play(id, a.clip(), true);
    }
    set(w, id, a.code() as u64);
}

/// `id` holds nothing: back to its idle and walk.
pub fn clear(w: &World, id: u64) {
    let Some(h) = w.component(id, &HELD) else { return };
    if h[0] != 0 {
        end(w, id, Act::of(h[0] as u32));
        set(w, id, 0);
    }
}

/// `id` plays `a` once, over what it does; a held state ends with it.
pub fn act(w: &World, id: u64, a: Act) {
    let Some(h) = w.component(id, &HELD) else { return };
    if h[0] != 0 {
        if Act::of(h[0] as u32) == Some(Act::Stalk) {
            w.action_override_clear(id, action::WALK);
        }
        set(w, id, 0);
    }
    w.action_play(id, a.clip(), false);
}

fn end(w: &World, id: u64, was: Option<Act>) {
    match was {
        Some(Act::Stalk) => w.action_override_clear(id, action::WALK),
        Some(_) => w.action_stop(id),
        None => {}
    }
}

fn set(w: &World, id: u64, code: u64) {
    if let Some(h) = w.component_mut(id, &HELD) {
        h[0] = code;
    }
}

/// Idle life: standing with nothing held and going nowhere, now and then it looks round, sniffs
/// the ground or scratches (whichever clips its species has). Called from the behaviour's idle.
pub fn idle_life(w: &World, id: u64) {
    let now = w.game_time();
    if !(now + id).is_multiple_of(20) {
        return;
    }
    let Some(h) = w.component(id, &HELD) else { return };
    if h[1] == 0 {
        // Not yet timed: its first in a while, so a herd met together doesn't move as one.
        if let Some(c) = w.component_mut(id, &HELD) {
            c[1] = now + IDLE_EVERY + w.random_below(IDLE_EVERY as u32) as u64;
        }
        return;
    }
    if now < h[1] || h[0] != 0 || w.navigating(id) {
        return;
    }
    let a = IDLE_LIFE[w.random_below(IDLE_LIFE.len() as u32) as usize];
    w.action_play(id, a.clip(), false);
    if let Some(c) = w.component_mut(id, &HELD) {
        c[1] = now + IDLE_EVERY + w.random_below(IDLE_EVERY as u32) as u64;
    }
}

/// `/anim <clip> [hold|once|clear]` (for checks and looks): the nearest animal within 16 blocks
/// holds a state or plays a clip once (`death` too, without dying).
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    let Some(p) = w.entity(player) else { return };
    let d2 = |id: u64| {
        w.entity(id)
            .map_or(f64::MAX, |e| crate::dist2(e.pos, p.pos))
    };
    let Some(id) = w
        .entities_near(p.pos, 16.0, None)
        .into_iter()
        .filter(|&id| w.component(id, &HELD).is_some())
        .min_by(|&a, &b| d2(a).total_cmp(&d2(b)))
    else {
        return;
    };
    let clip = cmd.text("clip").unwrap_or_default().to_string();
    let mode = cmd.text("mode").unwrap_or("once").to_string();
    match (Act::ALL.iter().find(|a| a.clip() == clip), mode.as_str()) {
        (_, "clear") => clear(&w, id),
        (Some(&a), "hold") => hold(&w, id, a),
        (Some(&a), _) => act(&w, id, a),
        (None, _) => w.action_play(id, &clip, mode == "hold"),
    }
    let line = format!("anim: {id} {clip} {mode}");
    weft_sdk::info!("{line}");
    cmd.reply("command.animals.line", &[Arg::Text(&line)]);
}
