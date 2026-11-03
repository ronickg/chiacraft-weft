//! The Account screen (chiacraft plan, phase 1): the account behind the player's key, its 24
//! words and restoring another account from words. Opened from Profile.
//!
//! The words are read from the account's store only while they are on screen, are never copied
//! to the clipboard, logged or sent, and the screen has no copy button.

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, button, frame, label};
use crate::ui::text::Arg;
use crate::ui::{Kind, Style, Ui};

#[derive(Default)]
pub struct Account {
    /// The words are on screen (until Hide, or the screen closes).
    show_words: bool,
    /// What the last action said: a lang key and its one argument.
    note: Option<(&'static str, String)>,
}

/// Opens this screen's own state from one of its buttons, then rebuilds it.
fn update(c: &mut Ctx, f: impl FnOnce(&mut Account)) {
    if let Some(a) = c.screen::<Account>() {
        f(a);
    }
    c.requests.push(Request::Rebuild);
}

/// The start and end of a long hex key, for reading at a glance.
fn short(hex: &str) -> String {
    if hex.len() > 20 { format!("{}…{}", &hex[..10], &hex[hex.len() - 8..]) } else { hex.to_string() }
}

impl Screen for Account {
    fn name(&self) -> &str {
        "core:account"
    }

    fn pauses(&self) -> bool {
        true
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        let session = chiacraft_account::current();
        let show_words = self.show_words;
        let note = self.note.clone();
        frame(ui, cx, "account.title", |ui| {
            let Some(s) = session else {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(360.0), |ui| {
                    label(ui, cx, "account.none", &[], "default");
                });
                button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
                return;
            };
            let info = s.fresh_info();
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(360.0), |ui| {
                label(ui, cx, "account.fingerprint", &[Arg::Int(info.fingerprint as i64)], "default");
                label(ui, cx, "account.identity", &[Arg::Text(short(&info.identity))], "dim");
                let store = if info.store == "os" { "account.store_os" } else { "account.store_file" };
                label(ui, cx, store, &[], "dim");
                if !info.backed_up {
                    label(ui, cx, "account.backup_needed", &[], "default");
                }
            });

            if show_words {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(360.0), |ui| match s.words() {
                    Ok(words) => {
                        label(ui, cx, "account.words_warning", &[], "dim");
                        for (r, row) in words.expose().chunks(4).enumerate() {
                            ui.container(Style::row().gap(8.0), |ui| {
                                for (i, w) in row.iter().enumerate() {
                                    label(ui, cx, "account.word", &[Arg::Int((r * 4 + i + 1) as i64), Arg::Text(w.clone())], "default");
                                }
                            });
                        }
                    }
                    Err(e) => {
                        label(ui, cx, "account.error", &[Arg::Text(e)], "default");
                    }
                });
                ui.container(Style::row().gap(6.0), |ui| {
                    button(ui, cx, "account.hide_words", &[], 140.0, |c| update(c, |a| a.show_words = false));
                    if !info.backed_up {
                        button(ui, cx, "account.wrote_them_down", &[], 180.0, |c| {
                            let result = chiacraft_account::current().map(|s| s.mark_backed_up());
                            update(c, |a| {
                                a.show_words = false;
                                a.note = Some(match result {
                                    Some(Ok(())) => ("account.backed_up", String::new()),
                                    Some(Err(e)) => ("account.error", e),
                                    None => ("account.none", String::new()),
                                });
                            });
                        });
                    }
                });
            } else {
                button(ui, cx, "account.show_words", &[], 200.0, |c| update(c, |a| a.show_words = true));
            }

            ui.panel("panel", Style::default().pad(8.0).gap(4.0).w(360.0), |ui| {
                label(ui, cx, "account.restore_title", &[], "default");
                label(ui, cx, "account.restore_hint", &[], "dim");
                let input = ui.add(Kind::input("", 400), Style::default().h(18.0).w(344.0));
                ui.set_id(input, "restore_words");
                let restore = button(ui, cx, "account.restore", &[], 160.0, |c| {
                    let words = c.input_text("restore_words").unwrap_or_default();
                    let result = chiacraft_account::current().map(|s| s.restore(&words));
                    update(c, |a| {
                        a.note = Some(match result {
                            Some(Ok(fingerprint)) => ("account.restored", fingerprint.to_string()),
                            Some(Err(e)) => ("account.error", e),
                            None => ("account.none", String::new()),
                        });
                    });
                });
                ui.set_id(restore, "restore");
            });

            if let Some((key, arg)) = &note {
                label(ui, cx, key, &[Arg::Text(arg.clone())], "default");
            }
            button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}
