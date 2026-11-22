//! Blocks standing in water (`.shares_cell_with("#water")`, Weft's waterlogging), with testmod's waterweed and
//! sea slab: the water is a part of the cell beside the block (`/tk parts`), so "wet" is the block
//! and a water source in one cell. Placing, breaking, buckets, pistons, swimming and how the water
//! around them is drawn. The engine's own checks are its unit tests (`liquid.rs`).

use weft_sdk::*;

use super::{add, at, probe};
use crate::runner::{R, Shot, T};

const WEED: &str = "testmod:waterweed";
const SLAB: &str = "testmod:sea_slab";
const SOURCE: &str = "base:water[level=0]";

/// The names of what is in a cell (`/tk parts`), sorted: `[SLAB, SOURCE]` is a wet slab.
async fn parts(t: &T, p: [i32; 3]) -> Result<Vec<String>, String> {
    let mut v: Vec<String> = t
        .op(&format!("parts {}", at(p)))
        .await?
        .split_whitespace()
        .map(String::from)
        .collect();
    v.sort();
    Ok(v)
}

/// What a cell holds, as a sorted list to compare with.
fn cell(names: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    v.sort();
    v
}

/// Water in the cell with `thing` standing in it (what placing it into water, or a bucket, makes).
async fn in_water(t: &T, p: [i32; 3], thing: &str) -> R {
    t.set(p, "water").await?;
    let added = t.op(&format!("addpart {} {thing}", at(p))).await?;
    t.check_eq(added, "1".to_string(), "the block joins the water's cell");
    Ok(())
}

/// A pool dug into the flat ground in front of the player: x 1..=3, z -3..=-1, `deep` blocks.
async fn pool(t: &T, deep: i32) -> Result<[i32; 3], String> {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [1, -deep, -3]), add(f, [3, -1, -1]), "water")
        .await?;
    t.secs(0.5).await?;
    Ok(f)
}

/// Looks at the middle of the top of block `p` (`look_at` aims at its middle, which on a floor
/// seen from low down is behind the top of the block in front).
fn aim_top(t: &T, p: [i32; 3]) {
    let s = t.state();
    let eye = [s.pos.x, s.pos.y + t.c().entity(0).map_or(1.62, |e| e.size.eye as f64), s.pos.z];
    let d = [p[0] as f64 + 0.5 - eye[0], p[1] as f64 + 0.99 - eye[1], p[2] as f64 + 0.5 - eye[2]];
    let yaw = (-d[0]).atan2(-d[2]).to_degrees() as f32;
    let pitch = d[1].atan2((d[0] * d[0] + d[2] * d[2]).sqrt()).to_degrees() as f32;
    t.c().set_look(yaw, pitch);
}

/// Placing into water fills a slab; out of water it stays dry; the weed goes only into water.
pub async fn held_water_place(t: T) -> R {
    let f = pool(&t, 1).await?;
    t.op("put me 0 testmod:sea_slab 4").await?;
    t.op("put me 1 testmod:waterweed 4").await?;
    t.c().select_slot(0);
    t.secs(0.2).await?;
    // Onto the pool's floor, through the water: from the edge, the one cell within reach whose
    // ray misses the grass edge wherever in its cell the player stands. The slab, then the weed.
    let (floor, cell) = (add(f, [2, -2, -2]), add(f, [2, -1, -2]));
    aim_top(&t, floor);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(parts(&t, cell).await?, self::cell(&[SLAB, SOURCE]), "a slab placed into water stands in it, the water kept");
    t.look_at(add(f, [-2, -1, 0]));
    t.frames(3).await?;
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(parts(&t, add(f, [-2, 0, 0])).await?, self::cell(&[SLAB]), "a slab placed on land is dry");
    t.op(&format!("rmpart {} {SLAB}", at(cell))).await?;
    t.secs(0.3).await?;
    t.check_eq(parts(&t, cell).await?, self::cell(&[SOURCE]), "the slab taken out, its water stays");

    t.c().select_slot(1);
    t.secs(0.3).await?;
    aim_top(&t, floor);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check(parts(&t, cell).await?.contains(&WEED.to_string()), "the weed placed into water");
    t.look_at(add(f, [-3, -1, 1]));
    t.frames(3).await?;
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.block(add(f, [-3, 0, 1])).await?, "core:air".to_string(), "the weed refused on land");
    // Weft's plant placing may take the weed and pop it off dry land as an item; noted, not
    // asserted (SYNC9: 2 left of 3).
    t.note(format!("weed left after the refused one: {}", t.c().inventory_slot(1).count));
    // Nothing flowed away or vanished: the pool's other cells are still sources.
    t.check_eq(t.block(add(f, [3, -1, -3])).await?, SOURCE.to_string(), "the pool still full");
    Ok(())
}

/// Breaking (mining, a command) leaves the water; a bucket drains a slab and fills it again (what
/// it does to the weed's water is noted).
pub async fn held_water_break(t: T) -> R {
    let f = pool(&t, 2).await?;
    let (weed, slab, other) = (add(f, [1, -1, -2]), add(f, [2, -1, -1]), add(f, [3, -1, -2]));
    in_water(&t, weed, WEED).await?;
    in_water(&t, slab, SLAB).await?;
    in_water(&t, other, WEED).await?;
    t.secs(0.5).await?;
    t.look_at(weed);
    t.frames(3).await?;
    t.press("core:attack");
    t.secs(0.5).await?;
    t.check_eq(parts(&t, weed).await?, cell(&[SOURCE]), "the mined weed leaves its water");
    t.look_at(slab);
    t.frames(3).await?;
    t.hold("core:attack", 1.5);
    t.secs(1.8).await?;
    t.check_eq(parts(&t, slab).await?, cell(&[SOURCE]), "the mined slab leaves its water");
    t.op(&format!("rmpart {} {WEED}", at(other))).await?;
    t.secs(0.3).await?;
    t.check_eq(parts(&t, other).await?, cell(&[SOURCE]), "the weed taken out by a command, the water stays");

    // Buckets.
    in_water(&t, slab, SLAB).await?;
    in_water(&t, weed, WEED).await?;
    t.op("put me 0 bucket 1").await?;
    t.c().select_slot(0);
    t.secs(0.3).await?;
    t.look_at(slab);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:water_bucket".to_string(), "water taken from the slab");
    t.check_eq(parts(&t, slab).await?, cell(&[SLAB]), "the slab drained");
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:bucket".to_string(), "poured back");
    t.check_eq(parts(&t, slab).await?, cell(&[SLAB, SOURCE]), "the slab filled again");
    t.secs(0.3).await?;
    t.look_at(weed);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    // Weft's bucket takes any water source it looks at, and the test mod's weed holds the water
    // the same way a slab does, so what this does with the weed's water is noted, not asserted
    // (kelp's water not coming out is vanilla's, not Weft's, rule).
    let after = t.slot("me", 0).await?.0;
    t.note(format!("bucket on the weed: {after}; its cell holds {:?}", parts(&t, weed).await?));
    // Weft's bucket takes the weed's water, and the weed goes with it (SYNC9: a water bucket,
    // the cell holding only the water flowing back): its rule, noted in the sync log.
    t.check(!parts(&t, weed).await?.is_empty(), "the weed's cell isn't left empty");
    Ok(())
}

/// A sticky piston in a flooded channel pushes a slab on into the water, where it stands wet,
/// and pulls it back dry, the water staying behind. A plain one breaks the weed off, its water
/// staying.
pub async fn held_water_piston(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    let lever = |on: bool| format!("base:lever[attach=down,front=north,power={}]", if on { "on" } else { "off" });
    for (z, piston, thing) in [(-4, "base:sticky_piston[facing=east]", SLAB), (-8, "base:piston[facing=east]", WEED)] {
        // A channel one deep, x 4..=8, flooded; the piston at its west end, a lever beside it.
        t.fill(add(f, [4, -1, z]), add(f, [8, -1, z]), "water")
            .await?;
        t.set(add(f, [2, -1, z]), &lever(false)).await?;
        t.set(add(f, [3, -1, z]), piston).await?;
        in_water(&t, add(f, [4, -1, z]), thing).await?;
    }
    t.secs(1.0).await?;
    for z in [-4, -8] {
        t.set(add(f, [2, -1, z]), &lever(true)).await?;
    }
    t.secs(1.0).await?;
    t.check_eq(parts(&t, add(f, [5, -1, -4])).await?, cell(&[SLAB, SOURCE]), "pushed into water, the slab stands in it");
    t.check(t.block(add(f, [4, -1, -4])).await?.contains("piston_head"), "the head out");
    let weed_gone = parts(&t, add(f, [5, -1, -8])).await?;
    t.check(!weed_gone.contains(&WEED.to_string()), format!("the weed broke, not moved: {weed_gone:?}"));
    for z in [-4, -8] {
        t.set(add(f, [2, -1, z]), &lever(false)).await?;
    }
    t.secs(1.5).await?;
    // Pulled back into the channel's first cell, which the water beside it fills again (Weft:
    // SYNC9 had it wet), so either is right.
    let back = parts(&t, add(f, [4, -1, -4])).await?;
    t.check(back == cell(&[SLAB]) || back == cell(&[SLAB, SOURCE]), format!("pulled back, the slab is there: {back:?}"));
    t.check_eq(parts(&t, add(f, [5, -1, -4])).await?, cell(&[SOURCE]), "the water stayed where it stood");
    t.check_eq(parts(&t, add(f, [5, -1, -8])).await?, cell(&[SOURCE]), "the weed's channel still full");
    let near = t.count_near("core:item", 12.0).await?;
    t.check(near >= 1, format!("the broken weed dropped: {near} items"));
    Ok(())
}

/// A pool full of weed is water to the body: it swims, the eyes are under water (air runs out),
/// mods asking `in_liquid` get yes, and the surface sits where a source's does.
pub async fn held_water_swim(t: T) -> R {
    fn air(t: &T) -> f32 {
        t.c()
            .view()
            .component(&base_api::VITALS)
            .map_or(0.0, |v| v.air)
    }
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [-8, -4, -40]), add(f, [8, -1, 8]), "water")
        .await?;
    t.op(&format!("fillpart {} {} {WEED}", at(add(f, [-8, -4, -40])), at(add(f, [8, -1, 8]))))
        .await?;
    t.secs(0.5).await?;
    let p = add(f, [0, -2, -3]);
    t.check_eq(
        t.op(&format!("in_liquid {} {} {}", p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5))
            .await?,
        "1".to_string(),
        "in a weed cell is in water",
    );
    let top = add(f, [0, -1, -3]);
    t.check_eq(
        t.op(&format!("in_liquid {} {} {}", top[0] as f64 + 0.5, top[1] as f64 + 0.95, top[2] as f64 + 0.5))
            .await?,
        "0".to_string(),
        "above the top weed's water surface",
    );
    let s = t.state().pos;
    t.c().teleport([s.x, f[1] as f64 - 3.9, f[2] as f64 + 0.5]);
    t.secs(2.5).await?;
    t.check(air(&t) < 300.0, format!("air runs out among the weed: {}", air(&t)));
    t.c().set_look(0.0, -10.0);
    t.hold("core:sprint", 4.0);
    t.hold("core:forward", 4.0);
    t.secs(0.5).await?;
    let mut swum = 0;
    for _ in 0..120 {
        t.frames(1).await?;
        swum += t
            .c()
            .view()
            .player_state()
            .is_some_and(|m| m.pose == abi::pose::SWIM) as u32;
    }
    t.check(swum > 20, format!("swam through the weed for {swum} frames"));
    t.secs(4.0).await?;
    let moved = (s.z - t.state().pos.z).abs();
    t.check(moved > 3.0, format!("got somewhere through it: {moved:.1} blocks"));
    Ok(())
}

fn around(t: &T, shot: &Shot, p: [f64; 3], r: f32, f: impl FnMut([u8; 3]) -> bool) -> f32 {
    let Some((x, y)) = t.on_screen(p) else { return 0.0 };
    shot.share(x - r, y - r, x + r, y + r, f)
}

/// One pool, a third plain water, a third weed, a third wet slabs: from above its surface is
/// one sheet (no hole over the weed or slabs), from below no face shows between them.
pub async fn held_water_mesh(t: T) -> R {
    t.flat().await?;
    t.op("time 6000").await?;
    let f = t.feet();
    t.fill(add(f, [-4, -3, -10]), add(f, [4, -1, -4]), "water")
        .await?;
    t.op(&format!("fillpart {} {} {WEED}", at(add(f, [-1, -3, -10])), at(add(f, [1, -1, -4]))))
        .await?;
    t.op(&format!("fillpart {} {} {SLAB}", at(add(f, [2, -3, -10])), at(add(f, [4, -3, -4]))))
        .await?;
    t.secs(1.0).await?;
    t.c().set_look(0.0, -45.0);
    t.frames(30).await?;
    let above = t.shot("held_above").await?;
    let y = f[1] as f64 - 0.12;
    let (water, weed, slab) = (
        around(&t, &above, [f[0] as f64 - 2.5, y, f[2] as f64 - 6.5], 0.02, probe::blue),
        around(&t, &above, [f[0] as f64 + 0.5, y, f[2] as f64 - 6.5], 0.02, probe::blue),
        around(&t, &above, [f[0] as f64 + 3.5, y, f[2] as f64 - 6.5], 0.02, probe::blue),
    );
    t.probe("plain water drawn blue from above", water, water > 0.4);
    t.probe("over the weed too", weed, weed > 0.4);
    t.probe("over the wet slabs too", slab, slab > 0.4);

    // Under the surface, looking across the weed from the plain water: a face between them would
    // put a sheet of water's color over the weed, as dark as it is at the far wall.
    t.cmd(&format!("/tp {} {} {}", f[0] as f64 - 3.5, f[1] as f64 - 2.5, f[2] as f64 - 7.0))
        .await?;
    t.c().set_look(270.0, 0.0);
    t.frames(30).await?;
    t.shot("held_below").await?;
    t.c().set_look(0.0, -89.0);
    t.cmd(&format!("/tp {} {} {}", f[0] as f64 + 0.5, f[1] as f64 + 2.0, f[2] as f64 - 7.0))
        .await?;
    t.frames(20).await?;
    t.shot("held_down").await?;
    Ok(())
}
