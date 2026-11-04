//! Whether the numbers hold together (`notes/SURVIVAL.md`), as the animals' `balance.rs` does for
//! species: every food fits a new player's stomach and digests in a sensible time, no single food
//! makes one well fed but a varied meal does, the level curve is as long as intended, and the
//! temperatures feel as they should. `problems()` lists what's out of line; the tests ask for
//! none, so a new food or a changed number that throws it off is caught.

use crate::effects::{self, Effect};
use crate::food::{self, FOODS, Freshness, Meal};
use crate::levels;
use crate::stamina;
use crate::stats;
use crate::stomach::{self, Stomach};
use crate::temperature::{self, Body, Feeling, Place};

pub fn problems() -> Vec<String> {
    let mut out = Vec::new();
    let capacity = stats::PLAYER_BASE[stats::Stat::Food.index()];

    for f in FOODS {
        let p = f.per100;
        if p.water + p.protein + p.carbs + p.fat + p.fiber > 101.0 {
            out.push(format!("{}: more than 100 g in 100 g", f.name));
        }
        if f.units() > capacity {
            out.push(format!("{}: {} units, more than a new player's stomach ({capacity})", f.name, f.units()));
        }
        let minutes = f.digest_minutes();
        if !(2.0..=60.0).contains(&minutes) {
            out.push(format!("{}: digests in {minutes:.0} minutes (2 to 60 wanted)", f.name));
        }
        // One food alone, as much as fits, must not make one well fed: variety does.
        let mut s = Stomach::default();
        let i = food::find(f.name).unwrap_or(0);
        while s.eat(i, 1.0, food::HOT_FOR, capacity).is_ok() {}
        if s.well_fed(capacity) {
            out.push(format!("{}: alone makes one well fed", f.name));
        }
    }

    // A new player's arrival meal: past half of max health and stamina, not well fed.
    let arrival = crate::stomach::arrival();
    let (health, stamina) = crate::stomach::max_health_stamina(100.0, 100.0, &arrival, capacity);
    if arrival.portions.iter().filter(|p| p.food > 0).count() != crate::stomach::ARRIVAL.len() || health < 50.0 || stamina < 50.0 {
        out.push(format!("a new player arrives with max health {health:.0} and stamina {stamina:.0} (over 50 wanted)"));
    }
    if arrival.well_fed(capacity) {
        out.push("a new player arrives well fed".into());
    }

    if best_meal(capacity).is_none() {
        out.push("no meal of cooked and fresh food makes a new player well fed".into());
    }

    if levels::to_next(1) != 20 {
        out.push(format!("level 1 to 2 asks {} XP (20 wanted)", levels::to_next(1)));
    }
    let all = levels::total_to(levels::MAX_LEVEL);
    if !(150_000..=400_000).contains(&all) {
        out.push(format!("{all} XP to level {} (150 000 to 400 000 wanted)", levels::MAX_LEVEL));
    }

    // A new player, well fed, sprints ten to twenty-five seconds.
    let sprint = 110.0 / stamina::SPRINT;
    if !(10.0..=25.0).contains(&sprint) {
        out.push(format!("a new player sprints {sprint:.0} s (10 to 25 wanted)"));
    }

    let person = Body { comfort: temperature::PERSON, ..Body::default() };
    let noon = |biome_c: f32| Place {
        biome_c,
        humidity: 0.5,
        altitude: 0.0,
        hour: 13.0,
        open_sky: true,
        raining: false,
        snowing: false,
        in_water: false,
        fire: 0.0,
        season: 0.0,
        underground: false,
        wind: 0.0,
    };
    let feels = |b: &Body, p: &Place| temperature::feeling(b.target(temperature::air(p)));
    if feels(&person, &noon(18.0)) != Feeling::Comfortable {
        out.push("unclothed at 18 °C is not comfortable".into());
    }
    if feels(&person, &noon(-10.0)) != Feeling::Freezing {
        out.push("unclothed at -10 °C doesn't freeze".into());
    }
    let leather = Body { cold_insulation: temperature::set_insulation("leather").0, ..person };
    let by_fire = Place { fire: temperature::fire_heat(3.0), hour: 3.0, ..noon(0.0) };
    if matches!(feels(&leather, &by_fire), Feeling::Cold | Feeling::Freezing) {
        out.push("in leather by a fire at 0 °C at night is still cold".into());
    }
    if !matches!(feels(&person, &Place { humidity: 0.1, ..noon(38.0) }), Feeling::Hot | Feeling::Heatstroke) {
        out.push("noon in a 38 °C desert isn't hot".into());
    }
    let wet = Body { wet: true, ..leather };
    if feels(&wet, &Place { hour: 20.0, ..noon(8.0) }) == Feeling::Comfortable {
        out.push("wet in leather on an 8 °C evening feels comfortable".into());
    }

    // Clothes (`clothing`): fur against a winter night, linen in the desert sun, wool wet.
    let wearing = |m: &str| {
        let w = crate::clothing::find(m)
            .map(crate::clothing::Worn::set)
            .unwrap_or_default();
        Body { cold_insulation: w.cold, heat_insulation: w.heat, wet_cold_insulation: w.cold_wet, ..person }
    };
    let night = Place { hour: 3.0, ..noon(-5.0) };
    if matches!(feels(&wearing("fur"), &night), Feeling::Cold | Feeling::Freezing) {
        out.push("a full set of fur on a -5 °C night is still cold".into());
    }
    if feels(&person, &night) != Feeling::Freezing {
        out.push("unclothed on a -5 °C night doesn't freeze (clothes would matter little)".into());
    }
    let desert = Place { humidity: 0.1, ..noon(38.0) };
    let core = |b: &Body, p: &Place| b.target(temperature::air(p));
    if core(&wearing("linen"), &desert) >= core(&person, &desert) {
        out.push("linen in the desert sun is no cooler than bare skin".into());
    }
    if core(&wearing("fur"), &desert) <= core(&person, &desert) {
        out.push("fur in the desert sun is no hotter than bare skin".into());
    }
    let soaked = |m: &str| Body { wet: true, ..wearing(m) };
    let evening = Place { hour: 20.0, ..noon(5.0) };
    if core(&soaked("wool"), &evening) <= core(&soaked("linen"), &evening) {
        out.push("wet wool is no warmer than wet linen".into());
    }
    let still = temperature::air(&Place { wind: 0.0, ..night });
    let chilled = temperature::air(&Place { wind: 10.0, ..night });
    if temperature::through_clothes(still, chilled, crate::clothing::Worn::set(&crate::clothing::MATERIALS[0]).windproof) <= chilled {
        out.push("fur doesn't keep out the wind".into());
    }

    // Meals (`meals`): a stew of three foods alone makes one well fed, and is more than its parts.
    let stew: Vec<usize> = ["base:beef", "base:bread", "base:apple"]
        .iter()
        .filter_map(|n| food::find(n))
        .map(crate::meals::cooked)
        .collect();
    let mut s = Stomach::default();
    if stew.len() != 3 || s.eat_meal(&stew, 1.0, food::HOT_FOR, capacity).is_err() || !s.well_fed(capacity) {
        out.push("a stew of beef, bread and an apple doesn't make one well fed".into());
    }
    if crate::meals::units(&stew) > capacity {
        out.push("a stew of three foods doesn't fit a new player's stomach".into());
    }

    // Preserving (`preserve`): every product is a food, keeping at least twice as long as what it's
    // made of (meat seven times and more; an apple keeps well already); a desert summer dries food in two game days, a German winter not in five.
    for m in crate::preserve::METHODS {
        for (i, f) in FOODS.iter().enumerate() {
            let Some(p) = crate::preserve::product(m, i) else { continue };
            if food::find(p).is_none() {
                out.push(format!("{} {} makes {p}, which isn't a food", m.name(), f.name));
            } else if crate::preserve::lasts_over(p, f.name) < 2.0 {
                out.push(format!("{p} keeps under twice as long as {}", f.name));
            }
        }
    }
    let dries_in = |celsius: f32, humidity: f32, wind: f32| {
        1.0 / crate::preserve::dry_rate(celsius, humidity, wind, false).max(1e-9) / crate::DAY_SECS
    };
    if dries_in(32.0, 0.18, 2.6) > 2.0 {
        out.push("a rack in a desert summer takes over two game days".into());
    }
    if dries_in(2.0, 0.77, 3.0) < 5.0 {
        out.push("a rack in a damp winter dries in under five game days".into());
    }

    // Research round 3: a new player in arrival clothes on a spring night in the plains (day 8 of
    // spring, 03:00) is no worse than cold; four hot steaks warm as one; leather armor is hide.
    let arrival = {
        let mut w = crate::clothing::Worn::default();
        for (m, piece) in crate::clothing::ARRIVAL {
            if let Some(m) = crate::clothing::find(m) {
                w.clothes(m, piece);
            }
        }
        w
    };
    let dressed = Body { cold_insulation: arrival.cold, heat_insulation: arrival.heat, wet_cold_insulation: arrival.cold_wet, ..person };
    let spring_night = Place {
        hour: 3.0,
        humidity: 0.42,
        season: temperature::season(14.2, crate::calendar::from_midwinter(crate::calendar::START_DAY as f32)),
        ..noon(12.8)
    };
    if feels(&dressed, &spring_night) == Feeling::Freezing {
        out.push("arrival clothes on a plains night at the world's start freeze".into());
    }
    if !(8.0..=14.0).contains(&arrival.cold) {
        out.push(format!("arrival clothes give {:.1} °C (about 1 clo, 8 to 14, wanted)", arrival.cold));
    }
    let mut steaks = Stomach::default();
    if let Some(steak) = food::find("base:cooked_beef") {
        for _ in 0..4 {
            let _ = steaks.eat(steak, 1.0, 0.0, capacity);
        }
    }
    if steaks.meal_warming() > food::HOT_MEAL.0 + 1e-3 {
        out.push(format!("four hot steaks warm {} °C, more than one", steaks.meal_warming()));
    }
    let mut leather = crate::clothing::Worn::default();
    let mut hide = crate::clothing::Worn::default();
    for p in 0..4 {
        leather.armor("leather", p);
        if let Some(m) = crate::clothing::find("hide") {
            hide.clothes(m, p);
        }
    }
    if leather != hide {
        out.push("leather armor isn't as warm as hide clothes".into());
    }

    // Rooms (`shelter`): a fire in a log cabin keeps a player in arrival clothes comfortable on a
    // -20 °C night; logs keep heat better than stone, stone and plank better than hide; an earthen
    // room is cooler than a desert afternoon.
    {
        use crate::shelter::{Room, WALLS, Wall};
        let room_of = |w: Wall| {
            let mut r = Room { volume: 40, ..Room::default() };
            r.faces[WALLS.iter().position(|&x| x == w).unwrap_or(0)] = 80;
            r
        };
        let cold = Place { hour: 3.0, ..noon(-15.0) };
        let (mean, still) = (temperature::day_mean(&cold), temperature::air(&cold));
        let warm = |w: Wall| room_of(w).air(mean, still, temperature::fire_heat(4.0));
        if feels(&dressed, &Place { fire: 0.0, ..cold }) == Feeling::Comfortable {
            out.push("arrival clothes are comfortable on a -20 °C night outside: a cabin would matter little".into());
        }
        if dressed.target(warm(Wall::Log)) < temperature::CORE {
            out.push(format!(
                "a fire in a log cabin on a -20 °C night leaves a player in arrival clothes cold (air {:.0} °C)",
                warm(Wall::Log)
            ));
        }
        if !(warm(Wall::Log) > warm(Wall::Stone) && warm(Wall::Stone) > warm(Wall::Hide)) {
            out.push("logs don't keep a fire's heat better than stone, and stone than hide".into());
        }
        let desert = Place { humidity: 0.18, season: 8.1, ..noon(25.7) };
        let (dm, ds) = (temperature::day_mean(&desert), temperature::air(&desert));
        if room_of(Wall::Earth).air(dm, ds, 0.0) > ds - 5.0 {
            out.push("an earthen room isn't cooler than a desert afternoon".into());
        }
    }

    // Freezing takes a full new player in two to five minutes at normal difficulty.
    let freeze = 1.0 / Effect::Freezing.modifiers().health_loss;
    if !(120.0..=300.0).contains(&freeze) {
        out.push(format!("freezing kills in {freeze:.0} s (120 to 300 wanted)"));
    }
    if effects::combine([Effect::Rested, Effect::WellFed]).stamina_regen <= 1.0 {
        out.push("rested and well fed don't raise stamina regeneration".into());
    }
    out
}

/// The smallest meal (up to six portions of fresh, cooked or safe foods) that makes a new player
/// well fed, as food names; `None` if there is none.
pub fn best_meal(capacity: f32) -> Option<Vec<&'static str>> {
    let safe: Vec<usize> = (0..FOODS.len())
        .filter(|&i| !FOODS[i].raw && FOODS[i].poison == 0.0 && FOODS[i].meal != Meal::Plain)
        .collect();
    let mut best: Option<Vec<usize>> = None;
    let mut pick = Vec::new();
    search(&safe, 0, &mut pick, capacity, &mut best);
    best.map(|b| b.iter().map(|&i| FOODS[i].name).collect())
}

fn search(safe: &[usize], from: usize, pick: &mut Vec<usize>, capacity: f32, best: &mut Option<Vec<usize>>) {
    let mut s = Stomach::default();
    let fits = pick
        .iter()
        .all(|&i| s.eat(i, Freshness::Fresh.quality(), 0.0, capacity).is_ok());
    if !fits {
        return;
    }
    if !pick.is_empty() && s.well_fed(capacity) {
        if best.as_ref().is_none_or(|b| pick.len() < b.len()) {
            *best = Some(pick.clone());
        }
        return;
    }
    if pick.len() >= 6 {
        return;
    }
    for k in from..safe.len() {
        pick.push(safe[k]);
        search(safe, k, pick, capacity, best);
        pick.pop();
    }
}

/// Max health and stamina of a new player after `meal` (food names), for the notes and checks.
pub fn after(meal: &[&str]) -> (f32, f32) {
    let capacity = stats::PLAYER_BASE[stats::Stat::Food.index()];
    let mut s = Stomach::default();
    for name in meal {
        if let Some(i) = food::find(name) {
            let _ = s.eat(i, 1.0, food::HOT_FOR, capacity);
        }
    }
    stomach::max_health_stamina(stats::PLAYER_BASE[0], stats::PLAYER_BASE[1], &s, capacity)
}

#[cfg(test)]
mod tests {
    #[test]
    fn in_balance() {
        let problems = super::problems();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn levels_round_trip() {
        for xp in [0, 19, 20, 1000, 50_000, 10_000_000] {
            let (level, into) = crate::levels::level_of(xp);
            assert_eq!(crate::levels::total_to(level) + into, xp);
        }
        assert_eq!(crate::levels::level_of(20).0, 2);
        assert_eq!(crate::levels::after_death(crate::levels::total_to(5) + 100, 0.25), crate::levels::total_to(5) + 75);
    }

    #[test]
    fn the_calendar_turns() {
        use crate::calendar::{self, Season};
        assert_eq!(calendar::date(0).season, Season::Spring);
        assert_eq!(calendar::date(0).day_of_season, 1);
        let summer = calendar::date(14 * calendar::DAY_TICKS);
        assert_eq!((summer.season, summer.day_of_season), (Season::Summer, 1));
        let next = calendar::date(56 * calendar::DAY_TICKS + 3000);
        assert_eq!((next.year, next.season), (1, Season::Spring));
        // Midsummer is the warmest, midwinter the coldest.
        let at = |day: f32| calendar::date((day * calendar::DAY_TICKS as f32) as u64).from_midwinter();
        assert!((at(calendar::MIDSUMMER) - 0.5).abs() < 0.01);
        assert!(crate::temperature::season(10.0, at(calendar::MIDSUMMER)) > 9.9);
        assert_eq!(calendar::date(calendar::on_day(3 * calendar::DAY_TICKS + 500, 42)).season, Season::Winter);
    }

    #[test]
    fn a_stomach_digests_and_fades() {
        let apple = crate::food::find("apple").unwrap();
        let mut s = crate::stomach::Stomach::default();
        s.eat(apple, 1.0, 0.0, 100.0).unwrap();
        let before = s.fullness(100.0)[crate::food::WATER];
        s.digest(60.0, 1.0);
        assert!(s.fullness(100.0)[crate::food::WATER] < before);
        s.digest(3600.0, 1.0);
        assert_eq!(s.used(), 0.0);
    }
}
