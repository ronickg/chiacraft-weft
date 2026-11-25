//! A window as on Ronald's UI sheet (the UI redo's windows batch): its title on an orange tab
//! sitting on the frame, more tabs beside it when the window has neighbours (the inventory's
//! stats and advancements), and a close button at the top right. Every base menu screen is one.

use weft_sdk::*;

/// A tab's height and padding, GUI pixels: the font's line (8.5) and two pixels of room above
/// and below it, the same at the sides.
const TAB_H: f32 = 13.0;
const TAB_PAD: f32 = 2.0;
/// Room either side of a tab's name (Ronald, 11-25: small buttons' text needed more).
const TAB_PAD_X: f32 = 5.0;

/// A window's padding, GUI pixels: the frame's rail is 4.5 of them, so content sits 7.5 inside
/// it, the margin for slot windows in `UI Style Guide.md` ("Spacing"); 7 left it 2.5 from the
/// rail (Planner 11-24, against Minecraft's, Hytale's and Vintage Story's).
pub const WINDOW_PAD: f32 = 12.0;

/// A tab beside the title: its label and what a click does (open another screen).
pub type Tab = (&'static str, fn(&ClientView));

/// The inventory's family of windows (the inventory, Stats, Advancements): other mods add tabs
/// to them at this place (`r.screen_part(&Menu::named(FAMILY), "tabs", ..)`, a button of class
/// `tab`): the animals' Field Journal. A mod that isn't there adds none.
pub const FAMILY: &str = "base:inventory";

/// The title tab, `tabs` beside it, the close button, then `body` in the `panel` with style `s`.
pub fn window(ui: &mut Ui, s: S, tabs: &[Tab], body: impl FnOnce(&mut Ui)) {
    framed(ui, s, None, tabs, false, body);
}

/// The same for a screen that isn't a menu (stats, advancements): its title from `title`, a
/// translation key.
pub fn window_titled(ui: &mut Ui, s: S, title: &str, tabs: &[Tab], body: impl FnOnce(&mut Ui)) {
    framed(ui, s, Some(title), tabs, false, body);
}

/// A window of the inventory's family (`FAMILY`): other mods' tabs after its own.
pub fn family_window(ui: &mut Ui, s: S, title: Option<&str>, tabs: &[Tab], body: impl FnOnce(&mut Ui)) {
    framed(ui, s, title, tabs, true, body);
}

fn framed(ui: &mut Ui, s: S, title: Option<&str>, tabs: &[Tab], family: bool, body: impl FnOnce(&mut Ui)) {
    // The tabs stand on the frame: the column's gap tucks their bottom 3 UI px under its top
    // line, which leaves an even 12 rows of face round a label's 9 px capitals (kit 2, UI guide D).
    ui.column(S::column().stretch().gap(-1.5), |ui| {
        ui.row(S::row().gap(2.0).pad(0.0), |ui| {
            ui.row(S::row().w(6.0), |_| {});
            ui.panel("tab_pressed", S::row().pad_xy(TAB_PAD_X, TAB_PAD).h(TAB_H).center(), |ui| match title {
                Some(key) => ui.text_styled(S::default().no_wrap(), key, &[]),
                None => ui.title(),
            });
            for &(key, open) in tabs {
                ui.button(key, S::default().class("tab").h(TAB_H).pad_xy(TAB_PAD_X, TAB_PAD), move |c| open(c));
            }
            if family {
                // Beside the window's own, in the same row (a column of them stood a row higher).
                ui.screen_parts_in(FAMILY, "tabs", S::row().gap(2.0));
            }
            ui.spacer();
            // A menu has the engine's X in its corner (`game_screens`); a screen of the family
            // (stats, advancements) its own, the same picture (the bare `close` frame read as a
            // stray square beside the tabs, Planner 11-24).
            if title.is_some() {
                ui.icon_button("base:gui/icon_close", 8.0, 8.0, S::default().tooltip("window.base.close_tip"), |c| c.client().menu_close());
                ui.row(S::row().w(4.0), |_| {});
            }
        });
        ui.panel("panel", s, body);
    });
}
