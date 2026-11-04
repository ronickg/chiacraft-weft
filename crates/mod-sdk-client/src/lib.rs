//! Write sandboxed chiacraft client halves in Rust (`docs/sandboxed-mods.md`, "Client halves").
//!
//! A client half is a package's `code/client.wasm`: a WebAssembly component that runs in each
//! player's own game, beside the package's server half. It adds HUD elements, screens, keys,
//! frame hooks and handlers for its server half's packets, with the names of Weft's client SDK:
//!
//! ```ignore
//! use chiacraft_mod_sdk_client::*;
//!
//! declare_client_mod!(init: init);
//!
//! static HUD: HudElement = HudElement::named("rings");
//! static RINGS: Channel<u32> = Channel::named("rings");
//!
//! fn init(r: &mut Registry) {
//!     r.hud_element(&HUD, hud_anchor::TOP_RIGHT, 0, |ui| {
//!         ui.panel("hud_badge", S::row().pad(3.0), |ui| ui.text_bind(|c, out| out.set("bell.hud", &[Arg::Int(rings())])));
//!     });
//!     r.client_packet(&RINGS, |_, n| set_rings(n));
//! }
//! ```
//!
//! Draw with the game's kit classes (`panel`, `tooltip`, `button`, `slot`, `hotbar`, `hud_badge`,
//! the `hud_health`, `hud_stamina`, `hud_xp` and `hud_temperature` bars, `tab`, `category`; text
//! classes `title`, `dim`, `button`, `chat`) and a mod looks like the game. A binding's closure
//! may only read (`ClientView`); keys, buttons, packets and frames may act (`Client`).

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        path: "../sandbox/wit/v1.2",
        world: "client-mod",
        pub_export_macro: true,
        export_macro_name: "__export",
        default_bindings_module: "chiacraft_mod_sdk_client::bindings",
    });
}

use bindings::chiacraft::mods::{client as wclient, client_registry, log as wlog, names, types, ui as wui};
#[doc(hidden)]
pub use bindings::exports::chiacraft::mods::client_hooks::Guest as __Guest;
#[doc(hidden)]
pub use types::Arg as __WireArg;
#[doc(hidden)]
pub use wui::Part as __Part;

/// Weft's alignment, justification, bar directions and part flags: one name each.
pub use weft_abi::{ui_align, ui_dir, ui_flags, ui_justify};

/// The API version this SDK builds for.
pub const API: &str = "1.2.0";

// ------------------------------------------------------------------ entry point

/// Declares the client half: `declare_client_mod!(init: init);`.
#[macro_export]
macro_rules! declare_client_mod {
    (init: $init:path $(,)?) => {
        struct __ChiacraftClientMod;
        impl $crate::__Guest for __ChiacraftClientMod {
            fn init() -> Result<(), String> {
                $crate::__init($init)
            }
            fn build(callback: u32) -> Vec<$crate::__Part> {
                $crate::__build(callback)
            }
            fn value(bind: u32) -> f32 {
                $crate::__value(bind)
            }
            fn text(bind: u32) -> (String, Vec<$crate::__WireArg>) {
                $crate::__text(bind)
            }
            fn click(bind: u32) {
                $crate::__click(bind)
            }
            fn key(callback: u32) {
                $crate::__key(callback)
            }
            fn packet(callback: u32, data: Vec<u8>) {
                $crate::__packet(callback, data)
            }
            fn frame(callback: u32, seconds: f32) {
                $crate::__frame(callback, seconds)
            }
        }
        $crate::bindings::__export!(__ChiacraftClientMod with_types_in $crate::bindings);
    };
}

type UiFn = Rc<dyn Fn(&mut Ui)>;
type ActFn = Rc<dyn Fn(&Client)>;
type PacketFn = Rc<dyn Fn(&Client, &[u8])>;
type FrameFn = Rc<dyn Fn(&Client, f32)>;
type ValueFn = Rc<dyn Fn(&ClientView) -> f32>;
type TextFn = Rc<dyn Fn(&ClientView, &mut TextOut)>;
type ClickFn = Rc<dyn Fn(&Client)>;

enum Callback {
    Ui(UiFn),
    Key(ActFn),
    Packet(PacketFn),
    Frame(FrameFn),
}

enum Bind {
    Value(ValueFn),
    Text(TextFn),
    Click(ClickFn),
}

thread_local! {
    static CALLBACKS: RefCell<Vec<Callback>> = const { RefCell::new(Vec::new()) };
    /// The binds of each build, by number, and which build made each (to drop on a rebuild).
    static BINDS: RefCell<Vec<(u32, Option<Bind>)>> = const { RefCell::new(Vec::new()) };
}

fn push(id: u32, cb: Callback) {
    CALLBACKS.with(|c| {
        let mut c = c.borrow_mut();
        if id as usize == c.len() {
            c.push(cb);
        } else {
            error!("the game numbered callback {id}, expected {}", c.len());
        }
    });
}

#[doc(hidden)]
pub fn __init(init: fn(&mut Registry)) -> Result<(), String> {
    CALLBACKS.with(|c| c.borrow_mut().clear());
    BINDS.with(|b| b.borrow_mut().clear());
    let mut r = Registry { errors: Vec::new() };
    init(&mut r);
    if r.errors.is_empty() { Ok(()) } else { Err(r.errors.join("; ")) }
}

#[doc(hidden)]
pub fn __build(callback: u32) -> Vec<__Part> {
    let f = CALLBACKS.with(|c| match c.borrow().get(callback as usize) {
        Some(Callback::Ui(f)) => Some(f.clone()),
        _ => None,
    });
    let Some(f) = f else { return Vec::new() };
    // A rebuild's binds replace the last build's.
    BINDS.with(|b| {
        for (owner, bind) in b.borrow_mut().iter_mut() {
            if *owner == callback {
                *bind = None;
            }
        }
    });
    let mut ui = Ui { parts: Vec::new(), owner: callback };
    f(&mut ui);
    ui.parts
}

fn bind<T>(id: u32, pick: impl FnOnce(&Bind) -> Option<T>) -> Option<T> {
    BINDS.with(|b| {
        b.borrow()
            .get(id as usize)
            .and_then(|(_, b)| b.as_ref())
            .and_then(pick)
    })
}

#[doc(hidden)]
pub fn __value(id: u32) -> f32 {
    match bind(id, |b| match b {
        Bind::Value(f) => Some(f.clone()),
        _ => None,
    }) {
        Some(f) => f(&ClientView { _p: PhantomData }),
        None => -1.0,
    }
}

#[doc(hidden)]
pub fn __text(id: u32) -> (String, Vec<__WireArg>) {
    let Some(f) = bind(id, |b| match b {
        Bind::Text(f) => Some(f.clone()),
        _ => None,
    }) else {
        return (String::new(), Vec::new());
    };
    let mut out = TextOut { key: String::new(), args: Vec::new() };
    f(&ClientView { _p: PhantomData }, &mut out);
    (out.key, out.args)
}

#[doc(hidden)]
pub fn __click(id: u32) {
    if let Some(f) = bind(id, |b| match b {
        Bind::Click(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&Client { _p: PhantomData });
    }
}

fn callback<T>(id: u32, pick: impl FnOnce(&Callback) -> Option<T>) -> Option<T> {
    CALLBACKS.with(|c| c.borrow().get(id as usize).and_then(pick))
}

#[doc(hidden)]
pub fn __key(id: u32) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Key(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&Client { _p: PhantomData });
    }
}

#[doc(hidden)]
pub fn __packet(id: u32, data: Vec<u8>) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Packet(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&Client { _p: PhantomData }, &data);
    }
}

#[doc(hidden)]
pub fn __frame(id: u32, seconds: f32) {
    if let Some(f) = callback(id, |c| match c {
        Callback::Frame(f) => Some(f.clone()),
        _ => None,
    }) {
        f(&Client { _p: PhantomData }, seconds);
    }
}

// ------------------------------------------------------------------ logging

#[doc(hidden)]
pub fn __log(level: u32, args: std::fmt::Arguments) {
    let level = match level {
        0 => wlog::Level::Error,
        1 => wlog::Level::Warn,
        2 => wlog::Level::Info,
        _ => wlog::Level::Debug,
    };
    wlog::write(level, &std::fmt::format(args));
}

#[macro_export]
macro_rules! error { ($($t:tt)*) => { $crate::__log(0, format_args!($($t)*)) } }
#[macro_export]
macro_rules! warn { ($($t:tt)*) => { $crate::__log(1, format_args!($($t)*)) } }
#[macro_export]
macro_rules! info { ($($t:tt)*) => { $crate::__log(2, format_args!($($t)*)) } }
#[macro_export]
macro_rules! debug { ($($t:tt)*) => { $crate::__log(3, format_args!($($t)*)) } }

// ------------------------------------------------------------------ values

/// One argument of a line players read (Weft's `Arg`).
#[derive(Clone, Copy, Debug)]
pub enum Arg<'a> {
    Text(&'a str),
    Key(&'a str),
    Int(i64),
    Float(f64, u32),
    Item(u32),
    Name(&'a str),
}

fn wire_args(args: &[Arg]) -> Vec<types::Arg> {
    args.iter()
        .map(|a| match *a {
            Arg::Text(s) => types::Arg::Text(s.into()),
            Arg::Key(s) => types::Arg::Key(s.into()),
            Arg::Int(v) => types::Arg::Int(v),
            Arg::Float(v, d) => types::Arg::Float((v, d)),
            Arg::Item(i) => types::Arg::Item(i),
            Arg::Name(s) => types::Arg::Name(s.into()),
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub item: u32,
    pub count: u32,
    pub data: u32,
}

impl ItemStack {
    pub fn is_empty(&self) -> bool {
        self.item == 0 || self.count == 0
    }
}

/// Plain data a component or packet holds (Weft's `Value`): `repr(C)`, `Copy`, no padding.
///
/// # Safety
/// Every byte pattern of its size must be a valid value, and it has no padding.
pub unsafe trait Value: Copy + 'static {}

macro_rules! primitive {
    ($($t:ty),*) => { $(unsafe impl Value for $t {})* };
}
primitive!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
unsafe impl<T: Value, const N: usize> Value for [T; N] {}

/// A component's or packet's struct, as Weft's `value!` writes it.
#[macro_export]
macro_rules! value {
    ($(#[$m:meta])* $vis:vis struct $name:ident { $($(#[$fm:meta])* $fvis:vis $field:ident : $ty:ty),* $(,)? }) => {
        $(#[$m])*
        #[repr(C)]
        #[derive(Clone, Copy, Debug, PartialEq)]
        $vis struct $name { $($(#[$fm])* $fvis $field: $ty),* }
        unsafe impl $crate::Value for $name {}
    };
}

fn value_from<T: Value>(b: &[u8]) -> Option<T> {
    // SAFETY: any bytes of the size are a valid `Value`.
    (b.len() >= std::mem::size_of::<T>()).then(|| unsafe { std::ptr::read_unaligned(b.as_ptr() as *const T) })
}

fn value_bytes<T: Value>(v: &T) -> Vec<u8> {
    // SAFETY: `Value` is plain data without padding.
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }.to_vec()
}

/// A component of the package's, read on the client when it is synced to its owner or watchers
/// (the server half declares it).
pub struct Component<T: Value> {
    name: &'static str,
    _t: PhantomData<fn() -> T>,
}

impl<T: Value> Component<T> {
    pub const fn named(name: &'static str) -> Self {
        Component { name, _t: PhantomData }
    }
}

/// What a channel carries (Weft's `Payload`): `String`, `Vec<u8>`, `()`, or any [`Value`].
pub trait Payload: Sized + 'static {
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Self>;
}

impl<T: Value> Payload for T {
    fn encode(&self) -> Vec<u8> {
        value_bytes(self)
    }
    fn decode(bytes: &[u8]) -> Option<T> {
        (bytes.len() == std::mem::size_of::<T>())
            .then(|| value_from(bytes))
            .flatten()
    }
}

impl Payload for String {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn decode(bytes: &[u8]) -> Option<String> {
        String::from_utf8(bytes.to_vec()).ok()
    }
}

impl Payload for Vec<u8> {
    fn encode(&self) -> Vec<u8> {
        self.clone()
    }
    fn decode(bytes: &[u8]) -> Option<Vec<u8>> {
        Some(bytes.to_vec())
    }
}

impl Payload for () {
    fn encode(&self) -> Vec<u8> {
        Vec::new()
    }
    fn decode(_: &[u8]) -> Option<()> {
        Some(())
    }
}

/// A channel of the package's, shared with its server half (`Channel::named("rings")` is
/// `<package>:rings`).
pub struct Channel<T: Payload> {
    name: &'static str,
    _t: PhantomData<fn() -> T>,
}

impl<T: Payload> Channel<T> {
    pub const fn named(name: &'static str) -> Self {
        Channel { name, _t: PhantomData }
    }
}

/// A HUD element of the package's.
pub struct HudElement(&'static str);
impl HudElement {
    pub const fn named(name: &'static str) -> Self {
        HudElement(name)
    }
}

/// A screen of the package's.
pub struct Screen(&'static str);
impl Screen {
    pub const fn named(name: &'static str) -> Self {
        Screen(name)
    }
}

/// A key of the package's, which the player can rebind in Controls.
pub struct Key(&'static str);
impl Key {
    pub const fn named(name: &'static str) -> Self {
        Key(name)
    }
}

/// Where a HUD element sits (Weft's `hud_anchor`).
pub mod hud_anchor {
    pub const TOP_LEFT: &str = "top_left";
    pub const TOP_RIGHT: &str = "top_right";
    pub const BOTTOM_LEFT: &str = "bottom_left";
    pub const BOTTOM_RIGHT: &str = "bottom_right";
    pub const HOTBAR: &str = "hotbar";
    pub const HOTBAR_LEFT: &str = "hotbar_left";
    pub const HOTBAR_RIGHT: &str = "hotbar_right";
    pub const ABOVE_HOTBAR: &str = "above_hotbar";
    pub const UNDER_CROSSHAIR: &str = "under_crosshair";
    pub const CENTER: &str = "center";
}

/// An item's full name, to show it.
pub fn item_id(name: &str) -> Option<u32> {
    names::item_id(name)
}

// ------------------------------------------------------------------ registering

/// What `init` registers with (Weft's `Registry`, its client half).
pub struct Registry {
    errors: Vec<String>,
}

impl Registry {
    /// A HUD element at `anchor`, `order` within it, built by `f` (Weft's `hud_element`).
    pub fn hud_element(&mut self, h: &HudElement, anchor: &str, order: i32, f: impl Fn(&mut Ui) + 'static) {
        match client_registry::hud_element(h.0, anchor, order) {
            Ok(id) => push(id, Callback::Ui(Rc::new(f))),
            Err(e) => self.errors.push(e),
        }
    }

    /// A screen built by `f` (Weft's `screen`); `.pauses()` stops a world played alone while it
    /// shows.
    pub fn screen<'a>(&'a mut self, s: &Screen, f: impl Fn(&mut Ui) + 'static) -> ScreenOptions<'a> {
        ScreenOptions { reg: self, name: s.0, pauses: false, f: Some(Rc::new(f)) }
    }

    /// A key with its default binding (`"J"`, `"ctrl+R"`): `.on_press(..)`.
    pub fn key<'a>(&'a mut self, k: &Key, default: &str) -> KeyOptions<'a> {
        KeyOptions { reg: self, name: k.0, default: default.to_string() }
    }

    /// What the server half sends on `channel` (Weft's `client_packet`).
    pub fn client_packet<T: Payload>(&mut self, channel: &Channel<T>, f: impl Fn(&Client, T) + 'static) {
        match client_registry::packet(channel.name) {
            Ok(id) => push(
                id,
                Callback::Packet(Rc::new(move |c: &Client, data: &[u8]| {
                    if let Some(v) = T::decode(data) {
                        f(c, v)
                    }
                })),
            ),
            Err(e) => self.errors.push(e),
        }
    }

    /// Every frame, with the seconds since the last (Weft's `client_frame`).
    pub fn client_frame(&mut self, f: impl Fn(&Client, f32) + 'static) {
        let id = client_registry::frame();
        push(id, Callback::Frame(Rc::new(f)));
    }
}

/// A screen being registered: registered when this is dropped.
pub struct ScreenOptions<'a> {
    reg: &'a mut Registry,
    name: &'static str,
    pauses: bool,
    f: Option<UiFn>,
}

impl ScreenOptions<'_> {
    pub fn pauses(mut self) -> Self {
        self.pauses = true;
        self
    }
}

impl Drop for ScreenOptions<'_> {
    fn drop(&mut self) {
        let Some(f) = self.f.take() else { return };
        match client_registry::screen(self.name, self.pauses) {
            Ok(id) => push(id, Callback::Ui(f)),
            Err(e) => self.reg.errors.push(e),
        }
    }
}

/// A key being registered.
pub struct KeyOptions<'a> {
    reg: &'a mut Registry,
    name: &'static str,
    default: String,
}

impl KeyOptions<'_> {
    /// On each press, while the world has the input.
    pub fn on_press(self, f: impl Fn(&Client) + 'static) {
        match client_registry::key(self.name, &self.default) {
            Ok(id) => push(id, Callback::Key(Rc::new(f))),
            Err(e) => self.reg.errors.push(e),
        }
    }
}

// ------------------------------------------------------------------ the player's game

/// What the player's game shows them (Weft's `ClientView`): reading only.
pub struct ClientView {
    _p: PhantomData<*const ()>,
}

impl ClientView {
    /// Feet position, health, max health.
    pub fn me(&self) -> Option<([f64; 3], f32, f32)> {
        wclient::me().map(|((x, y, z), h, m)| ([x, y, z], h, m))
    }
    pub fn day_time(&self) -> u64 {
        wclient::day_time()
    }
    pub fn raining(&self) -> bool {
        wclient::raining()
    }
    pub fn creative(&self) -> bool {
        wclient::creative()
    }
    /// The player's own value of one of the package's synced components.
    pub fn component<T: Value>(&self, c: &Component<T>) -> Option<T> {
        wclient::component(c.name).and_then(|b| value_from(&b))
    }
    pub fn inventory_slot(&self, slot: u32) -> ItemStack {
        let s = wclient::inventory_slot(slot);
        ItemStack { item: s.item, count: s.count, data: s.data }
    }
    /// The screen showing, empty for none.
    pub fn screen(&self) -> String {
        wclient::screen()
    }
}

/// The player's game, for a key, button, packet or frame (Weft's `Client`): reading and acting.
pub struct Client {
    _p: PhantomData<*const ()>,
}

impl std::ops::Deref for Client {
    type Target = ClientView;
    fn deref(&self) -> &ClientView {
        &ClientView { _p: PhantomData }
    }
}

impl Client {
    /// To the server half on `channel` (Weft's `send`).
    pub fn send<T: Payload>(&self, channel: &Channel<T>, value: &T) {
        wclient::send(channel.name, &value.encode());
    }
    pub fn open_screen(&self, s: &Screen) {
        wclient::open_screen(s.0);
    }
    pub fn close_screen(&self) {
        wclient::close_screen();
    }
    /// A line in the player's chat, in their language.
    pub fn show(&self, key: &str, args: &[Arg]) {
        wclient::show(key, &wire_args(args));
    }
}

// ------------------------------------------------------------------ building parts

/// How a part is laid out (Weft's `S`), sizes in GUI pixels.
#[derive(Clone, Debug)]
pub struct S {
    row: bool,
    width: Option<f32>,
    height: Option<f32>,
    gap: f32,
    pad: f32,
    grow: f32,
    align: u32,
    justify: u32,
    at: Option<[f32; 4]>,
    scale: f32,
    flags: u32,
    class: String,
}

impl Default for S {
    fn default() -> Self {
        S {
            row: false,
            width: None,
            height: None,
            gap: 0.0,
            pad: 0.0,
            grow: 0.0,
            align: ui_align::START,
            justify: ui_justify::START,
            at: None,
            scale: 1.0,
            flags: 0,
            class: String::new(),
        }
    }
}

impl S {
    pub fn row() -> S {
        S { row: true, align: ui_align::CENTER, ..S::default() }
    }
    pub fn col() -> S {
        S::default()
    }
    pub fn class(mut self, class: &str) -> S {
        self.class = class.to_string();
        self
    }
    pub fn size(mut self, w: f32, h: f32) -> S {
        self.width = Some(w);
        self.height = Some(h);
        self
    }
    pub fn w(mut self, w: f32) -> S {
        self.width = Some(w);
        self
    }
    pub fn h(mut self, h: f32) -> S {
        self.height = Some(h);
        self
    }
    pub fn gap(mut self, g: f32) -> S {
        self.gap = g;
        self
    }
    pub fn pad(mut self, p: f32) -> S {
        self.pad = p;
        self
    }
    pub fn grow(mut self) -> S {
        self.grow = 1.0;
        self
    }
    pub fn center(mut self) -> S {
        self.align = ui_align::CENTER;
        self.justify = ui_justify::CENTER;
        self
    }
    pub fn stretch(mut self) -> S {
        self.align = ui_align::STRETCH;
        self
    }
    pub fn end(mut self) -> S {
        self.justify = ui_justify::END;
        self
    }
    pub fn scale(mut self, s: f32) -> S {
        self.scale = s;
        self
    }
    pub fn no_wrap(mut self) -> S {
        self.flags |= ui_flags::NO_WRAP;
        self
    }
    pub fn mono(mut self) -> S {
        self.flags |= ui_flags::MONO;
        self
    }
    /// Placed at (x, y) of its parent, over its other parts (Weft's `S::at`).
    pub fn at(self, x: f32, y: f32) -> S {
        self.at_pivot(x, y, [0.0, 0.0])
    }
    /// `at`, with this point of the part on (x, y) ([0.5, 0.5]: its middle).
    pub fn at_pivot(mut self, x: f32, y: f32, pivot: [f32; 2]) -> S {
        self.at = Some([x, y, pivot[0], pivot[1]]);
        self
    }

    fn wire(&self) -> wui::Style {
        wui::Style {
            row: self.row,
            width: self.width,
            height: self.height,
            gap: self.gap,
            pad: self.pad,
            grow: self.grow,
            align: self.align,
            justify: self.justify,
            at: self.at.map(|a| (a[0], a[1], a[2], a[3])),
            scale: self.scale,
            flags: self.flags,
            class: self.class.clone(),
        }
    }
}

/// A text binding's output (Weft's `TextOut`).
pub struct TextOut {
    key: String,
    args: Vec<types::Arg>,
}

impl TextOut {
    pub fn set(&mut self, key: &str, args: &[Arg]) {
        self.key = key.to_string();
        self.args = wire_args(args);
    }
}

/// A HUD element's or screen's parts being built (Weft's `Ui`).
pub struct Ui {
    parts: Vec<__Part>,
    /// The build these parts belong to, which owns their binds.
    owner: u32,
}

impl Ui {
    fn part(&mut self, kind: wui::PartKind, s: &S) -> usize {
        self.parts.push(wui::Part {
            kind,
            style: s.wire(),
            class: String::new(),
            text: String::new(),
            args: Vec::new(),
            children: 0,
            bind: 0,
            count: 0,
            item: 0,
            size: (0.0, 0.0),
        });
        self.parts.len() - 1
    }

    fn bind(&self, b: Bind) -> u32 {
        BINDS.with(|binds| {
            let mut binds = binds.borrow_mut();
            // A slot a rebuild freed is used again, so the list doesn't grow.
            if let Some(i) = binds.iter().position(|(_, b)| b.is_none()) {
                binds[i] = (self.owner, Some(b));
                return i as u32;
            }
            binds.push((self.owner, Some(b)));
            binds.len() as u32 - 1
        })
    }

    fn children(&mut self, at: usize, f: impl FnOnce(&mut Ui)) {
        let before = self.parts.len();
        f(self);
        // Direct children only: walk the subtrees added.
        let mut n = 0;
        let mut i = before;
        while i < self.parts.len() {
            n += 1;
            i = skip(&self.parts, i);
        }
        self.parts[at].children = n;
    }

    pub fn column(&mut self, s: S, f: impl FnOnce(&mut Ui)) {
        let at = self.part(wui::PartKind::Container, &S { row: false, ..s });
        self.children(at, f);
    }
    pub fn row(&mut self, s: S, f: impl FnOnce(&mut Ui)) {
        let at = self.part(wui::PartKind::Container, &S { row: true, ..s });
        self.children(at, f);
    }
    /// A container drawn with the kit's `class` (`panel`, `tooltip`, `hud_badge`, `slot`).
    pub fn panel(&mut self, class: &str, s: S, f: impl FnOnce(&mut Ui)) {
        let at = self.part(wui::PartKind::Panel, &s);
        self.parts[at].class = class.to_string();
        self.children(at, f);
    }
    pub fn text(&mut self, key: &str) {
        self.text_styled(S::default(), key, &[]);
    }
    pub fn text_args(&mut self, key: &str, args: &[Arg]) {
        self.text_styled(S::default(), key, args);
    }
    pub fn text_styled(&mut self, s: S, key: &str, args: &[Arg]) {
        let at = self.part(wui::PartKind::Text, &s);
        self.parts[at].text = key.to_string();
        self.parts[at].args = wire_args(args);
    }
    /// Text worded every frame (Weft's `text_bind`).
    pub fn text_bind(&mut self, f: impl Fn(&ClientView, &mut TextOut) + 'static) {
        self.text_bind_styled(S::default(), f);
    }
    pub fn text_bind_styled(&mut self, s: S, f: impl Fn(&ClientView, &mut TextOut) + 'static) {
        let at = self.part(wui::PartKind::TextBind, &s);
        self.parts[at].bind = self.bind(Bind::Text(Rc::new(f)));
    }
    /// A bar of the kit's `class` (`hud_xp`, `hud_health`, ...), 0..1, hidden below 0.
    pub fn progress_bind(&mut self, class: &str, s: S, f: impl Fn(&ClientView) -> f32 + 'static) {
        self.progress_bind_dir(class, s, ui_dir::RIGHT, f);
    }
    pub fn progress_bind_dir(&mut self, class: &str, s: S, dir: u32, f: impl Fn(&ClientView) -> f32 + 'static) {
        let at = self.part(wui::PartKind::Progress, &s);
        self.parts[at].class = class.to_string();
        self.parts[at].count = dir;
        self.parts[at].bind = self.bind(Bind::Value(Rc::new(f)));
    }
    /// `count` icons of `class`, as many lit as `f` says.
    pub fn icons(&mut self, class: &str, count: u32, f: impl Fn(&ClientView) -> f32 + 'static) {
        let at = self.part(wui::PartKind::Icons, &S::default());
        self.parts[at].class = class.to_string();
        self.parts[at].count = count;
        self.parts[at].bind = self.bind(Bind::Value(Rc::new(f)));
    }
    /// A texture from `gui/` (`"logo"` the package's, `"base:logo"` another's).
    pub fn image(&mut self, texture: &str, w: f32, h: f32) {
        let at = self.part(wui::PartKind::Image, &S::default());
        self.parts[at].class = texture.to_string();
        self.parts[at].size = (w, h);
    }
    pub fn item(&mut self, item: u32, count: u32, s: S) {
        let at = self.part(wui::PartKind::Item, &s);
        self.parts[at].item = item;
        self.parts[at].count = count;
    }
    pub fn spacer(&mut self) {
        self.part(wui::PartKind::Spacer, &S::default());
    }
    pub fn button(&mut self, key: &str, s: S, f: impl Fn(&Client) + 'static) {
        self.button_args(key, &[], s, f);
    }
    pub fn button_args(&mut self, key: &str, args: &[Arg], s: S, f: impl Fn(&Client) + 'static) {
        let at = self.part(wui::PartKind::Button, &s);
        self.parts[at].text = key.to_string();
        self.parts[at].args = wire_args(args);
        self.parts[at].bind = self.bind(Bind::Click(Rc::new(f)));
    }
}

/// The index after the subtree that starts at `i`.
fn skip(parts: &[__Part], i: usize) -> usize {
    let mut next = i + 1;
    for _ in 0..parts[i].children {
        next = skip(parts, next);
    }
    next
}
