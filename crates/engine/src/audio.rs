//! Sound: events (`registry::SoundDef`) play one of their files, positioned in the world or not;
//! every category has its own volume (`core:volume_*` settings). World sounds go through one
//! track whose low-pass filter muffles them under water and whose send feeds a reverb in caves.
//! Music streams on its own track after quiet stretches. Without an audio device everything
//! here does nothing.
//!
//! Every sound is a voice. A client may start one with a handle (`start`), loop it, and ease its
//! volume, pitch, place and muffle while it plays (`change`), then fade it out (`stop`): weather
//! beds, fire, a creature's moving call. At most `VOICES` play at once; a new one replaces the
//! one that matters least (`SoundDef::priority`, then how loud it is where the listener is) or is
//! not played, and an event may cap its own (`SoundDef::voices`).
//!
//! Files are read on their own thread, never the frame's: a small one is decoded whole (once,
//! kept while memory allows), a large one streams. A file that is too large, broken or panics
//! its decoder logs once and plays nothing.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

use glam::{DVec3, Quat};
use kira::effect::filter::{FilterBuilder, FilterHandle};
use kira::effect::reverb::ReverbBuilder;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::{FromFileError, PlaybackState};
use kira::track::{SendTrackBuilder, SendTrackHandle, SpatialTrackBuilder, SpatialTrackHandle, TrackBuilder, TrackHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Mix, Tween};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::assets::Assets;
use crate::content::content;

/// Sounds playing at once, at most (music aside).
pub const VOICES: usize = 64;
/// Of them streaming from disk.
pub const STREAMS: usize = 8;
/// Files larger than this stream instead of being decoded whole.
const STREAM_ABOVE: u64 = 256 * 1024;
/// Files larger than this are refused (data packages come from other players).
const MAX_FILE: u64 = 32 << 20;
/// Decoded audio kept for reuse, in frames (8 bytes each): 128 MiB. Past it the cache is
/// emptied; playing sounds keep their own copy.
const CACHE_FRAMES: usize = 16 << 20;
/// A one-shot whose file took longer than this to read is dropped: heard late is wrong.
/// Loops start whenever theirs is ready.
const LATE: Duration = Duration::from_millis(300);
/// How fast a sound that gives way to a more important one fades out.
const STEAL: Tween = Tween { start_time: kira::StartTime::Immediate, duration: Duration::from_millis(60), easing: kira::Easing::Linear };

/// A sound to play this frame: event id, position (`None` = not positioned), volume, pitch (0 =
/// the event's random range).
#[derive(Clone, Copy, Debug)]
pub struct Play {
    pub sound: u32,
    pub pos: Option<DVec3>,
    pub volume: f32,
    pub pitch: f32,
}

/// Where the listener is, for positions and the environment.
pub struct Listener {
    pub pos: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub underwater: bool,
    /// No sky light at the head: caves echo.
    pub enclosed: bool,
    pub dim: u32,
    /// The conditions music asks for that hold now, by id.
    pub holds: Vec<u32>,
}

/// A file as far as it has been read.
enum File {
    Loading,
    Ready(Box<StaticSoundData>),
    /// Too large to decode whole: each voice opens its own stream.
    Stream(PathBuf),
    Failed,
}

/// Work for the reading thread.
enum Job {
    Decode(String, PathBuf),
    /// Opens a stream for voice `id`.
    Open(u64, PathBuf),
}

enum Done {
    Decoded(String, Option<StaticSoundData>),
    Opened(u64, Option<StreamingSoundData<FromFileError>>),
}

enum Handle {
    Static(StaticSoundHandle),
    Stream(StreamingSoundHandle<FromFileError>),
}

impl Handle {
    fn state(&self) -> PlaybackState {
        match self {
            Handle::Static(h) => h.state(),
            Handle::Stream(h) => h.state(),
        }
    }
    fn set_volume(&mut self, v: Decibels, t: Tween) {
        match self {
            Handle::Static(h) => h.set_volume(v, t),
            Handle::Stream(h) => h.set_volume(v, t),
        }
    }
    fn set_rate(&mut self, r: f64, t: Tween) {
        match self {
            Handle::Static(h) => h.set_playback_rate(r, t),
            Handle::Stream(h) => h.set_playback_rate(r, t),
        }
    }
    fn stop(&mut self, t: Tween) {
        match self {
            Handle::Static(h) => h.stop(t),
            Handle::Stream(h) => h.stop(t),
        }
    }
}

/// The voice's own track: needed to move it or muffle it.
enum Track {
    None,
    Plain(TrackHandle),
    Spatial(SpatialTrackHandle),
}

/// What a voice waits for before it starts.
struct Wait {
    file: String,
    asked: Instant,
}

/// How many started sounds `/sounds recent` remembers.
const RECENT: usize = 40;

struct Voice {
    id: u64,
    sound: u32,
    /// Started with a handle a mod holds: it gets a track of its own (moves, muffles).
    held: bool,
    looped: bool,
    stream: bool,
    priority: i32,
    /// Times the event's volume, as asked.
    volume: f32,
    pitch: f64,
    fade: f32,
    pos: Option<DVec3>,
    /// Blocks away it is still heard.
    reach: f32,
    /// Blocks within which it is at full volume (`SoundDesc::full_within`), and the curve it
    /// fades by beyond (`falloff`).
    near: f32,
    falloff: i32,
    /// Asked for before it started (its file was being read).
    lowpass: f32,
    started: Instant,
    track: Track,
    filter: Option<FilterHandle>,
    handle: Option<Handle>,
    wait: Option<Wait>,
}

pub struct Audio {
    mgr: AudioManager<DefaultBackend>,
    listener: kira::listener::ListenerHandle,
    ear: DVec3,
    reverb: SendTrackHandle,
    world: TrackHandle,
    lowpass: FilterHandle,
    /// Per category (`sound_category`): its track.
    groups: Vec<TrackHandle>,
    files: FxHashMap<String, File>,
    /// Frames decoded and kept in `files`.
    cached: usize,
    /// Files of a `name*` pattern.
    globs: FxHashMap<String, Vec<String>>,
    /// The file each event played last, so the next is another (round robin).
    last: FxHashMap<u32, String>,
    jobs: Option<Sender<Job>>,
    done: Option<Receiver<Done>>,
    voices: Vec<Voice>,
    /// Handles given out and not ended: answered at once, before their voice starts.
    live: FxHashSet<u64>,
    next_id: u64,
    /// Asked for by mods since the last frame (id, sound, looped, fade, low-pass): started at
    /// the next.
    queue: Vec<(u64, Play, bool, f32, f32)>,
    music: Option<StreamingSoundHandle<FromFileError>>,
    next_music: Instant,
    state: (bool, bool),
    /// Per category, then the master volume.
    volumes: Vec<f64>,
    rng: weft_noise::Rng,
    /// Sounds started so far.
    pub played: u32,
    /// The last sounds started, newest last (name, volume, looped): `/sounds recent`, for checking
    /// one-shot steps and calls after the fact.
    recent: std::collections::VecDeque<(String, f32, bool)>,
    /// Sounds not played or cut short for more important ones.
    pub dropped: u32,
    /// Automated runs (tests, scripts): everything plays and counts, silently.
    pub muted: bool,
    /// Each category's volume setting id, then the master volume's, looked up once.
    volume_ids: Option<Vec<Option<u32>>>,
    /// Every event's files were looked for (`check_files`).
    checked: bool,
}

fn sound_path(assets: &Assets, name: &str) -> Option<PathBuf> {
    assets
        .path(name, "sounds", "ogg")
        .or_else(|| assets.path(name, "sounds", "wav"))
}

const QUICK: Tween = Tween { start_time: kira::StartTime::Immediate, duration: Duration::from_millis(400), easing: kira::Easing::Linear };

fn tween(secs: f32) -> Tween {
    let secs = if secs.is_finite() { secs.clamp(0.0, 600.0) } else { 0.0 };
    Tween { duration: Duration::from_secs_f32(secs), ..Default::default() }
}

fn db(volume: f64) -> Decibels {
    if volume <= 0.001 { Decibels::SILENCE } else { Decibels((20.0 * volume.log10()) as f32) }
}

/// Reads files for `Audio` on its own thread. A decoder that panics on a broken file is caught:
/// the file plays nothing.
fn reader(jobs: Receiver<Job>, done: Sender<Done>) {
    let guard = |what: &str, f: &mut dyn FnMut() -> Result<(), String>| match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(())) => {}
        Ok(Err(e)) => log::warn!("sound {what}: {e}"),
        Err(_) => log::warn!("sound {what}: its decoder failed, it plays nothing"),
    };
    for job in jobs {
        let sent = match job {
            Job::Decode(name, path) => {
                let mut data = None;
                guard(&name, &mut || {
                    data = Some(StaticSoundData::from_file(&path).map_err(|e| e.to_string())?);
                    Ok(())
                });
                done.send(Done::Decoded(name, data))
            }
            Job::Open(id, path) => {
                let mut data = None;
                guard(&path.display().to_string(), &mut || {
                    data = Some(StreamingSoundData::from_file(&path).map_err(|e| e.to_string())?);
                    Ok(())
                });
                done.send(Done::Opened(id, data))
            }
        };
        if sent.is_err() {
            return;
        }
    }
}

impl Audio {
    pub fn new() -> Option<Audio> {
        let mut mgr = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("no sound: {e}");
                return None;
            }
        };
        let (job_tx, job_rx) = channel();
        let (done_tx, done_rx) = channel();
        let reading = std::thread::Builder::new()
            .name("sound files".into())
            .spawn(move || reader(job_rx, done_tx))
            .map_err(|e| log::warn!("no sound file thread: {e}"))
            .is_ok();
        let made = (|| -> Result<Audio, kira::ResourceLimitReached> {
            let listener = mgr.add_listener(glam::Vec3::ZERO, Quat::IDENTITY)?;
            let reverb = mgr.add_send_track(
                SendTrackBuilder::new().with_effect(
                    ReverbBuilder::new()
                        .mix(Mix::WET)
                        .damping(0.6)
                        .feedback(0.85),
                ),
            )?;
            let mut wb = TrackBuilder::new()
                .sub_track_capacity(64)
                .with_send(&reverb, Decibels::SILENCE);
            let lowpass = wb.add_effect(FilterBuilder::new().cutoff(20_000.0));
            let mut world = mgr.add_sub_track(wb)?;
            let mut groups = Vec::new();
            for k in &crate::content::content().game.sound_categories {
                let b = TrackBuilder::new()
                    .sub_track_capacity(256)
                    .sound_capacity(256);
                groups.push(if k.world { world.add_sub_track(b)? } else { mgr.add_sub_track(b)? });
            }
            Ok(Audio {
                mgr,
                listener,
                ear: DVec3::ZERO,
                reverb,
                world,
                lowpass,
                groups,
                files: FxHashMap::default(),
                cached: 0,
                globs: FxHashMap::default(),
                last: FxHashMap::default(),
                jobs: reading.then_some(job_tx),
                done: reading.then_some(done_rx),
                voices: Vec::new(),
                live: FxHashSet::default(),
                next_id: 1,
                queue: Vec::new(),
                music: None,
                next_music: Instant::now() + Duration::from_secs(20),
                state: (false, false),
                volumes: Vec::new(),
                rng: weft_noise::Rng(0x2545_F491_4F6C_DD1D),
                played: 0,
                recent: std::collections::VecDeque::new(),
                dropped: 0,
                muted: false,
                volume_ids: None,
                checked: false,
            })
        })();
        match made {
            Ok(a) => Some(a),
            Err(e) => {
                log::warn!("no sound: {e:?}");
                None
            }
        }
    }

    fn random(&mut self) -> f32 {
        self.rng.float()
    }

    /// Forgets read files, for packs that changed them (F3+T).
    pub fn reload(&mut self) {
        self.files.retain(|_, f| matches!(f, File::Loading));
        self.cached = 0;
        self.globs.clear();
    }

    /// Sounds playing now, and of them those a mod holds a handle to.
    pub fn counts(&self) -> (u32, u32) {
        (self.voices.len() as u32, self.voices.iter().filter(|v| v.held).count() as u32)
    }

    /// `/sounds recent`: the last sounds started, newest first (name, volume, looped).
    pub fn recent(&self) -> Vec<(String, f32, bool)> {
        self.recent.iter().rev().cloned().collect()
    }

    /// `/sounds`: what plays now, most important first.
    pub fn describe(&self) -> Vec<(String, f32, bool, bool)> {
        let c = content();
        let mut v: Vec<&Voice> = self.voices.iter().collect();
        v.sort_by(|a, b| {
            self.importance(b)
                .partial_cmp(&self.importance(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v.iter()
            .map(|v| {
                let name = c
                    .game
                    .sounds
                    .get(v.sound as usize)
                    .map_or_else(String::new, |d| d.name.clone());
                (name, v.volume, v.looped, v.wait.is_some())
            })
            .collect()
    }

    /// A sound with a handle, started at the next frame (`client_sound_start`).
    pub fn start(&mut self, p: Play, looped: bool, fade: f32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.live.insert(id);
        self.queue.push((id, p, looped, fade, 20_000.0));
        id
    }

    pub fn playing(&self, id: u64) -> bool {
        self.live.contains(&id)
    }

    /// Eases a held sound's volume, pitch, place or muffle.
    pub fn change(&mut self, id: u64, ch: &weft_abi::SoundChange) {
        use weft_abi::sound_change as sc;
        let Some(v) = self.voices.iter_mut().find(|v| v.id == id) else {
            // Still in the queue: start it as changed.
            if let Some(q) = self.queue.iter_mut().find(|q| q.0 == id) {
                if ch.what & sc::VOLUME != 0 {
                    q.1.volume = ch.volume;
                }
                if ch.what & sc::PITCH != 0 {
                    q.1.pitch = ch.pitch;
                }
                if ch.what & sc::POS != 0 && q.1.pos.is_some() {
                    q.1.pos = Some(DVec3::from_array(ch.pos));
                }
                if ch.what & sc::LOWPASS != 0 && ch.lowpass.is_finite() {
                    q.4 = ch.lowpass.clamp(20.0, 20_000.0);
                }
            }
            return;
        };
        let t = tween(ch.secs);
        let def_volume = content()
            .game
            .sounds
            .get(v.sound as usize)
            .map_or(1.0, |d| d.volume);
        if ch.what & sc::VOLUME != 0 && ch.volume.is_finite() {
            v.volume = ch.volume.max(0.0);
            if let Some(h) = &mut v.handle {
                h.set_volume(db((v.volume * def_volume) as f64), t);
            }
        }
        if ch.what & sc::PITCH != 0 && ch.pitch.is_finite() && ch.pitch > 0.0 {
            v.pitch = ch.pitch.clamp(0.05, 8.0) as f64;
            if let Some(h) = &mut v.handle {
                h.set_rate(v.pitch, t);
            }
        }
        if ch.what & sc::POS != 0 && ch.pos.iter().all(|c| c.is_finite()) {
            let p = DVec3::from_array(ch.pos);
            if v.pos.is_some() {
                v.pos = Some(p);
            }
            if let Track::Spatial(tr) = &mut v.track {
                tr.set_position(p.as_vec3(), t);
            }
        }
        if ch.what & sc::LOWPASS != 0 && ch.lowpass.is_finite() {
            v.lowpass = ch.lowpass.clamp(20.0, 20_000.0);
            if let Some(f) = &mut v.filter {
                f.set_cutoff(v.lowpass as f64, t);
            }
        }
    }

    /// Fades a held sound out over `fade` seconds.
    pub fn stop(&mut self, id: u64, fade: f32) {
        self.live.remove(&id);
        self.queue.retain(|q| q.0 != id);
        if let Some(i) = self.voices.iter().position(|v| v.id == id) {
            let mut v = self.voices.swap_remove(i);
            if let Some(h) = &mut v.handle {
                h.stop(tween(fade.max(0.01)));
            }
        }
    }

    /// How much a voice matters: its priority, then how loud it is at the listener.
    fn importance(&self, v: &Voice) -> (i32, f32) {
        (v.priority, audible(v.volume, v.pos, v.near, v.reach, self.ear))
    }

    /// Makes room for a sound of event `sound` that matters `new`: false if it should not play.
    fn room(&mut self, sound: u32, cap: u32, stream: bool, new: (i32, f32)) -> bool {
        let less = |a: (i32, f32), b: (i32, f32)| a.0 < b.0 || (a.0 == b.0 && a.1 < b.1);
        let take = |a: &mut Audio, pick: &dyn Fn(&Voice) -> bool, ties: bool| -> bool {
            // The one that matters least, the oldest of equals.
            let victim = a
                .voices
                .iter()
                .enumerate()
                .filter(|(_, v)| pick(v))
                .min_by(|(_, x), (_, y)| {
                    let (ix, iy) = (a.importance(x), a.importance(y));
                    ix.partial_cmp(&iy)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(x.started.cmp(&y.started))
                })
                .map(|(i, v)| (i, a.importance(v)));
            match victim {
                Some((i, imp)) if less(imp, new) || (ties && imp == new) => {
                    let mut v = a.voices.swap_remove(i);
                    if let Some(h) = &mut v.handle {
                        h.stop(STEAL);
                    }
                    a.live.remove(&v.id);
                    a.dropped += 1;
                    true
                }
                _ => false,
            }
        };
        if cap > 0 && self.voices.iter().filter(|v| v.sound == sound).count() >= cap as usize && !take(self, &|v| v.sound == sound, true) {
            return false;
        }
        if stream && self.voices.iter().filter(|v| v.stream).count() >= STREAMS && !take(self, &|v| v.stream, false) {
            return false;
        }
        self.voices.len() < VOICES || take(self, &|_| true, false)
    }

    /// The file of event `sound` to play now: another than last time when it has several.
    fn pick(&mut self, assets: &Assets, sound: u32) -> Option<String> {
        let c = content();
        let def = c.game.sounds.get(sound as usize)?;
        let mut names: Vec<String> = Vec::new();
        for f in &def.files {
            // `hurt*`: the files whose names start so, found once.
            if let Some(prefix) = f.strip_suffix('*') {
                let found = self.globs.entry(f.clone()).or_insert_with(|| {
                    let mut v: Vec<String> = ["ogg", "wav"]
                        .iter()
                        .flat_map(|e| assets.list("sounds", e))
                        .map(|f| f.0)
                        .filter(|n| n.starts_with(prefix))
                        .collect();
                    v.sort();
                    v.dedup();
                    if v.is_empty() {
                        log::warn!("no sound files {f}");
                    }
                    v
                });
                names.extend(found.iter().cloned());
            } else {
                names.push(f.clone());
            }
        }
        let last = self.last.get(&sound);
        if names.len() > 1 {
            names.retain(|n| Some(n) != last);
        }
        if names.is_empty() {
            return None;
        }
        let i = (self.random() * names.len() as f32) as usize % names.len();
        let name = names.swap_remove(i);
        self.last.insert(sound, name.clone());
        Some(name)
    }

    /// Where file `name` stands, asking the reading thread for it the first time.
    fn file(&mut self, assets: &Assets, name: &str) -> &File {
        if !self.files.contains_key(name) {
            let f = match sound_path(assets, name) {
                None => {
                    log::warn!("missing sound {name}");
                    File::Failed
                }
                Some(path) => match std::fs::metadata(&path).map(|m| m.len()) {
                    Err(e) => {
                        log::warn!("sound {name}: {e}");
                        File::Failed
                    }
                    Ok(n) if n > MAX_FILE => {
                        log::warn!("sound {name}: {} MiB is too large, it plays nothing", n >> 20);
                        File::Failed
                    }
                    Ok(n) if n > STREAM_ABOVE => File::Stream(path),
                    Ok(_) => match &self.jobs {
                        Some(j) if j.send(Job::Decode(name.to_string(), path)).is_ok() => File::Loading,
                        _ => File::Failed,
                    },
                },
            };
            self.files.insert(name.to_string(), f);
        }
        &self.files[name]
    }

    /// Logs, once, the sound events with a file that isn't there (`sound files:` in the log).
    fn check_files(&mut self, assets: &Assets) {
        self.checked = true;
        let c = content();
        let mut missing: Vec<String> = Vec::new();
        // The folders are walked once, not once a pattern.
        let listed: Vec<String> = ["ogg", "wav"]
            .iter()
            .flat_map(|e| assets.list("sounds", e))
            .map(|f| f.0)
            .collect();
        for d in &c.game.sounds {
            for f in &d.files {
                let found = match f.strip_suffix('*') {
                    Some(prefix) => listed.iter().any(|n| n.starts_with(prefix)),
                    None => sound_path(assets, f).is_some(),
                };
                if !found {
                    missing.push(format!("{} ({f})", d.name));
                }
            }
        }
        if missing.is_empty() {
            log::info!("sound files: all {} events have their files", c.game.sounds.len());
        } else {
            log::warn!("sound files: {} of {} events miss files: {}", missing.len(), c.game.sounds.len(), missing.join(", "));
        }
    }

    /// Settings, listener, environment, music, files read and sounds asked for, once a frame.
    pub fn update(&mut self, assets: &Assets, l: Option<&Listener>) {
        let ids = self.volume_ids.get_or_insert_with(|| {
            let mut ids: Vec<Option<u32>> = crate::content::content()
                .game
                .sound_categories
                .iter()
                .map(|k| k.setting.as_deref().and_then(crate::config::by_name))
                .collect();
            ids.push(crate::config::by_name("core:volume_master"));
            ids
        });
        // UI has no setting of its own: full volume.
        let mut volumes: Vec<f64> = ids
            .iter()
            .map(|id| id.map_or(1.0, |id| crate::config::get(id) / 100.0))
            .collect();
        if self.muted {
            *volumes.last_mut().unwrap() = 0.0;
        }
        if volumes != self.volumes {
            for (g, v) in self.groups.iter_mut().zip(&volumes) {
                g.set_volume(db(*v), Tween::default());
            }
            self.mgr
                .main_track()
                .set_volume(db(*volumes.last().unwrap()), Tween::default());
            self.volumes = volumes;
        }
        if !self.checked {
            self.check_files(assets);
        }
        self.read_done();
        // Ended sounds give back their room and their handles.
        let live = &mut self.live;
        self.voices.retain(|v| {
            let on = v
                .handle
                .as_ref()
                .is_none_or(|h| h.state() != PlaybackState::Stopped);
            if !on {
                live.remove(&v.id);
            }
            on
        });
        let Some(l) = l else {
            if let Some(m) = &mut self.music {
                m.stop(Tween { duration: Duration::from_secs(2), ..Default::default() });
                self.music = None;
            }
            // Out of the world: what mods started there ends.
            for v in &mut self.voices {
                if let Some(h) = &mut v.handle {
                    h.stop(QUICK);
                }
            }
            self.voices.clear();
            self.live.clear();
            self.queue.clear();
            return;
        };
        let orientation = Quat::from_rotation_y(l.yaw) * Quat::from_rotation_x(l.pitch);
        self.ear = l.pos;
        self.listener
            .set_position(l.pos.as_vec3(), Tween::default());
        self.listener.set_orientation(orientation, Tween::default());
        if (l.underwater, l.enclosed) != self.state {
            self.state = (l.underwater, l.enclosed);
            self.lowpass
                .set_cutoff(if l.underwater { 900.0 } else { 20_000.0 }, QUICK);
            if let Err(e) =
                self.world
                    .set_send(&self.reverb, if l.enclosed && !l.underwater { Decibels(-9.0) } else { Decibels::SILENCE }, QUICK)
            {
                log::debug!("cave reverb: {e:?}");
            }
        }
        for (id, p, looped, fade, lowpass) in std::mem::take(&mut self.queue) {
            self.begin(assets, id, p, looped, fade, lowpass);
        }
        self.music(assets, l);
    }

    /// Takes what the reading thread finished and starts the sounds that waited for it.
    fn read_done(&mut self) {
        let Some(rx) = &self.done else { return };
        let done: Vec<Done> = rx.try_iter().collect();
        for d in done {
            match d {
                Done::Decoded(name, data) => {
                    let f = match data {
                        Some(d) => {
                            self.cached += d.num_frames();
                            File::Ready(Box::new(d))
                        }
                        None => File::Failed,
                    };
                    let ready = match &f {
                        File::Ready(d) => Some((**d).clone()),
                        _ => None,
                    };
                    self.files.insert(name.clone(), f);
                    let waiting: Vec<u64> = self
                        .voices
                        .iter()
                        .filter(|v| v.wait.as_ref().is_some_and(|w| w.file == name))
                        .map(|v| v.id)
                        .collect();
                    for id in waiting {
                        match &ready {
                            Some(d) => self.go(id, |v, data| v.play_static(data), d.clone()),
                            None => self.forget(id),
                        }
                    }
                }
                Done::Opened(id, data) => match data {
                    Some(d) => self.go(id, |v, data| v.play_stream(data), d),
                    None => self.forget(id),
                },
            }
        }
        if self.cached > CACHE_FRAMES {
            self.files.retain(|_, f| !matches!(f, File::Ready(_)));
            self.cached = 0;
        }
    }

    fn forget(&mut self, id: u64) {
        self.voices.retain(|v| v.id != id);
        self.live.remove(&id);
    }

    /// Starts waiting voice `id` with its data, unless it waited too long for a one-shot.
    fn go<D>(&mut self, id: u64, play: impl FnOnce(&mut Voice, D) -> Result<Handle, String>, data: D) {
        let Some(i) = self.voices.iter().position(|v| v.id == id) else { return };
        if !self.voices[i].looped
            && self.voices[i]
                .wait
                .as_ref()
                .is_some_and(|w| w.asked.elapsed() > LATE)
        {
            self.forget(id);
            self.dropped += 1;
            return;
        }
        let g = self.category(self.voices[i].sound);
        let (listener, group) = (&self.listener, &mut self.groups[g]);
        let v = &mut self.voices[i];
        if let Err(e) = v.make_track(group, listener) {
            log::debug!("sound {id}: {e}");
            self.forget(id);
            return;
        }
        let (sound, volume, looped) = (v.sound, v.volume, v.looped);
        match play(v, data) {
            Ok(h) => {
                v.handle = Some(h);
                v.wait = None;
                v.started = Instant::now();
                self.played += 1;
                let name = content()
                    .game
                    .sounds
                    .get(sound as usize)
                    .map_or_else(String::new, |d| d.name.clone());
                if self.recent.len() >= RECENT {
                    self.recent.pop_front();
                }
                self.recent.push_back((name, volume, looped));
            }
            Err(e) => {
                log::debug!("sound {id}: {e}");
                self.forget(id);
            }
        }
    }

    fn category(&self, sound: u32) -> usize {
        let c = content();
        let cat = c.game.sounds.get(sound as usize).map_or(0, |d| d.category);
        (cat as usize).min(self.groups.len().saturating_sub(1))
    }

    /// A one-shot nobody holds a handle to.
    pub fn play(&mut self, assets: &Assets, p: Play) {
        let id = self.next_id;
        self.next_id += 1;
        self.begin(assets, id, p, false, 0.0, 20_000.0);
    }

    fn begin(&mut self, assets: &Assets, id: u64, p: Play, looped: bool, fade: f32, lowpass: f32) {
        let held = self.live.contains(&id);
        let c = content();
        let Some(def) = c.game.sounds.get(p.sound as usize) else {
            self.live.remove(&id);
            return;
        };
        if def.files.is_empty() || !p.volume.is_finite() || p.pos.is_some_and(|q| !q.is_finite()) {
            self.live.remove(&id);
            return;
        }
        let Some(name) = self.pick(assets, p.sound) else {
            self.live.remove(&id);
            return;
        };
        let volume = p.volume.max(0.0);
        let pitch = if p.pitch > 0.0 { p.pitch } else { def.pitch[0] + (def.pitch[1] - def.pitch[0]) * self.random() };
        // Heard up to `range` blocks (further for loud ones), full volume within `full_within`.
        let reach = (def.range * (volume * def.volume).max(1.0)).max(1.5);
        let near = def.full_within.clamp(0.0, reach - 0.5);
        let stream = matches!(self.file(assets, &name), File::Stream(_));
        if !self.room(p.sound, def.voices, stream, (def.priority, audible(volume, p.pos, near, reach, self.ear))) {
            log::debug!("sound {}: no room", def.name);
            self.live.remove(&id);
            self.dropped += 1;
            return;
        }
        self.voices.push(Voice {
            id,
            sound: p.sound,
            held,
            looped,
            stream,
            priority: def.priority,
            volume,
            pitch: pitch.clamp(0.05, 8.0) as f64,
            fade,
            pos: p.pos,
            reach,
            near,
            falloff: def.falloff,
            lowpass,
            started: Instant::now(),
            track: Track::None,
            filter: None,
            handle: None,
            wait: Some(Wait { file: name.clone(), asked: Instant::now() }),
        });
        let open = match self.file(assets, &name) {
            File::Ready(d) => Ok(Some((**d).clone())),
            File::Stream(path) => Err(Some(path.clone())),
            File::Loading => Ok(None),
            File::Failed => Err(None),
        };
        match open {
            Ok(Some(d)) => self.go(id, |v, data| v.play_static(data), d),
            Ok(None) => {}
            Err(Some(path)) => {
                if self
                    .jobs
                    .as_ref()
                    .is_none_or(|j| j.send(Job::Open(id, path)).is_err())
                {
                    self.forget(id);
                }
            }
            Err(None) => self.forget(id),
        }
    }

    fn music(&mut self, assets: &Assets, l: &Listener) {
        if self
            .music
            .as_ref()
            .is_some_and(|m| m.state() != kira::sound::PlaybackState::Stopped)
            || Instant::now() < self.next_music
        {
            return;
        }
        self.music = None;
        let c = content();
        let fits = |m: &crate::registry::MusicDef| m.dimension.is_none_or(|d| d == l.dim) && m.when.iter().all(|w| l.holds.contains(w));
        // Quiet for a few minutes between tracks, like vanilla's (shorter: there is less music).
        self.next_music = Instant::now() + Duration::from_secs(120 + (self.random() * 180.0) as u64);
        let Some(m) = c
            .game
            .music
            .iter()
            .find(|m| fits(m) && !m.tracks.is_empty())
        else {
            return;
        };
        let track = m.tracks[(self.random() * m.tracks.len() as f32) as usize % m.tracks.len()].clone();
        let Some(path) = sound_path(assets, &track) else {
            log::warn!("missing music {track}");
            return;
        };
        if std::fs::metadata(&path).is_ok_and(|m| m.len() > MAX_FILE * 4) {
            log::warn!("music {track}: too large, skipped");
            return;
        }
        // Opening only reads the header; the stream decodes on kira's own thread.
        match StreamingSoundData::from_file(&path) {
            Ok(data) => match self.groups[crate::registry::MUSIC_CATEGORY as usize]
                .play(data.fade_in_tween(Tween { duration: Duration::from_secs(3), ..Default::default() }))
            {
                Ok(h) => {
                    log::debug!("music: {track}");
                    self.music = Some(h);
                }
                Err(e) => log::warn!("music {track}: {e}"),
            },
            Err(e) => log::warn!("music {track}: {e}"),
        }
    }

    /// Ends the current track now and lets the next start soon (tests, `/music`).
    pub fn skip_music(&mut self) {
        if let Some(m) = &mut self.music {
            m.stop(QUICK);
        }
        self.music = None;
        self.next_music = Instant::now();
    }

    pub fn music_playing(&self) -> bool {
        self.music
            .as_ref()
            .is_some_and(|m| m.state() != kira::sound::PlaybackState::Stopped)
    }
}

/// How loud a sound of `volume` at `pos` is at `ear`: full within `near`, nothing past `reach`.
fn audible(volume: f32, pos: Option<DVec3>, near: f32, reach: f32, ear: DVec3) -> f32 {
    let Some(p) = pos else { return volume };
    let d = (p.distance(ear) as f32 - near).max(0.0);
    volume * (1.0 - d / (reach - near).max(0.5)).clamp(0.0, 1.0)
}

impl Voice {
    /// Its own track: positioned ones move with the listener's ears, held ones get a low-pass
    /// to muffle.
    fn make_track(&mut self, group: &mut TrackHandle, listener: &kira::listener::ListenerHandle) -> Result<(), String> {
        let filter = FilterBuilder::new().cutoff(self.lowpass as f64);
        self.track = match self.pos {
            Some(pos) => {
                let curve = match self.falloff {
                    0 => kira::Easing::Linear,
                    n if n > 0 => kira::Easing::InPowi(n),
                    n => kira::Easing::OutPowi(-n),
                };
                let mut b = SpatialTrackBuilder::new()
                    .distances((self.near, self.reach))
                    .attenuation_function(curve)
                    .persist_until_sounds_finish(true);
                if self.held {
                    self.filter = Some(b.add_effect(filter));
                }
                Track::Spatial(
                    group
                        .add_spatial_sub_track(listener, pos.as_vec3(), b)
                        .map_err(|e| format!("{e:?}"))?,
                )
            }
            None => {
                let mut b = TrackBuilder::new().persist_until_sounds_finish(true);
                if self.held {
                    self.filter = Some(b.add_effect(filter));
                }
                Track::Plain(group.add_sub_track(b).map_err(|e| format!("{e:?}"))?)
            }
        };
        Ok(())
    }

    fn volume_db(&self) -> Decibels {
        let def = content()
            .game
            .sounds
            .get(self.sound as usize)
            .map_or(1.0, |d| d.volume);
        db((self.volume * def) as f64)
    }

    fn play_static(&mut self, data: StaticSoundData) -> Result<Handle, String> {
        let mut data = data.volume(self.volume_db()).playback_rate(self.pitch);
        if self.looped {
            data = data.loop_region(..);
        }
        if self.fade > 0.0 {
            data = data.fade_in_tween(tween(self.fade));
        }
        match &mut self.track {
            Track::Spatial(t) => t.play(data).map(Handle::Static).map_err(|e| e.to_string()),
            Track::Plain(t) => t.play(data).map(Handle::Static).map_err(|e| e.to_string()),
            Track::None => Err("no track".into()),
        }
    }

    fn play_stream(&mut self, data: StreamingSoundData<FromFileError>) -> Result<Handle, String> {
        let mut data = data.volume(self.volume_db()).playback_rate(self.pitch);
        if self.looped {
            data = data.loop_region(..);
        }
        if self.fade > 0.0 {
            data = data.fade_in_tween(tween(self.fade));
        }
        match &mut self.track {
            Track::Spatial(t) => t.play(data).map(Handle::Stream).map_err(|e| e.to_string()),
            Track::Plain(t) => t.play(data).map(Handle::Stream).map_err(|e| e.to_string()),
            Track::None => Err("no track".into()),
        }
    }
}
