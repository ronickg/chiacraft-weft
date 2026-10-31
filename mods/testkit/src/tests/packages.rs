//! Data packages (chiacraft plan, phase 2): the `gems` example package in `packages/gems` is only
//! files, and its blocks, items, recipes and tags work like a code mod's.

use std::time::Duration;

use weft_sdk::*;

use super::{add, at};
use crate::runner::{R, T};

fn item(name: &str) -> u32 {
    Item::lookup(name).unwrap_or(u32::MAX)
}

pub async fn packages(t: T) -> R {
    for name in ["gems:ruby", "gems:ruby_block", "gems:ruby_lamp"] {
        t.check(Item::lookup(name).is_some(), format!("the package's {name} is an item"));
    }
    t.check(Block::lookup("gems:ruby_block").is_some(), "the package's block is registered");

    t.flat().await?;
    let f = t.feet();
    // Placed, drawn with the package's textures, and the lamp lights its surroundings.
    let block = add(f, [3, 0, -1]);
    let lamp = add(f, [3, 0, 1]);
    t.set(block, "gems:ruby_block").await?;
    t.set(lamp, "gems:ruby_lamp").await?;
    t.check_eq(t.block(block).await?, "gems:ruby_block".into(), "the ruby block placed");
    t.look_at(add(f, [3, 0, 0]));
    t.loaded().await?;
    t.frames(10).await?;
    t.shot("placed").await?;
    let light: u32 = t.op(&format!("light {}", at(add(lamp, [0, 1, 0])))).await?.parse().unwrap_or(0);
    t.check(light >= 13, format!("the lamp lights the block above it (light {light})"));

    // The package's shaped recipe: nine rubies in a crafting table make a block.
    let table = add(f, [0, 0, 2]);
    t.set(table, "crafting_table").await?;
    t.op("put me 0 gems:ruby 9").await?;
    t.look_at(table);
    t.frames(2).await?;
    t.press("core:use");
    t.screen("core:menu").await?;
    // Crafting table menu: 0-8 grid, 9 result, 10-36 main, 37-45 hotbar.
    let ruby = item("gems:ruby");
    t.click_ui("slot:37", 0);
    t.until("rubies on the cursor", Duration::from_secs(5), |t| t.c().cursor_stack().item == ruby).await?;
    for slot in 0..9 {
        t.click_ui(&format!("slot:{slot}"), 1);
        t.frames(2).await?;
    }
    let block_item = item("gems:ruby_block");
    let made = t.until("a ruby block in the result", Duration::from_secs(5), |t| t.c().menu_slot(9).item == block_item).await;
    t.check(made.is_ok(), "nine rubies craft a block of ruby (the package's shaped recipe)");
    t.shot("crafting").await?;
    t.c().close_screen();
    t.no_screen().await?;

    // Mined with a pickaxe it drops itself (the default), which is picked up.
    t.op("put me 1 diamond_pickaxe 1").await?;
    t.c().select(1);
    t.look_at(block);
    t.frames(2).await?;
    t.hold("core:attack", 3.0);
    t.secs(3.5).await?;
    t.check_eq(t.block(block).await?, "core:air".into(), "the ruby block mined");
    t.c().teleport([block[0] as f64 + 0.5, block[1] as f64, block[2] as f64 + 0.5]);
    let picked = t.until("a ruby block picked up", Duration::from_secs(5), |t| (0..36).any(|i| t.c().inventory_slot(i).item == block_item)).await;
    t.check(picked.is_ok(), "mining the ruby block gave the block");
    Ok(())
}
