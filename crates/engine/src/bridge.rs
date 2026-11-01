//! What chiacraft's own mods may ask the engine beyond Weft's mod interface (chiacraft plan,
//! phase 3). A mod that exports `chiacraft_bridge` is handed `query` when it loads, and calls it
//! with a question and an argument, as text, getting text back: the same way the `packages` mod is
//! handed packages, so Weft's interface stays as it is.
//!
//! Questions, asked on a server's threads (from a mod's server hooks):
//! - `player_key <player entity>`: that player's key (what saves are keyed by), or empty;
//! - `owner <player entity>`: `1` if that player is the world's owner (their key is this game's
//!   account key), else empty;
//! - `links`: the world's links, a line each: `<link id>\t<other world's name>\t<other world's key>`;
//! - `link <id>`: `<other world's name>\t<other world's key>` if the world has that link, else empty;
//! - `ticket <player entity> <link id>`: a travel ticket (`travel`) this world signs for that player
//!   to go through the link, or empty;
//! - `spot <link id> <x> <y> <z>`: arrivals through the link stand here; `unspot <link id>` forgets it.
//!
//! Asked anywhere (from a mod's client hooks too):
//! - `warm <world key>`: asks that world for its status through the player's own endpoint, in the
//!   background, so a trip there finds its way faster; `warmed <world key>` gives what it answered,
//!   `<name>\t<players>`, empty while unknown or offline.

use std::cell::RefCell;

use weft_abi::Str;

pub type Query = unsafe extern "C" fn(query: Str, arg: Str) -> Str;
pub type Install = unsafe extern "C" fn(Query);
pub const INSTALL: &[u8] = b"chiacraft_bridge\0";

thread_local! {
    static ANSWER: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Answers a mod's question. The answer stays valid until this thread asks again.
///
/// # Safety
/// `q` and `arg` must be valid UTF-8 for the call.
pub unsafe extern "C" fn query(q: Str, arg: Str) -> Str {
    let (q, arg) = unsafe { (q.as_str().to_string(), arg.as_str().to_string()) };
    let answer = std::panic::catch_unwind(|| answer(&q, &arg)).unwrap_or_default();
    ANSWER.with(|a| {
        *a.borrow_mut() = answer;
        let a = a.borrow();
        Str { ptr: a.as_ptr(), len: a.len() }
    })
}

fn answer(q: &str, arg: &str) -> String {
    match q {
        "warm" => {
            crate::net::p2p::warm(arg);
            return String::new();
        }
        "warmed" => {
            return crate::net::p2p::warmed(arg)
                .map(|s| format!("{}\t{}", s.name.replace(['\t', '\n'], " "), s.players))
                .unwrap_or_default();
        }
        _ => {}
    }
    let Some(info) = crate::server::current_info() else { return String::new() };
    match q {
        "player_key" => {
            let Ok(entity) = arg.parse::<u64>() else { return String::new() };
            info.shared
                .players
                .read()
                .iter()
                .find(|(e, _)| *e == entity)
                .map(|(_, k)| k.clone())
                .unwrap_or_default()
        }
        "owner" => {
            let Ok(entity) = arg.parse::<u64>() else { return String::new() };
            let key = info
                .shared
                .players
                .read()
                .iter()
                .find(|(e, _)| *e == entity)
                .map(|(_, k)| k.clone());
            let mine = chiacraft_account::current().map(|s| {
                iroh::SecretKey::from_bytes(s.network_seed())
                    .public()
                    .to_string()
            });
            if key.is_some() && key == mine { "1".into() } else { String::new() }
        }
        "links" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::of_world(world)
                .iter()
                .map(|l| format!("{}\t{}\n", l.id(), other(world, l)))
                .collect()
        }
        "link" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::of_world(world)
                .iter()
                .find(|l| l.id() == arg)
                .map(|l| other(world, l))
                .unwrap_or_default()
        }
        "ticket" => {
            let (Some((entity, link)), Some(id), Some(world)) = (arg.split_once(' '), &info.identity, &info.world_key) else {
                return String::new();
            };
            let Ok(entity) = entity.parse::<u64>() else { return String::new() };
            let Some(player) = info
                .shared
                .players
                .read()
                .iter()
                .find(|(e, _)| *e == entity)
                .map(|(_, k)| k.clone())
            else {
                return String::new();
            };
            let Some(l) = crate::links::of_world(world)
                .into_iter()
                .find(|l| l.id() == link)
            else {
                return String::new();
            };
            let to = if l.body.from == *world { &l.body.to } else { &l.body.from };
            crate::travel::issue(id, &player, link, to)
        }
        "spot" | "unspot" => {
            let Some(dir) = &info.dir else { return String::new() };
            let mut f = arg.split(' ');
            let link = f.next().unwrap_or("");
            let p: Vec<i32> = f.filter_map(|n| n.parse().ok()).collect();
            let pos = match (q, &p[..]) {
                ("spot", [x, y, z]) => Some([*x, *y, *z]),
                ("spot", _) => return String::new(),
                _ => None,
            };
            crate::travel::set_spot(dir, link, pos);
            "1".into()
        }
        _ => String::new(),
    }
}

/// The other world of a link of `world`: its name and key.
fn other(world: &str, l: &crate::links::Link) -> String {
    let (name, key) = if l.body.from == world { (&l.to_name, &l.body.to) } else { (&l.from_name, &l.body.from) };
    format!("{}\t{key}", name.replace(['\t', '\n'], " "))
}
