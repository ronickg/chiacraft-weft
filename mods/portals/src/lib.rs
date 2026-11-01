//! Portal blocks (chiacraft plan, phase 3): a portal holds a link between this world and another,
//! and walking into it takes the player there. Built into the game, not a data package, because a
//! portal must mean the same in every world whatever packages it plays with.
//!
//! The world's owner right-clicks a portal and picks one of the world's links (made on the Links
//! screen); the whole connected portal takes it. Only the owner can bind: the server checks again
//! before it changes anything. A player stepping into a bound portal is told to join the linked
//! world. What the engine knows beyond Weft's mod interface (is this player the owner, what links
//! does this world have) the mod asks through the engine's bridge (`chiacraft_bridge`).

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use weft_sdk::abi::Str;
use weft_sdk::*;

declare_mod!(deps: [], init: init);

static PORTAL: Block = Block::named("portal");
static PORTAL_ENTITY: Entity = Entity::named("portal");
/// The link the portal holds: its id as 64 hex digits, or zeros.
static LINK: Component<[u8; 64]> = Component::named("link");
static BUILDING: CreativeTab = CreativeTab::named("base:building");

/// Server to the owner's client: open the portal screen. `owner` or `guest`, the portal's
/// position, then the world's links, a line each (`id\tname`).
static OPEN: Channel<String> = Channel::named("open");
/// Client to server: bind the portal at `x y z` to link `id` (empty: unbind).
static BIND: Channel<String> = Channel::named("bind");
/// Server to a client: go to the world with this key, carrying the travel ticket this world signed
/// (`key\tname\tticket`).
static TRAVEL: Channel<String> = Channel::named("travel");
/// Server to a client: a bound portal is near; warm up the way to the world with this key
/// (`key\tname`).
static NEAR: Channel<String> = Channel::named("near");

/// Server to a client: a line to show the player, as a translation key and one argument
/// (`key\targ`), so it reads in the player's own language.
static SAY: Channel<String> = Channel::named("say");

fn say(w: &World, player: u64, key: &str, arg: &str) {
    w.send(player, &SAY, &format!("{key}\t{arg}"));
}

static SCREEN: Screen = Screen::named("portal");

// ------------------------------------------------------------------ the engine's bridge

type Query = unsafe extern "C" fn(query: Str, arg: Str) -> Str;
static BRIDGE: OnceLock<Query> = OnceLock::new();

/// The engine hands the mod its bridge when the mod loads.
///
/// # Safety
/// `q` must be the engine's `bridge::query`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chiacraft_bridge(q: Query) {
    let _ = BRIDGE.set(q);
}

fn ask(q: &str, arg: &str) -> String {
    BRIDGE.get().map(|f| unsafe { f(Str::new(q), Str::new(arg)).as_str().to_string() }).unwrap_or_default()
}

fn is_owner(player: u64) -> bool {
    ask("owner", &player.to_string()) == "1"
}

/// The other world of link `id`: its name and key, if this world has that link.
fn link(id: &str) -> Option<(String, String)> {
    let a = ask("link", id);
    let (name, key) = a.split_once('\t')?;
    Some((name.to_string(), key.to_string()))
}

// ------------------------------------------------------------------ registration

fn init(r: &mut Registry) {
    r.block(&PORTAL).hardness(1.5).not_solid().translucent().light(11).tab(&BUILDING).on_use(|u| {
        let (w, player, pos) = (u.world(), u.player(), u.pos());
        let owner = is_owner(player);
        let mut msg = format!("{}\n{} {} {}\n", if owner { "owner" } else { "guest" }, pos[0], pos[1], pos[2]);
        if owner {
            for l in ask("links", "").lines() {
                let mut f = l.split('\t');
                if let (Some(id), Some(name)) = (f.next(), f.next()) {
                    msg.push_str(&format!("{id}\t{name}\n"));
                }
            }
        }
        w.send(player, &OPEN, &msg);
        true
    });
    drop(r.entity(&PORTAL_ENTITY).flags(abi::entity_flags::NO_GRAVITY | abi::entity_flags::INVULNERABLE));
    r.entity_component(&LINK, [0u8; 64], abi::component_flags::SAVE);
    r.attach(&PORTAL_ENTITY, &LINK);
    r.block_entity(&PORTAL, &PORTAL_ENTITY, 0);

    r.packet(&BIND, |p, text| bind(p, &text));
    r.world_tick(walk_in);
    r.world_tick(near_portals);
    client(r);
}

/// The portal's link at `pos`, if bound.
fn link_at(w: &World, pos: [i32; 3]) -> Option<String> {
    let e = w.block_entity(pos);
    if e == 0 {
        return None;
    }
    let bytes = w.component(e, &LINK)?;
    (bytes[0] != 0).then(|| String::from_utf8_lossy(bytes).into_owned())
}

/// The owner binds a portal (and every portal block touching it) to a link of this world.
fn bind(p: &Packet, text: &str) {
    let w = p.world();
    let player = p.player();
    if !is_owner(player) {
        say(&w, player, "portals.msg.not_owner", "");
        return;
    }
    let (pos, id) = text.split_once('\t').unwrap_or((text, ""));
    let c: Vec<i32> = pos.split(' ').filter_map(|n| n.parse().ok()).collect();
    let [x, y, z] = c[..] else { return };
    if !PORTAL.is(w.get([x, y, z])) {
        return;
    }
    let old = link_at(&w, [x, y, z]);
    let mut value = [0u8; 64];
    let name = if id.is_empty() {
        String::new()
    } else {
        let Some((name, _)) = link(id) else {
            say(&w, player, "portals.msg.link_gone", "");
            return;
        };
        if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
            return;
        }
        value.copy_from_slice(id.as_bytes());
        name
    };
    // The whole portal: every portal block connected to this one, within reason.
    let mut todo = vec![[x, y, z]];
    let mut seen = HashSet::new();
    let mut blocks = Vec::new();
    while let Some(at) = todo.pop() {
        if seen.len() >= 64 || !seen.insert(at) || !PORTAL.is(w.get(at)) {
            continue;
        }
        blocks.push(at);
        let e = w.block_entity(at);
        if let Some(v) = (e != 0).then(|| w.component_mut(e, &LINK)).flatten() {
            *v = value;
        }
        for d in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
            todo.push([at[0] + d[0], at[1] + d[1], at[2] + d[2]]);
        }
    }
    // Where arrivals through this link stand: beside the portal, on solid ground, with room. The
    // link it led to before no longer arrives here.
    if let Some(old) = old.filter(|o| o != id) {
        ask("unspot", &old);
    }
    if !id.is_empty() {
        match arrival_spot(&w, &blocks) {
            Some(s) => {
                ask("spot", &format!("{id} {} {} {}", s[0], s[1], s[2]));
            }
            None => say(&w, player, "portals.msg.no_room", ""),
        }
    }
    if name.is_empty() {
        say(&w, player, "portals.msg.unbound", &seen.len().to_string());
    } else {
        say(&w, player, "portals.msg.bound", &name);
    }
}

/// A place beside the portal to put arrivals: next to one of its blocks, not in it, with two
/// blocks of room and solid ground below; the lowest such place.
fn arrival_spot(w: &World, blocks: &[[i32; 3]]) -> Option<[i32; 3]> {
    let solid = |p: [i32; 3]| w.flags(p) & abi::block_flags::SOLID != 0;
    let mut best: Option<[i32; 3]> = None;
    for b in blocks {
        for d in [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1]] {
            let at = [b[0] + d[0], b[1], b[2] + d[2]];
            let up = [at[0], at[1] + 1, at[2]];
            let down = [at[0], at[1] - 1, at[2]];
            let free = |p: [i32; 3]| !solid(p) && !PORTAL.is(w.get(p));
            if free(at) && free(up) && solid(down) && best.is_none_or(|b| at[1] < b[1]) {
                best = Some(at);
            }
        }
    }
    best
}

/// Players standing in a portal, so entering (not standing) sends them; and since when each player
/// has been outside every portal. A player is sent only after a second outside: one who arrives
/// standing in a portal (from the world it leads to, or back where they left) is not sent straight
/// back, and must step out and in again.
#[derive(Default)]
struct Steps {
    inside: HashSet<u64>,
    outside_since: std::collections::HashMap<u64, std::time::Instant>,
}

static STEPS: Mutex<Option<Steps>> = Mutex::new(None);

/// A player stepping into a bound portal is sent to the linked world.
fn walk_in(w: &World) {
    for player in w.each_player() {
        let Some(st) = w.entity(player) else { continue };
        let feet = [st.pos[0].floor() as i32, st.pos[1].floor() as i32, st.pos[2].floor() as i32];
        let at = [feet, [feet[0], feet[1] + 1, feet[2]]].into_iter().find(|p| PORTAL.is(w.get(*p)));
        let mut steps = STEPS.lock().unwrap();
        let steps = steps.get_or_insert_with(Steps::default);
        let Some(at) = at else {
            steps.inside.remove(&player);
            steps.outside_since.entry(player).or_insert_with(std::time::Instant::now);
            continue;
        };
        let out_for = steps.outside_since.remove(&player).map(|t| t.elapsed());
        if !steps.inside.insert(player) || out_for.is_none_or(|d| d.as_secs_f32() < 1.0) {
            continue;
        }
        weft_sdk::info!("player {player} stepped into the portal at {at:?}");
        match link_at(w, at) {
            None => say(w, player, "portals.msg.unbound_portal", ""),
            Some(id) => match link(&id) {
                Some((name, key)) => {
                    let ticket = ask("ticket", &format!("{player} {id}"));
                    w.send(player, &TRAVEL, &format!("{key}\t{name}\t{ticket}"));
                }
                None => say(w, player, "portals.msg.ended", ""),
            },
        }
    }
}

/// How far from a portal a player's game starts warming up the trip, in blocks.
const NEAR_BLOCKS: i32 = 4;

/// When each player was last looked at and last told about each link, so the search runs about
/// twice a second and a player hears about a portal once every half minute.
static NEARBY: Mutex<Option<std::collections::HashMap<u64, (std::time::Instant, Vec<(String, std::time::Instant)>)>>> = Mutex::new(None);

/// Players walking up to a bound portal: their game warms up the trip (see `bridge`'s `warm`).
fn near_portals(w: &World) {
    for player in w.each_player() {
        let Some(st) = w.entity(player) else { continue };
        let mut nearby = NEARBY.lock().unwrap();
        let entry = nearby.get_or_insert_with(Default::default).entry(player).or_insert_with(|| (std::time::Instant::now(), Vec::new()));
        if entry.0.elapsed().as_millis() < 500 {
            continue;
        }
        entry.0 = std::time::Instant::now();
        entry.1.retain(|(_, t)| t.elapsed().as_secs() < 30);
        let c = [st.pos[0].floor() as i32, st.pos[1].floor() as i32, st.pos[2].floor() as i32];
        let mut found = None;
        'search: for dy in -1..=2 {
            for dx in -NEAR_BLOCKS..=NEAR_BLOCKS {
                for dz in -NEAR_BLOCKS..=NEAR_BLOCKS {
                    let p = [c[0] + dx, c[1] + dy, c[2] + dz];
                    if PORTAL.is(w.get(p))
                        && let Some(id) = link_at(w, p)
                    {
                        found = Some(id);
                        break 'search;
                    }
                }
            }
        }
        let Some(id) = found else { continue };
        if entry.1.iter().any(|(l, _)| *l == id) {
            continue;
        }
        entry.1.push((id.clone(), std::time::Instant::now()));
        if let Some((name, key)) = link(&id) {
            w.send(player, &NEAR, &format!("{key}\t{name}"));
        }
    }
}

// ------------------------------------------------------------------ the client half

/// What the portal screen shows: owner or not, the portal's position, and the links to pick.
struct Menu {
    owner: bool,
    pos: String,
    links: Vec<(String, String)>,
}

static MENU: Mutex<Option<Menu>> = Mutex::new(None);
/// A world to go to (an address carrying the travel ticket), picked up by the next frame (joining tears down the world in play, which
/// must not happen in the middle of handling its messages).
static GO: Mutex<Option<String>> = Mutex::new(None);

fn client(r: &mut Registry) {
    r.client_packet(&OPEN, |c, text| {
        let mut lines = text.lines();
        let owner = lines.next() == Some("owner");
        let pos = lines.next().unwrap_or("").to_string();
        let links = lines.filter_map(|l| l.split_once('\t')).map(|(id, name)| (id.to_string(), name.to_string())).collect();
        *MENU.lock().unwrap() = Some(Menu { owner, pos, links });
        c.open_screen(&SCREEN);
    });
    // A line for the player goes through a command of the mod's own, whose reply the client
    // shows in its own language.
    r.client_packet(&SAY, |c, text| {
        let (key, arg) = text.split_once('\t').unwrap_or((&text, ""));
        c.chat(&format!("/portal_note {key} {arg}"));
    });
    r.client_packet(&NEAR, |c, text| {
        let (key, name) = text.split_once('\t').unwrap_or((&text, ""));
        ask("warm", key);
        c.chat(&format!("/portal_note portals.msg.near {name}"));
    });
    r.client_packet(&TRAVEL, |c, text| {
        let mut f = text.split('\t');
        let (key, name, ticket) = (f.next().unwrap_or(""), f.next().unwrap_or(""), f.next().unwrap_or(""));
        weft_sdk::info!("going to '{name}' through a portal");
        c.chat(&format!("/portal_note portals.msg.going {name}"));
        let mut address = format!("chiacraft://w/{key}");
        if !ticket.is_empty() {
            address.push_str("?t=");
            address.push_str(&ticket.replace('%', "%25").replace(' ', "%20").replace('&', "%26"));
        }
        *GO.lock().unwrap() = Some(address);
    });
    r.client_frame(|c| {
        if let Some(key) = GO.lock().unwrap().take() {
            c.join(&key);
        }
    });
    r.client_command("portal_note", "<line:text>", |cmd| {
        let line = cmd.text("line").unwrap_or("").to_string();
        let (key, arg) = line.split_once(' ').unwrap_or((&line, ""));
        if key.starts_with("portals.msg.") {
            cmd.reply(key, &[Arg::Text(arg)]);
        }
    });
    r.screen(&SCREEN, |ui| {
        let menu = MENU.lock().unwrap();
        let Some(m) = menu.as_ref() else { return };
        ui.panel("panel", S::col().pad(8.0).gap(4.0).min_w(300.0), |ui| {
            ui.text_styled(S::col().class("title").scale(1.5).no_wrap(), "portals.title", &[]);
            if !m.owner {
                ui.text("portals.not_owner");
            } else if m.links.is_empty() {
                ui.text("portals.no_links");
            } else {
                ui.text("portals.pick");
                for (id, name) in &m.links {
                    let msg = format!("{}\t{id}", m.pos);
                    ui.button_args("portals.to", &[Arg::Text(name)], S::default().w(280.0), move |v| {
                        v.client().send(&BIND, &msg);
                        v.client().close_screen();
                    });
                }
                let msg = format!("{}\t", m.pos);
                ui.button("portals.unbind", S::default().w(280.0), move |v| {
                    v.client().send(&BIND, &msg);
                    v.client().close_screen();
                });
            }
            ui.button("portals.close", S::default().w(280.0), |v| v.client().close_screen());
        });
    });
}
