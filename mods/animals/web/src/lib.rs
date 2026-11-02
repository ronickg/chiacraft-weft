//! The animals mod's logic for the website (`animals_core`, the game's own code), compiled to
//! WASM (`build.sh`). A page talks to it through JSON: `call` takes a request (`{"op": ..}`) and
//! answers one; `paint` paints a coat into a texture in place. A genome travels as hex (its 84
//! bytes, as the game saves it).
//!
//! The population simulator (`sim_*`) runs the game's breeding numbers over time: matings,
//! carrying, litters and clutches, growing up, families, and (if asked) old age, which the game
//! doesn't have yet.

use std::cell::RefCell;

use animals_core::coats::{self, Genome, Pool, Rng};
use animals_core::evolution::{self, HABITATS, Habitat};
use animals_core::life;
use animals_core::species::{ADULT, BABY, SPECIES, STATS, same_species};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// ------------------------------------------------------------------ memory and calls

thread_local! {
    static OUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SIM: RefCell<Sim> = RefCell::new(Sim::default());
}

/// Room for `len` bytes the page writes into.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut v = vec![0u8; len.max(1)];
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Gives back what `alloc` lent.
///
/// # Safety
/// `ptr` and `len` as `alloc` gave and was asked.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut u8, len: usize) {
    unsafe { drop(Vec::from_raw_parts(ptr, len.max(1), len.max(1))) };
}

/// Answers the JSON request at `ptr` (`len` bytes): the answer's address; its length is
/// `out_len()`. It stays until the next call.
///
/// # Safety
/// `ptr` points at `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn call(ptr: *const u8, len: usize) -> *const u8 {
    let text = unsafe { std::slice::from_raw_parts(ptr, len) };
    let answer = match serde_json::from_slice::<Value>(text) {
        Ok(req) => handle(&req).unwrap_or_else(|e| json!({ "error": e })),
        Err(e) => json!({ "error": format!("not JSON: {e}") }),
    };
    let mut answer = answer;
    let problems = animals_core::problems();
    if !problems.is_empty()
        && let Some(o) = answer.as_object_mut()
    {
        o.insert("problems".into(), json!(problems));
    }
    OUT.with(|o| {
        let mut o = o.borrow_mut();
        *o = serde_json::to_vec(&answer).unwrap_or_default();
        o.as_ptr()
    })
}

/// The length of the last answer (`call`).
#[unsafe(no_mangle)]
pub extern "C" fn out_len() -> usize {
    OUT.with(|o| o.borrow().len())
}

/// Paints the coat of genome `hex` (its bytes, `len` of them) over the texture at `rgba` (rows of
/// `width` pixels, `height` of them), as each player's game paints it.
///
/// # Safety
/// `genome` points at `len` bytes, `rgba` at `width * height * 4`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn paint(genome: *const u8, len: usize, rgba: *mut u8, width: usize, height: usize) {
    let g = Genome::from_bytes(unsafe { std::slice::from_raw_parts(genome, len) });
    let Some(index) = (g.species as usize).checked_sub(1) else { return };
    let px = unsafe { std::slice::from_raw_parts_mut(rgba, width * height * 4) };
    coats::paint(index, &g, px, width, height);
}

// ------------------------------------------------------------------ randomness

/// Seeded randomness (xorshift), so a page can repeat what happened.
struct Seeded(u32);

impl Seeded {
    fn new(seed: u64) -> Seeded {
        Seeded((seed as u32 ^ (seed >> 32) as u32) | 1)
    }
    fn u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { self.u32() % n }
    }
}

impl Rng for Seeded {
    fn next(&mut self) -> f32 {
        (self.u32() >> 8) as f32 / (1u32 << 24) as f32
    }
}

// ------------------------------------------------------------------ requests

fn hex(g: &Genome) -> String {
    g.to_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Result<Genome, String> {
    let bytes: Result<Vec<u8>, _> = (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16))
        .collect();
    let g = Genome::from_bytes(&bytes.map_err(|_| "a genome is hex")?);
    if g.species == 0 || g.species as usize > SPECIES.len() {
        return Err("a genome of no species".into());
    }
    Ok(g)
}

fn genome_arg(req: &Value, key: &str) -> Result<Genome, String> {
    unhex(
        req[key]
            .as_str()
            .ok_or(format!("`{key}`: a genome (hex)"))?,
    )
}

fn seed_arg(req: &Value) -> u64 {
    req["seed"].as_u64().unwrap_or(0x9e37_79b9)
}

/// A species row by name and breed (`{"species": "dog", "breed": "chihuahua"}`) or index.
fn species_arg(req: &Value) -> Result<usize, String> {
    if let Some(i) = req["species"].as_u64() {
        return (i as usize)
            .lt(&SPECIES.len())
            .then_some(i as usize)
            .ok_or("no such species".into());
    }
    let name = req["species"]
        .as_str()
        .ok_or("`species`: a name or index")?;
    let breed = req["breed"].as_str();
    SPECIES
        .iter()
        .position(|s| s.name == name && (breed.is_none() || s.breed == breed))
        .ok_or(format!("no species {name}"))
}

fn habitat_arg(req: &Value) -> Habitat {
    req["habitat"]
        .as_str()
        .and_then(Habitat::named)
        .unwrap_or(Habitat::Grassland)
}

/// Everything the page shows about a genome.
fn about(g: &Genome) -> Value {
    let Some((loci, look)) = coats::describe(g) else { return json!(null) };
    json!({
        "genome": hex(g),
        "species": g.species - 1,
        "female": g.female != 0,
        "stage": g.stage,
        "loci": loci,
        "look": look,
        "build": coats::build(g),
        "size": coats::size(g),
        "breath": coats::breath(g),
    })
}

fn handle(req: &Value) -> Result<Value, String> {
    match req["op"].as_str().unwrap_or("") {
        // The species table and the habitats.
        "species" => Ok(json!({
            "species": SPECIES.iter().enumerate().map(|(i, s)| json!({
                "index": i, "name": s.name, "breed": s.breed, "model": s.model, "binomial": s.binomial,
                "gestation": s.gestation, "incubation": s.incubation, "litter": s.litter, "growth": s.growth,
                "recovery": s.recovery, "birth_size": s.birth_size, "dimorphism": s.dimorphism,
                "height": s.height.0, "flight": s.flight, "camouflage": s.camouflage,
            })).collect::<Vec<_>>(),
            "habitats": HABITATS.iter().map(|h| h.name()).collect::<Vec<_>>(),
            "day": life::DAY,
        })),
        // A wild one from its habitat's kind (or the pool given), of the sex asked or either.
        "random" => {
            let index = species_arg(req)?;
            let mut r = Seeded::new(seed_arg(req));
            let pool = match req.get("pool") {
                Some(p) if !p.is_null() => serde_json::from_value::<Pool>(p.clone()).map_err(|e| e.to_string())?,
                _ => coats::wild_pool(index, habitat_arg(req).name()).ok_or("no gene file")?,
            };
            let female = req["female"].as_bool().unwrap_or_else(|| r.next() < 0.5);
            let g = coats::sample(index, female, &pool, &mut r).ok_or("no gene file")?;
            Ok(about(&g))
        }
        // A baby of two parents (as many as `count`), or why they can't have one.
        "breed" => {
            let (m, f) = (genome_arg(req, "mother")?, genome_arg(req, "father")?);
            can_breed(&m, &f)?;
            let mut r = Seeded::new(seed_arg(req));
            let n = req["count"].as_u64().unwrap_or(1).clamp(1, 64);
            let young: Result<Vec<Value>, String> = (0..n)
                .map(|_| {
                    let mut g = coats::breed(&m, &f, &mut r).ok_or("no gene file")?;
                    g.stage = BABY;
                    Ok(about(&g))
                })
                .collect();
            Ok(json!({ "young": young? }))
        }
        "describe" => Ok(about(&genome_arg(req, "genome")?)),
        // The genome changed: its stage, a scar, a gene's alleles or values (testing edge cases).
        "edit" => {
            let mut g = genome_arg(req, "genome")?;
            if let Some(st) = req["stage"].as_u64() {
                g.stage = st.min(ADULT as u64) as u8;
            }
            if let Some(k) = req["scar"].as_u64() {
                coats::scar(&mut g, k as u8, req["place"].as_u64().unwrap_or(0) as u8);
            }
            if req["clear_scars"].as_bool() == Some(true) {
                g.scars = [0; 8];
            }
            if let (Some(id), Some(values)) = (req["locus"].as_str(), req["values"].as_array()) {
                let v: Vec<Value> = values.clone();
                coats::set_locus(&mut g, id, &v).ok_or(format!("can't set {id} to {values:?}"))?;
            }
            Ok(about(&g))
        }
        // A habitat's wild pool, and it evolved `generations` more (the game's natural
        // selection, drift and mutation; one in-game day each), with a dozen of its animals.
        "evolve" => {
            let index = species_arg(req)?;
            let h = habitat_arg(req);
            let mut r = Seeded::new(seed_arg(req));
            let mut pool = match req.get("pool") {
                Some(p) if !p.is_null() => serde_json::from_value::<Pool>(p.clone()).map_err(|e| e.to_string())?,
                _ => coats::wild_pool(index, h.name()).ok_or("no gene file")?,
            };
            for _ in 0..req["generations"].as_u64().unwrap_or(0).min(2000) {
                if let Some(next) = evolution::generation(index, &pool, h, &mut r) {
                    pool = next;
                }
            }
            let sample: Vec<Value> = (0..12)
                .filter_map(|i| coats::sample(index, i % 2 == 0, &pool, &mut r))
                .map(|g| about(&g))
                .collect();
            Ok(json!({ "pool": pool, "text": coats::describe_pool(index, &pool), "sample": sample, "background": h.background() }))
        }
        op if op.starts_with("sim_") => SIM.with(|s| sim(&mut s.borrow_mut(), op, req)),
        op => Err(format!("unknown op {op:?}")),
    }
}

/// Why two animals can't have young together, as the game would refuse them.
fn can_breed(m: &Genome, f: &Genome) -> Result<(), String> {
    let (sm, sf) = (&SPECIES[m.species as usize - 1], &SPECIES[f.species as usize - 1]);
    if !same_species(sm, sf) {
        return Err(format!("{} and {} are different species", sm.binomial, sf.binomial));
    }
    if m.female == 0 || f.female != 0 {
        return Err("a mother must be female and a father male".into());
    }
    Ok(())
}

// ------------------------------------------------------------------ the simulator

#[derive(Serialize, Deserialize, Clone)]
struct Animal {
    id: u32,
    genome: String,
    species: usize,
    female: bool,
    /// Tick it was born (or hatched); wild ones start grown, born at 0 or when added.
    born: u64,
    mother: u32,
    father: u32,
    generation: u32,
    /// Ticks of growing up left.
    growing: u32,
    stage: u8,
    /// Ticks before it can breed again.
    rest: u32,
    /// While carrying: ticks left, the father and his genes.
    carrying: u32,
    mate: u32,
    mate_genome: String,
    wild: [u8; STATS],
    /// Tick it died, and why (0: alive).
    died: u64,
    cause: String,
    /// An egg not hatched yet: ticks left.
    egg: u32,
}

#[derive(Default)]
struct Sim {
    animals: Vec<Animal>,
    tick: u64,
    next: u32,
    rng: Option<Seeded>,
    /// Matings the simulator makes by itself (adults of a species, one a day each at most).
    auto: bool,
    /// Old age (not in the game yet): days an adult lives, 0 for ever.
    lifespan: u32,
    /// Animals alive at most: no matings past it (as a pen or a habitat holds only so many).
    cap: u32,
    log: Vec<String>,
}

impl Sim {
    fn rng(&mut self) -> &mut Seeded {
        self.rng.get_or_insert_with(|| Seeded::new(1))
    }
    fn get(&self, id: u32) -> Option<&Animal> {
        self.animals.iter().find(|a| a.id == id)
    }
    fn add(&mut self, mut a: Animal) -> u32 {
        self.next += 1;
        a.id = self.next;
        self.animals.push(a);
        self.next
    }
    fn say(&mut self, text: String) {
        let day = self.tick as f64 / life::DAY as f64;
        self.log.push(format!("day {day:.1}: {text}"));
        if self.log.len() > 400 {
            self.log.drain(..100);
        }
    }
    fn state(&self) -> Value {
        json!({
            "tick": self.tick,
            "day": self.tick as f64 / life::DAY as f64,
            "auto": self.auto,
            "lifespan": self.lifespan,
            "cap": self.cap,
            "animals": self.animals.iter().map(|a| {
                let mut v = serde_json::to_value(a).unwrap_or_default();
                if let Ok(g) = unhex(&a.genome) {
                    let look = coats::describe(&g).map(|(_, l)| l);
                    v["look"] = json!(look);
                    v["size"] = json!(coats::size(&g) * life::size(&SPECIES[a.species], a.growing));
                }
                v
            }).collect::<Vec<_>>(),
            "log": self.log,
        })
    }
}

fn wild(sim: &mut Sim, index: usize, female: Option<bool>, h: Habitat) -> Result<u32, String> {
    let pool = coats::wild_pool(index, h.name()).ok_or("no gene file")?;
    let r = sim.rng();
    let female = female.unwrap_or_else(|| r.next() < 0.5);
    let g = coats::sample(index, female, &pool, r).ok_or("no gene file")?;
    let points: [u8; STATS] = std::array::from_fn(|_| (r.below(8)) as u8);
    let born = sim.tick;
    let id = sim.add(Animal {
        id: 0,
        genome: hex(&g),
        species: index,
        female,
        born,
        mother: 0,
        father: 0,
        generation: 0,
        growing: 0,
        stage: ADULT,
        rest: 0,
        carrying: 0,
        mate: 0,
        mate_genome: String::new(),
        wild: points,
        died: 0,
        cause: String::new(),
        egg: 0,
    });
    let name = coats::describe(&g).map_or(String::new(), |(_, l)| l.name);
    sim.say(format!("#{id} joined from the {}: {name}", h.name()));
    Ok(id)
}

/// Mother `m` and father `f` mate, as the game lets two in love: or why not.
fn mate(sim: &mut Sim, m: u32, f: u32) -> Result<(), String> {
    let (Some(ma), Some(fa)) = (sim.get(m).cloned(), sim.get(f).cloned()) else { return Err("no such animal".into()) };
    for a in [&ma, &fa] {
        if a.died != 0 {
            return Err(format!("#{} is dead", a.id));
        }
        if a.egg > 0 || a.stage != ADULT {
            return Err(format!("#{} isn't grown up", a.id));
        }
        if a.rest > 0 {
            return Err(format!("#{} is resting ({} min more)", a.id, a.rest.div_ceil(1200)));
        }
    }
    if ma.carrying > 0 {
        return Err(format!("#{m} is already expecting"));
    }
    if ma.mother == f || fa.mother == m || (ma.mother != 0 && ma.mother == fa.mother) {
        // The game doesn't stop this; said so it can be watched.
        sim.say(format!("#{m} and #{f} are close family"));
    }
    can_breed(&unhex(&ma.genome)?, &unhex(&fa.genome)?)?;
    let s = &SPECIES[ma.species];
    let (carry, recovery) = (life::carrying(s), s.recovery);
    for a in sim.animals.iter_mut() {
        if a.id == m {
            (a.carrying, a.mate, a.mate_genome, a.rest) = (carry, f, fa.genome.clone(), carry + recovery);
        } else if a.id == f {
            a.rest = life::MALE_REST;
        }
    }
    sim.say(format!("#{m} and #{f} mated"));
    Ok(())
}

/// The time has come for mother `id`: her litter born, or her clutch laid.
fn deliver(sim: &mut Sim, id: u32) {
    let Some(m) = sim.get(id).cloned() else { return };
    let (Ok(mg), Ok(fg)) = (unhex(&m.genome), unhex(&m.mate_genome)) else { return };
    let father = sim.get(m.mate).cloned();
    let s = &SPECIES[m.species];
    let n = life::litter(s, sim.rng().u32());
    let generation = m
        .generation
        .max(father.as_ref().map_or(0, |f| f.generation))
        + 1;
    let laying = s.gestation == 0;
    let mut born = Vec::new();
    for _ in 0..n {
        let r = sim.rng();
        let Some(mut g) = coats::breed(&mg, &fg, r) else { continue };
        g.stage = BABY;
        let fw = father.as_ref().map_or(m.wild, |f| f.wild);
        let wild = life::wild_points(&m.wild, &fw, |k| r.below(k));
        let tick = sim.tick;
        let child = sim.add(Animal {
            id: 0,
            genome: hex(&g),
            species: m.species,
            female: g.female != 0,
            born: tick,
            mother: id,
            father: m.mate,
            generation,
            growing: s.growth,
            stage: BABY,
            rest: 0,
            carrying: 0,
            mate: 0,
            mate_genome: String::new(),
            wild,
            died: 0,
            cause: String::new(),
            egg: if laying { s.incubation } else { 0 },
        });
        born.push(child);
    }
    let what = if laying { "laid eggs" } else { "had young" };
    let list = born
        .iter()
        .map(|b| format!("#{b}"))
        .collect::<Vec<_>>()
        .join(", ");
    sim.say(format!("#{id} {what}: {list}"));
}

/// The simulator moves on `ticks`: timers, births, hatching, growing up, matings (if automatic)
/// and old age (if set), a step of at most a tenth of a day at a time.
fn advance(sim: &mut Sim, ticks: u64) {
    let mut left = ticks;
    while left > 0 {
        let dt = left.min(life::DAY / 10) as u32;
        left -= dt as u64;
        sim.tick += dt as u64;
        let mut due = Vec::new();
        let mut hatched = Vec::new();
        let mut died = Vec::new();
        let (tick, lifespan) = (sim.tick, sim.lifespan as u64 * life::DAY);
        for a in sim.animals.iter_mut().filter(|a| a.died == 0) {
            if a.egg > 0 {
                a.egg = a.egg.saturating_sub(dt);
                if a.egg == 0 {
                    a.born = tick;
                    hatched.push(a.id);
                }
                continue;
            }
            let s = &SPECIES[a.species];
            a.rest = a.rest.saturating_sub(dt);
            a.growing = a.growing.saturating_sub(dt);
            a.stage = life::stage(s, a.growing, a.stage);
            if a.carrying > 0 {
                a.carrying = a.carrying.saturating_sub(dt);
                if a.carrying == 0 {
                    due.push(a.id);
                }
            }
            if lifespan > 0 && tick.saturating_sub(a.born) > lifespan + s.growth as u64 {
                a.died = tick;
                a.cause = "old age".into();
                died.push(a.id);
            }
        }
        for id in hatched {
            sim.say(format!("#{id} hatched"));
        }
        for id in died {
            sim.say(format!("#{id} died of old age"));
        }
        for id in due {
            deliver(sim, id);
        }
        if sim.auto {
            auto_mate(sim);
        }
    }
}

/// Automatic matings: each grown, rested female finds a grown, rested male of her species now
/// and then (about once a day each).
fn auto_mate(sim: &mut Sim) {
    let alive = sim.animals.iter().filter(|a| a.died == 0).count() as u32;
    if alive >= sim.cap.max(2) {
        return;
    }
    let ready = |a: &Animal| a.died == 0 && a.egg == 0 && a.stage == ADULT && a.rest == 0 && a.carrying == 0;
    let females: Vec<u32> = sim
        .animals
        .iter()
        .filter(|a| a.female && ready(a))
        .map(|a| a.id)
        .collect();
    for m in females {
        if sim.rng().below(10) != 0 {
            continue;
        }
        let Some(ma) = sim.get(m).cloned() else { continue };
        let males: Vec<u32> = sim
            .animals
            .iter()
            .filter(|a| !a.female && ready(a) && same_species(&SPECIES[a.species], &SPECIES[ma.species]))
            .map(|a| a.id)
            .collect();
        if males.is_empty() {
            continue;
        }
        let f = males[sim.rng().below(males.len() as u32) as usize];
        let _ = mate(sim, m, f);
    }
}

fn sim(sim: &mut Sim, op: &str, req: &Value) -> Result<Value, String> {
    match op {
        "sim_reset" => {
            *sim = Sim { rng: Some(Seeded::new(seed_arg(req))), cap: 150, ..Sim::default() };
            sim.say("a new world".into());
        }
        "sim_wild" => {
            let index = species_arg(req)?;
            let n = req["count"].as_u64().unwrap_or(1).clamp(1, 50);
            for _ in 0..n {
                wild(sim, index, req["female"].as_bool(), habitat_arg(req))?;
            }
        }
        "sim_mate" => mate(sim, req["mother"].as_u64().unwrap_or(0) as u32, req["father"].as_u64().unwrap_or(0) as u32)?,
        "sim_advance" => advance(
            sim,
            req["ticks"]
                .as_u64()
                .unwrap_or(life::DAY)
                .min(life::DAY * 3650),
        ),
        "sim_kill" => {
            let id = req["id"].as_u64().unwrap_or(0) as u32;
            let tick = sim.tick;
            let a = sim
                .animals
                .iter_mut()
                .find(|a| a.id == id && a.died == 0)
                .ok_or("no such living animal")?;
            (a.died, a.cause) = (tick, req["cause"].as_str().unwrap_or("killed").into());
            sim.say(format!("#{id} died"));
        }
        "sim_scar" => {
            let id = req["id"].as_u64().unwrap_or(0) as u32;
            let (k, place) = (req["kind"].as_u64().unwrap_or(1) as u8, sim.rng().below(256) as u8);
            let a = sim
                .animals
                .iter_mut()
                .find(|a| a.id == id)
                .ok_or("no such animal")?;
            let mut g = unhex(&a.genome)?;
            coats::scar(&mut g, k, place);
            a.genome = hex(&g);
        }
        "sim_settings" => {
            if let Some(b) = req["auto"].as_bool() {
                sim.auto = b;
            }
            if let Some(d) = req["lifespan"].as_u64() {
                sim.lifespan = d.min(100_000) as u32;
            }
            if let Some(n) = req["cap"].as_u64() {
                sim.cap = n.clamp(2, 1000) as u32;
            }
        }
        "sim_state" => {}
        _ => return Err(format!("unknown op {op:?}")),
    }
    // The stage each genome shows (a young one looks young).
    for a in sim.animals.iter_mut() {
        if let Ok(mut g) = unhex(&a.genome)
            && g.stage != a.stage
        {
            g.stage = a.stage;
            a.genome = hex(&g);
        }
    }
    Ok(sim.state())
}
