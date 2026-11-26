//! Expected blocks for a testkit check: the first rock face (fall >= 6) near a point, and what
//! its column should show from its top down (`rock_set`), as `expect block` lines.
use base_api::land as l;
fn main() {
    let a: Vec<i32> = std::env::args()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    let f = l::fields(1);
    let h = |x: i32, z: i32| l::height(&f, x * 16, z * 16) >> 4;
    for r in 0..a[2] {
        for (x, z) in (-r..=r).flat_map(|i| [(a[0] + i, a[1] - r), (a[0] + i, a[1] + r), (a[0] - r, a[1] + i), (a[0] + r, a[1] + i)]) {
            let top = h(x, z);
            let fall = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .map(|&(dx, dz)| top - h(x + dx, z + dz))
                .max()
                .unwrap();
            if fall >= 6 && top > 64 {
                let set = l::rock_set(&f, x * 16, z * 16, top, false);
                println!("# face at {x} {z}: ground {top}, falls {fall}, {set:?} (sandy tops are strata too)");
                for dy in 0..4 {
                    println!("expect block {x} {} {z} base:{}", top - dy, set.block(1, [x, top - dy, z], false));
                }
                return;
            }
        }
    }
}
