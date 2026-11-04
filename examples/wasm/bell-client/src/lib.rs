//! The bell's client half (API 1.2, `examples/wasm/bell` is its server half): a HUD badge in the
//! game's kit (`hud_badge`, an `hud_xp` bar with the count inside it, placed with `at`) showing
//! how often the bell has rung, from what the server half sends on `bell:rings`; the key N pings
//! the server half, which answers with the count.

use std::cell::Cell;

use weft_sdk::*;

declare_client_mod!(init: init);

static HUD: HudElement = HudElement::named("rings");
static PING_KEY: Key = Key::named("ping");
static RINGS: Channel<u32> = Channel::named("rings");
static PING: Channel<String> = Channel::named("ping");

/// The bar's size, in GUI pixels.
const BAR: (f32, f32) = (60.0, 7.0);

thread_local! {
    static COUNT: Cell<Option<u32>> = const { Cell::new(None) };
}

fn init(r: &mut Registry) {
    r.hud_element(&HUD, hud_anchor::TOP_RIGHT, 0, |ui| {
        ui.panel("hud_badge", S::col().pad(3.0).gap(2.0), |ui| {
            ui.text_bind(|_, out| match COUNT.with(Cell::get) {
                Some(n) => out.set("bell.hud", &[Arg::Int(n as i64)]),
                None => out.set("bell.hud_none", &[]),
            });
            ui.column(S::col().size(BAR.0, BAR.1), |ui| {
                // Every tenth ring fills the bar; the count sits inside it.
                ui.progress_bind("hud_xp", S::default().size(BAR.0, BAR.1), |_| {
                    COUNT
                        .with(Cell::get)
                        .map_or(-1.0, |n| (n % 10) as f32 / 10.0)
                });
                ui.text_bind_styled(
                    S::default()
                        .at_pivot(BAR.0 / 2.0, BAR.1 / 2.0, [0.5, 0.5])
                        .scale(0.75)
                        .no_wrap(),
                    |_, out| match COUNT.with(Cell::get) {
                        Some(n) => out.set("bell.bar", &[Arg::Int((n % 10) as i64)]),
                        None => out.set("bell.bar", &[Arg::Int(0)]),
                    },
                );
            });
        });
    });
    r.client_packet(&RINGS, |_, n| COUNT.with(|c| c.set(Some(n))));
    r.key(&PING_KEY, "N").on_press(|c| {
        c.send(&PING, &"ping".to_string());
        c.show("bell.pinged", &[]);
    });
}
