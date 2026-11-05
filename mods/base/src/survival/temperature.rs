//! Body temperature (`vitals::temperature`): once a second the air where one stands
//! (`base_api::air_at`: the biome's climate, the season, the hour, height, sun, rain or snow,
//! water, caves, fires near by), wet from rain and water and drying (faster by a fire), armor's
//! and clothes' insulation (`vitals::clothing`: warmth, what of it stays wet, the wind they stop), and the core temperature moving toward what that leaves it at. Sets the weather's
//! effects, chilly to freezing and warm to heatstroke. Also `/season`, which jumps to a day of a
//! season.

use base_api::{Air, Vitals};
use vitals::clothing::{self, Worn};
use vitals::effects::{self, Effect, Group};
use vitals::shelter::WALLS;
use vitals::temperature::{self, Body};
use vitals::{calendar, stomach};
use weft_sdk::*;

use super::set_group;

/// Seconds one stays wet after the rain or the water.
const WET_SECS: f32 = 120.0;
/// Blocks around one searched for heat and comfort.
const HEAT_REACH: i32 = temperature::PLAYER_FIRE_REACH;

value! {
    /// The room a player is in (`shelter.rs`), for the HUD: its main wall class (`WALLS` index + 1, 0
    /// for none) and the degrees it is over the air outside.
    pub struct Shelter { pub wall: u32, pub gain: i32 }
}
pub static ROOM: Component<Shelter> = Component::named("base:room");
static HUD_ROOM: HudElement = HudElement::named("room");

pub fn register(r: &mut Registry) {
    r.entity_component(&ROOM, Shelter { wall: 0, gain: 0 }, component_flags::SYNC_OWNER);
    r.attach(&PLAYER, &ROOM);
    // "Sheltered · log walls, +18 °C" (over the air outside), under the vitals' bars while in a room.
    r.hud_element(&HUD_ROOM, hud_anchor::TOP_LEFT, 8, |ui| {
        // One line, wide enough not to wrap (in a narrow top-left column it broke a letter a line).
        ui.text_bind_styled(S::column().min_w(150.0).no_wrap(), |c, out| match c.component(&ROOM) {
            Some(r) if r.wall > 0 && !c.outside_survival() => {
                let wall = WALLS[(r.wall as usize - 1).min(WALLS.len() - 1)].name();
                let gain = format!("{:+}", r.gain);
                out.set("hud.base.room", &[Arg::Key(&format!("hud.base.wall.{wall}")), Arg::Text(&gain)]);
            }
            _ => out.text(""),
        });
    });
    // Jumps to a day of a season (the first without one), keeping the hour.
    r.command("season", "[season:word(spring,summer,autumn,winter)] [day:int]", |cmd| {
        let w = cmd.world();
        let Some(season) = calendar::SEASONS
            .iter()
            .position(|s| Some(s.name()) == cmd.text("season"))
        else {
            return;
        };
        let day = cmd
            .int("day")
            .unwrap_or(1)
            .clamp(1, calendar::SEASON_DAYS as i64) as u32;
        w.set_day_time(calendar::on_day(w.day_time(), season as u32 * calendar::SEASON_DAYS + day - 1));
        cmd.reply("command.base.season", &[Arg::Key(&format!("season.base.{}", calendar::SEASONS[season].name())), Arg::Int(day as i64)]);
    });
    // What one wears comes to, the air as felt and the core: for checking clothes.
    r.command("warmth", "", |cmd| {
        let w = cmd.world();
        let worn = worn(&w, cmd.player());
        let Some(v) = w.component(cmd.player(), &base_api::VITALS) else { return };
        cmd.reply(
            "command.base.warmth",
            &[
                Arg::Float(worn.cold as f64, 1),
                Arg::Float(worn.cold_wet as f64, 1),
                Arg::Float(worn.heat as f64, 1),
                Arg::Int((worn.windproof * 100.0).round() as i64),
                Arg::Float(v.outside as f64, 1),
                Arg::Float(v.core as f64, 2),
            ],
        );
    })
    .permission(permission::EVERYONE);
}

/// This second's air and core temperature. Returns the air where one stands and the kinds of
/// comfort around (`camp::comfort_of`), found in the same search for fires.
pub fn feel(w: &World, id: u64, st: &EntityState, v: &mut Vitals, s: &stomach::Stomach, fortitude: f32, fat: f32) -> (Air, u32) {
    let mut comfort = 0u32;
    let here =
        base_api::air_at(w, st.pos.into(), st.size.eye, st.in_liquid != 0, HEAT_REACH, |state| comfort |= super::camp::comfort_of(state));
    (v.date, v.calendar) = (here.day, here.calendar);
    let worn = worn(w, id);
    // A rain cape keeps the rain off; water still soaks.
    if st.in_liquid != 0 || (here.open_sky && here.raining && !worn.dry_in_rain) {
        v.wet = WET_SECS;
    } else {
        v.wet = (v.wet - 1.0 - here.fire / 3.0).max(0.0);
    }
    let body = Body {
        comfort: temperature::PERSON,
        cold_insulation: worn.cold,
        heat_insulation: worn.heat,
        wet_cold_insulation: worn.cold_wet,
        fortitude,
        fat,
        meal: s.meal_warming(),
        meal_cool: s.meal_cooling(),
        wet: v.wet > 0.0,
    };
    if v.core < 20.0 {
        v.core = temperature::CORE; // saved before the body had a temperature
    }
    // Through windproof clothes, only part of the wind chills; in a room (`shelter.rs`) the wind
    // is shut out, its walls even out the day and a fire in it warms it by its walls.
    let outdoors = temperature::through_clothes(here.still, here.celsius, worn.windproof);
    let feet = block_pos_of(st.pos).to_array();
    let room = if here.underground || st.in_liquid != 0 { None } else { super::shelter::room(w, feet) };
    let felt = room.map_or(outdoors, |r| r.air(here.mean, here.still - here.fire, here.fire));
    if let Some(r) = w.component_mut(id, &ROOM) {
        *r = match room {
            Some(room) => Shelter {
                wall: WALLS
                    .iter()
                    .position(|&x| x == room.main_wall())
                    .map_or(0, |i| i as u32 + 1),
                gain: (felt - outdoors).round() as i32,
            },
            None => Shelter { wall: 0, gain: 0 },
        };
    }
    v.core = temperature::step_core(v.core, body.target(felt), 1.0, 70.0);
    v.outside = felt;

    let mut on = Vec::new();
    on.extend(effects::from_feeling(temperature::feeling(v.core)));
    if v.wet > 0.0 {
        on.push(Effect::Wet);
    }
    let weather: Vec<Effect> = effects::ALL
        .into_iter()
        .filter(|e| e.group() == Group::Weather)
        .collect();
    set_group(v, &weather, &on);
    (here, comfort)
}

/// What one wears: armor by its material, a share a piece (head to feet), and clothes by theirs
/// (`vitals::clothing`, named `<material>_<piece>`).
fn worn(w: &World, id: u64) -> Worn {
    let mut worn = Worn::default();
    let Some(slots) = w.inventory(id).find_group(EQUIPMENT_GROUP) else { return worn };
    for i in 0..slots.len() {
        let stack = slots.get(i);
        if stack.is_empty() {
            continue;
        }
        let name = item_name(stack.item);
        if let Some(c) = clothing::cloak_of(name) {
            worn.cloak(c);
        } else if let Some((m, piece)) = clothing::piece_of(name) {
            worn.clothes(m, piece);
        } else if i < 4 {
            let material = name
                .split_once(':')
                .map_or(name, |(_, n)| n)
                .split('_')
                .next()
                .unwrap_or("");
            worn.armor(material, i as usize);
        }
    }
    worn
}
