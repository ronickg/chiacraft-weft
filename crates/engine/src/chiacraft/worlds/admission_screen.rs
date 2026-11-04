//! The owner's Admission screen (`notes/PLAN.md`, phase 4, slices 1-3): who may come into the
//! world this game runs, probation, bans and the ban lists it follows. It reads the world's files
//! and changes them through the same commands an operator types (`/allow`, `/ban`, `/probation`,
//! `/banlist`), so the server applies every change itself (sending away whoever is banned now).

use std::rc::Rc;

use crate::admission as adm;
use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label, on_off};
use crate::ui::text::Arg;
use crate::ui::{Kind, Style, Ui};

#[derive(Default)]
pub struct Admission;

/// Probation lengths the − and + buttons step through, in hours.
const HOURS: [u32; 8] = [0, 1, 2, 4, 8, 24, 72, 168];

fn run(c: &mut Ctx, line: String) {
    c.requests.push(Request::SendChat(line));
}

fn short(v: &str) -> String {
    if v.chars().count() > 20 { format!("{}…", v.chars().take(16).collect::<String>()) } else { v.to_string() }
}

/// A word a command can take: no spaces.
fn word(v: &str) -> String {
    v.split_whitespace().next().unwrap_or("").to_string()
}

impl Screen for Admission {
    fn name(&self) -> &str {
        "core:admission"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let dir = adm::local_world();
        let rules = dir.as_deref().map(adm::load_rules).unwrap_or_default();
        let bans = dir.as_deref().map(adm::load_bans).unwrap_or_default();
        let now = crate::saves::now_secs();
        frame(ui, cx, "admission.screen.title", |ui| {
            let list = ui.container(
                Style::default()
                    .gap(10.0)
                    .w(420.0)
                    .h(0.0)
                    .grow()
                    .min_h(60.0)
                    .scroll(),
                |ui| {
                    if dir.is_none() || !cx.in_game || cx.remote {
                        label(ui, cx, "admission.screen.not_yours", &[], "dim");
                        return;
                    }
                    // Who may come in, and what newcomers may do.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(400.0), |ui| {
                        label(ui, cx, "admission.screen.who", &[], "default");
                        let only = rules.only_listed;
                        let id = button(ui, cx, "admission.screen.only_listed", &[on_off(only)], 260.0, move |c| {
                            run(c, if only { "/allow everyone" } else { "/allow only" }.into())
                        });
                        ui.set_id(id, "only_listed");
                        let p = rules.probation.clone();
                        let hours = p.hours.ceil() as u32;
                        ui.container(Style::row().gap(4.0), |ui| {
                            label(ui, cx, "admission.screen.probation", &[Arg::Int(hours as i64)], "default");
                            let less = HOURS
                                .iter()
                                .rev()
                                .find(|h| **h < hours)
                                .copied()
                                .unwrap_or(0);
                            let more = HOURS.iter().find(|h| **h > hours).copied().unwrap_or(hours);
                            let id =
                                button(ui, cx, "admission.screen.less", &[], 30.0, move |c| run(c, format!("/probation hours {less}")));
                            ui.set_id(id, "probation_less");
                            let id =
                                button(ui, cx, "admission.screen.more", &[], 30.0, move |c| run(c, format!("/probation hours {more}")));
                            ui.set_id(id, "probation_more");
                        });
                        ui.container(Style::row().gap(4.0), |ui| {
                            let (b, k) = (p.build, p.containers);
                            let id = button(ui, cx, "admission.screen.no_build", &[on_off(b)], 190.0, move |c| {
                                run(c, format!("/probation build {}", if b { "off" } else { "on" }))
                            });
                            ui.set_id(id, "probation_build");
                            let id = button(ui, cx, "admission.screen.no_containers", &[on_off(k)], 190.0, move |c| {
                                run(c, format!("/probation containers {}", if k { "off" } else { "on" }))
                            });
                            ui.set_id(id, "probation_containers");
                        });
                    });
                    // Players here now, and anyone else by name, key or DID.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(400.0), |ui| {
                        label(ui, cx, "admission.screen.players", &[], "default");
                        let me = cx.settings.name();
                        let here: Vec<String> = cx
                            .client
                            .map(|c| c.players.clone())
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|n| *n != me)
                            .collect();
                        if here.is_empty() {
                            label(ui, cx, "admission.screen.nobody_here", &[], "dim");
                        }
                        for (i, name) in here.into_iter().enumerate() {
                            ui.container(Style::row().gap(4.0).stretch(), |ui| {
                                ui.container(Style::default().stretch(), |ui| {
                                    label(ui, cx, "admission.screen.name", &[Arg::Text(name.clone())], "default");
                                });
                                let n = word(&name);
                                let id = button(ui, cx, "admission.screen.ban", &[], 60.0, move |c| {
                                    let reason = c.input_text("admission_reason").unwrap_or_default();
                                    run(c, format!("/ban {n} {reason}"))
                                });
                                ui.set_id(id, &format!("ban{i}"));
                                let n = word(&name);
                                let id = button(ui, cx, "admission.screen.allow", &[], 60.0, move |c| run(c, format!("/allow add {n}")));
                                ui.set_id(id, &format!("allow{i}"));
                                let n = word(&name);
                                let id =
                                    button(ui, cx, "admission.screen.let_off", &[], 100.0, move |c| run(c, format!("/probation end {n}")));
                                ui.set_id(id, &format!("let_off{i}"));
                            });
                        }
                        label(ui, cx, "admission.screen.someone", &[], "dim");
                        let input = ui.add(Kind::input("", 200), Style::default().h(18.0).stretch());
                        ui.set_id(input, "admission_who");
                        label(ui, cx, "admission.screen.reason", &[], "dim");
                        let input = ui.add(Kind::input("", 200), Style::default().h(18.0).stretch());
                        ui.set_id(input, "admission_reason");
                        ui.container(Style::row().gap(4.0), |ui| {
                            let id = button(ui, cx, "admission.screen.ban", &[], 80.0, |c| {
                                let who = word(&c.input_text("admission_who").unwrap_or_default());
                                let reason = c.input_text("admission_reason").unwrap_or_default();
                                if !who.is_empty() {
                                    run(c, format!("/ban {who} {reason}"))
                                }
                            });
                            ui.set_id(id, "ban_someone");
                            let id = button(ui, cx, "admission.screen.allow", &[], 80.0, |c| {
                                let who = word(&c.input_text("admission_who").unwrap_or_default());
                                if !who.is_empty() {
                                    run(c, format!("/allow add {who}"))
                                }
                            });
                            ui.set_id(id, "allow_someone");
                        });
                    });
                    // The bans.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(400.0), |ui| {
                        label(ui, cx, "admission.screen.bans", &[], "default");
                        let active: Vec<_> = bans.bans.iter().filter(|b| b.active(now)).collect();
                        if active.is_empty() {
                            label(ui, cx, "admission.no_bans", &[], "dim");
                        }
                        for (i, b) in active.into_iter().enumerate() {
                            ui.container(Style::row().gap(4.0).stretch(), |ui| {
                                ui.container(Style::default().stretch(), |ui| {
                                    let name = if b.name.is_empty() { short(&b.value) } else { b.name.clone() };
                                    let reason = if b.reason.is_empty() {
                                        Arg::Key("admission.no_reason".into())
                                    } else {
                                        Arg::Text(b.reason.clone())
                                    };
                                    label(
                                        ui,
                                        cx,
                                        "admission.ban_line",
                                        &[Arg::Text(name), Arg::Key(format!("admission.kind.{}", b.kind.name())), reason],
                                        "default",
                                    );
                                    if let Some(u) = b.until {
                                        label(
                                            ui,
                                            cx,
                                            "admission.screen.hours_left",
                                            &[Arg::Int(u.saturating_sub(now).div_ceil(3600) as i64)],
                                            "dim",
                                        );
                                    }
                                });
                                let value = b.value.clone();
                                let id = button(ui, cx, "admission.screen.lift", &[], 60.0, move |c| run(c, format!("/unban {value}")));
                                ui.set_id(id, &format!("lift{i}"));
                            });
                        }
                    });
                    // The allow list.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(400.0), |ui| {
                        label(ui, cx, "admission.screen.allow_list", &[], "default");
                        if rules.allow.is_empty() {
                            label(ui, cx, "admission.screen.empty", &[], "dim");
                        }
                        for (i, a) in rules.allow.iter().enumerate() {
                            ui.container(Style::row().gap(4.0).stretch(), |ui| {
                                ui.container(Style::default().stretch(), |ui| {
                                    let name = if a.name.is_empty() { short(&a.value) } else { a.name.clone() };
                                    label(
                                        ui,
                                        cx,
                                        "admission.allow_line",
                                        &[Arg::Text(name), Arg::Key(format!("admission.kind.{}", a.kind.name()))],
                                        "default",
                                    );
                                });
                                let value = a.value.clone();
                                let id =
                                    button(ui, cx, "admission.screen.remove", &[], 70.0, move |c| run(c, format!("/allow remove {value}")));
                                ui.set_id(id, &format!("unallow{i}"));
                            });
                        }
                    });
                    // Ban lists: followed, and this world's own for others.
                    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(400.0), |ui| {
                        label(ui, cx, "admission.screen.lists", &[], "default");
                        if rules.lists.is_empty() {
                            label(ui, cx, "admission.no_lists", &[], "dim");
                        }
                        for (i, f) in rules.lists.iter().enumerate() {
                            ui.container(Style::row().gap(4.0).stretch(), |ui| {
                                ui.container(Style::default().stretch(), |ui| {
                                    let owners = dir.as_deref().map(adm::owners).unwrap_or_default();
                                    let m = adm::maintainer_of(f, &owners);
                                    match dir
                                        .as_deref()
                                        .zip(m.as_deref())
                                        .and_then(|(d, m)| adm::load_list(d, m))
                                    {
                                        Some(l) => label(
                                            ui,
                                            cx,
                                            "admission.list_line",
                                            &[
                                                Arg::Text(l.name),
                                                Arg::Text(short(&l.maintainer)),
                                                Arg::Int(l.entries.len() as i64),
                                                Arg::Int(l.seq as i64),
                                            ],
                                            "default",
                                        ),
                                        None => label(
                                            ui,
                                            cx,
                                            "admission.list_waiting",
                                            &[Arg::Text(short(f.from.first().unwrap_or(&f.maintainer)))],
                                            "default",
                                        ),
                                    };
                                });
                                let what = f
                                    .from
                                    .first()
                                    .cloned()
                                    .unwrap_or_else(|| f.maintainer.clone());
                                let id = button(ui, cx, "admission.screen.unfollow", &[], 80.0, move |c| {
                                    run(c, format!("/banlist unfollow {what}"))
                                });
                                ui.set_id(id, &format!("unfollow{i}"));
                            });
                        }
                        label(ui, cx, "admission.screen.from", &[], "dim");
                        let input = ui.add(Kind::input("", 4096), Style::default().h(18.0).stretch());
                        ui.set_id(input, "list_from");
                        let id = button(ui, cx, "admission.screen.follow", &[], 120.0, |c| {
                            let from = word(&c.input_text("list_from").unwrap_or_default());
                            if !from.is_empty() {
                                run(c, format!("/banlist follow {from}"))
                            }
                        });
                        ui.set_id(id, "follow");
                        // Publishing: this world's bans for good, signed by this account.
                        let me = chiacraft_account::current()
                            .map(|s| s.identity_hex())
                            .unwrap_or_default();
                        let mine = dir.as_deref().and_then(|d| adm::load_list(d, &me));
                        match &mine {
                            Some(l) => label(
                                ui,
                                cx,
                                "admission.screen.published",
                                &[Arg::Int(l.entries.len() as i64), Arg::Int(l.seq as i64)],
                                "dim",
                            ),
                            None => label(ui, cx, "admission.screen.not_published", &[], "dim"),
                        };
                        ui.container(Style::row().gap(4.0), |ui| {
                            let id = button(ui, cx, "admission.screen.publish", &[], 160.0, |c| run(c, "/banlist publish".into()));
                            ui.set_id(id, "publish");
                            if mine.is_some() {
                                // What someone else types to follow it: this world's address.
                                let line = format!("/banlist follow {}", crate::net::current_ticket().unwrap_or_default());
                                ui.button(
                                    "button",
                                    Style::default().min_w(160.0).min_h(20.0).pad_xy(6.0, 4.0),
                                    Rc::new(move |_: &mut Ctx| {
                                        crate::clipboard::set(&line);
                                    }),
                                    |ui| {
                                        ui.text(cx.t("admission.screen.copy_follow", &[]), "button");
                                    },
                                );
                            }
                        });
                    });
                },
            );
            ui.set_id(list, "admission_list");
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}
