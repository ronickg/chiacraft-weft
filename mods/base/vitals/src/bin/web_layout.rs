//! Prints a web layout as JSON for pictures (`mods/base/source/ui/web_map.py`):
//! `cargo run -p vitals --bin web_layout -- clusters|branches|wheel > layout.json`.

use vitals::perks;
use vitals::web::Node as Kind;
use vitals::web_layout::Style;

fn main() {
    let style = match std::env::args().nth(1).as_deref() {
        Some("branches") => Style::Branches,
        Some("wheel") => Style::Wheel,
        _ => Style::Clusters,
    };
    let web = perks::build_with(style);
    let mut out = String::from("{\"nodes\":[");
    for (i, n) in web.nodes.iter().enumerate() {
        let kind = match n.kind {
            _ if n.id == perks::START => "start",
            Kind::Small => "small",
            Kind::Notable => "notable",
            Kind::Keystone => "keystone",
            Kind::Socket => "socket",
        };
        let region = n.region.map_or(-1, |r| r.index() as i32);
        out += &format!(
            "{}{{\"id\":{},\"x\":{:.2},\"y\":{:.2},\"kind\":\"{kind}\",\"region\":{region},\"ring\":{}}}",
            if i > 0 { "," } else { "" },
            n.id,
            n.x,
            n.y,
            n.ring
        );
    }
    out += "],\"links\":[";
    for (i, &(a, b)) in web.links.iter().enumerate() {
        let arc = web
            .arcs
            .get(&vitals::web_layout::key(a, b))
            .map_or("null".to_string(), |c| format!("[{:.2},{:.2}]", c.0, c.1));
        out += &format!("{}[{a},{b},{arc}]", if i > 0 { "," } else { "" });
    }
    out += "]}";
    println!("{out}");
}
