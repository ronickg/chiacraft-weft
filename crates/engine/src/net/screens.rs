use std::rc::Rc;

use crate::app::{Ctx, Request};
use crate::screens::{BuildCx, Screen, back_button, button, frame, label};
use crate::ui::{Kind, Style, Ui};

#[derive(Default)]
pub struct Multiplayer;

impl Screen for Multiplayer {
    fn name(&self) -> &str {
        "core:multiplayer"
    }

    fn build(&mut self, ui: &mut Ui, cx: &BuildCx) {
        frame(ui, cx, "multiplayer.title", |ui| {
            if cx.in_game && !cx.remote {
                ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(340.0), |ui| {
                    let hosting = super::p2p::HOSTING.lock().is_some();
                    if hosting {
                        label(ui, cx, "multiplayer.your_port", &[crate::ui::text::Arg::Int(crate::settings::server_port() as i64)], "dim");
                    } else {
                        button(ui, cx, "multiplayer.host", &[], 200.0, |c| c.requests.push(Request::Host));
                    }
                });
            }
            ui.panel("panel", Style::default().pad(8.0).gap(4.0).stretch().w(340.0), |ui| {
                label(ui, cx, "multiplayer.ticket", &[], "default");
                ui.container(Style::row().gap(4.0).stretch(), |ui| {
                    let input = ui.add(Kind::input("", 4096), Style::default().h(18.0).w(0.0).grow());
                    ui.set_id(input, "ticket");
                    let copy = crate::mods_screen::icon_button(ui, cx, "button", "icon_copy", "multiplayer.copy_ticket", &[], |c| {
                        let ticket = super::current_ticket().unwrap_or_else(|| super::p2p::player_key(&c.settings.run_dir));
                        if !ticket.is_empty() {
                            crate::clipboard::set(&ticket);
                        }
                    });
                    ui.set_id(copy, "copy_ticket");
                });
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
            back_button(ui, cx, "menu.back", &[], 200.0, |c| c.requests.push(Request::Close));
        });
    }
}
