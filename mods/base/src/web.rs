//! The perk web's screen (K, and a tab beside Stats and Advancements): the skills down the side
//! with their levels, and the web (`vitals::perks`) in a well you pan (WASD or the arrows, or a
//! skill's row to jump to its region) and zoom (Z and X, or the buttons). A node shows taken,
//! open (next to one you have, with a point to spend and the skill for it), closed or locked by
//! skill; its tooltip says what it gives, what it needs and why it can't be taken. A click takes
//! it; with an unlearning stone used, refund mode gives nodes back.

use std::sync::Mutex;

use base_api::VITALS;
use base_api::skills::{SKILL_WEB, SkillWeb};
use vitals::perks::{self, Node as PNode, Refusal};
use vitals::skills::{self, SKILLS};
use vitals::web::Node as Kind;
use weft_sdk::*;

pub static SCREEN: Screen = Screen::named("web");
static OPEN: Key = Key::named("web");

/// The well the web is drawn in (GUI pixels).
const VIEW_W: f32 = 340.0;
const VIEW_H: f32 = 236.0;
/// Node sizes at zoom 1 (GUI pixels): small, notable, keystone, the start.
const SMALL: f32 = 8.0;
const NOTABLE: f32 = 12.0;
const KEYSTONE: f32 = 16.0;
const START: f32 = 12.0;
/// A link's dots: their size and the gap between them, at zoom 1.
const DOT: f32 = 2.0;
const DOT_STEP: f32 = 5.0;
/// The zoom steps; the screen opens at `ZOOM_START`.
const ZOOMS: [f32; 6] = [0.4, 0.55, 0.75, 1.0, 1.4, 2.0];
const ZOOM_START: usize = 1;
/// GUI pixels a pan key moves the view, at zoom 1.
const PAN_STEP: f32 = 40.0;
/// The sidebar's width.
const SIDE_W: f32 = 118.0;

#[derive(Clone, Copy)]
struct View {
    /// The web's point at the well's middle (GUI pixels at zoom 1).
    centre: (f32, f32),
    zoom: usize,
    refund: bool,
    /// What the screen was last built from (taken, levels, points, refunds).
    built: u64,
}

static VIEW: Mutex<View> = Mutex::new(View { centre: (0.0, 0.0), zoom: ZOOM_START, refund: false, built: 0 });

fn view() -> View {
    *VIEW.lock().unwrap()
}

fn change(c: &ClientView, f: impl FnOnce(&mut View)) {
    f(&mut VIEW.lock().unwrap());
    c.client().rebuild_screen();
}

/// Perk points the local player has.
fn points(c: &ClientView, s: &SkillWeb) -> u32 {
    let level = c
        .component(&VITALS)
        .map_or(1, |v| vitals::levels::level_of(v.xp).0);
    crate::skills::points_of(level, perks::count(&s.taken))
}

/// A hash of what the screen shows, so it's rebuilt when it changes.
fn signature(c: &ClientView) -> u64 {
    let Some(s) = c.component(&SKILL_WEB) else { return 0 };
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |x: u64| h = (h ^ x).wrapping_mul(0x100_0000_01b3);
    s.taken.iter().for_each(|&w| mix(w));
    s.levels().iter().for_each(|&l| mix(l as u64));
    mix(points(c, &s) as u64);
    mix(s.refunds as u64);
    h
}

pub fn register(r: &mut Registry) {
    r.key(&OPEN, "K").on_press_client(|c| {
        c.screen_open(&SCREEN);
    });
    // Rebuilt when a node is taken, a skill rises or a point comes (the server's answer).
    r.client_frame(|c| {
        if c.screen() != "base:web" {
            return;
        }
        let sig = signature(&c.view());
        let mut v = VIEW.lock().unwrap();
        if sig != v.built {
            v.built = sig;
            drop(v);
            c.rebuild_screen();
        }
    });
    // A tab beside the inventory's own (Stats, Advancements).
    r.screen_part(&Screen::named(crate::window::FAMILY), "tabs", 5, |ui| {
        ui.button("web.base.tab", S::default().class("tab").h(13.0).pad(2.0), |c| {
            c.client().screen_open(&SCREEN);
        });
    });
    r.screen(&SCREEN, screen).on_key(|c, key| {
        let step = PAN_STEP / ZOOMS[view().zoom];
        let pan = match key {
            "KeyW" | "ArrowUp" => Some((0.0, -step)),
            "KeyS" | "ArrowDown" => Some((0.0, step)),
            "KeyA" | "ArrowLeft" => Some((-step, 0.0)),
            "KeyD" | "ArrowRight" => Some((step, 0.0)),
            _ => None,
        };
        if let Some((dx, dy)) = pan {
            change(c, |v| v.centre = (v.centre.0 + dx, v.centre.1 + dy));
        }
        match key {
            "KeyZ" | "Minus" => change(c, |v| v.zoom = v.zoom.saturating_sub(1)),
            "KeyX" | "Equal" => change(c, |v| v.zoom = (v.zoom + 1).min(ZOOMS.len() - 1)),
            "KeyC" | "Home" => change(c, |v| v.centre = (0.0, 0.0)),
            _ => {}
        }
    });
}

fn screen(ui: &mut Ui) {
    let c = ui.view();
    let s = c.component(&SKILL_WEB).unwrap_or_default();
    let points = points(&c, &s);
    let mut v = view();
    if s.refunds == 0 {
        v.refund = false;
    }
    let tabs: [crate::window::Tab; 2] = [
        ("window.base.tab.stats", |c| {
            c.client().screen_open(&crate::levels::SCREEN);
        }),
        ("window.base.tab.advancements", |c| {
            c.client().screen_open(&crate::advancements::SCREEN);
        }),
    ];
    crate::window::window_titled(ui, S::row().pad(6.0).gap(6.0), "web.base.title", &tabs, |ui| {
        sidebar(ui, &s, points, v.refund);
        ui.column(S::column().gap(3.0), |ui| {
            ui.row(S::row().gap(3.0).center(), |ui| {
                ui.text_styled(S::default().class("dim").no_wrap(), "web.base.keys", &[]);
                ui.spacer();
                ui.button(
                    "web.base.zoom_out",
                    S::default()
                        .size(12.0, 12.0)
                        .tooltip("web.base.zoom_out_tip"),
                    |c| change(c, |v| v.zoom = v.zoom.saturating_sub(1)),
                );
                ui.button(
                    "web.base.zoom_in",
                    S::default()
                        .size(12.0, 12.0)
                        .tooltip("web.base.zoom_in_tip"),
                    |c| change(c, |v| v.zoom = (v.zoom + 1).min(ZOOMS.len() - 1)),
                );
                ui.button("web.base.centre", S::default().h(12.0).pad(2.0).tooltip("web.base.centre_tip"), |c| {
                    change(c, |v| v.centre = (0.0, 0.0))
                });
            });
            ui.panel("well", S::column().size(VIEW_W, VIEW_H), |ui| {
                web(ui, &s, points, &v);
            });
        });
    });
}

/// The level, points, refunds and each skill with its level and progress; a row's click centres
/// the web on that skill's region.
fn sidebar(ui: &mut Ui, s: &SkillWeb, points: u32, refund: bool) {
    ui.column(S::column().gap(2.0).w(SIDE_W), |ui| {
        ui.panel("chip_points", S::row().pad(2.0), |ui| {
            ui.text_styled(S::default().no_wrap(), "web.base.points", &[Arg::Int(points as i64)]);
        });
        if s.refunds > 0 {
            ui.button_args(
                if refund { "web.base.refund_on" } else { "web.base.refund_off" },
                &[Arg::Int(s.refunds as i64)],
                S::default()
                    .h(12.0)
                    .pad(2.0)
                    .selected(refund)
                    .tooltip("web.base.refund_tip"),
                |c| change(c, |v| v.refund = !v.refund),
            );
        }
        ui.column(S::column().gap(1.0).scroll().h(VIEW_H - 4.0), |ui| {
            for (i, sk) in SKILLS.iter().enumerate() {
                let (level, into) = skills::level_of(s.em[i]);
                let name = format!("web.base.skill.{}", sk.id());
                let by = if sk.by().is_empty() { "web.base.by.later".to_string() } else { format!("web.base.by.{}", sk.id()) };
                let region = perks::web()
                    .node(perks::id(*sk, 6))
                    .map(|n| n.pos())
                    .unwrap_or((0.0, 0.0));
                ui.button_with(
                    S::column()
                        .class("list_row")
                        .pad(1.5)
                        .gap(1.0)
                        .w(SIDE_W - 6.0)
                        .tooltip_args("web.base.skill_tip", &[Arg::Key(&name), Arg::Key(&by)]),
                    move |c| change(c, |v| v.centre = region),
                    |ui| {
                        ui.row(S::row().gap(2.0), |ui| {
                            ui.text_styled(S::default().no_wrap(), &name, &[]);
                            ui.spacer();
                            ui.text_styled(S::default().no_wrap().mono(), "web.base.level", &[Arg::Int(level as i64)]);
                        });
                        ui.progress_bind("stat_xp", S::default().size(SIDE_W - 10.0, 2.0), move |_| into);
                    },
                );
            }
        });
    });
}

/// Where a web point lands in the well, and how big things are, at this view.
fn place(v: &View, p: (f32, f32)) -> (f32, f32) {
    let z = ZOOMS[v.zoom];
    ((p.0 - v.centre.0) * z + VIEW_W / 2.0, (p.1 - v.centre.1) * z + VIEW_H / 2.0)
}

fn inside(p: (f32, f32), margin: f32) -> bool {
    p.0 >= margin && p.1 >= margin && p.0 <= VIEW_W - margin && p.1 <= VIEW_H - margin
}

/// How a node stands for the local player.
#[derive(Clone, Copy, PartialEq)]
enum State {
    Taken,
    /// Can be taken now.
    Open,
    /// Next to one taken and open by skill, but no point.
    Waiting,
    /// Not next to one taken.
    Far,
    /// Its skill is too low.
    Locked,
}

impl State {
    fn suffix(self) -> &'static str {
        match self {
            State::Taken => "taken",
            State::Open | State::Waiting => "open",
            State::Far => "far",
            State::Locked => "locked",
        }
    }
    fn key(self) -> &'static str {
        match self {
            State::Taken => "web.base.state.taken",
            State::Open => "web.base.state.open",
            State::Waiting => "web.base.state.waiting",
            State::Far => "web.base.state.far",
            State::Locked => "web.base.state.locked",
        }
    }
}

fn state(s: &SkillWeb, n: &PNode, points: u32) -> State {
    if perks::has(&s.taken, n.id) {
        return State::Taken;
    }
    match perks::can_take(&s.taken, n.id, &s.levels(), points.max(1)) {
        Ok(()) if points == 0 => State::Waiting,
        Ok(()) => State::Open,
        Err(Refusal::Skill) => State::Locked,
        Err(_) => State::Far,
    }
}

fn web(ui: &mut Ui, s: &SkillWeb, points: u32, v: &View) {
    let web = perks::web();
    let z = ZOOMS[v.zoom];
    // Links first, under the nodes: a row of dots, lit where both ends are taken.
    let dot = (DOT * z.max(0.75)).round().max(1.0);
    for &(a, b) in &web.links {
        let (Some(na), Some(nb)) = (web.node(a), web.node(b)) else { continue };
        let (pa, pb) = (place(v, na.pos()), place(v, nb.pos()));
        if !inside(pa, -60.0) && !inside(pb, -60.0) {
            continue;
        }
        let lit = perks::has(&s.taken, a) && perks::has(&s.taken, b);
        let class = if lit { "perk_link_taken" } else { "perk_link" };
        let len = (pb.0 - pa.0).hypot(pb.1 - pa.1);
        let steps = (len / (DOT_STEP * z.max(0.6))).floor().max(1.0) as u32;
        for k in 1..steps {
            let t = k as f32 / steps as f32;
            let p = (pa.0 + (pb.0 - pa.0) * t, pa.1 + (pb.1 - pa.1) * t);
            if inside(p, dot) {
                ui.panel(class, S::column().size(dot, dot).at_pivot(p.0, p.1, [0.5, 0.5]), |_| {});
            }
        }
    }
    for n in &web.nodes {
        let p = place(v, n.pos());
        let (kind, size) = match n.kind {
            _ if n.id == perks::START => ("start", START),
            Kind::Small => ("small", SMALL),
            Kind::Notable => ("notable", NOTABLE),
            Kind::Keystone => ("keystone", KEYSTONE),
        };
        let d = (size * z.max(0.6)).round();
        if !inside(p, d / 2.0) {
            continue;
        }
        let st = if n.id == perks::START { State::Taken } else { state(s, n, points) };
        let class: &'static str = Box::leak(format!("perk_{kind}_{}", st.suffix()).into_boxed_str());
        let (tip, args) = tooltip(n, st, s);
        let args: Vec<Arg> = args.iter().map(|a| a.as_arg()).collect();
        let id = n.id;
        let refund = v.refund;
        ui.button_with(
            S::default()
                .class(class)
                .size(d, d)
                .at_pivot(p.0, p.1, [0.5, 0.5])
                .tooltip_args(&tip, &args),
            move |c| {
                if refund {
                    c.client().send(&crate::skills::REFUND, &(id as u32));
                } else if st == State::Open {
                    c.client().send(&crate::skills::TAKE, &(id as u32));
                }
            },
            |_| {},
        );
    }
}

/// An owned tooltip argument.
enum A {
    Key(String),
    Text(String),
    Int(i64),
}

impl A {
    fn as_arg(&self) -> Arg<'_> {
        match self {
            A::Key(k) => Arg::Key(k),
            A::Text(t) => Arg::Text(t),
            A::Int(i) => Arg::Int(*i),
        }
    }
}

/// A node's tooltip: its name, what it gives (one or two lines), what it needs, and its state.
fn tooltip(n: &PNode, st: State, s: &SkillWeb) -> (String, Vec<A>) {
    if n.id == perks::START {
        return ("web.base.tip.start".into(), vec![]);
    }
    let mut args = vec![A::Key(format!("perk.base.{}", n.key))];
    for &(f, units) in n.gives.iter().take(2) {
        let pct = (units * vitals::web::UNIT * 100.0).round() as i64;
        args.push(A::Text(format!("{pct:+} %")));
        args.push(A::Key(format!("perk.base.family.{}", f.id())));
    }
    let gives = n.gives.len().min(2);
    let mut key = format!("web.base.tip.{gives}");
    if let Some(sp) = n.special {
        args.push(A::Key(format!("perk.base.special.{}", sp.id())));
        key.push('s');
    }
    if n.needs > 0 {
        let region = n
            .region
            .map_or(String::new(), |r| format!("web.base.skill.{}", r.id()));
        args.push(A::Key(region));
        if let Some(b) = n.bridge {
            args.push(A::Key(format!("web.base.skill.{}", b.id())));
            key.push('b');
        } else {
            key.push('n');
        }
        args.push(A::Int(n.needs as i64));
        // The player's level in it, so a locked node says how far.
        let have = n
            .region
            .map_or(0, |r| s.level(r))
            .min(n.bridge.map_or(u32::MAX, |b| s.level(b)));
        args.push(A::Int(have as i64));
    }
    args.push(A::Key(st.key().into()));
    (key, args)
}
