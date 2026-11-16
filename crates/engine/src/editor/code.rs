//! What the editor makes and the code for it, both ways. A thing is a handle and the builder
//! calls made on it, each argument kept as the code writes it; which calls there are comes from
//! the SDK itself (`METHODS`, listed by the build script), so nothing here knows any one method.

use crate::ui::text::Span;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum What {
    Block,
    Item,
}

impl What {
    /// Its bit in `Method::on`.
    fn bit(self) -> u8 {
        match self {
            What::Block => weft_abi::editor::BLOCK,
            What::Item => weft_abi::editor::ITEM,
        }
    }

    /// The folder its pictures are under.
    pub fn folder(self) -> &'static str {
        match self {
            What::Block => "block",
            What::Item => "item",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Call {
    pub method: String,
    /// Each argument as code (`"base:stone"`, `1.5`, `[0.0, 0.0, 0.0]`).
    pub args: Vec<String>,
}

impl Call {
    pub fn new(method: &str, args: &[&str]) -> Call {
        Call { method: method.into(), args: args.iter().map(|a| a.to_string()).collect() }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Thing {
    pub kind: What,
    pub name: String,
    pub calls: Vec<Call>,
}

impl Thing {
    /// A new one from the editor's buttons: something to start from.
    pub fn new(kind: What, name: &str) -> Thing {
        let calls = match kind {
            What::Block => vec![
                Call::new("texture", &["\"base:stone\""]),
                Call::new("sound_group", &["\"base:block.stone\""]),
                Call::new("hardness", &["1.5"]),
                Call::new("mined_by", &["\"pickaxe\"", "abi::tool_tier::WOOD"]),
            ],
            What::Item => vec![Call::new("texture", &["\"base:diamond\""])],
        };
        Thing { kind, name: name.into(), calls }
    }

    /// The arguments of its calls to `method`, in order.
    pub fn args_of<'a>(&'a self, method: &'a str) -> impl Iterator<Item = &'a [String]> + 'a {
        self.calls
            .iter()
            .filter(move |c| c.method == method)
            .map(|c| c.args.as_slice())
    }

    /// The picture its `texture` call names, as the content lists it (`base:block/stone`).
    pub fn picture(&self) -> Option<String> {
        let t = unquote(self.args_of("texture").next()?.first()?)?;
        picture_name(&t, self.kind.folder())
    }
}

/// The tool tiers `mined_by` names, as `abi::tool_tier::<name>`.
pub const TIERS: [&str; 5] = ["HAND", "WOOD", "STONE", "IRON", "DIAMOND"];

/// A builder's picture argument (`base:stone`) as the content names it (`base:block/stone`).
/// Relative names are the mod's own, which the game running the editor doesn't have.
pub fn picture_name(arg: &str, folder: &str) -> Option<String> {
    let mut valid = true;
    let name = weft_abi::map_layer_names(arg, |name| {
        let Some((ns, path)) = name.split_once(':') else {
            valid = false;
            return name.into();
        };
        if path.starts_with(&format!("{folder}/")) { name.into() } else { format!("{ns}:{folder}/{path}") }
    });
    valid.then_some(name)
}

pub use weft_abi::editor::{Method, Param};

include!(concat!(env!("OUT_DIR"), "/sdk_methods.rs"));

/// What `kind`'s builder can be told, by name.
pub fn methods(kind: What) -> impl Iterator<Item = &'static Method> {
    static EXTRA: std::sync::OnceLock<Vec<Method>> = std::sync::OnceLock::new();
    let extra = if let Some(c) = crate::content::try_content() {
        EXTRA
            .get_or_init(|| {
                let mut found = Vec::new();
                if let Some(component) = c.components.id(weft_abi::comp::EDITOR_METHOD) {
                    let mut records: Vec<_> = c
                        .game
                        .catalog
                        .ids(weft_abi::entry_kind::RECORD)
                        .iter()
                        .collect();
                    records.sort_by_key(|(name, _)| *name);
                    for (_, &id) in records {
                        if let Some(method) = c
                            .components
                            .bytes(weft_abi::entry_kind::RECORD as usize, id, component)
                            .and_then(weft_abi::data::from_bytes::<Method>)
                        {
                            found.push(method);
                        }
                    }
                }
                found
            })
            .as_slice()
    } else {
        &[]
    };
    METHODS
        .iter()
        .chain(extra)
        .filter(move |m| m.on & kind.bit() != 0)
}

pub fn method(kind: What, name: &str) -> Option<&'static Method> {
    methods(kind).find(|m| m.name == name)
}

/// A value of a parameter's type to start a new call with.
pub fn default_arg(p: &Param) -> String {
    if !p.default.is_empty() {
        return p.default.clone();
    }
    match p.ty.as_str() {
        "&str" => "\"\"".into(),
        "Gate" => "Gate::new()".into(),
        "GateLogic" => "GateLogic::And".into(),
        "f32" | "f64" => "1.0".into(),
        "bool" => "true".into(),
        // A box's far corner starts at the block's, so a new box is the whole block.
        "[f32;3]" if p.name == "max" => "[16.0, 16.0, 16.0]".into(),
        "[f32;3]" => "[0.0, 0.0, 0.0]".into(),
        "[(&str,[f32;4]);6]" => "[(\"\", abi::UV_FROM_BOX); 6]".into(),
        "[u8;6]" => "[0; 6]".into(),
        t if t.starts_with('[') => "[]".into(),
        t if t.starts_with("&[") => "&[]".into(),
        "&CreativeTab" => "&CreativeTab::named(\"base:building\")".into(),
        t if t.starts_with('u') || t.starts_with('i') => "0".into(),
        _ => String::new(),
    }
}

pub fn unquote(a: &str) -> Option<String> {
    serde_json::from_str(a.trim()).ok()
}

/// Comma-separated Rust literals, preserving nested arrays, tuples and strings.
pub fn parts(s: &str) -> Option<Vec<&str>> {
    split_top(s, ',')
}

fn split_top(s: &str, separator: char) -> Option<Vec<&str>> {
    let mut out = Vec::new();
    let (mut depth, mut quoted, mut escaped, mut start) = (0i32, false, false, 0);
    for (i, c) in s.char_indices() {
        if quoted {
            if !escaped && c == '"' {
                quoted = false;
            }
            escaped = !escaped && c == '\\';
            continue;
        }
        match c {
            '"' => quoted = true,
            '[' | '(' | '{' => depth += 1,
            ']' | ')' | '}' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            c if c == separator && depth == 0 => {
                out.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if depth != 0 || quoted {
        return None;
    }
    if !s[start..].trim().is_empty() {
        out.push(s[start..].trim());
    }
    Some(out)
}

pub fn array<T, const N: usize>(s: &str, read: impl Fn(&str) -> Option<T>) -> Option<[T; N]> {
    let inner = s.trim().strip_prefix('[')?.strip_suffix(']')?;
    let repeated = split_top(inner, ';')?;
    if repeated.len() == 2 {
        if repeated[1].parse::<usize>().ok()? != N {
            return None;
        }
        return (0..N)
            .map(|_| read(repeated[0]))
            .collect::<Option<Vec<_>>>()?
            .try_into()
            .ok();
    }
    parts(inner)?
        .into_iter()
        .map(read)
        .collect::<Option<Vec<_>>>()?
        .try_into()
        .ok()
}

pub fn number(s: &str) -> Option<f32> {
    let s = s.trim().replace('_', "");
    let s = s
        .strip_suffix("f32")
        .or_else(|| s.strip_suffix("f64"))
        .unwrap_or(&s);
    s.parse::<f32>().ok().filter(|v| v.is_finite())
}

pub fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// A number as code writes it: `1.5`, `2.0`, `0.25`; no float noise (`7.500005` is `7.5`).
pub fn num(v: f32) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    let s = format!("{r}");
    if s.contains('.') { s } else { format!("{s}.0") }
}

pub fn three(a: &str) -> Option<[f32; 3]> {
    array(a, number)
}

fn handle(t: &Thing) -> String {
    t.name.to_uppercase()
}

/// The code for everything: the handles, then each one's calls in the order they were made.
pub fn write(things: &[Thing]) -> String {
    let mut out = String::from("// Made in the editor\n");
    let mut imports: Vec<_> = things
        .iter()
        .flat_map(|t| {
            t.calls
                .iter()
                .filter_map(|call| method(t.kind, &call.method))
        })
        .map(|m| m.import.as_str())
        .filter(|name| !name.is_empty())
        .collect();
    imports.sort_unstable();
    imports.dedup();
    for import in imports {
        out += &format!("use {import};\n");
    }
    for t in things {
        let ty = if t.kind == What::Block { "Block" } else { "Item" };
        out += &format!("pub static {}: {ty} = {ty}::named({});\n", handle(t), quote(&t.name));
    }
    for t in things {
        out += &format!("\nr.{}(&{})", t.kind.folder(), handle(t));
        for c in &t.calls {
            out += &format!("\n    .{}({})", c.method, c.args.join(", "));
        }
        out += ";\n";
    }
    out
}

/// Code without its comments, strings kept whole.
fn strip_comments(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        let mut in_str = false;
        let mut prev = ' ';
        let mut cut = line.len();
        for (i, c) in line.char_indices() {
            if c == '"' && prev != '\\' {
                in_str = !in_str;
            }
            if !in_str && c == '/' && prev == '/' {
                cut = i - 1;
                break;
            }
            prev = c;
        }
        out += &line[..cut];
        out.push('\n');
    }
    out
}

/// Reads the code back. Each `r.block(&X)` / `r.item(&X)` chain becomes a thing, named by its
/// `static X: Block = Block::named("x")`; a mistake says which line.
pub fn read(src: &str) -> Result<Vec<Thing>, String> {
    let src = strip_comments(src);
    if src.trim().is_empty() {
        return Ok(Vec::new());
    }
    let line_of = |at: usize| src[..at].matches('\n').count() + 1;
    let mut names: Vec<(String, String)> = Vec::new();
    for decl in src.split(';') {
        let Some(i) = decl.find("static ") else { continue };
        let decl = &decl[i + 7..];
        if let (Some((h, _)), Some(name)) = (decl.split_once(':'), decl.split('"').nth(1)) {
            names.push((h.trim().to_string(), name.to_string()));
        }
    }
    let mut things = Vec::new();
    let mut at = 0;
    while let Some(i) = src[at..].find("r.").map(|i| at + i) {
        let open = &src[i + 2..];
        let (kind, skip) = if open.starts_with("block(") {
            (What::Block, 8)
        } else if open.starts_with("item(") {
            (What::Item, 7)
        } else {
            at = i + 2;
            continue;
        };
        let close = src[i + skip..]
            .find(')')
            .map(|c| i + skip + c)
            .ok_or_else(|| format!("line {}: no `)`", line_of(i)))?;
        let h = src[i + skip..close].trim().trim_start_matches('&').trim();
        let name = names
            .iter()
            .find(|n| n.0 == h)
            .map(|n| n.1.clone())
            .ok_or_else(|| format!("line {}: no `static {h}` above", line_of(i)))?;
        let mut t = Thing { kind, name, calls: Vec::new() };
        let mut pos = close + 1;
        loop {
            let s = &src[pos..];
            let trimmed = s.trim_start();
            pos += s.len() - trimmed.len();
            if trimmed.starts_with(';') {
                pos += 1;
                break;
            }
            if !trimmed.starts_with('.') {
                return Err(format!("line {}: expected `.call(...)` or `;`", line_of(pos)));
            }
            let (call, len) = one_call(trimmed).ok_or_else(|| format!("line {}: unfinished call", line_of(pos)))?;
            t.calls.push(call);
            pos += len;
        }
        things.push(t);
        at = pos;
    }
    if things.is_empty() {
        return Err("nothing registered: no `r.block(&X)` or `r.item(&X)`".into());
    }
    Ok(things)
}

/// `.name(args)` at the start of `s`, its arguments split at their top commas, and how long it is.
fn one_call(s: &str) -> Option<(Call, usize)> {
    let open = s.find('(')?;
    let method = s[1..open].trim().to_string();
    if method.is_empty() || !method.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let (mut depth, mut in_str, mut prev) = (0, false, ' ');
    let mut args = Vec::new();
    let mut cur = String::new();
    for (i, c) in s[open..].char_indices() {
        if c == '"' && prev != '\\' {
            in_str = !in_str;
        }
        prev = c;
        if !in_str {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                if !cur.trim().is_empty() {
                    args.push(cur.trim().to_string());
                }
                return Some((Call { method, args }, open + i + 1));
            }
            if depth == 1 && (c == ',' || (c == '(' && i == 0)) {
                if c == ',' {
                    args.push(cur.trim().to_string());
                    cur.clear();
                }
                continue;
            }
        }
        cur.push(c);
    }
    None
}

/// Colors for `highlight`, `None` for the field's own.
#[derive(Clone, Copy, Default)]
pub struct Colors {
    pub keyword: Option<u32>,
    pub call: Option<u32>,
    pub string: Option<u32>,
    pub number: Option<u32>,
    pub comment: Option<u32>,
}

/// Code as spans, every glyph the same width (`mono`), so a field puts its caret by counting.
pub fn highlight(src: &str, c: &Colors) -> Vec<Span> {
    const KEYWORDS: [&str; 8] = ["pub", "static", "let", "fn", "use", "mut", "move", "as"];
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let start = i;
        let ch = chars[i];
        let take = |i: &mut usize, f: &dyn Fn(char) -> bool| {
            while *i < chars.len() && f(chars[*i]) {
                *i += 1;
            }
        };
        let color = if ch == '/' && chars.get(i + 1) == Some(&'/') {
            take(&mut i, &|c| c != '\n');
            c.comment
        } else if ch == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' && chars[i] != '\n' {
                i += if chars[i] == '\\' { 2 } else { 1 };
            }
            i = (i + 1).min(chars.len());
            c.string
        } else if ch.is_ascii_digit() {
            take(&mut i, &|c| c.is_ascii_alphanumeric() || c == '.' || c == '_');
            c.number
        } else if ch.is_alphabetic() || ch == '_' {
            take(&mut i, &|c| c.is_alphanumeric() || c == '_');
            let word: String = chars[start..i].iter().collect();
            if KEYWORDS.contains(&word.as_str()) {
                c.keyword
            } else if chars.get(i) == Some(&'(') {
                c.call
            } else {
                None
            }
        } else {
            i += 1;
            take(&mut i, &|c| !(c.is_alphanumeric() || c == '_' || c == '"' || c == '/'));
            None
        };
        // Text drops spaces at the start of a line: these never break, and are as wide.
        let s: String = chars[start..i]
            .iter()
            .map(|&c| if c == ' ' { '\u{a0}' } else { c })
            .collect();
        let mut span = Span::text(s, color);
        span.mono = true;
        out.push(span);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_reads_back_the_same() {
        let mut b = Thing::new(What::Block, "ruby_block");
        b.calls
            .push(Call::new("model_box", &["[0.0, 0.0, 0.0]", "[16.0, 8.0, 16.0]"]));
        b.calls
            .push(Call::new("rule", &["Rule::random(4.0).becomes(\"base:stone\")"]));
        b.calls.push(Call::new("falling", &[]));
        b.calls
            .push(Call::new("tab", &["&CreativeTab::named(\"base:building\")"]));
        assert!(read(&write(&[])).unwrap().is_empty());
        let things = vec![b, Thing::new(What::Item, "ruby")];
        let code = write(&things);
        assert_eq!(read(&code).unwrap(), things, "{code}");
        assert_eq!(three(&things[0].args_of("model_box").next().unwrap()[1]), Some([16.0, 8.0, 16.0]));
        assert_eq!(things[0].picture().as_deref(), Some("base:block/stone"));
        // Written by hand: comments, other spacing.
        let hand = "pub static X: Block = Block::named(\"x\"); // mine\nr.block(&X).hardness(7.5) .light(15)\n  .tag(\"a, b\");";
        let t = &read(hand).unwrap()[0];
        assert_eq!(t.calls, vec![Call::new("hardness", &["7.5"]), Call::new("light", &["15"]), Call::new("tag", &["\"a, b\""])]);
        assert!(
            read("r.block(&Y).hardness(1.0);")
                .unwrap_err()
                .contains("line 1")
        );
        assert!(
            read("pub static X: Block = Block::named(\"x\");\nr.block(&X)\n    .hardness(1.0)\n    oops;")
                .unwrap_err()
                .contains("line 4")
        );
    }

    #[test]
    fn the_sdk_is_listed() {
        let tex = method(What::Block, "texture").unwrap();
        assert_eq!((tex.params[0].ty.as_str(), tex.params[0].picture.as_str()), ("&str", "block"));
        assert!(method(What::Block, "tint").is_some() && method(What::Item, "max_stack").is_some());
        assert!(method(What::Item, "hardness").is_none());
    }

    #[test]
    fn numbers_without_noise() {
        assert_eq!(num(7.500005), "7.5");
        assert_eq!(num(2.0), "2.0");
        assert_eq!(three("[0.0; 3]"), Some([0.0; 3]));
        assert_eq!(array::<u8, 6>("[0; 6]", |v| v.parse().ok()), Some([0; 6]));
        assert_eq!(parts("[0.0; 3], (\"a,b\", [1.0; 4])"), Some(vec!["[0.0; 3]", "(\"a,b\", [1.0; 4])"]));
        assert_eq!(unquote(&quote("a\\b\"c")), Some("a\\b\"c".into()));
        assert!(number("NaN").is_none());
    }
}

/// Fields of a literal struct argument, in the schema's order.
pub fn struct_values(value: &str, p: &Param) -> Option<Vec<(String, String, String)>> {
    let fields = p
        .schema
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .or_else(|| p.schema.strip_prefix('<').and_then(|s| s.strip_suffix('>')))?;
    let open = value.find('{')?;
    let body = value[open + 1..].trim().strip_suffix('}')?;
    let values: Vec<_> = parts(body)?
        .into_iter()
        .map(|s| s.split_once(':'))
        .collect::<Option<_>>()?;
    weft_abi::data::fields(fields)?
        .into_iter()
        .map(|(name, ty)| {
            let value = values.iter().find(|(n, _)| n.trim() == name)?.1.trim();
            Some((name.into(), ty.into(), value.into()))
        })
        .collect()
}

pub fn struct_code(ty: &str, values: &[(String, String, String)]) -> String {
    format!(
        "{ty} {{ {} }}",
        values
            .iter()
            .map(|(name, _, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub const LOGIC_NAMES: [&str; 9] = ["And", "Or", "Xor", "Not", "Nand", "Nor", "Xnor", "Max", "Min"];

pub const SIDES: [&str; 6] = ["Left", "Right", "Bottom", "Top", "Front", "Rear"];

#[derive(Clone)]
pub struct GateArgs {
    pub inputs: u8,
    pub outputs: u8,
    pub delay: u32,
}

impl GateArgs {
    pub fn read(value: &str) -> Option<Self> {
        let bits = |names: &[&str]| {
            SIDES
                .iter()
                .enumerate()
                .filter(|(_, side)| names.contains(side))
                .fold(0, |bits, (i, _)| bits | 1 << i)
        };
        let mut args = Self { inputs: bits(&["Left", "Right", "Rear"]), outputs: bits(&["Front"]), delay: 1 };
        let mut rest = value.trim().strip_prefix("Gate::new()")?;
        while !rest.is_empty() {
            rest = rest.strip_prefix('.')?;
            let open = rest.find('(')?;
            let end = rest.find(')')?;
            let (name, value) = (rest.get(..open)?, rest.get(open + 1..end)?);
            match name {
                "inputs" | "outputs" => {
                    let values = value
                        .trim()
                        .strip_prefix('&')?
                        .strip_prefix('[')?
                        .strip_suffix(']')?;
                    let mut bits = 0;
                    for side in parts(values)? {
                        let side = side.trim().strip_prefix("Side::")?;
                        bits |= 1 << SIDES.iter().position(|s| *s == side)?;
                    }
                    if name == "inputs" {
                        args.inputs = bits & !args.outputs;
                    } else {
                        args.outputs = bits;
                        args.inputs &= !bits;
                    }
                }
                "delay" => args.delay = value.trim().parse().ok()?,
                _ => return None,
            }
            rest = &rest[end + 1..];
        }
        Some(args)
    }

    pub fn code(&self) -> String {
        let sides = |bits: u8| {
            SIDES
                .iter()
                .enumerate()
                .filter(|(i, _)| bits & (1 << i) != 0)
                .map(|(_, name)| format!("Side::{name}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!("Gate::new().outputs(&[{}]).inputs(&[{}]).delay({})", sides(self.outputs), sides(self.inputs), self.delay)
    }
}

/// A registered component value as the same typed literal used by its builder argument.
pub fn literal(ty: &str, schema: &str, bytes: &[u8]) -> Option<String> {
    if let Some(fields) = schema
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .or_else(|| schema.strip_prefix('<').and_then(|s| s.strip_suffix('>')))
    {
        let mut values = Vec::new();
        for (name, _) in weft_abi::data::fields(fields)? {
            let (field_type, range) = weft_abi::data::field(schema, bytes, name)?;
            values.push(format!("{name}: {}", literal(field_type, field_type, bytes.get(range)?)?));
        }
        return Some(format!("{ty} {{ {} }}", values.join(", ")));
    }
    macro_rules! number {
        ($t:ty) => {
            weft_abi::data::from_bytes::<$t>(bytes).map(|n| format!("{n:?}"))
        };
    }
    match schema {
        "u8" => number!(u8),
        "i8" => number!(i8),
        "u16" => number!(u16),
        "i16" => number!(i16),
        "u32" => number!(u32),
        "i32" => number!(i32),
        "u64" => number!(u64),
        "i64" => number!(i64),
        "f32" => number!(f32),
        "f64" => number!(f64),
        "bool" => number!(bool),
        "str" => weft_abi::data::from_bytes::<String>(bytes).map(|s| quote(&s)),
        _ => None,
    }
}
