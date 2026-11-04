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
/// The same view opened without the spyglass in hand (`/animal show` on one you don't own): its
/// title says so (Playtest B6: it said "Through the spyglass" with no spyglass used).
static LOOK_MENU: Menu = Menu::named("look");
/// The animal screen's other pages (round 2: what was a wall of text moves to tabs): its genes
/// and kind, and its family and life. Each is a menu of its own on the same animal; a tab asks
/// the server to open the page (`PAGE`).
static GENES_MENU: Menu = Menu::named("animal_genes");
static FAMILY_MENU: Menu = Menu::named("animal_family");
static PAGE: Channel<[u64; 2]> = Channel::named("animal_page");
/// The pages' names, in tab order (`screen.animals.tab.<name>`).
const PAGES: [&str; 3] = ["overview", "genes", "family"];
pub static SPYGLASS: Item = Item::named("spyglass");
static TOOLS: CreativeTab = CreativeTab::named("base:tools");
static CRAFTING: RecipeKind = RecipeKind::named("base:crafting");
/// Client to server: place a point, as (animal, stat).
static PLACE: Channel<[u64; 2]> = Channel::named("place_point");

/// How far the spyglass sees.
const SPYGLASS_RANGE: f64 = 64.0;

static TAMING_HUD: HudElement = HudElement::named("taming");

pub fn init(r: &mut Registry) {
    r.menu(&ANIMAL_MENU).screen(animal_screen);
    taming_hud(r);
    r.menu(&SCOUT_MENU).screen(scout_screen);
    r.menu(&LOOK_MENU).screen(scout_screen);
    r.menu(&GENES_MENU).screen(genes_screen);
    r.menu(&FAMILY_MENU).screen(family_screen);
    r.packet(&PAGE, |p, [id, page]| {
        let (w, player) = (p.world(), p.player());
        if !animal::ensure_rolled(&w, id) || !(crate::commands::can_command(&w, id, player) || w.creative(player)) {
            return;
        }
        let menu = [&ANIMAL_MENU, &GENES_MENU, &FAMILY_MENU][(page as usize).min(2)];
        w.open_menu(player, menu, id);
    });
    r.packet(&PLACE, |p, [id, stat]| animal::place_point(&p.world(), id, p.player(), stat as usize));
    r.item(&SPYGLASS).max_stack(1).tab(&TOOLS).on_use(|u| {
        let (w, player) = (u.world(), u.player());
        match looked_at(&w, player, SPYGLASS_RANGE) {
            Some(id) => open(&w, player, id),
            None => false,
        }
    });
    r.shaped(&CRAFTING, "spyglass", 1, &["  g", " i ", "i  "], &[('g', "#glass"), ('i', "#ingots/iron")]);
    r.command("animal", "[action:word(show,tame,points,wild,here,gift,bite,ko,love,hurry,nest,evolve,fire,frost,lightning,venom,scar,age,speed,census,reset,room,away,habitats,hungry,thirsty)] [count:int] [habitat:word(grassland,forest,taiga,snow,desert,mountains)]", command)
        .permission(permission::GAME_MASTER);
}

/// Opens the animal screen for its owner (or anyone in creative), else the scout view. True if
/// `id` is an animal.
pub fn open(w: &World, player: u64, id: u64) -> bool {
    if w.component(id, &ANIMAL).is_none() || !animal::ensure_rolled(w, id) {
        return false;
    }
    let full = crate::commands::can_command(w, id, player) || w.creative(player);
    let spyglass = w.inventory(player).get(w.selected_slot(player)).item == SPYGLASS.id();
    w.open_menu(
        player,
        if full {
            &ANIMAL_MENU
        } else if spyglass {
            &SCOUT_MENU
        } else {
            &LOOK_MENU
        },
        id,
    );
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
    title(ui, true);
    facts(ui);
}

/// The species (and breed) as the screen's title, with its level unless a badge shows it.
fn title(ui: &mut Ui, with_level: bool) {
    ui.text_bind_styled(S::default().class("title").no_wrap(), move |c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        match a.species() {
            // A breed with its species: "Chihuahua (dog)".
            Some(s) if s.breed.is_some() && !with_level => {
                t.set("screen.animals.name_breed", &[Arg::Key(&species_key(&a)), Arg::Key(&format!("species.animals.{}", s.name))])
            }
            Some(s) if s.breed.is_some() => t.set(
                "screen.animals.heading_breed",
                &[Arg::Key(&species_key(&a)), Arg::Key(&format!("species.animals.{}", s.name)), Arg::Int(a.level() as i64)],
            ),
            _ if !with_level => t.set("screen.animals.name", &[Arg::Key(&species_key(&a))]),
            _ => t.set("screen.animals.heading", &[Arg::Key(&species_key(&a)), Arg::Int(a.level() as i64)]),
        }
    });
}

/// Where it sits in the tree of life, sex and stage, size, age, breeding, family, coat and what
/// its kind does: the lines under the title. `short`: without what the animal screen shows
/// elsewhere (sex and stage, the coat, what its kind does).
fn facts(ui: &mut Ui) {
    facts_of(ui, false);
}

fn facts_of(ui: &mut Ui, short: bool) {
    taxon_line(ui);
    if !short {
        about_line(ui);
    }
    height_line(ui);
    age_line(ui);
    breeds_line(ui);
    breeding_line(ui);
    family_line(ui);
    if short {
        return;
    }
    coat_line(ui);
    signature_line(ui);
}

/// Where it sits in the tree of life: "Canis familiaris · Mammalia › Carnivora › Canidae
/// (dog family) › Caninae".
fn taxon_line(ui: &mut Ui) {
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
}

fn about_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        let owner = if tamed(c) { "screen.animals.tamed" } else { "screen.animals.wild" };
        t.set("screen.animals.about", &[Arg::Key(sex_key(&a)), Arg::Key(&stage_key(&a)), Arg::Key(owner)]);
    });
}

/// How big it is ("Size: 21 cm at the shoulder"; a young one, what it grows to).
fn height_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        let (Some(a), Some(g)) = (c.menu_component(&ANIMAL), c.menu_component(&crate::coats::GENOME)) else { return };
        let Some(s) = a.species() else { return };
        let m = s.height.0 * crate::coats::size(&g);
        // The unit in the line's own translation ("cm" stayed English in Russian).
        let (shown, unit) = if m < 1.0 { (format!("{:.0}", m * 100.0), "cm") } else { (format!("{m:.2}"), "m") };
        let key = if a.stage < animal::ADULT { "screen.animals.height_young" } else { "screen.animals.height" };
        t.set(&format!("{key}_{unit}"), &[Arg::Text(&shown), Arg::Key(&format!("screen.animals.height.{}", s.height.1))]);
    });
}

/// How old it is, and how long its kind lives (`animal::old_age`).
fn age_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        let Some(a) = c.menu_component(&ANIMAL) else { return };
        let Some((age, span, vigor)) = animal::age_days(&a) else { return t.text("") };
        t.set("screen.animals.age", &[Arg::Int(age as i64), Arg::Int(span as i64), Arg::Int(vigor as i64)]);
    });
}

/// When its kind breeds, and whether it's that season now where the player is ("Breeds in
/// spring, summer and autumn · in season now"; the wet season in a world with a wet and a dry).
fn breeds_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        use animals_core::biology::seasons;
        let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return t.text("") };
        let v = c.component(&base_api::VITALS).filter(|v| v.date > 0);
        let wet_dry = v.is_some_and(|v| v.calendar == base_api::calendars::WET_AND_DRY);
        let now = v.map(|v| seasons::bit((v.date as u32 - 1) / vitals::calendar::SEASON_DAYS));
        let status = match now {
            None => "",
            Some(bit) if s.bio.breeds & bit != 0 => "screen.animals.in_season",
            Some(_) => "screen.animals.out_of_season",
        };
        let status = if status.is_empty() { Arg::Text("") } else { Arg::Key(status) };
        let mut names: Vec<String> = Vec::new();
        if s.bio.breeds == seasons::ALL_YEAR {
            return t.set("screen.animals.breeds_all", &[status]);
        } else if wet_dry {
            for (mask, name) in [(seasons::WET, "wet"), (seasons::ALL_YEAR & !seasons::WET, "dry")] {
                if s.bio.breeds & mask != 0 {
                    // Spring alone is the wet season's first half.
                    let early = mask == seasons::WET && s.bio.breeds & mask == seasons::SPRING;
                    names.push(format!("screen.animals.season.{}", if early { "early_wet" } else { name }));
                }
            }
        } else {
            for i in 0..4 {
                if s.bio.breeds & seasons::bit(i) != 0 {
                    names.push(format!("season.base.{}", seasons::NAMES[i as usize]));
                }
            }
        }
        let mut args: Vec<Arg> = names.iter().map(|n| Arg::Key(n)).collect();
        let key = format!("screen.animals.breeds_{}", args.len().clamp(1, 3));
        while args.len() < 3 {
            args.push(Arg::Text(""));
        }
        args.push(status);
        t.set(&key, &args);
    });
}

/// In love, expecting, growing up or resting (`breeding.rs`), when it is.
fn breeding_line(ui: &mut Ui) {
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
}

/// Its family: its generation and its parents' coats, or born in the wild.
fn family_line(ui: &mut Ui) {
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
            // In pieces, the coats' words translated each (`coat_pieces`).
            let mut p = vec![Piece::Key("screen.animals.family_generation".into()), Piece::Text(format!(" {} · ", b.generation))];
            p.push(Piece::Key("screen.animals.family_mother".into()));
            p.push(Piece::Text(" ".into()));
            p.extend(coat_pieces(&coat(&b.dam, &b.dam_more)));
            p.push(Piece::Text(" · ".into()));
            p.push(Piece::Key("screen.animals.family_father".into()));
            p.push(Piece::Text(" ".into()));
            p.extend(coat_pieces(&coat(&b.sire, &b.sire_more)));
            set_pieces(t, &p);
        }
    });
}

/// Its coat, read from its genes ("Fawn sable with white").
fn coat_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        let Some(look) = c
            .menu_component(&crate::coats::GENOME)
            .and_then(|g| crate::coats::phenotype(&g))
        else {
            return;
        };
        let mut p = vec![Piece::Key("screen.animals.coat_label".into()), Piece::Text(" ".into())];
        p.extend(coat_pieces(&look.name));
        set_pieces(t, &p);
    });
}

/// A gene's name as a translation key: `gene.<its name, spaces as _>` ("Dominant black").
fn gene_key(name: &str) -> String {
    format!("gene.{}", name.replace(' ', "_"))
}

/// A piece of a text put together in the client: a translation, or text as it is (numbers,
/// spaces, commas).
enum Piece {
    Key(String),
    Text(String),
}

/// The phrases a coat's name is made of that read as one in a translation ("Dark brown" is one
/// word in Russian, "and tan" one phrase), longest first; the other words are translated each.
const COAT_PHRASES: [&str; 30] = [
    "darker grey back",
    "breathes lightning",
    "breathes venom",
    "breathes frost",
    "breathes fire",
    "golden nape",
    "Grizzled buff",
    "Glossy black",
    "Rufous brown",
    "Female brown",
    "Sandy brown",
    "winter coat",
    "Rusty brown",
    "Tawny grey",
    "Sandy grey",
    "Warm brown",
    "Dark brown",
    "Dark olive",
    "with white",
    "Light tan",
    "black bib",
    "Pale gold",
    "Pale grey",
    "Pale sand",
    "Pale buff",
    "Warm sand",
    "Dark grey",
    "Deep gold",
    "grey back",
    "and tan",
];

/// A coat's name ("Fawn sable with white", made in `animals_core::coats` from its genes) as pieces
/// to translate: each phrase or word a key `coat.<words>`, the spaces and commas between as they
/// are (Ronald, 11-04: the coat chips were English in Russian).
fn coat_pieces(name: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut rest = name;
    while !rest.is_empty() {
        if let Some(p) = COAT_PHRASES
            .iter()
            .find(|p| rest.starts_with(**p) && !rest[p.len()..].starts_with(|c: char| c.is_alphabetic() || c == '-'))
        {
            out.push(Piece::Key(format!("coat.{}", p.replace(' ', "_"))));
            rest = &rest[p.len()..];
            continue;
        }
        let word: String = rest
            .chars()
            .take_while(|c| c.is_alphabetic() || *c == '-')
            .collect();
        if word.is_empty() {
            let sep: String = rest.chars().take_while(|c| !c.is_alphabetic()).collect();
            rest = &rest[sep.len()..];
            out.push(Piece::Text(sep));
        } else {
            rest = &rest[word.len()..];
            out.push(Piece::Key(format!("coat.{word}")));
        }
    }
    out
}

/// Sets `t` to `pieces` joined (`screen.animals.join_<n>`, "{0}{1}..."), at most `JOIN_MAX`.
fn set_pieces(t: &mut TextOut, pieces: &[Piece]) {
    let pieces = &pieces[..pieces.len().min(JOIN_MAX)];
    let args: Vec<Arg> = pieces
        .iter()
        .map(|p| match p {
            Piece::Key(k) => Arg::Key(k),
            Piece::Text(s) => Arg::Text(s),
        })
        .collect();
    t.set(&format!("screen.animals.join_{}", args.len()), &args);
}

/// The most pieces a joined text has (`screen.animals.join_<n>` up to it in the lang files).
const JOIN_MAX: usize = 48;

fn signature_line(ui: &mut Ui) {
    ui.text_bind(|c, t| {
        let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
        t.set("screen.animals.signature", &[Arg::Key(&format!("screen.animals.signature.{}", s.signature))]);
    });
}

/// A stat's value as it is read: health with a decimal (whole from 100: "369.0" was squeezed in
/// its column for a big animal, Playtest), damage and speed as a percentage of the species' own,
/// the rest whole.
fn shown(a: &Animal, i: usize) -> String {
    match i {
        species::HEALTH if a.stat(i) < 100.0 => format!("{:.1}", a.stat(i)),
        species::HEALTH => format!("{:.0}", a.stat(i)),
        species::DAMAGE | species::SPEED => format!("{:.0}%", a.factor(i) * 100.0),
        _ => format!("{:.0}", a.stat(i)),
    }
}

/// "now / most" for a life bar, without the spaces from four digits ("1991 / 2050" was squeezed in
/// its column for a big animal, Playtest).
fn of(now: f32, most: f32) -> String {
    if most >= 1000.0 { format!("{now:.0}/{most:.0}") } else { format!("{now:.0} / {most:.0}") }
}

/// A window as base's (`mods/base/src/window.rs`, the UI redo): the menu's title on an orange tab
/// on the frame, a close button at the top right, `body` in the panel.
fn window(ui: &mut Ui, s: S, body: impl FnOnce(&mut Ui)) {
    framed(ui, None, s, body);
}

/// The owner's window, its pages as tabs, `page` the open one.
fn window_page(ui: &mut Ui, page: usize, s: S, body: impl FnOnce(&mut Ui)) {
    framed(ui, Some(page), s, body);
}

/// The window: the menu's title on the orange tab (the spyglass view), or the owner's pages as
/// tabs with the open one orange.
fn framed(ui: &mut Ui, page: Option<usize>, s: S, body: impl FnOnce(&mut Ui)) {
    // The tabs stand on the frame: the column's gap tucks their bottom 3 UI px under its top
    // line, an even 12 rows of face round a label's capitals (kit 2, as in base's window.rs).
    ui.column(S::col().stretch().gap(-1.5), |ui| {
        ui.row(S::row().gap(2.0), |ui| {
            ui.row(S::row().w(6.0), |_| {});
            let Some(page) = page else {
                ui.panel("tab_pressed", S::row().pad(2.0).h(13.0).center(), |ui| ui.title());
                return framed_close(ui);
            };
            for (j, name) in PAGES.iter().enumerate() {
                let key = format!("screen.animals.tab.{name}");
                if j == page {
                    ui.panel("tab_pressed", S::row().pad(2.0).h(13.0).center(), |ui| ui.text_styled(S::default().no_wrap(), &key, &[]));
                } else {
                    ui.button(&key, S::default().class("tab").h(13.0).pad(2.0), move |c| {
                        if let Some(id) = c.menu_owner() {
                            c.client().send(&PAGE, &[id, j as u64]);
                        }
                    });
                }
            }
            framed_close(ui);
        });
        ui.panel("panel", s, body);
    });
}

/// The close button at the right of a window's tab row.
fn framed_close(ui: &mut Ui) {
    ui.spacer();
    ui.button(
        "window.base.close",
        S::default()
            .class("close")
            .size(7.0, 7.0)
            .pad(0.5)
            .tooltip("window.base.close_tip"),
        |c| c.client().close_screen(),
    );
    ui.row(S::row().w(4.0), |_| {});
}

/// Tamed points placed on the animal screen and not kept yet, for the animal they were placed on
/// (round 2: see a point before you keep it). This client's only: Keep sends them, Undo drops them.
static PENDING: std::sync::Mutex<(u64, [u8; species::STATS])> = std::sync::Mutex::new((0, [0; species::STATS]));

/// The points waiting for animal `id`: none if they were placed on another, or if there are more
/// than it has to place.
fn pending(id: u64, a: &Animal) -> [u8; species::STATS] {
    let (on, p) = *PENDING.lock().unwrap();
    if on != id || p.iter().map(|&n| n as u32).sum::<u32>() > a.unspent as u32 { [0; species::STATS] } else { p }
}

/// The animal as it would be with the waiting points placed.
fn after(id: u64, a: &Animal) -> Animal {
    let mut b = *a;
    for (i, n) in pending(id, a).into_iter().enumerate() {
        b.tamed[i] += n;
    }
    b
}

/// Wild points' quality, by a stat's wild points: the bar's colour (`stat_q0`..`stat_q4`, grey few,
/// then green, blue, purple, gold). A wild one's level is spread over nine stats, so 20 in one
/// is rare.
const QUALITY: [u8; 4] = [4, 8, 13, 20];

fn quality(points: u8) -> usize {
    QUALITY.iter().filter(|&&q| points >= q).count()
}

/// A stat bar's share: a sixth at no points, then its points (wild and tamed) against 40, a
/// strong one.
fn points_share(a: &Animal, i: usize) -> f32 {
    0.15 + 0.85 * ((a.wild[i] as f32 + a.tamed[i] as f32) / 40.0).min(1.0)
}

/// A bar of the kit (`class`) and its value beside it, right-aligned in a column of its own, so
/// the numbers line up and none is written on a bar's fill (kit 2, the UI guide's D).
fn life_bar(ui: &mut Ui, class: &'static str, value: impl Fn(&ClientView) -> Option<(f32, String)> + Send + Sync + Clone + 'static) {
    let v2 = value.clone();
    ui.progress_bind(class, S::default().size(LIFE_BAR.0, LIFE_BAR.1), move |c| value(c).map_or(-1.0, |v| v.0.clamp(0.0, 1.0)));
    ui.row(S::row().w(LIFE_VALUE_W).end(), move |ui| {
        ui.text_bind_styled(S::default().no_wrap(), move |c, t| match v2(c) {
            Some((_, s)) => t.text(&s),
            None => t.text(""),
        });
    });
}

/// The life bars and stat bars' sizes in GUI pixels (the kit's `life_*` and `stat_q*`), and the
/// columns beside them: an icon, then the bar, then the value (left column `LEFT_W`: 8 + 3 + 52 + 3 + 34).
const LIFE_BAR: (f32, f32) = (52.0, 6.0);
const LIFE_VALUE_W: f32 = 34.0;
/// The Overview's left column (the portrait, name, chips and life), and the window's padding and
/// the gap between its three columns, GUI px: the whole window is 6, 100, 6, 155 (`STAT_ROW_W`), 6,
/// 100 and 6, 379 in all, inside the 380 a 760 px wide window has at GUI scale 2 (Playtest B9c: at 396 it lost 8 px
/// each side).
const LEFT_W: f32 = 100.0;
const OVERVIEW_PAD: f32 = 6.0;
const OVERVIEW_GAP: f32 = 6.0;
/// The genes list (on the Genes page): how many lines it can hold.
const GENE_LINES: usize = 40;
/// The Genes page's list: taller, it has the page to itself.
const GENES_PAGE_H: f32 = 110.0;
/// The stat names' column: as wide as the longest name in either language ("Выносливость" 48 GUI px
/// in the Cyrillic drawn for PixelOperator; at 34 it was shrunk to 80%, the Russian pass); the bars 18 to make room (and the wild column
/// 16, so "Дикий" and "Ручной" over their columns keep 2 GUI px between them), so the window still
/// fits 380 with every column as wide as its words.
const STAT_NAME_W: f32 = 48.0;
const STAT_BAR: (f32, f32) = (18.0, 6.0);
/// The stat rows' columns after the bar: the value (right-aligned), wild and tamed points
/// (centred), then the + (GUI pixels).
/// ("152%" is 16.5 GUI px)
const STAT_VALUE_W: f32 = 17.0;
/// (as wide as their headings, "Wild" 13.5 and "Tamed" 19.5 GUI px: narrower, the row shrank them,
/// PC runs fit1280 and fit760b)
const STAT_WILD_W: f32 = 16.0;
const STAT_TAMED_W: f32 = 20.0;
/// A stat row's whole width, for the divider over torpor.
const STAT_ROW_W: f32 =
    8.0 + 3.0 + STAT_NAME_W + 3.0 + STAT_BAR.0 + 3.0 + STAT_VALUE_W + 3.0 + STAT_WILD_W + 3.0 + STAT_TAMED_W + 3.0 + 10.0;

/// The screen of a tame animal (`notes/ui/BATCHES.md`, the taming and stats upgrade): who it is
/// and how it is on the left, its stats as bars in the middle, its coat and orders on the right.
fn animal_screen(ui: &mut Ui) {
    window_page(ui, 0, S::row().pad(OVERVIEW_PAD).gap(OVERVIEW_GAP), |ui| {
        // Left: the level on a badge, the name, life, how it feels, and the facts.
        ui.column(S::col().gap(3.0).w(LEFT_W), |ui| {
            // The animal itself, turning with the mouse, its level on a badge in the corner.
            ui.panel("portrait", S::row().size(LEFT_W, 70.0), |ui| {
                ui.entity_model(LEFT_W - 2.0, 68.0);
                ui.panel("hud_badge", S::row().size(20.0, 24.0).center().at(2.0, 2.0), |ui| {
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let Some(a) = c.menu_component(&ANIMAL) else { return };
                        t.set("screen.animals.badge", &[Arg::Int(a.level() as i64)]);
                    });
                });
            });
            ui.row(S::row().gap(4.0).center(), |ui| {
                ui.column(S::col().gap(1.0), |ui| {
                    title(ui, false);
                    ui.text_bind_styled(S::default().class("dim").no_wrap(), |c, t| {
                        let Some(a) = c.menu_component(&ANIMAL) else { return };
                        let owner = if tamed(c) { "screen.animals.tamed" } else { "screen.animals.wild" };
                        t.set("screen.animals.about", &[Arg::Key(sex_key(&a)), Arg::Key(&stage_key(&a)), Arg::Key(owner)]);
                    });
                });
            });
            // Its coat, what its kind does and its temperament, as chips, one under another (a row
            // can't wrap, and "Hunts in packs" alone nearly fills the column).
            ui.column(S::col().gap(2.0).max_w(LEFT_W), |ui| {
                ui.panel("chip", S::row().pad(2.0), |ui| {
                    ui.text_bind_styled(S::default().max_w(LEFT_W - 14.0), |c, t| {
                        let Some(look) = c
                            .menu_component(&crate::coats::GENOME)
                            .and_then(|g| crate::coats::phenotype(&g))
                        else {
                            return;
                        };
                        set_pieces(t, &coat_pieces(&look.name));
                    });
                });
                ui.panel("chip", S::row().pad(2.0), |ui| {
                    ui.text_bind_styled(S::default().max_w(LEFT_W - 14.0), |c, t| {
                        let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
                        t.set("screen.animals.chip", &[Arg::Key(&format!("screen.animals.signature.{}", s.signature))]);
                    });
                });
                // Its temperament, an inherited trait: calm to skittish.
                ui.panel("chip", S::row().pad(2.0), |ui| {
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let Some(g) = c.menu_component(&crate::coats::GENOME) else { return };
                        let v = animals_core::coats::temperament(&g);
                        let word = if v < 0.34 {
                            "calm"
                        } else if v < 0.67 {
                            "steady"
                        } else {
                            "skittish"
                        };
                        t.set("screen.animals.temper", &[Arg::Key(&format!("screen.animals.temper.{word}"))]);
                    });
                });
            });
            // Icons, not names, before the life bars (the stat icons; the bond its heart-paw).
            for (label, class) in [("health", "life_health"), ("food", "life_food"), ("water", "life_water"), ("bond", "life_bond")] {
                ui.row(
                    S::row()
                        .gap(3.0)
                        .center()
                        .tooltip(&format!("screen.animals.life.{label}")),
                    move |ui| {
                        let icon = match label {
                            "water" => "base:gui/stat_icon_oxygen".to_string(),
                            "bond" => "base:gui/icon_taming".to_string(),
                            _ => format!("base:gui/stat_icon_{label}"),
                        };
                        ui.image(&icon, 8.0, 8.0);
                        match label {
                            "health" => life_bar(ui, class, |c| {
                                let a = c
                                    .menu_component(&ANIMAL)
                                    .filter(|a| a.species().is_some())?;
                                let e = c.menu_owner().and_then(|id| c.client().entity(id))?;
                                let max = a.stat(species::HEALTH) * crate::comfort::condition(&a);
                                Some((e.health / max.max(0.1), of(e.health, max)))
                            }),
                            "food" => life_bar(ui, class, |c| {
                                let a = c
                                    .menu_component(&ANIMAL)
                                    .filter(|a| a.species().is_some())?;
                                let max = a.stat(species::FOOD).round();
                                Some((a.food as f32 / max.max(1.0), of(a.food as f32, max)))
                            }),
                            "water" => life_bar(ui, class, |c| {
                                let a = c.menu_component(&ANIMAL)?;
                                if a.species()
                                    .is_none_or(|s| animals_core::needs::drinks(s) <= 0.0)
                                {
                                    return None;
                                }
                                let w = 1.0 - a.thirst.clamp(0.0, 1.0);
                                Some((w, format!("{:.0}%", w * 100.0)))
                            }),
                            _ => life_bar(ui, class, |c| {
                                let a = c.menu_component(&ANIMAL)?;
                                Some((a.bond as f32 / 100.0, a.bond.to_string()))
                            }),
                        }
                    },
                );
            }
            // How it feels in this air, against its comfort, and its condition.
            ui.text_bind_styled(S::default().max_w(LEFT_W), |c, t| {
                let (Some(a), Some(b)) = (c.menu_component(&ANIMAL), c.menu_component(&crate::comfort::BODY)) else { return };
                let Some(s) = a.species().filter(|_| b.core > 0.0) else { return };
                let (lo, hi) = s.bio.comfort;
                if s.bio.cold_blooded {
                    t.set(
                        "screen.animals.body_cold",
                        &[Arg::Int(b.air.round() as i64), Arg::Int(b.warmth as i64), Arg::Int(b.condition as i64)],
                    );
                    return;
                }
                let feel = crate::comfort::FEELING_NAMES[(b.feel as usize).min(6)];
                t.set(
                    "screen.animals.body",
                    &[
                        Arg::Key(&format!("screen.animals.feel.{feel}")),
                        Arg::Int(b.air.round() as i64),
                        Arg::Int(lo.round() as i64),
                        Arg::Int(hi.round() as i64),
                        Arg::Int(b.condition as i64),
                    ],
                );
            });
        });
        // Middle: the nine stats as bars, coloured by their wild points, with the points.
        ui.column(S::col().gap(2.0), |ui| {
            ui.row(S::row().gap(4.0).center(), |ui| {
                ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.stats", &[]);
                ui.panel("chip_points", S::row().pad(2.0), |ui| {
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let (Some(a), Some(id)) = (c.menu_component(&ANIMAL), c.menu_owner()) else { return };
                        let waiting: u32 = pending(id, &a).iter().map(|&n| n as u32).sum();
                        t.set(
                            "screen.animals.points_chip",
                            &[
                                Arg::Int(a.unspent as i64 - waiting as i64),
                                Arg::Int(a.tamed_points() as i64 + waiting as i64),
                                Arg::Int(animal::MAX_TAMED_POINTS as i64),
                            ],
                        );
                    });
                });
            });
            // The header in the rows' own columns: name, bar, value, wild, tamed, +. A header
            // lines up as its column's values do (centred over centred counts).
            // The columns' headings, each centred over its column and free to reach past it into
            // the empty heading room beside it: in a row the longer Russian ones ("Дикий" 17.5,
            // "Ручной" 20.5 GUI px) were shrunk to the columns' 14 and 20.
            ui.row(S::row().size(STAT_ROW_W, 8.5), |ui| {
                let wild = 8.0 + 3.0 + STAT_NAME_W + 3.0 + STAT_BAR.0 + 3.0 + STAT_VALUE_W + 3.0;
                let tamed = wild + STAT_WILD_W + 3.0;
                for (x, w, key) in [(wild, STAT_WILD_W, "screen.animals.wild_points"), (tamed, STAT_TAMED_W, "screen.animals.tamed_points")]
                {
                    ui.text_styled(
                        S::default()
                            .class("dim")
                            .no_wrap()
                            .at_pivot(x + w / 2.0, 0.0, [0.5, 0.0]),
                        key,
                        &[],
                    );
                }
            });
            for (i, name) in STAT_NAMES.iter().enumerate() {
                if i == species::TORPOR {
                    ui.image("base:gui/divider", STAT_ROW_W, 1.0);
                }
                ui.row(S::row().gap(3.0).center(), |ui| {
                    ui.image(&format!("base:gui/stat_icon_{name}"), 8.0, 8.0);
                    ui.text_styled(S::default().w(STAT_NAME_W).no_wrap(), &format!("stat.animals.{name}"), &[]);
                    ui.row(S::row().size(STAT_BAR.0, STAT_BAR.1), move |ui| {
                        // One bar per quality on the same place: the one matching its wild points shows.
                        for q in 0..5usize {
                            ui.progress_bind(
                                ["stat_q0", "stat_q1", "stat_q2", "stat_q3", "stat_q4"][q],
                                S::default().size(STAT_BAR.0, STAT_BAR.1).at(0.0, 0.0),
                                move |c| match c.menu_component(&ANIMAL) {
                                    Some(a) if quality(a.wild[i]) == q => points_share(&a, i),
                                    _ => -1.0,
                                },
                            );
                        }
                    });
                    ui.row(S::row().w(STAT_VALUE_W).end(), move |ui| {
                        ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                            let (Some(a), Some(id)) = (c.menu_component(&ANIMAL), c.menu_owner()) else { return };
                            if pending(id, &a)[i] > 0 {
                                t.set("screen.animals.after", &[Arg::Text(&shown(&after(id, &a), i))]);
                            } else {
                                t.text(&shown(&a, i));
                            }
                        });
                    });
                    ui.row(S::row().w(STAT_WILD_W).center(), move |ui| {
                        ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                            if let Some(a) = c.menu_component(&ANIMAL) {
                                t.set(&format!("screen.animals.wild_q{}", quality(a.wild[i])), &[Arg::Int(a.wild[i] as i64)]);
                            }
                        });
                    });
                    ui.row(S::row().w(STAT_TAMED_W).center(), move |ui| {
                        ui.text_bind_styled(S::default().no_wrap(), move |c, t| {
                            let (Some(a), Some(id)) = (c.menu_component(&ANIMAL), c.menu_owner()) else { return };
                            // Torpor takes no points: a dash, not a 0 that looks placeable.
                            if i == species::TORPOR {
                                return t.text("-");
                            }
                            match pending(id, &a)[i] {
                                0 => t.text(&a.tamed[i].to_string()),
                                n => t.set("screen.animals.after", &[Arg::Text(&(a.tamed[i] + n).to_string())]),
                            }
                        });
                    });
                    if i != species::TORPOR {
                        ui.button(
                            "screen.animals.plus",
                            S::default()
                                .size(10.0, 9.0)
                                .pad(0.5)
                                .tooltip("screen.animals.place"),
                            move |c| {
                                let (Some(a), Some(id)) = (c.menu_component(&ANIMAL), c.menu_owner()) else { return };
                                let waiting: u32 = pending(id, &a).iter().map(|&n| n as u32).sum();
                                if waiting < a.unspent as u32 {
                                    let mut p = PENDING.lock().unwrap();
                                    if p.0 != id {
                                        *p = (id, [0; species::STATS]);
                                    }
                                    p.1[i] += 1;
                                }
                            },
                        );
                    } else {
                        ui.row(S::row().w(10.0), |_| {});
                    }
                });
            }
            // Keep sends the waiting points (one place each, as the + used to); Undo drops them.
            ui.row(S::row().gap(3.0), |ui| {
                ui.button("screen.animals.keep", S::default().min_w(60.0).h(11.0), |c| {
                    let (Some(a), Some(id)) = (c.menu_component(&ANIMAL), c.menu_owner()) else { return };
                    let p = pending(id, &a);
                    *PENDING.lock().unwrap() = (0, [0; species::STATS]);
                    for (i, n) in p.into_iter().enumerate() {
                        for _ in 0..n {
                            c.client().send(&PLACE, &[id, i as u64]);
                        }
                    }
                });
                ui.button("screen.animals.undo", S::default().class("button_secondary").min_w(44.0).h(11.0), |_| {
                    *PENDING.lock().unwrap() = (0, [0; species::STATS]);
                });
            });
            ui.text_styled(S::default().class("dim").max_w(STAT_ROW_W), "screen.animals.after_legend", &[]);
            ui.text_styled(S::default().class("dim").max_w(STAT_ROW_W), "screen.animals.quality_legend", &[]);
        });
        // Right: its coat and what its kind does, then its orders.
        ui.column(S::col().gap(3.0).w(100.0), |ui| {
            ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.orders_title", &[]);
            ui.text_bind_styled(S::default().max_w(100.0), |c, t| {
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
            for pair in COMMAND_NAMES.chunks(2).enumerate() {
                let (row, names) = pair;
                ui.row(S::row().gap(2.0), move |ui| {
                    for (j, name) in names.iter().enumerate() {
                        order_button_w(ui, &format!("screen.animals.button.{name}"), (row * 2 + j) as u64, 49.0);
                    }
                });
            }
            ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.stance_title", &[]);
            for pair in STANCE_NAMES.chunks(2).enumerate() {
                let (row, names) = pair;
                ui.row(S::row().gap(2.0), move |ui| {
                    for (j, name) in names.iter().enumerate() {
                        order_button_w(ui, &format!("screen.animals.button.{name}"), 10 + (row * 2 + j) as u64, 49.0);
                    }
                });
            }
            ui.row(S::row().gap(2.0), |ui| {
                // Widths for the longest label in either language with 2 GUI px each side
                // ("Sniff out" / "Взять след", "Release" / "Отпустить"); equal thirds put "Sniff
                // out" against its edges (the label check, 11-04).
                order_button_w(ui, "screen.animals.share", 20, 22.0);
                order_button_w(ui, "screen.animals.track", 30, 39.0);
                order_button_w(ui, "screen.animals.release", 40, 35.0);
            });
            ui.text_styled(S::default().class("dim").max_w(100.0), "screen.animals.whistles", &[]);
        });
    });
}

/// The Genes page: its coat and kind as chips, every gene on a line of its own, then its kind.
fn genes_screen(ui: &mut Ui) {
    window_page(ui, 1, S::col().pad(8.0).gap(4.0).w(230.0), |ui| {
        ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.coat_title", &[]);
        ui.column(S::col().gap(2.0).max_w(214.0), |ui| {
            ui.panel("chip", S::row().pad(2.0), |ui| {
                ui.text_bind_styled(S::default().max_w(94.0), |c, t| {
                    let Some(look) = c
                        .menu_component(&crate::coats::GENOME)
                        .and_then(|g| crate::coats::phenotype(&g))
                    else {
                        return;
                    };
                    set_pieces(t, &coat_pieces(&look.name));
                });
            });
            ui.panel("chip", S::row().pad(2.0), |ui| {
                ui.text_bind_styled(S::default().max_w(94.0), |c, t| {
                    let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
                    t.set("screen.animals.chip", &[Arg::Key(&format!("screen.animals.signature.{}", s.signature))]);
                });
            });
            // Its temperament, an inherited trait: calm to skittish.
            ui.panel("chip", S::row().pad(2.0), |ui| {
                ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                    let Some(g) = c.menu_component(&crate::coats::GENOME) else { return };
                    let v = animals_core::coats::temperament(&g);
                    let word = if v < 0.34 {
                        "calm"
                    } else if v < 0.67 {
                        "steady"
                    } else {
                        "skittish"
                    };
                    t.set("screen.animals.temper", &[Arg::Key(&format!("screen.animals.temper.{word}"))]);
                });
            });
        });
        ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.genes_title", &[]);
        // Its genes, one line each: its kind's own first, then the core traits.
        ui.panel("well", S::col().pad(3.0).gap(1.0).w(214.0).h(GENES_PAGE_H).scroll(), |ui| {
            for i in 0..GENE_LINES {
                ui.text_bind_styled(S::default().max_w(206.0), move |c, t| {
                    let Some(g) = c.menu_component(&crate::coats::GENOME) else { return t.text("") };
                    let Some(l) = animals_core::coats::genes_of(&g).into_iter().nth(i) else { return t.text("") };
                    // Its name in the player's language (`gene.<name>`; the gene files name it in
                    // English); the alleles as they are written.
                    let name = gene_key(&l.name);
                    match l.value {
                        Some(v) => t.set("screen.animals.gene_value", &[Arg::Key(&name), Arg::Int((v * 100.0).round() as i64)]),
                        None => {
                            let pair = l.alleles.join("/");
                            t.set("screen.animals.gene_alleles", &[Arg::Key(&name), Arg::Text(&l.shows), Arg::Text(&pair)]);
                        }
                    }
                });
            }
        });
        ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.kind_title", &[]);
        ui.column(S::col().gap(1.0).max_w(214.0), |ui| {
            taxon_line(ui);
            height_line(ui);
            signature_line(ui);
        });
    });
}

/// The Family page: who it is, how old, when its kind breeds and how it stands now, its parents.
fn family_screen(ui: &mut Ui) {
    window_page(ui, 2, S::col().pad(8.0).gap(4.0).w(230.0), |ui| {
        title(ui, false);
        ui.column(S::col().gap(2.0).max_w(214.0), |ui| {
            about_line(ui);
            age_line(ui);
            ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.breeding_title", &[]);
            breeds_line(ui);
            breeding_line(ui);
            ui.text_styled(S::default().class("title").no_wrap(), "screen.animals.family_title", &[]);
            family_line(ui);
        });
    });
}

/// An order button of a fixed width, so the grid of orders lines up.
fn order_button_w(ui: &mut Ui, key: &str, order: u64, w: f32) {
    ui.button(key, S::default().size(w, 11.0), move |c| {
        if let Some(id) = c.menu_owner() {
            c.client().send(&ORDER, &[id, order]);
        }
    });
}

fn scout_screen(ui: &mut Ui) {
    window(ui, S::col().pad(8.0).gap(3.0).min_w(180.0), |ui| {
        heading(ui);
        ui.text_bind(|c, t| {
            let Some(s) = c.menu_component(&ANIMAL).and_then(|a| a.species()) else { return };
            let size = match s.size {
                Size::Small => "screen.animals.size.small",
                Size::Medium => "screen.animals.size.medium",
                Size::Large => "screen.animals.size.large",
            };
            let diet = match s.diet() {
                Diet::Carnivore => "screen.animals.diet.carnivore",
                Diet::Omnivore => "screen.animals.diet.omnivore",
                Diet::Herbivore => "screen.animals.diet.herbivore",
            };
            let taming = match s.taming {
                Taming::Passive => "screen.animals.taming.passive",
                Taming::Knockout => "screen.animals.taming.knockout",
                Taming::Wild => "screen.animals.taming.wild",
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
        // The land round the player by biome, and how much of it each species lives on
        // (`/animal habitats [radius]`), from the world's generator: generated or not.
        Some("habitats") => {
            let Some(p) = w.entity(player) else { return };
            let radius = cmd.int("count").unwrap_or(2048).clamp(64, 16_384) as i32;
            for line in crate::habitats::survey(&w, [p.pos[0] as i32, p.pos[2] as i32], radius) {
                weft_sdk::info!("habitats: {line}");
                cmd.reply("command.animals.line", &[Arg::Text(&line)]);
            }
            return;
        }
        // Whether natural spawning would bring a group of the nearest animal's kind (`count`, its
        // biggest group by default) to where the player stands (`forage::kind_near`,
        // `life::room_for`): how a full place turns them away, without waiting on the spawner.
        Some("room") => {
            let Some(p) = w.entity(player) else { return };
            let Some(s) = nearest(&w, player, 64.0)
                .and_then(|id| w.entity(id))
                .and_then(|e| species::SPECIES.iter().find(|s| s.kind.id() == e.kind))
            else {
                return;
            };
            let at = p.pos.map(|v| v.floor() as i32);
            let coming = cmd
                .int("count")
                .map_or(s.bio.social.group().1, |n| n.max(1) as u32);
            let near = crate::forage::kind_near(&w, s, at);
            let comes = animals_core::life::room_for(near, coming, s.crowd());
            let line = format!(
                "room: {} at {at:?}: {near:.1} near, room for {}: a group of {coming} {}",
                s.name,
                s.crowd(),
                if comes { "would come" } else { "is turned away" }
            );
            weft_sdk::info!("{line}");
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
    // Every action on one goes by its species and stats ("wild" resets them on purpose).
    if cmd.text("action") != Some("wild") {
        animal::ensure_rolled(&w, id);
    }
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
        // Its food down to `count` percent (testing foraging: `forage.rs`).
        "hungry" => {
            let share = cmd.int("count").unwrap_or(20).clamp(0, 100) as f32 / 100.0;
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.food = (a.stat(species::FOOD) * share).round() as u16;
            }
        }
        // Its water down to `count` percent (testing drinking: `drink.rs`).
        "thirsty" => {
            let share = cmd.int("count").unwrap_or(20).clamp(0, 100) as f32 / 100.0;
            if let Some(a) = w.component_mut(id, &ANIMAL) {
                a.thirst = 1.0 - share;
            }
        }
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

// ------------------------------------------------------------------ the taming plate

/// How far the taming plate looks for an animal being tamed.
const TAMING_RANGE: f64 = 12.0;
/// The plate's size in GUI pixels.
const PLATE: (f32, f32) = (150.0, 46.0);

/// The nearest animal within reach that the player is taming (fed at least once, not tame yet)
/// or has knocked out, with its taming and knock-out (synced to whoever sees it).
fn being_tamed(c: &ClientView) -> Option<(u64, crate::taming::Taming, Option<crate::knockout::Knockout>)> {
    let me = c.me()?;
    let client = c.client();
    client
        .entities()
        .into_iter()
        .filter_map(|id| {
            let e = client.entity(id)?;
            let d2: f64 = (0..3).map(|k| (e.pos[k] - me.pos[k]).powi(2)).sum();
            if d2 > TAMING_RANGE * TAMING_RANGE {
                return None;
            }
            // (a knocked-out one may have no taming yet: it starts at its first feeding)
            let k = c.component_of(id, &KNOCKOUT).filter(|k| k.out > 0);
            let g = c.component_of(id, &TAMING);
            let taming = g.is_some_and(|g| g.feedings > 0 && g.progress < 1.0);
            (taming || k.is_some()).then(|| (d2, id, g.unwrap_or(crate::taming::FRESH), k))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id, g, k)| (id, g, k))
}

/// Its species, from its entity kind.
fn species_of(c: &ClientView, id: u64) -> Option<&'static species::Species> {
    let kind = c.client().entity(id)?.kind;
    species::SPECIES.iter().find(|s| s.kind.id() == kind)
}

/// While taming, a plate under the crosshair (ARK's taming bar, in our parts): whom, how far
/// (the bar and its %), how clean (effectiveness), and what it asks for or, knocked out, how deep
/// it sleeps and what is in its mouth.
fn taming_hud(r: &mut Registry) {
    r.hud_element(&TAMING_HUD, hud_anchor::UNDER_CROSSHAIR, 1, |ui| {
        ui.row(S::row().size(PLATE.0, PLATE.1), |ui| {
            ui.progress_bind("hud_plate", S::default().size(PLATE.0, PLATE.1).at(0.0, 0.0), |c| {
                if being_tamed(c).is_some() { 1.0 } else { -1.0 }
            });
            ui.column(S::col().gap(1.0).at(5.0, 3.0), |ui| {
                ui.text_bind_styled(S::default().class("title").no_wrap(), |c, t| {
                    let Some(s) = being_tamed(c).and_then(|(id, _, _)| species_of(c, id)) else { return t.text("") };
                    t.set("hud.animals.taming.who", &[Arg::Key(&format!("entity.animals.{}", s.kind_name()))]);
                });
                ui.row(S::row().gap(3.0).center(), |ui| {
                    // as wide as "Приручено" (35 GUI px; narrower it was shrunk, the Russian pass)
                    ui.text_bind_styled(S::default().w(36.0).no_wrap(), |c, t| {
                        if being_tamed(c).is_some() { t.set("hud.animals.taming.bar", &[]) } else { t.text("") }
                    });
                    ui.progress_bind("life_taming", S::default().size(LIFE_BAR.0, LIFE_BAR.1), |c| {
                        being_tamed(c).map_or(-1.0, |(_, g, _)| g.progress.clamp(0.0, 1.0))
                    });
                    ui.text_bind_styled(S::default().no_wrap(), |c, t| {
                        let Some((_, g, _)) = being_tamed(c) else { return t.text("") };
                        t.set("hud.animals.taming.percent", &[Arg::Int((g.progress * 100.0).floor() as i64)]);
                    });
                });
                ui.text_bind_styled(S::default().class("dim").no_wrap(), |c, t| {
                    let Some((id, g, k)) = being_tamed(c) else { return t.text("") };
                    let eff = Arg::Int((g.effectiveness * 100.0).round() as i64);
                    match k {
                        Some(k) if k.count > 0 => t.set(
                            "hud.animals.taming.asleep_food",
                            &[eff, Arg::Int(k.torpor as i64), Arg::Item(k.food), Arg::Int(k.count as i64)],
                        ),
                        Some(k) => t.set("hud.animals.taming.asleep", &[eff, Arg::Int(k.torpor as i64)]),
                        None => match species_of(c, id).and_then(|s| s.foods.get((g.want as usize).wrapping_sub(1))) {
                            Some(f) if g.asking > 0 => t.set("hud.animals.taming.asks", &[eff, Arg::Name(&format!("item:{}", f.item))]),
                            _ => t.set("hud.animals.taming.waits", &[eff, Arg::Int(g.wait.div_ceil(20) as i64)]),
                        },
                    }
                });
            });
        });
    });
}

#[cfg(test)]
mod coat_name_tests {
    use super::*;

    fn keys(lang: &str) -> std::collections::HashSet<String> {
        lang.lines()
            .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.trim().to_string()))
            .collect()
    }

    /// Every word and phrase of every coat a species can have has its translation in English and
    /// Russian (a missing one would show its key on the animal screen).
    #[test]
    fn every_coat_word_has_a_translation() {
        let (en, ru) = (keys(include_str!("../assets/lang/en_us.lang")), keys(include_str!("../assets/lang/ru_ru.lang")));
        let mut seed = 12345u32;
        let mut rng = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let mut missing = std::collections::BTreeSet::new();
        for index in 0..crate::species::SPECIES.len() {
            for habitat in ["", "grassland", "forest", "taiga", "snow", "desert", "mountains", "swamp"] {
                let Some(pool) = crate::coats::wild_pool(index, habitat) else { continue };
                for n in 0..200 {
                    let Some(g) = crate::coats::sample(index, n % 2 == 0, &pool, &mut rng) else { continue };
                    let Some(look) = crate::coats::phenotype(&g) else { continue };
                    for p in coat_pieces(&look.name) {
                        if let Piece::Key(k) = p {
                            if !en.contains(&k) || !ru.contains(&k) {
                                missing.insert(format!("{k} (in \"{}\")", look.name));
                            }
                        }
                    }
                }
            }
        }
        assert!(missing.is_empty(), "coat words without a translation: {missing:?}");
        assert!(en.contains(&format!("screen.animals.join_{JOIN_MAX}")) && ru.contains(&format!("screen.animals.join_{JOIN_MAX}")));
    }
}

#[cfg(test)]
mod gene_name_tests {
    use super::*;

    /// Every gene line every species' Genes page shows has its name translated in English and
    /// Russian (its own loci and the core traits).
    #[test]
    fn every_gene_name_has_a_translation() {
        let keys = |lang: &str| -> std::collections::HashSet<String> {
            lang.lines()
                .filter_map(|l| l.split_once(" = ").map(|(k, _)| k.trim().to_string()))
                .collect()
        };
        let (en, ru) = (keys(include_str!("../assets/lang/en_us.lang")), keys(include_str!("../assets/lang/ru_ru.lang")));
        let mut seed = 777u32;
        let mut rng = move || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (seed >> 8) as f32 / (1u32 << 24) as f32
        };
        let mut missing = std::collections::BTreeSet::new();
        for index in 0..crate::species::SPECIES.len() {
            let Some(pool) = crate::coats::wild_pool(index, "") else { continue };
            let Some(g) = crate::coats::sample(index, true, &pool, &mut rng) else { continue };
            for l in animals_core::coats::genes_of(&g) {
                let k = gene_key(&l.name);
                if !en.contains(&k) || !ru.contains(&k) {
                    missing.insert(k);
                }
            }
        }
        assert!(missing.is_empty(), "gene names without a translation: {missing:?}");
    }
}
