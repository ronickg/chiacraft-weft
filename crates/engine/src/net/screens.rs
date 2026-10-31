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
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
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
