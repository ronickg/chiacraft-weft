//! What the player sees of an animal (`notes/ANIMALS.md`, "What the player sees"): the animal
//! screen (sneak and use one you own) with every stat, its value and its wild and tamed points,
//! where unspent points are placed; and the spyglass, which shows any animal you look at from
//! afar: species, sex, life stage and level.

use weft_sdk::*;

use crate::animal::{self, ANIMAL, Animal, OWNER, STAGE_NAMES};
use crate::commands::{COMMAND_NAMES, ORDER, ORDERS, SHARED, STANCE_NAMES};
use crate::knockout::KNOCKOUT;
use crate::species::{self, Diet, Liking, STAT_NAMES, STATS, Size, Taming};
use crate::taming::TAMING;

static ANIMAL_MENU: Menu = Menu::named("animal");
static SCOUT_MENU: Menu = Menu::named("scout");
pub static SPYGLASS: Item = Item::named("spyglass");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
/// Client to server: place a point, as (animal, stat).
static PLACE: Channel<[u64; 2]> = Channel::named("place_point");

/// How far the spyglass sees.
const SPYGLASS_RANGE: f64 = 64.0;

pub fn init(r: &mut Registry) {
    r.menu(&ANIMAL_MENU).screen(animal_screen);
    r.menu(&SCOUT_MENU).screen(scout_screen);
    r.packet(&PLACE, |p, [id, stat]| animal::place_point(&p.world(), id, p.player(), stat as usize));
    r.item(&SPYGLASS).max_stack(1).tab(&TOOLS).on_use(|u| {
        let (w, player) = (u.world(), u.player());
        match looked_at(&w, player, SPYGLASS_RANGE) {
            Some(id) => open(&w, player, id),
            None => false,
        }
    });
    r.shaped(&CRAFTING, "spyglass", 1, &["  g", " i ", "i  "], &[('g', "#glass"), ('i', "#ingots/iron")]);
    r.command("animal", "[action:word(show,tame,points,wild,here,gift,bite,ko,love,hurry,nest,evolve,fire,frost,lightning,venom,scar,age,speed,census,reset,away)] [count:int] [habitat:word(grassland,forest,taiga,snow,desert,mountains)]", command)
        .permission(permission::GAME_MASTER);
}

/// Opens the animal screen for its owner (or anyone in creative), else the scout view. True if
/// `id` is an animal.
pub fn open(w: &World, player: u64, id: u64) -> bool {
    if w.component(id, &ANIMAL).is_none() {
        return false;
    }
    let full = crate::commands::can_command(w, id, player) || w.creative(player);
    w.open_menu(player, if full { &ANIMAL_MENU } else { &SCOUT_MENU }, id);
    true
}

/// A player right clicks an animal: with the spyglass, its screen; sneaking, the animal screen
/// of one they own. True if it opened (the click is used).
pub fn interact(w: &World, id: u64, player: u64) -> bool {
    let held = w.inventory(player).get(w.selected_slot(player)).item;
    let sneaking = w
        .entity(player)
        .is_some_and(|p| p.pose == abi::pose::CROUCH);
    if held == SPYGLASS.id() || (sneaking && (crate::commands::can_command(w, id, player) || w.creative(player))) {
        return open(w, player, id);
    }
    false
}

/// The animal `player` looks at within `range`, in front of any block in the way.
pub fn looked_at(w: &World, player: u64, range: f64) -> Option<u64> {
    looked_at_any(w, player, range, |id| w.component(id, &ANIMAL).is_some())
}

/// The entity `player` looks at within `range` that passes `pick`, in front of any block in
/// the way.
pub fn looked_at_any(w: &World, player: u64, range: f64, pick: impl Fn(u64) -> bool) -> Option<u64> {
    let st = w.entity(player)?;
    let eye = [st.pos[0], st.pos[1] + st.size[2] as f64, st.pos[2]];
    let (sy, cy) = (st.yaw as f64).sin_cos();
    let (sp, cp) = (st.pitch as f64).sin_cos();
    let dir = [-sy * cp, sp, -cy * cp];
    let wall = w.look_at(player, range, false).map_or(range, |hit| {
        let c = hit.pos.map(|v| v as f64 + 0.5);
        ((c[0] - eye[0]).powi(2) + (c[1] - eye[1]).powi(2) + (c[2] - eye[2]).powi(2)).sqrt()
    });
    w.near(eye, range, None)
        .into_iter()
        .filter(|&id| id != player && pick(id))
        .filter_map(|id| {
            let e = w.entity(id)?;
            let (hw, h) = (e.size[0] as f64 / 2.0 + 0.1, e.size[1] as f64 + 0.1);
            let lo = [e.pos[0] - hw, e.pos[1] - 0.1, e.pos[2] - hw];
            let hi = [e.pos[0] + hw, e.pos[1] + h, e.pos[2] + hw];
            Some((id, ray_box(eye, dir, lo, hi)?))
        })
        .filter(|&(_, t)| t <= wall)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Where a ray from `o` along `d` enters the box, as a distance (slab test).
fn ray_box(o: [f64; 3], d: [f64; 3], lo: [f64; 3], hi: [f64; 3]) -> Option<f64> {
    let (mut near, mut far) = (0.0f64, f64::MAX);
    for i in 0..3 {
        if d[i].abs() < 1e-9 {
            if o[i] < lo[i] || o[i] > hi[i] {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo[i] - o[i]) / d[i], (hi[i] - o[i]) / d[i]);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

// ------------------------------------------------------------------ screens

fn species_key(a: &Animal) -> String {
    a.species()
        .map_or_else(String::new, |s| format!("entity.animals.{}", s.kind_name()))
}

fn sex_key(a: &Animal) -> &'static str {
    if a.female != 0 { "screen.animals.female" } else { "screen.animals.male" }
}

fn stage_key(a: &Animal) -> String {
    format!("screen.animals.stage.{}", STAGE_NAMES[(a.stage as usize).min(STAGE_NAMES.len() - 1)])
}

/// The owner as the screen knows it: whether there is one.
fn tamed(c: &ClientView) -> bool {
    c.menu_component(&OWNER).is_some_and(|k| k[0] != 0)
}

/// Name, sex and stage, level: the top of both screens.
fn heading(ui: &mut Ui) {
    ui.text_bind_styled(S::default().scale(1.5).no_wrap(), |c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        match a.species() {
            // A breed with its species: "Chihuahua (dog)".
            Some(s) if s.breed.is_some() => t.set(
                "screen.animals.heading_breed",
                &[Arg::Key(&species_key(&a)), Arg::Key(&format!("species.animals.{}", s.name)), Arg::Int(a.level() as i64)],
            ),
            _ => t.set("screen.animals.heading", &[Arg::Key(&species_key(&a)), Arg::Int(a.level() as i64)]),
        }
    });
    // Where it sits in the tree of life: "Canis familiaris · Mammalia › Carnivora › Canidae
    // (dog family) › Caninae".
    ui.text_bind(|c, t| {
        let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
        let above = s.family.above.join(" › ");
        t.set(
            "screen.animals.taxon",
            &[
                Arg::Text(s.binomial),
                Arg::Text(&above),
                Arg::Text(s.family.name),
                Arg::Key(&format!("family.animals.{}", s.family.name.to_lowercase())),
                Arg::Text(s.subfamily),
            ],
        );
    });
    ui.text_bind(|c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        let owner = if tamed(c) { "screen.animals.tamed" } else { "screen.animals.wild" };
        t.set("screen.animals.about", &[Arg::Key(sex_key(&a)), Arg::Key(&stage_key(&a)), Arg::Key(owner)]);
    });
    // How big it is ("Size: 21 cm at the shoulder"; a young one, what it grows to).
    ui.text_bind(|c, t| {
        let (Some(a), Some(g)) = (c.menu_component(&ANIMAL), c.menu_component(&crate::coats::GENOME)) else { return };
        let Some(s) = a.species() else { return };
        let m = s.height.0 * crate::coats::size(&g);
        let shown = if m < 1.0 { format!("{:.0} cm", m * 100.0) } else { format!("{m:.2} m") };
        let key = if a.stage < animal::ADULT { "screen.animals.height_young" } else { "screen.animals.height" };
        t.set(key, &[Arg::Text(&shown), Arg::Key(&format!("screen.animals.height.{}", s.height.1))]);
    });
    // How old it is, and how long its kind lives (`animal::old_age`).
    ui.text_bind(|c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        let Some((age, span, vigor)) = animal::age_days(&a) else { return t.text("") };
        t.set("screen.animals.age", &[Arg::Int(age as i64), Arg::Int(span as i64), Arg::Int(vigor as i64)]);
    });
    // In love, expecting, growing up or resting (`breeding.rs`), when it is.
    ui.text_bind(|c, t| {
        let Some(b) = c.menu_component(&crate::breeding::BREEDING) else { return };
        let laying = c
            .menu_component(&ANIMAL)
            .and_then(|a| a.species())
            .is_some_and(|s| s.gestation() == 0);
        let min = |ticks: u32| Arg::Int(ticks.div_ceil(1200) as i64);
        match () {
            _ if b.love > 0 => t.set("screen.animals.love", &[]),
            _ if b.carrying > 0 && laying => t.set("screen.animals.laying", &[min(b.carrying)]),
            _ if b.carrying > 0 => t.set("screen.animals.expecting", &[min(b.carrying)]),
            _ if b.growing > 0 => t.set("screen.animals.growing", &[min(b.growing)]),
            _ if b.rest > 0 => t.set("screen.animals.resting", &[min(b.rest)]),
            _ => t.text(""),
        }
    });
    // Its family: its generation and its parents' coats, or born in the wild.
    ui.text_bind(|c, t| {
        let Some(b) = c.menu_component(&crate::breeding::BREEDING) else { return };
        let coat = |g: &[u32; 10], more: &crate::breeding::More| {
            crate::breeding::unpack(g, more)
                .and_then(|g| crate::coats::phenotype(&g))
                .map_or_else(|| "?".to_string(), |l| l.name)
        };
        if b.generation == 0 {
            t.set("screen.animals.wild_born", &[]);
        } else {
            t.set(
                "screen.animals.family",
                &[Arg::Int(b.generation as i64), Arg::Text(&coat(&b.dam, &b.dam_more)), Arg::Text(&coat(&b.sire, &b.sire_more))],
            );
        }
    });
    // Its coat, read from its genes ("Fawn sable with white").
    ui.text_bind(|c, t| {
        let Some(look) = c
            .menu_component(&crate::coats::GENOME)
            .and_then(|g| crate::coats::phenotype(&g))
        else {
            return;
        };
        t.set("screen.animals.coat", &[Arg::Text(&look.name)]);
    });
    ui.text_bind(|c, t| {
        let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
        t.set("screen.animals.signature", &[Arg::Key(&format!("screen.animals.signature.{}", s.signature))]);
    });
}

/// A stat's value as it is read: health with a decimal, damage and speed as a percentage of
/// the species' own, the rest whole.
fn shown(a: &Animal, i: usize) -> String {
    match i {
        species::HEALTH => format!("{:.1}", a.stat(i)),
        species::DAMAGE | species::SPEED => format!("{:.0}%", a.factor(i) * 100.0),
        _ => format!("{:.0}", a.stat(i)),
    }
}

fn animal_screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(8.0).gap(3.0).min_w(250.0), |ui| {
        heading(ui);
        ui.text_bind(|c, t| {
            let Some(a) = c.menu_component(&ANIMAL) else { return };
            // Health now, and its stat as the maximum (clients see no attribute modifiers of
            // others: their max health is the kind's).
            let Some(e) = c.menu_owner().and_then(|id| c.client().entity(id)) else { return };
            t.set(
                "screen.animals.needs",
                &[
                    Arg::Float(e.health as f64, 1),
                    Arg::Float(a.stat(species::HEALTH) as f64, 1),
                    Arg::Int(a.food as i64),
                    Arg::Int(a.stat(species::FOOD).round() as i64),
                    Arg::Int(a.bond as i64),
                ],
            );
        });
        ui.row(S::row().gap(4.0), |ui| {
            ui.text_styled(S::default().w(70.0), "screen.animals.stat", &[]);
            ui.text_styled(S::default().w(50.0), "screen.animals.value", &[]);
            ui.text_styled(S::default().w(40.0), "screen.animals.wild_points", &[]);
            ui.text_styled(S::default().w(40.0), "screen.animals.tamed_points", &[]);
        });
        for (i, name) in STAT_NAMES.iter().enumerate() {
            ui.row(S::row().gap(4.0), |ui| {
                ui.text_styled(S::default().w(70.0), &format!("stat.animals.{name}"), &[]);
                ui.text_bind_styled(S::default().w(50.0).mono(), move |c, t| {
                    if let Some(a) = c.menu_component(&ANIMAL) {
                        t.text(&shown(&a, i));
                    }
                });
                ui.text_bind_styled(S::default().w(40.0).mono(), move |c, t| {
                    if let Some(a) = c.menu_component(&ANIMAL) {
                        t.text(&a.wild[i].to_string());
                    }
                });
                ui.text_bind_styled(S::default().w(40.0).mono(), move |c, t| {
                    if let Some(a) = c.menu_component(&ANIMAL) {
                        t.text(&a.tamed[i].to_string());
                    }
                });
                if i != species::TORPOR {
                    ui.button(
                        "screen.animals.plus",
                        S::default()
                            .size(14.0, 12.0)
                            .tooltip("screen.animals.place"),
                        move |c| {
                            if let Some(id) = c.menu_owner() {
                                c.client().send(&PLACE, &[id, i as u64]);
                            }
                        },
                    );
                }
            });
        }
        // Orders: what it does and how it fights, and sharing with your tribe.
        ui.text_bind(|c, t| {
            let (Some(a), Some(o)) = (c.menu_component(&ANIMAL), c.menu_component(&ORDERS)) else { return };
            let command = format!("screen.animals.command.{}", COMMAND_NAMES[(a.command as usize).min(3)]);
            let stance = format!("screen.animals.stance.{}", STANCE_NAMES[(o.stance as usize).min(3)]);
            let shared = c
                .menu_component(&SHARED)
                .map(|s| {
                    String::from_utf8_lossy(&s.name)
                        .trim_end_matches('\0')
                        .to_string()
                })
                .unwrap_or_default();
            if shared.is_empty() {
                t.set("screen.animals.orders", &[Arg::Key(&command), Arg::Key(&stance)]);
            } else {
                t.set("screen.animals.orders_shared", &[Arg::Key(&command), Arg::Key(&stance), Arg::Text(&shared)]);
            }
        });
        ui.row(S::row().gap(3.0), |ui| {
            for (i, name) in COMMAND_NAMES.iter().enumerate() {
                order_button(ui, &format!("screen.animals.do.{name}"), i as u64);
            }
        });
        ui.row(S::row().gap(3.0), |ui| {
            for (i, name) in STANCE_NAMES.iter().enumerate() {
                order_button(ui, &format!("screen.animals.stance.{name}"), 10 + i as u64);
            }
            order_button(ui, "screen.animals.share", 20);
            order_button(ui, "screen.animals.track", 30);
            order_button(ui, "screen.animals.release", 40);
        });
        ui.text_bind(|c, t| {
            let Some(a) = c.menu_component(&ANIMAL) else { return };
            t.set(
                "screen.animals.points",
                &[Arg::Int(a.unspent as i64), Arg::Int(a.tamed_points() as i64), Arg::Int(animal::MAX_TAMED_POINTS as i64)],
            );
        });
    });
}

/// A button that gives the animal whose screen is open `order` (`commands::ORDER`).
fn order_button(ui: &mut Ui, key: &str, order: u64) {
    ui.button(key, S::default().h(14.0).pad(2.0), move |c| {
        if let Some(id) = c.menu_owner() {
            c.client().send(&ORDER, &[id, order]);
        }
    });
}

fn scout_screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(8.0).gap(3.0).min_w(180.0), |ui| {
        heading(ui);
        ui.text_bind(|c, t| {
            let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
            let size = match s.size {
                Size::Small => "screen.animals.size.small",
                Size::Medium => "screen.animals.size.medium",
                Size::Large => "screen.animals.size.large",
            };
            let diet = match s.diet {
                Diet::Carnivore => "screen.animals.diet.carnivore",
                Diet::Herbivore => "screen.animals.diet.herbivore",
            };
            let taming = match s.taming {
                Taming::Passive => "screen.animals.taming.passive",
                Taming::Knockout => "screen.animals.taming.knockout",
            };
            t.set("screen.animals.kind", &[Arg::Key(size), Arg::Key(diet), Arg::Key(taming)]);
        });
        ui.text_bind(|c, t| {
            t.text("");
            let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
            if let Some(f) = s.foods.iter().find(|f| f.liking == Liking::Favorite) {
                t.set("screen.animals.favorite", &[Arg::Name(&format!("item:{}", f.item))]);
            }
        });
        // Knocked out: its torpor, and what it has to eat.
        ui.text_bind(|c, t| {
            t.text("");
            let Some(k) = c.menu_component(&KNOCKOUT).filter(|k| k.torpor > 0.0) else { return };
            let Some(a) = c.menu_component(&ANIMAL) else { return };
            let key = if k.out > 0 { "screen.animals.torpor_out" } else { "screen.animals.torpor" };
            t.set(key, &[Arg::Int(k.torpor as i64), Arg::Int(a.stat(species::TORPOR) as i64)]);
        });
        ui.text_bind(|c, t| {
            t.text("");
            let Some(k) = c.menu_component(&KNOCKOUT).filter(|k| k.out > 0) else { return };
            if k.count == 0 {
                t.set("screen.animals.mouth_empty", &[]);
            } else {
                t.set("screen.animals.mouth", &[Arg::Item(k.food), Arg::Int(k.count as i64), Arg::Int((k.bite / 20) as i64)]);
            }
        });
        // A wild one's taming so far, and what it asks for now.
        ui.text_bind(|c, t| {
            t.text("");
            let Some(g) = c
                .menu_component(&TAMING)
                .filter(|g| g.feedings > 0 && !tamed(c))
            else {
                return;
            };
            t.set("screen.animals.taming", &[Arg::Int((g.progress * 100.0) as i64), Arg::Int((g.effectiveness * 100.0).round() as i64)]);
        });
        ui.text_bind(|c, t| {
            t.text("");
            let Some(g) = c
                .menu_component(&TAMING)
                .filter(|g| g.asking > 0 && !tamed(c))
            else {
                return;
            };
            let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
            if let Some(f) = s.foods.get(g.want as usize - 1) {
                t.set("screen.animals.wants", &[Arg::Name(&format!("item:{}", f.item))]);
            }
        });
    });
}

// ------------------------------------------------------------------ the command

/// The animal nearest `player` within `range`.
fn nearest(w: &World, player: u64, range: f64) -> Option<u64> {
    let p = w.entity(player)?.pos;
    let d2 = |id: u64| {
        w.entity(id)
            .map_or(f64::MAX, |e| (0..3).map(|i| (e.pos[i] - p[i]).powi(2)).sum())
    };
    w.near(p, range, None)
        .into_iter()
        .filter(|&id| id != player && w.component(id, &ANIMAL).is_some())
        .min_by(|&a, &b| d2(a).total_cmp(&d2(b)))
}

/// `/animal` on the animal you look at (or the nearest): `show` its screen, `tame` it (yours), give it `points`
/// to place, make it `wild` again with new levels, bring it `here` (two blocks ahead), have a
/// cat bring its next `gift` now, knock one out (`ko`), an unconscious one take its next `bite`
/// now, every grown animal near you be yours and in `love`, or every mother, egg and young one
/// near you `hurry` (birth, hatching, growing up within a second), or the nearest wild female of a
/// laying kind build a `nest` and lay in it now; a wyvern breathe `fire`, `frost`, `lightning` or
/// `venom` from now on. `evolve [count] [habitat]` runs
/// `count` generations of every species' gene pool where you stand (`evolution.rs`) and says how
/// common each variant is.
fn command(cmd: &mut ServerCommand) {
    let (w, player) = (cmd.world(), cmd.player());
    if cmd.text("action") == Some("evolve") {
        let count = cmd.int("count").unwrap_or(0).clamp(0, 1000) as u32;
        let habitat = cmd
            .text("habitat")
            .and_then(crate::evolution::Habitat::named);
        for line in crate::evolution::evolve_here(&w, player, count, habitat) {
            weft_sdk::info!("{line}");
            cmd.reply("command.animals.pool", &[Arg::Text(&line)]);
        }
        return;
    }
    // Testing the animals' lives: faster (`/animal speed <n>`), and how they are doing round the
    // player (`/animal census [radius]`; `/animal reset` zeroes its counts).
    match cmd.text("action") {
        Some("speed") => {
            crate::breeding::set_speed(cmd.int("count").unwrap_or(1).max(1) as u32);
            let line = format!("animals live {} times faster", crate::breeding::speed());
            cmd.reply("command.animals.line", &[Arg::Text(&line)]);
            return;
        }
        Some("census") => {
            for line in census(&w, player, cmd.int("count").unwrap_or(256).clamp(8, 4096) as f64) {
                weft_sdk::info!("census: {line}");
                cmd.reply("command.animals.line", &[Arg::Text(&line)]);
            }
            return;
        }
        // Every animal round the player as if `count` days had passed while its area was away
        // (testing `animal::catch_up`; the world's clock itself can't be turned on).
        Some("away") => {
            let days = cmd.int("count").unwrap_or(30).clamp(1, 10_000) as u64;
            let Some(p) = w.entity(player) else { return };
            let mut n = 0;
            for id in w.near(p.pos, 256.0, None) {
                if let Some(a) = w.component_mut(id, &ANIMAL).filter(|a| a.seen != 0) {
                    a.seen = a.seen.saturating_sub(days * animals_core::life::DAY).max(1);
                    a.born = a.born.saturating_sub(days * animals_core::life::DAY).max(1);
                    n += 1;
                }
            }
            let line = format!("{n} animals: {days} days passed while away");
            cmd.reply("command.animals.line", &[Arg::Text(&line)]);
            return;
        }
        Some("reset") => {
            for c in crate::breeding::BORN
                .iter()
                .chain(&crate::breeding::OLD)
                .chain(&crate::breeding::DIED)
            {
                c.store(0, std::sync::atomic::Ordering::Relaxed);
            }
            cmd.reply("command.animals.line", &[Arg::Text("census counts reset")]);
            return;
        }
        _ => {}
    }
    let Some(id) = looked_at(&w, player, 16.0).or_else(|| nearest(&w, player, 12.0)) else {
        cmd.reply("command.animals.none", &[]);
        return;
    };
    match cmd.text("action").unwrap_or("show") {
        "tame" => {
            crate::taming::tame(&w, id, player, 0.0);
            cmd.reply("command.animals.tamed", &[]);
        }
        "points" => {
            let n = cmd.int("count").unwrap_or(10).clamp(0, STATS as i64 * 100) as u16;
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.unspent = a.unspent.saturating_add(n);
            }
            cmd.reply("command.animals.points", &[Arg::Int(n as i64)]);
        }
        "wild" => {
            if let Some(o) = w.component_mut(id, &OWNER) {
                *o = [0; 64];
            }
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.species = 0;
            }
            cmd.reply("command.animals.wild", &[]);
        }
        "gift" => crate::signatures::gift_now(&w, id),
        "bite" => crate::knockout::bite_now(&w, id),
        "ko" => crate::knockout::add_torpor(&w, id, 100_000.0),
        "love" => crate::breeding::love_near(&w, player),
        "hurry" => crate::breeding::hurry_near(&w, player),
        "nest" => {
            crate::eggs::nest_now(&w, player);
        }
        // Older by `count` in-game days (testing old age: `animal::old_age`).
        "age" => {
            let days = cmd.int("count").unwrap_or(10).clamp(0, 100_000) as u64;
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.born = a.born.saturating_sub(days * animals_core::life::DAY).max(1);
            }
        }
        // A scar of each kind (testing scars: `scars.rs`).
        "scar" => {
            if let Some(g) = w.component_mut(id, &crate::coats::GENOME) {
                for kind in [crate::coats::CLAW, crate::coats::BITE, crate::coats::SLASH, crate::coats::TEAR] {
                    crate::coats::scar(g, kind, w.random_below(256) as u8);
                }
            }
        }
        kind @ ("fire" | "frost" | "lightning" | "venom") => {
            if let Some(g) = w.component_mut(id, &crate::coats::GENOME) {
                crate::coats::set_breath(g, kind);
            }
        }
        "here" => {
            if let (Some(p), Some(mut e)) = (w.entity(player), w.entity(id)) {
                let (sy, cy) = (p.yaw as f64).sin_cos();
                e.pos = [p.pos[0] - sy * 2.0, p.pos[1], p.pos[2] - cy * 2.0];
                w.set_entity(id, &e);
                w.navigate(id, e.pos, 0.0);
            }
        }
        _ => {
            open(&w, player, id);
        }
    }
}

/// Every species' animals within `radius` of `player`, a line each: how many and at what age,
/// expecting and in love, their mean strength, and since the counts were reset how many were
/// born and died (of old age, or otherwise).
fn census(w: &World, player: u64, radius: f64) -> Vec<String> {
    use std::sync::atomic::Ordering::Relaxed;
    let Some(p) = w.entity(player) else { return Vec::new() };
    let mut lines = Vec::new();
    let near = w.near(p.pos, radius, None);
    for (i, s) in species::SPECIES.iter().enumerate() {
        let mut n = [0u32; 5];
        let (mut expecting, mut love, mut vigor, mut tame) = (0, 0, 0.0, 0);
        for &id in &near {
            let Some(a) = w
                .component(id, &ANIMAL)
                .filter(|a| a.species as usize == i + 1)
            else {
                continue;
            };
            n[(a.stage as usize).min(4)] += 1;
            vigor += if a.vigor == 0 { 100.0 } else { a.vigor as f32 };
            if let Some(b) = w.component(id, &crate::breeding::BREEDING) {
                expecting += (b.carrying > 0) as u32;
                love += (b.love > 0) as u32;
            }
            tame += animal::tamed(w, id) as u32;
        }
        let total: u32 = n.iter().sum();
        let (born, old, died) =
            (crate::breeding::BORN[i].load(Relaxed), crate::breeding::OLD[i].load(Relaxed), crate::breeding::DIED[i].load(Relaxed));
        if total == 0 && born + old + died == 0 {
            continue;
        }
        lines.push(format!(
            "{}: {total} ({} young, {} grown, {} old; {tame} tame) - {expecting} expecting, {love} in love, strength {:.0}% - born {born}, died {old} of age + {died} otherwise",
            s.breed.unwrap_or(s.name),
            n[0] + n[1] + n[2],
            n[3],
            n[4],
            if total > 0 { vigor / total as f32 } else { 0.0 },
        ));
    }
    if lines.is_empty() {
        lines.push("no animals near".into());
    }
    lines
}
