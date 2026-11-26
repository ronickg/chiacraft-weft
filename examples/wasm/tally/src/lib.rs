//! Tally: counts the blocks each player breaks (a sandboxed mod for API 1.1, the shape of base's
//! levels in `mods/base/src/levels.rs`: an event, a per-player component, a command). Every tenth
//! block plays a chime with a puff of particles and tells the player; `/tally pillar` builds a
//! glass pillar above them in one `set_blocks`.

use weft_sdk::*;

declare_mod!(deps: [], init: init);

value! {
    /// Blocks broken, kept in the save and on respawn.
    pub struct Mined { pub blocks: u32, pub spare: u32 }
}

static MINED: Component<Mined> = Component::named("mined");
static CHIME: Sound = Sound::named("base:ui.advancement");
static POOF: Particle = Particle::named("core:poof");
static GLASS: Block = Block::named("base:glass");

fn init(r: &mut Registry) {
    r.entity_component(&MINED, Mined { blocks: 0, spare: 0 }, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &MINED);
    r.on(&BLOCK_BROKEN, |b| {
        let w = b.world();
        for e in b.iter() {
            let mut now = 0;
            w.update_component(e.player, &MINED, |m| {
                m.blocks += 1;
                now = m.blocks;
            });
            if now > 0 && now % 10 == 0 {
                let at = e.pos.map(|v| v as f64 + 0.5);
                w.play_sound(&CHIME, at, 1.0);
                w.spawn_particles(&POOF, at).count(20);
                w.tell(e.player, "tally.tenth", &[Arg::Int(now as i64)]);
                info!("player {} has broken {now} blocks", e.player);
            }
        }
    });
    r.command("tally", "[action:word(pillar)]", |cmd| {
        let (w, player) = (cmd.world(), cmd.player());
        if cmd.text("action") == Some("pillar") {
            let glass = GLASS.id();
            let Some(st) = w.entity(player).filter(|_| glass != u32::MAX) else {
                cmd.reply("command.tally.no_glass", &[]);
                return;
            };
            let [x, y, z] = st.block_pos();
            let changes: Vec<([i32; 3], u32)> = (3..8).map(|dy| ([x + 2, y + dy - 3, z], glass)).collect();
            let n = w.set_blocks(&changes);
            cmd.reply("command.tally.pillar", &[Arg::Int(n as i64)]);
            return;
        }
        let n = w.component(player, &MINED).map_or(0, |m| m.blocks);
        cmd.reply("command.tally.count", &[Arg::Int(n as i64)]);
    })
    .permission(permission::EVERYONE);
}
