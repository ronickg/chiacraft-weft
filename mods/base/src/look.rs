//! The look's knobs: chiacraft's shader pack reads these settings (`options` in its `pack.cfg`,
//! `scene.options` in the shaders, `look_options` in `look.wgsl`), so the day and the night can be
//! tuned in Mod Settings, or for one run with `--config "base:look_exposure=1.2"` (a PC job can sweep
//! a few values), without editing a shader or an asset. Each is a factor of 1 (or 0) at the look
//! as made; the order here is the order the pack reads them in (`OPTIONS`).

use weft_sdk::*;

pub static LOOK_EXPOSURE: Config<f32> = Config::named("look_exposure");
pub static LOOK_SUN: Config<f32> = Config::named("look_sun");
pub static LOOK_SUN_WARMTH: Config<f32> = Config::named("look_sun_warmth");
pub static LOOK_FILL: Config<f32> = Config::named("look_fill");
pub static LOOK_FILL_COOL: Config<f32> = Config::named("look_fill_cool");
pub static LOOK_CONTRAST: Config<f32> = Config::named("look_contrast");
pub static LOOK_SATURATION: Config<f32> = Config::named("look_saturation");
pub static LOOK_SHADE: Config<f32> = Config::named("look_shade");
pub static LOOK_HAZE: Config<f32> = Config::named("look_haze");
pub static LOOK_FOG: Config<f32> = Config::named("look_fog");
pub static NIGHT_LIGHT: Config<f32> = Config::named("night_light");
pub static NIGHT_MOON: Config<f32> = Config::named("night_moon");
pub static LOOK_CANOPY: Config<f32> = Config::named("look_canopy");
pub static LOOK_SHAFTS: Config<f32> = Config::named("look_shafts");

/// The pack's `options` line: the settings in the order `look.wgsl` reads them.
pub const OPTIONS: &str = "base:look_exposure, base:look_sun, base:look_sun_warmth, base:look_fill, base:look_fill_cool, \
base:look_contrast, base:look_saturation, base:look_shade, base:look_haze, base:look_fog, base:night_light, base:night_moon, \
base:look_canopy, base:look_shafts";

pub fn register(r: &mut Registry) {
    // By day: how bright the picture is taken, the sun's strength and warmth (-1 whiter, +1 toward
    // a 4300 K orange), the sky's fill light in the shade and how cool it is (-1 neutral, +1 bluer),
    // the curve's contrast, the colour after it, how fast the sky's light fades under cover (more:
    // darker forest floors), the haze over distance and the fog's light.
    r.config(&LOOK_EXPOSURE).float(1.0, 0.5, 2.0);
    r.config(&LOOK_SUN).float(1.0, 0.5, 2.0);
    r.config(&LOOK_SUN_WARMTH).float(0.0, -1.0, 1.0);
    r.config(&LOOK_FILL).float(1.0, 0.5, 2.0);
    r.config(&LOOK_FILL_COOL).float(0.0, -1.0, 1.0);
    r.config(&LOOK_CONTRAST).float(1.0, 0.8, 1.3);
    r.config(&LOOK_SATURATION).float(1.0, 0.7, 1.3);
    r.config(&LOOK_SHADE).float(1.0, 0.5, 2.0);
    r.config(&LOOK_HAZE).float(1.0, 0.0, 2.0);
    r.config(&LOOK_FOG).float(1.0, 0.0, 2.0);
    // By night (dark by design): the night's sky light and the moon's light.
    r.config(&NIGHT_LIGHT).float(1.0, 0.3, 2.0);
    r.config(&NIGHT_MOON).float(1.0, 0.3, 2.0);
    // Under a forest's canopy (how closed it is comes from the biome, `vitals::air::CANOPY`): how
    // dim and green the shade under the leaves is, with its sunflecks, and how strong the light
    // shafts through the crown gaps are.
    r.config(&LOOK_CANOPY).float(1.0, 0.2, 2.0);
    r.config(&LOOK_SHAFTS).float(1.0, 0.2, 3.0);
}
