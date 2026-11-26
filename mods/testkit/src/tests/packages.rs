//! Data packages (chiacraft plan, phase 2): the `gems` example package is only files, and its
//! blocks, items, recipes and tags work like a code mod's. Run with the examples' packages:
//! `weft --packages examples/packages --test packages`.

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
    let light: u32 = t
        .op(&format!("light {}", at(add(lamp, [0, 1, 0]))))
        .await?
        .parse()
        .unwrap_or(0);
    t.check(light >= 13, format!("the lamp lights the block above it (light {light})"));

    // The package's shaped recipe: nine rubies in a crafting table make a block.
    let table = add(f, [0, 0, 2]);
    t.set(table, "crafting_table").await?;
    t.op("put me 0 gems:ruby 9").await?;
    t.look_at(table);
    t.frames(2).await?;
    t.press("core:use");
    t.screen("core:menu").await?;
    // Crafting table menu: 0-8 grid, 9 result, then the player's inventory (`t.hotbar` finds
    // its hotbar: base's has more slots than Weft's 36).
    let ruby = item("gems:ruby");
    t.click_settled(&t.hotbar(0), 0).await?;
    t.until("rubies on the cursor", Duration::from_secs(5), |t| t.c().cursor_stack().item == ruby)
        .await?;
    for slot in 0..9 {
        t.click_settled(&format!("slot:{slot}"), 1).await?;
    }
    let block_item = item("gems:ruby_block");
    let made = t
        .until("a ruby block in the result", Duration::from_secs(5), |t| t.c().menu_slot(9).item == block_item)
        .await;
    t.check(made.is_ok(), "nine rubies craft a block of ruby (the package's shaped recipe)");
    t.shot("crafting").await?;
    t.c().menu_close();
    t.no_screen().await?;

    // Mined with a pickaxe it drops itself (the default), which is picked up.
    t.cmd("/gear pickaxe diamond slot=1").await?;
    t.c().select_slot(1);
    t.look_at(block);
    t.frames(2).await?;
    t.hold("core:attack", 3.0);
    t.secs(3.5).await?;
    t.check_eq(t.block(block).await?, "core:air".into(), "the ruby block mined");
    t.c()
        .teleport([block[0] as f64 + 0.5, block[1] as f64, block[2] as f64 + 0.5]);
    let picked = t
        .until("a ruby block picked up", Duration::from_secs(5), |t| (0..36).any(|i| t.c().inventory_slot(i).item == block_item))
        .await;
    t.check(picked.is_ok(), "mining the ruby block gave the block");
    Ok(())
}

/// A world names the exact packages it was played with (`package.<id> = <hash>` in `world.txt`):
/// opening it with another version of one, or without one, asks first.
pub async fn package_pins(t: T) -> R {
    if Item::lookup("gems:ruby").is_none() {
        t.note("needs the examples' packages (--packages examples/packages); skipped");
        return Ok(());
    }
    t.screen("core:title").await?;
    let dir = format!("{}/saves/test-package_pins", t.c().run_dir());
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let zeros = "0".repeat(64);
    let world = format!(
        "format = 3\nname = Pinned World\nseed = 1\nlast_played = 99999999999\nnext_entity = 1\nmod.base = 26.10.2\nmod.gems = 0.1.0\nmod.gone = 1.0\npackage.gems = {zeros}\npackage.gone = {}\n",
        "1".repeat(64)
    );
    std::fs::write(format!("{dir}/world.txt"), world).map_err(|e| e.to_string())?;

    t.check(t.c().menu_open(&screens::WORLDS), "cannot open the world list");
    t.screen("core:worlds").await?;
    t.frames(2).await?;
    // Select, then Play: either click can land while the list rebuilds and be lost, so the pair
    // is repeated until the gate opens.
    for _ in 0..10 {
        t.click_ui("text:Pinned World", 0);
        t.frames(10).await?;
        t.click_ui("id:play", 0);
        t.frames(15).await?;
        if t.c().screen() == "core:missing_mods" {
            break;
        }
    }
    t.screen("core:missing_mods").await?;
    t.frames(2).await?;
    t.shot("pins").await?;
    t.laid_out(&["text:gone 1.0", "text:gems: another version", "text:000000000000", "id:continue"]);
    t.check(t.c().ui_rect("text:base 26").is_none(), "a loaded mod is not listed");
    t.c().menu_close();
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}
