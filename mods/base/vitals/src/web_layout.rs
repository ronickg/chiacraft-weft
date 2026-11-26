//! Where the web's nodes sit, and which links curve (Ronald, 11-05, on the even wheel of 16 equal
//! spokes: "it sohuldnt be uniform, if that makes sesne check poe2 one"). Path of Exile 2's tree,
//! its layout only: regions of different sizes, notables on rings of small nodes (two ways round:
//! paths that fork and rejoin), curved arcs, paths of different lengths, keystones at uneven
//! spots on the rim, links across regions, uneven spacing.
//!
//! It only places nodes and adds: every node `perks` builds keeps its id, its effect and every link
//! it had, so saved builds stay whole. What it adds are links (the other way round a ring, a side
//! branch) and small "travel" nodes on them (a region's places `EXTRA_FROM` on, giving its lanes'
//! families), more in a bigger region.
//!
//! Two styles for Ronald's pick: `Clusters` (rings round the notables, as Path of Exile 2's) and
//! `Branches` (curving paths that fork and rejoin, notables at the bends). Positions are seeded per
//! region, so they're the same on every machine and every run.

use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

use crate::perks::{Node, REGIONS, START, id};
use crate::skills::COUNT;
use crate::web::Node as Kind;

/// The layout styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// The first web: 16 equal spokes (kept to compare).
    Wheel,
    /// Rings of small nodes round the notables (Path of Exile 2's clusters).
    Clusters,
    /// Curving branches that fork and rejoin, notables at the bends.
    Branches,
    /// Ronald's pick (11-05, "A, with clearer islands"): the clusters, each region an island of
    /// its own with empty space round it, a small start hub of gates, bridges across the gaps.
    Islands,
}

/// The style the game uses.
pub const STYLE: Style = Style::Islands;

/// How much room each region gets round the web (`skills::SKILLS` order): the skills with most to
/// do today, and Taming (a whole group, Beasts, in one region), get the widest; Trading, with
/// nothing to raise it yet, the narrowest.
pub const ROOM: [f32; COUNT] = [1.35, 1.0, 1.15, 1.25, 0.8, 1.1, 1.05, 1.0, 0.9, 1.0, 0.85, 0.85, 1.4, 1.0, 0.9, 0.7];
/// A region's first extra travel node's place (they go up from here).
pub const EXTRA_FROM: u16 = 21;
/// Travel nodes a region gets for its room: none at 0.7, four at 1.4.
pub fn extras(room: f32) -> usize {
    (((room - 0.7) * 5.0).round().max(0.0) as usize).min(4)
}

/// A link's curve: the centre of the circle its arc follows.
pub type Arcs = HashMap<(u16, u16), (f32, f32)>;

pub fn key(a: u16, b: u16) -> (u16, u16) {
    (a.min(b), a.max(b))
}

/// A number in 0..1 from a seed and a step: the same on every machine.
fn rnd(seed: u32, k: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ k.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// -1..1.
fn sym(seed: u32, k: u32) -> f32 {
    rnd(seed, k) * 2.0 - 1.0
}

/// Each region's middle angle and half its width, radians clockwise from the top.
pub fn wedges() -> [(f32, f32); COUNT] {
    let total: f32 = ROOM.iter().sum();
    let mut out = [(0.0, 0.0); COUNT];
    // Melee's middle at the top.
    let mut at = -ROOM[0] / total * PI;
    for i in 0..COUNT {
        let w = ROOM[i] / total * TAU;
        out[i] = (at + w / 2.0, w / 2.0);
        at += w;
    }
    out
}

/// A point in a region's frame: `u` out from the centre, `v` across (GUI px at zoom 1).
fn frame(mid: f32, u: f32, v: f32) -> (f32, f32) {
    let a = mid + v / u.max(1.0);
    (u * a.sin(), -u * a.cos())
}

fn add(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 + b.0, a.1 + b.1)
}

fn on_ring(c: (f32, f32), r: f32, angle: f32) -> (f32, f32) {
    (c.0 + r * angle.cos(), c.1 + r * angle.sin())
}

fn angle_to(c: (f32, f32), p: (f32, f32)) -> f32 {
    (p.1 - c.1).atan2(p.0 - c.0)
}

/// The centre of the arc through `a` and `b` bulging `bulge` of their distance to the left (negative:
/// right).
fn bulge(a: (f32, f32), b: (f32, f32), bulge: f32) -> (f32, f32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.hypot(dy).max(1e-3);
    let h = bulge * len;
    // The circle through a, b and the bulge's tip: its centre on the chord's normal.
    let d = (len * len / 4.0 - h * h) / (2.0 * h);
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let n = (-dy / len, dx / len);
    (mid.0 - n.0 * d, mid.1 - n.1 * d)
}

/// A disc the layout keeps clear: a cluster's ring, or a single node or two with its room.
#[derive(Clone, Copy)]
struct Blob {
    c: (f32, f32),
    r: f32,
    /// Fixed where it is (the entries stay round the start).
    pinned: bool,
}

/// Blob roles in a region, in order.
const ENTRY: usize = 0;
const RING_A1: usize = 1;
const RING_B1: usize = 2;
const MID: usize = 3;
const RING_A2: usize = 4;
const RING_B2: usize = 5;
const MEET: usize = 6;
const KEY: usize = 7;
const BRIDGE: usize = 8;
const ROLES: usize = 9;

/// Pushes overlapping discs apart (each half the overlap), never inside `inner` of the centre.
fn relax(blobs: &mut [Blob], gap: f32, inner: f32) {
    for _ in 0..80 {
        let mut moved = false;
        for i in 0..blobs.len() {
            for j in i + 1..blobs.len() {
                let (a, b) = (blobs[i], blobs[j]);
                let (dx, dy) = (b.c.0 - a.c.0, b.c.1 - a.c.1);
                let d = dx.hypot(dy).max(1e-3);
                let want = a.r + b.r + gap;
                if d >= want {
                    continue;
                }
                let push = (want - d) / 2.0 + 0.01;
                let (ux, uy) = (dx / d, dy / d);
                let (wa, wb) = match (a.pinned, b.pinned) {
                    (true, true) => continue,
                    (true, false) => (0.0, 2.0),
                    (false, true) => (2.0, 0.0),
                    _ => (1.0, 1.0),
                };
                blobs[i].c = (a.c.0 - ux * push * wa, a.c.1 - uy * push * wa);
                blobs[j].c = (b.c.0 + ux * push * wb, b.c.1 + uy * push * wb);
                moved = true;
            }
        }
        for b in blobs.iter_mut() {
            let l = b.c.0.hypot(b.c.1).max(1e-3);
            if !b.pinned && l < inner + b.r {
                let k = (inner + b.r) / l;
                b.c = (b.c.0 * k, b.c.1 * k);
            }
        }
        if !moved {
            break;
        }
    }
}

/// Lays out the nodes `perks` built (and adds travel nodes and links) in `style`.
pub fn lay(style: Style, nodes: &mut Vec<Node>, links: &mut Vec<(u16, u16)>, arcs: &mut Arcs) {
    if style == Style::Wheel {
        return;
    }
    if style == Style::Islands {
        return islands(nodes, links, arcs);
    }
    let wedges = wedges();
    let branches = style == Style::Branches;
    // Where each cluster sits, per region. There isn't room round the web for two rings side by
    // side in every region (at 200 px out, 16 regions' pairs need about 2,200 px of the 1,260
    // round), so one lane's ring sits inner and the other's outer, alternating with the next
    // region: the rings interleave round the web, and a bridge joins two rings at one depth.
    let mut blobs: Vec<Blob> = Vec::new();
    let mut ring_r = vec![[0.0f32; 4]; COUNT];
    for i in 0..COUNT {
        let (mid, half) = wedges[i];
        let room = ROOM[i];
        let seed = 1000 + i as u32 * 7919 + if branches { 50_000 } else { 0 };
        let hw = |u: f32| (u * half).max(1.0);
        let base = 23.0 + 8.0 * (room - 0.7);
        let rr =
            [base + 3.0 * rnd(seed, 30), base + 3.0 * rnd(seed, 31), base - 3.0 + 3.0 * rnd(seed, 32), base - 3.0 + 3.0 * rnd(seed, 33)];
        ring_r[i] = rr;
        // Lane A inner in even regions, lane B inner in odd ones.
        let a_inner = i % 2 == 0;
        let (u_in, u_out) = (170.0 + 14.0 * sym(seed, 4), 262.0 + 18.0 * sym(seed, 6));
        let (ua1, ub1) = if a_inner { (u_in, u_out) } else { (u_out, u_in) };
        let (ua2, ub2) = if a_inner {
            (u_in + 170.0 + 20.0 * sym(seed, 10), u_out + 150.0 + 20.0 * sym(seed, 11))
        } else {
            (u_out + 150.0 + 20.0 * sym(seed, 10), u_in + 170.0 + 20.0 * sym(seed, 11))
        };
        let um = ua2.max(ub2) + 58.0 + 10.0 * sym(seed, 12);
        let uk = um + 50.0 + 25.0 * rnd(seed, 14);
        let ksign = if rnd(seed, 16) < 0.5 { -1.0 } else { 1.0 };
        let entry = frame(mid, 64.0, 0.25 * hw(64.0) * sym(seed, 3));
        let mut b = [Blob { c: (0.0, 0.0), r: 0.0, pinned: false }; ROLES];
        b[ENTRY] = Blob { c: entry, r: 6.0, pinned: true };
        // A lane leans to its own side (A counter-clockwise, B clockwise), its inner ring less.
        b[RING_A1] =
            Blob { c: frame(mid, ua1, -(if a_inner { 0.15 } else { 0.4 }) * hw(ua1) + 6.0 * sym(seed, 34)), r: rr[0] + 6.0, pinned: false };
        b[RING_B1] =
            Blob { c: frame(mid, ub1, (if a_inner { 0.4 } else { 0.15 }) * hw(ub1) + 6.0 * sym(seed, 35)), r: rr[1] + 6.0, pinned: false };
        b[MID] = Blob { c: frame(mid, (ua1 + ub1) / 2.0 + 20.0, 0.1 * hw(200.0) * sym(seed, 36)), r: 12.0, pinned: false };
        b[RING_A2] = Blob { c: frame(mid, ua2, -0.35 * hw(ua2) + 10.0 * sym(seed, 13)), r: rr[2] + 6.0, pinned: false };
        b[RING_B2] = Blob { c: frame(mid, ub2, 0.35 * hw(ub2) + 10.0 * sym(seed, 15)), r: rr[3] + 6.0, pinned: false };
        b[MEET] = Blob { c: frame(mid, um, 0.2 * hw(um) * sym(seed, 17)), r: 9.0, pinned: false };
        b[KEY] = Blob { c: frame(mid, uk, ksign * (0.15 + 0.5 * rnd(seed, 18)) * hw(uk)), r: 14.0, pinned: false };
        b[BRIDGE] = Blob { c: (0.0, 0.0), r: 0.0, pinned: true };
        blobs.extend_from_slice(&b);
    }
    // Only what still overlaps moves (the bridges are placed after, between their rings).
    let mut movable: Vec<Blob> = blobs.iter().copied().filter(|b| b.r > 0.0).collect();
    relax(&mut movable, 6.0, 70.0);
    let mut k = 0;
    for b in blobs.iter_mut() {
        if b.r > 0.0 {
            *b = movable[k];
            k += 1;
        }
    }
    let blob = |i: usize, role: usize| blobs[i * ROLES + role];

    let mut pos: HashMap<u16, (f32, f32)> = HashMap::new();
    pos.insert(START, (0.0, 0.0));
    for (i, r) in REGIONS.iter().enumerate() {
        let seed = 1000 + i as u32 * 7919 + if branches { 50_000 } else { 0 };
        let s = r.skill;
        let p = |at: u16| id(s, at);
        let n_extra = extras(ROOM[i]);
        let mut extra_ids = Vec::new();
        for e in 0..n_extra {
            let eid = p(EXTRA_FROM + e as u16);
            // Lane A, B, B, A: a region's main family never gets two (Melee went to 31 units).
            let fam = if e == 0 || e == 3 { r.a } else { r.b };
            let ring = if e >= 2 { 6.5 } else { 3.5 };
            nodes.push(Node {
                id: eid,
                kind: Kind::Small,
                region: Some(s),
                bridge: None,
                ring,
                x: 0.0,
                y: 0.0,
                gives: vec![(fam, 1.0)],
                special: None,
                key: format!("family.{}", fam.id()),
                needs: crate::perks::gate(ring),
                socket: None,
            });
            extra_ids.push(eid);
        }
        let ex = |k: usize| extra_ids.get(k).copied();
        let entry = blob(i, ENTRY).c;
        pos.insert(p(0), entry);
        // A ring: its in node facing where the lane comes from, its out node facing where it
        // goes, the notable on the arc between them on the outer side, extras round the other.
        let rr = ring_r[i];
        let (a1, b1, a2, b2) = (blob(i, RING_A1), blob(i, RING_B1), blob(i, RING_A2), blob(i, RING_B2));
        let meet = blob(i, MEET).c;
        if branches {
            // Each lane one smooth curve out through its two clusters to where the lanes meet,
            // its nodes along it at uneven steps; a travel node forks off the curve and rejoins
            // it two nodes on.
            let lane = |pts: [(f32, f32); 4], at: [u16; 6], k: u32| -> Vec<(u16, (f32, f32))> {
                let fr = [0.14, 0.3, 0.42, 0.54, 0.7, 0.84];
                at.iter()
                    .enumerate()
                    .map(|(q, &n)| (n, spline(&pts, fr[q] + 0.04 * sym(seed, k + q as u32))))
                    .collect()
            };
            for (n, xy) in lane([entry, a1.c, a2.c, meet], [p(1), p(3), p(5), p(7), p(10), p(12)], 60) {
                pos.insert(n, xy);
            }
            for (n, xy) in lane([entry, b1.c, b2.c, meet], [p(2), p(4), p(6), p(8), p(11), p(13)], 70) {
                pos.insert(n, xy);
            }
            for (e, (from, to, side)) in [(p(3), p(7), -1.0), (p(4), p(8), 1.0), (p(10), p(12), -1.0), (p(11), p(13), 1.0)]
                .into_iter()
                .enumerate()
            {
                let Some(eid) = ex(e) else { continue };
                let (fa, ta) = (pos[&from], pos[&to]);
                let (dx, dy) = (ta.0 - fa.0, ta.1 - fa.1);
                let l = dx.hypot(dy).max(1.0);
                let m = mid_bent(fa, ta, 0.0);
                // Out to the lane's own side, clockwise for B.
                let n = (-dy / l * side, dx / l * side);
                let off = 18.0 + 6.0 * rnd(seed, 80 + e as u32);
                pos.insert(eid, add(m, (n.0 * off, n.1 * off)));
                links.push((from, eid));
                links.push((eid, to));
            }
        } else {
            // A branch's cluster is a wide lens: the same ring, its centre pushed out sideways so its
            // arcs bend gently (fork and rejoin, no full circle).
            let lens = |c: (f32, f32), r: f32, from: (f32, f32), to: (f32, f32)| -> ((f32, f32), f32) {
                if !branches {
                    return (c, r);
                }
                let (dx, dy) = (to.0 - from.0, to.1 - from.1);
                let l = dx.hypot(dy).max(1.0);
                let big = r * 1.7;
                ((c.0 - dy / l * (big - r), c.1 + dx / l * (big - r)), big)
            };
            let ea: Vec<u16> = ex(0).into_iter().collect();
            let eb: Vec<u16> = ex(1).into_iter().collect();
            let (ca, ra) = lens(a1.c, rr[0], entry, a2.c);
            let mid_c = blob(i, MID).c;
            place_ring(&mut pos, links, arcs, (ca, ra), [p(3), p(5), p(7)], (entry, a2.c, mid_c), &ea, 0.12 * sym(seed, 40));
            let (cb, rb) = lens(b1.c, rr[1], entry, b2.c);
            place_ring(&mut pos, links, arcs, (cb, rb), [p(4), p(6), p(8)], (entry, b2.c, mid_c), &eb, 0.12 * sym(seed, 41));
            let meet = blob(i, MEET).c;
            let ea2: Vec<u16> = ex(2).into_iter().collect();
            let eb2: Vec<u16> = ex(3).into_iter().collect();
            let (ca2, ra2) = lens(a2.c, rr[2], pos[&p(7)], meet);
            // The outer rings have two template nodes (10 and 12, 11 and 13); the far side an extra.
            let a_in = angle_to(ca2, pos[&p(7)]);
            let a_out = angle_to(ca2, meet);
            pos.insert(p(10), on_ring(ca2, ra2, a_in));
            pos.insert(p(12), on_ring(ca2, ra2, a_out));
            arcs.insert(key(p(10), p(12)), ca2);
            for &e in &ea2 {
                pos.insert(e, on_ring(ca2, ra2, a_out + ((a_in - a_out).rem_euclid(TAU) - TAU) / 2.0 + PI));
                links.push((p(10), e));
                links.push((e, p(12)));
                arcs.insert(key(p(10), e), ca2);
                arcs.insert(key(e, p(12)), ca2);
            }
            let (cb2, rb2) = lens(b2.c, rr[3], pos[&p(8)], meet);
            let b_in = angle_to(cb2, pos[&p(8)]);
            let b_out = angle_to(cb2, meet);
            pos.insert(p(11), on_ring(cb2, rb2, b_in));
            pos.insert(p(13), on_ring(cb2, rb2, b_out));
            arcs.insert(key(p(11), p(13)), cb2);
            for &e in &eb2 {
                pos.insert(e, on_ring(cb2, rb2, b_out + ((b_in - b_out).rem_euclid(TAU) - TAU) / 2.0 + PI));
                links.push((p(11), e));
                links.push((e, p(13)));
                arcs.insert(key(p(11), e), cb2);
                arcs.insert(key(e, p(13)), cb2);
            }
        }
        // The paths in, bent a little; the link across and its socket; where the lanes meet;
        // the keystone at its uneven spot, on a gentle arc.
        pos.insert(p(1), mid_bent(entry, pos[&p(3)], 0.15 * sym(seed, 8)));
        pos.insert(p(2), mid_bent(entry, pos[&p(4)], 0.15 * sym(seed, 9)));
        let m = blob(i, MID).c;
        pos.insert(p(9), m);
        let l = m.0.hypot(m.1).max(1.0);
        pos.insert(p(crate::perks::REGION_SOCKET), add(m, (m.0 / l * 24.0, m.1 / l * 24.0)));
        pos.insert(p(14), meet);
        pos.insert(p(15), blob(i, KEY).c);
        arcs.insert(key(p(14), p(15)), bulge(meet, blob(i, KEY).c, 0.2 * sym(seed, 50)));
    }
    // Bridges: between the two clusters they join (this region's lane B, the next one's lane A),
    // the notable and its socket out from there.
    for (i, r) in REGIONS.iter().enumerate() {
        let next = REGIONS[(i + 1) % COUNT].skill;
        let (a, b) = (pos[&id(r.skill, 8)], pos[&id(next, 7)]);
        let m = mid_bent(a, b, 0.0);
        let l = m.0.hypot(m.1).max(1.0);
        let o = (m.0 / l, m.1 / l);
        let out = 6.0 + 10.0 * rnd(2000 + i as u32, 1);
        let small = add(m, (o.0 * out, o.1 * out));
        pos.insert(id(r.skill, crate::perks::BRIDGE_SMALL), small);
        let notable = add(small, (o.0 * 30.0, o.1 * 30.0));
        pos.insert(id(r.skill, crate::perks::BRIDGE_NOTABLE), notable);
        pos.insert(id(r.skill, crate::perks::BRIDGE_SOCKET), add(notable, (o.0 * 26.0, o.1 * 26.0)));
    }
    for n in nodes.iter() {
        if n.region.is_none() && n.socket.is_some() {
            // A centre socket: between the lane nodes it links, a little out.
            let near: Vec<(f32, f32)> = links
                .iter()
                .filter_map(|&(a, b)| if b == n.id { pos.get(&a).copied() } else { None })
                .collect();
            if near.len() == 2 {
                let m = mid_bent(near[0], near[1], 0.0);
                let l = m.0.hypot(m.1).max(1.0);
                pos.insert(n.id, add(m, (m.0 / l * 14.0, m.1 / l * 14.0)));
            }
        }
    }
    // A branch's straight links bend a little, each its own way.
    if branches {
        for (k, &(a, b)) in links.iter().enumerate() {
            if a == START || arcs.contains_key(&key(a, b)) {
                continue;
            }
            let (Some(&pa), Some(&pb)) = (pos.get(&a), pos.get(&b)) else { continue };
            arcs.insert(key(a, b), bulge(pa, pb, 0.12 * sym(3000, k as u32)));
        }
    }
    nudge(nodes, &mut pos, arcs);
    for n in nodes.iter_mut() {
        if let Some(&(x, y)) = pos.get(&n.id) {
            (n.x, n.y) = (x, y);
        }
    }
}

/// Nodes keep clear of each other at the screen's widest zoom (`MIN_ZOOM`): any two closer than
/// half their sizes and a gap are pushed apart. Nodes on a ring's arc stay on their ring; the
/// others move.
fn nudge(nodes: &[Node], pos: &mut HashMap<u16, (f32, f32)>, arcs: &Arcs) {
    let on_arc: std::collections::HashSet<u16> = arcs.keys().flat_map(|&(a, b)| [a, b]).collect();
    let size = |n: &Node| match n.kind {
        Kind::Small if n.id != START => 8.0,
        Kind::Keystone => 16.0,
        _ => 12.0,
    };
    let ids: Vec<(u16, f32, bool)> = nodes
        .iter()
        .map(|n| (n.id, size(n), n.id == START || on_arc.contains(&n.id)))
        .collect();
    for _ in 0..40 {
        let mut moved = false;
        for i in 0..ids.len() {
            for j in i + 1..ids.len() {
                let ((ia, sa, fa), (ib, sb, fb)) = (ids[i], ids[j]);
                if fa && fb {
                    continue;
                }
                let (Some(&a), Some(&b)) = (pos.get(&ia), pos.get(&ib)) else { continue };
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let d = dx.hypot(dy).max(1e-3);
                let want = ((sa + sb) / 2.0 + GAP) / MIN_ZOOM;
                if d >= want {
                    continue;
                }
                let push = want - d;
                let (ux, uy) = (dx / d, dy / d);
                let (wa, wb) = if fa {
                    (0.0, 1.0)
                } else if fb {
                    (1.0, 0.0)
                } else {
                    (0.5, 0.5)
                };
                pos.insert(ia, (a.0 - ux * push * wa, a.1 - uy * push * wa));
                pos.insert(ib, (b.0 + ux * push * wb, b.1 + uy * push * wb));
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
}

/// The screen's widest zoom (`mods/base/src/web.rs`), where nodes come closest.
pub const MIN_ZOOM: f32 = 0.55;
/// GUI px of air between two nodes there.
const GAP: f32 = 3.0;

/// A ring's nodes: its in node facing where the lane comes from, its out node facing where it
/// goes (at least a third of a turn apart, so both ways round show), the notable between them
/// (`bend` off the middle), extras round the other side.
#[allow(clippy::too_many_arguments)]
fn place_ring(
    pos: &mut HashMap<u16, (f32, f32)>,
    links: &mut Vec<(u16, u16)>,
    arcs: &mut Arcs,
    (c, rad): ((f32, f32), f32),
    at: [u16; 3],
    (from, to, toward): ((f32, f32), (f32, f32), (f32, f32)),
    extra: &[u16],
    bend: f32,
) {
    let a_in = angle_to(c, from);
    // In to out one way round or the other: the way whose middle (the notable) faces `toward`.
    let cw = (angle_to(c, to) - a_in)
        .rem_euclid(TAU)
        .clamp(TAU / 3.0, TAU * 2.0 / 3.0);
    let ccw = -(TAU - (angle_to(c, to) - a_in).rem_euclid(TAU)).clamp(TAU / 3.0, TAU * 2.0 / 3.0);
    let dist = |g: f32| {
        let p = on_ring(c, rad, a_in + g / 2.0);
        (p.0 - toward.0).hypot(p.1 - toward.1)
    };
    let gapa = if dist(cw) <= dist(ccw) { cw } else { ccw };
    let a_out = a_in + gapa;
    pos.insert(at[0], on_ring(c, rad, a_in));
    pos.insert(at[1], on_ring(c, rad, a_in + gapa * (0.5 + bend)));
    pos.insert(at[2], on_ring(c, rad, a_out));
    let rest = gapa.signum() * TAU - gapa;
    for (q, &e) in extra.iter().enumerate() {
        pos.insert(e, on_ring(c, rad, a_out + rest * (q as f32 + 1.0) / (extra.len() as f32 + 1.0)));
    }
    ring_links(&at, extra, c, links, arcs);
}

/// The links round a ring: the notable's side (`at`: in, notable, out) and the far side through
/// the extras, every link an arc of the ring.
fn ring_links(at: &[u16; 3], extra: &[u16], centre: (f32, f32), links: &mut Vec<(u16, u16)>, arcs: &mut Arcs) {
    arcs.insert(key(at[0], at[1]), centre);
    arcs.insert(key(at[1], at[2]), centre);
    let mut prev = at[2];
    for &e in extra {
        links.push((prev, e));
        arcs.insert(key(prev, e), centre);
        prev = e;
    }
    if !extra.is_empty() {
        links.push((prev, at[0]));
        arcs.insert(key(prev, at[0]), centre);
    }
}

/// The middle of a and b, pushed `bend` of their distance sideways.
fn mid_bent(a: (f32, f32), b: (f32, f32), bend: f32) -> (f32, f32) {
    let m = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    (m.0 - dy * bend, m.1 + dx * bend)
}

/// A point `t` (0..1) along a Catmull-Rom curve through four points (ends repeated).
fn spline(p: &[(f32, f32); 4], t: f32) -> (f32, f32) {
    let t = t.clamp(0.0, 1.0) * 3.0;
    let seg = (t.floor() as usize).min(2);
    let u = t - seg as f32;
    let get = |i: isize| p[i.clamp(0, 3) as usize];
    let (p0, p1, p2, p3) = (get(seg as isize - 1), get(seg as isize), get(seg as isize + 1), get(seg as isize + 2));
    let f = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b + (-a + c) * u + (2.0 * a - 5.0 * b + 4.0 * c - d) * u * u + (-a + 3.0 * b - 3.0 * c + d) * u * u * u)
    };
    (f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1))
}

/// An island's size for a region's room: an oval, long outwards (`along`, 150 to 190 GUI px from
/// its middle) and narrower across (`across`, 80 to 105), and its rings' radius (30 to 36:
/// readable at the screen's opening zoom).
fn island_size(room: f32) -> (f32, f32, f32) {
    let t = ((room - 0.7) / 0.7).clamp(0.0, 1.0);
    (150.0 + 40.0 * t, 80.0 + 25.0 * t, 30.0 + 6.0 * t)
}
/// GUI px of empty space between islands, and how far out the hub's gates sit.
const ISLAND_GAP: f32 = 46.0;
const GATE_R: f32 = 70.0;

/// The islands: each region's nodes inside its own oval, the ovals in one ring round the start
/// hub (so every island faces the hub and its keystone the rim, and nothing stands between a
/// road and its island), empty space between them, bridges and the centre sockets in the gaps.
fn islands(nodes: &mut Vec<Node>, links: &mut Vec<(u16, u16)>, arcs: &mut Arcs) {
    // The ring: each island's share of the way round is its width and a gap.
    let sizes: Vec<(f32, f32, f32)> = ROOM.iter().map(|&r| island_size(r)).collect();
    let round: f32 = sizes.iter().map(|s| 2.0 * s.1 + ISLAND_GAP).sum();
    let ring_r = round / TAU;
    let mut discs: Vec<Blob> = Vec::new();
    let mut at_angle = -(2.0 * sizes[0].1 + ISLAND_GAP) / 2.0 / ring_r;
    for (i, &(along, across, _)) in sizes.iter().enumerate() {
        let span = (2.0 * across + ISLAND_GAP) / ring_r;
        let mid = at_angle + span / 2.0;
        at_angle += span;
        // Out a little more or less each (uneven), the middle of the oval past the ring.
        let d = ring_r + along * 0.55 + 28.0 * sym(4000 + i as u32, 1);
        discs.push(Blob { c: (d * mid.sin(), -d * mid.cos()), r: across, pinned: false });
    }
    let mut pos: HashMap<u16, (f32, f32)> = HashMap::new();
    pos.insert(START, (0.0, 0.0));
    for (i, r) in REGIONS.iter().enumerate() {
        let seed = 7000 + i as u32 * 7919;
        let s = r.skill;
        let p = |at: u16| id(s, at);
        let room = ROOM[i];
        let c = discs[i].c;
        let (along, across, rr) = sizes[i];
        // The island's frame: `a` out from the start, `b` across (clockwise), both in its radii.
        let l = c.0.hypot(c.1).max(1.0);
        let (ax, ay) = (c.0 / l, c.1 / l);
        let (bx, by) = (-ay, ax);
        let at = |a: f32, b: f32| (c.0 + ax * a * along + bx * b * across, c.1 + ay * a * along + by * b * across);
        let j = |k: u32| 0.06 * sym(seed, k);
        // Travel nodes, as the other styles make them (lane A, B, B, A).
        let mut extra_ids = Vec::new();
        for e in 0..extras(room) {
            let eid = p(EXTRA_FROM + e as u16);
            let fam = if e == 0 || e == 3 { r.a } else { r.b };
            let ring = if e >= 2 { 6.5 } else { 3.5 };
            nodes.push(Node {
                id: eid,
                kind: Kind::Small,
                region: Some(s),
                bridge: None,
                ring,
                x: 0.0,
                y: 0.0,
                gives: vec![(fam, 1.0)],
                special: None,
                key: format!("family.{}", fam.id()),
                needs: crate::perks::gate(ring),
                socket: None,
            });
            extra_ids.push(eid);
        }
        let ex = |k: usize| -> Vec<u16> { extra_ids.get(k).copied().into_iter().collect() };
        let entry = at(-0.9, j(1));
        pos.insert(p(0), entry);
        let a1 = at(-0.45 + j(2), -0.5 + j(3));
        let b1 = at(-0.2 + j(4), 0.52 + j(5));
        let mid = at(0.06 + j(6), j(7));
        let a2 = at(0.34 + j(8), -0.48 + j(9));
        let b2 = at(0.56 + j(10), 0.46 + j(11));
        let meet = at(0.84 + j(12), 0.12 * sym(seed, 13));
        place_ring(&mut pos, links, arcs, (a1, rr), [p(3), p(5), p(7)], (entry, a2, mid), &ex(0), 0.1 * sym(seed, 14));
        place_ring(&mut pos, links, arcs, (b1, rr - 2.0), [p(4), p(6), p(8)], (entry, b2, mid), &ex(1), 0.1 * sym(seed, 15));
        pos.insert(p(1), mid_bent(entry, pos[&p(3)], 0.12 * sym(seed, 16)));
        pos.insert(p(2), mid_bent(entry, pos[&p(4)], 0.12 * sym(seed, 17)));
        pos.insert(p(9), mid);
        pos.insert(p(crate::perks::REGION_SOCKET), at(0.2 + j(18), 0.02));
        // The outer rings: two template nodes each (10 and 12, 11 and 13), an extra the far way.
        for (centre, rad, from, to, (n_in, n_out), extra) in
            [(a2, rr - 4.0, pos[&p(7)], meet, (p(10), p(12)), ex(2)), (b2, rr - 4.0, pos[&p(8)], meet, (p(11), p(13)), ex(3))]
        {
            let a_in = angle_to(centre, from);
            let a_out = a_in
                + (angle_to(centre, to) - a_in)
                    .rem_euclid(TAU)
                    .clamp(TAU / 3.0, TAU * 2.0 / 3.0);
            pos.insert(n_in, on_ring(centre, rad, a_in));
            pos.insert(n_out, on_ring(centre, rad, a_out));
            arcs.insert(key(n_in, n_out), centre);
            for &e in &extra {
                pos.insert(e, on_ring(centre, rad, a_out + (TAU - (a_out - a_in)) / 2.0));
                links.push((n_in, e));
                links.push((e, n_out));
                arcs.insert(key(n_in, e), centre);
                arcs.insert(key(e, n_out), centre);
            }
        }
        pos.insert(p(14), meet);
        // The keystone just past the island's outer edge, off to one side.
        let k = at(1.06 + 0.06 * rnd(seed, 19), (0.15 + 0.3 * rnd(seed, 20)) * if rnd(seed, 21) < 0.5 { -1.0 } else { 1.0 });
        pos.insert(p(15), k);
        arcs.insert(key(p(14), p(15)), bulge(meet, k, 0.2 * sym(seed, 22)));
    }
    // Bridges across the gap to the next island; the centre sockets between two islands' lanes.
    for (i, r) in REGIONS.iter().enumerate() {
        let next = REGIONS[(i + 1) % COUNT].skill;
        let (a, b) = (pos[&id(r.skill, 8)], pos[&id(next, 7)]);
        let m = mid_bent(a, b, 0.0);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = dx.hypot(dy).max(1.0);
        // Out from the web's middle, across the bridge.
        let o = {
            let q = m.0.hypot(m.1).max(1.0);
            (m.0 / q, m.1 / q)
        };
        let _ = l;
        pos.insert(id(r.skill, crate::perks::BRIDGE_SMALL), m);
        let notable = add(m, (o.0 * 28.0, o.1 * 28.0));
        pos.insert(id(r.skill, crate::perks::BRIDGE_NOTABLE), notable);
        pos.insert(id(r.skill, crate::perks::BRIDGE_SOCKET), add(notable, (o.0 * 26.0, o.1 * 26.0)));
    }
    // The junctions: inside the ring, facing the middle of their islands.
    // (entries sit about 60 px inside the ring: the junctions well inside them)
    let inner = ring_r - 150.0;
    for (j, run) in crate::perks::junctions().iter().enumerate() {
        let (mut sx, mut sy) = (0.0, 0.0);
        for &i in run {
            let c = discs[i].c;
            let l = c.0.hypot(c.1).max(1.0);
            sx += c.0 / l;
            sy += c.1 / l;
        }
        let l = sx.hypot(sy).max(1e-3);
        pos.insert(crate::perks::JUNCTION_FROM + j as u16, (sx / l * inner, sy / l * inner));
    }
    // The hub: each gate towards its regions (the middle of their islands' directions).
    for (g, &(group, _)) in crate::perks::GATES_OF.iter().enumerate() {
        let (mut sx, mut sy) = (0.0, 0.0);
        for (i, r) in REGIONS.iter().enumerate() {
            if r.skill.group() == group {
                let c = discs[i].c;
                let l = c.0.hypot(c.1).max(1.0);
                sx += c.0 / l;
                sy += c.1 / l;
            }
        }
        let l = sx.hypot(sy).max(1e-3);
        let _ = g;
        pos.insert(crate::perks::gate_id(group), (sx / l * GATE_R, sy / l * GATE_R));
    }
    for n in nodes.iter() {
        if n.region.is_none() && n.socket.is_some() {
            let near: Vec<(f32, f32)> = links
                .iter()
                .filter_map(|&(a, b)| if b == n.id { pos.get(&a).copied() } else { None })
                .collect();
            if near.len() == 2 {
                pos.insert(n.id, mid_bent(near[0], near[1], 0.0));
            }
        }
    }
    // A gate's road out to an island's entry: straight if nothing is in the way, else an arc
    // through the gap between the islands it would cross (the outer islands sit behind two inner
    // ones).
    for &(a, b) in links.iter() {
        let road = (crate::perks::GATE_FROM..crate::perks::JUNCTION_FROM + 8).contains(&a);
        if !road {
            continue;
        }
        let (Some(&pa), Some(&pb)) = (pos.get(&a), pos.get(&b)) else { continue };
        let Some(own) = REGIONS.iter().position(|r| id(r.skill, 0) == b) else { continue };
        let blocked: Vec<usize> = (0..COUNT)
            .filter(|&k| k != own && crosses(pa, pb, discs[k]))
            .collect();
        if blocked.is_empty() {
            continue;
        }
        // The gap: between the blocking island and its nearer neighbour on the road's side.
        let k = blocked[0];
        let side = |q: usize| {
            let c = discs[q].c;
            (pb.0 - pa.0) * (c.1 - pa.1) - (pb.1 - pa.1) * (c.0 - pa.0)
        };
        let other = if side(k) > 0.0 { (k + COUNT - 1) % COUNT } else { (k + 1) % COUNT };
        let (d1, d2) = (discs[k], discs[other]);
        let (dx, dy) = (d2.c.0 - d1.c.0, d2.c.1 - d1.c.1);
        let l = dx.hypot(dy).max(1.0);
        // The middle of the empty space between the two discs' edges.
        let t = (d1.r + (l - d1.r - d2.r) / 2.0) / l;
        let gap = (d1.c.0 + dx * t, d1.c.1 + dy * t);
        if let Some(c) = circle(pa, gap, pb) {
            arcs.insert(key(a, b), c);
        }
    }
    nudge(nodes, &mut pos, arcs);
    for n in nodes.iter_mut() {
        if let Some(&(x, y)) = pos.get(&n.id) {
            (n.x, n.y) = (x, y);
        }
    }
}

/// Whether the segment from a to b passes through a disc.
fn crosses(a: (f32, f32), b: (f32, f32), d: Blob) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = (dx * dx + dy * dy).max(1e-3);
    let t = (((d.c.0 - a.0) * dx + (d.c.1 - a.1) * dy) / len2).clamp(0.0, 1.0);
    let (x, y) = (a.0 + dx * t, a.1 + dy * t);
    (x - d.c.0).hypot(y - d.c.1) < d.r && t > 0.02 && t < 0.98
}

/// The centre of the circle through three points (none if they're in a line).
fn circle(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> Option<(f32, f32)> {
    let d = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
    if d.abs() < 1e-3 {
        return None;
    }
    let (a2, b2, c2) = (a.0 * a.0 + a.1 * a.1, b.0 * b.0 + b.1 * b.1, c.0 * c.0 + c.1 * c.1);
    Some(((a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / d, (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / d))
}
