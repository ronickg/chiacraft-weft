//! The perk web's screen (K, and a tab beside Stats and Advancements): the skills down the side
//! with their levels, and the web (`vitals::perks`) in a well you pan (WASD or the arrows, or a
//! skill's row to jump to its region) and zoom (Z and X, or the buttons). A node shows taken,
//! open (next to one you have, with a point to spend and the skill for it), closed or locked by
//! skill; its tooltip says what it gives, what it needs and why it can't be taken. A click takes
//! it; with an unlearning stone used, refund mode gives nodes back.

use std::sync::Mutex;

use base_api::VITALS;
use base_api::skills::{SKILL_WEB, STONE, SkillWeb, StoneKind};
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
const SOCKET: f32 = 12.0;
/// The stone picker's row under the well: its icons' size (32 px items at 1x).
const PICK_ICON: f32 = 16.0;
/// A link's dots: their size and the gap between them (GUI pixels).
const DOT: f32 = 1.0;
const DOT_STEP: f32 = 3.0;
/// The zoom steps; the screen opens at `ZOOM_START`. Zoom spreads the web; the nodes keep their
/// size (the UI guide's one pixel grid: art only at whole multiples), doubled at the closest step.
const ZOOMS: [f32; 5] = [0.55, 0.75, 1.0, 1.4, 2.0];
const ZOOM_START: usize = 1;
/// The zoom from which nodes and links are drawn at twice their size.
const ZOOM_DOUBLE: f32 = 2.0;
/// GUI pixels a pan key moves the view, at zoom 1.
const PAN_STEP: f32 = 40.0;
/// The sidebar's width, and a skill row's: the list is the sidebar's width and its scrollbar draws
/// over its right edge, so a row stops short of it with air to spare (the levels were under it).
const SIDE_W: f32 = 118.0;
const ROW_W: f32 = SIDE_W - 16.0;

#[derive(Clone, Copy)]
struct View {
    /// The web's point at the well's middle (GUI pixels at zoom 1).
    centre: (f32, f32),
    zoom: usize,
    refund: bool,
    /// The taken socket a stone is being chosen for.
    picking: Option<u16>,
    /// What the screen was last built from (taken, levels, points, refunds, stones).
    built: u64,
}

static VIEW: Mutex<View> = Mutex::new(View { centre: (0.0, 0.0), zoom: ZOOM_START, refund: false, picking: None, built: 0 });

/// Inventory slots a player has (hotbar, the main rows, armour and the off hand), for the picker.
const INVENTORY_SLOTS: u32 = 46;

fn view() -> View {
    *VIEW.lock().unwrap()
}

fn change(c: &ClientView, f: impl FnOnce(&mut View)) {
    f(&mut VIEW.lock().unwrap());
    c.client().rebuild_screen();
}

/// Opens on the island of the player's highest skill (Melee before any), at the opening zoom,
/// where its rings read (Ronald, 11-05: "A, with clearer islands"); C goes back to the hub.
fn open_on_home(c: &ClientView) {
    let s = c.component(&SKILL_WEB).unwrap_or_default();
    let best = SKILLS
        .iter()
        .copied()
        .max_by_key(|sk| (s.level(*sk), std::cmp::Reverse(sk.index())))
        .unwrap_or(vitals::skills::Skill::Melee);
    let mut v = VIEW.lock().unwrap();
    v.centre = perks::region_centre(best);
    v.zoom = ZOOM_START;
}

/// Perk points the local player has.
fn points(c: &ClientView, s: &SkillWeb) -> u32 {
    let level = c
        .component(&VITALS)
        .map_or(1, |v| vitals::levels::level_of(v.xp).0);
    crate::skills::points_of(level, perks::spent(&s.taken))
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
    s.stones
        .iter()
        .for_each(|d| mix(d.kind as u64 ^ (d.level as u64) << 8));
    // The picker lists the stones carried.
    (0..INVENTORY_SLOTS).for_each(|i| {
        mix(STONE
            .on_stack_own(&c.inventory_slot(i))
            .map_or(0, |d| d.kind as u64 + 1))
    });
    h
}

pub fn register(r: &mut Registry) {
    r.key(&OPEN, "K").on_press_client(|c| {
        open_on_home(&c.view());
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
            open_on_home(c);
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
            if let Some(socket) = v.picking.filter(|&id| perks::has(&s.taken, id)) {
                picker(ui, &c, socket);
            }
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
        ui.column(S::column().gap(1.0).scroll().w(SIDE_W).h(VIEW_H - 4.0), |ui| {
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
                        .pad(2.0)
                        .gap(1.0)
                        .w(ROW_W)
                        .tooltip_args("web.base.skill_tip", &[Arg::Key(&name), Arg::Key(&by)]),
                    move |c| change(c, |v| v.centre = region),
                    |ui| {
                        // The name left, the level right-aligned in its column.
                        ui.row(S::row().gap(2.0).w(ROW_W - 4.0), |ui| {
                            ui.text_styled(S::default().no_wrap(), &name, &[]);
                            ui.spacer();
                            ui.text_styled(S::default().no_wrap().mono(), "web.base.level", &[Arg::Int(level as i64)]);
                        });
                        ui.progress_bind("stat_xp", S::default().size(ROW_W - 4.0, 2.0), move |_| into);
                    },
                );
            }
        });
    });
}

/// Where a web point lands in the well at this view, on a whole GUI pixel.
fn place(v: &View, p: (f32, f32)) -> (f32, f32) {
    let z = ZOOMS[v.zoom];
    (((p.0 - v.centre.0) * z + VIEW_W / 2.0).round(), ((p.1 - v.centre.1) * z + VIEW_H / 2.0).round())
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
    match perks::can_take(&s.taken, n.id, &s.levels(), points.max(perks::cost(n))) {
        Ok(()) if points < perks::cost(n) => State::Waiting,
        Ok(()) => State::Open,
        Err(Refusal::Skill) => State::Locked,
        Err(_) => State::Far,
    }
}

fn web(ui: &mut Ui, s: &SkillWeb, points: u32, v: &View) {
    let web = perks::web();
    let z = ZOOMS[v.zoom];
    // Links first, under the nodes: a row of dots, lit where both ends are taken.
    let times = if z >= ZOOM_DOUBLE { 2.0 } else { 1.0 };
    let dot = DOT * times;
    for &(a, b) in &web.links {
        let (Some(na), Some(nb)) = (web.node(a), web.node(b)) else { continue };
        let (pa, pb) = (place(v, na.pos()), place(v, nb.pos()));
        if !inside(pa, -60.0) && !inside(pb, -60.0) {
            continue;
        }
        let lit = perks::has(&s.taken, a) && perks::has(&s.taken, b);
        let class = if lit { "perk_link_taken" } else { "perk_link" };
        // Along the link's arc where it curves (a ring, a branch: `web_layout`), else straight.
        let points: Vec<(f32, f32)> = match web.arcs.get(&vitals::web_layout::key(a, b)) {
            Some(&c) => {
                let c = place(v, c);
                let r = (pa.0 - c.0).hypot(pa.1 - c.1);
                let a0 = (pa.1 - c.1).atan2(pa.0 - c.0);
                let a1 = (pb.1 - c.1).atan2(pb.0 - c.0);
                let da = (a1 - a0 + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                let steps = ((da.abs() * r) / (DOT_STEP * times)).floor().max(1.0) as u32;
                (1..steps)
                    .map(|k| {
                        let t = a0 + da * k as f32 / steps as f32;
                        ((c.0 + r * t.cos()).round(), (c.1 + r * t.sin()).round())
                    })
                    .collect()
            }
            None => {
                let len = (pb.0 - pa.0).hypot(pb.1 - pa.1);
                let steps = (len / (DOT_STEP * times)).floor().max(1.0) as u32;
                (1..steps)
                    .map(|k| {
                        let t = k as f32 / steps as f32;
                        ((pa.0 + (pb.0 - pa.0) * t).round(), (pa.1 + (pb.1 - pa.1) * t).round())
                    })
                    .collect()
            }
        };
        for p in points {
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
            Kind::Socket => ("socket", SOCKET),
        };
        let d = size * times;
        if !inside(p, d / 2.0) {
            continue;
        }
        let st = if n.id == perks::START { State::Taken } else { state(s, n, points) };
        let stone = n
            .socket
            .and_then(|i| s.stones[i as usize].stone())
            .filter(|_| st == State::Taken);
        let class = match stone.map(|t| t.kind) {
            Some(StoneKind::Mind) => "perk_socket_mind",
            Some(StoneKind::Flow) => "perk_socket_flow",
            Some(StoneKind::Unique(_)) => "perk_socket_relic",
            None => node_class(kind, st),
        };
        let (tip, args) = match n.socket {
            Some(_) => socket_tooltip(stone, st),
            None => tooltip(n, st, s),
        };
        let is_socket = n.socket.is_some();
        let filled = stone.is_some();
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
                } else if is_socket && st == State::Taken {
                    // A full socket gives its stone back; an empty one opens the picker.
                    if filled {
                        c.client().send(&crate::skills::UNSOCKET, &(id as u32));
                    } else {
                        change(c, |v| v.picking = Some(id));
                    }
                }
            },
            |_| {},
        );
    }
}

/// A node's class: its kind and state (`perk_<kind>_<state>`, the textures in `gui/`).
fn node_class(kind: &str, st: State) -> &'static str {
    match (kind, st.suffix()) {
        ("start", _) => "perk_start_taken",
        ("small", "taken") => "perk_small_taken",
        ("small", "open") => "perk_small_open",
        ("small", "far") => "perk_small_far",
        ("small", _) => "perk_small_locked",
        ("notable", "taken") => "perk_notable_taken",
        ("notable", "open") => "perk_notable_open",
        ("notable", "far") => "perk_notable_far",
        ("notable", _) => "perk_notable_locked",
        ("socket", "taken") => "perk_socket_taken",
        ("socket", "open") => "perk_socket_open",
        ("socket", "far") => "perk_socket_far",
        ("socket", _) => "perk_socket_locked",
        (_, "taken") => "perk_keystone_taken",
        (_, "open") => "perk_keystone_open",
        (_, "far") => "perk_keystone_far",
        _ => "perk_keystone_locked",
    }
}

/// The stones carried, as a row under the well: a click puts one in the socket being filled.
fn picker(ui: &mut Ui, c: &ClientView, socket: u16) {
    let stones: Vec<(u32, ItemStack)> = (0..INVENTORY_SLOTS)
        .map(|i| (i, c.inventory_slot(i)))
        .filter(|(_, st)| !st.is_empty() && STONE.on_stack_own(st).is_some())
        .collect();
    ui.row(S::row().gap(4.0).center(), |ui| {
        if stones.is_empty() {
            ui.text_styled(S::default().class("dim").no_wrap(), "web.base.pick_none", &[]);
        } else {
            ui.text_styled(S::default().no_wrap(), "web.base.pick", &[]);
        }
        for (slot, st) in stones {
            // Its rolls, as a socketed stone's tooltip says them (an item in a button has none).
            let (tip, args) = STONE
                .on_stack_own(&st)
                .and_then(|d| d.stone())
                .map_or(("web.base.pick".to_string(), Vec::new()), |stone| stone_tooltip(stone, "pick"));
            let args: Vec<Arg> = args.iter().map(|a| a.as_arg()).collect();
            ui.button_with(
                S::default()
                    .class("slot")
                    .size(PICK_ICON + 2.0, PICK_ICON + 2.0)
                    .center()
                    .tooltip_args(&tip, &args),
                move |c| {
                    c.client()
                        .send(&crate::skills::SOCKET, &((socket as u32) << 16 | slot));
                    change(c, |v| v.picking = None);
                },
                |ui| ui.item(st.item, 1, S::default().size(PICK_ICON, PICK_ICON)),
            );
        }
        ui.spacer();
        ui.button("web.base.pick_cancel", S::default().h(12.0).pad(2.0), |c| change(c, |v| v.picking = None));
    });
}

/// A socket's tooltip: empty with its state, or its stone and what it does.
fn socket_tooltip(stone: Option<vitals::sockets::Stone>, st: State) -> (String, Vec<A>) {
    let Some(stone) = stone else {
        return ("web.base.tip.socket".into(), vec![A::Int(perks::SOCKET_POINTS as i64), A::Key(st.key().into())]);
    };
    stone_tooltip(stone, "stone")
}

/// A stone's tooltip: its name, item level and what it does, one line a boost (`web.base.tip.<kind><n>`:
/// "stone" in a socket, "pick" in the picker).
fn stone_tooltip(stone: vitals::sockets::Stone, kind: &str) -> (String, Vec<A>) {
    use vitals::sockets::{Flow, Kind as K};
    let mut args = vec![A::Key(format!("item.base.{}", stone.kind.id())), A::Int(stone.level as i64)];
    let mut lines = 0;
    for (f, units) in stone.rolls.iter().flatten() {
        args.push(A::Text(format!("{:+} %", (units * vitals::web::UNIT * 100.0).round() as i64)));
        args.push(A::Key(format!("perk.base.family.{}", f.id())));
        lines += 1;
    }
    match (stone.kind, stone.flow) {
        (_, Some(Flow::Amplify(x))) => {
            args.push(A::Text(format!("{:+} %", (x * 100.0).round() as i64)));
            args.push(A::Key("web.base.stone.amplify".into()));
            lines += 1;
        }
        (_, Some(Flow::Turn(f))) => {
            args.push(A::Text("→".into()));
            args.push(A::Key(format!("perk.base.family.{}", f.id())));
            lines += 1;
        }
        (K::Unique(u), _) => {
            args.push(A::Text(String::new()));
            args.push(A::Key(format!("web.base.stone.{}", u.id())));
            lines += 1;
        }
        _ => {}
    }
    (format!("web.base.tip.{kind}{lines}"), args)
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
