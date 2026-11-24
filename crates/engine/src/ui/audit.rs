//! The text-fit check (`--ui-audit`, chiacraft 11-24, Ronald: "It should be fairly simple to have
//! it correct everywhere"): after a screen is laid out, every text and widget is measured against
//! the box it is drawn in, and each fault is logged once as one `ui-audit:` line naming the
//! screen, the widget's path and its words. `notes/ui/audit/fit_all.txt` opens every screen in
//! every language and GUI scale; `notes/ui/audit/fit_report.py` turns run.log into the list.
//!
//! The faults:
//! - `uneven scale`: a text drawn at a scale that isn't whole (a step down from a `fit` scale to
//!   1x is allowed; the layout no longer squeezes, so this would be a widget setting its own).
//! - `lines past box`: lines taller than the box they are in: wrapped text, or one-line text its
//!   box was too narrow for (the layout wraps it rather than squeeze it).
//! - `on frame`: a text reaching into its button's or panel's frame: within `inset` of the frame's
//!   left or right edge (its `.gui`: how far the drawn frame comes into the box) plus one GUI
//!   pixel, or past its top or bottom.
//! - `cut by panel`: a widget sticking out of the framed widget it is in.
//! - `cut by list`: a widget wider than the scrolling list it is in (lists scroll up and down only).
//! - `off screen`: a widget past the screen's edge.
//! - `siblings overlap`: two widgets laid out side by side (or one under the other) that meet.
//! - `text over text`, `text over widget`: a text's ink on another text, or on something drawn
//!   that doesn't hold it (a slider's knob, a picture, an item, another button's frame).

use rustc_hash::{FxHashMap, FxHashSet};

use super::{Kind, Rect, Ui};

/// Space a text keeps from its frame beyond the frame itself, GUI pixels.
const MARGIN: f32 = 1.0;
/// Rounding the layout may leave (it snaps to half GUI pixels).
const SLACK: f32 = 0.6;

/// One fault: what, where in the tree, the words it concerns, by how much.
pub struct Fault {
    pub what: &'static str,
    pub path: String,
    pub text: String,
    pub by: f32,
}

/// The faults in `ui` (laid out at `size`). `inset` gives a framed widget's frame reach into its
/// box (left, top, right, bottom), from its class.
pub fn check(ui: &Ui, size: [f32; 2], inset: &mut dyn FnMut(&str) -> [f32; 4]) -> Vec<Fault> {
    let mut out = Vec::new();
    if ui.nodes.is_empty() {
        return out;
    }
    let mut parent = vec![usize::MAX; ui.nodes.len()];
    for (i, n) in ui.nodes.iter().enumerate() {
        for &c in &n.children {
            if c < parent.len() {
                parent[c] = i;
            }
        }
    }
    let screen = Rect { x: 0.0, y: 0.0, w: size[0], h: size[1] };
    let unit = ui.text_unit;
    for (id, n) in ui.nodes.iter().enumerate() {
        let r = n.rect;
        if r.w <= 0.0 || r.h <= 0.0 {
            continue;
        }
        // Hidden under a scroll: what can't be seen can't be cut.
        if let Some(c) = n.clip
            && (r.y >= c.y + c.h || r.y + r.h <= c.y)
        {
            continue;
        }
        let words = words_of(ui, id);
        // The box the text or widget is drawn in: a text's ink, else the widget's rect.
        let ink = match &n.kind {
            Kind::Text(t) => {
                let w = t.layout.width * t.scale * unit;
                let h = t.layout.height * t.scale * unit;
                if t.spans.is_empty() || w <= 0.0 {
                    continue;
                }
                if (t.scale - t.scale.round()).abs() > 0.01 || t.scale < 1.0 {
                    out.push(Fault { what: "uneven scale", path: path(ui, &parent, id), text: words.clone(), by: t.scale });
                }
                // Wrapped lines, or a one-line text its box made wrap, taller than the box.
                if h > r.h + SLACK {
                    out.push(Fault { what: "lines past box", path: path(ui, &parent, id), text: words.clone(), by: h - r.h });
                }
                let x = if t.center { r.x + (r.w - w) / 2.0 } else { r.x };
                Rect { x, y: r.y, w, h: h.min(r.h) }
            }
            _ => r,
        };
        if out_by(ink, screen) > SLACK {
            out.push(Fault { what: "off screen", path: path(ui, &parent, id), text: words.clone(), by: out_by(ink, screen) });
        }
        if let Some(c) = n.clip {
            let by = (c.x - ink.x).max(ink.x + ink.w - (c.x + c.w));
            if by > SLACK {
                out.push(Fault { what: "cut by list", path: path(ui, &parent, id), text: words.clone(), by });
            }
        }
        // The nearest framed widget above: its frame and its box.
        let mut p = parent[id];
        while p != usize::MAX && frame_class(&ui.nodes[p].kind).is_none() {
            p = parent[p];
        }
        if p == usize::MAX {
            continue;
        }
        let f = &ui.nodes[p];
        let class = frame_class(&f.kind).unwrap_or_default();
        // Its own scroll, or one between: the list scrolls, its rows may pass up and down.
        let scrolled = n.clip.is_some() && n.clip != f.clip;
        if matches!(n.kind, Kind::Text(_)) {
            let [l, _, rr, _] = inset(class);
            // Across: off the frame's picture by a pixel. Up and down the line box is taller than
            // the ink, so only past the box itself counts.
            let room = Rect { x: f.rect.x + l + MARGIN, y: f.rect.y, w: f.rect.w - l - rr - 2.0 * MARGIN, h: f.rect.h };
            let mut by = (room.x - ink.x).max(ink.x + ink.w - (room.x + room.w));
            if !scrolled {
                by = by.max((room.y - ink.y).max(ink.y + ink.h - (room.y + room.h)));
            }
            if by > SLACK && (l + rr) > 0.0 {
                out.push(Fault { what: "on frame", path: path(ui, &parent, id), text: words.clone(), by });
                continue;
            }
        }
        let mut by = (f.rect.x - r.x).max(r.x + r.w - (f.rect.x + f.rect.w));
        if !scrolled {
            by = by.max((f.rect.y - r.y).max(r.y + r.h - (f.rect.y + f.rect.h)));
        }
        if by > SLACK && n.style.at.is_none() {
            out.push(Fault { what: "cut by panel", path: path(ui, &parent, id), text: words, by });
        }
    }
    overlaps(ui, &parent, &mut out);
    out
}

/// What a widget draws, where it can be seen: a text's ink, else its rect for the kinds that
/// draw (frames, sliders, pictures, items, bars, fields); none for a plain layout box.
fn drawn(ui: &Ui, id: usize) -> Option<(Rect, &'static str)> {
    let n = &ui.nodes[id];
    let r = n.rect;
    let (rect, what) = match &n.kind {
        Kind::Text(t) => {
            let unit = ui.text_unit;
            let w = t.layout.width * t.scale * unit;
            if t.spans.is_empty() || w <= 0.0 {
                return None;
            }
            let x = if t.center { r.x + (r.w - w) / 2.0 } else { r.x };
            (Rect { x, y: r.y, w, h: (t.layout.height * t.scale * unit).min(r.h.max(1.0)) }, "text")
        }
        Kind::Slider(_) => (r, "slider"),
        Kind::Image { .. } | Kind::BoundImage { .. } => (r, "image"),
        Kind::Item { .. } => (r, "item"),
        Kind::Icons { .. } => (r, "icons"),
        Kind::Progress { .. } => (r, "bar"),
        Kind::Input { .. } => (r, "field"),
        Kind::Custom { .. } => (r, "slot"),
        Kind::Button { class, .. } if !class.is_empty() => (r, "button"),
        Kind::Container { class: Some(c) } if !c.is_empty() => (r, "panel"),
        _ => return None,
    };
    let seen = match n.clip {
        Some(c) => rect.intersect(c),
        None => rect,
    };
    (seen.w > 0.0 && seen.h > 0.0).then_some((seen, what))
}

/// Overlaps: two flow siblings on each other, and a text over another text or over anything
/// drawn that doesn't hold it (a knob, a picture, another button's frame).
fn overlaps(ui: &Ui, parent: &[usize], out: &mut Vec<Fault>) {
    let hit = |a: Rect, b: Rect| {
        let i = a.intersect(b);
        (i.w > SLACK && i.h > SLACK).then_some(i.w.min(i.h))
    };
    // Siblings laid out side by side or one under another never meet.
    for n in &ui.nodes {
        let flow: Vec<usize> = n.children.iter().copied().filter(|&c| ui.nodes[c].style.at.is_none()).collect();
        for (i, &a) in flow.iter().enumerate() {
            for &b in &flow[i + 1..] {
                let (ra, rb) = (ui.nodes[a].rect, ui.nodes[b].rect);
                if ra.w <= 0.0 || rb.w <= 0.0 || ra.h <= 0.0 || rb.h <= 0.0 {
                    continue;
                }
                if let Some(by) = hit(ra, rb) {
                    let text = format!("{} / {}", words_of(ui, a), words_of(ui, b));
                    out.push(Fault { what: "siblings overlap", path: path(ui, parent, a), text, by });
                }
            }
        }
    }
    let all: Vec<(usize, Rect, &'static str)> = (0..ui.nodes.len()).filter_map(|i| drawn(ui, i).map(|(r, w)| (i, r, w))).collect();
    let holds = |a: usize, b: usize| {
        // whether `a` is above `b` in the tree
        let mut i = b;
        while i != usize::MAX {
            if i == a {
                return true;
            }
            i = parent[i];
        }
        false
    };
    for &(t, tr, tw) in &all {
        if tw != "text" {
            continue;
        }
        for &(o, or, ow) in &all {
            if o == t || holds(o, t) || (ow == "text" && o < t) {
                continue;
            }
            if let Some(by) = hit(tr, or) {
                let what = if ow == "text" { "text over text" } else { "text over widget" };
                let text = format!("{} / {ow} {}", words_of(ui, t), words_of(ui, o));
                out.push(Fault { what, path: path(ui, parent, t), text, by });
            }
        }
    }
}

/// How far `r` reaches past `outer` on its worst side (0 or less: inside).
fn out_by(r: Rect, outer: Rect) -> f32 {
    (outer.x - r.x)
        .max(outer.y - r.y)
        .max(r.x + r.w - (outer.x + outer.w))
        .max(r.y + r.h - (outer.y + outer.h))
}

/// A widget that draws a frame its content must keep clear of.
fn frame_class(k: &Kind) -> Option<&str> {
    match k {
        Kind::Button { class, .. } if !class.is_empty() => Some(class),
        Kind::Container { class: Some(c) } if !c.is_empty() => Some(c),
        _ => None,
    }
}

/// The words a widget shows or holds (its first text, for a button or panel).
fn words_of(ui: &Ui, id: usize) -> String {
    let mut stack = vec![id];
    while let Some(i) = stack.pop() {
        if let Kind::Text(t) = &ui.nodes[i].kind {
            return super::text::plain_text(&t.spans);
        }
        stack.extend(ui.nodes[i].children.iter().rev().copied());
    }
    String::new()
}

/// `window>well>row3>button#back>text`: classes, ids and child places from the root down.
fn path(ui: &Ui, parent: &[usize], id: usize) -> String {
    let mut parts = Vec::new();
    let mut i = id;
    while i != usize::MAX {
        let n = &ui.nodes[i];
        let mut s = match &n.kind {
            Kind::Text(_) => "text".to_string(),
            Kind::Button { class, .. } => format!("button.{class}"),
            Kind::Container { class: Some(c) } => c.clone(),
            Kind::Container { class: None } => {
                let p = parent[i];
                let at = if p == usize::MAX { 0 } else { ui.nodes[p].children.iter().position(|&c| c == i).unwrap_or(0) };
                if n.style.row { format!("row{at}") } else { format!("col{at}") }
            }
            Kind::Input { .. } => "input".into(),
            Kind::Slider(_) => "slider".into(),
            Kind::Image { .. } | Kind::BoundImage { .. } => "image".into(),
            _ => "node".into(),
        };
        if let Some(name) = &n.id {
            s.push('#');
            s.push_str(name);
        }
        parts.push(s);
        i = parent[i];
    }
    parts.reverse();
    parts.join(">")
}

/// Frames' reach into their box by GUI texture, read once from their `.gui` (`inset`).
#[derive(Default)]
pub struct Insets {
    by_class: FxHashMap<String, [f32; 4]>,
    /// Faults already logged (the same screen is laid out again on every change).
    said: FxHashSet<String>,
}

impl Insets {
    pub fn get(&mut self, look: &super::look::Look, assets: &crate::assets::Assets, class: &str) -> [f32; 4] {
        if let Some(v) = self.by_class.get(class) {
            return *v;
        }
        let v = look
            .texture(class, "")
            .and_then(|tex| assets.read(&tex, "textures", "gui"))
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .and_then(|src| crate::kv::get(&src, "inset").map(|v| v.to_string()))
            .map(|v| {
                let n: Vec<f32> = v.split_whitespace().filter_map(|x| x.parse().ok()).collect();
                match n.as_slice() {
                    [a] => [*a; 4],
                    [l, t, r, b] => [*l, *t, *r, *b],
                    _ => [0.0; 4],
                }
            })
            .unwrap_or([0.0; 4]);
        self.by_class.insert(class.to_string(), v);
        v
    }

    /// Logs `faults` of `screen` not said before, with the language, GUI scale and window.
    pub fn log(&mut self, screen: &str, faults: &[Fault], lang: &str, scale: f32, window: (u32, u32)) {
        for f in faults {
            let line = format!(
                "{screen} | {} | {} | \"{}\" | {:.2} | {lang} | {scale} | {}x{}",
                f.what, f.path, f.text, f.by, window.0, window.1
            );
            if self.said.insert(line.clone()) {
                log::warn!("ui-audit: {line}");
            }
        }
    }
}
