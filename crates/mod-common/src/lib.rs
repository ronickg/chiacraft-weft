//! What the server SDK (`chiacraft-mod-sdk`) and the client SDK (`chiacraft-mod-sdk-client`)
//! share. Each SDK generates its own bindings for its own WIT world, so what touches the wire
//! (logging, line arguments) is written once here as `__glue!` and expanded in each over its
//! bindings; the rest is plain Rust, re-exported by both.

/// One argument of a line players read (Weft's `Arg`).
#[derive(Clone, Copy, Debug)]
pub enum Arg<'a> {
    /// Shown as it is.
    Text(&'a str),
    /// A translation key, worded too.
    Key(&'a str),
    Int(i64),
    /// A number with this many decimals.
    Float(f64, u32),
    Item(u32),
    /// A display name, as `{name:..}` writes it.
    Name(&'a str),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemStack {
    pub item: u32,
    pub count: u32,
    pub data: u32,
}

impl ItemStack {
    pub const EMPTY: ItemStack = ItemStack { item: 0, count: 0, data: 0 };

    pub fn new(item: u32, count: u32) -> ItemStack {
        ItemStack { item, count, data: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.item == 0 || self.count == 0
    }
}

/// Plain data a component or packet holds (Weft's `Value`): `repr(C)`, `Copy`, no pointers.
/// Primitives and arrays of them are; structs with [`value!`].
///
/// # Safety
/// Every byte pattern of its size must be a valid value (no `bool`, `char` or enums inside), and
/// it has no padding the game might keep.
pub unsafe trait Value: Copy + 'static {}

macro_rules! primitive {
    ($($t:ty),*) => { $(unsafe impl Value for $t {})* };
}
primitive!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64);
unsafe impl<T: Value, const N: usize> Value for [T; N] {}

/// A component's or packet's struct, as Weft's `value!` writes it: `repr(C)`, `Copy`.
///
/// ```ignore
/// value! {
///     #[derive(Default)]
///     pub struct Tribe { pub id: [u8; 16], pub name: [u8; 24] }
/// }
/// ```
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

#[doc(hidden)]
pub fn value_bytes<T: Value>(v: &T) -> Vec<u8> {
    // SAFETY: `Value` is plain data without padding.
    unsafe { std::slice::from_raw_parts(v as *const T as *const u8, std::mem::size_of::<T>()) }.to_vec()
}

#[doc(hidden)]
pub fn value_from<T: Value>(b: &[u8]) -> Option<T> {
    // SAFETY: any bytes of the size are a valid `Value`.
    (b.len() >= std::mem::size_of::<T>()).then(|| unsafe { std::ptr::read_unaligned(b.as_ptr() as *const T) })
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

/// Keeps a callback under the number the game gave it: the game numbers registrations in order,
/// and so does the SDK. `Err` with the number expected if they disagree (the callback is
/// dropped).
#[doc(hidden)]
pub fn number<T>(list: &mut Vec<T>, id: u32, callback: T) -> Result<(), usize> {
    if id as usize == list.len() {
        list.push(callback);
        Ok(())
    } else {
        Err(list.len())
    }
}

/// An SDK's `__log` (behind its `error!` to `debug!`) and `wire_args` over its own bindings:
/// `__glue!(crate::bindings::..::log, crate::bindings::..::types)`.
#[doc(hidden)]
#[macro_export]
macro_rules! __glue {
    ($log:path, $types:path) => {
        #[doc(hidden)]
        pub fn __log(level: u32, args: ::std::fmt::Arguments) {
            use $log as wlog;
            let level = match level {
                0 => wlog::Level::Error,
                1 => wlog::Level::Warn,
                2 => wlog::Level::Info,
                _ => wlog::Level::Debug,
            };
            wlog::write(level, &::std::fmt::format(args));
        }

        fn wire_args(args: &[$crate::Arg]) -> ::std::vec::Vec<__glue_types::Arg> {
            use $crate::Arg;
            args.iter()
                .map(|a| match *a {
                    Arg::Text(s) => __glue_types::Arg::Text(s.into()),
                    Arg::Key(s) => __glue_types::Arg::Key(s.into()),
                    Arg::Int(v) => __glue_types::Arg::Int(v),
                    Arg::Float(v, d) => __glue_types::Arg::Float((v, d)),
                    Arg::Item(i) => __glue_types::Arg::Item(i),
                    Arg::Name(s) => __glue_types::Arg::Name(s.into()),
                })
                .collect()
        }

        use $types as __glue_types;
    };
}
