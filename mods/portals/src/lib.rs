//! Portal blocks (chiacraft plan, phase 3): a portal holds a link between this world and another,
//! and walking into it takes the player there. Built into the game, not a data package, because a
//! portal must mean the same in every world whatever packages it plays with.
//!
//! The world's owner right-clicks a portal and picks one of the world's links (made on the Links
//! screen); the whole connected portal takes it. Only the owner can bind: the server checks again
//! before it changes anything. A player stepping into a bound portal is told to join the linked
//! world. What the engine knows beyond Weft's mod interface (is this player the owner, what links
//! does this world have) the mod asks through the engine's bridge (`chiacraft_bridge`).
//!
//! Linking at the portal (`notes/portals-easy-link.md`): a visitor right-clicking an unbound portal
//! picks one of their own worlds, and their own game asks this world for a link from it (signed by
//! their world, checked by the owner's inbox as any request is). The owner's portal screen lists
//! friends' worlds, to ask one of them the same way. Either way the portal *waits* for that link
//! (its id is known when it is asked: the hash of the request) and binds when the link is made:
//! at once when the owner's inbox accepts friends, or when the owner answers at the portal.

use std::collections::HashSet;
use std::sync::Mutex;

use weft_sdk::*;

declare_mod!(deps: [], init: init);

static PORTAL: Block = Block::named("portal");
static PORTAL_ENTITY: EntityKind = EntityKind::named("portal");
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

/// Client to server, a visitor: their game asked for a link from their world for the portal at
/// `x y z` (`x y z\tlink id\tworld key\tworld name`).
static ASKED: Channel<String> = Channel::named("asked");
/// Client to server, the owner: the answer to the request for link `id` (`id\taccept`, `always`,
/// `decline` or `block`).
static ANSWER: Channel<String> = Channel::named("answer");
/// Client to server, the owner: ask a friend's world to link, for the portal at `x y z`
/// (`x y z\tworld key\tfriend's inbox\tworld name`).
static FRIEND: Channel<String> = Channel::named("friend");
/// Client to server, the owner: ask about friends' link requests too (`1`) or not (`0`).
static FRIENDS_ASK: Channel<String> = Channel::named("friends_ask");

fn say(w: &World, player: u64, key: &str, arg: &str) {
    w.send(player, &SAY, &format!("{key}\t{arg}"));
}

static SCREEN: Screen = Screen::named("portal");

// ------------------------------------------------------------------ the engine's bridge

chiacraft_bridge::install!();
use chiacraft_bridge::ask;

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
    // A pad set flat on the ground. Once its link is whole (a portal bound in each world), a
    // doorway two blocks tall stands over it: lit and moving while the other world is open, dim
    // and still while it is offline. Drawn, not solid: the player walks onto the pad through it.
    r.block(&PORTAL)
        .sound_group("base:block.glass")
        .hardness(1.5)
        .not_solid()
        .translucent()
        .tab(&BUILDING)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("state", ["base", "closed", "open"])
        .states(|p, s| {
            let state = if p.is("state", "open") {
                "open"
            } else if p.is("state", "closed") {
                "closed"
            } else {
                "base"
            };
            let pad = "portal_pad";
            let s = s
                .oriented(p.orientation())
                .hitbox([0.0, 0.0, 0.0], [16.0, 2.0, 16.0])
                .model_box_faces([0.0, 0.0, 0.0], [16.0, 2.0, 16.0], [(pad, abi::UV_FROM_BOX); 6]);
            if state == "base" {
                return s.light(3);
            }
            let (door, frame) = if state == "open" { ("portal", "portal_frame") } else { ("portal_dim", "portal_frame_dim") };
            let no = abi::UV_NO_FACE;
            let full = [0.0, 0.0, 16.0, 16.0];
            // The doorway's sheet (both faces show it whole), its posts and lintel.
            s.model_box_faces(
                [1.0, 2.0, 7.5],
                [15.0, 34.0, 8.5],
                [(door, no), (door, no), (door, no), (door, no), (door, full), (door, full)],
            )
            .model_box_faces([0.0, 2.0, 6.0], [1.0, 35.0, 10.0], [(frame, abi::UV_FROM_BOX); 6])
            .model_box_faces([15.0, 2.0, 6.0], [16.0, 35.0, 10.0], [(frame, abi::UV_FROM_BOX); 6])
            .model_box_faces([1.0, 34.0, 6.0], [15.0, 35.0, 10.0], [(frame, abi::UV_FROM_BOX); 6])
            .light(if state == "open" { 11 } else { 4 })
        })
        .block_entity()
        .on_use(|u| {
            let (w, player, pos) = (u.world(), u.player(), u.pos());
            w.send(player, &OPEN, &menu_for(&w, player, [pos.x, pos.y, pos.z]));
            true
        });
    // The block's own block entity (`block_entity`, named like the block) holds the link.
    r.entity_component(&LINK, [0u8; 64], abi::component_flags::SAVE);
    r.attach(&PORTAL_ENTITY, &LINK);

    r.packet(&BIND, |p, text| bind(p, &text));
    r.packet(&ASKED, |p, text| asked(&p.world(), p.player(), &text));
    r.packet(&ANSWER, |p, text| answer(&p.world(), p.player(), &text));
    r.packet(&FRIEND, |p, text| ask_friend(&p.world(), p.player(), &text));
    r.packet(&FRIENDS_ASK, |p, text| set_friends_ask(&p.world(), p.player(), text == "1"));
    r.command("portalfriends", "[mode:word(on,off)]", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        if !is_owner(player) {
            cmd.reply("portals.msg.not_owner", &[]);
            return;
        }
        match cmd.text("mode") {
            Some("on") => ask("set_friends_ask", "0"),
            Some("off") => ask("set_friends_ask", "1"),
            _ => String::new(),
        };
        let key = if ask("friends_ask", "") == "1" { "portals.msg.friends_off" } else { "portals.msg.friends_on" };
        let _ = w;
        cmd.reply(key, &[]);
    })
    .permission(permission::EVERYONE);
    r.world_tick(settle_waits);
    r.world_tick(refresh_portals);
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
    (bytes[0] != 0).then(|| String::from_utf8_lossy(&bytes).into_owned())
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
    if !PORTAL.is(w.block([x, y, z])) {
        return;
    }
    let name = if id.is_empty() {
        String::new()
    } else {
        let Some((name, _)) = link(id) else {
            say(&w, player, "portals.msg.link_gone", "");
            return;
        };
        if !is_id(id) {
            return;
        }
        name
    };
    let (blocks, room) = bind_portal(&w, [x, y, z], id);
    // The owner chose: whatever this portal waited for no longer binds it.
    with_waits(&w, |ws| ws.list.retain(|x| !blocks.contains(&x.pos)));
    if !id.is_empty() && !room {
        say(&w, player, "portals.msg.no_room", "");
    }
    if name.is_empty() {
        say(&w, player, "portals.msg.unbound", &blocks.len().to_string());
    } else {
        say(&w, player, "portals.msg.bound", &name);
    }
}

fn is_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Binds the portal at `at` (and every portal block touching it) to link `id` (empty: to
/// nothing), with the arrival spot beside it: its blocks, and whether arrivals have room.
fn bind_portal(w: &World, at: [i32; 3], id: &str) -> (Vec<[i32; 3]>, bool) {
    let old = link_at(w, at);
    let mut value = [0u8; 64];
    if !id.is_empty() {
        value.copy_from_slice(id.as_bytes());
    }
    let blocks = portal_blocks(w, at);
    for b in &blocks {
        let e = w.block_entity(*b);
        if let Some(v) = (e != 0).then(|| w.component_mut(e, &LINK)).flatten() {
            *v = value;
        }
    }
    // Where arrivals through this link stand: beside the portal, on solid ground, with room. The
    // link it led to before no longer arrives here.
    if let Some(old) = old.filter(|o| o != id) {
        ask("unspot", &old);
    }
    let room = id.is_empty() || save_spot(w, id, &blocks);
    (blocks, room)
}

// ------------------------------------------------------------------ linking at the portal

/// Waiting asks, at most: per visitor, and per world.
const MAX_WAITS_PER_PLAYER: usize = 3;
const MAX_WAITS: usize = 16;
/// A wait gives up after this long (the link requests it waits for do too).
const WAIT_SECS: u64 = 14 * 86_400;
/// A link that was already made when a visitor said they asked for it binds only if it is this
/// new: one made by their ask moments ago (a friend's is accepted before their game tells us),
/// never an older link of the world's.
const FRESH_SECS: u64 = 600;

/// A portal waiting for a link to be made: the one asked for at it.
#[derive(Clone)]
struct Wait {
    /// The portal block it was asked at.
    pos: [i32; 3],
    /// The link's id (the hash of the request: known when it is asked).
    id: String,
    /// The other world's key and name: the link must be with that world.
    world: String,
    name: String,
    /// Who asked (a player entity), and whether that was the owner (for a friend's world).
    player: u64,
    by_owner: bool,
    since: u64,
    /// The owner has been told about the request.
    told: bool,
}

#[derive(Default)]
struct Waits {
    /// The server run they belong to (`world` on the bridge): another world opened anew loads its own.
    run: String,
    file: String,
    list: Vec<Wait>,
    last: Option<std::time::Instant>,
    /// When each visitor's ask was last told to the owner.
    told: std::collections::HashMap<u64, std::time::Instant>,
    /// Bound portals seen near players this run, by position: their link.
    known: std::collections::HashMap<[i32; 3], String>,
    /// When each other world was last asked whether it is open.
    asked: std::collections::HashMap<String, std::time::Instant>,
    /// When portals' looks were last set.
    looked: Option<std::time::Instant>,
    /// Owners told this run which links still need a portal here.
    reminded: HashSet<u64>,
}

static WAITS: Mutex<Option<Waits>> = Mutex::new(None);

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// The waits of the world being served (loaded from `waits.txt` in its save the first time).
fn with_waits<R>(w: &World, f: impl FnOnce(&mut Waits) -> R) -> R {
    let _ = w;
    let mut g = WAITS.lock().unwrap();
    let run = ask("world", "");
    if g.as_ref().is_none_or(|x| x.run != run) {
        let dir = ask("data_dir", "portals");
        let file = if dir.is_empty() { String::new() } else { format!("{dir}/waits.txt") };
        let list = std::fs::read_to_string(&file)
            .unwrap_or_default()
            .lines()
            .filter_map(parse_wait)
            .collect();
        *g = Some(Waits { run, file, list, ..Default::default() });
    }
    f(g.as_mut().unwrap())
}

fn parse_wait(line: &str) -> Option<Wait> {
    let mut f = line.split('\t');
    let c: Vec<i32> = f
        .next()?
        .split(' ')
        .filter_map(|n| n.parse().ok())
        .collect();
    let [x, y, z] = c[..] else { return None };
    let id = f.next()?.to_string();
    let world = f.next()?.to_string();
    let name = f.next()?.to_string();
    let by_owner = f.next()? == "1";
    let since = f.next()?.parse().ok()?;
    is_id(&id).then_some(Wait { pos: [x, y, z], id, world, name, player: 0, by_owner, since, told: false })
}

fn save_waits(ws: &Waits) {
    if ws.file.is_empty() {
        return;
    }
    let text: String = ws
        .list
        .iter()
        .map(|x| {
            format!(
                "{} {} {}\t{}\t{}\t{}\t{}\t{}\n",
                x.pos[0],
                x.pos[1],
                x.pos[2],
                x.id,
                x.world,
                x.name.replace(['\t', '\n'], " "),
                if x.by_owner { 1 } else { 0 },
                x.since
            )
        })
        .collect();
    if let Err(e) = std::fs::write(&ws.file, text) {
        weft_sdk::warn!("saving portal waits: {e}");
    }
}

fn pos_of(text: &str) -> Option<[i32; 3]> {
    let c: Vec<i32> = text.split(' ').filter_map(|n| n.parse().ok()).collect();
    let [x, y, z] = c[..] else { return None };
    Some([x, y, z])
}

/// What the portal screen shows the player who right-clicked the portal at `at` (see `Menu`).
fn menu_for(w: &World, player: u64, at: [i32; 3]) -> String {
    let owner = is_owner(player);
    let mut msg = format!("{}\n{} {} {}\n", if owner { "owner" } else { "guest" }, at[0], at[1], at[2]);
    if let Some((id, (name, key))) = link_at(w, at).and_then(|id| Some((id.clone(), link(&id)?))) {
        msg.push_str(&format!("N\t{name}\t{}\n", look_of(w, &id, &key)));
    }
    let blocks = portal_blocks(w, at);
    let waits = with_waits(w, |ws| {
        ws.list
            .iter()
            .filter(|x| blocks.contains(&x.pos) && (owner || x.player == player))
            .cloned()
            .collect::<Vec<_>>()
    });
    for x in &waits {
        let incoming = if owner { ask("incoming", &x.id) } else { String::new() };
        if incoming.is_empty() {
            msg.push_str(&format!("P\t{}\n", x.name));
        } else {
            msg.push_str(&format!("W\t{}\t{}\n", x.id, x.name));
        }
    }
    if owner {
        msg.push_str(&format!("A\t{}\n", ask("friends_ask", "")));
        // Links no portal here leads to yet come first, marked new.
        let mut links: Vec<(bool, String)> = ask("links", "")
            .lines()
            .filter_map(|l| {
                let mut f = l.split('\t');
                let (id, name) = (f.next()?, f.next()?);
                let new = ask("portal_state", id).split('\t').next() != Some("1");
                Some((new, format!("L\t{id}\t{name}\t{}\n", if new { 1 } else { 0 })))
            })
            .collect();
        links.sort_by_key(|(new, _)| !*new);
        for (_, l) in links {
            msg.push_str(&l);
        }
        for l in ask("friend_worlds", "").lines() {
            msg.push_str(&format!("F\t{l}\n"));
        }
    }
    msg
}

/// A visitor's game asked for a link from their world for the portal at `x y z`.
fn asked(w: &World, player: u64, text: &str) {
    let mut f = text.split('\t');
    let (Some(at), Some(id), Some(world), name) = (f.next().and_then(pos_of), f.next(), f.next(), f.next().unwrap_or("")) else {
        return;
    };
    if is_owner(player) || !is_id(id) || world.is_empty() || !PORTAL.is(w.block(at)) {
        return;
    }
    if link_at(w, at).is_some() {
        say(w, player, "portals.msg.bound_already", "");
        return;
    }
    // Made already: only if just now, with their world (a friend's ask is accepted at once).
    if let Some((_, key)) = link(id) {
        let age: u64 = ask("link_age", id).parse().unwrap_or(u64::MAX);
        if key != world || age > FRESH_SECS {
            return;
        }
    }
    let name: String = name.chars().filter(|c| !c.is_control()).take(64).collect();
    let added = with_waits(w, |ws| {
        if ws.list.iter().any(|x| x.id == id) {
            return Ok(false);
        }
        let mine = ws
            .list
            .iter()
            .filter(|x| !x.by_owner && x.player == player)
            .count();
        if mine >= MAX_WAITS_PER_PLAYER || ws.list.len() >= MAX_WAITS {
            return Err(());
        }
        ws.list.push(Wait {
            pos: at,
            id: id.to_string(),
            world: world.to_string(),
            name: name.clone(),
            player,
            by_owner: false,
            since: now_secs(),
            told: false,
        });
        save_waits(ws);
        Ok(true)
    });
    match added {
        Ok(true) => say(w, player, "portals.msg.asked", &name),
        Ok(false) => {}
        Err(()) => say(w, player, "portals.msg.too_many", ""),
    }
}

/// The owner answers the request a portal waits for.
fn answer(w: &World, player: u64, text: &str) {
    if !is_owner(player) {
        say(w, player, "portals.msg.not_owner", "");
        return;
    }
    let (id, how) = text.split_once('\t').unwrap_or((text, ""));
    if !is_id(id) || !["accept", "always", "decline", "block"].contains(&how) {
        return;
    }
    let r = ask("answer", &format!("{id} {how}"));
    if let Some(why) = r.strip_prefix('!') {
        say(w, player, "portals.msg.failed", why);
    } else if how == "decline" || how == "block" {
        with_waits(w, |ws| {
            ws.list.retain(|x| x.id != id);
            save_waits(ws);
        });
        say(w, player, "portals.msg.declined", "");
    }
    // Accepted: the link is made, and `settle_waits` binds the portal.
}

/// The owner asks a friend's world to link, for the portal at `x y z`.
fn ask_friend(w: &World, player: u64, text: &str) {
    if !is_owner(player) {
        say(w, player, "portals.msg.not_owner", "");
        return;
    }
    let mut f = text.split('\t');
    let (Some(at), Some(key), inbox, name) = (f.next().and_then(pos_of), f.next(), f.next().unwrap_or(""), f.next().unwrap_or("")) else {
        return;
    };
    if !PORTAL.is(w.block(at)) {
        return;
    }
    let id = ask("ask_from_here", &format!("{key}\t{inbox}"));
    if let Some(why) = id.strip_prefix('!') {
        say(w, player, "portals.msg.failed", why);
        return;
    }
    if !is_id(&id) {
        return;
    }
    let name: String = name.chars().filter(|c| !c.is_control()).take(64).collect();
    with_waits(w, |ws| {
        let blocks = portal_blocks(w, at);
        ws.list.retain(|x| !blocks.contains(&x.pos));
        ws.list.push(Wait {
            pos: at,
            id,
            world: key.to_string(),
            name: name.clone(),
            player,
            by_owner: true,
            since: now_secs(),
            told: true,
        });
        save_waits(ws);
    });
    say(w, player, "portals.msg.asked_friend", &name);
}

fn set_friends_ask(w: &World, player: u64, ask_them: bool) {
    if !is_owner(player) {
        say(w, player, "portals.msg.not_owner", "");
        return;
    }
    ask("set_friends_ask", if ask_them { "1" } else { "0" });
    say(w, player, if ask_them { "portals.msg.friends_off" } else { "portals.msg.friends_on" }, "");
}

/// About once a second: portals whose link was made bind to it; the owner hears of new asks.
fn settle_waits(w: &World) {
    let due = with_waits(w, |ws| {
        if ws.last.is_some_and(|t| t.elapsed().as_millis() < 1000) {
            return false;
        }
        ws.last = Some(std::time::Instant::now());
        !ws.list.is_empty()
    });
    if !due {
        return;
    }
    let players = w.players();
    let owners: Vec<u64> = players.iter().copied().filter(|p| is_owner(*p)).collect();
    let now = now_secs();
    let list = with_waits(w, |ws| ws.list.clone());
    let mut done = Vec::new();
    let mut told = Vec::new();
    for x in &list {
        if !PORTAL.is(w.block(x.pos)) || x.since + WAIT_SECS < now {
            done.push(x.id.clone());
            continue;
        }
        if let Some((name, key)) = link(&x.id) {
            done.push(x.id.clone());
            // A visitor's ask never replaces where the portal leads now; the owner's does.
            if key != x.world || (!x.by_owner && link_at(w, x.pos).is_some()) {
                continue;
            }
            let (blocks, room) = bind_portal(w, x.pos, &x.id);
            weft_sdk::info!("the portal at {:?} now leads to '{name}' ({} blocks)", x.pos, blocks.len());
            for o in &owners {
                say(w, *o, "portals.msg.linked", &name);
                if !room {
                    say(w, *o, "portals.msg.no_room", "");
                }
            }
            if !x.by_owner && players.contains(&x.player) {
                say(w, x.player, "portals.msg.finish", &x.name);
            }
            continue;
        }
        if !x.told && !x.by_owner && !owners.is_empty() && !ask("incoming", &x.id).is_empty() {
            told.push(x.id.clone());
            let quiet = with_waits(w, |ws| {
                ws.told
                    .get(&x.player)
                    .is_some_and(|t| t.elapsed().as_secs() < 60)
            });
            if !quiet {
                with_waits(w, |ws| ws.told.insert(x.player, std::time::Instant::now()));
                for o in &owners {
                    say(w, *o, "portals.msg.asks", &x.name);
                }
            }
        }
    }
    if !done.is_empty() || !told.is_empty() {
        with_waits(w, |ws| {
            ws.list.retain(|x| !done.contains(&x.id));
            for x in ws.list.iter_mut().filter(|x| told.contains(&x.id)) {
                x.told = true;
            }
            if !done.is_empty() {
                save_waits(ws);
            }
        });
    }
}

/// Every portal block connected to the one at `start`, at most 64.
fn portal_blocks(w: &World, start: [i32; 3]) -> Vec<[i32; 3]> {
    let mut todo = vec![start];
    let mut seen = HashSet::new();
    let mut blocks = Vec::new();
    while let Some(at) = todo.pop() {
        if seen.len() >= 64 || !seen.insert(at) || !PORTAL.is(w.block(at)) {
            continue;
        }
        blocks.push(at);
        for d in [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]] {
            todo.push([at[0] + d[0], at[1] + d[1], at[2] + d[2]]);
        }
    }
    blocks
}

/// Saves where arrivals through link `id` stand for the portal made of `blocks`, with its
/// dimension (by name: ids shift when mods come or go) and the way to face. False if it has no room.
fn save_spot(w: &World, id: &str, blocks: &[[i32; 3]]) -> bool {
    let Some((s, yaw)) = arrival_spot(w, blocks) else { return false };
    let dim = name_of(abi::entry_kind::DIMENSION, w.dimension());
    ask("spot", &format!("{id} {} {} {} {dim} {yaw:.4}", s[0], s[1], s[2]));
    true
}

/// Links whose arrival spot was worked out again this run: a portal bound before spots kept their
/// dimension and facing (11-05), or whose ground changed, is set right the first time a player
/// comes near it.
static RESPOTTED: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Where arrivals stand, and the way they face (yaw in radians, 0 looking along -z): in front of
/// one of the portal's faces, right against it, facing away. Walking straight on leads away from
/// it; turning round and stepping forward walks back in (the join test, 11-05: arriving at the
/// sheet's end, a guest walked straight past the portal without touching it).
fn arrival_spot(w: &World, blocks: &[[i32; 3]]) -> Option<([i32; 3], f32)> {
    let solid = |p: [i32; 3]| w.block_flags(p) & abi::block_flags::SOLID != 0;
    let free = |p: [i32; 3]| !solid(p) && !PORTAL.is(w.block(p));
    arrival_in(blocks, &solid, &free)
}

/// `arrival_spot` on any world: `solid` and `free` say what is at a block.
fn arrival_in(blocks: &[[i32; 3]], solid: &dyn Fn([i32; 3]) -> bool, free: &dyn Fn([i32; 3]) -> bool) -> Option<([i32; 3], f32)> {
    if blocks.is_empty() {
        return None;
    }
    // A sheet standing along z (one x) faces east and west; along x (one z), north and south. A
    // single column or a shape of neither kind: any side.
    let one = |f: fn(&[i32; 3]) -> i32| blocks.iter().all(|b| f(b) == f(&blocks[0]));
    let (one_x, one_z) = (one(|b| b[0]), one(|b| b[2]));
    let faces: &[[i32; 2]] = match (one_x, one_z) {
        (true, false) => &[[1, 0], [-1, 0]],
        (false, true) => &[[0, 1], [0, -1]],
        _ => &[[1, 0], [-1, 0], [0, 1], [0, -1]],
    };
    let n = blocks.len() as f64;
    let middle = [blocks.iter().map(|b| b[0] as f64).sum::<f64>() / n, blocks.iter().map(|b| b[2] as f64).sum::<f64>() / n];
    // Lowest first, then the side with the most open ground ahead (not facing a wall), then the
    // middle of the sheet.
    // The spot and its facing, and its rank: (height, minus the open ground ahead, off the middle).
    type Ranked = (([i32; 3], f32), (i32, i32, f64));
    let mut best: Option<Ranked> = None;
    for b in blocks {
        for d in faces {
            let at = [b[0] + d[0], b[1], b[2] + d[1]];
            if !(free(at) && free([at[0], at[1] + 1, at[2]]) && solid([at[0], at[1] - 1, at[2]])) {
                continue;
            }
            let open = (1..=3)
                .take_while(|k| {
                    let p = [at[0] + d[0] * k, at[1], at[2] + d[1] * k];
                    free(p) && free([p[0], p[1] + 1, p[2]])
                })
                .count() as i32;
            let off = (b[0] as f64 - middle[0]).abs() + (b[2] as f64 - middle[1]).abs();
            let rank = (at[1], -open, off);
            if best
                .as_ref()
                .is_none_or(|(_, r)| rank.0 < r.0 || (rank.0 == r.0 && (rank.1, rank.2) < (r.1, r.2)))
            {
                // Facing along the face's normal, away from the portal.
                let yaw = (-(d[0] as f32)).atan2(-(d[1] as f32));
                best = Some(((at, yaw), rank));
            }
        }
    }
    best.map(|(s, _)| s)
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
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let feet = [st.pos.x.floor() as i32, st.pos.y.floor() as i32, st.pos.z.floor() as i32];
        let at = [feet, [feet[0], feet[1] + 1, feet[2]]]
            .into_iter()
            .find(|p| PORTAL.is(w.block(*p)));
        let mut steps = STEPS.lock().unwrap();
        let steps = steps.get_or_insert_with(Steps::default);
        let Some(at) = at else {
            steps.inside.remove(&player);
            steps
                .outside_since
                .entry(player)
                .or_insert_with(std::time::Instant::now);
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
                // Open only with a portal in each world, and the other world answering.
                Some((name, _)) if !far_portal(&id) => say(w, player, "portals.msg.waiting_far", &name),
                Some((name, key)) if !online(w, &key) => say(w, player, "portals.msg.offline", &name),
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
static NEARBY: Mutex<Option<std::collections::HashMap<u64, Nearby>>> = Mutex::new(None);

/// When a player was last looked at, and which links they were told about when.
type Nearby = (std::time::Instant, Vec<(String, std::time::Instant)>);

/// Players walking up to a bound portal: their game warms up the trip (see `bridge`'s `warm`).
fn near_portals(w: &World) {
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let mut nearby = NEARBY.lock().unwrap();
        let entry = nearby
            .get_or_insert_with(Default::default)
            .entry(player)
            .or_insert_with(|| (std::time::Instant::now(), Vec::new()));
        if entry.0.elapsed().as_millis() < 500 {
            continue;
        }
        entry.0 = std::time::Instant::now();
        entry.1.retain(|(_, t)| t.elapsed().as_secs() < 30);
        let c = [st.pos.x.floor() as i32, st.pos.y.floor() as i32, st.pos.z.floor() as i32];
        let mut found = None;
        'search: for dy in -1..=2 {
            for dx in -NEAR_BLOCKS..=NEAR_BLOCKS {
                for dz in -NEAR_BLOCKS..=NEAR_BLOCKS {
                    let p = [c[0] + dx, c[1] + dy, c[2] + dz];
                    if PORTAL.is(w.block(p))
                        && let Some(id) = link_at(w, p)
                    {
                        found = Some((id, p));
                        break 'search;
                    }
                }
            }
        }
        let Some((id, p)) = found else { continue };
        if RESPOTTED
            .lock()
            .unwrap()
            .get_or_insert_with(HashSet::new)
            .insert(id.clone())
        {
            save_spot(w, &id, &portal_blocks(w, p));
        }
        if entry.1.iter().any(|(l, _)| *l == id) {
            continue;
        }
        entry.1.push((id.clone(), std::time::Instant::now()));
        if let Some((name, key)) = link(&id) {
            let state = look_of(w, &id, &key);
            w.send(player, &NEAR, &format!("{key}\t{name}\t{state}"));
        }
    }
}

// ------------------------------------------------------------------ both portals, and the look

/// Whether the other world of link `id` has a portal bound to it too (it said so, signed).
fn far_portal(id: &str) -> bool {
    ask("portal_state", id)
        .split('\t')
        .nth(1)
        .is_some_and(|f| f == "1")
}

/// How often the other worlds are asked whether they are open, at most.
const STATUS_SECS: u64 = 8;

/// Whether the world with key `key` answers now: its status is asked through this game's own
/// endpoint every few seconds while a portal to it is near players (`chiacraft/status/1`).
fn online(w: &World, key: &str) -> bool {
    let ask_now = with_waits(w, |ws| {
        let due = ws
            .asked
            .get(key)
            .is_none_or(|t| t.elapsed().as_secs() >= STATUS_SECS);
        if due {
            ws.asked.insert(key.to_string(), std::time::Instant::now());
        }
        due
    });
    if ask_now {
        ask("warm", key);
    }
    !ask("warmed", key).is_empty()
}

/// What the portal bound to link `id` (to world `key`) shows: only its pad until the other world
/// has a portal too, then its doorway, open or (the other world offline) closed.
fn look_of(w: &World, id: &str, key: &str) -> &'static str {
    if !far_portal(id) {
        "base"
    } else if online(w, key) {
        "open"
    } else {
        "closed"
    }
}

/// How far around players portals are looked for to set their look, in blocks.
const LOOK_BLOCKS: i32 = 12;

/// Every two seconds: portals near players show what their link is now (pad, doorway open or
/// closed); a portal broken stops counting as the world's portal for its link; owners hear once a
/// run which links still need a portal here.
fn refresh_portals(w: &World) {
    let due = with_waits(w, |ws| {
        if ws.looked.is_some_and(|t| t.elapsed().as_secs() < 2) {
            return false;
        }
        ws.looked = Some(std::time::Instant::now());
        true
    });
    if !due {
        return;
    }
    // Portals near players, bound or not.
    let mut found: Vec<([i32; 3], Option<String>)> = Vec::new();
    for player in w.players() {
        let Some(st) = w.entity(player) else { continue };
        let c = [st.pos.x.floor() as i32, st.pos.y.floor() as i32, st.pos.z.floor() as i32];
        for dy in -3..=3 {
            for dx in -LOOK_BLOCKS..=LOOK_BLOCKS {
                for dz in -LOOK_BLOCKS..=LOOK_BLOCKS {
                    let p = [c[0] + dx, c[1] + dy, c[2] + dz];
                    if PORTAL.is(w.block(p)) && !found.iter().any(|(q, _)| *q == p) {
                        found.push((p, link_at(w, p)));
                    }
                }
            }
        }
    }
    // Portals known before and gone now (broken, or bound elsewhere).
    let gone: Vec<String> = with_waits(w, |ws| {
        let before = std::mem::take(&mut ws.known);
        let mut gone = Vec::new();
        for (p, id) in before {
            if PORTAL.is(w.block(p)) && link_at(w, p).as_deref() == Some(id.as_str()) {
                ws.known.insert(p, id);
            } else {
                gone.push(id);
            }
        }
        for (p, id) in &found {
            if let Some(id) = id {
                ws.known.insert(*p, id.clone());
            }
        }
        gone.retain(|id| !ws.known.values().any(|k| k == id));
        gone
    });
    for id in gone {
        weft_sdk::info!("the portal for a link is gone: arrivals through it have no place here now");
        ask("unspot", &id);
    }
    for (p, id) in &found {
        let state = match id.as_deref().and_then(|id| Some((id, link(id)?))) {
            Some((id, (_, key))) => look_of(w, id, &key),
            None => "base",
        };
        if w.prop(*p, "state") != Some(state) {
            w.prop_set(*p, "state", state);
        }
    }
    remind_owners(w);
}

/// An owner in the world hears, once a run, of each link no portal here leads to yet.
fn remind_owners(w: &World) {
    let owners: Vec<u64> = w.players().into_iter().filter(|p| is_owner(*p)).collect();
    let new: Vec<u64> = with_waits(w, |ws| {
        owners
            .into_iter()
            .filter(|o| ws.reminded.insert(*o))
            .collect()
    });
    if new.is_empty() {
        return;
    }
    for l in ask("links", "").lines() {
        let mut f = l.split('\t');
        let (Some(id), Some(name)) = (f.next(), f.next()) else { continue };
        let here = ask("portal_state", id).split('\t').next() == Some("1");
        if !here {
            for o in &new {
                say(w, *o, "portals.msg.needs_portal", name);
            }
        }
    }
}

// ------------------------------------------------------------------ the client half

/// A visitor's side of the portal screen: an unbound portal can be linked with one of their own
/// worlds. Their own game signs the request (`ask_link`), towards the world it joined, and only
/// on this click; the server is told the link id so the portal waits for it.
fn guest(ui: &mut Ui, m: &Menu) {
    if m.leads_to.is_some() {
        ui.text("portals.not_owner");
        return;
    }
    if m.here.is_empty() || m.mine.is_empty() {
        ui.text("portals.guest_no_worlds");
        return;
    }
    ui.text("portals.guest_pick");
    for (key, name) in &m.mine {
        let (pos, here, key, label) = (m.pos.clone(), m.here.clone(), key.clone(), name.clone());
        ui.button_args("portals.to_mine", &[Arg::Text(name)], S::default().w(280.0), move |v| {
            let id = ask("ask_link", &format!("{key}\t{here}"));
            match id.strip_prefix('!') {
                Some(why) => v
                    .client()
                    .say(&format!("/portal_note portals.msg.failed {why}")),
                None if id.len() == 64 => v
                    .client()
                    .send(&ASKED, &format!("{pos}\t{id}\t{key}\t{label}")),
                None => {}
            }
            v.client().screen_close();
        });
    }
}

/// What the portal screen shows: owner or not, the portal's position, where it leads, and what
/// can be picked. The server sends it as lines: the role, the position, then tagged lines
/// (`N` leads to, `W` a request waiting for the owner, `P` an ask waiting for the other side,
/// `A` friends asked about too, `L` a link, `F` a friend's world).
#[derive(Default)]
struct Menu {
    owner: bool,
    pos: String,
    leads_to: Option<String>,
    /// What the link is now: `base` (waiting for the other world's portal), `closed` (that world
    /// is offline) or `open`.
    look: String,
    /// Requests for this portal waiting for the owner: link id and world name.
    waiting: Vec<(String, String)>,
    /// Asks for this portal waiting for the other side's answer: world names.
    pending: Vec<String>,
    friends_ask: bool,
    /// Links: id, name, and whether no portal here leads there yet (new).
    links: Vec<(String, String, bool)>,
    /// Friends' worlds: key, name, the friend's inbox.
    friends: Vec<(String, String, String)>,
    /// A visitor's own worlds (from their own game, never from the server): key and name.
    mine: Vec<(String, String)>,
    /// The world the visitor is in, as their own game joined it.
    here: String,
}

fn parse_menu(text: &str) -> Menu {
    let mut lines = text.lines();
    let mut m = Menu { owner: lines.next() == Some("owner"), pos: lines.next().unwrap_or("").to_string(), ..Default::default() };
    for l in lines {
        let f: Vec<&str> = l.split('\t').collect();
        match f[..] {
            ["N", name, look] => {
                m.leads_to = Some(name.to_string());
                m.look = look.to_string();
            }
            ["W", id, name] => m.waiting.push((id.to_string(), name.to_string())),
            ["P", name] => m.pending.push(name.to_string()),
            ["A", a] => m.friends_ask = a == "1",
            ["L", id, name, new] => m.links.push((id.to_string(), name.to_string(), new == "1")),
            ["F", key, name, inbox] => m
                .friends
                .push((key.to_string(), name.to_string(), inbox.to_string())),
            _ => {}
        }
    }
    if !m.owner && m.leads_to.is_none() {
        m.here = ask("joined", "")
            .split_once('\t')
            .map(|(k, _)| k.to_string())
            .unwrap_or_default();
        m.mine = ask("my_worlds", "")
            .lines()
            .filter_map(|l| l.split_once('\t'))
            .filter(|(k, _)| *k != m.here)
            .map(|(k, n)| (k.to_string(), n.to_string()))
            .collect();
    }
    m
}

static MENU: Mutex<Option<Menu>> = Mutex::new(None);
/// A world to go to (an address carrying the travel ticket), picked up by the next frame (joining tears down the world in play, which
/// must not happen in the middle of handling its messages).
static GO: Mutex<Option<String>> = Mutex::new(None);

fn client(r: &mut Registry) {
    r.client_packet(&OPEN, |c, text| {
        *MENU.lock().unwrap() = Some(parse_menu(&text));
        c.screen_open(&SCREEN);
    });
    // A line for the player goes through a command of the mod's own, whose reply the client
    // shows in its own language.
    r.client_packet(&SAY, |c, text| {
        let (key, arg) = text.split_once('\t').unwrap_or((&text, ""));
        c.say(&format!("/portal_note {key} {arg}"));
    });
    r.client_packet(&NEAR, |c, text| {
        let mut f = text.split('\t');
        let (key, name, look) = (f.next().unwrap_or(""), f.next().unwrap_or(""), f.next().unwrap_or("open"));
        let note = match look {
            "base" => "portals.msg.waiting_far",
            "closed" => "portals.msg.offline",
            _ => {
                ask("warm", key);
                "portals.msg.near"
            }
        };
        c.say(&format!("/portal_note {note} {name}"));
    });
    r.client_packet(&TRAVEL, |c, text| {
        let mut f = text.split('\t');
        let (key, name, ticket) = (f.next().unwrap_or(""), f.next().unwrap_or(""), f.next().unwrap_or(""));
        weft_sdk::info!("going to '{name}' through a portal");
        c.say(&format!("/portal_note portals.msg.going {name}"));
        let mut address = format!("chiacraft://w/{key}");
        if !ticket.is_empty() {
            address.push_str("?t=");
            address.push_str(
                &ticket
                    .replace('%', "%25")
                    .replace(' ', "%20")
                    .replace('&', "%26"),
            );
        }
        *GO.lock().unwrap() = Some(address);
    });
    r.client_frame(|c| {
        if let Some(key) = GO.lock().unwrap().take() {
            c.world_join(&key);
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
        let wide = || S::default().w(280.0);
        ui.panel("panel", S::column().pad(8.0).gap(4.0).min_w(300.0), |ui| {
            ui.text_styled(S::column().class("title").scale(1.5).no_wrap(), "portals.title", &[]);
            match (&m.leads_to, m.look.as_str()) {
                (Some(name), "base") => ui.text_args("portals.waiting_far", &[Arg::Text(name)]),
                (Some(name), "closed") => ui.text_args("portals.offline", &[Arg::Text(name)]),
                (Some(name), _) => ui.text_args("portals.leads_to", &[Arg::Text(name)]),
                (None, _) => ui.text("portals.leads_nowhere"),
            }
            for name in &m.pending {
                ui.text_args("portals.pending", &[Arg::Text(name)]);
            }
            if !m.owner {
                guest(ui, m);
            } else {
                for (id, name) in &m.waiting {
                    ui.text_args("portals.waiting", &[Arg::Text(name)]);
                    for (key, how) in [
                        ("portals.accept", "accept"),
                        ("portals.always", "always"),
                        ("portals.decline", "decline"),
                        ("portals.block", "block"),
                    ] {
                        let msg = format!("{id}\t{how}");
                        ui.button(key, wide(), move |v| {
                            v.client().send(&ANSWER, &msg);
                            v.client().screen_close();
                        });
                    }
                }
                if m.links.is_empty() && m.friends.is_empty() {
                    ui.text("portals.no_links");
                }
                if !m.links.is_empty() {
                    ui.text("portals.pick");
                }
                for (id, name, new) in &m.links {
                    let msg = format!("{}\t{id}", m.pos);
                    let key = if *new { "portals.to_new" } else { "portals.to" };
                    ui.button_args(key, &[Arg::Text(name)], wide(), move |v| {
                        v.client().send(&BIND, &msg);
                        v.client().screen_close();
                    });
                }
                if !m.friends.is_empty() {
                    ui.text("portals.pick_friend");
                }
                for (key, name, inbox) in &m.friends {
                    let msg = format!("{}\t{key}\t{inbox}\t{name}", m.pos);
                    ui.button_args("portals.to_friend", &[Arg::Text(name)], wide(), move |v| {
                        v.client().send(&FRIEND, &msg);
                        v.client().screen_close();
                    });
                }
                if m.leads_to.is_some() {
                    let msg = format!("{}\t", m.pos);
                    ui.button("portals.unbind", wide(), move |v| {
                        v.client().send(&BIND, &msg);
                        v.client().screen_close();
                    });
                }
                let (key, to) = if m.friends_ask { ("portals.friends_ask_on", "0") } else { ("portals.friends_ask_off", "1") };
                ui.button(key, wide(), move |v| {
                    v.client().send(&FRIENDS_ASK, &to.to_string());
                    v.client().screen_close();
                });
            }
            ui.button("portals.close", wide(), |v| v.client().screen_close());
        });
    });
}

#[cfg(test)]
mod tests {
    use super::arrival_in;

    /// Flat ground at y 63 (solid at 63 and below), a portal sheet, open air elsewhere.
    fn world(portal: &[[i32; 3]]) -> (impl Fn([i32; 3]) -> bool + '_, impl Fn([i32; 3]) -> bool + '_) {
        let solid = |p: [i32; 3]| p[1] <= 63;
        let free = move |p: [i32; 3]| p[1] > 63 && !portal.contains(&p);
        (solid, free)
    }

    fn ahead(yaw: f32) -> [i32; 2] {
        [(-yaw.sin()).round() as i32, (-yaw.cos()).round() as i32]
    }

    #[test]
    fn arrivals_stand_before_a_face_and_look_away() {
        // A sheet along x (one z, at 10): its faces look north and south.
        let sheet: Vec<[i32; 3]> = (0..3)
            .flat_map(|x| (64..67).map(move |y| [x, y, 10]))
            .collect();
        let (solid, free) = world(&sheet);
        let (at, yaw) = arrival_in(&sheet, &solid, &free).unwrap();
        assert_eq!(at[1], 64);
        assert!(at[2] == 9 || at[2] == 11, "beside a face, not the sheet's end: {at:?}");
        assert_eq!(at[0], 1, "in front of the middle");
        let a = ahead(yaw);
        assert_eq!([0, at[2] - 10], a, "facing away from the portal");
        // Turning round and stepping forward walks into the portal.
        let back = [at[0] - a[0], at[1], at[2] - a[1]];
        assert!(sheet.contains(&back));

        // A sheet along z (one x): east or west of it.
        let sheet: Vec<[i32; 3]> = (0..2)
            .flat_map(|z| (64..67).map(move |y| [5, y, z]))
            .collect();
        let (solid, free) = world(&sheet);
        let (at, yaw) = arrival_in(&sheet, &solid, &free).unwrap();
        assert!(at[0] == 4 || at[0] == 6);
        assert_eq!(ahead(yaw), [at[0] - 5, 0]);
    }

    #[test]
    fn a_wall_on_one_side_sends_arrivals_to_the_other() {
        let sheet: Vec<[i32; 3]> = (0..3)
            .flat_map(|x| (64..67).map(move |y| [x, y, 10]))
            .collect();
        // A wall two blocks south of the sheet: the north face has the open ground.
        let solid = |p: [i32; 3]| p[1] <= 63 || p[2] == 12;
        let free = |p: [i32; 3]| p[1] > 63 && p[2] != 12 && !sheet.contains(&p);
        let (at, _) = arrival_in(&sheet, &solid, &free).unwrap();
        assert_eq!(at[2], 9);
    }
}
