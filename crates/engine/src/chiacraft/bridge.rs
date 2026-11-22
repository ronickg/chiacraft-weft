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
//! - `link_age <id>`: seconds since link `id` was asked for, if the world has that link;
//! - `ticket <player entity> <link id>`: a travel ticket (`travel`) this world signs for that player
//!   to go through the link, or empty;
//! - `spot <link id> <x> <y> <z>`: arrivals through the link stand here; `unspot <link id>` forgets it;
//! - `data_dir <mod>`: a folder in the world's save for that mod's own files (made if missing),
//!   or empty (no world open, or a name that isn't lowercase letters, digits and `_`).
//! - `achievement <player entity>\t<id>\t<title>\t<description>\t<icon item or empty>`: the
//!   player earned it here; the world signs a credential for them (`achievements`), the title and
//!   description as lang keys or text;
//! - `world`: `<run>\t<folder>`: which run of a server this is (a world opened again is a new one)
//!   and the world's folder (empty for a world that isn't saved). Sandboxed mods run anew in each.
//!
//! - `incoming <link id>`: `<asking world's name>\t<its key>` if a request with that id waits for
//!   this world's owner, else empty;
//! - `answer <link id> <accept|always|decline|block>`: the owner's answer to it; `1`, or `!<why>`;
//! - `friend_worlds`: the owner's friends' worlds not linked with this one yet, a line each:
//!   `<key>\t<name>\t<friend's account endpoint>`;
//! - `ask_from_here <world key>\t<inbox>`: asks that world (its owner's inbox, if known) to link
//!   with this one; the link's id, or `!<why>`;
//! - `friends_ask`: `1` if friends' link requests are asked about too, else empty;
//!   `set_friends_ask <1|0>` sets it (`links::Store::friends_ask`).
//!
//! Asked anywhere (from a mod's client hooks too):
//! - `warm <world key>`: asks that world for its status through the player's own endpoint, in the
//!   background, so a trip there finds its way faster; `warmed <world key>` gives what it answered,
//!   `<name>\t<players>`, empty while unknown or offline.
//! - `my_worlds`: the player's own worlds, a line each: `<key>\t<name>`;
//! - `joined`: `<key>\t<name>` of the world joined last (the one the player is in, while visiting);
//! - `ask_link <own world key>\t<world key>`: asks that world, which must be the one joined last
//!   (never one a server names), to link with the player's own world; the link's id, or `!<why>`.
//!   Asked only when the player clicked for it (`notes/portals-easy-link.md`).

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
        "my_worlds" => {
            return crate::links::my_worlds()
                .into_iter()
                .map(|(k, n)| format!("{k}\t{}\n", n.replace(['\t', '\n'], " ")))
                .collect();
        }
        "joined" => {
            return crate::saved_worlds::joined_last()
                .map(|w| format!("{}\t{}", w.key, w.name.replace(['\t', '\n'], " ")))
                .unwrap_or_default();
        }
        "ask_link" => {
            let Some((from, to)) = arg.split_once('\t') else { return String::new() };
            let result = (|| {
                let joined = crate::saved_worlds::joined_last().ok_or("not in a world over the network")?;
                if joined.key != to {
                    return Err("that is not the world you are in".to_string());
                }
                let settings = crate::saved_worlds::settings().ok_or("the game is not ready")?;
                let key = crate::net::p2p::account_key(&settings)?;
                crate::links::request_from(&key, from, &joined.address, &joined.inbox, "")
            })();
            return result.unwrap_or_else(|e| format!("!{e}"));
        }
        _ => {}
    }
    let Some(info) = crate::server::current_info() else { return String::new() };
    match q {
        "world" => format!(
            "{}\t{}",
            info.run,
            info.dir
                .as_ref()
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default()
        ),
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
        "link_age" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::of_world(world)
                .iter()
                .find(|l| l.id() == arg)
                .map(|l| {
                    crate::saves::now_secs()
                        .saturating_sub(l.body.created)
                        .to_string()
                })
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
        "incoming" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::incoming(arg)
                .filter(|i| i.request.body.to == *world)
                .map(|i| format!("{}\t{}", i.request.from_name.replace(['\t', '\n'], " "), i.request.body.from))
                .unwrap_or_default()
        }
        "answer" => {
            let Some(world) = &info.world_key else { return String::new() };
            let (id, how) = arg.split_once(' ').unwrap_or((arg, ""));
            if crate::links::incoming(id).is_none_or(|i| i.request.body.to != *world) {
                return "!no such request".into();
            }
            let r = match how {
                "accept" => crate::links::accept(id, false),
                "always" => crate::links::accept(id, true),
                "decline" | "block" => {
                    crate::links::decline(id, how == "block");
                    Ok(())
                }
                _ => Err("no such answer".into()),
            };
            r.map_or_else(|e| format!("!{e}"), |()| "1".into())
        }
        "friend_worlds" => {
            let Some(world) = &info.world_key else { return String::new() };
            let linked: Vec<String> = crate::links::of_world(world)
                .iter()
                .map(|l| if l.body.from == *world { l.body.to.clone() } else { l.body.from.clone() })
                .collect();
            crate::friends::known_worlds()
                .into_iter()
                .filter(|(k, _, _)| k != world && !linked.contains(k))
                .map(|(k, n, a)| format!("{k}\t{}\t{a}\n", n.replace(['\t', '\n'], " ")))
                .collect()
        }
        "ask_from_here" => {
            let Some(world) = &info.world_key else { return String::new() };
            let (to, inbox) = arg.split_once('\t').unwrap_or((arg, ""));
            let result = (|| {
                if to.parse::<iroh::PublicKey>().is_err() {
                    return Err("that is not a world key".to_string());
                }
                let settings = crate::saved_worlds::settings().ok_or("the game is not ready")?;
                let key = crate::net::p2p::account_key(&settings)?;
                let address = crate::world_address::address(to, "", None, &[]);
                crate::links::request_from(&key, world, &address, inbox, "")
            })();
            result.unwrap_or_else(|e| format!("!{e}"))
        }
        "friends_ask" => {
            if crate::links::friends_ask() {
                "1".into()
            } else {
                String::new()
            }
        }
        "set_friends_ask" => {
            crate::links::set_friends_ask(arg == "1");
            "1".into()
        }
        "achievement" => {
            let mut f = arg.split('\t');
            let (Some(player), Some(id), Some(title)) = (f.next().and_then(|p| p.parse().ok()), f.next(), f.next()) else {
                return String::new();
            };
            let description = f.next().unwrap_or("").to_string();
            let icon = f.next().filter(|i| !i.is_empty()).map(str::to_string);
            info.shared
                .grants
                .lock()
                .push(crate::chiacraft::achievements::Grant { player, id: id.to_string(), title: title.to_string(), description, icon });
            String::new()
        }
        "data_dir" => {
            let Some(dir) = &info.dir else { return String::new() };
            if arg.is_empty()
                || !arg
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                return String::new();
            }
            let d = dir.join("mods").join(arg);
            match std::fs::create_dir_all(&d) {
                Ok(()) => d.to_string_lossy().into_owned(),
                Err(e) => {
                    log::warn!("mod data folder {}: {e}", d.display());
                    String::new()
                }
            }
        }
        "spot" | "unspot" => {
            let Some(dir) = &info.dir else { return String::new() };
            let mut f = arg.split(' ');
            let link = f.next().unwrap_or("");
            // `link x y z [dimension yaw]`: the portal's dimension by name and the way arrivals face.
            let p: Vec<i32> = f.by_ref().take(3).filter_map(|n| n.parse().ok()).collect();
            let dim = f.next().filter(|d| !d.is_empty()).map(str::to_string);
            let yaw = f
                .next()
                .and_then(|v| v.parse::<f32>().ok())
                .filter(|v| v.is_finite());
            let spot = match (q, &p[..]) {
                ("spot", [x, y, z]) => Some(([*x, *y, *z], dim, yaw)),
                ("spot", _) => return String::new(),
                _ => None,
            };
            crate::travel::set_spot(dir, link, spot);
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
