//! The knowledge gate's first ruin over the real overworld (`vitals::ruins::spawn_ruin`,
//! `base_api::land::spawn`): on every seed tried a Stone ring stands within `SPAWN_REACH` blocks of
//! the world's spawn, on ground above the sea (Planner, 11-23: "so a new player is never stuck
//! before bronze by bad luck"). The game's spawn also keeps off caves' mouths (`fair_spawn`), a
//! few blocks at most from `land::spawn`.

use base_api::land as l;
use vitals::ruins as r;

#[test]
fn a_stone_ring_stands_near_every_spawn() {
    for seed in 1..=6u64 {
        let s = l::spawn(seed);
        let f = l::fields(seed);
        let dry = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4 >= base_api::SEA;
        let ruin = r::spawn_ruin(seed, [s[0], s[2]], &dry);
        let d = ((ruin.x - s[0]) as f64).hypot((ruin.z - s[2]) as f64);
        assert!(d <= r::SPAWN_REACH, "seed {seed}: spawn {s:?}, the ring at {} {} is {d:.0} blocks off", ruin.x, ruin.z);
        let h = l::height(&f, ruin.x * 16, ruin.z * 16) >> 4;
        assert!(h >= base_api::SEA - 1, "seed {seed}: the ring at {} {} is under the sea (ground {h})", ruin.x, ruin.z);
    }
}
