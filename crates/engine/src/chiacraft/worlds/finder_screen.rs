//! The finder's screen (`notes/PLAN.md`, phase 5, slice 2): the worlds listed on chain that check
//! out, online first, each with what it says about itself and a Join button. From Multiplayer.

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label};
use crate::ui::text::Arg;
use crate::ui::{Style, Ui};

#[derive(Default)]
pub struct FinderScreen;

impl Screen for FinderScreen {
    fn name(&self) -> &str {
        "core:finder"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        use super::finder;
        let f = finder::snapshot();
        // Open: read again every minute while it stays open (a new listing shows by itself).
        finder::watch();
        if f.phase.0.is_empty() {
            finder::refresh(cx.settings);
        }
        frame(ui, cx, "finder.title", |ui| {
            // On a panel, as the other Multiplayer screens: dim text straight on the title's
            // picture couldn't be read (the UI audit, 11-04).
            let (key, args) = f.phase.clone();
            if !key.is_empty() || f.left_out > 0 {
                ui.panel("panel", Style::default().pad(6.0).gap(2.0).w(400.0), |ui| {
                    if !key.is_empty() {
                        label(ui, cx, &key, &args.into_iter().map(Arg::Text).collect::<Vec<_>>(), "dim");
                    }
                    if f.left_out > 0 {
                        label(ui, cx, "finder.left_out", &[Arg::Int(f.left_out as i64)], "dim");
                    }
                    // Honest about the wait: a listing shows once the chain has it.
                    label(ui, cx, "finder.new_listings", &[], "dim");
                });
            }
            // This game's own listings still on their way to the chain.
            for name in &f.pending {
                ui.panel("panel", Style::default().pad(6.0).gap(2.0).w(400.0), |ui| {
                    label(ui, cx, "finder.name", &[Arg::Text(name.clone())], "default");
                    label(ui, cx, "finder.pending", &[], "dim");
                });
            }
            let mut worlds = f.worlds.clone();
            // Online first, then the longest listed.
            worlds.sort_by_key(|w| (w.status.is_none(), std::cmp::Reverse(w.days())));
            let list = ui.container(
                Style::default()
                    .gap(6.0)
                    .w(420.0)
                    .h(0.0)
                    .grow()
                    .min_h(60.0)
                    .scroll(),
                |ui| {
                    if worlds.is_empty() {
                        ui.panel("panel", Style::default().pad(6.0).w(400.0), |ui| {
                            label(ui, cx, "finder.none", &[], "dim");
                        });
                    }
                    for (i, w) in worlds.iter().enumerate() {
                        ui.panel("panel", Style::default().pad(6.0).gap(2.0).stretch().w(400.0), |ui| {
                            ui.container(Style::row().gap(6.0).stretch(), |ui| {
                                ui.container(Style::default().gap(1.0).stretch(), |ui| {
                                    let name = w
                                        .status
                                        .as_ref()
                                        .map_or_else(|| w.key.chars().take(12).collect(), |s| s.name.clone());
                                    label(ui, cx, "finder.name", &[Arg::Text(name)], "default");
                                    match &w.status {
                                        Some(s) => label(ui, cx, "finder.online", &[Arg::Int(s.players as i64)], "dim"),
                                        None if w.asked => label(ui, cx, "finder.offline", &[], "dim"),
                                        None => label(ui, cx, "finder.asking_world", &[], "dim"),
                                    };
                                    label(
                                        ui,
                                        cx,
                                        "finder.listed",
                                        &[Arg::Int(w.days() as i64), Arg::Text(crate::chiacraft::worlds::listing::txch(w.deposit))],
                                        "dim",
                                    );
                                    // What its rules ask, as the Multiplayer list shows it.
                                    if let Some(s) = &w.status {
                                        for (k, n) in &s.admission {
                                            label(ui, cx, k, &[Arg::Int(*n)], "dim");
                                        }
                                    }
                                });
                                // Only a world that answered: an offline one says so above, and
                                // joining it could only fail ("no addressing information").
                                if w.status.is_some() {
                                    let address = w.key.clone();
                                    let id = button(ui, cx, "multiplayer.join", &[], 60.0, move |c| {
                                        c.requests.push(Request::Join(address.clone()))
                                    });
                                    ui.set_id(id, &format!("finder_join{i}"));
                                }
                            });
                        });
                    }
                },
            );
            ui.set_id(list, "finder_list");
            ui.container(Style::row().gap(6.0), |ui| {
                let id = button(ui, cx, "finder.refresh", &[], 120.0, |c| {
                    super::finder::refresh(c.settings);
                    c.requests.push(Request::Rebuild);
                });
                ui.set_id(id, "finder_refresh");
                button(ui, cx, "menu.back", &[], 120.0, |c: &mut Ctx| c.requests.push(Request::Close));
            });
        });
    }
}
