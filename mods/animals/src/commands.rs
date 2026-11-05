//! Commands for tames (`notes/ANIMALS.md`, "Commands"): follow, stay, wander (around where it was
//! left) and guard (an area around a point); stances: passive (never fights, runs from harm),
//! neutral (fights back, defends its owner), aggressive (also attacks monsters nearby) and assist
//! (also attacks what its owner hits). Whistle keys order every tame in earshot; tribes share
//! tames between players.

use weft_sdk::*;

use crate::dist2;

use crate::animal::{self, ANIMAL, OWNER};
use crate::species::{self, SPECIES};

pub const WANDER: u8 = 0;
pub const FOLLOW: u8 = 1;
pub const STAY: u8 = 2;
pub const GUARD: u8 = 3;
pub const COMMAND_NAMES: [&str; 4] = ["wander", "follow", "stay", "guard"];
pub const PASSIVE: u32 = 0;
pub const NEUTRAL: u32 = 1;
pub const AGGRESSIVE: u32 = 2;
pub const ASSIST: u32 = 3;
pub const STANCE_NAMES: [&str; 4] = ["passive", "neutral", "aggressive", "assist"];

/// How far a whistle carries, a wanderer roams from home, a tame chases.
const WHISTLE_RANGE: f64 = 32.0;
const ROAM: f64 = 12.0;
const CHASE: f64 = 32.0;
/// Monsters this close to an aggressive tame are attacked.
const AGGRO: f64 = 10.0;
/// What its owner hit, or what hit its owner, counts this long (ten seconds).
const FIGHT_TICKS: u64 = 200;

value! {
    /// A tame's orders, saved with it.
    pub struct Orders {
        /// Where it stays, guards or wanders around.
        pub home: [f64; 3],
        /// What it is fighting (0: nothing).
        pub target: u64,
        pub stance: u32,
        /// How close it keeps when following.
        pub follow: f32,
        /// Ticks left running from harm (passive).
        pub flee: u32,
        /// The guarded area's radius.
        pub guard: f32,
    }
}

value! {
    #[derive(Default)]
    /// A tribe: a random id (shared by its members) and a name.
    pub struct Tribe {
        pub id: [u8; 16],
        pub name: [u8; 24],
    }
}

impl Tribe {
    fn is_some(&self) -> bool {
        self.id.iter().any(|&b| b != 0)
    }
    fn name(&self) -> String {
        String::from_utf8_lossy(&self.name)
            .trim_end_matches('\0')
            .to_string()
    }
}

pub static ORDERS: Component<Orders> = Component::named("orders");
/// The tribe a tame is shared with.
pub static SHARED: Component<Tribe> = Component::named("shared");
/// A player's tribe, and the one they are invited to.
static TRIBE: Component<Tribe> = Component::named("tribe");
static INVITE: Component<Tribe> = Component::named("tribe_invite");
/// A player's fights: what they last hit and when, what last hit them and when (not saved).
static FIGHT: Component<[u64; 4]> = Component::named("fight");
/// The owner's entity as last found (not saved).
static OWNER_ID: Component<u64> = Component::named("owner_id");
static HOSTILE: Tag = Tag::named("hostile");
static NOTE: Particle = Particle::named("heart");
/// Client to server: an order from the animal screen, as (animal, order): a command (0-3), a
/// stance (10-13), sharing with the tribe turned on or off (20), or tracking what the player
/// holds (30).
pub static ORDER: Channel<[u64; 2]> = Channel::named("order");
static WHISTLES: [(Key, &str); 5] = [
    (Key::named("whistle_follow"), "J"),
    (Key::named("whistle_stay"), "K"),
    (Key::named("whistle_come"), "H"),
    // Not G: Weft's swap hands took it in the 26.11.2 sync.
    (Key::named("whistle_attack"), "Y"),
    (Key::named("whistle_passive"), "B"),
];

const DEFAULT: Orders = Orders { home: [0.0; 3], target: 0, stance: NEUTRAL, follow: 3.0, flee: 0, guard: 12.0 };

pub fn init(r: &mut Registry) {
    r.entity_component(&ORDERS, DEFAULT, component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&SHARED, Tribe::default(), component_flags::SAVE | component_flags::SYNC_MENU);
    r.entity_component(&OWNER_ID, 0u64, 0);
    r.entity_component(&TRIBE, Tribe::default(), component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.entity_component(&INVITE, Tribe::default(), component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.entity_component(&FIGHT, [0u64; 4], 0);
    for s in &SPECIES {
        r.attach(s.kind, &ORDERS);
        r.attach(s.kind, &SHARED);
        r.attach(s.kind, &OWNER_ID);
    }
    for c in [&TRIBE, &INVITE] {
        r.attach(&PLAYER, c);
    }
    r.attach(&PLAYER, &FIGHT);
    r.on(&actions::HURT, |b| {
        let w = b.world();
        for h in b.iter() {
            hurt(&w, h);
        }
    });
    r.packet(&ORDER, |p, [id, order]| {
        let (w, player) = (p.world(), p.player());
        if can_command(&w, id, player) {
            order_one(&w, id, player, order as u32);
        }
    });
    for (i, (key, default)) in WHISTLES.iter().enumerate() {
        r.key(key, default)
            .on_press_server(move |p| whistle(&p.world(), p.player(), i));
    }
    r.command("tribe", "[action:word(create,join,leave,info)] [name:text]", tribe_command)
        .permission(permission::EVERYONE);
    r.command("tribeinvite", "<player:targets>", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        let Some(t) = w.component(player, &TRIBE).filter(Tribe::is_some) else {
            cmd.reply("command.animals.tribe.none", &[]);
            return;
        };
        for other in cmd.targets("player") {
            if let Some(i) = w.component_mut(other, &INVITE) {
                *i = t;
            }
        }
        cmd.reply("command.animals.tribe.invited", &[Arg::Text(&t.name())]);
    })
    .permission(permission::EVERYONE);
}

// ------------------------------------------------------------------ who may command

/// The owner's entity, if they are here (looked up by key now and then).
pub fn owner(w: &World, id: u64) -> Option<u64> {
    let key = w
        .component(id, &OWNER)
        .filter(|k| k[0] != 0)
        .map(|k| String::from_utf8_lossy(&k).into_owned())?;
    let cached = w.component(id, &OWNER_ID).unwrap_or(0);
    if cached != 0 && w.entity(cached).is_some() && (w.game_time() + id) % 40 != 0 {
        return Some(cached);
    }
    let found = w
        .players()
        .into_iter()
        .find(|&p| crate::player_key(p) == key);
    if let Some(o) = w.component_mut(id, &OWNER_ID) {
        *o = found.unwrap_or(0);
    }
    found
}

/// Whether `player` may give `id` orders: its owner, or of the tribe it is shared with.
pub fn can_command(w: &World, id: u64, player: u64) -> bool {
    if animal::owns(w, id, player) {
        return true;
    }
    let shared = w.component(id, &SHARED).unwrap_or_default();
    shared.is_some()
        && w.component(player, &TRIBE)
            .is_some_and(|t| t.id == shared.id)
}

/// Whether `other` is on `id`'s side: its owner, a player of its tribe, or a tame either has.
fn friend(w: &World, id: u64, other: u64) -> bool {
    if other == id || can_command(w, id, other) {
        return true;
    }
    let mine = w.component(id, &OWNER).unwrap_or([0; 64]);
    let theirs = w.component(other, &OWNER).unwrap_or([0; 64]);
    if mine[0] != 0 && mine == theirs {
        return true;
    }
    let shared = w.component(id, &SHARED).unwrap_or_default();
    shared.is_some()
        && w.component(other, &SHARED)
            .is_some_and(|s| s.id == shared.id)
}

pub fn command(w: &World, id: u64) -> u8 {
    w.component(id, &ANIMAL).map_or(WANDER, |a| a.command)
}

/// Gives `id` command `c`, from where it is (or `at`).
pub fn set_command(w: &World, id: u64, c: u8, at: Option<[f64; 3]>) {
    let Some(st) = w.entity(id) else { return };
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        a.command = c;
    }
    if let Some(o) = w.component_mut(id, &ORDERS) {
        o.home = at.unwrap_or(st.pos.to_array());
        o.target = 0;
    }
}

/// A tame was just tamed: it stays where it is, neutral.
pub fn tamed(w: &World, id: u64) {
    set_command(w, id, STAY, None);
    if let Some(o) = w.component_mut(id, &ORDERS) {
        *o = Orders { home: o.home, ..DEFAULT };
    }
}

/// Its owner (or a tribe member) right clicks it with nothing it eats: follow, or stay.
pub fn toggle(w: &World, id: u64, player: u64) -> bool {
    if !can_command(w, id, player) {
        return false;
    }
    let c = if command(w, id) == FOLLOW { STAY } else { FOLLOW };
    set_command(w, id, c, None);
    if let Some(st) = w.entity(id) {
        notes(w, st.pos.to_array());
    }
    true
}

/// Its owner (or a tribe member) right clicks a tame: food it eats is eaten if it is hungry (and
/// its bond grows), anything else makes it follow or stay.
pub fn interact(w: &World, id: u64, player: u64) -> bool {
    if !can_command(w, id, player) {
        return false;
    }
    let held = w.inventory(player).get(w.selected_slot(player));
    let Some(a) = w.component(id, &ANIMAL) else { return false };
    if let Some((_, f)) = a.species().and_then(|s| s.food(held.item)) {
        // A favorite: a grown one falls in love, a young one grows (`breeding.rs`).
        if f.liking == species::Liking::Favorite && crate::breeding::fed(w, id, player) {
            animal::feed(w, id, f.food);
            return true;
        }
        if (a.food as f32) < a.stat(species::FOOD).round() {
            crate::taming::take_one(w, player);
            animal::feed(w, id, f.food);
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.bond = a.bond.saturating_add(2).min(100);
            }
            if let Some(st) = w.entity(id) {
                notes(w, st.pos.to_array());
            }
        }
        return true;
    }
    toggle(w, id, player)
}

/// An order from the animal screen.
fn order_one(w: &World, id: u64, player: u64, order: u32) {
    match order {
        0..4 => set_command(w, id, order as u8, None),
        10..14 => {
            let fights = species_of(w, id).is_some_and(|s| s.fights);
            if let Some(o) = w.component_mut(id, &ORDERS) {
                // One that never fights stays passive.
                o.stance = if fights { order - 10 } else { PASSIVE };
                o.target = 0;
            }
        }
        20 => {
            let tribe = w.component(player, &TRIBE).unwrap_or_default();
            if let Some(s) = w.component_mut(id, &SHARED) {
                *s = if s.is_some() || !tribe.is_some() { Tribe::default() } else { tribe };
            }
        }
        30 => {
            let held = w.inventory(player).get(w.selected_slot(player));
            crate::signatures::track(w, id, held.item);
        }
        40 if animal::owns(w, id, player) => release(w, id),
        _ => {}
    }
}

/// Its owner lets it go: wild again, with its levels (its bond, orders and tribe's share gone,
/// its saddle dropped beside it). Its genes join the wild ones where it is (`evolution.rs`).
fn release(w: &World, id: u64) {
    if crate::riding::rider_of(w, id).is_some() {
        return;
    }
    if let Some(o) = w.component_mut(id, &OWNER) {
        *o = [0; 64];
    }
    if let Some(o) = w.component_mut(id, &OWNER_ID) {
        *o = 0;
    }
    if let Some(s) = w.component_mut(id, &SHARED) {
        *s = Tribe::default();
    }
    if let Some(o) = w.component_mut(id, &ORDERS) {
        *o = DEFAULT;
    }
    set_command(w, id, WANDER, None);
    if let Some(a) = w.component_mut(id, &ANIMAL) {
        a.bond = 0;
    }
    crate::riding::unsaddle(w, id);
    crate::taming::released(w, id);
    crate::dog::on_released(w, id);
    crate::evolution::count(w, id, 1.0);
    weft_sdk::info!("animal {id} released");
}

fn species_of(w: &World, id: u64) -> Option<&'static species::Species> {
    w.component(id, &ANIMAL)?.species()
}

// ------------------------------------------------------------------ behavior

/// A goal for tames of species `index`, before resting: running from harm, fighting, catching
/// up with its owner, going back to where it stays or guards. Gets up from resting to do it.
pub fn act(index: usize) -> impl Fn(&World, u64, &EntityState) -> bool + Send + Sync + 'static {
    move |w, id, st| {
        if !animal::tamed(w, id) {
            return false;
        }
        let Some(mut o) = w.component(id, &ORDERS) else { return false };
        if o.home == [0.0; 3] {
            // Not set yet (a puppy born tame): where it is.
            o.home = st.pos.to_array();
            if let Some(h) = w.component_mut(id, &ORDERS) {
                h.home = st.pos.to_array();
            }
        }
        if o.flee > 0 {
            if let Some(o) = w.component_mut(id, &ORDERS) {
                o.flee = o.flee.saturating_sub(w.tick_elapsed(id) as _);
            }
            if (w.game_time() + id) % 20 == 0 {
                let a = w.random_below(628) as f64 / 100.0;
                crate::wake(w, id, index);
                w.navigate(id, [st.pos.x + a.cos() * 8.0, st.pos.y, st.pos.z + a.sin() * 8.0], 1.6);
            }
            return true;
        }
        if let Some(t) = target(w, id, st, &o, index) {
            crate::wake(w, id, index);
            fight(w, id, st, t);
            return true;
        }
        match command(w, id) {
            FOLLOW => {
                let Some(p) = owner(w, id).and_then(|p| w.entity(p)) else { return false };
                let d2 = dist2(p.pos, st.pos);
                if d2 > 24.0 * 24.0 || (p.pos.y - st.pos.y).abs() > 12.0 {
                    // Far behind: there in a moment, beside the owner.
                    w.entity_edit(id)
                        .pos([p.pos.x + 1.0, p.pos.y, p.pos.z + 1.0]);
                    return true;
                }
                let near = o.follow as f64 + 0.5;
                if d2 > near * near {
                    crate::wake(w, id, index);
                    w.navigate(id, p.pos, 1.3);
                    return true;
                }
                false
            }
            STAY | GUARD => {
                let leash = if command(w, id) == STAY { 1.5 } else { 4.0 };
                if dist2(o.home, st.pos) > leash * leash {
                    if (w.game_time() + id) % 10 == 0 || !w.navigating(id) {
                        crate::wake(w, id, index);
                        w.navigate(id, o.home, 1.1);
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

/// A goal for tames, after resting and before wandering: one that stays, guards or follows
/// doesn't wander off; a wanderer keeps near home.
pub fn hold(w: &World, id: u64, st: &EntityState) -> bool {
    if !animal::tamed(w, id) {
        return false;
    }
    match command(w, id) {
        STAY | GUARD | FOLLOW => {
            if w.navigating(id) {
                w.navigate(id, st.pos, 0.0);
            }
            true
        }
        _ => {
            let home = w
                .component(id, &ORDERS)
                .map_or(st.pos.to_array(), |o| o.home);
            if dist2(home, st.pos) > ROAM * ROAM {
                w.navigate(id, home, 1.0);
                return true;
            }
            false
        }
    }
}

/// What `id` should be fighting: the one it is on while that is still worth it, else (now and
/// then) whatever its stance says: what hit its owner, what its owner hit (assist), monsters
/// nearby (aggressive, or in the guarded area).
fn target(w: &World, id: u64, st: &EntityState, o: &Orders, index: usize) -> Option<u64> {
    if !SPECIES[index].fights || o.stance == PASSIVE {
        if o.target != 0 {
            set_target(w, id, 0);
        }
        return None;
    }
    let guarding = command(w, id) == GUARD;
    let worth = |t: u64| {
        w.entity(t).is_some_and(|e| {
            e.health > 0.0 && dist2(e.pos, st.pos) < CHASE * CHASE && (!guarding || dist2(e.pos, o.home) < (o.guard as f64 + 8.0).powi(2))
        }) && !friend(w, id, t)
    };
    if o.target != 0 && worth(o.target) {
        return Some(o.target);
    }
    if o.target != 0 {
        set_target(w, id, 0);
    }
    if (w.game_time() + id) % 10 != 0 {
        return None;
    }
    let now = w.game_time();
    let fight = owner(w, id).and_then(|p| w.component(p, &FIGHT));
    let mut found = None;
    if let Some([hit, hit_at, by, by_at]) = fight {
        if by != 0 && now.saturating_sub(by_at) < FIGHT_TICKS && worth(by) {
            found = Some(by);
        } else if o.stance == ASSIST && hit != 0 && now.saturating_sub(hit_at) < FIGHT_TICKS && worth(hit) {
            found = Some(hit);
        }
    }
    if found.is_none() && (o.stance == AGGRESSIVE || guarding) {
        let (center, radius) = if guarding { (o.home, o.guard as f64) } else { (st.pos.to_array(), AGGRO) };
        found = w
            .entities_near(center, radius, None)
            .into_iter()
            .filter(|&t| w.entity(t).is_some_and(|e| HOSTILE.on_entity(e.kind)) && worth(t))
            .min_by(|&a, &b| {
                let d = |t| w.entity(t).map_or(f64::MAX, |e| dist2(e.pos, st.pos));
                d(a).total_cmp(&d(b))
            });
    }
    if let Some(t) = found {
        set_target(w, id, t);
    }
    found
}

fn set_target(w: &World, id: u64, t: u64) {
    if let Some(o) = w.component_mut(id, &ORDERS) {
        o.target = t;
    }
}

/// Blocks any animal strikes from, whatever its size.
const LUNGE: f64 = 1.6;

/// After `t`, striking when in reach (the engine keeps the pace of its attacks), each hit shown
/// by one of its species' attacks (a bite, a swipe...), at random.
pub fn fight(w: &World, id: u64, st: &EntityState, t: u64) {
    fight_at(w, id, st, t, 1.4);
}

/// `fight` at `speed` times its kind's speed while it closes in (a pack runs its prey down).
pub fn fight_at(w: &World, id: u64, st: &EntityState, t: u64, speed: f32) {
    fight_aiming(w, id, st, t, speed, None);
}

/// `fight_at`, running at `aim` instead of straight at it while it isn't within reach (a pack's
/// flank, where the prey will be).
pub fn fight_aiming(w: &World, id: u64, st: &EntityState, t: u64, speed: f32, aim: Option<[f64; 3]>) {
    let Some(e) = w.entity(t) else { return };
    let species = w.component(id, &ANIMAL).and_then(|a| a.species());
    // At least a lunge: the path ends a block and a half short of a target that stands still.
    let reach = ((st.size.width + e.size.width) as f64 / 2.0 + species.map_or(0.8, |s| s.reach) as f64).max(LUNGE);
    // Running prey down, a pack bites at its flanks and hindquarters as it runs: a block further.
    let reach = if speed > 1.4 { reach + 1.0 } else { reach };
    let flat = (e.pos.x - st.pos.x).powi(2) + (e.pos.z - st.pos.z).powi(2);
    if flat < reach * reach && (e.pos.y - st.pos.y).abs() < 2.0 + st.size.height as f64 / 2.0 && w.attack(id, t) {
        let attacks = species.map_or(&[][..], |s| s.attacks);
        if !attacks.is_empty() {
            w.action_play(id, attacks[w.random_below(attacks.len() as u32) as usize], false);
        }
    }
    if (w.game_time() + id) % 5 == 0 || !w.navigating(id) {
        let to = aim
            .filter(|_| flat >= reach * reach * 4.0)
            .map_or(e.pos, Vec3::from);
        w.navigate(id, to, speed);
    }
}

/// Every hurt: who a player hit and who hit a player (for their tames); a tame hurt runs off
/// (passive) or fights back.
fn hurt(w: &World, h: &abi::Hurt) {
    let now = w.game_time();
    if h.source != 0
        && let Some(f) = w.component_mut(h.source, &FIGHT)
    {
        f[0] = h.entity;
        f[1] = now;
    }
    if h.source != 0
        && let Some(f) = w.component_mut(h.entity, &FIGHT)
    {
        f[2] = h.source;
        f[3] = now;
    }
    if h.source == 0 || !animal::tamed(w, h.entity) || friend(w, h.entity, h.source) {
        return;
    }
    let fights = species_of(w, h.entity).is_some_and(|s| s.fights);
    if let Some(o) = w.component_mut(h.entity, &ORDERS) {
        if o.stance == PASSIVE || !fights {
            o.flee = 60;
        } else if o.target == 0 {
            o.target = h.source;
        }
    }
}

// ------------------------------------------------------------------ whistles

/// Whistle `which` (follow, stay, come, attack my target, passive) to every tame in earshot the
/// player may command.
fn whistle(w: &World, player: u64, which: usize) {
    let Some(p) = w.entity(player) else { return };
    let target = (which == 3)
        .then(|| crate::screen::looked_at_any(w, player, CHASE, |t| !w.component(t, &OWNER).is_some_and(|k| k[0] != 0)))
        .flatten();
    let tames: Vec<u64> = w
        .entities_near(p.pos, WHISTLE_RANGE, None)
        .into_iter()
        .filter(|&id| w.component(id, &ANIMAL).is_some() && animal::tamed(w, id) && can_command(w, id, player))
        .collect();
    for (n, &id) in tames.iter().enumerate() {
        match which {
            0 => set_command(w, id, FOLLOW, None),
            1 => set_command(w, id, STAY, None),
            2 => {
                // Around the player, not all on one spot.
                let a = n as f64 * 2.4;
                set_command(w, id, STAY, Some([p.pos.x + a.cos() * 1.5, p.pos.y, p.pos.z + a.sin() * 1.5]));
            }
            3 => {
                let fights = species_of(w, id).is_some_and(|s| s.fights);
                if let (Some(t), Some(o)) = (target, w.component_mut(id, &ORDERS))
                    && fights
                    && o.stance != PASSIVE
                {
                    o.target = t;
                }
            }
            _ => {
                if let Some(o) = w.component_mut(id, &ORDERS) {
                    o.stance = PASSIVE;
                    o.target = 0;
                }
            }
        }
        if let Some(st) = w.entity(id) {
            notes(w, st.pos.to_array());
        }
    }
}

/// A sign it heard.
fn notes(w: &World, at: [f64; 3]) {
    drop(
        w.spawn_particles(&NOTE, [at[0], at[1] + 0.8, at[2]])
            .count(1)
            .spread(0.1)
            .velocity([0.0, 0.5, 0.0]),
    );
}

// ------------------------------------------------------------------ tribes

/// `/tribe create <name>`, `join` (the one you were invited to), `leave`, `info`.
fn tribe_command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    let mine = w.component(player, &TRIBE).unwrap_or_default();
    match cmd.text("action").unwrap_or("info") {
        "create" => {
            let name = cmd.text("name").unwrap_or("").trim().to_string();
            if name.is_empty() {
                cmd.reply("command.animals.tribe.name", &[]);
                return;
            }
            let mut t = Tribe::default();
            for b in t.id.iter_mut() {
                *b = w.random_below(256) as u8;
            }
            t.id[0] |= 1;
            for (d, s) in t.name.iter_mut().zip(name.bytes()) {
                *d = s;
            }
            if let Some(m) = w.component_mut(player, &TRIBE) {
                *m = t;
            }
            cmd.reply("command.animals.tribe.created", &[Arg::Text(&t.name())]);
        }
        "join" => {
            let invite = w.component(player, &INVITE).unwrap_or_default();
            if !invite.is_some() {
                cmd.reply("command.animals.tribe.no_invite", &[]);
                return;
            }
            if let Some(m) = w.component_mut(player, &TRIBE) {
                *m = invite;
            }
            if let Some(i) = w.component_mut(player, &INVITE) {
                *i = Tribe::default();
            }
            cmd.reply("command.animals.tribe.joined", &[Arg::Text(&invite.name())]);
        }
        "leave" => {
            if let Some(m) = w.component_mut(player, &TRIBE) {
                *m = Tribe::default();
            }
            cmd.reply("command.animals.tribe.left", &[Arg::Text(&mine.name())]);
        }
        _ => {
            if mine.is_some() {
                cmd.reply("command.animals.tribe.info", &[Arg::Text(&mine.name())]);
            } else {
                cmd.reply("command.animals.tribe.none", &[]);
            }
        }
    }
}
