//! Every creature's box against its model (`animals_core::bodies`): its height near the model's
//! top, its width near the geometric mean of the model's width and length, and the model centred
//! on it, so what you see is what a sword, an arrow or a click hits. The model is measured at
//! rest (its bones' resting rotations, as the engine draws it with no animation: x and z
//! negated, Z·Y·X), from `assets/models/<model>.geo.json`, 16 model pixels a block.

use animals_core::bodies::{BOXES, LOOSE, LOW};
use serde_json::Value;

type M = [[f64; 4]; 4];

fn mul(a: &M, b: &M) -> M {
    let mut m = [[0.0; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            m[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn translate(v: [f64; 3]) -> M {
    [[1.0, 0.0, 0.0, v[0]], [0.0, 1.0, 0.0, v[1]], [0.0, 0.0, 1.0, v[2]], [0.0, 0.0, 0.0, 1.0]]
}

fn rotate(deg: [f64; 3]) -> M {
    let [x, y, z] = [-deg[0].to_radians(), deg[1].to_radians(), -deg[2].to_radians()];
    let (cx, sx, cy, sy, cz, sz) = (x.cos(), x.sin(), y.cos(), y.sin(), z.cos(), z.sin());
    let rx = [[1.0, 0.0, 0.0, 0.0], [0.0, cx, -sx, 0.0], [0.0, sx, cx, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let ry = [[cy, 0.0, sy, 0.0], [0.0, 1.0, 0.0, 0.0], [-sy, 0.0, cy, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let rz = [[cz, -sz, 0.0, 0.0], [sz, cz, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    mul(&rz, &mul(&ry, &rx))
}

fn v3(v: Option<&Value>) -> [f64; 3] {
    let n = |i: usize| {
        v.and_then(|v| v.get(i))
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    };
    [n(0), n(1), n(2)]
}

/// The model's bounds at rest, in blocks: lowest and highest corner.
fn bounds(model: &str) -> ([f64; 3], [f64; 3]) {
    let path = format!("{}/../assets/models/{model}.geo.json", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let geo: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    let bones = geo["minecraft:geometry"][0]["bones"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let names: Vec<&str> = bones
        .iter()
        .map(|b| b["name"].as_str().unwrap_or(""))
        .collect();
    let mut world: Vec<M> = Vec::new();
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for b in &bones {
        let pivot = v3(b.get("pivot"));
        let local = mul(&translate(pivot), &mul(&rotate(v3(b.get("rotation"))), &translate(pivot.map(|v| -v))));
        let parent = b["parent"]
            .as_str()
            .and_then(|p| names.iter().position(|n| *n == p))
            .filter(|&i| i < world.len());
        let m = match parent {
            Some(i) => mul(&world[i], &local),
            None => local,
        };
        let mut corners: Vec<[f64; 3]> = Vec::new();
        let bone_inflate = b["inflate"].as_f64().unwrap_or(0.0);
        for c in b["cubes"].as_array().into_iter().flatten() {
            let (o, s) = (v3(c.get("origin")), v3(c.get("size")));
            let f = c["inflate"].as_f64().unwrap_or(bone_inflate);
            for k in 0..8 {
                corners.push(std::array::from_fn(|i| if k >> i & 1 == 1 { o[i] + s[i] + f } else { o[i] - f }));
            }
        }
        for p in b["poly_mesh"]["positions"].as_array().into_iter().flatten() {
            corners.push(v3(Some(p)));
        }
        for c in corners {
            for i in 0..3 {
                let q = m[i][0] * c[0] + m[i][1] * c[1] + m[i][2] * c[2] + m[i][3];
                lo[i] = lo[i].min(q / 16.0);
                hi[i] = hi[i].max(q / 16.0);
            }
        }
        world.push(m);
    }
    (lo, hi)
}

#[test]
fn every_box_fits_its_model() {
    let mut problems = Vec::new();
    for &(model, (w, h)) in BOXES {
        let (lo, hi) = bounds(model);
        let (x, z, top) = (hi[0] - lo[0], hi[2] - lo[2], hi[1]);
        let mean = (x * z).sqrt();
        let (w, h) = (w as f64, h as f64);
        let off = ((hi[0] + lo[0]) / 2.0)
            .abs()
            .max(((hi[2] + lo[2]) / 2.0).abs());
        let low = LOW.iter().any(|(m, _)| *m == model);
        if !(0.75 * top..=(1.35 * top).max(top + 0.1)).contains(&h) && !(low && h <= top) {
            problems.push(format!("{model}: box {h} high, its model's top {top:.2}"));
        }
        let loose = LOOSE.iter().any(|(m, _)| *m == model);
        if !loose && !(0.75 * mean..=(1.5 * mean).max(mean + 0.1)).contains(&w) {
            problems.push(format!("{model}: box {w} wide, its model {x:.2} by {z:.2} (mean {mean:.2})"));
        }
        if off > 0.3 * w {
            problems.push(format!("{model}: its model's middle is {off:.2} off the box's, over 0.3 of its width {w}"));
        }
    }
    // Every creature model has a box (the flying looks are drawn on the bird's own box).
    let dir = format!("{}/../assets/models", env!("CARGO_MANIFEST_DIR"));
    for e in std::fs::read_dir(&dir).expect("the models").flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let Some(model) = name.strip_suffix(".geo.json") else { continue };
        if !model.ends_with("_fly") && !BOXES.iter().any(|(m, _)| *m == model) {
            problems.push(format!("{model}: a model with no box in bodies::BOXES"));
        }
    }
    assert!(problems.is_empty(), "boxes that don't fit their models:\n{}", problems.join("\n"));
}
