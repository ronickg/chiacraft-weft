//! The ley network (`vitals::ley`) over the real overworld (`base_api::land`): every biome holds
//! nodes, the nodes and waystones are spaced inside their band, and no ground is far from a line.
//! The bands are from `examples/ley_map.rs` on seeds 1-3 within 6000 blocks (11-05): nodes 177-672
//! blocks from the nearest other, waystones up to 2305 from the next, ground at most 549 from a
//! line; every biome 1 node or more (the sequoia grove the fewest, 1-4).

use base_api::land as l;
use vitals::ley;

const SEEDS: [u64; 3] = [1, 2, 3];
const RADIUS: i32 = 6000;

fn nearest_other(all: &[ley::Node], n: &ley::Node) -> f64 {
    all.iter()
        .filter(|o| o.cell != n.cell)
        .map(|o| ((o.x - n.x) as f64).hypot((o.z - n.z) as f64))
        .fold(f64::MAX, f64::min)
}

#[test]
fn every_biome_has_nodes() {
    for seed in SEEDS {
        let f = l::fields(seed);
        let nodes = ley::nodes_near(seed, 0, 0, RADIUS);
        let biomes: Vec<&str> = nodes
            .iter()
            .map(|n| l::overworld_biome(&f, n.x * 16, n.z * 16))
            .collect();
        // Every land biome there in some amount (a fifth of a percent of the land or more).
        for (biome, share, _) in l::biome_shares(seed, RADIUS, 48) {
            if share >= 0.002 {
                let k = biomes.iter().filter(|b| **b == biome).count();
                assert!(k >= 1, "seed {seed}: {biome} ({:.1}% of the land) has no ley node", share * 100.0);
            }
        }
        assert!(biomes.contains(&"ocean"), "seed {seed}: no node at sea");
        // And the strong ones, the waystones, on land and at sea alike.
        let stones = nodes.iter().filter(|n| n.waystone()).count();
        assert!(
            stones * 100 >= nodes.len() * 10 && stones * 100 <= nodes.len() * 30,
            "seed {seed}: {stones} waystones of {} nodes",
            nodes.len()
        );
    }
}

#[test]
fn nodes_and_waystones_are_spaced_in_band() {
    for seed in SEEDS {
        let nodes = ley::nodes_near(seed, 0, 0, RADIUS);
        // Inside the area, so each has its neighbours round it.
        let inner: Vec<&ley::Node> = nodes
            .iter()
            .filter(|n| n.x.abs() < RADIUS - 1000 && n.z.abs() < RADIUS - 1000)
            .collect();
        for n in &inner {
            let d = nearest_other(&nodes, n);
            assert!((2.0 * ley::INSET as f64..=800.0).contains(&d), "seed {seed}: node {n:?} has its nearest at {d:.0}");
        }
        let stones: Vec<ley::Node> = nodes.iter().copied().filter(ley::Node::waystone).collect();
        for n in stones
            .iter()
            .filter(|n| n.x.abs() < RADIUS - 3000 && n.z.abs() < RADIUS - 3000)
        {
            let d = nearest_other(&stones, n);
            assert!(d <= 3000.0, "seed {seed}: waystone {n:?} has the next at {d:.0}");
        }
    }
}

#[test]
fn no_ground_is_far_from_a_line() {
    let seed = 1;
    let r = 4000;
    let lines = ley::lines_near(seed, 0, 0, r + ley::CELL);
    let mut far = 0f64;
    for z in (-r..=r).step_by(32) {
        for x in (-r..=r).step_by(32) {
            let d = lines
                .iter()
                .map(|li| li.nearest(x as f64, z as f64).1)
                .fold(f64::MAX, f64::min);
            far = far.max(d);
        }
    }
    assert!(far <= 700.0, "ground {far:.0} blocks from the nearest line");
}
