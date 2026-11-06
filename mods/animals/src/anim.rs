//! What an animal's body shows (`notes/ANIMALS.md`, "Animations"): the animation states its
//! behaviour asks for, as the engine's actions (`World::action_play`, `action_override`), each a
//! clip by name in the species' file (`source/<model>/build.py`). A species without that clip
//! goes on as it was, so a state can be asked for before every species has it.
//!
//! - **Held** (`hold`): a state it stays in while its behaviour keeps asking for it, each tick it
//!   does (or at least once in `GRACE` ticks): staring at what it half noticed, grazing or
//!   drinking heads down, asleep, a sentinel on watch. Asked no more, it ends by itself, so a
//!   goal that loses the turn needs no clean-up; `clear` ends one at once.
//! - **Gaits** (held too): its walk (and run) played as another clip: stalking in a crouched creep,
//!   a curious one's slow head-low approach, a charge with the head down.
//! - **Once** (`act`, or `play` for a clip by name and length: a boss's own): a clip over whatever
//!   it does: an alarm call, a pounce, a bite, rearing, a howl, play, grooming, nuzzling a young
//!   one, a stretch on getting up. A held state waits under it and comes back after, if it is
//!   still asked for; a gait goes on under it.
//! - **By itself**, here (`tick`, from `animal::upkeep`): a flinch when hit (`HURT`), and idle life
//!   now and then while it stands with nothing held (looking round, sniffing the ground, a
//!   scratch). Breathing and ear and tail flicks are in every idle clip; the death fall is the
//!   engine's `death`.
//!
//! Cheap: the state is kept on the animal (`HELD`, not saved) and only a change is sent; idle life
//! looks once a second and plays a clip every 15 to 40 seconds.

use std::sync::atomic::{AtomicBool, Ordering};

use weft_sdk::*;

use crate::species::SPECIES;

/// `/anim log`: every change of state and every clip played once goes to the log (for checks).
static LOG: AtomicBool = AtomicBool::new(false);

fn log(id: u64, what: &str, clip: &str) {
    if LOG.load(Ordering::Relaxed) {
        weft_sdk::info!("anim: {id} {what} {clip}");
    }
}

/// An animation state, by its clip's name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    // Held.
    Stare,
    Graze,
    Drink,
    Sleep,
    /// A sentinel on watch: head high, turning to look round.
    Watch,
    // Held gaits: its walk (and run) played as this clip.
    Stalk,
    /// A curious one's slow, head-low approach.
    Approach,
    /// Head down, running at what it charges.
    Charge,
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
            Act::Watch => "watch",
            Act::Stalk => "stalk",
            Act::Approach => "approach",
            Act::Charge => "charge",
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

    /// The engine's clips a gait plays in place of (none: not a gait).
    fn gait(self) -> &'static [&'static str] {
        match self {
            Act::Stalk | Act::Approach => &[action::WALK],
            Act::Charge => &[action::WALK, RUN],
            _ => &[],
        }
    }

    /// How long a clip played once lasts, in seconds: the longest of the species' clips of that
    /// name (a held state waits this long under it). 0 for a held one.
    pub fn secs(self) -> f32 {
        match self {
            Act::Alarm | Act::Rear => 1.4,
            Act::Pounce => 1.0,
            Act::Bite => 0.9,
            Act::Howl => 4.0,
            Act::Play => 1.6,
            Act::Groom => 2.6,
            Act::Nuzzle => 2.0,
            Act::Stretch => 2.5,
            Act::Flinch => 0.45,
            Act::Look => 3.0,
            Act::Sniff => 2.5,
            Act::Scratch => 2.0,
            _ => 0.0,
        }
    }

    const ALL: [Act; 21] = [
        Act::Stare,
        Act::Graze,
        Act::Drink,
        Act::Sleep,
        Act::Watch,
        Act::Stalk,
        Act::Approach,
        Act::Charge,
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

    /// The state by its clip's name.
    pub fn named(clip: &str) -> Option<Act> {
        Act::ALL.iter().copied().find(|a| a.clip() == clip)
    }

    fn code(self) -> u64 {
        Act::ALL
            .iter()
            .position(|&a| a == self)
            .map_or(0, |i| i as u64 + 1)
    }

    fn of(state: u64) -> Option<Act> {
        (state & CODE)
            .checked_sub(1)
            .and_then(|i| Act::ALL.get(i as usize).copied())
    }
}

/// The engine's run (it has no name for it in `action`).
const RUN: &str = "run";

/// On the animal (not saved): its held state (`Act` code + 1, 0: none; `APPLIED` once its clip
/// plays), when idle life looks next, the tick its held state was last asked for until, and the
/// tick a clip played once ends.
static HELD: Component<[u64; 4]> = Component::named("anim_held");
const STATE: usize = 0;
const IDLE_NEXT: usize = 1;
const ASKED: usize = 2;
const BUSY: usize = 3;
const CODE: u64 = 0xff;
const APPLIED: u64 = 0x100;

/// Ticks a held state lasts past the last tick it was asked for (beyond the ticks its animal
/// skips far from players): more than the goals that ask now and then wait (`senses::EVERY`, a
/// curious one's look every 20).
const GRACE: u64 = 30;
/// Ticks between idle life's clips: at least, and as much again at random.
const IDLE_EVERY: u64 = 300;
/// The idle extras (a species without one of these clips goes on idling).
const IDLE_LIFE: [Act; 4] = [Act::Look, Act::Sniff, Act::Scratch, Act::Groom];
/// A pack's or herd's idle life: grooming twice as often.
const IDLE_LIFE_GROUP: [Act; 5] = [Act::Look, Act::Sniff, Act::Scratch, Act::Groom, Act::Groom];
/// Seconds a clip played once by name lasts when it isn't an `Act` (a species' own attack).
const CLIP_SECS: f32 = 0.8;

pub fn init(r: &mut Registry) {
    r.entity_component(&HELD, [0u64; 4], 0);
    for s in &SPECIES {
        r.attach(s.kind, &HELD);
    }
    // The bosses (`boss.rs`) hold watch and stare too; their own AI plays the rest (`expire`).
    for k in [&crate::drenkaun::DRENKAUN, &crate::borkon::BORKON] {
        r.attach(k, &HELD);
    }
    r.command("anim", "<clip:word> [mode:word(hold,once,clear,loop)]", command)
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
    w.component(id, &HELD).and_then(|h| Act::of(h[STATE]))
}

/// Whether a clip played once (`act`, `play`) is still on for `id`.
pub fn busy(w: &World, id: u64) -> bool {
    w.component(id, &HELD)
        .is_some_and(|h| w.game_time() < h[BUSY])
}

/// `id` holds state `a` (a gait too) this tick: call it each tick the behaviour wants it (once in
/// `GRACE` ticks at least); it ends by itself once no longer asked for. Sent only when it changes.
pub fn hold(w: &World, id: u64, a: Act) {
    hold_for(w, id, a, 0);
}

/// `hold`, asked for `ticks` ahead: for a behaviour that looks seldom, or a check that pins a
/// state (`/anim <clip> hold`).
pub fn hold_for(w: &World, id: u64, a: Act, ticks: u64) {
    let Some(h) = w.component(id, &HELD) else { return };
    let now = w.game_time();
    let until = now + ticks;
    if h[STATE] & CODE == a.code() {
        if h[STATE] & APPLIED == 0 && now >= h[BUSY] {
            // Waiting under a clip played once that has ended: back on now (`tick` would too).
            w.action_play(id, a.clip(), true);
            edit(w, id, |c| {
                c[STATE] |= APPLIED;
                c[ASKED] = c[ASKED].max(until);
            });
        } else if until > h[ASKED] + 4 {
            // Written only now and then: the grace covers the ticks between.
            edit(w, id, |c| c[ASKED] = until);
        }
        return;
    }
    end(w, id, h[STATE]);
    log(id, "holds", a.clip());
    let state = if !a.gait().is_empty() {
        for from in a.gait() {
            w.action_override(id, from, a.clip());
        }
        a.code() | APPLIED
    } else if now < h[BUSY] {
        // A clip played once is on: it comes on after (`tick`).
        a.code()
    } else {
        w.action_play(id, a.clip(), true);
        a.code() | APPLIED
    };
    edit(w, id, |c| {
        c[STATE] = state;
        c[ASKED] = until;
    });
}

/// `id` holds nothing: back to its idle, walk and run.
pub fn clear(w: &World, id: u64) {
    let Some(h) = w.component(id, &HELD) else { return };
    if h[STATE] != 0 {
        end(w, id, h[STATE]);
        log(id, "holds", "nothing");
        edit(w, id, |c| c[STATE] = 0);
    }
}

/// `id` plays `a` once, over what it does: a held state waits under it, a gait goes on.
pub fn act(w: &World, id: u64, a: Act) {
    play(w, id, a.clip(), a.secs());
}

/// `id` plays clip `clip` once, `secs` long (its file's length; a boss's own clip, a slam, a
/// roar), as `act`.
pub fn play(w: &World, id: u64, clip: &str, secs: f32) {
    let Some(h) = w.component(id, &HELD) else { return };
    log(id, "plays", clip);
    w.action_play(id, clip, false);
    let busy = w.game_time() + (secs * 20.0).ceil() as u64;
    edit(w, id, |c| {
        // A held clip is cut by this one: it comes back after.
        if Act::of(h[STATE]).is_some_and(|a| a.gait().is_empty()) {
            c[STATE] &= CODE;
        }
        c[BUSY] = busy;
    });
}

/// `play` of one of its species' clips by name (an attack: a bite, a swipe, a kick).
pub fn play_named(w: &World, id: u64, clip: &str) {
    play(w, id, clip, Act::named(clip).map_or(CLIP_SECS, |a| a.secs().max(0.1)));
}

/// Ends what state `state` showed.
fn end(w: &World, id: u64, state: u64) {
    let Some(a) = Act::of(state) else { return };
    if !a.gait().is_empty() {
        for from in a.gait() {
            w.action_override_clear(id, from);
        }
    } else if state & APPLIED != 0 {
        w.action_stop(id);
    }
}

fn edit(w: &World, id: u64, f: impl FnOnce(&mut [u64; 4])) {
    if let Some(c) = w.component_mut(id, &HELD) {
        f(c);
    }
}

/// Every tick, from `animal::upkeep`: a held state no longer asked for ends, one that waited
/// under a clip played once comes back on, and idle life (`idle_life`).
pub fn tick(w: &World, id: u64) {
    if let Some(h) = expire_(w, id) {
        idle_life(w, id, h);
    }
}

/// `tick` without idle life: for a creature whose own AI plays its other clips (a boss, which
/// calls `clear` before each of them).
pub fn expire(w: &World, id: u64) {
    expire_(w, id);
}

/// Ends a state no longer asked for and puts one back that waited; its held state if it holds none.
fn expire_(w: &World, id: u64) -> Option<[u64; 4]> {
    let h = w.component(id, &HELD)?;
    if h[STATE] != 0 {
        let now = w.game_time();
        let skipped = w.tick_elapsed(id) as u64;
        if now > h[ASKED] + GRACE + skipped {
            clear(w, id);
        } else if h[STATE] & APPLIED == 0 && now >= h[BUSY] && now <= h[ASKED] + skipped + 2 {
            // Asked for still (not just lingering in its grace): back on.
            if let Some(a) = Act::of(h[STATE]) {
                w.action_play(id, a.clip(), true);
            }
            edit(w, id, |c| c[STATE] |= APPLIED);
        }
        return None;
    }
    Some(h)
}

/// Idle life: standing with nothing held, nothing playing and going nowhere, now and then it looks
/// round, sniffs the ground, scratches or grooms, a young one plays and a mother nuzzles her young
/// (`idle_act`; whichever clips its species has).
fn idle_life(w: &World, id: u64, h: [u64; 4]) {
    let now = w.game_time();
    // No `(now + id) % 20` gate: under the AI's level of detail its upkeep skips ticks in a
    // rhythm, and some animals never met that tick (a PC look: one idle moment in five minutes
    // among seven deer). `IDLE_NEXT` spaces them out by itself.
    if h[IDLE_NEXT] == 0 {
        // Not yet timed: its first in a while, so a herd met together doesn't move as one.
        edit(w, id, |c| c[IDLE_NEXT] = now + IDLE_EVERY + w.random_below(IDLE_EVERY as u32) as u64);
        return;
    }
    if now < h[IDLE_NEXT] || now < h[BUSY] || w.navigating(id) {
        return;
    }
    let a = idle_act(w, id);
    act(w, id, a);
    edit(w, id, |c| c[IDLE_NEXT] = now + IDLE_EVERY + w.random_below(IDLE_EVERY as u32) as u64);
}

/// Its idle moment's clip, by who it is (`animals_core::roles::PLAY_GROUP`...): a young one
/// plays, a mother with her young near nuzzles one (turned to it), and anyone else looks round,
/// sniffs, scratches or grooms.
fn idle_act(w: &World, id: u64) -> Act {
    use animals_core::biology::Social;
    use animals_core::roles;
    let s = w.component(id, &crate::animal::ANIMAL).and_then(|a| a.species());
    let group = s.is_some_and(|s| s.bio.social != Social::Solitary);
    let roll = w.random_below(100);
    let young = w.component(id, &crate::breeding::BREEDING).is_some_and(|b| b.growing > 0);
    if young {
        if roll < if group { roles::PLAY_GROUP } else { roles::PLAY_ALONE } {
            return Act::Play;
        }
    } else if roll < roles::NUZZLE
        && let (Some(s), Some(st)) = (s, w.entity(id))
        && let Some(y) = w
            .entities_near(st.pos, roles::NUZZLE_NEAR, Some(s.kind))
            .into_iter()
            .find(|&o| {
                o != id && w.component(o, &crate::breeding::BREEDING).is_some_and(|b| b.mother == id && b.growing > 0)
            })
            .and_then(|o| w.entity(o))
    {
        crate::senses::face(w, id, &st, [y.pos.x as f32, y.pos.y as f32, y.pos.z as f32]);
        return Act::Nuzzle;
    }
    let pool: &[Act] = if group { &IDLE_LIFE_GROUP } else { &IDLE_LIFE };
    pool[w.random_below(pool.len() as u32) as usize]
}

/// Ticks `/anim <clip> hold` pins a state for: ten minutes, for looks.
const PINNED: u64 = 20 * 600;

/// `/anim <clip> [hold|once|clear|loop]` (for checks and looks): the nearest animal within 16
/// blocks holds a state (pinned for ten minutes), plays a clip once (`death` too, without dying),
/// or plays any clip looping where it stands (`loop`: a gait seen in place, ticks frozen). `/anim
/// log` logs every animal's changes of state from then on.
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    if cmd.text("clip") == Some("log") {
        LOG.store(true, Ordering::Relaxed);
        cmd.reply("command.animals.line", &[Arg::Text("anim: logging")]);
        return;
    }
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
    match (Act::named(&clip), mode.as_str()) {
        (_, "clear") => {
            clear(&w, id);
            w.action_stop(id);
        }
        (_, "loop") => w.action_play(id, &clip, true),
        (Some(a), "hold") => {
            // At once, even under a clip played once: with ticks frozen (`/tick freeze`, for
            // looks) the game's clock stops and that clip would never end for it.
            edit(&w, id, |c| c[BUSY] = 0);
            hold_for(&w, id, a, PINNED);
        }
        (Some(a), _) => act(&w, id, a),
        (None, "hold") => w.action_play(id, &clip, true),
        (None, _) => play_named(&w, id, &clip),
    }
    let line = format!("anim: {id} {clip} {mode}");
    weft_sdk::info!("{line}");
    cmd.reply("command.animals.line", &[Arg::Text(&line)]);
}
