//! The recipe book beside the crafting grids (`notes/CRAFTING.md`, slice 1): every crafting recipe
//! in pages by creative tab, the ones you can make first. Picking one shows what it takes and
//! what you have. Craft makes it from your bag, and at a crafting table also from every chest
//! within `CHEST_REACH` blocks of the table; the grid keeps working by hand.
//!
//! The client builds the book from its own state (`BOOK`) and rebuilds the screen when that
//! changes. What the chests hold comes from the server (`POOL`), since the client doesn't see
//! inside them. Crafting is the server's (`CRAFT`): it finds the table, takes the ingredients and
//! gives the result.

use std::sync::Mutex;

use weft_sdk::*;

use crate::recipes::CRAFTING;

/// Blocks from the crafting table whose chests it takes from (the same reach as comfort).
pub const CHEST_REACH: i32 = 7;
/// How near the player a crafting table must be for its recipes and chests.
const TABLE_REACH: i32 = 5;
/// Recipes on a page of the book: 5 columns of 4.
const COLS: usize = 5;
const PER_PAGE: usize = 20;
/// The most one Craft makes ("all").
const MAX_BATCH: u32 = 64;

/// Where the book is: the inventory's 2 x 2 (hands, the bag only) or a crafting table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Station {
    Hands = 0,
    Table = 1,
}

impl Station {
    fn from(v: u64) -> Station {
        if v == 1 { Station::Table } else { Station::Hands }
    }
}

/// Craft: [recipe, how many, station].
static CRAFT: Channel<[u64; 3]> = Channel::named("craft_from_book");
/// The client asks what the chests around its table hold: [station].
static POOL_ASK: Channel<[u64; 1]> = Channel::named("crafting_pool_ask");
/// What the chests hold, as (item, count) pairs of little-endian u32s.
static POOL: Channel<Vec<u8>> = Channel::named("crafting_pool");

/// The player's bag, hotbar first: base's groups of the player's inventory (`player.rs`).
const BAG_GROUPS: [&str; 2] = ["hotbar", "main"];

static CHESTS: Tag = Tag::named("chests");
static TABLES: Tag = Tag::named("crafting_tables");

/// The book's tabs: all, then base's creative tabs.
const TABS: [&str; 8] = ["", "base:tools", "base:combat", "base:food", "base:building", "base:functional", "base:ingredients", PARTS_TAB];
/// The tab of part recipes (at a table).
const PARTS_TAB: &str = "parts";
/// Book numbers of part recipes start here; below are crafting recipes'.
const PART_IDS: u32 = 1 << 20;

/// The recipe with book number `id`.
fn recipe_of(id: u32) -> Option<Recipe> {
    if id >= PART_IDS { Recipe::at(&crate::parts::PART_RECIPES, id - PART_IDS) } else { Recipe::at(&CRAFTING, id) }
}

/// A recipe's book number.
fn id_of(r: &Recipe) -> u32 {
    if r.kind() == crate::parts::PART_RECIPES.name() { PART_IDS + r.index() } else { r.index() }
}

/// The client's book: which tab, page and recipe, and the chests' contents last sent.
struct Book {
    tab: usize,
    page: usize,
    picked: Option<u32>,
    pool: Vec<(u32, u32)>,
    /// The menu window the pool was asked for.
    asked: u32,
}

/// The bag as the open book last saw it (`client_frame` above).
static BAG_SIG: Mutex<u64> = Mutex::new(0);

static BOOK: Mutex<Book> = Mutex::new(Book { tab: 0, page: 0, picked: None, pool: Vec::new(), asked: u32::MAX });

pub fn register(r: &mut Registry) {
    r.tag("chests", &["chest"]);
    r.packet(&CRAFT, |p, v| {
        let w = p.world();
        let player = p.player();
        let station = Station::from(v[2]);
        if let Some(recipe) = recipe_of(v[0] as u32) {
            craft(&w, player, station, &recipe, (v[1] as u32).clamp(1, MAX_BATCH));
        }
        p.reply(&POOL, &encode(&chest_totals(&w, player, station)));
    });
    r.packet(&POOL_ASK, |p, v| {
        let w = p.world();
        let station = Station::from(v[0]);
        let pool = chest_totals(&w, p.player(), station);
        // What the book will count besides the bag (asked when a crafting screen opens).
        if station == Station::Table {
            let table = table_near(&w, p.player());
            let chests = table.map_or(0, |t| chests_near(&w, t).len());
            weft_sdk::info!(
                "crafting: book at {station:?}: table {table:?}, {chests} chests within {CHEST_REACH}, {} kinds of item in them",
                pool.len()
            );
        }
        p.reply(&POOL, &encode(&pool));
    });
    // The book is built when its screen opens; the bag can change after (an item given, moved,
    // picked up), so it's built again when it does.
    r.client_frame(|c| {
        let open = BOOK.lock().map_or(u32::MAX, |b| b.asked);
        let window = c.menu().map_or(u32::MAX, |m| m[1]);
        if open == u32::MAX || window != open {
            return;
        }
        let v = c.view();
        let mut stacks = v.inventory_group(BAG_GROUPS[0]);
        stacks.extend(v.inventory_group(BAG_GROUPS[1]));
        let sig = stacks
            .iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |h, s| (h ^ ((s.item as u64) << 32 | s.count as u64)).wrapping_mul(0x100_0000_01b3));
        let changed = BAG_SIG
            .lock()
            .map(|mut last| std::mem::replace(&mut *last, sig) != sig)
            .unwrap_or(false);
        if changed {
            c.rebuild_screen();
        }
    });
    r.client_packet(&POOL, |c, bytes| {
        if let Ok(mut b) = BOOK.lock() {
            b.pool = decode(&bytes);
        }
        c.rebuild_screen();
    });
}

// ---------------------------------------------------------------- what a recipe takes

/// The recipes the book shows at `station`: crafting recipes not hidden, small enough for the
/// 2 x 2 by hand.
fn shown(station: Station) -> Vec<Recipe> {
    let parts = if station == Station::Table { recipes(&crate::parts::PART_RECIPES) } else { Vec::new() };
    recipes(&CRAFTING)
        .into_iter()
        .filter(|r| !r.hidden() && (station == Station::Table || fits_hands(r)))
        .chain(parts)
        .collect()
}

/// Fits the inventory's 2 x 2: a shape at most 2 wide and 2 high, or 4 ingredients.
fn fits_hands(r: &Recipe) -> bool {
    match r.shape() {
        Some((w, h)) => w <= 2 && h <= 2,
        None => r.inputs().len() <= 4,
    }
}

/// The inputs that take something, as (input number, how many).
fn needs(r: &Recipe) -> Vec<(u32, u32)> {
    r.inputs()
        .iter()
        .enumerate()
        .filter(|(_, i)| i.kind != abi::recipe_input_kind::EMPTY)
        .map(|(n, i)| (n as u32, i.count.max(1)))
        .collect()
}

/// How many times `r` can be made from `have` (item, count), up to `cap`: each input takes from
/// the first item that fits it with enough left.
fn times(r: &Recipe, have: &[(u32, u32)], cap: u32) -> u32 {
    let needs = needs(r);
    if needs.is_empty() {
        return 0;
    }
    let mut left: Vec<(u32, u32)> = have.to_vec();
    let mut made = 0;
    'outer: while made < cap {
        let mut take: Vec<(usize, u32)> = Vec::new();
        for &(input, n) in &needs {
            let free = |i: usize, left: &[(u32, u32)], take: &[(usize, u32)]| {
                left[i].1 - take.iter().filter(|t| t.0 == i).map(|t| t.1).sum::<u32>()
            };
            match (0..left.len()).find(|&i| free(i, &left, &take) >= n && r.accepts(input, left[i].0)) {
                Some(i) => take.push((i, n)),
                None => break 'outer,
            }
        }
        for (i, n) in take {
            left[i].1 -= n;
        }
        made += 1;
    }
    made
}

/// Totals by item.
fn totals(stacks: impl IntoIterator<Item = ItemStack>) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for s in stacks.into_iter().filter(|s| !s.is_empty()) {
        match out.iter_mut().find(|(i, _)| *i == s.item) {
            Some(t) => t.1 += s.count,
            None => out.push((s.item, s.count)),
        }
    }
    out
}

fn encode(pool: &[(u32, u32)]) -> Vec<u8> {
    pool.iter()
        .flat_map(|(i, n)| i.to_le_bytes().into_iter().chain(n.to_le_bytes()))
        .collect()
}

fn decode(bytes: &[u8]) -> Vec<(u32, u32)> {
    bytes
        .chunks_exact(8)
        .map(|c| (u32::from_le_bytes([c[0], c[1], c[2], c[3]]), u32::from_le_bytes([c[4], c[5], c[6], c[7]])))
        .collect()
}

// ---------------------------------------------------------------- the server: crafting

/// A slot something can be taken from: the player's bag, then the chests.
struct Source {
    owner: u64,
    slot: u32,
}

/// The crafting table nearest `player` within `TABLE_REACH`, if any.
fn table_near(w: &World, player: u64) -> Option<[i32; 3]> {
    let p = w.entity(player)?.pos.map(|v| v.floor() as i32);
    let mut best: Option<([i32; 3], i32)> = None;
    for dx in -TABLE_REACH..=TABLE_REACH {
        for dy in -TABLE_REACH..=TABLE_REACH {
            for dz in -TABLE_REACH..=TABLE_REACH {
                let at = [p[0] + dx, p[1] + dy, p[2] + dz];
                let d = dx * dx + dy * dy + dz * dz;
                if TABLES.on_block(w.get(at)) && best.is_none_or(|b| d < b.1) {
                    best = Some((at, d));
                }
            }
        }
    }
    best.map(|b| b.0)
}

/// The chests within `CHEST_REACH` of `table`, as their entities.
fn chests_near(w: &World, table: [i32; 3]) -> Vec<u64> {
    let r = CHEST_REACH;
    let mut out = Vec::new();
    for dx in -r..=r {
        for dy in -r..=r {
            for dz in -r..=r {
                let at = [table[0] + dx, table[1] + dy, table[2] + dz];
                if CHESTS.on_block(w.get(at)) {
                    let e = w.block_entity(at);
                    if e != 0 {
                        out.push(e);
                    }
                }
            }
        }
    }
    out
}

/// The slots crafting at `station` takes from, bag first. `None` at a table with no table near.
fn sources(w: &World, player: u64, station: Station) -> Option<Vec<Source>> {
    let mut out = Vec::new();
    // The player's own groups, as base names them (`player.rs`); `core:hotbar` and
    // `core:inventory` are only what a menu calls them.
    for group in [BAG_GROUPS[0], BAG_GROUPS[1]] {
        if let Some(g) = w.inventory(player).find_group(group) {
            out.extend((0..g.len()).map(|i| Source { owner: player, slot: g.slot(i) }));
        }
    }
    if station == Station::Table {
        let table = table_near(w, player)?;
        for chest in chests_near(w, table) {
            if let Some(g) = w.inventory(chest).find_group("chest") {
                out.extend((0..g.len()).map(|i| Source { owner: chest, slot: g.slot(i) }));
            }
        }
    }
    Some(out)
}

/// What the chests around the player's table hold (nothing by hand).
fn chest_totals(w: &World, player: u64, station: Station) -> Vec<(u32, u32)> {
    if station != Station::Table {
        return Vec::new();
    }
    let Some(table) = table_near(w, player) else { return Vec::new() };
    totals(chests_near(w, table).into_iter().flat_map(|chest| {
        let inv = w.inventory(chest);
        let g = inv.find_group("chest");
        let n = g.as_ref().map_or(0, |g| g.len());
        (0..n).map(move |i| g.as_ref().map_or(ItemStack::EMPTY, |g| g.get(i)))
    }))
}

/// Makes `r` up to `count` times at `station` from the player's bag (and the chests by the
/// table), giving each result to the bag or dropping it at the player's feet.
fn craft(w: &World, player: u64, station: Station, r: &Recipe, count: u32) -> u32 {
    if station == Station::Hands && !fits_hands(r) {
        return 0;
    }
    let Some(src) = sources(w, player, station) else { return 0 };
    let needs = needs(r);
    if needs.is_empty() {
        return 0;
    }
    let pos = w.entity(player).map_or([0.0; 3], |e| e.pos);
    let mut made = 0;
    let mut from_chests = 0;
    while made < count {
        // Find every input first, so nothing is taken unless all of them are there.
        let mut take: Vec<(usize, u32)> = Vec::new();
        let mut found = true;
        for &(input, n) in &needs {
            let pick = (0..src.len()).find(|&i| {
                let s = w.inventory(src[i].owner).get(src[i].slot);
                let taken: u32 = take.iter().filter(|t| t.0 == i).map(|t| t.1).sum();
                !s.is_empty() && s.count >= taken + n && r.accepts(input, s.item)
            });
            match pick {
                Some(i) => take.push((i, n)),
                None => {
                    found = false;
                    break;
                }
            }
        }
        if !found {
            break;
        }
        for &(i, n) in &take {
            if src[i].owner != player {
                from_chests += n;
            }
            let inv = w.inventory(src[i].owner);
            let mut s = inv.get(src[i].slot);
            for _ in 0..n {
                s = use_one(s);
            }
            inv.set(src[i].slot, s);
        }
        for (out, chance) in r.outputs() {
            if out.is_empty() || chance < 1.0 && roll(player, made) >= chance {
                continue;
            }
            // A part gets its material and a rolled grade (`parts.rs`).
            let out = if r.kind() == crate::parts::PART_RECIPES.name() {
                crate::parts::finish(w, player, r, out, (w.random() >> 40) as f32 / (1u64 << 24) as f32)
            } else {
                out
            };
            let left = w.inventory(player).insert(out);
            if !left.is_empty() {
                w.spawn_item(pos, left);
            }
        }
        base_api::give_xp(w, player, vitals::levels::xp::craft(needs.len() as u32));
        made += 1;
    }
    if let Some((out, _)) = r.outputs().into_iter().next() {
        weft_sdk::info!(
            "crafting: made {made} x {} (asked {count}) at {station:?} from the book, {from_chests} ingredients from chests",
            item_name(out.item)
        );
    }
    made
}

/// A roll in 0..1 for a chance result, different for each craft.
fn roll(player: u64, n: u32) -> f32 {
    let mut x = player ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
    x ^= x >> 33;
    x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    x ^= x >> 33;
    (x >> 40) as f32 / (1u64 << 24) as f32
}

// ---------------------------------------------------------------- the client: the book

/// What the player has to craft with here: the bag, and at a table the chests' totals.
fn have(ui: &Ui, station: Station, pool: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let v = ui.client();
    let mut stacks = v.inventory_group(BAG_GROUPS[0]);
    stacks.extend(v.inventory_group(BAG_GROUPS[1]));
    let mut out = totals(stacks);
    if station == Station::Table {
        for &(item, n) in pool {
            match out.iter_mut().find(|(i, _)| *i == item) {
                Some(t) => t.1 += n,
                None => out.push((item, n)),
            }
        }
    }
    out
}

fn edit(f: impl FnOnce(&mut Book)) {
    if let Ok(mut b) = BOOK.lock() {
        f(&mut b);
    }
}

/// The book: tabs, a page of recipes (what you can make first), the picked one's ingredients
/// and the Craft buttons.
pub fn book(ui: &mut Ui, station: Station) {
    let client = ui.client().client();
    let window = client.menu().map_or(0, |m| m[1]);
    let (tab, page, picked, pool) = {
        let Ok(mut b) = BOOK.lock() else { return };
        if b.asked != window {
            // A new menu: ask for its chests, start on the first page.
            b.asked = window;
            b.pool.clear();
            b.page = 0;
            b.picked = None;
            client.send(&POOL_ASK, &[station as u64]);
        }
        (b.tab, b.page, b.picked, b.pool.clone())
    };
    let have = have(ui, station, &pool);
    // What the player has ever held (`discovery.rs`), and what they hold now; creative knows all.
    let creative = ui.client().creative();
    let mut touched = if creative { Vec::new() } else { crate::discovery::held_items(&ui.client()) };
    for &(item, _) in &have {
        if !touched.contains(&item) {
            touched.push(item);
        }
    }
    let mut list: Vec<(Recipe, u32, Option<(u32, u32, u32)>)> = shown(station)
        .into_iter()
        .filter(|r| match TABS[tab] {
            "" => true,
            PARTS_TAB => r.kind() == crate::parts::PART_RECIPES.name(),
            t => r
                .outputs()
                .first()
                .is_some_and(|(o, _)| item_has_tag(o.item, &format!("tab/{t}"))),
        })
        .filter_map(|r| {
            let n = times(&r, &have, MAX_BATCH);
            if creative || n > 0 || crate::discovery::starter(&r) {
                return Some((r, n, None));
            }
            // Known once every ingredient has been held; some of them: a "?" naming one still to
            // find.
            let (got, all, missing) = crate::discovery::standing(&r, &touched);
            match got {
                0 => None,
                g if g == all => Some((r, 0, None)),
                g => Some((r, 0, Some((g, all, missing.unwrap_or(0))))),
            }
        })
        .collect();
    // What you can make first, then what you know, then the near misses; otherwise as registered.
    list.sort_by_key(|(_, n, near)| ((*n == 0) as u8, near.is_some() as u8));
    let pages = list.len().div_ceil(PER_PAGE).max(1);
    let page = page.min(pages - 1);

    ui.panel("panel", S::col().pad(7.0).gap(3.0), |ui| {
        ui.text_styled(S::col().class("title"), "crafting.book.title", &[]);
        ui.row(S::row().gap(1.0), |ui| {
            for (i, t) in TABS.iter().enumerate() {
                let key = match *t {
                    "" => "crafting.book.all".to_string(),
                    PARTS_TAB => "crafting.book.parts".to_string(),
                    t => format!("tab.{}", t.replace(':', ".")),
                };
                // Categories (UI redo's class): transparent at rest, a gold outline when open.
                ui.button(
                    &key,
                    S::default()
                        .h(12.0)
                        .pad(2.0)
                        .class("category")
                        .selected(i == tab),
                    move |c| {
                        edit(|b| {
                            b.tab = i;
                            b.page = 0;
                        });
                        c.client().rebuild_screen();
                    },
                );
            }
        });
        // Slots touch: their dark groove is the gap (UI guide).
        ui.column(S::col(), |ui| {
            let start = page * PER_PAGE;
            let on_page = &list[start.min(list.len())..(start + PER_PAGE).min(list.len())];
            if on_page.is_empty() {
                ui.text_styled(S::col().class("dim"), "crafting.book.empty", &[]);
            }
            for row in on_page.chunks(COLS) {
                ui.row(S::row(), |ui| {
                    for (r, n, near) in row {
                        if let Some((got, all, missing)) = *near {
                            ui.button_with(
                                S::default()
                                    .size(18.0, 18.0)
                                    .class("slot")
                                    .tooltip_args("crafting.book.near", &[Arg::Int(got as i64), Arg::Int(all as i64), Arg::Item(missing)]),
                                |_| {},
                                |ui| ui.text_styled(S::col().class("dim"), "crafting.book.unknown", &[]),
                            );
                            continue;
                        }
                        let Some((out, _)) = r.outputs().into_iter().next() else { continue };
                        let index = id_of(r);
                        let class = if picked == Some(index) { "slot_highlight" } else { "slot" };
                        let tip = if *n > 0 { "crafting.book.can" } else { "crafting.book.missing" };
                        ui.button_with(
                            S::default()
                                .size(18.0, 18.0)
                                .class(class)
                                .tooltip_args(tip, &[Arg::Item(out.item)]),
                            move |c| {
                                edit(|b| b.picked = Some(index));
                                c.client().rebuild_screen();
                            },
                            |ui| ui.item(out.item, out.count, S::default().size(16.0, 16.0)),
                        );
                    }
                });
            }
        });
        if pages > 1 {
            ui.row(S::row().gap(4.0), |ui| {
                ui.button("crafting.book.prev", S::default().h(12.0).pad(2.0).disabled(page == 0), move |c| {
                    edit(|b| b.page = page.saturating_sub(1));
                    c.client().rebuild_screen();
                });
                ui.text_args("crafting.book.page", &[Arg::Int(page as i64 + 1), Arg::Int(pages as i64)]);
                ui.button("crafting.book.next", S::default().h(12.0).pad(2.0).disabled(page + 1 >= pages), move |c| {
                    edit(|b| b.page = page + 1);
                    c.client().rebuild_screen();
                });
            });
        }
        if let Some(r) = picked.and_then(recipe_of) {
            picked_recipe(ui, station, &r, &have);
        }
    });
}

/// The picked recipe: its result, each ingredient with how many you have, and Craft.
fn picked_recipe(ui: &mut Ui, station: Station, r: &Recipe, have: &[(u32, u32)]) {
    let Some((out, _)) = r.outputs().into_iter().next() else { return };
    let can = times(r, have, MAX_BATCH);
    let index = id_of(r);
    ui.row(S::row().gap(3.0), |ui| {
        ui.item(out.item, out.count, S::default().size(16.0, 16.0));
        ui.text_args("crafting.book.makes", &[Arg::Item(out.item), Arg::Int(out.count as i64)]);
    });
    // A part: of what, and the grade it's likely to come out (`parts.rs`).
    if let Some(key) = crate::parts::material_key(r) {
        ui.text_args("crafting.parts.of", &[Arg::Key(key)]);
    }
    if let Some((skill, level)) = crate::parts::skill_of(&ui.client(), r) {
        ui.text_args("crafting.parts.skill", &[Arg::Key(skill), Arg::Int(level as i64)]);
    }
    // Each distinct ingredient once, with the total it takes.
    let inputs = r.inputs();
    let mut lines: Vec<(u32, u32, u32)> = Vec::new(); // (input number, shown item, total)
    for (n, input) in inputs.iter().enumerate() {
        if input.kind == abi::recipe_input_kind::EMPTY {
            continue;
        }
        match lines.iter_mut().find(|l| l.1 == input.shown) {
            Some(l) => l.2 += input.count.max(1),
            None => lines.push((n as u32, input.shown, input.count.max(1))),
        }
    }
    for (n, shown, need) in lines {
        let got: u32 = have
            .iter()
            .filter(|(i, _)| r.accepts(n, *i))
            .map(|(_, c)| c)
            .sum();
        let key = if got >= need { "crafting.book.have" } else { "crafting.book.lack" };
        ui.row(S::row().gap(3.0), |ui| {
            ui.item(shown, 1, S::default().size(16.0, 16.0));
            ui.text_args(key, &[Arg::Item(shown), Arg::Int(got as i64), Arg::Int(need as i64)]);
        });
    }
    if station == Station::Hands && !fits_hands(r) {
        ui.text_styled(S::col().class("dim"), "crafting.book.needs_table", &[]);
        return;
    }
    ui.row(S::row().gap(2.0), |ui| {
        for (key, n) in [("crafting.book.craft_one", 1), ("crafting.book.craft_five", 5)] {
            ui.button(key, S::default().h(14.0).pad(3.0).disabled(can < n), move |c| {
                c.client()
                    .send(&CRAFT, &[index as u64, n as u64, station as u64]);
            });
        }
        ui.button_args("crafting.book.craft_all", &[Arg::Int(can as i64)], S::default().h(14.0).pad(3.0).disabled(can == 0), move |c| {
            c.client()
                .send(&CRAFT, &[index as u64, can as u64, station as u64]);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_bytes_round_trip() {
        let pool = vec![(3, 64), (17, 1), (u32::MAX, 9)];
        assert_eq!(decode(&encode(&pool)), pool);
        assert!(decode(&[1, 2, 3]).is_empty(), "a torn pair is dropped");
    }

    #[test]
    fn rolls_stay_in_range() {
        for n in 0..1000 {
            let x = roll(42, n);
            assert!((0.0..1.0).contains(&x));
        }
    }
}
