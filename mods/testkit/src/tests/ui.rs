use std::time::Duration;

use weft_sdk::*;

use crate::runner::{R, T};

/// Screens open by name and lay out inside the window: the engine's menus and a mod's own.
pub async fn screens(t: T) -> R {
    t.c().menu_open(&screens::TITLE);
    t.screen("core:title").await?;
    t.shot("title").await?;
    t.laid_out(&[
        "text:Singleplayer",
        "text:Multiplayer",
        "text:Mods",
        "text:Settings",
        "id:profile",
        "text:Quit Game",
        "text:About Test Mod",
    ]);
    // A mod's button in the main menu opens its own screen, with no world around.
    t.check(t.click_ui("text:About Test Mod", 0), "the test mod's main menu button");
    t.screen("testmod:about").await?;
    t.c().menu_close();
    t.screen("core:title").await?;
    for (screen, texts) in [
        (&screens::SETTINGS, &["text:View distance", "text:Language", "text:Controls", "text:Shader Packs", "text:Done"][..]),
        (&screens::LANGUAGE, &["text:English", "text:Done"][..]),
        (&screens::CONTROLS, &["text:Reset All", "text:Done"][..]),
        (&screens::WORLDS, &["text:Create New World", "id:saves_folder"][..]),
        (&screens::CREATE_WORLD, &["text:World Rules", "text:Create New World"][..]),
        (&screens::WORLD_RULES, &["text:World Rules", "text:Done"][..]),
        (&screens::MODS, &["id:mod:base", "id:mod:testmod", "id:mod_description"][..]),
        (&screens::PROFILE, &["id:profile_name", "id:change_skin", "entity", "text:Done"][..]),
    ] {
        let name = screen.name();
        t.check(t.c().menu_open(screen), format!("cannot open {name}"));
        t.screen(name).await?;
        t.shot(name.trim_start_matches("core:")).await?;
        t.laid_out(texts);
        t.c().menu_close();
        t.screen("core:title").await?;
    }
    t.flat().await?;
    t.c().say("/about");
    t.screen("testmod:about").await?;
    t.until("its text", Duration::from_secs(2), |t| t.c().ui_rect("text:opened by name").is_some())
        .await?;
    t.shot("about").await?;
    t.laid_out(&["text:Test Mod", "text:opened by name"]);
    // A part of a mod's screen with a tooltip of its own.
    let Rect { x, y, w, h } = t.c().ui_rect("text:Hover me").ok_or("no part to hover")?;
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(4).await?;
    let shot = t.shot("about_tooltip").await?;
    let red = shot.share(x + w / 2.0, y - 0.02, (x + 0.3).min(1.0), y + 0.06, super::probe::red);
    t.probe("the tooltip's red text", red, red > 0.005);
    // What the engine's screens have, a mod's has too: it pauses, takes text, slides, shows items.
    t.check(t.flag(client::client_flags::PAUSED), "a mod's screen declared to pause pauses single player");
    t.check(t.click_ui("id:about_name", 0), "the mod screen's text field");
    t.frames(2).await?;
    t.c().input_type("Steve");
    t.until("the typed name", Duration::from_secs(3), |t| t.c().ui_rect("text:Hello, Steve").is_some())
        .await?;
    let Rect { x, y, w, h } = t.c().ui_rect("slider").ok_or("no slider")?;
    t.c().input_cursor(x + w * 0.75, y + h / 2.0);
    t.c().input_click(0, 0.1);
    t.until("the slider moved", Duration::from_secs(3), |t| (70..=80).any(|v| t.c().ui_rect(&format!("text:Volume: {v}")).is_some()))
        .await?;
    t.check(t.c().ui_rect("text:Field of view here: 70").is_some(), "a mod's screen reads the settings");
    let coal = Item::lookup("base:coal").ok_or("no coal")?;
    t.check(t.c().ui_rect(&format!("icon:{coal}")).is_some(), "the item on the mod's screen");
    t.shot("about_parts").await?;
    t.c().menu_close();
    t.no_screen().await?;
    t.check(!t.flag(client::client_flags::PAUSED), "running again when it closed");
    Ok(())
}

/// The HUD editor moves, hides and restores anything on the overlay, and the arrangement is kept
/// in a file (in a test run, beside the player's).
pub async fn hud_editor(t: T) -> R {
    let file = format!("{}/test-config/hud.txt", t.c().run_dir());
    let _ = std::fs::remove_file(&file);
    t.flat().await?;
    let rect = |t: &T, name: &str| t.c().ui_rect(&format!("id:hud:{name}"));
    let before = rect(&t, "core:hotbar").ok_or("no hotbar element")?;
    t.check(t.c().menu_open(&screens::HUD_EDITOR), "cannot open the HUD editor");
    t.screen("core:hud_editor").await?;
    t.frames(4).await?;
    t.shot("editor").await?;
    t.laid_out(&[
        "id:hud:core:hotbar",
        "id:hud:core:crosshair",
        "id:hud:core:chat",
        "id:hud:core:held_name",
        "id:hud:base:health",
        "id:hud:base:food",
        "id:hud_tools",
    ]);
    // A drag takes the hotbar to the top left corner.
    let Rect { x, y, w, h } = rect(&t, "core:hotbar").ok_or("no hotbar in the editor")?;
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(3).await?;
    // Held for seconds: a CI frame (software Vulkan) can take a good part of one, and a 0.4 s
    // press had let go before the cursor moved. Dropped once it lets go.
    t.c().input_click(0, 2.0);
    t.frames(3).await?;
    t.c().input_cursor(0.25, 0.2);
    t.frames(4).await?;
    t.shot("dragging").await?;
    t.secs(2.0).await?;
    let moved = rect(&t, "core:hotbar").ok_or("no hotbar after the drag")?;
    t.check(
        (moved.x + moved.w / 2.0 - 0.25).abs() < 0.02 && (moved.y + moved.h / 2.0 - 0.2).abs() < 0.02,
        format!("the hotbar under the cursor: {moved:?}"),
    );
    let saved = std::fs::read_to_string(&file).unwrap_or_default();
    t.check(saved.contains("core:hotbar = at"), format!("saved: {saved:?}"));
    // The right button hides what is under it.
    let Rect { x, y, w, h } = rect(&t, "core:crosshair").ok_or("no crosshair")?;
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(3).await?;
    t.c().input_click(1, 0.1);
    t.secs(0.3).await?;
    let saved = std::fs::read_to_string(&file).unwrap_or_default();
    t.check(saved.contains("core:crosshair = hidden"), format!("saved: {saved:?}"));
    t.c().menu_close();
    t.no_screen().await?;
    t.frames(3).await?;
    t.shot("arranged").await?;
    t.check(rect(&t, "core:crosshair").is_none(), "the crosshair hidden in the game");
    let kept = rect(&t, "core:hotbar").ok_or("no hotbar in the game")?;
    t.check((kept.x - moved.x).abs() < 0.01, "the hotbar stays where it was put");
    // Reset all brings back the first picture.
    t.c().menu_open(&screens::HUD_EDITOR);
    t.screen("core:hud_editor").await?;
    t.frames(3).await?;
    t.check(t.click_ui("text:Reset all", 0), "reset all");
    t.secs(0.3).await?;
    // Weft asks first now: the question's own Reset all, 90 units left of its Cancel (two 84
    // wide buttons 6 apart; a unit is a twentieth of a button's height). A close only cancelled
    // the question, and the editor stayed open.
    let Rect { x, y, w, h } = t.c().ui_rect("text:Cancel").ok_or("no reset question")?;
    let unit = t
        .c()
        .ui_rect("id:hud_reset_all")
        .ok_or("no Reset all button")?
        .h
        / 20.0;
    let shot = t.shot("reset_question").await?;
    let aspect = shot.w as f32 / shot.h.max(1) as f32;
    t.c()
        .input_cursor(x + w / 2.0 - 90.0 * unit / aspect, y + h / 2.0);
    t.frames(3).await?;
    t.c().input_click(0, 0.0);
    t.secs(0.3).await?;
    t.c().menu_close();
    t.no_screen().await?;
    t.frames(3).await?;
    let back = rect(&t, "core:hotbar").ok_or("no hotbar after the reset")?;
    t.check(
        (back.x - before.x).abs() < 0.001 && (back.y - before.y).abs() < 0.001,
        format!("the hotbar back in place: {back:?} {before:?}"),
    );
    t.check(rect(&t, "core:crosshair").is_some(), "the crosshair back");
    Ok(())
}

/// The debug options (F3+F6) say when each line shows: with F3, always (no F3 needed), never.
pub async fn debug_lines(t: T) -> R {
    t.flat().await?;
    t.debug(false).await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("text:fps").is_none(), "no frame counter without F3");
    t.c().input_key("F3", 0.3);
    t.frames(3).await?;
    t.c().input_key("F6", 0.0);
    t.screen("core:debug_options").await?;
    t.frames(3).await?;
    t.shot("options").await?;
    t.laid_out(&["id:debug_options", "text:Done"]);
    // One press: always. Two: never.
    t.check(t.click_ui("id:line:fps", 0), "the frame counter's row");
    t.until("set to always", Duration::from_secs(2), |t| t.c().ui_rect("text:Always").is_some())
        .await?;
    t.c().menu_close();
    t.no_screen().await?;
    t.secs(0.5).await?;
    t.debug(false).await?;
    t.frames(4).await?;
    t.shot("always").await?;
    t.check(t.c().ui_rect("text:fps").is_some(), "the frame counter without F3 when set to always");
    t.check(t.c().ui_rect("text:XYZ").is_none(), "and nothing else with it");
    t.c().menu_open(&screens::DEBUG_OPTIONS);
    t.screen("core:debug_options").await?;
    t.frames(3).await?;
    t.check(t.click_ui("id:line:fps", 0), "the row again");
    t.until("set to never", Duration::from_secs(2), |t| t.c().ui_rect("text:Never").is_some())
        .await?;
    t.c().menu_close();
    t.no_screen().await?;
    t.debug(true).await?;
    t.frames(4).await?;
    t.check(t.c().ui_rect("text:fps").is_none(), "no frame counter set to never, F3 or not");
    t.check(t.c().ui_rect("text:XYZ").is_some(), "the other lines stay");
    t.debug(false).await?;
    t.c().menu_open(&screens::DEBUG_OPTIONS);
    t.screen("core:debug_options").await?;
    t.frames(3).await?;
    t.click_ui("text:Reset", 0);
    t.until("reset", Duration::from_secs(2), |t| t.c().ui_rect("text:Never").is_none())
        .await?;
    t.c().menu_close();
    t.no_screen().await?;
    Ok(())
}

/// F3 shows its panels inside the window, also in the widest case (Russian at GUI scale 3), the
/// engine's on the left and right halves and a mod's own panel, which its group bit hides.
pub async fn f3(t: T) -> R {
    t.world(5).await?;
    t.idle().await?;
    t.c().set_look(0.0, 0.0);
    t.debug(true).await?;
    t.frames(10).await?;
    t.shot("f3").await?;
    let sides = |t: &T| {
        for q in ["text:XYZ", "text:base:overworld"] {
            t.check(t.c().ui_rect(q).is_some_and(|r| r.x < 0.5), format!("{q} on the left"));
        }
        for q in ["text:calls", "text:1600x900"] {
            t.check(t.c().ui_rect(q).is_some_and(|r| r.x + r.w > 0.5), format!("{q} on the right"));
        }
    };
    t.laid_out(&["text:XYZ", "text:base:overworld", "text:calls", "text:1600x900"]);
    sides(&t);
    t.until("testmod panel shown", Duration::from_secs(2), |t| t.c().ui_rect("text:fall").is_some())
        .await?;
    t.laid_out(&["text:fall"]);
    weft_sdk::settings::DEBUG_GROUPS.set(u32::MAX & !(1 << 5));
    t.until("testmod panel hidden", Duration::from_secs(2), |t| t.c().ui_rect("text:fall").is_none())
        .await?;
    t.check(t.c().ui_rect("text:XYZ").is_some(), "engine panels stay");
    weft_sdk::settings::DEBUG_GROUPS.set(u32::MAX);
    weft_sdk::settings::LANGUAGE.set("ru_ru");
    weft_sdk::settings::GUI_SCALE.set(3);
    t.frames(10).await?;
    t.shot("f3_ru_3").await?;
    t.laid_out(&["text:XYZ", "text:base:overworld", "text:calls", "text:1600x900"]);
    sides(&t);
    weft_sdk::settings::LANGUAGE.set("en_us");
    weft_sdk::settings::GUI_SCALE.set(2);
    t.debug(false).await
}

fn id(name: &str) -> u32 {
    Item::lookup(name).unwrap_or(u32::MAX)
}

/// The 2x2 grid of the inventory screen, driven by clicks on slots like a player would.
pub async fn inventory_crafting(t: T) -> R {
    t.flat().await?;
    t.op("put me 0 oak_log 16").await?;
    t.press("core:inventory");
    t.screen("core:menu").await?;
    // Menu slots: 0-3 grid, 4 result, 5-13 equipment (armor, clothes, cloak), 14-40 main, 41-49 the
    // hotbar (`t.hotbar`, which finds it).
    let (log, planks) = (id("base:oak_log"), id("base:oak_planks"));
    t.click_settled(&t.hotbar(0), 0).await?;
    t.until("logs on the cursor", Duration::from_secs(5), |t| t.c().cursor_stack().item == log)
        .await?;
    t.click_settled("slot:0", 1).await?;
    t.until("planks in the result", Duration::from_secs(5), |t| t.c().menu_slot(4).item == planks)
        .await
        .map_err(|e| {
            let c = t.c();
            // Where the log went: every slot of the menu holding something.
            let filled: Vec<String> = (0..c.menu().map_or(0, |m| m.slots))
                .filter(|&i| !c.menu_slot(i).is_empty())
                .map(|i| format!("{i}:{}x{}", c.menu_slot(i).item, c.menu_slot(i).count))
                .collect();
            let at = c.ui_rect("slot:0");
            format!(
                "{e} (filled {filled:?}, slot:0 at {at:?}, cursor {:?}, grid {:?} {:?} {:?} {:?}, result {:?})",
                c.cursor_stack(),
                c.menu_slot(0),
                c.menu_slot(1),
                c.menu_slot(2),
                c.menu_slot(3),
                c.menu_slot(4)
            )
        })?;
    t.check_eq(t.c().menu_slot(4).count, 4, "planks from one log");
    t.click_settled("slot:0", 1).await?;
    t.until("two logs in the grid", Duration::from_secs(5), |t| t.c().menu_slot(0).count == 2)
        .await?;
    t.click_settled(&t.hotbar(0), 0).await?;
    t.until("cursor empty", Duration::from_secs(5), |t| t.c().cursor_stack().is_empty())
        .await?;
    t.shot("grid").await?;
    t.click_settled("slot:4", 0).await?;
    t.frames(3).await?;
    t.click_settled("slot:4", 0).await?;
    t.until("two batches of planks on the cursor", Duration::from_secs(5), |t| {
        let stack = t.c().cursor_stack();
        stack.item == planks && stack.count == 8
    })
    .await?;
    t.click_settled(&t.hotbar(1), 0).await?;
    t.until("planks in the hotbar", Duration::from_secs(5), |t| t.c().inventory_slot(1).item == planks)
        .await?;
    t.c().menu_close();
    t.no_screen().await?;
    t.check_eq(t.slot("me", 0).await?, ("base:oak_log".into(), 14, 0), "logs left");
    t.check_eq(t.slot("me", 1).await?, ("base:oak_planks".into(), 8, 0), "planks made");
    t.sync_inventory().await
}

pub async fn target_info(t: T) -> R {
    t.flat().await?;
    let f = t.feet();
    let p = super::add(f, [0, 1, -3]);
    for (block, name) in [("stone", "Stone"), ("oak_log", "Oak Log")] {
        t.fill(p, p, block).await?;
        t.look_at(p);
        t.until("target name follows the block", Duration::from_secs(5), |t| t.c().ui_rect(&format!("text:{name}")).is_some())
            .await?;
        let target = t.c().target().ok_or("no target")?;
        t.check_eq(target.pos.to_array(), p, "target uses block coordinates");
        t.check_eq(target.entity, 0, "a block target is not an entity");
        t.laid_out(&["id:hud:base:target_info", "text:Tool:"]);
        let r = t
            .c()
            .ui_rect("id:hud:base:target_info")
            .ok_or("no target panel")?;
        t.check((r.x + r.w / 2.0 - 0.5).abs() < 0.01, "the target panel stays centered as its width changes");
        t.shot(block).await?;
    }
    t.fill(p, p, "air").await?;
    let at = super::add(f, [0, 0, -2]);
    let cow: u64 = t
        .op(&format!("spawn base:cow {} {} {}", at[0], at[1], at[2]))
        .await?
        .parse()
        .map_err(|_| "bad cow id")?;
    t.look_at(at);
    t.until("entity wins the target", Duration::from_secs(5), |t| t.c().target().is_some_and(|a| a.entity == cow))
        .await?;
    t.until("entity name and health shown", Duration::from_secs(5), |t| {
        t.c().ui_rect("text:Cow").is_some() && t.c().ui_rect("text:Health:").is_some()
    })
    .await?;
    let health = t.c().entity(cow).ok_or("cow not visible")?;
    t.check(health.max_health > 0.0 && health.health <= health.max_health, "a newly seen entity has a valid health maximum");
    t.op(&format!("attr_mod {cow} core:max_health testkit:target_health core:set 40 100"))
        .await?;
    t.until("server health maximum reaches the client", Duration::from_secs(5), |t| {
        t.c().entity(cow).is_some_and(|e| e.max_health == 40.0)
    })
    .await?;
    t.until("target health binding updates", Duration::from_secs(5), |t| t.c().ui_rect("text:/ 40").is_some())
        .await?;
    t.shot("entity").await?;
    t.c().set_look(0.0, 80.0);
    t.until("no stale panel after looking away", Duration::from_secs(5), |t| {
        t.c().target().is_none()
            && t.c()
                .ui_rect("id:hud:base:target_info")
                .is_none_or(|r| r.w == 0.0 || r.h == 0.0)
    })
    .await?;
    Ok(())
}

pub async fn creative_inventory(t: T) -> R {
    t.flat().await?;
    t.op("mode creative").await?;
    t.until("creative", Duration::from_secs(5), |t| t.state().mode == 1)
        .await?;
    t.press("core:inventory");
    t.screen("core:menu").await?;
    t.shot("tab").await?;
    let stone = id("base:bricks");
    // Found by searching: the palette scrolls, and base's bricks are below its first rows (the
    // click landed outside the grid's view).
    t.click_settled("id:creative_search", 0).await?;
    t.c().input_type("bricks");
    t.frames(4).await?;
    if t.c().ui_rect(&format!("item:{stone}")).is_some() {
        t.click_settled(&format!("item:{stone}"), 0).await?;
        t.until("bricks on the cursor", Duration::from_secs(5), |t| t.c().cursor_stack().item == stone)
            .await?;
        t.click_ui(&t.hotbar(0), 0);
        t.until("bricks in the hotbar", Duration::from_secs(5), |t| t.c().inventory_slot(0).item == stone)
            .await?;
        // A number key over an entry: a stack of it in that hotbar slot. The entry is read once it
        // has a size: the palette lays out again after the hotbar click, and read in between it
        // was all zeros (under --jobs 2 the pointer went to 0 0, hovering nothing).
        let entry = format!("item:{stone}");
        let mut got = false;
        // Pressed again until it lands (a second press gives the same stack): under load a press
        // was let go before the screen read it.
        for _ in 0..4 {
            t.until("the bricks entry laid out", Duration::from_secs(5), |t| t.c().ui_rect(&entry).is_some_and(|r| r.w > 0.0))
                .await?;
            let Rect { x, y, w, h } = t.c().ui_rect(&entry).ok_or("no bricks entry")?;
            t.c().input_cursor(x + w / 2.0, y + h / 2.0);
            t.frames(3).await?;
            t.press("core:hotbar5");
            got = t
                .until("bricks by the number key", Duration::from_secs(2), |t| t.c().inventory_slot(4).count == 64)
                .await
                .is_ok();
            if got {
                break;
            }
        }
        t.check(got, "bricks by the number key");
    }
    t.c().menu_close();
    t.no_screen().await?;
    t.check_eq(t.slot("me", 0).await?.0, "base:bricks".into(), "creative pick");
    Ok(())
}

/// However much there is in the chat and however big the interface, its line stays on the screen;
/// the wheel goes back through what was said, and `/help` gives a line per command.
pub async fn chat_overflow(t: T) -> R {
    t.world(3).await?;
    for i in 0..40 {
        t.cmd(&format!("/help line{i}")).await?;
    }
    t.cmd("/help").await?;
    t.frames(4).await?;
    weft_sdk::settings::GUI_SCALE.set(4);
    t.frames(2).await?;
    t.press("core:chat");
    t.screen("core:chat").await?;
    t.frames(4).await?;
    let out = async {
        let line = t.c().ui_rect("id:chat").ok_or("no chat line")?;
        t.check(line.y + line.h <= 1.0 && line.y > 0.5, format!("its line at the bottom of the screen: {line:?}"));
        t.shot("full").await?;
        // With the completions up, it stays there.
        t.c().input_type("/setblock 0 0 0 ");
        t.frames(4).await?;
        let shown = t.c().ui_rect("id:chat").ok_or("no chat line")?;
        t.check(shown.y == line.y, format!("it doesn't move for completions: {shown:?}"));
        t.shot("completing").await?;
        for _ in 0.."/setblock 0 0 0 ".len() {
            t.c().input_key("Backspace", 0.0);
            t.frames(1).await?;
        }
        // As narrow as set, and dark behind the text.
        let area = t.c().ui_rect("id:chat_lines").ok_or("no chat lines")?;
        let window = t.state().window[0] as f32 / t.state().gui_scale;
        t.check((area.w - 320.0 / window).abs() < 0.01, format!("as wide as the setting: {area:?}"));
        let beside = t.shot("dark").await?;
        // A spot right of the text, inside the chat and outside it.
        let (right, y) = (area.x + area.w, area.y + area.h / 2.0);
        let (dark, plain) = (beside.avg(right - 0.02, y, right - 0.01, y + 0.01), beside.avg(right + 0.01, y, right + 0.02, y + 0.01));
        t.check(crate::runner::lum(dark) < crate::runner::lum(plain) * 0.8, format!("dark behind it: {dark:?} beside {plain:?}"));
        // The first thing said is out of sight, then scrolled to.
        let area = t.c().ui_rect("id:chat_lines").ok_or("no chat lines")?;
        t.c()
            .input_cursor(area.x + area.w / 2.0, area.y + area.h / 2.0);
        t.frames(1).await?;
        let first = "line39";
        let last = t.c().ui_rect(&format!("text:{first}"));
        for _ in 0..200 {
            t.c().input_scroll(1.0);
            t.frames(1).await?;
        }
        t.frames(2).await?;
        t.check(t.c().ui_rect(&format!("text:{first}")).map(|r| r.y) != last.map(|r| r.y), "the wheel moves what was said");
        t.check(t.c().ui_rect("id:chat").is_some_and(|r| r.y == line.y), "and not its line");
        t.shot("scrolled").await?;
        Ok::<(), String>(())
    }
    .await;
    weft_sdk::settings::GUI_SCALE.set(2);
    t.c().menu_close();
    t.no_screen().await?;
    out?;
    // The settings for it, reached from the settings.
    t.c().menu_open(&screens::SETTINGS);
    t.screen("core:settings").await?;
    t.frames(2).await?;
    t.c().input_cursor(0.5, 0.5);
    t.frames(1).await?;
    t.c().input_scroll(-40.0);
    t.frames(3).await?;
    t.check(t.click_ui("text:Chat", 0), "a button for the chat's settings");
    t.screen("core:chat_settings").await?;
    t.frames(2).await?;
    t.laid_out(&["text:Chat width: 320", "text:Open chat height: 180", "text:Dark behind the chat: 50%", "text:Chat: Shown"]);
    t.shot("settings").await?;
    t.check(t.click_ui("text:Chat: Shown", 0), "a button for who shows in the chat");
    t.until("the next choice on the button", Duration::from_secs(2), |t| t.c().ui_rect("text:Chat: Commands only").is_some())
        .await?;
    weft_sdk::settings::CHAT_VISIBILITY.set(0);
    t.c().menu_close();
    Ok(())
}

/// One way of putting text on the screen: every place and audience, its place's defaults and what
/// a mod can change of them, and the chat's hooks and setting on top.
pub async fn text_on_screen(t: T) -> R {
    t.world(3).await?;
    let seen = |t: &T, query: &str| t.c().ui_rect(query);

    t.op("show near hint a_hint").await?;
    t.until("the hint", Duration::from_secs(5), |t| t.c().ui_rect("text:a_hint").is_some())
        .await?;
    let hint = seen(&t, "text:a_hint").unwrap_or_default();
    t.check(hint.y > 0.7, format!("a hint is above the hotbar, at {hint:?}"));
    t.until("the hint to go", Duration::from_secs(6), |t| t.c().ui_rect("text:a_hint").is_none())
        .await?;
    t.op("show far hint not_here").await?;
    t.secs(0.3).await?;
    t.check(seen(&t, "text:not_here").is_none(), "no hint for who is far away");

    t.op("show dim title a_title").await?;
    t.op("show all subtitle a_subtitle").await?;
    t.until("the title", Duration::from_secs(5), |t| t.c().ui_rect("text:a_title").is_some())
        .await?;
    t.frames(2).await?;
    let (title, sub) = (seen(&t, "text:a_title").ok_or("no title text")?, seen(&t, "text:a_subtitle").ok_or("no subtitle text")?);
    t.check(title.h > sub.h, "a title is bigger than a subtitle");
    t.check(title.y + title.h <= 0.5 && sub.y >= 0.5, format!("the crosshair is between them: {title:?} {sub:?}"));
    t.cmd("/title @a title {#ff5555}Red {#ffffff}and white")
        .await?;
    t.op("show me corner a_corner").await?;
    t.op("show me hint a_hint").await?;
    t.frames(30).await?;
    t.shot("all").await?;
    t.cmd("/title @a clear").await?;
    t.frames(2).await?;
    t.check(seen(&t, "text:a_title").is_none(), "cleared titles are gone");

    t.op("show me corner a_corner").await?;
    t.until("the free text", Duration::from_secs(5), |t| t.c().ui_rect("text:a_corner").is_some())
        .await?;
    let corner = seen(&t, "text:a_corner").unwrap_or_default();
    t.check(corner.x + corner.w > 0.99 && corner.y < 0.01, format!("at(1, 0) is the corner: {corner:?}"));

    let line = |t: &T, i| t.c().chat_line(i);
    t.c().chat("before", &[]);
    t.c().chat("one", &[]).id("testkit:line");
    t.c().chat("after", &[]);
    t.c().chat("two", &[]).id("testkit:line");
    t.frames(2).await?;
    t.check_eq(
        [line(&t, 2), line(&t, 1), line(&t, 0)],
        ["before".to_string(), "two".to_string(), "after".to_string()],
        "a line replaced where it stands",
    );
    // Own times against the place's (a hint 3 s, a chat line 10 s), by script time: the HUD fades
    // by real time, and on CI (software Vulkan) a frame can take up to a second, past 0.2 s.
    t.c().hint("own_hint", &[]).times(0.0, 10.0, 0.0);
    let shown = t
        .until("the hint", Duration::from_secs(3), |t| t.c().ui_rect("text:own_hint").is_some())
        .await
        .is_ok();
    t.check(shown, "the client's own hint");
    t.secs(4.0).await?;
    t.frames(1).await?;
    t.check(seen(&t, "text:own_hint").is_some(), "times are kept (past a hint's 3 s)");

    t.c().chat("own_line", &[]).times(0.0, 3.0, 0.0);
    let shown = t
        .until("the line", Duration::from_secs(3), |t| t.c().ui_rect("text:own_line").is_some())
        .await
        .is_ok();
    t.check(shown, "a line with its own times");
    let gone = t
        .until("the line to go", Duration::from_secs(5), |t| t.c().ui_rect("text:own_line").is_none())
        .await
        .is_ok();
    t.check(gone, "the HUD follows a chat line's times (gone before the chat's 10 s)");

    t.cmd("/tellraw @a {#55ff55}green").await?;
    t.check_eq(line(&t, 0), "green".to_string(), "/tellraw formats");
    t.c().say("{#55ff55}typed");
    t.until("the typed line", Duration::from_secs(5), |t| t.c().chat_line(0).contains("typed"))
        .await?;
    t.check(line(&t, 0).contains("{#55ff55}typed"), "typing doesn't format");

    t.cmd("/rule testkit:chat_hooks 1").await?;
    t.c().say("forbidden");
    t.c().say("allowed");
    t.until("the allowed line", Duration::from_secs(5), |t| t.c().chat_line(0).contains("allowed"))
        .await?;
    t.check_eq(line(&t, 0), "allowed from a mod".to_string(), "chat_format writes the line");
    t.check(!line(&t, 1).contains("forbidden"), "actions::CHAT keeps a line out");
    t.secs(0.2).await?;
    t.check_eq(t.op("last_said").await?, "allowed".to_string(), "actions::CHAT tells what was said");
    t.cmd("/rule testkit:chat_hooks 0").await?;

    weft_sdk::settings::CHAT_VISIBILITY.set(1);
    t.c().say("no_players");
    t.cmd("/tellraw @a system_line").await?;
    let lines: Vec<String> = (0..3).map(|i| line(&t, i)).collect();
    t.check(lines[0] == "system_line" && !lines.iter().any(|l| l.contains("no_players")), format!("only system lines: {lines:?}"));
    weft_sdk::settings::CHAT_VISIBILITY.set(2);
    let reply = t.cmd("/time query").await?;
    t.check(reply.contains("time"), "one's commands still answer in a hidden chat");
    t.c().say("/tellraw @a hidden_system");
    t.secs(0.3).await?;
    t.check(!line(&t, 0).contains("hidden_system"), "a hidden chat shows no system lines");
    t.c().say("still_hidden");
    t.secs(0.3).await?;
    t.check(!line(&t, 0).contains("still_hidden"), "a hidden chat shows no one");
    weft_sdk::settings::CHAT_VISIBILITY.set(0);
    Ok(())
}

/// The chat: `/` opens it with the command list shown; Tab completes items, client commands,
/// word sets and other mods' branches; commands run.
pub async fn chat(t: T) -> R {
    t.world(3).await?;
    t.press("core:command");
    t.screen("core:chat").await?;
    // The list itself (its first ten, alphabetical: with every mod's commands "gamemode" is
    // not among them any more).
    t.until("the command list", Duration::from_secs(2), |t| t.c().ui_rect("class:suggest").is_some())
        .await?;
    t.check_eq(t.c().focused_text(), "/".to_string(), "the line");
    // A mod's key does not fire while the chat has the input.
    let probes = crate::runner::PROBES.with(|p| p.get());
    t.c().input_key("F9", 0.3);
    t.secs(0.5).await?;
    t.check(crate::runner::PROBES.with(|p| p.get()) == probes, "the testkit:probe key fired while typing in the chat");
    t.c().input_key("Backspace", 0.0);
    t.frames(2).await?;
    for (typed, want) in [
        ("/give @p birch_lo", "/give @p base:birch_log"),
        ("/giz", "/gizmo"),
        ("/gizmo cu", "/gizmo cubes"),
        ("/gizmo testm", "/gizmo testmod:energy"),
        ("/gizmo pro", "/gizmo probe"),
        ("/probe al", "/probe alpha"),
        ("/rule mob", "/rule core:mob_spawning"),
    ] {
        t.c().input_type(typed);
        t.frames(2).await?;
        t.c().input_key("Tab", 0.0);
        t.frames(3).await?;
        let text = t.c().focused_text();
        t.check(
            text.starts_with(want) || (typed.starts_with("/give") && text.contains("birch_log")),
            format!("Tab completed {typed} to '{text}', want {want}"),
        );
        if typed.starts_with("/give") {
            t.c().input_key("Enter", 0.0);
            t.no_screen().await?;
            t.until("logs given", Duration::from_secs(5), |t| t.c().inventory_slot(0).item == id("base:birch_log"))
                .await?;
            t.press("core:chat");
            t.screen("core:chat").await?;
            continue;
        }
        for _ in 0..text.chars().count() {
            t.c().input_key("Backspace", 0.0);
            t.frames(1).await?;
        }
    }
    t.c().input_key("Escape", 0.0);
    t.no_screen().await?;
    let reply = t.cmd("/gizmo cubes").await?;
    t.check(reply.contains("cubes on"), format!("/gizmo cubes said '{reply}'"));
    // Any word goes, the list is only for completion (mods' groups).
    let reply = t.cmd("/gizmo testmod:energy").await?;
    t.check(reply.contains("testmod:energy on"), format!("/gizmo testmod:energy said '{reply}'"));
    let reply = t.cmd("/gizmo probe 7").await?;
    t.check_eq(reply, "gizmo probe 7 from testkit".into(), "another mod's branch of /gizmo");
    let help = t.cmd("/help gizmo").await?;
    t.check(help.contains("#core:gizmos") && help.contains("probe"), format!("/help gizmo said '{help}'"));
    let reply = t.cmd("/probe beta").await?;
    t.check_eq(reply, "probe beta on the client".into(), "mod client command");
    let reply = t.cmd("/regions").await?;
    t.check_eq(reply, "regions from testkit".into(), "a mod's command takes the engine's place");
    // What a player types shows as typed, braces included; it can't format.
    t.c().say("braces {#ff0000}stay");
    t.until("the chat line", Duration::from_secs(5), |t| t.c().chat_line(0).contains("stay"))
        .await?;
    t.press("core:chat");
    t.screen("core:chat").await?;
    t.frames(2).await?;
    t.check(t.c().ui_rect("text:{#ff0000}stay").is_some(), "a player's braces show literally in chat");
    t.c().input_key("Escape", 0.0);
    t.no_screen().await?;

    // Replies travel as a key and arguments: the client words them, and again after a language
    // change. The engine's, a mod's and a parse error alike.
    let replies = [
        ("/time query", "time is", "время"),
        ("/feed 20", "stomach filled with 18 units", "желудок заполнен: 18 единиц"),
        ("/give @p base:nothing", "usage: /give", "использование: /give"),
    ];
    for (line, _, _) in replies {
        t.cmd(line).await?;
    }
    let lines = |t: &T| (0..3).rev().map(|i| t.c().chat_line(i)).collect::<Vec<_>>();
    let en = lines(&t);
    weft_sdk::settings::LANGUAGE.set("ru_ru");
    t.frames(3).await?;
    let ru = lines(&t);
    weft_sdk::settings::LANGUAGE.set("en_us");
    for (i, (line, want_en, want_ru)) in replies.into_iter().enumerate() {
        t.check(en[i].contains(want_en), format!("{line} in English: '{}'", en[i]));
        t.check(ru[i].contains(want_ru), format!("{line} in Russian: '{}'", ru[i]));
    }
    Ok(())
}

/// Settings mods declare get a screen built from the declarations; changes apply and are saved.
pub async fn mod_settings(t: T) -> R {
    static HUM: Config<bool> = Config::named("testmod:generator_hum");
    t.c().menu_open(&screens::MOD_SETTINGS);
    t.screen("core:mod_settings").await?;
    t.frames(2).await?;
    t.check(t.click_ui("text:Test Mod", 0), "test mod listed");
    t.screen("core:config").await?;
    t.frames(3).await?;
    t.shot("testmod").await?;
    let before = HUM.get();
    t.check(t.click_ui("text:Generator hum", 0), "hum toggle shown");
    t.until("toggled", std::time::Duration::from_secs(2), |_| HUM.get() != before)
        .await?;
    // A test run writes beside the player's settings, never over them.
    let file = std::fs::read_to_string(format!("{}/test-config/testmod.txt", t.c().run_dir())).unwrap_or_default();
    t.check(file.contains(&format!("generator_hum = {}", !before)), format!("saved: {file:?}"));
    HUM.set(before);
    t.c().menu_close();
    t.c().menu_close();
    // The Sound screen has the volumes and nothing else.
    t.c().menu_open(&screens::SOUND);
    t.screen("core:config").await?;
    t.frames(3).await?;
    t.shot("sound").await?;
    t.laid_out(&["text:Master Volume", "text:Blocks", "text:Done"]);
    let y = |t: &T, q: &str| t.c().ui_rect(q).map_or(0.0, |r| r.y);
    t.check(y(&t, "text:Master Volume") < y(&t, "text:Ambient"), "the master volume comes first");
    for other in ["text:config.core", "text:Field of view", "text:Generator hum"] {
        t.check(t.c().ui_rect(other).is_none(), format!("{other} on the Sound screen"));
    }
    t.c().menu_close();
    Ok(())
}

/// The recipe viewer: R over an item shows what makes it, U what it is used in; tagged
/// ingredients and machines of every mod show up.
pub async fn recipe_viewer(t: T) -> R {
    t.flat().await?;
    t.op("put me 0 iron_ingot 3").await?;
    t.press("core:inventory");
    t.screen("core:menu").await?;
    let Some(Rect { x, y, w, h }) = t.c().ui_rect(&t.hotbar(0)) else { return Err("no hotbar slot".into()) };
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(3).await?;
    t.press("core:recipes");
    t.screen("core:recipes").await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("id:tab:base:smelting").is_some(), "smelting recipes for the ingot");
    t.shot("iron_recipes").await?;
    t.click_ui(&format!("icon:{}", id("base:iron_ingot")), 1);
    t.frames(4).await?;
    t.check(t.c().ui_rect("text:uses").is_some(), "right click opens uses of the displayed item");
    t.c().menu_close();
    t.screen("core:recipes").await?;
    t.c().menu_close();
    t.screen("core:menu").await?;
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(3).await?;
    t.press("core:uses");
    t.screen("core:recipes").await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("text:crafting").is_some(), "crafting uses of the ingot");
    t.shot("iron_uses").await?;
    // One tab per kind; the test mod's alloying shows its info line from the recipe's values.
    t.check(t.c().ui_rect("id:tab:base:crafting").is_some(), "a crafting tab");
    t.c().input_scroll(-1.0);
    t.frames(4).await?;
    t.check(t.c().ui_rect("text:2 /").is_some(), "wheel turns to the next page");
    t.c().input_scroll(1.0);
    t.frames(4).await?;
    t.check(t.c().ui_rect("text:1 /").is_some(), "wheel returns to the previous page");
    t.check(t.click_ui("id:tab:testmod:alloying", 0), "an alloying tab");
    t.secs(0.2).await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("text:32 EU/t").is_some(), "the alloying recipe's energy in its info line");
    t.shot("alloying").await?;
    // Tags: the list, then the items of one.
    t.check(t.click_ui("text:Tags", 0), "a tags button");
    t.screen("core:tags").await?;
    t.frames(3).await?;
    t.click_ui("id:tag_search", 0);
    t.frames(2).await?;
    t.c().input_type("ingots/iron");
    t.frames(4).await?;
    t.check(t.click_ui("id:tag:ingots/iron", 0), "the ingots/iron tag in the list");
    t.screen("core:tag").await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("text:#ingots/iron").is_some(), "the tag's items");
    t.shot("tag").await?;
    t.c().menu_close();
    t.screen("core:tags").await?;
    t.c().menu_close();
    t.screen("core:recipes").await?;
    t.c().menu_close();
    t.screen("core:menu").await?;
    // A kind laid out by its mod, with a fluid.
    t.op("put me 1 diamond 1").await?;
    t.frames(3).await?;
    let Some(Rect { x, y, w, h }) = t.c().ui_rect(&t.hotbar(1)) else { return Err("no hotbar slot".into()) };
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(3).await?;
    t.press("core:uses");
    t.screen("core:recipes").await?;
    t.frames(3).await?;
    t.check(t.click_ui("id:tab:testmod:infusing", 0), "an infusing tab");
    t.secs(0.2).await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("text:laid out by the test mod").is_some(), "the infusing kind's own layout");
    t.shot("infusing").await?;
    t.c().menu_close();
    t.screen("core:menu").await?;
    // The arrow of a menu that crafts opens its recipes.
    let Some(Rect { x, y, w, h }) = t.c().ui_rect("class:arrow") else { return Err("no arrow in the inventory".into()) };
    t.c().input_cursor(x + w / 2.0, y + h / 2.0);
    t.frames(2).await?;
    t.c().input_click(0, 0.05);
    t.screen("core:recipes").await?;
    t.frames(3).await?;
    t.check(
        t.c().ui_rect("text:Recipes").is_some() && t.c().ui_rect("id:tab:base:crafting").is_some(),
        "the arrow shows the crafting recipes",
    );
    // The crafting table opens the crafting menu, so it is the station of its recipes.
    t.check(
        t.c()
            .ui_rect(&format!("icon:{}", id("base:crafting_table")))
            .is_some(),
        "the crafting table shown as the station",
    );
    t.shot("arrow").await?;
    t.c().menu_close();
    t.c().menu_close();
    t.no_screen().await
}

/// Inventory hotkeys: a number key over a slot swaps it with that hotbar slot, a double click
/// gathers matching items onto the cursor, and shift with the left button held moves every
/// stack swept over (mouse tweaks).
pub async fn inventory_hotkeys(t: T) -> R {
    t.flat().await?;
    t.op("put me 9 stone 5").await?;
    t.op("put me 10 dirt 10").await?;
    t.op("put me 11 dirt 7").await?;
    t.op("put me 20 cobblestone 3").await?;
    t.op("put me 21 cobblestone 4").await?;
    t.press("core:inventory");
    t.screen("core:menu").await?;
    // The menu's slot of the player's inventory slot `i` (9..36 the backpack): the backpack comes
    // just before the hotbar, past the grid, the worn slots (clothes among them) and the result.
    let hot: u32 = t
        .hotbar(0)
        .trim_start_matches("slot:")
        .parse()
        .map_err(|_| "no hotbar".to_string())?;
    let menu = |i: u32| hot - 27 + (i - 9);
    let hover = |inv: u32| -> R {
        let slot = menu(inv);
        let Some(Rect { x, y, w, h }) = t.c().ui_rect(&format!("slot:{slot}")) else { return Err(format!("no slot {slot}")) };
        t.c().input_cursor(x + w / 2.0, y + h / 2.0);
        Ok(())
    };
    hover(9)?;
    t.frames(3).await?;
    t.press("core:hotbar3");
    t.secs(0.3).await?;
    t.check_eq(t.slot("me", 2).await?.0, "base:stone".to_string(), "number key 3 moved the stack to hotbar slot 3");
    t.check_eq(t.slot("me", 9).await?.1, 0, "and emptied the slot it was in");

    hover(10)?;
    t.frames(3).await?;
    t.c().input_click(0, 0.0);
    t.frames(4).await?;
    t.c().input_click(0, 0.0);
    t.secs(0.3).await?;
    t.check_eq(t.c().cursor_stack().count, 17, "a double click gathered both dirt stacks");
    t.c().input_click(0, 0.0);
    t.secs(0.3).await?;

    t.c().input_key("ShiftLeft", 1.0);
    hover(20)?;
    t.frames(3).await?;
    t.c().input_click(0, 0.6);
    t.frames(3).await?;
    hover(21)?;
    t.secs(0.8).await?;
    t.check(t.slot("me", 20).await?.1 == 0 && t.slot("me", 21).await?.1 == 0, "shift-sweeping moved both cobblestone stacks to the hotbar");
    t.secs(0.5).await?;

    // The drop keys over a slot: one, then with Ctrl the rest.
    t.op("put me 12 stick 5").await?;
    t.secs(0.2).await?;
    hover(12)?;
    t.frames(3).await?;
    t.press("core:drop");
    t.secs(0.3).await?;
    t.check_eq(t.slot("me", 12).await?.1, 4, "Q over a slot threw one");
    t.press("core:drop_all");
    t.secs(0.3).await?;
    t.check_eq(t.slot("me", 12).await?.1, 0, "Ctrl+Q threw the rest");

    // A stack picked up and the button still held: slots swept over give theirs.
    for s in [13, 14, 15] {
        t.op(&format!("put me {s} sand 1")).await?;
    }
    t.secs(0.2).await?;
    hover(13)?;
    t.frames(3).await?;
    t.c().input_click(0, 1.0);
    t.frames(4).await?;
    hover(14)?;
    t.frames(4).await?;
    hover(15)?;
    t.secs(1.0).await?;
    t.check_eq(t.c().cursor_stack().count, 3, "sweeping on with the stack picked up gathered the sand");
    hover(13)?;
    t.frames(3).await?;
    t.c().input_click(0, 0.0);
    t.secs(0.3).await?;

    // Shift-clicked off with a full hotbar, worn armor goes to the main inventory.
    for s in 0..9 {
        t.op(&format!("put me {s} dirt 1")).await?;
    }
    t.op("put me 37 iron_chestplate 1").await?;
    t.secs(0.3).await?;
    let plate = id("base:iron_chestplate");
    let at = (0..64)
        .find(|&i| t.c().menu_slot(i).item == plate)
        .ok_or("no chestplate in the menu")?;
    t.c().input_key("ShiftLeft", 0.5);
    t.frames(1).await?;
    t.click_ui(&format!("slot:{at}"), 0);
    t.secs(0.4).await?;
    let now = (0..64).find(|&i| t.c().menu_slot(i).item == plate);
    t.check(now.is_some_and(|i| i != at), format!("the chestplate left its slot {at} for {now:?}"));

    // The trash key over a slot: its stack goes to the trash (base's, in the inventory), the
    // next one takes its place.
    let slots = t.c().menu().map_or(0, |m| m.slots);
    t.op("put me 12 stick 3").await?;
    t.op("put me 13 sand 2").await?;
    t.secs(0.3).await?;
    for (slot, item) in [(12, "base:stick"), (13, "base:sand")] {
        hover(slot)?;
        t.frames(3).await?;
        t.press("core:trash");
        t.secs(0.3).await?;
        t.check_eq(t.slot("me", slot).await?.1, 0, "the stack left for the trash");
        let shown: Vec<u32> = (0..slots).map(|i| t.c().menu_slot(i).item).collect();
        t.check(shown.contains(&id(item)), format!("{item} shown in the trash"));
    }
    t.check(!(0..slots).any(|i| t.c().menu_slot(i).item == id("base:stick")), "the sticks gone for the sand");

    t.op("put me 12 stick 5").await?;
    t.until("sticks in inventory", Duration::from_secs(5), |t| t.c().menu_slot(12).count == 5)
        .await?;
    t.click_ui("slot:12", 1);
    t.until("half the sticks on the cursor", Duration::from_secs(5), |t| t.c().cursor_stack().count == 3)
        .await?;
    let trash = slots - 1;
    for count in 1..=2 {
        t.click_ui(&format!("slot:{trash}"), 1);
        t.until("one stick added to trash", Duration::from_secs(5), |t| {
            t.c().menu_slot(trash).item == id("base:stick")
                && t.c().menu_slot(trash).count == count
                && t.c().cursor_stack().count == 3 - count
        })
        .await?;
    }

    t.click_ui("slot:12", 0);
    t.frames(3).await?;
    t.click_ui("slot:12", 0);
    t.until("whole stack picked up again", Duration::from_secs(5), |t| t.c().cursor_stack().count == 3)
        .await?;
    t.check(t.c().menu_slot(12).is_empty(), "the second left click emptied the slot");
    Ok(())
}

/// Rebinding in Controls: the clicked entry waits for a key (Escape cancels and leaves the screen
/// open). A key pressed with Ctrl makes a chord that the key alone does not trigger; a modifier
/// let go of with nothing else pressed is the key itself. A key pressed twice binds a double tap
/// in the second slot, and a right click clears it.
pub async fn rebind(t: T) -> R {
    t.flat().await?;
    // H: K, Weft's, opens base's skills (`mods/base/src/web.rs`), and the screen it opened
    // stopped the walks.
    // `keys`: (key, seconds held, seconds before the next one).
    let bind = async |keys: &[(&str, f32, f32)]| -> R {
        t.open_screen("core:controls", |t| {
            t.c().menu_open(&screens::CONTROLS);
        })
        .await?;
        t.frames(2).await?;
        t.click_ui("id:bind:core:forward:0", 0);
        // A click acts when the button comes up, a frame or two on: by the script clock alone a CI
        // frame (over 0.2 s) let the Escape below land first, nothing waiting, and it closed Controls.
        t.frames(3).await?;
        t.secs(0.2).await?;
        for &(key, held, after) in keys {
            t.c().input_key(key, held);
            t.secs(after).await?;
        }
        // A key pressed once binds when the time for a second press is over.
        t.secs(0.5).await?;
        Ok(())
    };
    let walked = async |keys: &[&str]| -> Result<f64, String> {
        t.c().menu_close();
        t.no_screen().await?;
        let before = t.state().pos;
        for k in keys {
            t.c().input_key(k, 0.6);
        }
        t.secs(0.8).await?;
        let after = t.state().pos;
        Ok((after.x - before.x).hypot(after.z - before.z))
    };
    bind(&[("Escape", 0.0, 0.0)]).await?;
    t.screen("core:controls").await?;
    t.c().menu_close();
    t.no_screen().await?;
    let k = {
        bind(&[("KeyH", 0.0, 0.0)]).await?;
        walked(&["KeyH"]).await?
    };
    let (alone, chord) = {
        bind(&[("ControlLeft", 0.5, 0.1), ("KeyH", 0.0, 0.5)]).await?;
        t.check(t.c().ui_rect("text:Ctrl+H").is_some(), "the entry shows Ctrl+H");
        t.shot("chord").await?;
        (walked(&["KeyH"]).await?, walked(&["ControlLeft", "KeyH"]).await?)
    };
    let shift = {
        bind(&[("ShiftRight", 0.1, 0.3)]).await?;
        walked(&["ShiftRight"]).await?
    };
    bind(&[("KeyW", 0.0, 0.0)]).await?;
    t.click_ui("id:bind:core:forward:1", 0);
    t.frames(3).await?;
    t.secs(0.2).await?;
    t.c().input_key("KeyJ", 0.05);
    t.secs(0.1).await?;
    t.c().input_key("KeyJ", 0.05);
    t.secs(0.5).await?;
    t.check(t.c().ui_rect("text:J*2").is_some(), "a key pressed twice binds a double tap");
    t.click_ui("id:bind:core:forward:1", 1);
    t.frames(4).await?;
    t.check(t.c().ui_rect("text:J*2").is_none(), "a right click clears it");
    t.c().menu_close();
    t.no_screen().await?;
    t.check(k > 1.0, format!("H walks after rebinding forward to it (moved {k:.2})"));
    t.check(alone < 0.2 && chord > 1.0, format!("bound to Ctrl+H, H alone moved {alone:.2} and Ctrl+H {chord:.2}"));
    t.check(shift > 1.0, format!("a right Shift let go of alone is the key (moved {shift:.2})"));
    Ok(())
}

// Restored from dev b5815f09 (Weft v26.11.2 removed them): see notes/crew/upstream/lost-tests.md

async fn drag_packs(t: &T) -> R {
    use weft_sdk::settings::RESOURCE_PACKS;
    t.screen("core:title").await?;
    RESOURCE_PACKS.set("testkit_a,testkit_b,testkit_c");
    t.check(t.c().menu_open(&screens::RESOURCE_PACKS), "cannot open the resource packs");
    t.screen("core:resource_packs").await?;
    t.until("the rows", Duration::from_secs(2), |t| t.c().ui_rect("id:pack_handle:testkit_c").is_some())
        .await?;
    t.shot("three_packs").await?;
    t.laid_out(&["id:pack_handle:testkit_a", "id:pack_handle:testkit_b", "id:pack_handle:testkit_c"]);
    let drag = async |id: &str, x: f32, y: f32, name: &str| -> R {
        let Rect { x: hx, y: hy, w: hw, h: hh } = t
            .c()
            .ui_rect(&format!("id:pack_handle:{id}"))
            .ok_or(format!("no handle of {id}"))?;
        t.c().input_cursor(hx + hw / 2.0, hy + hh / 2.0);
        t.frames(3).await?;
        t.c().input_click(0, 0.4);
        t.frames(3).await?;
        t.c().input_cursor(x, y);
        t.frames(4).await?;
        t.check(t.c().ui_rect("id:pack_ghost").is_some(), format!("the copy of {id} follows the cursor"));
        t.shot(name).await?;
        t.until("the drop", Duration::from_secs(3), |t| t.c().ui_rect("id:pack_ghost").is_none())
            .await
    };
    let Rect { x: ax, y: ay, .. } = t.c().ui_rect("id:pack:testkit_a").ok_or("no row")?;
    drag("testkit_c", ax + 0.02, ay + 0.002, "to_top").await?;
    t.check_eq(RESOURCE_PACKS.get(), "testkit_c,testkit_a,testkit_b".to_string(), "dragged to the top");
    let Rect { x: lx, y: ly, w: lw, h: lh } = t.c().ui_rect("id:packs_available").ok_or("no list")?;
    drag("testkit_a", lx + lw / 2.0, ly + lh / 2.0, "out").await?;
    t.check_eq(RESOURCE_PACKS.get(), "testkit_c,testkit_b".to_string(), "dragged out of the selected");
    let Rect { x: sx, y: sy, w: sw, h: sh } = t.c().ui_rect("id:packs_selected").ok_or("no list")?;
    drag("testkit_a", sx + sw / 2.0, sy + sh - 0.01, "back").await?;
    t.check_eq(RESOURCE_PACKS.get(), "testkit_c,testkit_b,testkit_a".to_string(), "dragged back in, at the bottom");
    Ok(())
}

/// A long list shows a scroll bar; clicking low on its track scrolls the list down.
pub async fn scrollbar(t: T) -> R {
    t.c().menu_open(&screens::CONTROLS);
    t.screen("core:controls").await?;
    t.frames(2).await?;
    let Rect { x, y, w, h } = t
        .c()
        .ui_rect("scrollbar")
        .ok_or("no scroll bar on the controls screen")?;
    let first = t.c().ui_rect("text:Walk forward").ok_or("no first entry")?;
    t.c().input_cursor(x + w / 2.0, y + h * 0.95);
    t.frames(1).await?;
    t.c().input_click(0, 0.05);
    t.frames(4).await?;
    let moved = t.c().ui_rect("text:Walk forward").map_or(-1.0, |r| r.y);
    t.check(moved < first.y - 0.05, format!("the list scrolled (first entry at {:.3}, was {:.3})", moved, first.y));
    t.c().menu_close();
    Ok(())
}

/// Resource packs move by their handles: up and down the list, out of it and back in. Every row
/// keeps its handle inside the window however many packs are on.
pub async fn pack_drag(t: T) -> R {
    let dir = format!("{}/resourcepacks", t.c().run_dir());
    let ids = ["testkit_a", "testkit_b", "testkit_c"];
    for id in ids {
        std::fs::create_dir_all(format!("{dir}/{id}")).map_err(|e| e.to_string())?;
    }
    let out = drag_packs(&t).await;
    weft_sdk::settings::RESOURCE_PACKS.set("");
    for id in ids {
        let _ = std::fs::remove_dir_all(format!("{dir}/{id}"));
    }
    t.c().menu_close();
    out
}

/// A mod's LED switches it for the next start and leaving does nothing else; the restart button
/// shows only with something to bring in.
pub async fn mods_switch(t: T) -> R {
    t.c().menu_open(&screens::MODS);
    t.screen("core:mods").await?;
    t.frames(3).await?;
    t.check(t.c().ui_rect("id:mods_restart").is_none(), "nothing to restart for yet");
    let file = format!("{}/test-config/disabled_mods.txt", t.c().run_dir());
    t.check(t.click_ui("id:mod_led:testmod", 0), "a LED for the mod");
    t.until("the restart button comes", Duration::from_secs(2), |t| t.c().ui_rect("id:mods_restart").is_some())
        .await?;
    t.check(
        std::fs::read_to_string(&file)
            .unwrap_or_default()
            .contains("testmod"),
        "it is written down",
    );
    // The LED is in the mod's row, and clicking it isn't clicking the row.
    let (led, row) = (t.c().ui_rect("id:mod_led:testmod").ok_or("no LED")?, t.c().ui_rect("id:mod:testmod").ok_or("no row")?);
    t.check(
        led.x > row.x && led.x + led.w <= row.x + row.w + 0.001,
        format!("the LED {:?} inside its row {:?}", (led.x, led.y, led.w, led.h), (row.x, row.y, row.w, row.h)),
    );
    t.check(t.c().ui_rect("text:Needs").is_none(), "the row isn't selected by it (only testmod needs something)");
    t.shot("switched").await?;
    t.check(t.click_ui("id:mod_led:testmod", 0), "and back");
    t.until("nothing to restart for again", Duration::from_secs(2), |t| t.c().ui_rect("id:mods_restart").is_none())
        .await?;
    t.c().menu_close();
    t.frames(3).await?;
    t.check(
        !std::fs::read_to_string(&file)
            .unwrap_or_default()
            .contains("testmod"),
        "back on",
    );
    Ok(())
}
