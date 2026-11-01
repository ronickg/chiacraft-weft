//! What chiacraft's own mods may ask the engine beyond Weft's mod interface (chiacraft plan,
//! phase 3). A mod that exports `chiacraft_bridge` is handed `query` when it loads, and calls it
//! with a question and an argument, as text, getting text back: the same way the `packages` mod is
//! handed packages, so Weft's interface stays as it is.
//!
//! Questions, asked on a server's threads (from a mod's server hooks):
//! - `owner <player entity>`: `1` if that player is the world's owner (their key is this game's
//!   account key), else empty;
//! - `links`: the world's links, a line each: `<link id>\t<other world's name>\t<other world's key>`;
//! - `link <id>`: `<other world's name>\t<other world's key>` if the world has that link, else empty.

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
    let Some(info) = crate::server::current_info() else { return String::new() };
    match q {
        "owner" => {
            let Ok(entity) = arg.parse::<u64>() else { return String::new() };
            let key = info.shared.players.read().iter().find(|(e, _)| *e == entity).map(|(_, k)| k.clone());
            let mine = chiacraft_account::current().map(|s| iroh::SecretKey::from_bytes(s.network_seed()).public().to_string());
            if key.is_some() && key == mine { "1".into() } else { String::new() }
        }
        "links" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::of_world(world).iter().map(|l| format!("{}\t{}\n", l.id(), other(world, l))).collect()
        }
        "link" => {
            let Some(world) = &info.world_key else { return String::new() };
            crate::links::of_world(world).iter().find(|l| l.id() == arg).map(|l| other(world, l)).unwrap_or_default()
        }
        _ => String::new(),
    }
}

/// The other world of a link of `world`: its name and key.
fn other(world: &str, l: &crate::links::Link) -> String {
    let (name, key) = if l.body.from == world { (&l.to_name, &l.body.to) } else { (&l.from_name, &l.body.from) };
    format!("{}\t{key}", name.replace(['\t', '\n'], " "))
}
