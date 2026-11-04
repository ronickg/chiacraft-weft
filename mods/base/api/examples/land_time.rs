//! What base's terrain features cost on the CPU: each mod feature's time a column, and a chunk
//! column's (281 columns: 256 and the 25 corners the engine asks for).
//!
//!     cargo run -p base_api --example land_time -- <seed>

use base_api::land as l;
use weft_sdk::noise::fixed::Fields;

fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let f = Fields { seed };
    let n = 400_000;
    let at = |i: i32| ((i % 800) * 16 * 3, (i / 800) * 16 * 3);
    let mut sum = 0i64;
    let mut time = |name: &str, g: &dyn Fn(i32, i32) -> i32| {
        let t = std::time::Instant::now();
        for i in 0..n {
            let (x, z) = at(i);
            sum += g(x, z) as i64;
        }
        let ns = t.elapsed().as_nanos() as f64 / n as f64;
        println!("  {name:8} {ns:6.0} ns a column, {:.2} ms a chunk column", ns * 281.0 / 1e6);
        ns
    };
    println!("seed {seed}, {n} columns 3 blocks apart");
    let mesa = time("mesas", &|x, z| l::mesa(&f, x, z));
    let lakes = time("lakes", &|x, z| l::lakes(&f, x, z));
    let rivers = time("rivers", &|x, z| l::rivers(&f, x, z));
    time("bank", &|x, z| l::bank(&f, x, z).map_or(0, |b| b.wet));
    time("edge", &|x, z| l::biome_edge(&f, x, z, 80).map_or(0, |e| e.near));
    println!(
        "  mods' features: {:.2} ms a chunk column, the lakes {:.0}% of it ({sum})",
        (mesa + lakes + rivers) * 281.0 / 1e6,
        lakes * 100.0 / (mesa + lakes + rivers)
    );
}
