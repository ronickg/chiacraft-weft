//! A player coming and going: its file, joining, the mode it left in.

use super::*;

impl Server {
    pub(super) fn join(&mut self, c: &'static Content, pi: usize, name: String, key: String) {
        let key = if key.is_empty() { name.clone() } else { key };
        let name = self.name_for(&name, &key);
        if self.players.iter().any(|p| p.joined && p.key == key) {
            self.players[pi]
                .conn
                .send(ToClient::Disconnect(format!("{name} is already playing")));
            return;
        }
        // The world's admission rules and bans (chiacraft, `admission`), read again when they
        // change. The key is what the connection proved, not what the client claims.
        let trusted = self.trusted(pi, &key);
        let p = &self.players[pi];
        let who = crate::admission::Who {
            name: name.clone(),
            key: key.clone(),
            identity: p.identity.clone(),
            did: p.did.clone(),
            did_days: p.did_days,
            achievements: p.achievements.clone(),
        };
        if let crate::admission::Verdict::Refuse(why) = self.gate.verdict(&who, trusted) {
            log::info!("{name} ({key}) is not let in: {}", why.key);
            self.players[pi].conn.send(ToClient::Refused(why));
            return;
        }
        // Time played here, for probation, and whether the owner let them off it.
        let file = self
            .storage
            .as_ref()
            .map(|s| std::fs::read_to_string(s.lock().player_path(&key)).unwrap_or_default())
            .unwrap_or_default();
        self.players[pi].played = crate::kv::get(&file, "played")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        self.players[pi].lifted = crate::kv::get(&file, "probation") == Some("lifted");
        self.players[pi].since = Some(Instant::now());
        let saved = self
            .storage
            .as_ref()
            .and_then(|s| load_player(c, &s.lock(), &key));
        let spawn = || (c.spawn_dim, self.dims[c.spawn_dim as usize].spawn_point(c));
        let (mut dim, mut pos, health, inv, comps, look, selected, flying, fire) = match saved {
            // A dimension that is gone (its mod removed), or the void while a real one exists:
            // everything else is kept, the player starts at the spawn.
            Some(s) => {
                let ((dim, pos), keep) = match s.0 {
                    Some(d) if d != c.void_dim || c.spawn_dim == c.void_dim => ((d, s.1), true),
                    _ => (spawn(), false),
                };
                if !keep {
                    log::warn!("player {name} was in a dimension that is not there now, placed at the spawn");
                }
                (dim, pos, s.2, s.3, s.4, s.5, s.6, s.7, s.8)
            }
            None => {
                let (d, p) = spawn();
                (d, p, -1.0, None, Vec::new(), [0.0, 0.0], 0, false, 0)
            }
        };
        // Through a portal (chiacraft): beside this world's portal for that link.
        let arrived = self
            .storage
            .as_ref()
            .map(|s| s.lock().meta.dir.clone())
            .and_then(|d| crate::travel::arrival(&d, &key));
        if let Some(at) = arrived {
            dim = c.spawn_dim;
            pos = glam::DVec3::from_array(at);
            log::info!("{name} arrives through a portal at {:.0} {:.0} {:.0}", pos.x, pos.y, pos.z);
        }
        let time = self.dims[dim as usize].day_time();
        let r = self.dims[dim as usize].ensure_region(c, pos);
        let id = r.new_id();
        let mut e = Entity::new(id, c.game.player_kind, pos, health);
        e.player = true;
        e.mode = self
            .storage
            .as_ref()
            .map_or(GameMode::Survival, |s| saved_mode(&s.lock(), &key));
        (e.yaw, e.pitch) = (look[0], look[1]);
        (e.flying, e.fire) = (flying, fire);
        // The first connection is the one playing on this machine, the world's owner.
        let permission = if pi == 0 && !self.cfg.dedicated {
            abi::permission::OWNER
        } else {
            self.storage
                .as_ref()
                .and_then(|s| s.lock().meta.ops.iter().find(|o| o.0 == key).map(|o| o.1))
                .unwrap_or(abi::permission::EVERYONE)
        };
        e.permission = permission;
        e.selected = selected.min(c.game.player_layout.hotbar.saturating_sub(1));
        let selected = e.selected;
        let mode = e.mode;
        r.entities.insert_with(c, e, inv, comps);
        mode_modifiers(c, r, id, mode);
        let worn = worn_of(c, &r.entities, id);
        // Known only now that the modifiers are back; a new player (no saved health) starts full.
        let max = r.max_health(c, id);
        let mut dead = false;
        if let Some(e) = r.entities.get_mut(id) {
            e.worn = worn;
            if e.health < 0.0 {
                e.health = max;
            }
            // Out of a portal: a few seconds before anything can hurt them, to find their feet.
            if arrived.is_some() {
                e.hurt_cooldown = e.hurt_cooldown.max(ARRIVAL_PROTECTION);
            }
            dead = !e.alive();
        }
        r.emit_engine(c, abi::event::PLAYER_JOINED, &abi::PlayerEvent { player: id });
        let p = &mut self.players[pi];
        p.name = name;
        p.key = key;
        p.joined = true;
        p.dim = dim;
        p.entity = id;
        p.last_y = pos.y;
        p.first_sync = true;
        match &p.identity {
            Some(identity) => log::info!("{} joined (account identity {identity})", p.name),
            None => log::info!("{} joined", p.name),
        }
        if let Some(probation) = self.probation_of(pi) {
            let left = self.players[pi].probation_left(&probation);
            self.players[pi]
                .conn
                .send(ToClient::Chat(Msg::new("admission.probation_on", vec![Arg::Int(left.div_ceil(60) as i64)])));
        }
        let p = &mut self.players[pi];
        let line = Msg::new("multiplayer.joined", vec![Arg::Text(p.name.clone())]);
        for q in self.players.iter_mut().filter(|q| q.joined) {
            q.conn.send(ToClient::Chat(line.clone()));
        }
        let p = &mut self.players[pi];
        p.conn
            .send(ToClient::Welcome { player: id, dim, pos, seed: self.cfg.seed, look, selected, flying });
        p.conn.send(ToClient::Edits {
            dim,
            specs: self.dims[dim as usize]
                .edits
                .iter()
                .map(|e| e.spec(c))
                .collect(),
            all: true,
        });
        p.conn.send(ToClient::Configs(crate::config::synced()));
        let rate = self.dims[dim as usize].day_rate as f32;
        p.conn.send(ToClient::Time { dim, time, rate });
        p.conn.send(ToClient::GameMode(mode));
        p.conn.send(ToClient::Permission(permission));
        // Left while dead: still dead, as in vanilla.
        if dead {
            p.conn.send(ToClient::Died);
        }
        if dim == c.void_dim {
            p.conn.send(ToClient::Chat(void_note(c)));
        }
        // The anchors of their achievements from this world (chiacraft), since they were here.
        self.send_anchor_proofs(pi);
        // The skins of who is here already.
        let skins: Vec<(u64, Arc<Vec<u8>>)> = self
            .players
            .iter()
            .filter(|o| o.joined && o.entity != id)
            .filter_map(|o| o.skin.clone().map(|s| (o.entity, s)))
            .collect();
        for (entity, png) in skins {
            self.players[pi]
                .conn
                .send(ToClient::Skin { entity, png: png.to_vec() });
        }
    }
}

pub(super) fn save_player(s: &Storage, p: &Player, d: &Dim) {
    let c = content();
    let path = s.player_path(&p.key);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let Some(e) = d.entities.get(p.entity) else { return };
    let mut text = format!(
        "name = {}\ndim = {}\npos = {} {} {}\nlook = {} {}\nselected = {}\nhealth = {}\n",
        p.name, c.dims[p.dim as usize].name, e.pos.x, e.pos.y, e.pos.z, e.yaw, e.pitch, e.selected, e.health
    );
    text.push_str(&format!("flying = {}\nfire = {}\n", e.flying as u8, e.fire));
    // chiacraft: who vouched for the key (to ban an absent player by identity), the time played
    // here (probation), and whether the owner let them off probation.
    if let Some(identity) = &p.identity {
        text.push_str(&format!("identity = {identity}\n"));
    }
    if let Some(did) = &p.did {
        text.push_str(&format!("did = {did}\n"));
    }
    text.push_str(&format!("played = {}\n", p.played_now()));
    if p.lifted {
        text.push_str("probation = lifted\n");
    }
    if e.mode != GameMode::Survival {
        text.push_str(&format!("mode = {}\n", e.mode.name()));
    }
    if let Some(inv) = d.entities.inventories.get(&p.entity) {
        let items: Vec<String> = inv
            .iter()
            .map(|s| {
                if s.is_empty() {
                    "-".into()
                } else {
                    format!("{}*{}*{}", c.items[s.item as usize].name, s.count, crate::stack::to_text(crate::stack::for_save(s.data)))
                }
            })
            .collect();
        text.push_str(&format!("inventory = {}\n", items.join(" ")));
    }
    for ((id, comp), bytes) in &d.entities.comps {
        if *id == p.entity && c.game.components[*comp as usize].flags & cf::SAVE != 0 {
            text.push_str(&format!("comp.{} = {}\n", c.game.components[*comp as usize].name, persist::hex(bytes)));
        }
    }
    // Whole or not at all: a crash in the middle must not leave half a player.
    let tmp = path.with_extension("new");
    if let Err(err) = std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, &path)) {
        log::error!("saving player {}: {err}", p.name);
    }
}

pub(super) fn saved_mode(s: &Storage, key: &str) -> GameMode {
    let src = std::fs::read_to_string(s.player_path(key)).unwrap_or_default();
    crate::kv::get(&src, "mode")
        .and_then(GameMode::parse)
        .unwrap_or_default()
}

/// Dimension (`None`: unknown now), position, health, inventory, components.
pub(super) type PlayerData = (Option<u32>, DVec3, f32, Option<Vec<ItemStack>>, Vec<(u32, Box<[u8]>)>, [f32; 2], u32, bool, u32);

/// A saved player. Content that no longer exists is dropped with a log line; a file that cannot
/// be read is kept as `.bak` before it gets overwritten by a fresh player.
impl Server {
    /// The name a joining player goes by here (chiacraft): the one their game sent, without
    /// control characters, trimmed, at most 32 characters; an empty one (a game that sent none)
    /// becomes the name they had here last time, else `player-` and the start of their key, so
    /// nobody is ever nameless in the chat, the player list or the bans.
    fn name_for(&self, given: &str, key: &str) -> String {
        let clean = |s: &str| -> String {
            s.chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
                .trim()
                .chars()
                .take(32)
                .collect()
        };
        let name = clean(given);
        if !name.is_empty() {
            return name;
        }
        let before = self
            .storage
            .as_ref()
            .and_then(|s| std::fs::read_to_string(s.lock().player_path(key)).ok())
            .and_then(|f| crate::kv::get(&f, "name").map(clean))
            .filter(|n| !n.is_empty());
        let name = before.unwrap_or_else(|| format!("player-{}", key.chars().take(6).collect::<String>()));
        log::warn!("a player ({key}) joined with no name: called {name}");
        name
    }
}

pub(super) fn load_player(c: &Content, s: &Storage, key: &str) -> Option<PlayerData> {
    let path = s.player_path(key);
    let src = match std::fs::read_to_string(&path) {
        Ok(src) => src,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::error!("reading player {key}: {e}; kept as .bak, the player starts anew");
            let _ = std::fs::copy(&path, path.with_extension("bak"));
            return None;
        }
    };
    let (mut dim, mut pos, mut health, mut inv, mut comps) = (None, None, -1.0, None, Vec::new());
    let (mut look, mut selected, mut flying, mut fire) = ([0.0f32; 2], 0u32, false, 0u32);
    for (_, k, v) in crate::kv::pairs(&src) {
        match k {
            "dim" => {
                dim = c.dim_names.get(v).copied();
                if dim.is_none() {
                    log::warn!("player {key}: dimension {v} does not exist now");
                }
            }
            "look" => {
                let n: Vec<f32> = v
                    .split_whitespace()
                    .filter_map(|x| x.parse().ok())
                    .collect();
                if n.len() == 2 {
                    look = [n[0], n[1]];
                }
            }
            "selected" => selected = v.parse().unwrap_or(0),
            "flying" => flying = v == "1",
            "fire" => fire = v.parse().unwrap_or(0),
            "pos" => {
                let n: Vec<f64> = v
                    .split_whitespace()
                    .filter_map(|x| x.parse().ok())
                    .collect();
                if n.len() == 3 {
                    pos = Some(DVec3::new(n[0], n[1], n[2]));
                }
            }
            "health" => health = v.parse().unwrap_or(-1.0),
            "inventory" => {
                let mut slots = vec![ItemStack::EMPTY; c.game.player_layout.slots as usize];
                for (i, tok) in v.split_whitespace().enumerate().take(slots.len()) {
                    let parts: Vec<&str> = tok.split('*').collect();
                    if parts.len() == 3 {
                        let item = c.item_names.get(parts[0]).copied();
                        if item.is_none() {
                            log::warn!("player {key}: dropped {} {} (no such item now)", parts[1], parts[0]);
                        }
                        if let Some(item) = item {
                            slots[i] = ItemStack { item, count: parts[1].parse().unwrap_or(1), data: crate::stack::from_text(parts[2]) };
                        }
                    }
                }
                inv = Some(slots);
            }
            _ => {
                if let Some(name) = k.strip_prefix("comp.") {
                    let id = c.game.component(name);
                    if id.is_none() {
                        log::warn!("player {key}: dropped component {name} (not registered now)");
                    }
                    if let Some(id) = id {
                        let mut b = persist::unhex(v);
                        let size = c.game.components[id as usize].size as usize;
                        if size > 0 {
                            b.resize(size, 0);
                        }
                        comps.push((id, b.into_boxed_slice()));
                    }
                }
            }
        }
    }
    let Some(pos) = pos else {
        log::error!("player {key}: no position in the file; kept as .bak, the player starts anew");
        let _ = std::fs::copy(&path, path.with_extension("bak"));
        return None;
    };
    Some((dim, pos, health, inv, comps, look, selected, flying, fire))
}
