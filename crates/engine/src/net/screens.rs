use std::rc::Rc;

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label};
use crate::ui::text::Arg;
use crate::ui::{Kind, Style, Ui};

#[derive(Default)]
pub struct Multiplayer;

impl Screen for Multiplayer {
    fn name(&self) -> &str {
        "core:multiplayer"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let ticket = super::current_ticket();
        frame(ui, cx, "multiplayer.title", |ui| {
            if cx.in_game {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(340.0), |ui| {
                    let hosting = super::p2p::HOSTING.lock().is_some();
                    if hosting || ticket.is_some() {
                        label(ui, cx, "multiplayer.your_ticket", &[], "default");
                        ui.text_bound(
                            "dim",
                            Box::new(|_: &Ctx| {
                                let t = super::current_ticket().unwrap_or_else(|| "...".into());
                                let short = if t.len() > 52 { format!("{}...{}", &t[..28], &t[t.len() - 20..]) } else { t };
                                (String::new(), vec![Arg::Text(short)])
                            }),
                        );
                        button(ui, cx, "multiplayer.copy", &[], 120.0, move |_| {
                            if let Some(t) = super::current_ticket() {
                                crate::clipboard::set(&t);
                            }
                        });
                    } else {
                        button(ui, cx, "multiplayer.host", &[], 200.0, |c| c.requests.push(Request::Host));
                    }
                });
            }
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(340.0), |ui| {
                label(ui, cx, "multiplayer.ticket", &[], "default");
                let input = ui.add(Kind::input("", 4096), Style::default().h(18.0).stretch());
                ui.set_id(input, "ticket");
                ui.button(
                    "button",
                    Style::default().min_w(120.0).min_h(20.0).pad_xy(6.0, 4.0),
                    Rc::new(|c: &mut Ctx| {
                        let ticket = c.input_text("ticket").unwrap_or_default();
                        c.requests.push(Request::Join(ticket));
                    }),
                    |ui| {
                        ui.text(cx.t("multiplayer.join", &[]), "button");
                    },
                );
            });
            saved_worlds(ui, cx);
            let waiting = crate::links::snapshot().incoming.len();
            let id = button(ui, cx, "links.open", &[Arg::Text(waiting.to_string())], 200.0, |c| c.requests.push(Request::Open(Box::new(Links))));
            ui.set_id(id, "links");
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}

/// The worlds this player has joined (chiacraft), each with its status, to join again.
fn saved_worlds(ui: &mut Ui, cx: &BuildCx) {
    let run_dir = cx.settings.run_dir.clone();
    let worlds = crate::saved_worlds::load(&run_dir);
    crate::saved_worlds::ping_all(&run_dir, cx.settings);
    ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(340.0), |ui| {
        label(ui, cx, "multiplayer.saved", &[], "default");
        if worlds.is_empty() {
            label(ui, cx, "multiplayer.saved_none", &[], "dim");
        }
        for w in worlds.into_iter().take(8) {
            ui.container(Style::row().gap(6.0).stretch(), |ui| {
                ui.container(Style::col().gap(1.0).stretch(), |ui| {
                    let saved = w.clone();
                    ui.text_bound("default", Box::new(move |_: &Ctx| crate::saved_worlds::title_line(&saved)));
                    let (dir, key) = (run_dir.clone(), w.key.clone());
                    ui.text_bound("dim", Box::new(move |_: &Ctx| crate::saved_worlds::status_line(&dir, &key)));
                });
                let address = w.address.clone();
                let id = button(ui, cx, "multiplayer.join", &[], 60.0, move |c| c.requests.push(Request::Join(address.clone())));
                ui.set_id(id, &format!("join:{}", w.name));
                let key = w.key.clone();
                button(ui, cx, "multiplayer.remove", &[], 60.0, move |c| {
                    crate::saved_worlds::forget(&c.settings.run_dir, &key);
                    c.requests.push(Request::Rebuild);
                });
            });
        }
    });
}

/// What the join in progress is doing (chiacraft): a lang key and its arguments.
pub static JOIN_STATUS: parking_lot::Mutex<(String, Vec<String>)> = parking_lot::Mutex::new((String::new(), Vec::new()));

pub fn set_join_status(key: &str, args: &[String]) {
    *JOIN_STATUS.lock() = (key.to_string(), args.to_vec());
}

/// Shown while joining a friend's world: connecting, downloading its packages, loading them.
pub struct Joining;

impl Screen for Joining {
    fn name(&self) -> &str {
        "core:joining"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        frame(ui, cx, "multiplayer.joining", |ui| {
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(340.0), |ui| {
                let status = ui.text_bound(
                    "default",
                    Box::new(|_: &Ctx| {
                        let (key, args) = JOIN_STATUS.lock().clone();
                        (key, args.into_iter().map(Arg::Text).collect())
                    }),
                );
                ui.set_id(status, "join_status");
            });
            button(ui, cx, "menu.cancel", &[], 200.0, |c| c.requests.push(Request::CancelJoin));
        });
    }
}

/// A package a world needs and this game lacks, as the join sheet shows it.
#[derive(Clone)]
pub struct OfferRow {
    pub id: String,
    pub version: String,
    pub hash: String,
    pub size: u64,
    /// Who made it, if it is signed and the signature checks out.
    pub creator: Option<crate::package_trust::Creator>,
    /// Why a signature it carries doesn't check out.
    pub problem: Option<String>,
    pub trusted: bool,
}

/// Asks before downloading packages the player has not trusted yet (chiacraft): each with its
/// size and who made it (content key fingerprint and account), or that nobody vouches for it.
pub struct JoinSheet {
    pub rows: Vec<OfferRow>,
}

impl Screen for JoinSheet {
    fn name(&self) -> &str {
        "core:join_sheet"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let rows = self.rows.clone();
        frame(ui, cx, "join_sheet.title", |ui| {
            label(ui, cx, "join_sheet.question", &[], "default");
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(380.0), |ui| {
                for r in &rows {
                    label(ui, cx, "join_sheet.package", &[Arg::Text(r.id.clone()), Arg::Text(r.version.clone()), Arg::Text(r.size.div_ceil(1024).to_string())], "default");
                    match (&r.creator, &r.problem) {
                        (Some(c), _) => {
                            let key = if r.trusted { "join_sheet.trusted" } else { "join_sheet.by" };
                            label(ui, cx, key, &[Arg::Text(crate::package_trust::fingerprint(&c.key)), Arg::Text(c.identity.get(..12).unwrap_or(&c.identity).to_string())], "dim");
                        }
                        (None, Some(p)) => {
                            label(ui, cx, "join_sheet.bad_signature", &[Arg::Text(p.clone())], "dim");
                        }
                        (None, None) => {
                            label(ui, cx, "join_sheet.unsigned", &[], "dim");
                        }
                    }
                }
            });
            label(ui, cx, "join_sheet.note", &[], "dim");
            ui.container(Style::row().gap(6.0), |ui| {
                let go = button(ui, cx, "join_sheet.download", &[], 150.0, |c| c.requests.push(Request::JoinConsent(false)));
                ui.set_id(go, "download");
                if rows.iter().any(|r| r.creator.is_some()) {
                    let trust = button(ui, cx, "join_sheet.trust_creators", &[], 220.0, |c| c.requests.push(Request::JoinConsent(true)));
                    ui.set_id(trust, "trust");
                }
            });
            button(ui, cx, "menu.cancel", &[], 150.0, |c| c.requests.push(Request::CancelJoin));
        });
    }
}

/// Links between worlds (chiacraft): ask another world to link with yours, answer requests for
/// your worlds, see and end links.
pub struct Links;

fn short(key: &str) -> String {
    key.chars().take(12).collect()
}

impl Screen for Links {
    fn name(&self) -> &str {
        "core:links"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        crate::links::UNSEEN.store(0, std::sync::atomic::Ordering::Relaxed);
        let store = crate::links::snapshot();
        frame(ui, cx, "links.title", |ui| {
            // Asking: only from one of your own worlds, which is what gets linked.
            if cx.in_game && !cx.remote {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(380.0), |ui| {
                    label(ui, cx, "links.ask", &[], "default");
                    let input = ui.add(Kind::input("", 4096), Style::default().h(18.0).stretch());
                    ui.set_id(input, "link_address");
                    label(ui, cx, "links.note", &[], "dim");
                    let note = ui.add(Kind::input("", 200), Style::default().h(18.0).stretch());
                    ui.set_id(note, "link_note");
                    let id = button(ui, cx, "links.send", &[], 120.0, |c| {
                        let address = c.input_text("link_address").unwrap_or_default();
                        let note = c.input_text("link_note").unwrap_or_default();
                        c.requests.push(Request::AskLink(address, note));
                    });
                    ui.set_id(id, "ask");
                });
            }
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(380.0), |ui| {
                label(ui, cx, "links.incoming", &[], "default");
                if store.incoming.is_empty() {
                    label(ui, cx, "links.none", &[], "dim");
                }
                for (i, inc) in store.incoming.iter().enumerate() {
                    let r = &inc.request;
                    label(ui, cx, "links.request", &[Arg::Text(r.from_name.clone()), Arg::Text(short(&inc.owner)), Arg::Text(inc.to_name.clone())], "default");
                    if !r.body.note.is_empty() {
                        label(ui, cx, "links.says", &[Arg::Text(r.body.note.clone())], "dim");
                    }
                    ui.container(Style::row().gap(4.0), |ui| {
                        let id = r.body.id();
                        let (a, b, d) = (id.clone(), id.clone(), id);
                        let n = button(ui, cx, "links.accept", &[], 80.0, move |c| {
                            if let Err(e) = crate::links::accept(&a, false) {
                                log::warn!("{e}");
                            }
                            c.requests.push(Request::Rebuild);
                        });
                        ui.set_id(n, &format!("accept{i}"));
                        let n = button(ui, cx, "links.always", &[], 140.0, move |c| {
                            if let Err(e) = crate::links::accept(&b, true) {
                                log::warn!("{e}");
                            }
                            c.requests.push(Request::Rebuild);
                        });
                        ui.set_id(n, &format!("always{i}"));
                        let n = button(ui, cx, "links.decline", &[], 80.0, move |c| {
                            crate::links::decline(&d, false);
                            c.requests.push(Request::Rebuild);
                        });
                        ui.set_id(n, &format!("decline{i}"));
                    });
                }
            });
            if !store.outgoing.is_empty() {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(380.0), |ui| {
                    label(ui, cx, "links.outgoing", &[], "default");
                    for o in &store.outgoing {
                        let r = &o.request;
                        let to = if o.to_name.is_empty() { short(&r.body.to) } else { o.to_name.clone() };
                        let queued = store.outbox.iter().any(|q| matches!(&q.message, crate::links::Message::Request(x) if x.body.id() == r.body.id()));
                        ui.container(Style::row().gap(6.0).stretch(), |ui| {
                            label(ui, cx, if queued { "links.sending" } else { "links.waiting" }, &[Arg::Text(r.from_name.clone()), Arg::Text(to)], "default");
                            let id = r.body.id();
                            button(ui, cx, "links.withdraw", &[], 80.0, move |c| {
                                crate::links::withdraw(&id);
                                c.requests.push(Request::Rebuild);
                            });
                        });
                    }
                });
            }
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(380.0), |ui| {
                label(ui, cx, "links.links", &[], "default");
                if store.links.is_empty() {
                    label(ui, cx, "links.no_links", &[], "dim");
                }
                for (i, l) in store.links.iter().enumerate() {
                    ui.container(Style::row().gap(6.0).stretch(), |ui| {
                        label(ui, cx, "links.link", &[Arg::Text(l.from_name.clone()), Arg::Text(l.to_name.clone())], "default");
                        let id = l.id();
                        let n = button(ui, cx, "links.end", &[], 80.0, move |c| {
                            if let Err(e) = crate::links::revoke(&id) {
                                log::warn!("{e}");
                            }
                            c.requests.push(Request::Rebuild);
                        });
                        ui.set_id(n, &format!("end{i}"));
                    });
                }
            });
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}
