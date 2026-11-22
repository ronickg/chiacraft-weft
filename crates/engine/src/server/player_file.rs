//! A player coming and going: its file, joining, the mode it left in.

use super::*;
use weft_abi::entry_kind as ek;

pub(super) struct Arrival {
    pub data: PlayerData,
    clock: u64,
    started: Instant,
    pub ready: bool,
    /// Through a portal (chiacraft, `travel`): a few seconds before anything can hurt them.
    portal: bool,
}

impl Server {
    pub(super) fn prepare_join(&mut self, c: &'static Content, pi: usize, name: String, key: String) {
        let key = if key.is_empty() { name.clone() } else { key };
        let name = self.name_for(&name, &key);
        if self
            .players
            .iter()
            .any(|p| p.streams_world() && p.identity.key == key)
        {
            self.players[pi]
                .conn
                .send(ToClient::Disconnect(format!("{name} is already playing")));
            return;
        }
        // The world's admission rules and bans (chiacraft, `admission`, which has the allow list
        // in place of Weft's whitelist), read again when they change. The key is what the
        // connection proved, not what the client claims.
        let trusted = self.trusted(pi, &key);
        let p = &self.players[pi];
        let who = crate::admission::Who {
            name: name.clone(),
            key: key.clone(),
            identity: p.account.clone(),
            did: p.did.clone(),
            did_days: p.did_days,
            did_unchecked: p.did_unchecked,
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
            .and_then(|s| load_player(c, &s.lock(), &key, |dim| self.dims[dim as usize].time));
        // chiacraft, test only: an imported Minecraft world says where players start
        // (`chiacraft::mc_import::spawn`).
        let imported = self
            .storage
            .as_ref()
            .and_then(|s| crate::chiacraft::mc_import::spawn(c, &s.lock().meta.dir));
        let spawn = || imported.unwrap_or_else(|| (c.spawn_dim, self.dims[c.spawn_dim as usize].spawn_point(c)));
        let mut data = saved.unwrap_or_else(|| {
            let (dim, pos) = spawn();
            let mut entity = Entity::new(0, c.game.player_kind, pos, -1.0);
            entity.mode = self
                .storage
                .as_ref()
                .map_or(GameMode::SURVIVAL, |s| s.lock().meta.game_mode);
            PlayerData { dim: Some(dim), entity, inventory: None, components: Vec::new(), deaths: 0 }
        });
        let dim = match data.dim {
            Some(d) if d != c.void_dim || c.spawn_dim == c.void_dim => d,
            _ => {
                let (d, pos) = spawn();
                data.entity.pos = pos;
                log::warn!("player {name} was in a dimension that is not there now, placed at the spawn");
                d
            }
        };
        // Through a portal (chiacraft): beside this world's portal for that link.
        let arrived = self
            .storage
            .as_ref()
            .map(|s| s.lock().meta.dir.clone())
            .and_then(|d| crate::travel::arrival(&d, &key));
        let mut dim = dim;
        if let Some(at) = &arrived {
            // In the portal's own dimension (by name; the spawn's for spots kept before it was
            // saved, or if that dimension is gone), facing away from the portal.
            dim = match &at.dim {
                Some(n) => c.dims.iter().position(|d| d.name == *n).map_or_else(
                    || {
                        log::warn!("{name}'s portal is in dimension {n}, which is not there now: placed in the spawn's");
                        c.spawn_dim
                    },
                    |i| i as u32,
                ),
                None => c.spawn_dim,
            };
            data.entity.pos = glam::DVec3::from_array(at.pos);
            if let Some(yaw) = at.yaw {
                (data.entity.yaw, data.entity.pitch) = (yaw, 0.0);
            }
            log::info!(
                "{name} arrives through a portal at {:.0} {:.0} {:.0} in {}, facing {:.2}",
                data.entity.pos.x,
                data.entity.pos.y,
                data.entity.pos.z,
                c.dims[dim as usize].name,
                data.entity.yaw
            );
        }
        let pos = data.entity.pos;
        let look = [data.entity.yaw, data.entity.pitch];
        let p = &mut self.players[pi];
        p.identity = Arc::new(PlayerIdentity { name, key });
        p.dim = dim;
        let portal = arrived.is_some();
        p.arrival = Some(Arrival { data, clock: self.dims[dim as usize].time, started: Instant::now(), ready: false, portal });
        log::debug!(target: "load", "streaming spawn for {} before joining", p.identity.name);
        p.conn
            .send(ToClient::PrepareJoin { dim, pos, seed: self.cfg.seed, look });
        self.send_lod_columns(pi, dim);
        let p = &mut self.players[pi];
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
    }

    /// Activate after the simulation step, so this tick ends with a welcome and
    /// the player can begin moving on the next one.
    pub(super) fn finish_joins(&mut self, c: &'static Content) {
        for pi in 0..self.players.len() {
            let p = &mut self.players[pi];
            let Some(arrival) = &p.arrival else { continue };
            if p.conn.is_closed() {
                p.arrival = None;
            } else if arrival.ready {
                let arrival = p.arrival.take().unwrap();
                self.join(c, pi, arrival);
            } else if (self.cfg.dedicated || pi != 0) && arrival.started.elapsed() >= Duration::from_secs(60) {
                p.conn
                    .send(ToClient::Disconnect("world entry timed out".into()));
                p.conn.close();
                p.arrival = None;
            }
        }
    }

    fn join(&mut self, c: &'static Content, pi: usize, arrival: Arrival) {
        let dim = self.players[pi].dim;
        let portal = arrival.portal;
        let PlayerData { mut entity, inventory, mut components, deaths, .. } = arrival.data;
        let time = self.dims[dim as usize].time;
        extend::rebase_clock(c, &mut entity, &mut components, time as i64 - arrival.clock as i64);
        let pos = entity.pos;
        let r = self.dims[dim as usize].ensure_region(c, pos);
        let id = r.new_id();
        let mut e = entity;
        e.id = id;
        e.kind = c.game.player_kind;
        e.player = true;
        let flying = e.flying;
        let vel = e.vel;
        // The first connection is the one playing on this machine, the world's owner, with
        // commands if the world was made with cheats.
        let cheats = self.storage.as_ref().is_none_or(|s| s.lock().meta.cheats);
        let permission = if pi == 0 && !self.cfg.dedicated && cheats {
            abi::permission::OWNER
        } else {
            self.storage
                .as_ref()
                .and_then(|s| {
                    s.lock()
                        .meta
                        .ops
                        .iter()
                        .find(|o| o.0 == self.players[pi].identity.key)
                        .map(|o| o.1)
                })
                .unwrap_or(abi::permission::EVERYONE)
        };
        e.permission = permission;
        self.identities
            .write()
            .insert(id, self.players[pi].identity.clone());
        e.selected = e
            .selected
            .min(c.game.player_layout.hotbar.saturating_sub(1));
        let selected = e.selected;
        r.entities.insert_with(c, e, inventory, components);
        r.keep_sources(c, id);
        // Known only now that the modifiers are back; a new player (no saved health) starts full.
        let max = r.max_health(0, c, id);
        let mut dead = false;
        if let Some(e) = r.entities.get_mut(id) {
            if e.health < 0.0 {
                e.health = max;
            }
            // Out of a portal: a few seconds before anything can hurt them, to find their feet.
            if portal {
                e.hurt_cooldown = e.hurt_cooldown.max(ARRIVAL_PROTECTION);
            }
            dead = !e.alive();
        }
        r.emit_engine(0, c, abi::event::PLAYER_JOINED, &abi::PlayerEvent { player: id });
        let p = &mut self.players[pi];
        p.joined = true;
        p.entity = id;
        p.last_place = (None, pos);
        p.deaths = deaths;
        p.first_sync = true;
        match &p.account {
            Some(account) => log::info!("{} joined (account identity {account})", p.identity.name),
            None => log::info!("{} joined", p.identity.name),
        }
        p.conn
            .send(ToClient::Welcome { player: id, vel, selected, flying });
        if let Some(probation) = self.probation_of(pi) {
            let left = self.players[pi].probation_left(&probation);
            self.players[pi]
                .conn
                .send(ToClient::Show(crate::net::Shown::chat(Msg::new(
                    "admission.probation_on",
                    vec![Arg::Int(left.div_ceil(60) as i64)],
                ))));
        }
        // What came of their DID's check (chiacraft): checked, from the cache, not theirs, or
        // couldn't be asked: never left unsaid.
        if let Some(note) = self.players[pi].did_note.take() {
            self.players[pi]
                .conn
                .send(ToClient::Show(crate::net::Shown::chat(note)));
        }
        let p = &mut self.players[pi];
        let line = Msg::new("multiplayer.joined", vec![Arg::Text(p.identity.name.clone())]);
        let skin = p.skin.clone();
        let avatar = p.avatar.clone();
        for q in self.players.iter_mut().filter(|q| q.joined) {
            q.conn
                .send(ToClient::Show(crate::net::Shown::chat(line.clone())));
            if let Some(skin) = &skin {
                q.conn
                    .send(ToClient::Skin { entity: id, skin: (**skin).clone() });
            }
            if let Some(avatar) = &avatar {
                q.conn
                    .send(ToClient::Avatar { entity: id, avatar: (**avatar).clone() });
            }
        }
        let p = &mut self.players[pi];
        // Left while dead: still dead, as in vanilla.
        if dead {
            p.conn.send(ToClient::Died);
        }
        if dim == c.void_dim {
            p.conn
                .send(ToClient::Show(crate::net::Shown::chat(void_note(c))));
        }
        // The anchors of their achievements from this world (chiacraft), since they were here.
        self.send_anchor_proofs(pi);
        // The skins of who is here already.
        let skins: Vec<(u64, Arc<crate::net::Skin>)> = self
            .players
            .iter()
            .filter(|o| o.joined && o.entity != id)
            .filter_map(|o| o.skin.clone().map(|s| (o.entity, s)))
            .collect();
        for (entity, skin) in skins {
            self.players[pi]
                .conn
                .send(ToClient::Skin { entity, skin: (*skin).clone() });
        }
        let avatars: Vec<(u64, Arc<crate::net::AvatarData>)> = self
            .players
            .iter()
            .filter(|o| o.joined && o.entity != id)
            .filter_map(|o| o.avatar.clone().map(|a| (o.entity, a)))
            .collect();
        for (entity, avatar) in avatars {
            self.players[pi]
                .conn
                .send(ToClient::Avatar { entity, avatar: (*avatar).clone() });
        }
    }
}

pub(super) fn save_player(s: &Storage, p: &Player, d: &Dim) {
    let c = content();
    let path = s.player_path(&p.identity.key);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let Some(e) = d.entities.get(p.entity) else { return };
    let comps = d
        .entities
        .comps
        .iter()
        .filter(|((id, _), _)| *id == p.entity)
        .map(|((_, comp), bytes)| (*comp, bytes.as_ref()))
        .collect();
    let mut bytes = Vec::new();
    super::persist::write(c, &mut bytes, &[(e, d.entities.inventories.get(&p.entity), comps, None)], d.time);
    let mut text = format!(
        "name = {}\ndim = {}\ndeaths = {}\nentity = {}\n",
        p.identity.name,
        c.dims[p.dim as usize].name,
        p.deaths,
        crate::kv::hex(&bytes)
    );
    // chiacraft: who vouched for the key (to ban an absent player by identity), the time played
    // here (probation), and whether the owner let them off probation.
    if let Some(account) = &p.account {
        text.push_str(&format!("identity = {account}\n"));
    }
    if let Some(did) = &p.did {
        text.push_str(&format!("did = {did}\n"));
    }
    text.push_str(&format!("played = {}\n", p.played_now()));
    if p.lifted {
        text.push_str("probation = lifted\n");
    }
    // Whole or not at all: a crash in the middle must not leave half a player.
    let tmp = path.with_extension("new");
    if let Err(err) = std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, &path)) {
        log::error!("saving player {}: {err}", p.identity.name);
    }
}

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

pub(super) struct PlayerData {
    pub dim: Option<u32>,
    pub entity: Entity,
    pub inventory: Option<Vec<ItemStack>>,
    pub components: Vec<(u32, Box<[u8]>)>,
    pub deaths: u32,
}

/// Account metadata wraps the same entity record as cube saves.
pub(super) fn load_player(c: &Content, s: &Storage, key: &str, time: impl Fn(u32) -> u64) -> Option<PlayerData> {
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
    let dim = crate::kv::get(&src, "dim").and_then(|name| c.game.catalog.find(ek::DIMENSION, name));
    let destination = dim
        .filter(|&d| d != c.void_dim || c.spawn_dim == c.void_dim)
        .unwrap_or(c.spawn_dim);
    let now = time(destination);
    let bytes = crate::kv::unhex(crate::kv::get(&src, "entity").unwrap_or_default());
    let Some((entity, inventory, components, _)) = super::persist::read(c, &bytes, now).pop() else {
        log::error!("player {key}: invalid entity record; kept as .bak, the player starts anew");
        let _ = std::fs::copy(&path, path.with_extension("bak"));
        return None;
    };
    let deaths = crate::kv::get(&src, "deaths")
        .and_then(|d| d.parse().ok())
        .unwrap_or(0);
    Some(PlayerData { dim, entity, inventory, components, deaths })
}
