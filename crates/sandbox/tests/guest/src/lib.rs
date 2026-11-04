//! What the sandbox tests run: a mod that does as it is told through `/probe <action>`, so each
//! limit can be met on purpose.
use std::cell::Cell;

use chiacraft_mod_sdk::*;

declare_mod!(init: init);

value! {
    pub struct Counter { pub n: u32, pub spare: u32, pub last: u64 }
}
static COUNTER: Component<Counter> = Component::named("counter");
static CLICK: Sound = Sound::named("base:ui.click");
static FLAME: Particle = Particle::named("base:flame");

thread_local! {
    /// Lost when the mod starts over: shows a strike dropped its memory.
    static CALLS: Cell<u32> = const { Cell::new(0) };
}

fn init(r: &mut Registry) {
    r.entity_component(&COUNTER, Counter { n: 5, spare: 0, last: 0 }, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &COUNTER);
    r.command("probe", "<action:word> [n:int]", |cmd| {
        CALLS.with(|c| c.set(c.get() + 1));
        let w = cmd.world();
        match cmd.text("action").unwrap_or("") {
            "count" => {
                let n = CALLS.with(Cell::get) as i64;
                cmd.reply("probe.count", &[Arg::Int(n)]);
            }
            "spin" => {
                let mut x: u64 = 1;
                loop {
                    x = std::hint::black_box(x.wrapping_mul(31).wrapping_add(7));
                }
            }
            "grow" => {
                let big: Vec<u8> = vec![1; 128 << 20];
                cmd.reply("probe.grew", &[Arg::Int(big.len() as i64)]);
            }
            "panic" => panic!("asked to"),
            "store" => {
                let n = cmd.int("n").unwrap_or(0);
                storage::set("n", n.to_string().as_bytes()).unwrap();
                let back = storage::get_text("n").unwrap_or_default();
                cmd.reply("probe.stored", &[Arg::Text(&back)]);
            }
            "load" => {
                let back = storage::get_text("n").unwrap_or_default();
                cmd.reply("probe.loaded", &[Arg::Text(&back)]);
            }
            "bad_key" => {
                let e = storage::set("../escape", b"x").unwrap_err();
                cmd.reply("probe.refused", &[Arg::Text(&e)]);
            }
            "flood" => {
                for _ in 0..100 {
                    w.tell(cmd.player(), "probe.flood", &[]);
                }
            }
            "world" => {
                let state = w.get([1, 2, 3]);
                cmd.reply("probe.world", &[Arg::Int(w.time() as i64), Arg::Int(state as i64), Arg::Text(&w.player_key(cmd.player()))]);
            }
            "late" => {
                // Registering outside init is refused.
                let mut r = Registry::__late();
                r.world_tick(|_| {});
                cmd.reply("probe.late", &[]);
            }
            "comp" => {
                let t = w.time();
                w.update_component(cmd.player(), &COUNTER, |c| {
                    c.n += 1;
                    c.last = t;
                });
                let c = w
                    .component(cmd.player(), &COUNTER)
                    .map_or(-1, |c| c.n as i64);
                cmd.reply("probe.comp", &[Arg::Int(c)]);
            }
            "blocks" => {
                let changes: Vec<([i32; 3], u32)> = (0..5000).map(|i| ([i, 0, 0], 1)).collect();
                let n = w.set_blocks(&changes);
                cmd.reply("probe.blocks", &[Arg::Int(n as i64)]);
            }
            "effects" => {
                for _ in 0..100 {
                    w.play_sound(&CLICK, [0.0, 64.0, 0.0], 1.0);
                    w.spawn_particles(&FLAME, [0.0, 64.0, 0.0]).count(1000);
                }
            }
            _ => cmd.reply("probe.unknown", &[]),
        }
    })
    .permission(permission::EVERYONE);
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            w.tell(e.player, "probe.broken", &[Arg::Int(e.pos[1] as i64)]);
        }
    });
    r.intercept(&question::BREAK_BLOCK, |q| if q.pos()[1] < 0 { Verdict::Deny } else { Verdict::Abstain });
    r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| m.value *= 0.5);
    r.world_tick(|w| {
        if w.time() % 20 == 0 {
            info!("tick {}", w.time());
        }
    });
}
