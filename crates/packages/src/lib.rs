//! Data-only content packages (chiacraft plan, phase 2): blocks, items, recipes and tags written
//! as files, with their art, and no code.
//!
//! A package is a folder:
//! - `package.toml`: the manifest ([`Manifest`]);
//! - `defs/blocks/<path>.toml`, `defs/items/<path>.toml`, `defs/recipes/<path>.toml`,
//!   `defs/tags/<path>.toml`: one definition per file, named by its path: `defs/blocks/ruby_block.toml`
//!   is the block `<package id>:ruby_block`, `defs/tags/ores/ruby.toml` the tag `ores/ruby`;
//! - `assets/`, laid out like a mod's: `textures/block`, `textures/item`, `models/block` (Blockbench
//!   Java models, picked up by the block's name), `lang/en_us.lang`, sounds.
//!
//! Inside a package a name without a namespace is the package's own (`ruby` is `<id>:ruby`); tags
//! (`#ores/ruby`) are shared vocabulary and stay as written. This crate reads and checks packages;
//! the game registers them through the `packages` mod.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The definition format this game reads: packages of the same major version load, an older
/// minor version is fine, a newer one may use fields this game does not know yet.
pub const FORMAT: (u32, u32) = (1, 0);

/// At most this many definition files, and this big each: a package is data someone else wrote.
const MAX_DEFS: usize = 4096;
const MAX_DEF_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// `"<major>.<minor>"` of the definition format the package is written in.
    pub format: String,
    /// The package's namespace: lowercase letters, digits and `_`, starting with a letter.
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Mods and packages it needs, by id, with the versions it works with (`"*"` for any).
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

/// A definition and the id its file gives it.
#[derive(Debug, Clone)]
pub struct Def<T> {
    /// `<package id>:<path>` for blocks, items and recipes; `<path>` for tags.
    pub id: String,
    pub def: T,
}

#[derive(Debug, Clone)]
pub struct Package {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub blocks: Vec<Def<BlockDef>>,
    pub items: Vec<Def<ItemDef>>,
    pub recipes: Vec<Def<RecipeDef>>,
    pub tags: Vec<Def<TagDef>>,
}

impl Package {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn assets(&self) -> PathBuf {
        self.dir.join("assets")
    }
}

/// How a block is drawn: solid cubes hide their neighbours' faces, cutout ones have holes
/// (leaves, glass), translucent ones are see-through (stained glass, ice).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Render {
    #[default]
    Solid,
    Cutout,
    Translucent,
}

/// Per-face textures; `side` stands for the four sides not named.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faces {
    pub top: Option<String>,
    pub bottom: Option<String>,
    pub side: Option<String>,
    pub north: Option<String>,
    pub south: Option<String>,
    pub east: Option<String>,
    pub west: Option<String>,
}

/// What breaking the block gives: `"self"` (the default), `"nothing"`, an item name, or
/// `{ item = "...", count = 2 }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Drops {
    Name(String),
    Item {
        item: String,
        #[serde(default = "one")]
        count: u32,
    },
}

fn one() -> u32 {
    1
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockDef {
    /// One texture for every face; defaults to `block/<path>` of the package.
    pub texture: Option<String>,
    pub textures: Option<Faces>,
    /// Seconds to break by hand, roughly (stone 1.5, dirt 0.5).
    pub hardness: Option<f32>,
    #[serde(default)]
    pub unbreakable: bool,
    /// Light it gives, 0 to 15.
    pub light: Option<u8>,
    /// Step, dig and place sounds: a sound group such as `stone`, `wood`, `grass`, `glass`.
    pub sound: Option<String>,
    /// The tool class that mines it fast (`pickaxe`, `axe`, `shovel`).
    pub mineable: Option<String>,
    /// The tool level needed for it to drop anything.
    pub tool_level: Option<u32>,
    pub drops: Option<Drops>,
    #[serde(default)]
    pub render: Render,
    /// False for blocks players walk through (plants, decorations).
    #[serde(default = "yes")]
    pub solid: bool,
    #[serde(default)]
    pub climbable: bool,
    pub friction: Option<f32>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether the block has an item that places it (the default).
    #[serde(default = "yes")]
    pub item: bool,
    pub item_texture: Option<String>,
    /// Creative inventory tab, e.g. `base:building`.
    pub tab: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDef {
    /// Defaults to `item/<path>` of the package.
    pub texture: Option<String>,
    pub max_stack: Option<u32>,
    pub durability: Option<u32>,
    /// A tool of this class (`pickaxe`, `axe`, `shovel`, `sword`).
    pub tool: Option<String>,
    pub tool_level: Option<u32>,
    pub mining_speed: Option<f32>,
    pub attack_damage: Option<f32>,
    /// Burns in a furnace for this many ticks.
    pub fuel: Option<u32>,
    /// The block it places.
    pub places: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub tab: Option<String>,
}

/// A shaped recipe (`pattern` and `key`) or a shapeless or processing one (`inputs`, and `time`
/// in ticks for a furnace).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeDef {
    /// The recipe kind: `base:crafting` (the default) or `base:smelting`.
    #[serde(default = "crafting")]
    pub kind: String,
    #[serde(default)]
    pub pattern: Vec<String>,
    #[serde(default)]
    pub key: BTreeMap<String, String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub time: u32,
    pub result: String,
    #[serde(default = "one")]
    pub count: u32,
}

fn crafting() -> String {
    "base:crafting".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagDef {
    /// Blocks and items (or `#other_tags`) that get the tag.
    pub entries: Vec<String>,
}

/// A name as written in package `ns`: one without a namespace is the package's; tags and empty
/// names stay as written, and so does the `fluid:` prefix of a liquid ingredient.
pub fn qualify(ns: &str, name: &str) -> String {
    if let Some(f) = name.strip_prefix("fluid:") {
        return format!("fluid:{}", qualify(ns, f));
    }
    let base = name.split('[').next().unwrap_or(name);
    if name.starts_with('#') || base.contains(':') || name.is_empty() { name.to_string() } else { format!("{ns}:{name}") }
}

fn valid_id(id: &str) -> bool {
    id.len() <= 64 && id.starts_with(|c: char| c.is_ascii_lowercase()) && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn valid_path(path: &str) -> bool {
    !path.is_empty() && path.split('/').all(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))
}

fn parse_format(text: &str) -> Option<(u32, u32)> {
    let (a, b) = text.split_once('.')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Reads and checks `<dir>/package.toml`.
pub fn read_manifest(dir: &Path) -> Result<Manifest, String> {
    let text = std::fs::read_to_string(dir.join("package.toml")).map_err(|e| format!("package.toml: {e}"))?;
    let m: Manifest = toml::from_str(&text).map_err(|e| format!("package.toml: {e}"))?;
    if !valid_id(&m.id) {
        return Err(format!("package.toml: id `{}`: use lowercase letters, digits and `_`, starting with a letter", m.id));
    }
    if m.id == "core" {
        return Err("package.toml: id `core` belongs to the engine".into());
    }
    if m.version.is_empty() || !m.version.chars().all(|c| c.is_ascii_alphanumeric() || ".+-".contains(c)) {
        return Err(format!("package.toml: version `{}`: use something like 1.0.0", m.version));
    }
    match parse_format(&m.format) {
        Some((major, _)) if major == FORMAT.0 => {}
        Some((major, minor)) => {
            return Err(format!("package.toml: written for format {major}.{minor}; this game reads format {}.x", FORMAT.0));
        }
        None => return Err(format!("package.toml: format `{}`: write it as \"{}.{}\"", m.format, FORMAT.0, FORMAT.1)),
    }
    Ok(m)
}

/// Package folders (those with a `package.toml`) in `dirs`, earlier dirs winning for a folder name
/// found twice; sorted by folder name.
pub fn discover(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.join("package.toml").is_file()).collect();
        entries.sort();
        for p in entries {
            if !found.iter().any(|f| f.file_name() == p.file_name()) {
                found.push(p);
            }
        }
    }
    found.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    found
}

/// Reads a whole package, checking everything and reporting every mistake at once (each with
/// its file, and the line where the file says).
pub fn load(dir: &Path) -> Result<Package, Vec<String>> {
    let manifest = read_manifest(dir).map_err(|e| vec![e])?;
    let ns = manifest.id.clone();
    let mut errors = Vec::new();
    let mut count = 0;
    let mut pkg = Package { dir: dir.to_path_buf(), manifest, blocks: Vec::new(), items: Vec::new(), recipes: Vec::new(), tags: Vec::new() };
    read_kind(dir, "blocks", &mut count, &mut errors, |path, d: BlockDef| pkg.blocks.push(Def { id: format!("{ns}:{path}"), def: qualify_block(&ns, d) }));
    read_kind(dir, "items", &mut count, &mut errors, |path, d: ItemDef| pkg.items.push(Def { id: format!("{ns}:{path}"), def: qualify_item(&ns, d) }));
    read_kind(dir, "recipes", &mut count, &mut errors, |path, d: RecipeDef| pkg.recipes.push(Def { id: format!("{ns}:{path}"), def: qualify_recipe(&ns, d) }));
    read_kind(dir, "tags", &mut count, &mut errors, |path, d: TagDef| pkg.tags.push(Def { id: path.to_string(), def: TagDef { entries: d.entries.iter().map(|e| qualify(&ns, e)).collect() } }));
    if let Ok(entries) = std::fs::read_dir(dir.join("defs")) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !["blocks", "items", "recipes", "tags"].contains(&name.as_str()) {
                errors.push(format!("defs/{name}: not a kind of definition this game reads (blocks, items, recipes, tags)"));
            }
        }
    }
    check(&pkg, &mut errors);
    if errors.is_empty() { Ok(pkg) } else { Err(errors) }
}

/// Every `defs/<kind>/**.toml`, parsed into `T` and handed to `add` with its path.
fn read_kind<T: serde::de::DeserializeOwned>(dir: &Path, kind: &str, count: &mut usize, errors: &mut Vec<String>, mut add: impl FnMut(&str, T)) {
    let root = dir.join("defs").join(kind);
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    for file in files {
        let rel = file.strip_prefix(&root).unwrap_or(&file).to_string_lossy().replace('\\', "/");
        let shown = format!("defs/{kind}/{rel}");
        let Some(path) = rel.strip_suffix(".toml") else {
            errors.push(format!("{shown}: definitions are .toml files"));
            continue;
        };
        if !valid_path(path) {
            errors.push(format!("{shown}: name files and folders with lowercase letters, digits and `_`"));
            continue;
        }
        *count += 1;
        if *count > MAX_DEFS {
            errors.push(format!("more than {MAX_DEFS} definitions"));
            return;
        }
        if std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0) > MAX_DEF_BYTES {
            errors.push(format!("{shown}: bigger than {} KiB", MAX_DEF_BYTES / 1024));
            continue;
        }
        match std::fs::read_to_string(&file).map_err(|e| e.to_string()).and_then(|t| toml::from_str::<T>(&t).map_err(|e| e.to_string())) {
            Ok(d) => add(path, d),
            Err(e) => errors.push(format!("{shown}: {}", e.trim_end())),
        }
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        // Links could lead out of the package.
        let Ok(t) = e.file_type() else { continue };
        if t.is_dir() {
            walk(&p, out);
        } else if t.is_file() {
            out.push(p);
        }
    }
}

fn q(ns: &str, s: Option<String>) -> Option<String> {
    s.map(|s| qualify(ns, &s))
}

fn qualify_block(ns: &str, mut d: BlockDef) -> BlockDef {
    d.texture = q(ns, d.texture);
    d.item_texture = q(ns, d.item_texture);
    if let Some(f) = &mut d.textures {
        for t in [&mut f.top, &mut f.bottom, &mut f.side, &mut f.north, &mut f.south, &mut f.east, &mut f.west] {
            *t = q(ns, t.take());
        }
    }
    d.drops = d.drops.map(|dr| match dr {
        Drops::Name(n) if n == "self" || n == "nothing" => Drops::Name(n),
        Drops::Name(n) => Drops::Item { item: qualify(ns, &n), count: 1 },
        Drops::Item { item, count } => Drops::Item { item: qualify(ns, &item), count },
    });
    d
}

fn qualify_item(ns: &str, mut d: ItemDef) -> ItemDef {
    d.texture = q(ns, d.texture);
    d.places = q(ns, d.places);
    d
}

fn qualify_recipe(ns: &str, mut d: RecipeDef) -> RecipeDef {
    d.kind = qualify(ns, &d.kind);
    d.result = qualify(ns, &d.result);
    d.inputs = d.inputs.iter().map(|i| qualify(ns, i)).collect();
    d.key = d.key.into_iter().map(|(k, v)| (k, qualify(ns, &v))).collect();
    d
}

/// Mistakes a file's own parsing cannot see.
fn check(p: &Package, errors: &mut Vec<String>) {
    for b in &p.blocks {
        let at = format!("defs/blocks/{}.toml", b.id.split_once(':').map_or("", |x| x.1));
        if b.def.light.is_some_and(|l| l > 15) {
            errors.push(format!("{at}: light goes from 0 to 15"));
        }
        if b.def.texture.is_some() && b.def.textures.is_some() {
            errors.push(format!("{at}: `texture` and `textures` both set; use one"));
        }
        if let Some(Drops::Item { count: 0, .. }) = b.def.drops {
            errors.push(format!("{at}: drops count 0; write drops = \"nothing\""));
        }
    }
    for i in &p.items {
        if i.def.max_stack == Some(0) {
            errors.push(format!("defs/items/{}.toml: max_stack 0", i.id.split_once(':').map_or("", |x| x.1)));
        }
    }
    for r in &p.recipes {
        let at = format!("defs/recipes/{}.toml", r.id.split_once(':').map_or("", |x| x.1));
        let d = &r.def;
        match (d.pattern.is_empty(), d.inputs.is_empty()) {
            (false, false) => errors.push(format!("{at}: a recipe has a `pattern` (shaped) or `inputs` (shapeless), not both")),
            (true, true) => errors.push(format!("{at}: give a `pattern` and `key` (shaped) or `inputs` (shapeless)")),
            (false, true) => {
                for k in d.key.keys() {
                    if k.chars().count() != 1 {
                        errors.push(format!("{at}: key `{k}`: keys are single characters"));
                    }
                }
                for c in d.pattern.iter().flat_map(|row| row.chars()).filter(|c| *c != ' ') {
                    if !d.key.contains_key(&c.to_string()) {
                        errors.push(format!("{at}: `{c}` in the pattern has no key"));
                        break;
                    }
                }
            }
            (true, false) => {
                if !d.key.is_empty() {
                    errors.push(format!("{at}: `key` belongs to a shaped recipe (with a `pattern`)"));
                }
            }
        }
        if d.count == 0 {
            errors.push(format!("{at}: count 0"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, path: &str, text: &str) {
        let p = dir.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("chiacraft-packages-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// Ids come from file paths, names without a namespace are the package's, tags stay shared.
    #[test]
    fn reads_a_package() {
        let d = temp("ok");
        write(&d, "package.toml", "format = \"1.0\"\nid = \"gems\"\nversion = \"0.1.0\"\n[dependencies]\nbase = \"*\"\n");
        write(&d, "defs/blocks/ruby_block.toml", "hardness = 3.0\nsound = \"stone\"\nmineable = \"pickaxe\"\ntags = [\"ore_blocks/ruby\"]\n");
        write(&d, "defs/items/ruby.toml", "max_stack = 64\n");
        write(&d, "defs/recipes/ruby_block.toml", "pattern = [\"RRR\", \"RRR\", \"RRR\"]\nkey = { R = \"ruby\" }\nresult = \"ruby_block\"\n");
        write(&d, "defs/tags/gems/red.toml", "entries = [\"ruby\", \"#ores/ruby\", \"base:redstone\"]\n");
        let p = load(&d).unwrap();
        assert_eq!(p.id(), "gems");
        assert_eq!(p.blocks[0].id, "gems:ruby_block");
        assert_eq!(p.recipes[0].def.kind, "base:crafting");
        assert_eq!(p.recipes[0].def.key["R"], "gems:ruby");
        assert_eq!(p.recipes[0].def.result, "gems:ruby_block");
        assert_eq!(p.tags[0].id, "gems/red");
        assert_eq!(p.tags[0].def.entries, ["gems:ruby", "#ores/ruby", "base:redstone"]);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Every mistake is reported at once, each with its file.
    #[test]
    fn reports_every_mistake() {
        let d = temp("bad");
        write(&d, "package.toml", "format = \"1.0\"\nid = \"gems\"\nversion = \"0.1.0\"\n");
        write(&d, "defs/blocks/a.toml", "hardnes = 3.0\n");
        write(&d, "defs/blocks/b.toml", "light = 20\n");
        write(&d, "defs/blocks/Bad Name.toml", "\n");
        write(&d, "defs/recipes/r.toml", "pattern = [\"X\"]\nresult = \"a\"\n");
        write(&d, "defs/sounds/s.toml", "\n");
        let errors = load(&d).unwrap_err();
        let all = errors.join("\n");
        assert!(all.contains("defs/blocks/a.toml") && all.contains("hardnes"), "{all}");
        assert!(all.contains("defs/blocks/b.toml: light"), "{all}");
        assert!(all.contains("Bad Name"), "{all}");
        assert!(all.contains("`X` in the pattern has no key"), "{all}");
        assert!(all.contains("defs/sounds"), "{all}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn checks_the_manifest() {
        let d = temp("manifest");
        write(&d, "package.toml", "format = \"2.0\"\nid = \"gems\"\nversion = \"1\"\n");
        assert!(read_manifest(&d).unwrap_err().contains("format 2.0"));
        write(&d, "package.toml", "format = \"1.0\"\nid = \"Gems\"\nversion = \"1\"\n");
        assert!(read_manifest(&d).unwrap_err().contains("id `Gems`"));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn qualifies_names() {
        assert_eq!(qualify("p", "ruby"), "p:ruby");
        assert_eq!(qualify("p", "base:stone"), "base:stone");
        assert_eq!(qualify("p", "#ores"), "#ores");
        assert_eq!(qualify("p", "fluid:oil"), "fluid:p:oil");
        assert_eq!(qualify("p", "lamp[lit=true]"), "p:lamp[lit=true]");
    }
}
