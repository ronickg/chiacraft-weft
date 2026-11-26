//! The `hostile` fuzz target's mod: `/probe <ops>` runs the hex in `ops` as a program of API 1.0
//! calls with the worst arguments it can find (NaN and infinite floats, `u64::MAX` ids, 10k
//! character strings, keys like `../x`), some of them in loops that meet every per-call cap and
//! the fuel. The fuzz target's game checks what reaches it; what the mod itself can see of a
//! broken promise (too many entities from `near`, a registration outside `init`, a storage key
//! that isn't one) it reports by teleporting entity [`REPORT`].
//!
//! It goes past the SDK's wrappers to the raw imports, so every argument of every call is its own.
use chiacraft_mod_sdk::bindings::chiacraft::mods::types::{Arg, ItemStack};
use chiacraft_mod_sdk::bindings::chiacraft::mods::{command, log, names, registry, storage, world};
use chiacraft_mod_sdk::*;

declare_mod!(init: init);

/// The entity the mod teleports to say a promise broke: `(code, detail, 0)`. The fuzz target's
/// game fails on it. The mod's own teleports never go there.
pub const REPORT: u64 = 0x0BAD_0BAD_0BAD_0BAD;

/// Broken promises the mod can see.
const NEAR_TOO_MANY: f64 = 1.0;
const REGISTERED_LATE: f64 = 2.0;
const STORED_BAD_KEY: f64 = 3.0;
const TOO_MANY_KEYS: f64 = 4.0;
const STORED_TOO_MUCH: f64 = 5.0;

fn init(r: &mut Registry) {
    r.command("probe", "<ops:text>", |cmd| {
        let program = unhex(cmd.text("ops").unwrap_or(""));
        let player = cmd.player();
        let mut ops = Ops { b: &program, at: 0 };
        while !ops.done() {
            step(&mut ops, player, 0);
        }
    })
    .permission(permission::EVERYONE);
}

fn unhex(s: &str) -> Vec<u8> {
    let digit = |c: u8| (c as char).to_digit(16).map(|d| d as u8);
    let digits: Vec<u8> = s.bytes().filter_map(digit).collect();
    digits.chunks_exact(2).map(|p| p[0] << 4 | p[1]).collect()
}

/// The program's bytes; past the end every read is 0.
struct Ops<'a> {
    b: &'a [u8],
    at: usize,
}

impl Ops<'_> {
    fn done(&self) -> bool {
        self.at >= self.b.len()
    }
    fn byte(&mut self) -> u8 {
        let v = self.b.get(self.at).copied().unwrap_or(0);
        self.at = self.at.saturating_add(1);
        v
    }
    fn take(&mut self, n: usize) -> &[u8] {
        let start = self.at.min(self.b.len());
        let end = self.at.saturating_add(n).min(self.b.len());
        self.at = self.at.saturating_add(n);
        &self.b[start..end]
    }
    fn raw32(&mut self) -> u32 {
        u32::from_le_bytes([self.byte(), self.byte(), self.byte(), self.byte()])
    }
    fn raw64(&mut self) -> u64 {
        (self.raw32() as u64) | (self.raw32() as u64) << 32
    }

    /// Values from a table of edges three times in four, raw bits otherwise.
    fn f64(&mut self) -> f64 {
        const EDGES: [f64; 18] = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            f64::NAN,
            -f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            5e-324,
            1e300,
            10.000_000_001,
            -10.000_000_001,
            64.0,
            64.000_000_001,
            -64.000_000_001,
        ];
        match self.byte() {
            s @ 0..0xC0 => EDGES[s as usize % EDGES.len()],
            _ => f64::from_bits(self.raw64()),
        }
    }
    fn f32(&mut self) -> f32 {
        const EDGES: [f32; 16] = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            f32::NAN,
            -f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MAX,
            f32::MIN,
            f32::MIN_POSITIVE,
            1e-45,
            1.0e6,
            1_000_001.0,
            1.0e7,
            -1.0e-30,
        ];
        match self.byte() {
            s @ 0..0xC0 => EDGES[s as usize % EDGES.len()],
            _ => f32::from_bits(self.raw32()),
        }
    }
    fn u64(&mut self) -> u64 {
        const EDGES: [u64; 8] = [0, 1, 7, u64::MAX, u64::MAX - 1, 1 << 63, 1000, 1001];
        match self.byte() {
            s @ 0..0xC0 => EDGES[s as usize % EDGES.len()],
            _ => self.raw64(),
        }
    }
    fn u32(&mut self) -> u32 {
        const EDGES: [u32; 8] = [0, 1, 8, 9, u32::MAX, u32::MAX - 1, 1 << 31, i32::MAX as u32];
        match self.byte() {
            s @ 0..0xC0 => EDGES[s as usize % EDGES.len()],
            _ => self.raw32(),
        }
    }
    fn i32(&mut self) -> i32 {
        const EDGES: [i32; 8] = [0, 1, -1, i32::MIN, i32::MAX, i32::MIN + 1, 30_000_000, -30_000_000];
        match self.byte() {
            s @ 0..0xC0 => EDGES[s as usize % EDGES.len()],
            _ => self.raw32() as i32,
        }
    }
    fn i64(&mut self) -> i64 {
        match self.byte() % 4 {
            0 => i64::MIN,
            1 => i64::MAX,
            2 => -1,
            _ => self.raw64() as i64,
        }
    }
    fn pos(&mut self) -> (i32, i32, i32) {
        (self.i32(), self.i32(), self.i32())
    }
    fn vec3(&mut self) -> (f64, f64, f64) {
        (self.f64(), self.f64(), self.f64())
    }
    /// An id, never [`REPORT`].
    fn id(&mut self) -> u64 {
        let id = self.u64();
        if id == REPORT { id ^ 1 } else { id }
    }
    fn stack(&mut self) -> ItemStack {
        ItemStack { item: self.u32(), count: self.u32(), data: self.u32() }
    }

    /// Strings at every edge the sandbox has: 64 (storage keys, props), 128 (keys, kinds), 256
    /// (names), 512 (log lines), 1024 (text args), clipped where no char boundary is.
    fn text(&mut self) -> String {
        match self.byte() % 20 {
            0 => String::new(),
            1 => "a".repeat(10_000),
            // Three bytes a char: 1024, 512, 128 and 64 all fall inside one.
            2 => "€".repeat(3_400),
            3 => format!("a{}", "€".repeat(3_400)),
            4 => format!("ab{}", "€".repeat(400)),
            5 => "../x".into(),
            6 => "../../../../etc/passwd".into(),
            7 => ".hidden".into(),
            8 => "a/b\\c\0d".into(),
            9 => "k".repeat(127 + self.byte() as usize % 3),
            10 => "s".repeat(63 + self.byte() as usize % 3),
            11 => "n".repeat(255 + self.byte() as usize % 3),
            12 => "t".repeat(1023 + self.byte() as usize % 3),
            13 => "l".repeat(511 + self.byte() as usize % 3),
            14 => "x".repeat(100_000),
            15 => "base:furnace[lit=true]".into(),
            16 => "probe".into(),
            _ => {
                let n = self.byte() as usize;
                String::from_utf8_lossy(self.take(n)).into_owned()
            }
        }
    }
    fn arg(&mut self) -> Arg {
        match self.byte() % 6 {
            0 => Arg::Text(self.text()),
            1 => Arg::Key(self.text()),
            2 => Arg::Int(self.i64()),
            3 => Arg::Float((self.f64(), self.u32())),
            4 => Arg::Item(self.u32()),
            _ => Arg::Name(self.text()),
        }
    }
    fn args(&mut self) -> Vec<Arg> {
        let n = match self.byte() % 8 {
            0 => 0,
            1 => 16,
            2 => 17,
            3 => 1000,
            _ => self.byte() as usize % 40,
        };
        (0..n).map(|_| self.arg()).collect()
    }
    fn level(&mut self) -> log::Level {
        match self.byte() % 4 {
            0 => log::Level::Error,
            1 => log::Level::Warn,
            2 => log::Level::Info,
            _ => log::Level::Debug,
        }
    }
    /// A loop's count: around a cap, or far past it.
    fn many(&mut self, cap: u32) -> u32 {
        match self.byte() % 4 {
            0 => cap,
            1 => cap + 1,
            2 => cap * 3,
            _ => self.byte() as u32 * cap / 64,
        }
    }
    fn bytes(&mut self) -> Vec<u8> {
        let n = match self.byte() % 8 {
            0 => 0,
            1 => 1 << 20,
            2 => (1 << 20) + 1,
            3 => 4 << 20,
            4 => (4 << 20) + 1,
            _ => self.byte() as usize * 1024,
        };
        vec![0xA5; n]
    }
}

fn report(code: f64, detail: f64) {
    world::teleport(REPORT, (code, detail, 0.0));
}

/// The sandbox's storage key rule, so the mod can tell when one that breaks it was kept.
fn key_ok(key: &str) -> bool {
    (1..=64).contains(&key.len())
        && !key.starts_with('.')
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_-.".contains(c))
}

fn set(key: &str, value: &[u8]) {
    if storage::set(key, value).is_ok() && !key_ok(key) {
        report(STORED_BAD_KEY, key.len() as f64);
    }
}

fn check_storage() {
    let keys = storage::keys();
    if keys.len() > 1024 {
        report(TOO_MANY_KEYS, keys.len() as f64);
    }
    let mut total = 0u64;
    for k in &keys {
        if !key_ok(k) {
            report(STORED_BAD_KEY, k.len() as f64);
        }
        total += storage::get(k).map_or(0, |v| v.len() as u64);
    }
    if total > 4 << 20 {
        report(STORED_TOO_MUCH, total as f64);
    }
}

fn near(ops: &mut Ops) {
    let found = world::near(ops.vec3(), ops.f64());
    if found.len() > 256 {
        report(NEAR_TOO_MANY, found.len() as f64);
    }
}

/// One op of the program, and its arguments.
fn step(ops: &mut Ops, player: u64, depth: u32) {
    let op = ops.byte() % 48;
    match op {
        0 => _ = world::time(),
        1 => _ = world::seed(),
        2 => _ = world::random(),
        3 => _ = world::dimension(),
        4 => _ = world::get_block(ops.pos()),
        5 => _ = world::set_block(ops.pos(), ops.u32()),
        // Past the 4096 changes a call.
        6 => {
            let (n, (x, y, z), state) = (ops.many(4096), ops.pos(), ops.u32());
            for i in 0..n {
                world::set_block((x.wrapping_add(i as i32), y, z), state);
            }
        }
        7 => _ = world::light(ops.pos()),
        8 => _ = world::biome(ops.pos()),
        9 => _ = world::players(),
        10 => {
            if let Some(e) = world::get_entity(ops.id()) {
                // What the game gave back, straight into the next calls.
                world::push(ops.id(), e.vel);
                world::teleport(ops.id(), e.pos);
            }
        }
        11 => near(ops),
        12 => world::teleport(ops.id(), ops.vec3()),
        13 => world::push(ops.id(), ops.vec3()),
        14 => _ = world::damage(ops.id(), ops.f32(), ops.id(), ops.u32()),
        15 => _ = world::spawn(&ops.text(), ops.vec3()),
        // Past the 64 spawns a call.
        16 => {
            let (n, kind, p) = (ops.many(64), ops.text(), ops.vec3());
            for _ in 0..n {
                world::spawn(&kind, p);
            }
        }
        17 => world::remove(ops.id()),
        18 => _ = world::creative(ops.id()),
        19 => _ = world::give(ops.id(), ops.stack()),
        20 => world::spawn_item(ops.vec3(), ops.stack()),
        21 => world::tell(ops.id(), &ops.text(), &ops.args()),
        // Past the 32 lines a call.
        22 => {
            let (n, to, key, args) = (ops.many(32), ops.id(), ops.text(), ops.args());
            for _ in 0..n {
                world::tell(to, &key, &args);
            }
        }
        23 => _ = world::player_key(ops.id()),
        24 => command::reply(&ops.text(), &ops.args()),
        25 => {
            let (n, key, args) = (ops.many(32), ops.text(), ops.args());
            for _ in 0..n {
                command::reply(&key, &args);
            }
        }
        26 => _ = names::block(ops.u32()),
        27 => _ = names::block_state(&ops.text()),
        28 => _ = names::prop(ops.u32(), &ops.text()),
        29 => _ = names::with_prop(ops.u32(), &ops.text(), &ops.text()),
        30 => _ = names::item(ops.u32()),
        31 => _ = names::item_id(&ops.text()),
        32 => _ = names::entity_kind(ops.u32()),
        33 => _ = names::entity_kind_id(&ops.text()),
        34 => _ = storage::get(&ops.text()),
        35 => set(&ops.text(), &ops.bytes()),
        36 => storage::remove(&ops.text()),
        37 => check_storage(),
        // Past the key and byte quotas.
        38 => {
            let (n, value) = (ops.many(1024), ops.bytes());
            for i in 0..n {
                set(&format!("k{i}"), &value);
            }
            check_storage();
        }
        39 => log::write(ops.level(), &ops.text()),
        // Past the 64 lines a call.
        40 => {
            let (n, level, line) = (ops.many(64), ops.level(), ops.text());
            for _ in 0..n {
                log::write(level, &line);
            }
        }
        // Registering outside `init` is refused.
        41 => {
            let late = [
                registry::command(&ops.text(), &ops.text(), ops.u32()).is_ok(),
                registry::command("late", "", 0).is_ok(),
                registry::on(registry::Event::BlockBroken) != u32::MAX,
                registry::intercept(registry::Question::Attack) != u32::MAX,
                registry::modify(registry::Chain::Damage, ops.u32()) != u32::MAX,
                registry::world_tick() != u32::MAX,
            ];
            if let Some(i) = late.iter().position(|&l| l) {
                report(REGISTERED_LATE, i as f64);
            }
        }
        // Many lookups at once: the game's answers cost it, not the mod's fuel.
        42 => {
            for _ in 0..ops.many(256) {
                near(ops);
                world::players();
                storage::keys();
            }
        }
        // The next op, many times over (nested a few deep).
        43 => {
            let n = ops.byte() as u32 * 64 + 1;
            let at = ops.at;
            if depth < 4 {
                for _ in 0..n {
                    ops.at = at;
                    step(ops, player, depth + 1);
                }
            }
        }
        // Out of fuel: a strike.
        44 => {
            if ops.byte() % 4 == 0 {
                let mut x = 1u64;
                loop {
                    x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(7));
                }
            }
        }
        // Past the memory cap: a strike.
        45 => {
            if ops.byte() % 4 == 0 {
                let big: Vec<u8> = vec![1; 128 << 20];
                std::hint::black_box(&big);
            }
        }
        // A trap: a strike.
        46 => {
            if ops.byte() % 4 == 0 {
                panic!("asked to");
            }
        }
        // To whoever ran it, and about them.
        _ => {
            world::tell(player, &ops.text(), &ops.args());
            world::damage(player, ops.f32(), player, ops.u32());
            world::push(player, ops.vec3());
        }
    }
}
