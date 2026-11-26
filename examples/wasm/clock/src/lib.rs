//! Clock: a personal mod (`docs/sandboxed-mods.md`, "Personal mods"), a client half only: the
//! time of day in a tooltip panel at the top left, and a key (the first personal key slot,
//! Shift+Alt+1 unless rebound) that hides and shows it.

use std::cell::Cell;

use weft_sdk::*;

declare_client_mod!(init: init);

static HUD: HudElement = HudElement::named("time");
static TOGGLE: Key = Key::named("toggle");

thread_local! {
    static HIDDEN: Cell<bool> = const { Cell::new(false) };
}

fn init(r: &mut Registry) {
    r.hud_element(&HUD, hud_anchor::TOP_LEFT, 0, |ui| {
        ui.panel("tooltip", S::row().pad(3.0), |ui| {
            ui.text_bind_styled(S::default().mono().no_wrap(), |c, out| {
                if HIDDEN.with(Cell::get) {
                    out.set("", &[]);
                    return;
                }
                // 0 is 6:00.
                let minutes = (c.day_time() * 60 / 1000 + 6 * 60) % (24 * 60);
                out.set("clock.time", &[Arg::Text(&format!("{:02}", minutes / 60)), Arg::Text(&format!("{:02}", minutes % 60))]);
            });
        });
    });
    r.key(&TOGGLE, "ctrl+T").on_press(|c| {
        let hidden = !HIDDEN.with(Cell::get);
        HIDDEN.with(|h| h.set(hidden));
        c.show(if hidden { "clock.hidden" } else { "clock.shown" }, &[]);
    });
}
