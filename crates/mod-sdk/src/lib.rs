//! Write sandboxed chiacraft mods in Rust (`docs/sandboxed-mods.md`).
//!
//! A sandboxed mod is a package's server half: a WebAssembly component the game downloads with
//! the package and runs where the world is simulated, with no way to the player's account, files
//! or the network. This crate gives it the names of Weft's SDK (`crates/sdk`), so the server code
//! of a Weft mod recompiles with few changes:
//!
//! ```ignore
//! use chiacraft_mod_sdk::*;
//!
//! declare_mod!(init: init);
//!
//! fn init(r: &mut Registry) {
//!     r.command("hello", "[who:targets]", |cmd| cmd.reply("command.hello.hi", &[]))
//!         .permission(permission::EVERYONE);
//!     r.on(&BLOCK_BROKEN, |b| for e in b.iter() { info!("{} broke {:?}", e.player, e.pos) });
//!     r.intercept(&question::BREAK_BLOCK, |q| if q.pos()[1] < 0 { Verdict::Deny } else { Verdict::Abstain });
//!     r.modify(&chain::DAMAGE, stage::MULTIPLY, |m| m.value *= 0.5);
//! }
//! ```
//!
//! What differs from Weft's SDK: only server code (no content builders, screens or client hooks
//! yet: content comes from the package's definitions); a handle's id is looked up by name in the
//! world the mod runs in; a component's value is read and set by value
//! ([`World::component`], [`World::set_component`], [`World::update_component`]) rather than by
//! reference; and a few calls of our own ([`World::tell`], [`World::player_key`],
//! [`World::teleport`], [`storage`]).
//!
//! API 1.1: components, `set_blocks`, sounds and particles. API 1.2 (this SDK): callbacks on the
//! package's own blocks and items (`r.block(&B).on_use(..)`, `r.item(&I).on_use(..)`), channels
//! (`r.packet`, `w.send`) and `w.schedule`. A mod built with it needs a game that runs 1.2; one
//! built with an older SDK runs in every later 1.x game.
//!
//! Build with `cargo build --release --target wasm32-unknown-unknown`, then
//! `cargo run -p chiacraft-sandbox --features pack --bin componentize -- <module.wasm>
//! <package>/code/server.wasm`.

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "../sandbox/wit/v1.3",
        world: "server-mod",
        pub_export_macro: true,
        export_macro_name: "__export",
        default_bindings_module: "chiacraft_mod_sdk::bindings",
    });
}

use bindings::chiacraft::mods::{command as wcommand, names, registry, storage as wstorage, types, world as wworld};
#[doc(hidden)]
pub use bindings::exports::chiacraft::mods::content_hooks::Guest as __ContentGuest;
use bindings::exports::chiacraft::mods::hooks as whooks;
#[doc(hidden)]
pub use bindings::exports::chiacraft::mods::hooks::{
    EventValue as __EventValue, Guest as __Guest, Invocation as __Invocation, Value as __Value, Verdict as __Verdict,
};
/// What this SDK shares with the client SDK (`chiacraft-mod-common`).
pub use chiacraft_mod_common::{Arg, ItemStack, Payload, Value, value};
use chiacraft_mod_common::{value_bytes, value_from};
#[doc(hidden)]
pub use types::{
    BlockUpdateEvent as __BlockUpdate, BlockUseEvent as __BlockUse, ItemUseEvent as __ItemUse, ItemUseResult as __ItemUseResult,
};

/// Permission levels (`permission::EVERYONE` .. `permission::OWNER`) and component flags: Weft's
/// own, so each has one name. A sandboxed component takes `SAVE`, `SYNC_OWNER`, `SYNC_WATCHERS` and
/// `KEEP_ON_RESPAWN` (`SYNC_MENU` is for machines' menus, which a sandboxed mod has none of).
pub use weft_abi::{component_flags, permission};

/// A value chain's stages for [`Registry::modify`]: the numbers the API has always carried. The
/// host runs them in Weft's phases (`ADD`, `MULTIPLY`, `FINAL`); Weft's own `stage` is now
/// world generation's.
pub mod stage {
    pub const ADD: u32 = 100;
    pub const MULTIPLY: u32 = 200;
    pub const FINAL: u32 = 300;
}

/// The API version this SDK builds for.
pub const API: &str = "1.3.0";

// ------------------------------------------------------------------ entry point

/// Declares the mod. `deps` is accepted for Weft's sake and ignored: a package names what it
/// needs in its `package.toml`.
///
/// ```ignore
/// chiacraft_mod_sdk::declare_mod!(init: init);
/// ```
#[macro_export]
macro_rules! declare_mod {
    (deps: [$($dep:literal),* $(,)?], init: $init:path $(,)?) => {
        $crate::declare_mod!(init: $init);
    };
    (init: $init:path $(,)?) => {
        struct __ChiacraftMod;
        impl $crate::__Guest for __ChiacraftMod {
            fn init() -> Result<(), String> {
                $crate::__init($init)
            }
            fn command(callback: u32, call: $crate::__Invocation) {
                $crate::__command(callback, call)
            }
            fn event(callback: u32, batch: Vec<$crate::__EventValue>) {
                $crate::__event(callback, batch)
            }
            fn question(callback: u32, q: $crate::Query) -> $crate::__Verdict {
                $crate::__question(callback, q)
            }
            fn modify(callback: u32, q: $crate::Query, base: f64, value: f64) -> f64 {
                $crate::__modify(callback, q, base, value)
            }
            fn world_tick(callback: u32) {
                $crate::__world_tick(callback)
            }
        }
        impl $crate::__ContentGuest for __ChiacraftMod {
            fn block_use(callback: u32, used: $crate::__BlockUse) -> bool {
                $crate::__block_use(callback, used)
            }
            fn block_update(callback: u32, update: $crate::__BlockUpdate) {
                $crate::__block_update(callback, update)
            }
            fn item_use(callback: u32, used: $crate::__ItemUse) -> $crate::__ItemUseResult {
                $crate::__item_use(callback, used)
            }
            fn packet(callback: u32, player: u64, data: Vec<u8>) {
                $crate::__packet(callback, player, data)
            }
        }
        $crate::bindings::__export!(__ChiacraftMod with_types_in $crate::bindings);
    };
}

type CommandFn = Rc<dyn Fn(&mut ServerCommand)>;
type EventFn = Rc<dyn Fn(&[__EventValue])>;
type BlockUseFn = Rc<dyn Fn(&mut Use) -> bool>;
type ItemUseFn = Rc<dyn Fn(&mut ItemUse) -> bool>;
type PacketFn = Rc<dyn Fn(&Packet, &[u8])>;

enum Callback {
    Command(CommandFn),
    Event(EventFn),
    BlockUse(BlockUseFn),
    BlockUpdate(Rc<dyn Fn(&mut BlockEvent)>),
    ItemUse(ItemUseFn),
    Packet(PacketFn),
    Question(Rc<dyn Fn(&Ask) -> Verdict>),
    Modify(Rc<dyn Fn(&mut Modify)>),
    Tick(Rc<dyn Fn(&World)>),
}

thread_local! {
    /// The callbacks by number: the game numbers registrations in order, and so do we.
    static CALLBACKS: RefCell<Vec<Callback>> = const { RefCell::new(Vec::new()) };
}

fn push(id: u32, cb: Callback) {
    CALLBACKS.with(|c| {
        if let Err(expected) = chiacraft_mod_common::number(&mut c.borrow_mut(), id, cb) {
            error!("the game numbered callback {id}, expected {expected}");
        }
    });
}

fn callback<T>(id: u32, pick: impl FnOnce(&Callback) -> Option<T>) -> Option<T> {
    CALLBACKS.with(|c| c.borrow().get(id as usize).and_then(pick))
}

#[doc(hidden)]
pub fn __init(init: fn(&mut Registry)) -> Result<(), String> {
    CALLBACKS.with(|c| c.borrow_mut().clear());
    let mut r = Registry { errors: Vec::new() };
    init(&mut r);
    if r.errors.is_empty() { Ok(()) } else { Err(r.errors.join("; ")) }
}

#[doc(hidden)]
pub fn __command(id: u32, call: __Invocation) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Command(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&mut ServerCommand { call });
    }
}

#[doc(hidden)]
pub fn __event(id: u32, batch: Vec<__EventValue>) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Event(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&batch);
    }
}

#[doc(hidden)]
pub fn __question(id: u32, q: Query) -> __Verdict {
    let f = callback(id, |c| match c {
        Callback::Question(f) => Some(f.clone()),
        _ => None,
    });
    match f.map(|f| f(&Ask { query: q })) {
        Some(Verdict::Allow) => __Verdict::Allow,
        Some(Verdict::Deny) => __Verdict::Deny,
        _ => __Verdict::Abstain,
    }
}

#[doc(hidden)]
pub fn __modify(id: u32, q: Query, base: f64, value: f64) -> f64 {
    let Some(f) = callback(id, |c| match c {
        Callback::Modify(f) => Some(f.clone()),
        _ => None,
    }) else {
        return value;
    };
    let mut m = Modify { query: q, base, value };
    f(&mut m);
    m.value
}

#[doc(hidden)]
pub fn __world_tick(id: u32) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Tick(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&World { _p: PhantomData });
    }
}

#[doc(hidden)]
pub fn __block_use(id: u32, u: __BlockUse) -> bool {
    let Some(f) = callback(id, |c| match c {
        Callback::BlockUse(f) => Some(f.clone()),
        _ => None,
    }) else {
        return false;
    };
    f(&mut Use { player: u.player, pos: p3(u.pos), state: u.state, face: u.face })
}

#[doc(hidden)]
pub fn __block_update(id: u32, u: __BlockUpdate) {
    if let Some(f) = callback(id, |c| match c {
        Callback::BlockUpdate(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&mut BlockEvent { pos: p3(u.pos), state: u.state, other: p3(u.other), entity: u.entity });
    }
}

#[doc(hidden)]
pub fn __item_use(id: u32, u: __ItemUse) -> __ItemUseResult {
    let Some(f) = callback(id, |c| match c {
        Callback::ItemUse(f) => Some(f.clone()),
        _ => None,
    }) else {
        return __ItemUseResult { handled: false, stack: None };
    };
    let stack: ItemStack = u.stack.into();
    let mut used = ItemUse { player: u.player, stack, slot: u.slot, target: u.target.map(|(p, face)| (p3(p), face)), changed: false };
    let handled = f(&mut used);
    __ItemUseResult { handled, stack: used.changed.then(|| used.stack.into()) }
}

#[doc(hidden)]
pub fn __packet(id: u32, player: u64, data: Vec<u8>) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Packet(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&Packet { player }, &data);
    }
}

// ------------------------------------------------------------------ logging

chiacraft_mod_common::__glue!(bindings::chiacraft::mods::log, bindings::chiacraft::mods::types);

#[macro_export]
macro_rules! error { ($($t:tt)*) => { $crate::__log(0, format_args!($($t)*)) } }
#[macro_export]
macro_rules! warn { ($($t:tt)*) => { $crate::__log(1, format_args!($($t)*)) } }
#[macro_export]
macro_rules! info { ($($t:tt)*) => { $crate::__log(2, format_args!($($t)*)) } }
#[macro_export]
macro_rules! debug { ($($t:tt)*) => { $crate::__log(3, format_args!($($t)*)) } }
#[macro_export]
macro_rules! trace { ($($t:tt)*) => { $crate::__log(3, format_args!($($t)*)) } }

// ------------------------------------------------------------------ shared values

pub use types::Query;

impl From<types::ItemStack> for ItemStack {
    fn from(s: types::ItemStack) -> ItemStack {
        ItemStack { item: s.item, count: s.count, data: s.data }
    }
}

impl From<ItemStack> for types::ItemStack {
    fn from(s: ItemStack) -> types::ItemStack {
        types::ItemStack { item: s.item, count: s.count, data: s.data }
    }
}

/// An entity as the world has it now (the fields of Weft's `EntityState` the sandbox carries).
#[derive(Clone, Copy, Debug)]
pub struct EntityState {
    pub kind: u32,
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: u32,
    pub in_liquid: u32,
    pub health: f32,
    pub max_health: f32,
    pub age: u64,
}

impl EntityState {
    /// The block its feet are in.
    pub fn block_pos(&self) -> [i32; 3] {
        self.pos.map(|v| v.floor() as i32)
    }
}

// Event values, as Weft's `abi` has them.

#[derive(Clone, Copy, Debug)]
pub struct BlockChange {
    pub player: u64,
    pub pos: [i32; 3],
    pub state: u32,
    pub item: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Death {
    pub entity: u64,
    pub killer: u64,
    pub kind: u32,
    pub cause: u32,
    pub pos: [f64; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerEvent {
    pub player: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct Hurt {
    pub entity: u64,
    pub source: u64,
    pub amount: f32,
    pub cause: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemUsed {
    pub player: u64,
    pub stack: ItemStack,
    pub pos: [i32; 3],
    /// 1 when used on the block at `pos`.
    pub has_target: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Pickup {
    pub player: u64,
    pub stack: ItemStack,
}

fn p3(p: (i32, i32, i32)) -> [i32; 3] {
    [p.0, p.1, p.2]
}

fn v3(p: (f64, f64, f64)) -> [f64; 3] {
    [p.0, p.1, p.2]
}

/// A value an event carries, from what the game sent.
pub trait EventValue: Sized + 'static {
    #[doc(hidden)]
    fn from_wire(v: &__EventValue) -> Option<Self>;
}

impl EventValue for BlockChange {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::BlockChange(b) => Some(BlockChange { player: b.player, pos: p3(b.pos), state: b.state, item: b.item }),
            _ => None,
        }
    }
}

impl EventValue for Death {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::Death(d) => Some(Death { entity: d.entity, killer: d.killer, kind: d.kind, cause: d.cause, pos: v3(d.pos) }),
            _ => None,
        }
    }
}

impl EventValue for PlayerEvent {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::Player(p) => Some(PlayerEvent { player: *p }),
            _ => None,
        }
    }
}

impl EventValue for Hurt {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::Hurt(h) => Some(Hurt { entity: h.entity, source: h.source, amount: h.amount, cause: h.cause }),
            _ => None,
        }
    }
}

impl EventValue for ItemUsed {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::ItemUsed(u) => {
                Some(ItemUsed { player: u.player, stack: u.stack.into(), pos: p3(u.pos), has_target: u.on_block as u32 })
            }
            _ => None,
        }
    }
}

impl EventValue for Pickup {
    fn from_wire(v: &__EventValue) -> Option<Self> {
        match v {
            __EventValue::Pickup(p) => Some(Pickup { player: p.player, stack: p.stack.into() }),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------ handles

/// A built-in event (Weft's `Event<T>`): the ones below.
pub struct Event<T> {
    which: registry::Event,
    _t: PhantomData<fn() -> T>,
}

impl<T> Event<T> {
    const fn new(which: registry::Event) -> Self {
        Event { which, _t: PhantomData }
    }
}

pub static BLOCK_BROKEN: Event<BlockChange> = Event::new(registry::Event::BlockBroken);
pub static BLOCK_PLACED: Event<BlockChange> = Event::new(registry::Event::BlockPlaced);
pub static ENTITY_DIED: Event<Death> = Event::new(registry::Event::EntityDied);
pub static ENTITY_HURT: Event<Hurt> = Event::new(registry::Event::EntityHurt);
pub static PLAYER_JOINED: Event<PlayerEvent> = Event::new(registry::Event::PlayerJoined);
pub static PLAYER_RESPAWNED: Event<PlayerEvent> = Event::new(registry::Event::PlayerRespawned);
pub static ITEM_USED: Event<ItemUsed> = Event::new(registry::Event::ItemUsed);
pub static ITEM_PICKED_UP: Event<Pickup> = Event::new(registry::Event::ItemPickedUp);

/// A question (Weft's `Question`).
pub struct Question(registry::Question);

/// The engine's questions, as in Weft's `question` module.
pub mod question {
    use super::{Question, registry::Question as Q};
    pub static BREAK_BLOCK: Question = Question(Q::BreakBlock);
    pub static PLACE_BLOCK: Question = Question(Q::PlaceBlock);
    pub static USE_BLOCK: Question = Question(Q::UseBlock);
    pub static USE_ITEM: Question = Question(Q::UseItem);
    pub static INTERACT_ENTITY: Question = Question(Q::InteractEntity);
    pub static ATTACK: Question = Question(Q::Attack);
    pub static HURT: Question = Question(Q::Hurt);
    pub static PICKUP_ITEM: Question = Question(Q::PickupItem);
    pub static ENTER_DIMENSION: Question = Question(Q::EnterDimension);
    pub static NATURAL_SPAWN: Question = Question(Q::NaturalSpawn);
}

/// A value chain (Weft's `Chain`).
pub struct Chain(registry::Chain);

pub mod chain {
    use super::{Chain, registry::Chain as C};
    pub static DAMAGE: Chain = Chain(C::Damage);
    pub static ATTACK_DAMAGE: Chain = Chain(C::AttackDamage);
}

/// A named thing of the game's content, looked up in the world the mod runs in. Each world has
/// its own ids, but a mod runs anew in each world, so a looked-up id is kept.
struct Lookup {
    name: &'static str,
    id: std::sync::OnceLock<Option<u32>>,
}

impl Lookup {
    const fn new(name: &'static str) -> Lookup {
        Lookup { name, id: std::sync::OnceLock::new() }
    }

    fn get(&self, find: fn(&str) -> Option<u32>) -> Option<u32> {
        if let Some(v) = self.id.get() {
            return *v;
        }
        let v = find(self.name);
        // Outside a callback nothing is found yet: look again next time.
        if v.is_some() {
            let _ = self.id.set(v);
        }
        v
    }
}

/// A block, by its full name (`Block::named("base:stone")`).
pub struct Block(Lookup);

impl Block {
    pub const fn named(name: &'static str) -> Block {
        Block(Lookup::new(name))
    }
    pub fn name(&self) -> &str {
        self.0.name
    }
    /// Its first state; `u32::MAX` if this world has no such block.
    pub fn id(&self) -> u32 {
        self.0.get(names::block_state).unwrap_or(u32::MAX)
    }
    /// Whether `state` is a state of this block.
    pub fn is(&self, state: u32) -> bool {
        let n = names::block(state);
        n.split('[').next() == Some(self.0.name)
    }
}

/// An item, by its full name.
pub struct Item(Lookup);

impl Item {
    pub const fn named(name: &'static str) -> Item {
        Item(Lookup::new(name))
    }
    pub fn name(&self) -> &str {
        self.0.name
    }
    pub fn id(&self) -> u32 {
        self.0.get(names::item_id).unwrap_or(u32::MAX)
    }
}

/// An entity kind, by its full name.
pub struct Entity(Lookup);

impl Entity {
    pub const fn named(name: &'static str) -> Entity {
        Entity(Lookup::new(name))
    }
    pub fn name(&self) -> &str {
        self.0.name
    }
    pub fn id(&self) -> u32 {
        self.0.get(names::entity_kind_id).unwrap_or(u32::MAX)
    }
}

/// The player kind (`core:player`).
pub static PLAYER: Entity = Entity::named("core:player");

/// Full name of a block state (`base:furnace[facing=north,lit=false]`).
pub fn block_name(state: u32) -> String {
    names::block(state)
}

pub fn item_name(item: u32) -> String {
    names::item(item)
}

pub fn state_prop(state: u32, name: &str) -> Option<String> {
    names::prop(state, name)
}

pub fn state_with(state: u32, name: &str, value: &str) -> Option<u32> {
    names::with_prop(state, name, value)
}

// ------------------------------------------------------------------ registering

/// What `init` registers with (Weft's `Registry`, its server half).
pub struct Registry {
    errors: Vec<String>,
}

impl Registry {
    /// A registry outside `init`, for tests: the game refuses what it registers.
    #[doc(hidden)]
    pub fn __late() -> Registry {
        Registry { errors: Vec::new() }
    }

    /// A server command; `usage` as Weft's. Asks for game master unless `.permission(..)` says
    /// otherwise, like Weft's.
    pub fn command(&mut self, name: &str, usage: &str, f: impl Fn(&mut ServerCommand) + 'static) -> CommandOptions<'_> {
        CommandOptions {
            reg: self,
            name: name.to_string(),
            usage: usage.to_string(),
            permission: permission::GAME_MASTER,
            f: Some(Rc::new(f)),
        }
    }

    /// Called with every batch of `event` (once per tick).
    pub fn on<T: EventValue>(&mut self, event: &Event<T>, f: impl Fn(&EventBatch<T>) + 'static) {
        let id = registry::on(event.which);
        push(
            id,
            Callback::Event(Rc::new(move |batch: &[__EventValue]| {
                let values: Vec<T> = batch.iter().filter_map(T::from_wire).collect();
                f(&EventBatch { values, world: World { _p: PhantomData } });
            })),
        );
    }

    /// Asked `question`: any deny wins.
    pub fn intercept(&mut self, question: &Question, f: impl Fn(&Ask) -> Verdict + 'static) {
        let id = registry::intercept(question.0);
        push(id, Callback::Question(Rc::new(f)));
    }

    /// A step of `chain` at `stage` (`stage::ADD`, `MULTIPLY`, `FINAL`).
    pub fn modify(&mut self, chain: &Chain, stage: u32, f: impl Fn(&mut Modify) + 'static) {
        let id = registry::modify(chain.0, stage);
        push(id, Callback::Modify(Rc::new(f)));
    }

    /// Once per tick in every region, for logic that is nobody's own.
    pub fn world_tick(&mut self, f: impl Fn(&World) + 'static) {
        let id = registry::world_tick();
        push(id, Callback::Tick(Rc::new(f)));
    }

    /// A per-entity value of this mod's (Weft's `entity_component`, 1.1): at most 256 bytes,
    /// `flags` from [`component_flags`]. Entities get it through [`Registry::attach`].
    pub fn entity_component<T: Value>(&mut self, c: &Component<T>, default: T, flags: u32) {
        let mut f = registry::ComponentFlags::empty();
        for (bit, flag) in [
            (component_flags::SAVE, registry::ComponentFlags::SAVE),
            (component_flags::SYNC_OWNER, registry::ComponentFlags::SYNC_OWNER),
            (component_flags::SYNC_WATCHERS, registry::ComponentFlags::SYNC_WATCHERS),
            (component_flags::KEEP_ON_RESPAWN, registry::ComponentFlags::KEEP),
        ] {
            if flags & bit != 0 {
                f = f | flag;
            }
        }
        if let Err(e) = registry::component(c.name, std::mem::size_of::<T>() as u32, f, &value_bytes(&default)) {
            self.errors.push(e);
        }
    }

    /// Every entity of `kind` carries component `c` (Weft's `attach`, 1.1).
    pub fn attach<T: Value>(&mut self, kind: &Entity, c: &Component<T>) {
        if let Err(e) = registry::attach(kind.name(), c.name) {
            self.errors.push(e);
        }
    }

    /// Callbacks on one of the package's own blocks (Weft's block builder, its callbacks only,
    /// 1.2): `r.block(&LAMP).on_use(|u| ..)`. The block itself comes from `defs/blocks`.
    pub fn block<'a>(&'a mut self, block: &Block) -> BlockCallbacks<'a> {
        BlockCallbacks { reg: self, name: block.name().to_string() }
    }

    /// Callbacks on one of the package's own items (1.2): `r.item(&WAND).on_use(|u| ..)`.
    pub fn item<'a>(&'a mut self, item: &Item) -> ItemCallbacks<'a> {
        ItemCallbacks { reg: self, name: item.name().to_string() }
    }

    /// What players' games send on `channel` (Weft's `packet`, 1.2). Its native counterpart on a
    /// client is a `Channel<Vec<u8>>` of the same full name: the bytes are `T`'s encoding.
    pub fn packet<T: Payload>(&mut self, channel: &Channel<T>, f: impl Fn(&Packet, T) + 'static) {
        match registry::channel(channel.local()) {
            Ok(id) => push(
                id,
                Callback::Packet(Rc::new(move |p: &Packet, data: &[u8]| {
                    if let Some(v) = T::decode(data) {
                        f(p, v)
                    }
                })),
            ),
            Err(e) => self.errors.push(e),
        }
    }
}

/// Callbacks being put on a block (1.2).
pub struct BlockCallbacks<'a> {
    reg: &'a mut Registry,
    name: String,
}

impl BlockCallbacks<'_> {
    /// A player used (right-clicked) it: true if that did something.
    pub fn on_use(self, f: impl Fn(&mut Use) -> bool + 'static) -> Self {
        match registry::on_block_use(&self.name) {
            Ok(id) => push(id, Callback::BlockUse(Rc::new(f))),
            Err(e) => self.reg.errors.push(e),
        }
        self
    }
    /// A neighbour changed (`e.other()`).
    pub fn on_neighbor(self, f: impl Fn(&mut BlockEvent) + 'static) -> Self {
        match registry::on_block_neighbor(&self.name) {
            Ok(id) => push(id, Callback::BlockUpdate(Rc::new(f))),
            Err(e) => self.reg.errors.push(e),
        }
        self
    }
    /// A tick scheduled with `w.schedule` came.
    pub fn on_scheduled(self, f: impl Fn(&mut BlockEvent) + 'static) -> Self {
        match registry::on_block_scheduled(&self.name) {
            Ok(id) => push(id, Callback::BlockUpdate(Rc::new(f))),
            Err(e) => self.reg.errors.push(e),
        }
        self
    }
}

/// Callbacks being put on an item (1.2).
pub struct ItemCallbacks<'a> {
    reg: &'a mut Registry,
    name: String,
}

impl ItemCallbacks<'_> {
    /// A player used it: true if that did something. `u.set_stack(..)` changes what is left in
    /// the hand.
    pub fn on_use(self, f: impl Fn(&mut ItemUse) -> bool + 'static) -> Self {
        match registry::on_item_use(&self.name) {
            Ok(id) => push(id, Callback::ItemUse(Rc::new(f))),
            Err(e) => self.reg.errors.push(e),
        }
        self
    }
}

/// A player using a block (Weft's `Use`, 1.2).
pub struct Use {
    player: u64,
    pos: [i32; 3],
    state: u32,
    face: u32,
}

impl Use {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn player(&self) -> u64 {
        self.player
    }
    pub fn pos(&self) -> [i32; 3] {
        self.pos
    }
    pub fn state(&self) -> u32 {
        self.state
    }
    pub fn face(&self) -> u32 {
        self.face
    }
}

/// A block told something happened (Weft's `BlockEvent`, 1.2).
pub struct BlockEvent {
    pos: [i32; 3],
    state: u32,
    other: [i32; 3],
    entity: u64,
}

impl BlockEvent {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn pos(&self) -> [i32; 3] {
        self.pos
    }
    pub fn state(&self) -> u32 {
        self.state
    }
    /// The neighbour that changed (its own position for a scheduled tick).
    pub fn other(&self) -> [i32; 3] {
        self.other
    }
    /// Its block entity, 0 if none.
    pub fn entity(&self) -> u64 {
        self.entity
    }
}

/// A player using an item (Weft's `ItemUse`, 1.2).
pub struct ItemUse {
    player: u64,
    stack: ItemStack,
    slot: u32,
    target: Option<([i32; 3], u32)>,
    changed: bool,
}

impl ItemUse {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn player(&self) -> u64 {
        self.player
    }
    /// The block it was used on and the face, if any.
    pub fn target(&self) -> Option<([i32; 3], u32)> {
        self.target
    }
    pub fn stack(&self) -> ItemStack {
        self.stack
    }
    /// Where the stack is in the player's inventory (ours).
    pub fn slot(&self) -> u32 {
        self.slot
    }
    /// What is left in the hand (its item and count; what the stack carries stays).
    pub fn set_stack(&mut self, s: ItemStack) {
        self.stack = s;
        self.changed = true;
    }
}

/// A message on a channel, typed (Weft's `Channel<T>`, 1.2): `Channel::named("boost")` is the
/// package's `<package>:boost`.
pub struct Channel<T: Payload> {
    name: &'static str,
    _t: PhantomData<fn() -> T>,
}

impl<T: Payload> Channel<T> {
    pub const fn named(name: &'static str) -> Self {
        Channel { name, _t: PhantomData }
    }
    pub fn name(&self) -> &str {
        self.name
    }
    /// The name within the package.
    fn local(&self) -> &str {
        self.name.rsplit(':').next().unwrap_or(self.name)
    }
}

/// A packet from a player's game (Weft's `Packet`, 1.2).
pub struct Packet {
    player: u64,
}

impl Packet {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn player(&self) -> u64 {
        self.player
    }
    /// Answers the sender on `channel`.
    pub fn reply<R: Payload>(&self, channel: &Channel<R>, value: &R) {
        self.world().send(self.player, channel, value);
    }
}

/// A command being registered: it is registered when this is dropped.
pub struct CommandOptions<'a> {
    reg: &'a mut Registry,
    name: String,
    usage: String,
    permission: u32,
    f: Option<CommandFn>,
}

impl CommandOptions<'_> {
    pub fn permission(mut self, level: u32) -> Self {
        self.permission = level;
        self
    }
}

impl Drop for CommandOptions<'_> {
    fn drop(&mut self) {
        let Some(f) = self.f.take() else { return };
        match registry::command(&self.name, &self.usage, self.permission) {
            Ok(id) => push(id, Callback::Command(f)),
            Err(e) => self.reg.errors.push(e),
        }
    }
}

/// Weft's verdicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Abstain,
    Allow,
    Deny,
}

/// A batch of one event (Weft's `EventBatch`).
pub struct EventBatch<T> {
    values: Vec<T>,
    world: World,
}

impl<T> EventBatch<T> {
    pub fn world(&self) -> World {
        self.world
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// A question being asked (Weft's `Ask`).
pub struct Ask {
    query: Query,
}

impl Ask {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
    pub fn entity(&self) -> u64 {
        self.query.entity
    }
    pub fn pos(&self) -> [i32; 3] {
        p3(self.query.pos)
    }
    pub fn state(&self) -> u32 {
        self.query.state
    }
    pub fn item(&self) -> u32 {
        self.query.item
    }
}

/// One step of a value chain (Weft's `Modify`).
pub struct Modify {
    query: Query,
    pub base: f64,
    pub value: f64,
}

impl Modify {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }
    pub fn query(&self) -> &Query {
        &self.query
    }
}

// ------------------------------------------------------------------ commands

/// A command being run (Weft's `ServerCommand`).
pub struct ServerCommand {
    call: __Invocation,
}

impl ServerCommand {
    pub fn world(&self) -> World {
        World { _p: PhantomData }
    }

    /// Who ran it.
    pub fn player(&self) -> u64 {
        self.call.player
    }

    fn value(&self, name: &str) -> Option<&__Value> {
        self.call
            .args
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    pub fn int(&self, name: &str) -> Option<i64> {
        match self.value(name)? {
            __Value::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn float(&self, name: &str) -> Option<f64> {
        match self.value(name)? {
            __Value::Int(v) => Some(*v as f64),
            __Value::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// Words, text, player names, and the full names of items, blocks, entities, dimensions.
    pub fn text(&self, name: &str) -> Option<&str> {
        match self.value(name)? {
            __Value::Text(s) => Some(s),
            __Value::Id((_, s)) => Some(s),
            _ => None,
        }
    }

    /// The id of an item, block state, entity kind or dimension argument.
    pub fn id(&self, name: &str) -> Option<u32> {
        match self.value(name)? {
            __Value::Id((id, _)) => Some(*id),
            _ => None,
        }
    }

    /// What a `targets` argument picked.
    pub fn targets(&self, name: &str) -> Vec<u64> {
        match self.value(name) {
            Some(__Value::Targets(t)) => t.clone(),
            _ => Vec::new(),
        }
    }

    pub fn pos(&self, name: &str) -> Option<[f64; 3]> {
        match self.value(name)? {
            __Value::Pos(p) => Some(v3(*p)),
            _ => None,
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.value(name).is_some()
    }

    /// A line for whoever ran the command: the translation `key` with `args`.
    pub fn reply(&mut self, key: &str, args: &[Arg]) {
        wcommand::reply(key, &wire_args(args));
    }
}

// ------------------------------------------------------------------ the world

/// The world a callback runs in (Weft's `World`). Only meaningful during a callback.
#[derive(Clone, Copy)]
pub struct World {
    _p: PhantomData<*const ()>,
}

/// A player's or block entity's inventory, as far as the sandbox reaches it.
pub struct Inventory {
    owner: u64,
}

impl Inventory {
    /// Puts `stack` in; what didn't fit comes back.
    pub fn insert(&self, stack: ItemStack) -> ItemStack {
        wworld::give(self.owner, stack.into()).into()
    }
}

impl World {
    pub fn get(&self, pos: [i32; 3]) -> u32 {
        wworld::get_block((pos[0], pos[1], pos[2]))
    }
    /// False if the position is not loaded.
    pub fn set(&self, pos: [i32; 3], state: u32) -> bool {
        wworld::set_block((pos[0], pos[1], pos[2]), state)
    }
    pub fn time(&self) -> u64 {
        wworld::time()
    }
    pub fn seed(&self) -> u64 {
        wworld::seed()
    }
    pub fn random(&self) -> u64 {
        wworld::random()
    }
    /// Uniform in `0..n`.
    pub fn random_below(&self, n: u32) -> u32 {
        (((self.random() >> 32) * n as u64) >> 32) as u32
    }
    pub fn dimension(&self) -> u32 {
        wworld::dimension()
    }
    pub fn light(&self, pos: [i32; 3]) -> u32 {
        wworld::light((pos[0], pos[1], pos[2]))
    }
    pub fn biome(&self, pos: [i32; 3]) -> u32 {
        wworld::biome((pos[0], pos[1], pos[2]))
    }
    pub fn players(&self) -> Vec<u64> {
        wworld::players()
    }
    pub fn entity(&self, id: u64) -> Option<EntityState> {
        wworld::get_entity(id).map(|e| EntityState {
            kind: e.kind,
            pos: v3(e.pos),
            vel: v3(e.vel),
            yaw: e.yaw,
            pitch: e.pitch,
            on_ground: e.on_ground as u32,
            in_liquid: e.in_liquid as u32,
            health: e.health,
            max_health: e.max_health,
            age: e.age,
        })
    }
    /// Entities within `radius` of `pos`, of `kind` if given.
    pub fn near(&self, pos: [f64; 3], radius: f64, kind: Option<&Entity>) -> Vec<u64> {
        let all = wworld::near((pos[0], pos[1], pos[2]), radius);
        match kind {
            None => all,
            Some(k) => {
                let id = k.id();
                all.into_iter()
                    .filter(|&e| wworld::get_entity(e).is_some_and(|s| s.kind == id))
                    .collect()
            }
        }
    }
    /// Moves an entity, a player too (ours: Weft does it with `set_entity`).
    pub fn teleport(&self, id: u64, pos: [f64; 3]) {
        wworld::teleport(id, (pos[0], pos[1], pos[2]));
    }
    pub fn push(&self, id: u64, v: [f64; 3]) {
        wworld::push(id, (v[0], v[1], v[2]));
    }
    pub fn damage(&self, id: u64, amount: f32, source: u64, cause: u32) -> bool {
        wworld::damage(id, amount, source, cause)
    }
    pub fn spawn(&self, kind: &Entity, pos: [f64; 3]) -> u64 {
        wworld::spawn(kind.name(), (pos[0], pos[1], pos[2]))
    }
    pub fn remove(&self, id: u64) {
        wworld::remove(id);
    }
    pub fn creative(&self, id: u64) -> bool {
        wworld::creative(id)
    }
    pub fn inventory(&self, owner: u64) -> Inventory {
        Inventory { owner }
    }
    pub fn spawn_item(&self, pos: [f64; 3], stack: ItemStack) {
        wworld::spawn_item((pos[0], pos[1], pos[2]), stack.into());
    }
    /// A line in a player's chat, the translation `key` with `args` (ours: Weft's mods send
    /// their own channel for this).
    pub fn tell(&self, player: u64, key: &str, args: &[Arg]) {
        wworld::tell(player, key, &wire_args(args));
    }
    /// The player earned an achievement here (1.3): the world signs a credential for them that
    /// outlives the world (`docs/sandboxed-mods.md`, "Achievements"). `id` is the mod's own name
    /// (the package's namespace goes in front); `title` and `description` are the package's lang
    /// keys or text; `icon` an item's full name. Granting it again signs it again: keep who has
    /// what in `storage`.
    pub fn grant_achievement(&self, player: u64, id: &str, title: &str, description: &str, icon: Option<&str>) {
        wworld::grant_achievement(player, id, title, description, icon);
    }
    /// Who a player is: their key, the same in every world (ours).
    pub fn player_key(&self, player: u64) -> String {
        wworld::player_key(player)
    }
    /// Many blocks at once (1.1): how many changed.
    pub fn set_blocks(&self, changes: &[([i32; 3], u32)]) -> u32 {
        let list: Vec<((i32, i32, i32), u32)> = changes
            .iter()
            .map(|(p, s)| ((p[0], p[1], p[2]), *s))
            .collect();
        wworld::set_blocks(&list)
    }
    /// An entity's value of one of this mod's components (Weft's `component`, by value; 1.1).
    pub fn component<T: Value>(&self, id: u64, c: &Component<T>) -> Option<T> {
        wworld::get_component(id, c.name).and_then(|b| value_from(&b))
    }
    /// Sets it (1.1); false if the entity doesn't carry it.
    pub fn set_component<T: Value>(&self, id: u64, c: &Component<T>, value: T) -> bool {
        wworld::set_component(id, c.name, &value_bytes(&value))
    }
    /// Changes it in place, the way Weft's `component_mut` is used (1.1); false if the entity
    /// doesn't carry it.
    pub fn update_component<T: Value>(&self, id: u64, c: &Component<T>, f: impl FnOnce(&mut T)) -> bool {
        let Some(mut v) = self.component(id, c) else { return false };
        f(&mut v);
        self.set_component(id, c, v)
    }
    /// A sound at `pos` (Weft's `play_sound`, 1.1).
    pub fn play_sound(&self, sound: &Sound, pos: [f64; 3], volume: f32) {
        wworld::play_sound(sound.0, (pos[0], pos[1], pos[2]), volume);
    }
    /// Particles at `pos` (Weft's `spawn_particles`, 1.1): `.count(n)`, sent when dropped.
    pub fn spawn_particles(&self, kind: &Particle, pos: [f64; 3]) -> Burst {
        Burst { kind: kind.0, pos, count: 1 }
    }
    /// A tick for the block at `pos` in `ticks`, which its `on_scheduled` gets (Weft's
    /// `schedule`, 1.2).
    pub fn schedule(&self, pos: [i32; 3], ticks: u32) {
        wworld::schedule((pos[0], pos[1], pos[2]), ticks);
    }
    /// `value` to `player`'s game on `channel` (Weft's `send`, 1.2).
    pub fn send<T: Payload>(&self, player: u64, channel: &Channel<T>, value: &T) {
        wworld::send(player, channel.local(), &value.encode());
    }
    /// The value of property `name` of the block at `pos`.
    pub fn prop(&self, pos: [i32; 3], name: &str) -> Option<String> {
        state_prop(self.get(pos), name)
    }
    /// Changes one property of the block at `pos`, keeping the others.
    pub fn set_prop(&self, pos: [i32; 3], name: &str, value: &str) -> bool {
        match state_with(self.get(pos), name, value) {
            Some(s) => {
                self.set(pos, s);
                true
            }
            None => false,
        }
    }
}

// ------------------------------------------------------------------ components, sounds, particles (1.1)

/// A per-entity value of this mod's, by its own name (Weft's `Component`); `size_of::<T>()` at
/// most 256 bytes.
pub struct Component<T: Value> {
    name: &'static str,
    _t: PhantomData<fn() -> T>,
}

impl<T: Value> Component<T> {
    pub const fn named(name: &'static str) -> Self {
        Component { name, _t: PhantomData }
    }
    pub fn name(&self) -> &str {
        self.name
    }
}

/// A sound, by full name (`Sound::named("base:block.stone.break")`).
pub struct Sound(&'static str);

impl Sound {
    pub const fn named(name: &'static str) -> Sound {
        Sound(name)
    }
}

/// A particle kind, by full name.
pub struct Particle(&'static str);

impl Particle {
    pub const fn named(name: &'static str) -> Particle {
        Particle(name)
    }
}

/// Particles being made (Weft's `Burst`): sent when dropped.
pub struct Burst {
    kind: &'static str,
    pos: [f64; 3],
    count: u32,
}

impl Burst {
    /// How many (1 to 256).
    pub fn count(mut self, n: u32) -> Self {
        self.count = n;
        self
    }
}

impl Drop for Burst {
    fn drop(&mut self) {
        wworld::spawn_particles(self.kind, (self.pos[0], self.pos[1], self.pos[2]), self.count);
    }
}

// ------------------------------------------------------------------ storage

/// The mod's own data in this world, kept in the world's save. Keys are 1 to 64 lowercase
/// letters, digits, `_`, `-` and `.`; at most 1024 keys and 4 MiB in all.
pub mod storage {
    use super::wstorage;

    pub fn get(key: &str) -> Option<Vec<u8>> {
        wstorage::get(key)
    }
    pub fn get_text(key: &str) -> Option<String> {
        get(key).and_then(|b| String::from_utf8(b).ok())
    }
    pub fn set(key: &str, value: &[u8]) -> Result<(), String> {
        wstorage::set(key, value)
    }
    pub fn remove(key: &str) {
        wstorage::remove(key)
    }
    pub fn keys() -> Vec<String> {
        wstorage::keys()
    }
}

#[doc(hidden)]
pub use whooks::Verdict as __WireVerdict;
