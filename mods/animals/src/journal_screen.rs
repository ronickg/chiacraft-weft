//! The Field Journal's screen (Ronald, 11-05: "we need a dossier for our animals"; the mockup,
//! `notes/ui/mock/mock_journal.py`, "Yes, build it"): a page per species, its facts from
//! `journal::entries` (what the reader has seen, hunted, tamed, bred and ridden). On the left the
//! animals by family, "???" for one not met; on the right the page: the animal turning (its kind's
//! model), its name and Latin name, chips (weight, when it's active, temper, how it's found),
//! where it lives, then the survival facts, each one not learned yet saying how to learn it.
//! It opens from the inventory's tabs (base's `window::FAMILY` place) and with its key, I; a
//! page first opened says so for a moment (`journal::NEW`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use animals_core::journal::{DEEDS, Fact, Section, Value};
use weft_sdk::*;

use crate::journal::{Entry, NEW, entries};

pub static SCREEN: Screen = Screen::named("journal");
static KEY: Key = Key::named("journal");

/// The page open: its place in `entries`' order.
static OPEN: AtomicUsize = AtomicUsize::new(0);
/// A page just opened for the first time (its species' name key), and since when.
static TOLD: Mutex<Option<(String, Instant)>> = Mutex::new(None);
const TOLD_SECS: f32 = 3.5;
const TOLD_FADE: f32 = 0.6;
/// Where it shows, a share of the window's height down: at the top, clear of a screen's window
/// (at 0.2 it lay on the journal's own tabs).
const TOLD_Y: f32 = 0.04;

/// The list's width and the page's, GUI px; the picture's size.
const LIST_W: f32 = 110.0;
const PAGE_W: f32 = 250.0;
const PICTURE: (f32, f32) = (96.0, 76.0);
/// The labels' column in the facts' rows.
const LABEL_W: f32 = 70.0;
/// The most pieces a joined line has (`journal.animals.join_<n>`).
const JOIN_MAX: usize = 24;

pub fn register(r: &mut Registry) {
    r.key(&KEY, "I").on_press_client(|c| {
        if c.screen().is_empty() {
            c.screen_open(&SCREEN);
        }
    });
    r.client_packet(&NEW, |_, id| {
        let Some(sp) = animals_core::journal::by_id(id) else { return };
        let Some(s) = crate::species::SPECIES.iter().find(|s| s.name == sp.name) else { return };
        *TOLD.lock().unwrap() = Some((format!("entity.animals.{}", s.kind_name()), Instant::now()));
    });
    r.client_frame(|c| {
        let mut told = TOLD.lock().unwrap();
        let Some((name, at)) = told.as_ref() else { return };
        let t = at.elapsed().as_secs_f32();
        if t > TOLD_SECS {
            *told = None;
            return;
        }
        let alpha = ((TOLD_SECS - t) / TOLD_FADE).clamp(0.0, 1.0);
        c.draw_screen_text(0.5, TOLD_Y, "journal.animals.new", &[Arg::Key(name)], 0xffbe4500 | (alpha * 255.0) as u32, 1.0);
    });
    // A tab beside the inventory's own (Stats, Advancements), where base keeps a place for them.
    r.screen_part(&Screen::named("base:inventory"), "tabs", 10, |ui| {
        ui.button("journal.animals.tab", S::default().class("tab").h(13.0).pad(2.0), |c| {
            c.client().screen_open(&SCREEN);
        });
    });
    r.screen(&SCREEN, screen);
}

fn screen(ui: &mut Ui) {
    let all = entries(&ui.view());
    let open = OPEN.load(Ordering::Relaxed).min(all.len().saturating_sub(1));
    framed(ui, |ui| {
        ui.row(S::row().gap(8.0), |ui| {
            list(ui, &all, open);
            if let Some(e) = all.get(open) {
                page(ui, e);
            }
        });
    });
}

/// The window: its title on the orange tab, Stats and Advancements beside it (the inventory's
/// family, as base's windows show them), the close button.
fn framed(ui: &mut Ui, body: impl FnOnce(&mut Ui)) {
    ui.column(S::column().stretch().gap(-1.5), |ui| {
        ui.row(S::row().gap(2.0), |ui| {
            ui.row(S::row().w(6.0), |_| {});
            ui.panel("tab_pressed", S::row().pad(2.0).h(13.0).center(), |ui| {
                ui.text_styled(S::default().no_wrap(), "journal.animals.title", &[])
            });
            for (key, screen) in [("window.base.tab.stats", "base:stats"), ("window.base.tab.advancements", "base:advancements")] {
                ui.button(key, S::default().class("tab").h(13.0).pad(2.0), move |c| {
                    c.client().screen_open(&Screen::named(screen));
                });
            }
            ui.spacer();
            ui.button(
                "window.base.close",
                S::default()
                    .class("close")
                    .size(7.0, 7.0)
                    .pad(0.5)
                    .tooltip("window.base.close_tip"),
                |c| c.client().screen_close(),
            );
            ui.row(S::row().w(4.0), |_| {});
        });
        ui.panel("panel", S::column().pad(7.0).gap(4.0), body);
    });
}

/// The animals, one a line, the open one picked out; how many are met.
fn list(ui: &mut Ui, all: &[Entry], open: usize) {
    let met = all.iter().filter(|e| e.deeds != 0).count();
    ui.column(S::column().w(LIST_W).gap(2.0), |ui| {
        ui.text_styled(S::column().class("title"), "journal.animals.animals", &[]);
        ui.text_args("journal.animals.found", &[Arg::Int(met as i64), Arg::Int(all.len() as i64)]);
        ui.column(S::column().w(LIST_W).h(160.0).gap(1.0).scroll(), |ui| {
            for (i, e) in all.iter().enumerate() {
                let s = S::default()
                    .w(LIST_W - 8.0)
                    .h(12.0)
                    .pad(2.0)
                    .class("category")
                    .selected(i == open);
                let pick = move |c: &ClientView| {
                    OPEN.store(i, Ordering::Relaxed);
                    c.client().rebuild_screen();
                };
                if e.deeds == 0 {
                    ui.button("journal.animals.unknown", s, pick);
                } else {
                    ui.button(&name_key(e), s, pick);
                }
            }
        });
    });
}

/// `entity.animals.<kind>`: the species' name.
fn name_key(e: &Entry) -> String {
    format!("entity.{}", e.kind.replace(':', "."))
}

/// The facts of `section` the reader knows, in page order.
fn in_section(e: &Entry, section: Section) -> impl Iterator<Item = &(Fact, bool)> {
    e.facts.iter().filter(move |(f, known)| f.section == section && *known)
}

fn page(ui: &mut Ui, e: &Entry) {
    let met = e.deeds != 0;
    ui.column(S::column().w(PAGE_W).gap(4.0), |ui| {
        ui.row(S::row().gap(6.0), |ui| {
            // The animal turning, or not met: a question mark where it would be.
            ui.panel("portrait", S::row().size(PICTURE.0, PICTURE.1).center().self_align(abi::ui_align::START), |ui| {
                if met {
                    ui.kind_model(&e.kind, PICTURE.0 - 2.0, PICTURE.1 - 2.0);
                } else {
                    ui.text_styled(S::default().class("dim").scale(2.0), "journal.animals.unknown_mark", &[]);
                }
            });
            ui.column(S::column().gap(2.0).grow().self_align(abi::ui_align::START), |ui| {
                if !met {
                    ui.text_styled(S::column().class("title"), "journal.animals.unknown", &[]);
                    ui.text_styled(S::column().class("dim").max_w(PAGE_W - PICTURE.0 - 8.0), "journal.animals.not_met", &[]);
                    return;
                }
                ui.text_styled(S::column().class("title"), &name_key(e), &[]);
                for (f, _) in in_section(e, Section::Header) {
                    value_text(ui, &f.value, "dim");
                }
                // Chips: two a row.
                let chips: Vec<&Fact> = in_section(e, Section::Chips).map(|(f, _)| f).collect();
                for pair in chips.chunks(2) {
                    ui.row(S::row().gap(2.0), |ui| {
                        for f in pair {
                            ui.panel("chip", S::row().pad(2.0), |ui| value_text(ui, &f.value, ""));
                        }
                    });
                }
                for (f, _) in in_section(e, Section::Lives) {
                    ui.text_styled(S::column().class("title"), &format!("journal.animals.{}", f.name), &[]);
                    ui.column(S::column().max_w(PAGE_W - PICTURE.0 - 8.0), |ui| value_text(ui, &f.value, ""));
                }
            });
        });
        ui.panel("divider", S::row().h(1.0).stretch(), |_| {});
        // The survival facts, the ones not learned yet saying how to learn them.
        ui.column(S::column().gap(2.0).h(96.0).scroll(), |ui| {
            for (f, known) in e.facts.iter().filter(|(f, _)| f.section == Section::Survival) {
                ui.row(S::row().gap(4.0), |ui| {
                    let label = format!("journal.animals.{}", f.name);
                    ui.text_styled(S::default().w(LABEL_W).class(if *known { "title" } else { "dim" }), &label, &[]);
                    ui.column(S::column().max_w(PAGE_W - LABEL_W - 20.0), |ui| {
                        if *known && met {
                            value_text(ui, &f.value, "");
                        } else {
                            let deed = DEEDS.iter().find(|d| d.0 == f.by).map_or("seen", |d| d.1);
                            ui.text_styled(S::default().class("dim"), &format!("journal.animals.learn.{deed}"), &[]);
                        }
                    });
                });
            }
        });
    });
}

/// A fact's value as text of class `class` ("" for the default).
fn value_text(ui: &mut Ui, v: &Value, class: &'static str) {
    let s = if class.is_empty() { S::default() } else { S::default().class(class) };
    match v {
        Value::Text(t) => ui.text_styled(s, "journal.animals.as_is", &[Arg::Text(t)]),
        Value::Key(k) => ui.text_styled(s, k, &[]),
        // An empty list says so (the golden eagle drops nothing: "join_0" showed).
        Value::Keys(ks) if ks.is_empty() => ui.text_styled(s, "journal.animals.none", &[]),
        Value::Names(ns) if ns.is_empty() => ui.text_styled(s, "journal.animals.none", &[]),
        Value::Keys(ks) => {
            let args: Vec<Arg> = joined(ks.iter().map(|k| Arg::Key(k)));
            ui.text_styled(s, &format!("journal.animals.join_{}", args.len()), &args)
        }
        Value::Names(ns) => {
            let args: Vec<Arg> = joined(ns.iter().map(|n| Arg::Name(n)));
            ui.text_styled(s, &format!("journal.animals.join_{}", args.len()), &args)
        }
        Value::Num(v, d, unit) => ui.text_styled(s, unit, &[Arg::Float(*v as f64, *d)]),
        Value::Range(lo, hi, unit) if lo == hi => ui.text_styled(s, unit, &[Arg::Int(*lo as i64)]),
        Value::Range(lo, hi, unit) => {
            let both = format!("{lo}-{hi}");
            ui.text_styled(s, unit, &[Arg::Text(&both)])
        }
    }
}

/// Pieces with ", " between them, at most `JOIN_MAX` in all.
fn joined<'a>(items: impl Iterator<Item = Arg<'a>>) -> Vec<Arg<'a>> {
    let mut out = Vec::new();
    for a in items {
        if out.len() + 2 > JOIN_MAX {
            break;
        }
        if !out.is_empty() {
            out.push(Arg::Text(", "));
        }
        out.push(a);
    }
    out
}
