//! Biome bosses in the game (`notes/PROGRESSION.md`, "Biome bosses"; their data in
//! `animals_core::bosses`): what every boss shares, so each boss's own file (`drenkaun.rs`,
//! `borkon.rs`) is only its `Def`, its altar's place and its attacks.
//!
//! - **Rising** at its altar with its offering (`offering`, then the altar's own check, then
//!   `rise`), one at a time near an altar.
//! - **The fight** (`fight`, a goal on the creature AI): it rises, chases the nearest player and
//!   asks its `Def` which attack to make (`choose`), shows its tell for the attack's wind-up
//!   (`tell`), then makes it (`act`); under `fierce_at` of its health its cooldowns are shorter. It
//!   stands its ground (knockback resistance 1, `core:knockback_resistance`) and goes back to its
//!   place when no one has been near for a minute. A health line at the top of the screen
//!   (`BAR`) for players near.
//! - **Stagger** (`hurt`, the damage step on `armour.rs`'s cap and floor): what spills over the
//!   rate floor and a share of every hit gather as stagger; enough knocks it down, and its parts
//!   break at their shares, each dropping its piece and softening an attack.
//! - **Loot** (`died`) for each player who fought it near, with bad-luck help (misses kept on the
//!   player, `LUCK`), a socket stone each (the perk web's boss find), and its young now and then,
//!   which its offering tames and which then follows its owner.
//! - `/boss state|hurt <n>|kill|luck` (and `/drenkaun`, its first name) for the checks.

use std::collections::HashMap;
use std::sync::Mutex;

use animals_core::bosses::{self, Attack, Boss, Form, STAGGER_SHARE};
use vitals::curve::Age;
use weft_sdk::ai::Goals;
use weft_sdk::*;

use crate::ModelFile;

/// A boss as the game has it: its data's name, its kinds and look, and its own parts of the fight.
pub struct Def {
    /// Its name in `animals_core::bosses` and its text keys (`title.animals.<name>`...).
    pub name: &'static str,
    pub entity: &'static EntityKind,
    /// Its young, tamed with `tame_with`, drawn `young_scale` times its own size.
    pub young: &'static EntityKind,
    /// Its young's name in its text keys (`hint.animals.<young_name>.tamed`).
    pub young_name: &'static str,
    pub young_scale: f32,
    pub tame_with: &'static Item,
    pub model: &'static str,
    pub texture: &'static str,
    pub scale: f32,
    pub speed: f32,
    /// Block bits for its dust (rising, stamping) and its breaks (parts, a stun, leaving).
    pub dust: &'static Block,
    pub bits: &'static Block,
    /// Where its head is (part breaks fall from it, tells show at it).
    pub head: fn(&EntityState) -> [f64; 3],
    /// Which attack to make now, if any.
    pub choose: fn(&Ctx) -> Option<usize>,
    /// Starting an attack's wind-up: where it will land (`Warden::marks`).
    pub begin: fn(&Ctx, &mut Warden),
    /// Its tell, every tick of the wind-up.
    pub tell: fn(&World, u64, &EntityState, &Warden, &Attack),
    /// The attack, every tick until it says it's done.
    pub act: fn(&World, u64, &EntityState, &mut Warden, &Attack, Strike) -> Step,
}

/// What `choose` and `begin` see.
pub struct Ctx<'a> {
    pub w: &'a World,
    pub st: &'a EntityState,
    pub t: &'a EntityState,
    pub b: &'static Boss,
    pub c: &'a Warden,
    /// Its target's distance past its body, and the cosine of the angle off its nose.
    pub dist: f64,
    pub facing: f64,
    pub fierce: bool,
}

impl Ctx<'_> {
    /// Attack `name`'s index if it is off cooldown.
    pub fn ready(&self, name: &str) -> Option<usize> {
        self.b
            .attacks
            .iter()
            .position(|a| a.name == name)
            .filter(|&i| self.c.cooldown[i] == 0)
    }

    pub fn reach(&self, name: &str) -> f64 {
        self.b.attack(name).map_or(0.0, |a| a.reach as f64)
    }

    /// Whether it sees its target.
    pub fn sees(&self) -> bool {
        sees(self.w, self.st, self.t)
    }

    /// How far above it its target stands (negative: below).
    pub fn above(&self) -> f64 {
        self.t.pos.y - self.st.pos.y
    }
}

/// An attack's numbers as it is made: its hit (the form's, the attack's share, softened by broken
/// parts), its cooldown, and the ticks since the last call.
#[derive(Clone, Copy)]
pub struct Strike {
    pub hit: f32,
    pub cooldown: u32,
    pub elapsed: u32,
}

impl Strike {
    /// Its first tick (an attack that lands at once lands on it).
    pub fn first(&self, c: &Warden) -> bool {
        c.t <= self.elapsed.max(1)
    }
}

/// What an attack says after its tick.
pub enum Step {
    Going,
    Done,
    /// It ended in a stun (a charge into a wall).
    Stunned,
}

static PLAYER: EntityKind = EntityKind::named("core:player");
static BITS: Particle = Particle::named("core:block");
/// Its health line at the top of the screen, for players near it (a place in the HUD editor).
static BAR: TextPlace = TextPlace::named("boss_health");

/// How far it looks for players, and how long it waits with none before going back.
pub const SIGHT: f64 = 40.0;
const LEAVE_TICKS: u32 = 1200;
/// Ticks it takes to rise when summoned, and stays stunned.
const RISE_TICKS: u32 = 60;
pub const STUN_TICKS: u32 = 60;
/// No second one of a kind within this of an altar.
const ONE_AT_A_TIME: f64 = 96.0;
/// Players this near when it dies get their loot.
const FOUGHT: f64 = 48.0;
/// How near its edge it comes to a player before it stops and turns to them.
const CLOSE: f64 = 1.5;
pub const TICKS: f32 = vitals::TICKS as f32;

pub const RISING: u32 = 0;
pub const CHASE: u32 = 1;
pub const WINDUP: u32 = 2;
pub const ACT: u32 = 3;
pub const DOWN: u32 = 4;
pub const STUNNED: u32 = 5;
const STATES: [&str; 6] = ["rising", "chase", "windup", "act", "down", "stunned"];

value! {
    #[derive(Default)]
    /// A boss's fight (saved, so a loaded one goes on as it was).
    pub struct Warden {
        /// Its form's age (tier).
        pub form: u32,
        pub state: u32,
        /// Ticks in its state.
        pub t: u32,
        /// The attack it is making (an index into its boss's `attacks`).
        pub attack: u32,
        pub target: u64,
        /// Ticks before each attack can come again.
        pub cooldown: [u32; 4],
        /// Stagger gathered in all (health points), and toward its next knockdown.
        pub stagger: f32,
        pub knock: f32,
        /// Parts broken, a bit each.
        pub broken: u32,
        /// Where an attack runs, and where it will land (up to three places).
        pub dir: [f32; 2],
        pub marks: [f32; 9],
        /// Ticks with no player near.
        pub alone: u32,
        /// Its altar.
        pub altar: [i32; 3],
    }
}

pub static WARDEN: Component<Warden> = Component::named("warden");
/// Drawn at its size since it loaded (not saved: `set_scale` isn't).
static SCALED: Component<u32> = Component::named("warden_scaled");
/// A player's misses for each boss's loot lines (bad-luck help): 8 a boss, in `BOSSES`' order, its
/// loot lines then its young last.
static LUCK: Component<[u8; 16]> = Component::named("boss_luck");
const LUCK_PER_BOSS: usize = 8;
/// A young one's owner's player key (zeros while wild).
static YOUNG_OWNER: Component<[u8; 64]> = Component::named("calf_owner");
/// Each one's last fight state (not saved).
static LAST: Mutex<Option<HashMap<u64, Warden>>> = Mutex::new(None);
/// Who each charge has already hit (not saved).
static TRAMPLED: Mutex<Option<HashMap<u64, Vec<u64>>>> = Mutex::new(None);

/// Every boss in the game.
pub fn defs() -> [&'static Def; 2] {
    [&crate::drenkaun::DEF, &crate::borkon::DEF]
}

/// The boss `id` is, if any.
pub fn def_of(w: &World, id: u64) -> Option<&'static Def> {
    let st = w.entity(id)?;
    defs().into_iter().find(|d| d.entity.id() == st.kind)
}

pub fn data(def: &Def) -> &'static Boss {
    bosses::boss(def.name).expect("every Def is in animals_core::bosses")
}

pub fn form_of(def: &Def, c: &Warden) -> &'static Form {
    data(def).form(Age::of_tier(c.form as u8))
}

/// What every boss shares: its fight state, the players' luck, the damage step, the health line
/// and the check command. Each boss calls `register` for its own kinds.
pub fn init(r: &mut Registry) {
    assert!(bosses::BOSSES.len() * LUCK_PER_BOSS <= 16, "boss_luck holds two bosses");
    r.entity_component(&WARDEN, Warden::default(), component_flags::SAVE);
    r.entity_component(&SCALED, 0, 0);
    r.entity_component(&LUCK, [0; 16], component_flags::SAVE);
    r.attach(&PLAYER, &LUCK);
    r.entity_component(&YOUNG_OWNER, [0; 64], component_flags::SAVE);
    // The damage step: the gear cap, its armour, the floor, and stagger from what spills over.
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| {
        let q = *m.query();
        let w = m.world();
        if w.component(q.entity, &WARDEN).is_none() {
            return;
        }
        let Some(def) = def_of(&w, q.entity) else { return };
        m.value = hurt(def, &w, q.entity, q.other, q.cause, m.value as f32) as f64;
    });
    r.text_place(&BAR)
        .at(0.5, 0.0, 0.0, 30.0)
        .scale(1.2)
        .times(0.0, 1.5, 0.4)
        .sample("hud.animals.boss_health.sample");
    r.command("boss", "<action:word(state,hurt,kill,luck)> [amount:float]", command);
    r.command("drenkaun", "<action:word(state,hurt,kill,luck)> [amount:float]", command);
}

/// Registers a boss's kinds: itself (its fight `goal`, which calls `fight`) and its young.
pub fn register(
    r: &mut Registry,
    def: &'static Def,
    goal: fn(&World, u64, &EntityState) -> bool,
    young_goal: fn(&World, u64, &EntityState) -> bool,
) {
    let b = data(def);
    let base = b.stats(&b.forms[0]);
    let (bw, bh) = animals_core::bodies::of(def.model);
    r.entity(def.entity)
        .size(bw, bh)
        .max_health(base.health)
        .walk_speed(def.speed)
        .attack_damage(0.0)
        .unpushable()
        .always_active()
        .model_file(def.model)
        .animations(def.model)
        .texture(def.texture)
        .attach(&WARDEN)
        .attach(&SCALED)
        .attach(&crate::armour::FLOOR)
        .goals(Goals::new().then(goal))
        .on_died(move |w, d| died(def, w, d.entity, d.pos.to_array()));
    r.entity(def.young)
        .size(bw, bh)
        .max_health(base.health / 40.0)
        .walk_speed(def.speed)
        .passive()
        .model_file(def.model)
        .animations(def.model)
        .texture(def.texture)
        .attach(&SCALED)
        .attach(&YOUNG_OWNER)
        .on_interact(move |e| young_interact(def, &e.world(), e.entity(), e.other()))
        .goals(Goals::new().then(young_goal).then(ai::wander(0.3, 8.0)));
}

// ------------------------------------------------------------------ summoning

/// The form the offering in `player`'s hand calls at an altar of `def`, if it is one.
pub fn offering(def: &Def, w: &World, player: u64, at: [i32; 3]) -> Option<&'static Form> {
    let held = w.inventory(player).get(w.selected_slot(player));
    let name = name_of(abi::entry_kind::ITEM, held.item);
    let form = data(def).form_for(name);
    if form.is_none() {
        w.hint(player, &format!("hint.animals.{}.offering", def.name), &[]);
        weft_sdk::info!("{} altar {at:?}: no offering in hand ({name})", def.name);
    }
    form
}

/// It rises at the altar `at` for `player` in `form` (its altar's place already checked, `why`
/// says how): one at a time, a few blocks past the altar away from the player.
pub fn rise(def: &'static Def, w: &World, player: u64, at: [i32; 3], form: &Form, why: &str) -> bool {
    let mid = [at[0] as f64 + 0.5, at[1] as f64, at[2] as f64 + 0.5];
    if !w
        .entities_near(mid, ONE_AT_A_TIME, Some(def.entity))
        .is_empty()
    {
        w.hint(player, &format!("hint.animals.{}.awake", def.name), &[]);
        weft_sdk::info!("{} altar {at:?}: one is already awake near", def.name);
        return true;
    }
    crate::taming::take_one(w, player);
    let p = w.entity(player).map_or(mid, |s| s.pos.to_array());
    let (dx, dz) = (mid[0] - p[0], mid[2] - p[2]);
    let len = (dx * dx + dz * dz).sqrt().max(0.1);
    let spot = [mid[0] + dx / len * 6.0, mid[1], mid[2] + dz / len * 6.0];
    let spot = ground(w, spot).unwrap_or(mid);
    let id = w.spawn(def.entity, spot);
    if id == 0 {
        return true;
    }
    let b = data(def);
    let max = b.stats(form).health;
    w.attribute_add_modifier(
        id,
        &attribute::MAX_HEALTH,
        "animals:form",
        attr_op::MULTIPLY_BASE,
        (max / b.stats(&b.forms[0]).health) as f64 - 1.0,
        0,
    );
    w.entity_edit(id).health(max);
    w.set_component(id, &WARDEN, Warden { form: form.age.tier() as u32, state: RISING, target: player, altar: at, ..Warden::default() });
    w.action_play(id, "rise", false);
    burst(w, spot, def.dust.id(), 40, 1.5);
    for p in w.entities_near(spot, SIGHT, Some(&PLAYER)) {
        w.title(p, &format!("title.animals.{}", def.name), &[])
            .times(0.5, 3.0, 1.0);
        w.subtitle(p, &format!("subtitle.animals.{}", def.name), &[])
            .times(0.5, 3.0, 1.0);
    }
    weft_sdk::info!("{} {id}: summoned at {at:?} ({} form, {max:.0} health, {why})", def.name, form.age.name());
    true
}

/// The ground under (or a little over) `p`: where something can stand.
pub fn ground(w: &World, p: [f64; 3]) -> Option<[f64; 3]> {
    let (x, z) = (p[0].floor() as i32, p[2].floor() as i32);
    let y0 = p[1].floor() as i32;
    (y0 - 6..=y0 + 6)
        .rev()
        .find(|&y| solid(w, [x, y - 1, z]) && !solid(w, [x, y, z]) && !solid(w, [x, y + 1, z]))
        .map(|y| [x as f64 + 0.5, y as f64, z as f64 + 0.5])
}

pub fn solid(w: &World, p: [i32; 3]) -> bool {
    block_info(w.block(p)).is_some_and(|b| b.solid())
}

// ------------------------------------------------------------------ the fight

/// A boss's goal, every tick: rise, chase, wind up and make an attack, or lie stunned or knocked
/// down.
pub fn fight(def: &'static Def, w: &World, id: u64, st: &EntityState) -> bool {
    let Some(mut c) = w.component(id, &WARDEN) else { return false };
    let e = w.tick_elapsed(id);
    if w.component(id, &SCALED) != Some(1) {
        if def.scale != 1.0 {
            w.set_scale(id, def.scale);
        }
        w.set_component(id, &SCALED, 1);
        // A titan stands its ground (`core:knockback_resistance`).
        w.attribute_add_modifier(
            id,
            &attribute::KNOCKBACK_RESISTANCE,
            "animals:size",
            attr_op::ADD,
            bosses::SIZE.band().knockback as f64,
            0,
        );
    }
    for cd in c.cooldown.iter_mut() {
        *cd = cd.saturating_sub(e);
    }
    c.t = c.t.saturating_add(e);
    let b = data(def);
    // The nearest player it can fight; none for a while and it goes back to its place.
    let target = nearest_player(w, st);
    match target {
        Some(p) => {
            c.target = p;
            c.alone = 0;
        }
        None => {
            c.alone += e;
            if c.alone >= LEAVE_TICKS {
                weft_sdk::info!("{} {id}: no one to fight, back to its place", def.name);
                burst(w, st.pos.to_array(), def.bits.id(), 40, 2.0);
                w.despawn(id);
                LAST.lock()
                    .unwrap()
                    .get_or_insert_with(HashMap::new)
                    .remove(&id);
                return true;
            }
        }
    }
    let fierce = st.health < st.max_health * b.fierce_at;
    health_line(def, w, id, st, &c, fierce);
    match c.state {
        RISING => {
            stand(w, id, st);
            if c.t >= RISE_TICKS {
                go(&mut c, CHASE);
            }
        }
        DOWN | STUNNED => {
            stand(w, id, st);
            let ticks = if c.state == DOWN { (b.down_secs * TICKS) as u32 } else { STUN_TICKS };
            if c.t >= ticks {
                w.action_play(id, "", false);
                go(&mut c, CHASE);
            }
        }
        CHASE => {
            let Some(t) = target.and_then(|p| w.entity(p)) else {
                stand(w, id, st);
                w.set_component(id, &WARDEN, c);
                return true;
            };
            let to = [t.pos.x - st.pos.x, t.pos.z - st.pos.z];
            let dist = (to[0] * to[0] + to[1] * to[1]).sqrt();
            let half = st.size.width as f64 / 2.0;
            let dir = [(to[0] / dist.max(0.1)) as f32, (to[1] / dist.max(0.1)) as f32];
            let ctx = Ctx { w, st, t: &t, b, c: &c, dist: dist - half, facing: facing(st.yaw, to), fierce };
            match (def.choose)(&ctx) {
                Some(i) => {
                    let mut next = c;
                    next.attack = i as u32;
                    next.dir = dir;
                    (def.begin)(&ctx, &mut next);
                    c = next;
                    w.navigate(id, st.pos, 0.0);
                    w.action_play(id, &format!("{}_tell", b.attacks[i].name), false);
                    go(&mut c, WINDUP);
                }
                // Close enough: it stops and turns to them rather than walk into them (PC look
                // boss_drenkaun1: it stood on the player's spot and every swing missed).
                None if dist - half < CLOSE => {
                    stand(w, id, st);
                    face(w, id, dir);
                }
                None => {
                    w.navigate(id, t.pos, 1.0);
                }
            }
        }
        WINDUP => {
            let a = &b.attacks[c.attack as usize];
            stand(w, id, st);
            face(w, id, c.dir);
            (def.tell)(w, id, st, &c, a);
            if c.t as f32 >= a.windup * TICKS {
                w.action_play(id, a.name, false);
                TRAMPLED
                    .lock()
                    .unwrap()
                    .get_or_insert_with(HashMap::new)
                    .insert(id, Vec::new());
                go(&mut c, ACT);
            }
        }
        ACT => {
            let i = c.attack as usize;
            let a = b.attacks[i];
            let strike = Strike {
                hit: b.stats(form_of(def, &c)).hit * a.hit * weakened(b, &c, a.name),
                cooldown: (a.cooldown * TICKS * if fierce { bosses::FIERCE } else { 1.0 }) as u32,
                elapsed: e,
            };
            match (def.act)(w, id, st, &mut c, &a, strike) {
                Step::Going => {}
                Step::Done => {
                    c.cooldown[i] = strike.cooldown;
                    go(&mut c, CHASE);
                }
                Step::Stunned => {
                    c.cooldown[i] = strike.cooldown;
                    weft_sdk::info!("{} {id}: stunned", def.name);
                    burst(w, (def.head)(st), def.bits.id(), 20, 1.0);
                    w.action_play(id, "stunned", true);
                    go(&mut c, STUNNED);
                }
            }
        }
        _ => go(&mut c, CHASE),
    }
    w.set_component(id, &WARDEN, c);
    remember(id, c);
    true
}

/// Its health line, once a second, for the players near.
fn health_line(def: &Def, w: &World, id: u64, st: &EntityState, c: &Warden, fierce: bool) {
    if !(w.game_time() + id).is_multiple_of(20) {
        return;
    }
    let state = match c.state {
        DOWN => "hud.animals.boss_health.down",
        STUNNED => "hud.animals.boss_health.stunned",
        _ if fierce => "hud.animals.boss_health.fierce",
        _ => "hud.animals.boss_health.fighting",
    };
    let pct = (100.0 * st.health / st.max_health.max(1.0)).ceil() as i64;
    let title = format!("title.animals.{}", def.name);
    for p in w.entities_near(st.pos, SIGHT, Some(&PLAYER)) {
        drop(w.show_at(
            p,
            &BAR,
            "hud.animals.boss_health",
            &[Arg::Key(&title), Arg::Int(st.health.ceil() as i64), Arg::Int(st.max_health.ceil() as i64), Arg::Int(pct), Arg::Key(state)],
        ));
    }
}

/// Its last state, for when it has died (its components are gone by then).
fn remember(id: u64, c: Warden) {
    LAST.lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(id, c);
}

pub fn go(c: &mut Warden, state: u32) {
    c.state = state;
    c.t = 0;
}

/// A softer attack for each part broken that weakens it.
fn weakened(b: &Boss, c: &Warden, attack: &str) -> f32 {
    b.parts
        .iter()
        .enumerate()
        .filter(|(i, p)| c.broken & (1 << i) != 0 && p.weakens == Some(attack))
        .map(|_| bosses::BROKEN)
        .product()
}

/// Whether nothing solid stands between its eyes and `t`'s middle.
pub fn sees(w: &World, st: &EntityState, t: &EntityState) -> bool {
    let from = [st.pos.x, st.pos.y + st.size.eye as f64, st.pos.z];
    let to = [t.pos.x, t.pos.y + t.size.height as f64 * 0.5, t.pos.z];
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    len < 1.0 || w.raycast(from, d.map(|v| v / len), len, false).is_none()
}

/// The cosine of the angle between where it faces (`yaw`) and `to`.
pub fn facing(yaw: f32, to: [f64; 2]) -> f64 {
    let f = yaw_dir(yaw);
    let len = (to[0] * to[0] + to[1] * to[1]).sqrt().max(1e-6);
    (f[0] as f64 * to[0] + f[1] as f64 * to[1]) / len
}

pub fn face(w: &World, id: u64, dir: [f32; 2]) {
    drop(w.entity_edit(id).yaw((-dir[0]).atan2(-dir[1])));
}

pub fn stand(w: &World, id: u64, st: &EntityState) {
    if w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
}

/// The nearest player it can fight (in survival, alive, in sight range).
fn nearest_player(w: &World, st: &EntityState) -> Option<u64> {
    w.entities_near(st.pos, SIGHT, Some(&PLAYER))
        .into_iter()
        .filter(|&p| !w.outside_survival(p))
        .filter_map(|p| w.entity(p).filter(|s| s.health > 0.0).map(|s| (p, s)))
        .min_by(|a, b| dist2(a.1.pos.to_array(), st.pos.to_array()).total_cmp(&dist2(b.1.pos.to_array(), st.pos.to_array())))
        .map(|(p, _)| p)
}

pub fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

/// A charge's tick (`def`'s run at `speed` times its walk for at most `ticks`): what is in front of
/// it takes the hit and is thrown (`toss` up as much as the knockback across), once a charge; into
/// a trunk or a wall it is stunned.
#[allow(clippy::too_many_arguments)]
pub fn charge(
    def: &Def,
    w: &World,
    id: u64,
    st: &EntityState,
    c: &Warden,
    a: &Attack,
    s: Strike,
    speed: f32,
    ticks: u32,
    toss: f64,
) -> Step {
    w.walk(id, c.dir, speed, false);
    face(w, id, c.dir);
    let half = st.size.width as f64 / 2.0;
    let f = yaw_dir(st.yaw);
    let front = [st.pos.x + f[0] as f64 * half, st.pos.y, st.pos.z + f[1] as f64 * half];
    {
        let mut guard = TRAMPLED.lock().unwrap();
        let done = guard
            .get_or_insert_with(HashMap::new)
            .entry(id)
            .or_default();
        let k = a.knockback as f64;
        for e in w.entities_near(front, half + a.reach as f64, None) {
            if e == id || done.contains(&e) || spared(w, e) {
                continue;
            }
            let Some(o) = w.entity(e) else { continue };
            if (o.pos.y - st.pos.y).abs() > st.size.height as f64 {
                continue;
            }
            done.push(e);
            w.damage(e, s.hit, id, damage_cause::ATTACK);
            w.push(e, [f[0] as f64 * k, k * toss, f[1] as f64 * k]);
            weft_sdk::info!("{} {id}: {} hits {e} for {:.1}", def.name, a.name, s.hit);
        }
    }
    let ahead = [
        (st.pos.x + c.dir[0] as f64 * (half + 0.6)).floor() as i32,
        (st.pos.y + 1.0).floor() as i32,
        (st.pos.z + c.dir[1] as f64 * (half + 0.6)).floor() as i32,
    ];
    let blocked = c.t > 6 && solid(w, ahead) && solid(w, [ahead[0], ahead[1] + 1, ahead[2]]);
    if blocked {
        weft_sdk::info!("{} {id}: charged into {ahead:?}", def.name);
        Step::Stunned
    } else if c.t >= ticks {
        Step::Done
    } else {
        Step::Going
    }
}

/// Hits each of `hits` (`spared` ones left out) and throws it by its push.
pub fn land(def: &Def, w: &World, id: u64, a: &Attack, hit: f32, hits: Vec<(u64, [f64; 3])>) {
    for (e, push) in hits {
        if e == id || spared(w, e) {
            continue;
        }
        w.damage(e, hit, id, damage_cause::ATTACK);
        w.push(e, push);
        weft_sdk::info!("{} {id}: {} hits {e} for {hit:.1}", def.name, a.name);
    }
}

/// What a boss's attacks never hit: a boss, and every boss's young (PC look boss_drenkaun_hud1: a
/// second Drenkaun trampled the first one's calf).
pub fn spared(w: &World, e: u64) -> bool {
    w.component(e, &WARDEN).is_some()
        || w.entity(e)
            .is_some_and(|s| defs().iter().any(|d| d.young.id() == s.kind))
}

pub fn burst(w: &World, at: [f64; 3], state: u32, count: u32, spread: f64) {
    drop(
        w.spawn_particles(&BITS, at)
            .block(state)
            .count(count)
            .spread(spread as f32)
            .outward(),
    );
}

// ------------------------------------------------------------------ the damage step and stagger

/// A hit of `raw` on boss `id`: what it takes after the gear cap, its armour and the floor; its
/// stagger grows by what spilled over and a share of what it took, knocking it down and breaking
/// its parts.
fn hurt(def: &Def, w: &World, id: u64, other: u64, cause: u32, raw: f32) -> f32 {
    let Some(mut c) = w.component(id, &WARDEN) else { return raw };
    let Some(st) = w.entity(id) else { return raw };
    if cause == damage_cause::VOID || raw <= 0.0 {
        return raw;
    }
    // Rising it can't be hurt yet.
    if c.state == RISING {
        return 0.0;
    }
    let b = data(def);
    let f = form_of(def, &c);
    let level = b.level(f);
    let size = bosses::SIZE;
    let mut hit = raw * crate::armour::player_cap(w, other, cause, size, level);
    let capped = hit;
    if DamageCause::flags_of(cause) & damage_flags::BYPASSES_ARMOR == 0 {
        hit *= 1.0 - vitals::combat::armour_cut(b.stats(f).armour, hit);
    }
    let (took, over, _) = crate::armour::floor(w, id, size, st.max_health, hit);
    let gained = over + took * STAGGER_SHARE;
    c.stagger += gained;
    c.knock += gained;
    weft_sdk::info!(
        "{} {id}: hit {raw:.2}, capped {capped:.2}, armour {hit:.2}, took {took:.2}, stagger {:.1} ({:.0}% of its health), health {:.1}/{:.1}",
        def.name,
        c.stagger,
        100.0 * c.stagger / st.max_health,
        (st.health - took).max(0.0),
        st.max_health
    );
    // Its parts, in order.
    for (i, p) in b.parts.iter().enumerate() {
        if c.broken & (1 << i) == 0 && c.stagger >= p.at * st.max_health {
            c.broken |= 1 << i;
            let at = (def.head)(&st);
            if let Some(item) = item_id(p.drop) {
                w.spawn_item(at, ItemStack::new(item, p.count));
            }
            burst(w, at, def.bits.id(), 30, 1.5);
            for pl in w.entities_near(st.pos, SIGHT, Some(&PLAYER)) {
                w.subtitle(pl, &format!("subtitle.animals.{}.{}", def.name, p.name), &[])
                    .times(0.2, 2.0, 0.5);
            }
            weft_sdk::info!("{} {id}: its {} breaks", def.name, p.name);
        }
    }
    // A knockdown, unless it is already down.
    if c.knock >= b.knockdown * st.max_health && c.state != DOWN {
        c.knock = 0.0;
        TRAMPLED
            .lock()
            .unwrap()
            .get_or_insert_with(HashMap::new)
            .remove(&id);
        w.action_play(id, "down", true);
        go(&mut c, DOWN);
        weft_sdk::info!("{} {id}: knocked down", def.name);
    }
    w.set_component(id, &WARDEN, c);
    remember(id, c);
    took
}

fn item_id(name: &str) -> Option<u32> {
    let id = Item::generated(name.to_string()).id();
    (id != 0).then_some(id)
}

// ------------------------------------------------------------------ loot and the young

/// It died at `pos`: every player who fought it near gets their roll of its loot, with bad-luck
/// help, and a socket stone; its young may stay behind.
fn died(def: &Def, w: &World, id: u64, pos: [f64; 3]) {
    let b = data(def);
    let last = LAST
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .remove(&id);
    let c = last
        .or_else(|| w.component(id, &WARDEN))
        .unwrap_or_default();
    let f = form_of(def, &c);
    let index = bosses::BOSSES
        .iter()
        .position(|x| x.name == b.name)
        .unwrap_or(0);
    let lines: Vec<&bosses::Loot> = b.loot_for(f).collect();
    let mut young = false;
    for p in w.entities_near(pos, FOUGHT, Some(&PLAYER)) {
        let mut luck = w.component(p, &LUCK).unwrap_or([0; 16]);
        let mut got = Vec::new();
        let base = index * LUCK_PER_BOSS;
        for (i, l) in lines.iter().enumerate().take(LUCK_PER_BOSS - 1) {
            let misses = luck[base + i] as u32;
            if roll(w) < l.chance_after(misses) {
                luck[base + i] = 0;
                let n = l.count_at(roll(w));
                if let Some(item) = item_id(l.item) {
                    w.spawn_item([pos[0], pos[1] + 1.0, pos[2]], ItemStack::new(item, n));
                }
                got.push(format!("{} x{n}", l.item));
            } else {
                luck[base + i] = luck[base + i].saturating_add(1);
                got.push(format!("{} missed ({} in a row)", l.item, luck[base + i]));
            }
        }
        let y = base + LUCK_PER_BOSS - 1;
        if roll(w) < b.young.chance_after(luck[y] as u32) {
            luck[y] = 0;
            young = true;
        } else {
            luck[y] = luck[y].saturating_add(1);
        }
        w.set_component(p, &LUCK, luck);
        // A socket stone for the perk web (`vitals::sockets`, a boss's find: one a kill, rolled at
        // its form's age).
        if base_api::skills::drop_stone(w, base_api::skills::StoneSource::Boss, f.age.tier(), [pos[0], pos[1] + 1.0, pos[2]]) {
            got.push("a socket stone".into());
        }
        weft_sdk::info!("{} {id}: loot for player {p}: {}", def.name, got.join(", "));
    }
    if young {
        let at = ground(w, pos).unwrap_or(pos);
        let y = w.spawn(def.young, at);
        weft_sdk::info!("{} {id}: its young {y} stays behind", def.name);
    }
    weft_sdk::info!("{} {id}: died ({} form, stagger {:.0}, parts broken {:03b})", def.name, f.age.name(), c.stagger, c.broken);
    TRAMPLED
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .remove(&id);
}

fn roll(w: &World) -> f32 {
    w.random_below(1_000_000) as f32 / 1_000_000.0
}

/// A young one: offered its kind's offering, it takes to that player and follows them.
fn young_interact(def: &Def, w: &World, id: u64, player: u64) -> bool {
    let held = w.inventory(player).get(w.selected_slot(player));
    if held.item != def.tame_with.id() {
        return false;
    }
    let owner = w.component(id, &YOUNG_OWNER).unwrap_or([0; 64]);
    if owner != [0; 64] {
        return false;
    }
    let key = crate::player_key(player);
    let mut k = [0u8; 64];
    for (i, b) in key.bytes().take(64).enumerate() {
        k[i] = b;
    }
    w.set_component(id, &YOUNG_OWNER, k);
    crate::taming::take_one(w, player);
    if let Some(st) = w.entity(id) {
        drop(
            w.spawn_particles(&crate::HEART, [st.pos.x, st.pos.y + 1.2, st.pos.z])
                .count(6)
                .spread(0.5),
        );
    }
    w.hint(player, &format!("hint.animals.{}.tamed", def.young_name), &[]);
    weft_sdk::info!("{} young {id}: tamed by player {player}", def.name);
    true
}

/// A young one's goal: drawn small; a tamed one follows its owner.
pub fn young(def: &Def, w: &World, id: u64, st: &EntityState) -> bool {
    if w.component(id, &SCALED) != Some(1) {
        w.set_scale(id, def.young_scale);
        w.set_component(id, &SCALED, 1);
    }
    let owner = w.component(id, &YOUNG_OWNER).unwrap_or([0; 64]);
    if owner == [0; 64] {
        return false;
    }
    let Some(ps) = w
        .players_iter()
        .find(|&p| {
            crate::player_key(p)
                .as_bytes()
                .iter()
                .take(64)
                .eq(owner.iter().take_while(|&&b| b != 0))
        })
        .and_then(|p| w.entity(p))
    else {
        return false;
    };
    let d = dist2(ps.pos.to_array(), st.pos.to_array());
    if d > 16.0 && d < 48.0 * 48.0 {
        w.navigate(id, ps.pos, 1.2);
        return true;
    }
    if d <= 9.0 {
        stand(w, id, st);
    }
    d <= 16.0
}

// ------------------------------------------------------------------ the check command

/// `/boss state|hurt <n>|kill|luck` (or `/drenkaun`), for the checks: the nearest boss's fight
/// state; a hit of `n` from the player as their held weapon's (through the cap, its armour and the
/// floor); its death with loot as from the void; the player's misses.
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    if cmd.text("action") == Some("luck") {
        let luck = w.component(player, &LUCK).unwrap_or([0; 16]);
        cmd.reply("command.animals.line", &[Arg::Text(&format!("misses {luck:?}"))]);
        return;
    }
    let Some(me) = w.entity(player) else { return };
    let Some(id) = w
        .entities_near(me.pos, 64.0, None)
        .into_iter()
        .filter(|&e| w.component(e, &WARDEN).is_some())
        .min_by(|a, b| {
            let d = |e: u64| {
                w.entity(e)
                    .map_or(f64::MAX, |s| dist2(s.pos.to_array(), me.pos.to_array()))
            };
            d(*a).total_cmp(&d(*b))
        })
    else {
        cmd.reply("command.animals.line", &[Arg::Text("no boss near")]);
        return;
    };
    match cmd.text("action").unwrap_or("state") {
        "hurt" => {
            let n = cmd.float("amount").unwrap_or(10.0) as f32;
            w.damage(id, n, player, damage_cause::ATTACK);
        }
        "kill" => {
            w.damage(id, 1.0e6, player, damage_cause::VOID);
        }
        _ => {}
    }
    let name = def_of(&w, id).map_or("boss", |d| d.name);
    let c = w.component(id, &WARDEN).unwrap_or_default();
    let st = w.entity(id);
    let line = format!(
        "{name} {id}: state {} t {} health {:.1}/{:.1} stagger {:.1} knock {:.1} broken {:03b} cooldowns {:?} at {:.1} {:.1} {:.1}",
        STATES.get(c.state as usize).unwrap_or(&"?"),
        c.t,
        st.map_or(0.0, |s| s.health),
        st.map_or(0.0, |s| s.max_health),
        c.stagger,
        c.knock,
        c.broken,
        c.cooldown,
        st.map_or(0.0, |s| s.pos.x),
        st.map_or(0.0, |s| s.pos.y),
        st.map_or(0.0, |s| s.pos.z),
    );
    weft_sdk::info!("{line}");
    cmd.reply("command.animals.line", &[Arg::Text(&line)]);
}
