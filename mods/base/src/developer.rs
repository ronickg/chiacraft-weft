//! The Developer screen's switch and the look's tuning knobs (Ronald, 11-24: "all this tuning...
//! that's more for developer things. The actual tuning of the shader should not be something the
//! normal user can do"). Players set presets and each feature's quality on the Video screen; the
//! fitted numbers (`look.rs`: exposure, fill, glow, curve, saturation and the rest) are moved here,
//! on the Video screens' Developer tab, which shows only while `base:developer_menu` is on (off by
//! default: in `config/base.txt`, or `/developer_menu on`). Their settings are on a page of their
//! own (`PAGE`), so Mod Settings leaves them out.

use weft_sdk::*;

/// Whether the Video screens show the Developer tab.
pub static MENU: Config<bool> = Config::named("developer_menu");

/// The settings page a setting is on (the engine's `config.page`): the look's knobs are on
/// `base:developer`, which no settings screen lists, so only this screen shows them.
pub static PAGE: Component<Text, kind::Config> = Component::named("config.page");

/// Puts a setting on the Developer page (for `look.rs`'s knobs: `developer::page(r.config(..))`).
pub fn page(b: ConfigBuilder<'_>) -> ConfigBuilder<'_> {
    b.set_text(&PAGE, "base:developer")
}

/// The knobs, their settings' names and ranges as `look.rs` registers them (change both together).
/// Not the ones a feature's row sets (leaf holes, mist banks, shadow reach: `video.rs`' parts).
pub static KNOBS: [(&Config<f32>, &str, f32, f32); 54] = {
    use crate::look::*;
    [
        (&LOOK_EXPOSURE, "look_exposure", 0.5, 2.0),
        (&LOOK_SUN, "look_sun", 0.5, 2.0),
        (&LOOK_SUN_WARMTH, "look_sun_warmth", -1.0, 1.0),
        (&LOOK_FILL, "look_fill", 0.5, 2.0),
        (&LOOK_FILL_COOL, "look_fill_cool", -1.0, 1.0),
        (&LOOK_CONTRAST, "look_contrast", 0.8, 1.3),
        (&LOOK_SATURATION, "look_saturation", 0.7, 1.3),
        (&LOOK_SHADE, "look_shade", 0.5, 2.0),
        (&LOOK_HAZE, "look_haze", 0.0, 3.0),
        (&LOOK_FOG, "look_fog", 0.0, 2.0),
        (&NIGHT_LIGHT, "night_light", 0.3, 2.0),
        (&NIGHT_MOON, "night_moon", 0.3, 2.0),
        (&LOOK_CANOPY, "look_canopy", 0.2, 2.0),
        (&LOOK_SHAFTS, "look_shafts", 0.2, 3.0),
        (&LOOK_CURVE, "look_curve", 0.0, 2.0),
        (&LOOK_BLOOM, "look_bloom", 0.0, 3.0),
        (&LOOK_AUTO_EXPOSURE, "look_auto_exposure", 0.0, 2.0),
        (&LOOK_SKY_LIGHT, "look_sky_light", 0.2, 1.5),
        (&LOOK_FOLIAGE_TEST, "look_foliage_test", 0.0, 1.0),
        (&LOOK_WHITE, "look_white", -1.0, 2.5),
        (&LOOK_AO_TEST, "look_ao_test", 0.0, 1.0),
        (&LOOK_SHARPEN, "look_sharpen", 0.0, 1.0),
        (&LOOK_LEAF_SUN, "look_leaf_sun", 0.5, 4.0),
        (&LOOK_DEBUG, "look_debug", 0.0, 5.0),
        (&LOOK_SKY_HUE, "look_sky_hue", 0.0, 1.0),
        (&LOOK_EXPOSURE_ABS, "look_exposure_abs", 0.0, 1.0),
        (&LOOK_SPLIT_WARM, "look_split_warm", 0.0, 1.0),
        (&LOOK_EVENING_MIST, "look_evening_mist", 0.0, 3.0),
        (&LOOK_BLOOM_ADD, "look_bloom_add", 0.0, 1.0),
        (&LOOK_CIRRUS_OFF, "look_cirrus_off", 0.0, 1.0),
        (&LOOK_GLARE, "look_glare", 0.0, 2.0),
        (&LOOK_AO_POWER, "look_ao_power", 0.0, 8.0),
        (&LOOK_WATER_ABSORB, "look_water_absorb", 0.0, 4.0),
        (&LOOK_WATER_FOAM, "look_water_foam", 0.0, 1.0),
        (&LOOK_WATER_MIRROR, "look_water_mirror", 0.0, 1.0),
        (&LOOK_TWILIGHT, "look_twilight", 0.0, 8.0),
        (&LOOK_WATER_BODY, "look_water_body", 0.0, 2.0),
        (&LOOK_LOW_AIR, "look_low_air", 0.0, 2.0),
        (&LOOK_HAZE_SUN, "look_haze_sun", 0.0, 1.0),
        (&LOOK_SUN_GLOW, "look_sun_glow", 0.0, 4.0),
        (&LOOK_EVENING_AWAY, "look_evening_away", 0.0, 1.0),
        (&LOOK_MIST_TINT, "look_mist_tint", 0.0, 1.0),
        (&LOOK_LOW_SUN_KEEP, "look_low_sun_keep", 0.0, 1.0),
        (&LOOK_EVENING_SHADE, "look_evening_shade", 0.0, 1.0),
        (&LOOK_AIR_FADE, "look_air_fade", 0.0, 1.0),
        (&LOOK_GLOW_FILL, "look_glow_fill", 0.0, 1.0),
        (&LOOK_MIST_DAWN_TINT, "look_mist_dawn_tint", 0.0, 1.0),
        (&LOOK_HORIZON_LIFT, "look_horizon_lift", 0.0, 1.0),
        (&LOOK_SUN_DISC, "look_sun_disc", 0.0, 1.0),
        (&LOOK_DAWN_SKY, "look_dawn_sky", 0.0, 1.0),
        (&LOOK_BANK_DENSE, "look_bank_dense", 0.0, 5.0),
        (&LOOK_BANK_SUN, "look_bank_sun", 0.0, 5.0),
        (&LOOK_BANK_SHADE, "look_bank_shade", 0.0, 10.0),
        (&LOOK_MIST_CLOCK, "look_mist_clock", 0.0, 3.0),
    ]
};

/// Each knob on a line: its setting's name, a slider over its range, its value.
pub fn knob_rows(ui: &mut Ui) {
    ui.text_styled(S::default().class("dim").max_w(320.0), "video.base.developer.knobs", &[]);
    for &(knob, name, min, max) in KNOBS.iter() {
        ui.row(S::row().gap(3.0).center(), move |ui| {
            ui.text_styled(S::default().w(110.0).no_wrap(), "video.base.developer.knob", &[Arg::Text(name)]);
            ui.slider(
                S::default().w(140.0).h(10.0),
                move |_| ((knob.get() - min) / (max - min)).clamp(0.0, 1.0),
                move |_, v| knob.set(min + v.clamp(0.0, 1.0) * (max - min)),
            );
            ui.text_bind_styled(S::default().w(40.0).no_wrap(), move |_, t| {
                t.set("video.base.developer.value", &[Arg::Text(&format!("{:.2}", knob.get()))]);
            });
        });
    }
}

pub fn register(r: &mut Registry) {
    r.config(&MENU).bool(false);
    r.client_command("developer_menu", "<state:word(on,off)>", |cmd| {
        let on = cmd.text("state") == Some("on");
        MENU.set(on);
        cmd.reply(if on { "command.base.developer_menu.on" } else { "command.base.developer_menu.off" }, &[]);
    });
}
