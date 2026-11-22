use weft_sdk::Chain;

/// A furnace's cooking time: base is the recipe's time, `entity` the furnace, `item` the input.
pub static SMELT_TIME: Chain = Chain::named("base:smelt_time");

/// The share of a piece of gear's power that counts against a foe (base 1): `entity` wears or
/// holds it, `other` is the foe, `amount` the gear's item level (`ITEM_LEVEL`). A big creature
/// lets gear count at most its size's cap in item levels over its own area level
/// (`vitals::combat::cap_share`); each creature's mod answers for its own.
pub static GEAR_COUNTS: Chain = Chain::named("base:gear_counts");
