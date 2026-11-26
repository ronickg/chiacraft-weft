//! The first hours' clock (OVERVIEW: "the first 30 minutes timed"): for each player, the seconds
//! of survival play at which each step of the way in first happened (a rock or bone in hand, an
//! edge, a head, string, the axe, a fire, logs, planks, the table, a pick, stone, copper, a copper tool). Any playtest,
//! a person's or a script's, then gives the timeline: a `first-clock:` log line as each step
//! comes, a summary line at `vitals::first::CLOCK_REPORT_SECS`, and `/firstclock` for the player's
//! own so far. Read by the timing table in `notes/FIRST_HOURS.md` against Ronald's "about 13-15
//! min" to the first axe.

use vitals::first::{self as f, CLOCK_STEPS};
use weft_sdk::*;

value! {
    /// Seconds of survival play so far, and when each of `CLOCK_STEPS` first came (0: not yet;
    /// the second it came, from 1).
    pub struct Clock { pub secs: u32, pub at: [u32; 14] }
}
static CLOCK: Component<Clock> = Component::named("first_clock");
const _: () = assert!(CLOCK_STEPS.len() == 14, "Clock::at holds one second per step");

static TAGS: [Tag; 9] = [
    Tag::named("rocks"),
    Tag::named("bone"),
    Tag::named("cutters"),
    Tag::named("string"),
    Tag::named("tools/axe"),
    Tag::named("logs"),
    Tag::named("planks"),
    Tag::named("crafting_tables"),
    Tag::named("tools/pickaxe"),
];
static COBBLESTONE: Tag = Tag::named("cobblestone");
static CAMPFIRES: Tag = Tag::named("campfires");
static NATIVE_COPPER: Tag = Tag::named("native_copper");
static COPPER_RAW: Tag = Tag::named("raw_ores/copper");
static COPPER_INGOTS: Tag = Tag::named("ingots/copper");

/// Copper in a stack: a native lump, raw ore, an ingot, or a part of copper (a head, or a tool
/// with any copper part).
fn copper_in(s: &ItemStack) -> bool {
    let copper = crafting::code("copper");
    [&NATIVE_COPPER, &COPPER_RAW, &COPPER_INGOTS]
        .iter()
        .any(|t| t.on_item(s.item))
        || crate::parts::PART
            .on_stack_own(s)
            .is_some_and(|p| p.material == copper)
        || crate::parts::PARTS
            .on_stack_own(s)
            .is_some_and(|p| p.materials[..(p.n as usize).min(4)].contains(&copper))
}

/// A tool whose working part (a head or blade) is copper.
fn copper_tool(s: &ItemStack) -> bool {
    let Some(p) = crate::parts::PARTS.on_stack_own(s) else { return false };
    let Some(kind) = crafting::KINDS
        .iter()
        .find(|k| crafting::code(k.id) == p.kind)
    else {
        return false;
    };
    let working = crafting::code(kind.working());
    (0..(p.n as usize).min(4)).any(|i| p.slots[i] == working && p.materials[i] == crafting::code("copper"))
}

pub fn register(r: &mut Registry) {
    r.entity_component(&CLOCK, Clock { secs: 0, at: [0; 14] }, component_flags::SAVE | component_flags::KEEP_ON_RESPAWN);
    r.edit(&PLAYER).attach(&CLOCK);
    r.world_tick(tick);
    r.command("firstclock", "", |cmd| {
        let w = cmd.world();
        let Some(c) = w.component(cmd.player(), &CLOCK) else { return };
        let text = summary(&c);
        cmd.reply("command.base.firstclock", &[Arg::Int(c.secs as i64), Arg::Text(&text)]);
    });
}

/// `step 40 s, step 95 s, ...` for the steps done, in `CLOCK_STEPS`' order.
fn summary(c: &Clock) -> String {
    CLOCK_STEPS
        .iter()
        .enumerate()
        .filter(|&(i, _)| c.at[i] > 0)
        .map(|(i, s)| format!("{s} {} s", c.at[i]))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether step `i` of `CLOCK_STEPS` is done for a bag of `stacks` near `fire`.
fn done(i: usize, stacks: &[ItemStack], fire: bool) -> bool {
    let items: Vec<u32> = stacks.iter().map(|s| s.item).collect();
    let has = |t: &Tag| items.iter().any(|&it| t.on_item(it));
    match CLOCK_STEPS[i] {
        "rock" => has(&TAGS[0]),
        "bone" => has(&TAGS[1]),
        "edge" => has(&TAGS[2]),
        "head" => items.iter().any(|&it| {
            let n = item_name(it);
            n.starts_with("base:part_") && f::KNAPPED.iter().any(|k| n.ends_with(k.shape))
        }),
        "string" => has(&TAGS[3]),
        "axe" => has(&TAGS[4]),
        "fire" => fire,
        "logs" => has(&TAGS[5]),
        "planks" => has(&TAGS[6]),
        "table" => has(&TAGS[7]),
        "pickaxe" => has(&TAGS[8]),
        "stone" => has(&COBBLESTONE),
        "copper" => stacks.iter().any(copper_in),
        "copper_tool" => stacks.iter().any(copper_tool),
        _ => false,
    }
}

/// Once a second: each survival player's clock on, and any step new since the last.
fn tick(w: &World) {
    for id in w.players() {
        let Some(st) = w.entity(id) else { continue };
        if st.age % 20 != 11 || w.outside_survival(id) {
            continue;
        }
        let Some(mut c) = w.component(id, &CLOCK) else { continue };
        c.secs += 1;
        let open: Vec<usize> = (0..CLOCK_STEPS.len()).filter(|&i| c.at[i] == 0).collect();
        if !open.is_empty() {
            let inv = w.inventory(id);
            let stacks: Vec<ItemStack> = (0..inv.len())
                .map(|i| inv.get(i))
                .filter(|s| !s.is_empty())
                .collect();
            // A fire burning near: lit by the drill, or built (only looked for until one is).
            let fire = c.at[CLOCK_STEPS.iter().position(|&s| s == "fire").unwrap_or(0)] == 0 && fire_near(w, st.block_pos());
            for i in open {
                if done(i, &stacks, fire) {
                    c.at[i] = c.secs;
                    weft_sdk::info!("first-clock: {id} {} at {} s", CLOCK_STEPS[i], c.secs);
                }
            }
        }
        if c.secs == f::CLOCK_REPORT_SECS {
            weft_sdk::info!("first-clock: {id} at {} s: {}", c.secs, summary(&c));
        }
        if let Some(m) = w.component_mut(id, &CLOCK) {
            *m = c;
        }
    }
}

/// A campfire within `vitals::first::CLOCK_FIRE_REACH` blocks.
fn fire_near(w: &World, at: BlockPos) -> bool {
    let r = f::CLOCK_FIRE_REACH;
    (-r..=r).any(|dx| (-2..=2).any(|dy| (-r..=r).any(|dz| CAMPFIRES.on_block(w.block(at.offset(dx, dy, dz))))))
}
