//! The profile's achievements (`notes/PLAN.md`, phase 4, slice 9): the credentials worlds signed
//! for this account, by world, each with how sure it is. Opened from Profile.

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label};
use crate::ui::text::Arg;
use crate::ui::{Style, Ui};

#[derive(Default)]
pub struct Achievements;

/// A lang key as itself when the language has it, else the text as written.
fn shown(cx: &BuildCx, s: &str) -> Arg {
    if cx.lang.has(s) { Arg::Key(s.to_string()) } else { Arg::Text(s.to_string()) }
}

impl Screen for Achievements {
    fn name(&self) -> &str {
        "core:achievements"
    }

    fn pauses(&self) -> bool {
        true
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let store = super::achievements::load();
        let session = chiacraft_account::current();
        frame(ui, cx, "achievements.title", |ui| {
            let list = ui.container(
                Style::default()
                    .gap(10.0)
                    .w(400.0)
                    .h(0.0)
                    .grow()
                    .min_h(60.0)
                    .scroll(),
                |ui| {
                    // Who: the account, and its DID.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(380.0), |ui| {
                        let did = session
                            .and_then(|s| s.did())
                            .and_then(|d| d.launcher_id())
                            .and_then(|l| chiacraft_wallet::keys::encode_did(&l));
                        match did {
                            Some(d) => label(ui, cx, "achievements.did", &[Arg::Text(d)], "dim"),
                            None => label(ui, cx, "achievements.no_did", &[], "dim"),
                        };
                        let n = store.credentials.len() as i64;
                        let form = super::plural(&crate::settings::language(), n);
                        let key = format!("achievements.count.{form}");
                        // A language without that form uses its `other`.
                        let key = if cx.lang.has(&key) { key } else { "achievements.count.other".into() };
                        label(ui, cx, &key, &[Arg::Int(n)], "default");
                    });
                    if store.credentials.is_empty() {
                        label(ui, cx, "achievements.none", &[], "dim");
                    }
                    // By world, the newest first within each.
                    let mut worlds: Vec<(String, String)> = Vec::new();
                    for c in &store.credentials {
                        let k = (c.issuer.key.clone(), c.issuer.name.clone());
                        if !worlds.contains(&k) {
                            worlds.push(k);
                        }
                    }
                    for (key, name) in worlds {
                        let mut here: Vec<_> = store
                            .credentials
                            .iter()
                            .filter(|c| c.issuer.key == key && c.issuer.name == name)
                            .collect();
                        here.sort_by_key(|c| std::cmp::Reverse(c.issued));
                        ui.panel("panel", Style::default().pad(8.0).gap(3.0).stretch().w(380.0), |ui| {
                            let first = here[0];
                            let how = if first.self_signed() {
                                "achievements.self"
                            } else {
                                match super::achievements::issuer_checked(first) {
                                    None if first.issuer.did.is_some() => "achievements.checking",
                                    Some(true) => "achievements.owned",
                                    Some(false) => "achievements.unconfirmed",
                                    None => "achievements.no_world_did",
                                }
                            };
                            label(ui, cx, "achievements.world", &[Arg::Text(name.clone()), Arg::Key(how.into())], "default");
                            for c in here {
                                let anchored = c
                                    .anchor
                                    .as_ref()
                                    .map(|a| {
                                        Arg::Msg(crate::ui::text::Msg::new(
                                            "achievements.anchored",
                                            vec![Arg::Text(super::achievements::date(a.day * 86_400))],
                                        ))
                                    })
                                    .unwrap_or(Arg::Text(String::new()));
                                label(
                                    ui,
                                    cx,
                                    "achievements.line",
                                    &[shown(cx, &c.achievement.title), Arg::Text(super::achievements::date(c.issued)), anchored],
                                    "default",
                                );
                                if !c.achievement.description.is_empty() {
                                    label(ui, cx, "achievements.desc", &[shown(cx, &c.achievement.description)], "dim");
                                }
                                if store.revoked(c) {
                                    label(ui, cx, "achievements.withdrawn", &[], "dim");
                                }
                            }
                        });
                    }
                },
            );
            ui.set_id(list, "achievements_list");
            button(ui, cx, "menu.back", &[], 200.0, |c: &mut Ctx| c.requests.push(Request::Close));
        });
    }
}
