//! Capybaras: the biggest rodents, calm and always chewing, in small groups on grassland. They
//! wander slowly, graze (their idle puts the head down to the grass now and then), and settle
//! down: they sit, and after half a minute flop onto their bellies with the head held up. A pose
//! is a look (`World::set_look`), as the chihuahua's. The model, its animations and texture are
//! built from `source/capybara` (see `docs/creatures.md`).

use weft_sdk::ai::Goals;
use weft_sdk::*;

pub(crate) static CAPYBARA: Entity = Entity::named("capybara");
const TEXTURE: &str = "capybara/capybara";
/// Resting this long (thirty seconds), a sitting capybara lies down.
const LIE_TICKS: u32 = 600;

value! {
    #[derive(Default)]
    /// What a capybara is doing (not saved: it just settles down again).
    pub struct Capybara {
        /// Ticks left resting (0: up and about).
        pub rest: u32,
        /// Ticks rested so far.
        pub rested: u32,
        /// The look applied, 1 + pose (0: none yet).
        pub look: u32,
    }
}

static STATE: Component<Capybara> = Component::named("capybara_state");

const LOOKS: [(&str, &str); 3] = [("", ""), ("_sit", "capybara_sit"), ("_lie", "capybara_lie")];

pub fn init(r: &mut Registry) {
    r.entity_component(&STATE, Capybara::default(), 0);
    // The poses, drawn in place of a capybara sitting or lying down.
    for (suffix, kind) in &LOOKS[1..] {
        drop(
            r.entity(Entity::generated(kind.to_string()))
                .size(0.7, 0.6)
                .passive()
                .model_file("capybara")
                .animations(&format!("capybara{suffix}"))
                .texture(TEXTURE),
        );
    }
    r.entity(&CAPYBARA)
        .size(0.7, 0.6)
        .health(12.0)
        .speed(2.5)
        .passive()
        .model_file("capybara")
        .animations("capybara")
        .texture(TEXTURE)
        .attach(&STATE)
        .on_interact(|e| {
            let (w, id, player) = (e.world(), e.entity(), e.other());
            if !crate::screen::interact(&w, id, player) && !crate::taming::feed(&w, id, player) {
                crate::commands::interact(&w, id, player);
            }
        })
        .goals(
            Goals::new()
                .then(crate::animal::upkeep(2))
                .then(crate::taming::behave(2))
                .then(crate::signatures::calm_aura)
                .then(crate::commands::act(2))
                .then(crate::breeding::behave(2))
                .then(settle)
                .then(crate::commands::hold)
                .then(crate::comfort::seek)
                .then(ai::wander(0.35, 6.0)), // slow and calm
        )
        .on_hurt(|w, h| {
            crate::taming::hurt(w, h.entity);
            if let Some(c) = w.component_mut(h.entity, &STATE) {
                c.rest = 0;
            }
        });
}

/// Every tick: now and then settle down for a while (sit, then lie), and show the pose.
fn settle(w: &World, id: u64, st: &EntityState) -> bool {
    let Some(c) = w.component_mut(id, &STATE) else { return false };
    if c.rest == 0 && !w.navigating(id) && w.random_below(300) == 0 {
        c.rest = 400 + w.random_below(1200);
        c.rested = 0;
    }
    let resting = c.rest > 0;
    if resting {
        c.rest -= 1;
        c.rested += 1;
    }
    let pose = match (resting, c.rested > LIE_TICKS) {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    };
    if c.look != pose + 1 {
        c.look = pose + 1;
        let kind = LOOKS[pose as usize].1;
        w.set_look(
            id,
            (!kind.is_empty())
                .then(|| format!("animals:{kind}"))
                .as_deref(),
        );
    }
    if resting && w.navigating(id) {
        w.navigate(id, st.pos, 0.0);
    }
    resting
}

/// Up from resting, to follow or fight.
pub fn wake(w: &World, id: u64) {
    if let Some(c) = w.component_mut(id, &STATE)
        && (c.rest > 0 || c.look > 1)
    {
        c.rest = 0;
        c.look = 1;
        w.set_look(id, None);
    }
}
