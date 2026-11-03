//! The climate table against the research it comes from (`reports/ecology/biome_climates.json`):
//! each overworld biome's year mean, its seasons and its rainiest season stay those of its real
//! place, so the table can't drift from the research unnoticed. The Cretaceous's are estimates
//! from the rocks, with no months to compare.

use vitals::climate::BIOMES;

const RESEARCH: &str = include_str!("../../../../reports/ecology/biome_climates.json");

/// Months to the calendar's seasons (spring first), a southern place six months on.
fn season_of(month: usize, south: bool) -> usize {
    let m = if south { (month + 6) % 12 } else { month };
    // March, April, May are spring; December, January, February winter.
    ((m + 10) % 12) / 3
}

#[test]
fn the_climates_are_their_real_places() {
    let places: Vec<serde_json::Value> = serde_json::from_str(RESEARCH).expect("biome_climates.json");
    let mut problems = Vec::new();
    for c in BIOMES.iter().filter(|c| !c.underground) {
        let Some(p) = places.iter().find(|p| p["biome"] == c.biome) else {
            problems.push(format!("{}: not in the research (reports/ecology/biome_climates.json)", c.biome));
            continue;
        };
        if p["estimate"].as_bool() == Some(true) {
            continue;
        }
        let months = p["months"].as_array().expect("months");
        let t: Vec<f32> = months
            .iter()
            .map(|m| m["t_mean"].as_f64().unwrap_or(0.0) as f32)
            .collect();
        let mean = t.iter().sum::<f32>() / 12.0;
        let half = (t.iter().cloned().fold(f32::MIN, f32::max) - t.iter().cloned().fold(f32::MAX, f32::min)) / 2.0;
        // The mountains' mean is at sea level (the air cools with height): not compared.
        if c.biome != "base:mountains" && (mean - c.celsius).abs() > 1.0 {
            problems.push(format!("{}: year mean {} °C, {} has {mean:.1}", c.biome, c.celsius, c.place));
        }
        if (half - c.seasons).abs() > 1.0 {
            problems.push(format!("{}: seasons ±{} °C, {} has ±{half:.1}", c.biome, c.seasons, c.place));
        }
        let south = p["hemisphere"].as_str().is_some_and(|h| h.starts_with('s'));
        let mut days = [0.0f32; 4];
        for (i, m) in months.iter().enumerate() {
            days[season_of(i, south)] += m["rain_days"].as_f64().unwrap_or(0.0) as f32;
        }
        // The first of equals (max_by keeps the last, so from the end).
        let rainiest = |v: &[f32]| (0..4).rev().max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap();
        if rainiest(&days) != rainiest(&c.rain) {
            problems.push(format!(
                "{}: rainiest in season {} of the table, but {} rains most in season {}",
                c.biome,
                rainiest(&c.rain),
                c.place,
                rainiest(&days)
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}\n(vitals::climate::BIOMES against the research)\n", problems.join("\n"));
}
