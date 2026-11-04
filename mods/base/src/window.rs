//! A window as on Ronald's UI sheet (the UI redo's windows batch): its title on an orange tab
//! sitting on the frame, more tabs beside it when the window has neighbours (the inventory's
//! stats and advancements), and a close button at the top right. Every base menu screen is one.

use weft_sdk::*;

/// A tab's height and padding, GUI pixels: the font's line (8.5) and two pixels of room above
/// and below it, the same at the sides.
const TAB_H: f32 = 13.0;
const TAB_PAD: f32 = 2.0;

/// A tab beside the title: its label and what a click does (open another screen).
pub type Tab = (&'static str, fn(&ClientView));

/// The title tab, `tabs` beside it, the close button, then `body` in the `panel` with style `s`.
pub fn window(ui: &mut Ui, s: S, tabs: &[Tab], body: impl FnOnce(&mut Ui)) {
    // The tabs stand on the frame: the column's gap of -2 tucks their bottom under its top line.
    ui.column(S::col().stretch().gap(-2.0), |ui| {
        ui.row(S::row().gap(2.0).pad(0.0), |ui| {
            ui.row(S::row().w(6.0), |_| {});
            ui.panel("tab_pressed", S::row().pad(TAB_PAD).h(TAB_H).center(), |ui| ui.title());
            for &(key, open) in tabs {
                ui.button(key, S::default().class("tab").h(TAB_H).pad(TAB_PAD), move |c| open(c));
            }
            ui.spacer();
            // (a button's padding of 0 means the engine's own; half a GUI pixel is none here)
            ui.button("window.base.close", S::default().class("close").size(7.0, 7.0).pad(0.5).tooltip("window.base.close_tip"), |c| {
                c.client().close_screen()
            });
            ui.row(S::row().w(4.0), |_| {});
        });
        ui.panel("panel", s, body);
    });
}
