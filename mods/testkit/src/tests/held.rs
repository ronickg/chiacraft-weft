//! Blocks standing in water (`holds_liquid`), with testmod's waterweed (always in water) and sea
//! slab (`waterlogged=true` its wet twin): placing, breaking, buckets, pistons, swimming and how
//! the water around them is drawn. The engine's own checks are its unit tests (`liquid.rs`).

use weft_sdk::*;

use super::{add, probe};
use crate::runner::{R, Shot, T};

const WEED: &str = "testmod:waterweed";
const WET: &str = "testmod:sea_slab[waterlogged=true]";
const DRY: &str = "testmod:sea_slab[waterlogged=false]";
const SOURCE: &str = "base:water[level=0]";

/// A pool dug into the flat ground in front of the player: x 1..=3, z -3..=-1, `deep` blocks.
async fn pool(t: &T, deep: i32) -> Result<[i32; 3], String> {
    t.flat().await?;
    let f = t.feet();
    t.fill(add(f, [1, -deep, -3]), add(f, [3, -1, -1]), "water")
        .await?;
    t.secs(0.5).await?;
    Ok(f)
}

/// Placing into water fills a slab; out of water it stays dry; the weed goes only into water.
pub async fn held_water_place(t: T) -> R {
    let f = pool(&t, 2).await?;
    t.op("put me 0 testmod:sea_slab 4").await?;
    t.op("put me 1 testmod:waterweed 4").await?;
    t.c().select(0);
    t.secs(0.2).await?;
    // Onto the pool's floor, through the water.
    t.look_at(add(f, [2, -3, -2]));
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.block(add(f, [2, -2, -2])).await?, WET.to_string(), "a slab placed into water stands in it");
    t.look_at(add(f, [-2, -1, 0]));
    t.frames(3).await?;
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.block(add(f, [-2, 0, 0])).await?, DRY.to_string(), "a slab placed on land is dry");

    t.c().select(1);
    t.secs(0.3).await?;
    t.look_at(add(f, [1, -3, -2]));
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.block(add(f, [1, -2, -2])).await?, WEED.to_string(), "the weed placed into water");
    t.look_at(add(f, [-3, -1, 1]));
    t.frames(3).await?;
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.block(add(f, [-3, 0, 1])).await?, "core:air".to_string(), "the weed refused on land");
    t.check_eq(t.c().inventory_slot(1).count, 3, "the refused weed kept");
    // Nothing flowed away or vanished: the pool's other cells are still sources.
    t.check_eq(t.block(add(f, [3, -1, -3])).await?, SOURCE.to_string(), "the pool still full");
    Ok(())
}

/// Breaking (mining, a command) leaves the water; a bucket drains a slab and fills it again, and
/// takes nothing from the weed.
pub async fn held_water_break(t: T) -> R {
    let f = pool(&t, 2).await?;
    let (weed, slab, other) = (add(f, [1, -1, -2]), add(f, [2, -1, -1]), add(f, [3, -1, -2]));
    t.set(weed, WEED).await?;
    t.set(slab, WET).await?;
    t.set(other, WEED).await?;
    t.secs(0.5).await?;
    t.look_at(weed);
    t.frames(3).await?;
    t.press("core:attack");
    t.secs(0.5).await?;
    t.check_eq(t.block(weed).await?, SOURCE.to_string(), "the mined weed leaves its water");
    t.look_at(slab);
    t.frames(3).await?;
    t.hold("core:attack", 1.5);
    t.secs(1.8).await?;
    t.check_eq(t.block(slab).await?, SOURCE.to_string(), "the mined slab leaves its water");
    t.set(other, "air").await?;
    t.secs(0.3).await?;
    t.check_eq(t.block(other).await?, SOURCE.to_string(), "set to air by a command, the water stays");

    // Buckets.
    t.set(slab, WET).await?;
    t.set(weed, WEED).await?;
    t.op("put me 0 bucket 1").await?;
    t.c().select(0);
    t.secs(0.3).await?;
    t.look_at(slab);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:water_bucket".to_string(), "water taken from the slab");
    t.check_eq(t.block(slab).await?, DRY.to_string(), "the slab drained");
    t.secs(0.3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:bucket".to_string(), "poured back");
    t.check_eq(t.block(slab).await?, WET.to_string(), "the slab filled again");
    t.secs(0.3).await?;
    t.look_at(weed);
    t.frames(3).await?;
    t.press("core:use");
    t.secs(0.4).await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:bucket".to_string(), "nothing taken from the weed");
    t.check_eq(t.block(weed).await?, WEED.to_string(), "the weed still there");
    Ok(())
}

/// A sticky piston in a flooded channel pushes a slab on into the water, where it stands wet,
/// and pulls it back dry, the water staying behind. A plain one breaks the weed off, its water
/// staying.
pub async fn held_water_piston(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    let lever = |on: bool| format!("base:lever[attach=down,front=north,power={}]", if on { "on" } else { "off" });
    for (z, piston, thing) in [(-4, "base:sticky_piston[facing=east]", WET), (-8, "base:piston[facing=east]", WEED)] {
        // A channel one deep, x 4..=8, flooded; the piston at its west end, a lever beside it.
        t.fill(add(f, [4, -1, z]), add(f, [8, -1, z]), "water")
            .await?;
        t.set(add(f, [2, -1, z]), &lever(false)).await?;
        t.set(add(f, [3, -1, z]), piston).await?;
        t.set(add(f, [4, -1, z]), thing).await?;
    }
    t.secs(1.0).await?;
    for z in [-4, -8] {
        t.set(add(f, [2, -1, z]), &lever(true)).await?;
    }
    t.secs(1.0).await?;
    t.check_eq(t.block(add(f, [5, -1, -4])).await?, WET.to_string(), "pushed into water, the slab stands in it");
    t.check(t.block(add(f, [4, -1, -4])).await?.contains("piston_head"), "the head out");
    let weed_gone = t.block(add(f, [5, -1, -8])).await?;
    t.check(!weed_gone.contains("waterweed"), format!("the weed broke, not moved: {weed_gone}"));
    for z in [-4, -8] {
        t.set(add(f, [2, -1, z]), &lever(false)).await?;
    }
    t.secs(1.5).await?;
    t.check_eq(t.block(add(f, [4, -1, -4])).await?, DRY.to_string(), "pulled back out of the water's way, dry");
    t.check_eq(t.block(add(f, [5, -1, -4])).await?, SOURCE.to_string(), "the water stayed where it stood");
    t.check_eq(t.block(add(f, [5, -1, -8])).await?, SOURCE.to_string(), "the weed's channel still full");
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
    t.fill(add(f, [-8, -4, -40]), add(f, [8, -1, 8]), WEED)
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
    t.c().teleport([s[0], f[1] as f64 - 3.9, f[2] as f64 + 0.5]);
    t.secs(2.5).await?;
    t.check(air(&t) < 300.0, format!("air runs out among the weed: {}", air(&t)));
    t.c().look(0.0, -10.0);
    t.hold("core:sprint", 4.0);
    t.hold("core:forward", 4.0);
    t.secs(0.5).await?;
    let mut swum = 0;
    for _ in 0..120 {
        t.frames(1).await?;
        swum += t.c().view().me().is_some_and(|m| m.pose == abi::pose::SWIM) as u32;
    }
    t.check(swum > 20, format!("swam through the weed for {swum} frames"));
    t.secs(4.0).await?;
    let moved = (s[2] - t.state().pos[2]).abs();
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
    t.fill(add(f, [-1, -3, -10]), add(f, [1, -1, -4]), WEED)
        .await?;
    t.fill(add(f, [2, -3, -10]), add(f, [4, -3, -4]), WET)
        .await?;
    t.secs(1.0).await?;
    t.c().look(0.0, -45.0);
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
    t.c().look(270.0, 0.0);
    t.frames(30).await?;
    t.shot("held_below").await?;
    t.c().look(0.0, -89.0);
    t.cmd(&format!("/tp {} {} {}", f[0] as f64 + 0.5, f[1] as f64 + 2.0, f[2] as f64 - 7.0))
        .await?;
    t.frames(20).await?;
    t.shot("held_down").await?;
    Ok(())
}
