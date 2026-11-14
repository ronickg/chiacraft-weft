//! Hold-to-zoom camera control.

use std::sync::Mutex;

use weft_sdk::*;

static ZOOM_KEY: Key = Key::named("zoom");
static ZOOM_IN: Key = Key::named("zoom_in");
static ZOOM_OUT: Key = Key::named("zoom_out");

const DEFAULT_ZOOM: f32 = 5.0;
const MAX_ZOOM: f32 = 50.0;
const ZOOM_PER_NOTCH: f32 = 0.1;
const ZOOM_RESPONSE: f32 = 14.0;

pub fn register(r: &mut Registry) {
    use abi::comp::{camera::*, key::DEBUG};
    r.key(&ZOOM_KEY, "Z").tag(DEBUG);
    r.key(&ZOOM_IN, "wheel_up").while_held(&ZOOM_KEY).tag(DEBUG);
    r.key(&ZOOM_OUT, "wheel_down")
        .while_held(&ZOOM_KEY)
        .tag(DEBUG);

    let zoom = Mutex::new(DEFAULT_ZOOM);
    r.edit(&cameras::FIRST_PERSON)
        .shot(Shot::new(&[HAND, HUD, EFFECTS], move |a| {
            let keys = a.keys();
            let held = keys.held(&ZOOM_KEY);
            let delta = keys.state(&ZOOM_IN).amount - keys.state(&ZOOM_OUT).amount;
            let mut zoom = zoom.lock().unwrap();
            if a.first != 0 || !held {
                *zoom = DEFAULT_ZOOM;
            }
            if held {
                *zoom = (*zoom * (delta * ZOOM_PER_NOTCH).exp()).clamp(1.0, MAX_ZOOM);
            }
            let target = if held { 1.0 / *zoom } else { 1.0 };
            let blend = 1.0 - (-ZOOM_RESPONSE * a.dt.max(0.0)).exp();
            let fov = a.out.fov + (target - a.out.fov) * blend;
            a.out = abi::CameraPose { pos: a.eye, yaw: a.eye_yaw, pitch: a.eye_pitch, fov, ..abi::CameraPose::DEFAULT };
        }));
}
