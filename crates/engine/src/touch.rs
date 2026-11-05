//! Touch screens (Android): in play, on-screen controls (a move stick on the left, looking by
//! dragging anywhere else, buttons on the right, the hotbar by tapping a slot); in menus the
//! first finger is the mouse (left button) and a second one, dragged, scrolls.
//!
//! Everything is pressed through the actions' own bindings (`Input::binding_of`), so the game
//! sees the same keys a keyboard would press, and rebinding keys doesn't break the controls.

use glam::Vec2;
use winit::event::TouchPhase;
use winit::keyboard::KeyCode;

use crate::input::{Input, Key};
use crate::ui::{Kind, Style, Ui};

/// On for Android; elsewhere with `WEFT_TOUCH=1` (a touch screen laptop, or to look at the
/// controls).
pub fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| cfg!(target_os = "android") || std::env::var_os("WEFT_TOUCH").is_some())
}

/// The move stick's reach, GUI pixels.
const STICK: f32 = 44.0;
/// How far from the stick's middle (of its reach) a direction counts.
const DEAD: f32 = 0.3;
/// Beyond its reach forward: sprint.
const SPRINT: f32 = 1.25;
/// Look: mouse pixels per physical pixel dragged.
const LOOK: f32 = 0.9;
const MARGIN: f32 = 12.0;
const BIG: f32 = 40.0;
const SMALL: f32 = 26.0;
const GAP: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Jump,
    Sneak,
    Attack,
    Use,
    Inventory,
    Drop,
    Chat,
    Pause,
}

const BUTTONS: [Button; 8] =
    [Button::Jump, Button::Sneak, Button::Attack, Button::Use, Button::Inventory, Button::Drop, Button::Chat, Button::Pause];

impl Button {
    /// The action it presses (`None`: Escape).
    fn action(self) -> Option<&'static str> {
        Some(match self {
            Button::Jump => "core:jump",
            Button::Sneak => "core:sneak",
            Button::Attack => "core:attack",
            Button::Use => "core:use",
            Button::Inventory => "core:inventory",
            Button::Drop => "core:drop",
            Button::Chat => "core:chat",
            Button::Pause => return None,
        })
    }

    /// Its label's translation key.
    fn label(self) -> &'static str {
        match self {
            Button::Jump => "touch.jump",
            Button::Sneak => "touch.sneak",
            Button::Attack => "touch.attack",
            Button::Use => "touch.use",
            Button::Inventory => "touch.inventory",
            Button::Drop => "touch.drop",
            Button::Chat => "touch.chat",
            Button::Pause => "touch.pause",
        }
    }

    /// Where it sits, GUI pixels (x, y, size) on a screen of `size` GUI pixels: the hand buttons
    /// in a square at the bottom right under the right thumb, the rest small in a row at the top
    /// middle (the HUD's clock and bars hold the corners).
    fn rect(self, size: Vec2) -> (f32, f32, f32) {
        let right = |n: f32, s: f32| size.x - MARGIN - n * (BIG + GAP) - s;
        let bottom = |n: f32, s: f32| size.y - MARGIN - n * (BIG + GAP) - s;
        let top = |n: f32| size.x / 2.0 - (4.0 * SMALL + 3.0 * GAP) / 2.0 + n * (SMALL + GAP);
        match self {
            Button::Jump => (right(0.0, BIG), bottom(0.0, BIG), BIG),
            Button::Sneak => (right(1.0, BIG), bottom(0.0, BIG), BIG),
            Button::Attack => (right(0.0, BIG), bottom(1.0, BIG), BIG),
            Button::Use => (right(1.0, BIG), bottom(1.0, BIG), BIG),
            Button::Pause => (top(0.0), 4.0, SMALL),
            Button::Inventory => (top(1.0), 4.0, SMALL),
            Button::Chat => (top(2.0), 4.0, SMALL),
            Button::Drop => (top(3.0), 4.0, SMALL),
        }
    }

    fn at(size: Vec2, p: Vec2) -> Option<Button> {
        BUTTONS.into_iter().find(|b| {
            let (x, y, s) = b.rect(size);
            // A little more than drawn: thumbs are wide.
            let m = 4.0;
            p.x >= x - m && p.x <= x + s + m && p.y >= y - m && p.y <= y + s + m
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    /// The mouse, in a menu.
    Pointer,
    /// A second finger in a menu: scrolls.
    Scroll,
    Look,
    Stick {
        origin: Vec2,
    },
    Button(Button),
    /// A hotbar slot tapped (0-based).
    Slot(usize),
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    id: u64,
    role: Role,
    /// Where it is now, physical pixels.
    at: Vec2,
    /// What it holds down.
    keys: [Option<Key>; 3],
}

#[derive(Default)]
pub struct Touch {
    fingers: Vec<Finger>,
    /// Scroll not yet a whole step, GUI pixels.
    scroll_rest: f32,
}

/// What a touch needs from the game around it.
pub struct Cx<'a> {
    pub input: &'a mut Input,
    /// In a world with no screen open: the on-screen controls.
    pub play: bool,
    /// Physical pixels per GUI pixel.
    pub scale: f32,
    /// The window, physical pixels.
    pub size: Vec2,
    /// The hotbar slot under a point (GUI pixels).
    pub slot_at: &'a dyn Fn(Vec2) -> Option<usize>,
}

fn key_of(input: &Input, action: &str) -> Option<Key> {
    input.binding_of(action).map(|b| b.chord.key)
}

impl Touch {
    /// A finger came down, moved or left. Returns true when it starts play's controls (the
    /// game should take the pointer, as a click into the world does).
    pub fn event(&mut self, phase: TouchPhase, id: u64, at: Vec2, cx: &mut Cx) -> bool {
        let gui = at / cx.scale;
        let size = cx.size / cx.scale;
        match phase {
            TouchPhase::Started => {
                let role = if !cx.play {
                    if self.fingers.iter().any(|f| f.role == Role::Pointer) { Role::Scroll } else { Role::Pointer }
                } else if let Some(b) = Button::at(size, gui) {
                    Role::Button(b)
                } else if let Some(i) = (cx.slot_at)(gui) {
                    Role::Slot(i)
                } else if gui.x < size.x * 0.4
                    && gui.y > size.y * 0.3
                    && !self
                        .fingers
                        .iter()
                        .any(|f| matches!(f.role, Role::Stick { .. }))
                {
                    Role::Stick { origin: gui }
                } else {
                    Role::Look
                };
                let mut f = Finger { id, role, at, keys: [None; 3] };
                match role {
                    Role::Pointer => {
                        cx.input.cursor = at;
                        f.keys[0] = Some(Key::Mouse(0));
                    }
                    Role::Button(b) => {
                        f.keys[0] = match b.action() {
                            Some(a) => key_of(cx.input, a),
                            None => Some(Key::Code(KeyCode::Escape)),
                        }
                    }
                    Role::Slot(i) => f.keys[0] = key_of(cx.input, &format!("core:hotbar{}", i + 1)),
                    _ => {}
                }
                for k in f.keys.iter().flatten() {
                    cx.input.key_down(*k);
                }
                self.fingers.retain(|f| f.id != id);
                self.fingers.push(f);
                cx.play
            }
            TouchPhase::Moved => {
                let Some(f) = self.fingers.iter_mut().find(|f| f.id == id) else { return false };
                let delta = at - f.at;
                f.at = at;
                match f.role {
                    Role::Pointer => cx.input.cursor = at,
                    Role::Scroll => {
                        self.scroll_rest += delta.y / cx.scale;
                        // A wheel step for every 12 GUI pixels dragged, the list following the finger.
                        let steps = (self.scroll_rest / 12.0).trunc();
                        self.scroll_rest -= steps * 12.0;
                        cx.input.scroll += steps;
                    }
                    Role::Look => cx.input.mouse_delta += delta * LOOK,
                    Role::Stick { origin } => {
                        let v = (gui - origin) / STICK;
                        let want = stick_keys(cx.input, v);
                        for k in f
                            .keys
                            .iter()
                            .flatten()
                            .filter(|k| !want.contains(&Some(**k)))
                        {
                            cx.input.key_up(*k);
                        }
                        for k in want
                            .iter()
                            .flatten()
                            .filter(|k| !f.keys.contains(&Some(**k)))
                        {
                            cx.input.key_down(*k);
                        }
                        f.keys = want;
                    }
                    Role::Button(_) | Role::Slot(_) => {}
                }
                false
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if let Some(i) = self.fingers.iter().position(|f| f.id == id) {
                    let f = self.fingers.remove(i);
                    if f.role == Role::Pointer {
                        cx.input.cursor = at;
                    }
                    for k in f.keys.iter().flatten() {
                        cx.input.key_up(*k);
                    }
                }
                false
            }
        }
    }

    /// Lets go of every finger (the app went to the background).
    pub fn release_all(&mut self, input: &mut Input) {
        for f in self.fingers.drain(..) {
            for k in f.keys.iter().flatten() {
                input.key_up(*k);
            }
        }
    }

    /// The controls, drawn over play: GUI pixels of a screen `size` large.
    /// `scale`: physical pixels per GUI pixel.
    pub fn draw(&self, ui: &mut Ui, size: Vec2, scale: f32, label: &dyn Fn(&str) -> Vec<crate::ui::text::Span>) {
        let pressed = |b: Button| self.fingers.iter().any(|f| f.role == Role::Button(b));
        let square = |ui: &mut Ui, x: f32, y: f32, s: f32, fill: u32| {
            let mut st = Style::default().at(x, y, [0.0, 0.0]).size(s, s);
            st.fill = Some(fill);
            ui.add(Kind::Container { class: None }, st)
        };
        // The stick: its reach where the finger came down, or where it rests.
        let stick = self.fingers.iter().find_map(|f| match f.role {
            Role::Stick { origin } => Some((origin, f.at)),
            _ => None,
        });
        let rest = Vec2::new(MARGIN + STICK + 8.0, size.y - MARGIN - STICK - 8.0);
        let (origin, knob) = match stick {
            Some((o, at)) => (o, o + (at / scale - o).clamp_length_max(STICK)),
            None => (rest, rest),
        };
        square(ui, origin.x - STICK, origin.y - STICK, 2.0 * STICK, 0x00000040);
        let k = STICK * 0.7;
        square(ui, knob.x - k / 2.0, knob.y - k / 2.0, k, if stick.is_some() { 0xFFFFFF90 } else { 0xFFFFFF50 });
        for b in BUTTONS {
            let (x, y, s) = b.rect(size);
            let mut st = Style::default().at(x, y, [0.0, 0.0]).size(s, s);
            st.fill = Some(if pressed(b) { 0xFFFFFF80 } else { 0x00000060 });
            st.align = crate::ui::Align::Center;
            st.justify = crate::ui::Justify::Center;
            ui.open(Kind::Container { class: None }, st);
            let t = ui.text(label(b.label()), "label");
            if let Kind::Text(text) = &mut ui.nodes[t].kind {
                text.center = true;
                text.wrap = false;
            }
            ui.close();
        }
    }
}

/// The movement keys for the stick pushed by `v` (its reach is 1).
fn stick_keys(input: &Input, v: Vec2) -> [Option<Key>; 3] {
    let mut keys = [None; 3];
    if v.length() < DEAD {
        return keys;
    }
    let dir = v.normalize();
    // Eight ways: a direction counts within 67.5° of it.
    let lean = 0.38;
    keys[0] = if dir.y < -lean {
        key_of(input, "core:forward")
    } else if dir.y > lean {
        key_of(input, "core:back")
    } else {
        None
    };
    keys[1] = if dir.x < -lean {
        key_of(input, "core:left")
    } else if dir.x > lean {
        key_of(input, "core:right")
    } else {
        None
    };
    if dir.y < -0.9 && v.length() > SPRINT {
        keys[2] = key_of(input, "core:sprint");
    }
    keys
}
