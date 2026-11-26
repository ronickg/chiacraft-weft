//! Bell: a sandboxed mod for API 1.2, in the shape of base's machines (callbacks on a block of its
//! own, `mods/base/src/machines.rs`). Using the bell rings it: a chime, a line to the players
//! near it, and a puff of particles a second later (a scheduled tick). A neighbour changing is
//! logged. The striker, an item of the package's, wears down one use at a time. Its client half
//! (`examples/wasm/bell-client`) shows the count on the HUD from `bell:rings`, and asks for it on
//! `bell:ping`.

use weft_sdk::*;

declare_mod!(deps: [], init: init);

static BELL: Block = Block::named("bell:bell");
static STRIKER: Item = Item::named("bell:striker");
static CHIME: Sound = Sound::named("base:ui.advancement");
static POOF: Particle = Particle::named("core:poof");
static PING: Channel<String> = Channel::named("ping");
/// To players' client halves: how often the bell has rung.
static RINGS: Channel<u32> = Channel::named("rings");

fn ring(w: &World, pos: [i32; 3]) {
    let at = pos.map(|v| v as f64 + 0.5);
    let n = storage::get_text("rings")
        .and_then(|t| t.parse::<u64>().ok())
        .unwrap_or(0)
        + 1;
    let _ = storage::set("rings", n.to_string().as_bytes());
    w.play_sound(&CHIME, at, 1.0);
    for p in w.players() {
        if w.entity(p)
            .is_some_and(|e| (0..3).map(|i| (e.pos[i] - at[i]).powi(2)).sum::<f64>() < 16.0 * 16.0)
        {
            w.tell(p, "bell.rang", &[Arg::Int(n as i64)]);
            w.send(p, &RINGS, &(n as u32));
        }
    }
    w.schedule(pos, 20);
    info!("the bell at {pos:?} rang ({n})");
}

fn init(r: &mut Registry) {
    r.block(&BELL)
        .on_use(|u| {
            ring(&u.world(), u.pos());
            true
        })
        .on_scheduled(|e| {
            // Above the bell, so the puff isn't inside the block.
            let at = e.pos().map(|v| v as f64 + 0.5);
            e.world()
                .spawn_particles(&POOF, [at[0], at[1] + 0.8, at[2]])
                .count(12);
            info!("a puff above the bell at {:?}", e.pos());
        })
        .on_neighbor(|e| info!("beside the bell at {:?}, {:?} changed", e.pos(), e.other()));
    r.item(&STRIKER).on_use(|u| {
        let mut s = u.stack();
        s.count = s.count.saturating_sub(1);
        let left = s.count;
        u.set_stack(s);
        u.world()
            .tell(u.player(), "bell.struck", &[Arg::Int(left as i64)]);
        true
    });
    // The client half's key: answered with the count.
    r.packet(&PING, |p, text| {
        info!("player {} pinged: {text}", p.player());
        let n = storage::get_text("rings")
            .and_then(|t| t.parse::<u32>().ok())
            .unwrap_or(0);
        p.reply(&RINGS, &n);
    });
    // Sent on only: registered so `send` may use it.
    r.packet(&RINGS, |_, _: u32| {});
}
