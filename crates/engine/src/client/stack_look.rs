//! A stack's own picture (`abi::comp::LOOK`): layers back to front, each a painted texture or a
//! shade map painted with ramps, composed into one image. Items built from parts show what they
//! are made of without a texture per combination.

/// The picture of `look`, at the size of its first layer: (size, RGBA). `read` gives a texture's
/// width and pixels by name (a strip's first frame is used, a ramp's first row). `None` when no
/// layer could be read.
pub fn compose(look: &str, read: impl Fn(&str) -> Option<(u32, Vec<u8>)>) -> Option<(u32, Vec<u8>)> {
    let mut out: Option<(u32, Vec<u8>)> = None;
    for layer in look.split(';').map(str::trim).filter(|l| !l.is_empty()) {
        // `+dx,dy` at the end moves the layer that many pixels (a handle meeting its head).
        let (layer, shift) = match layer
            .rsplit_once('+')
            .and_then(|(l, at)| Some((l, parse_shift(at)?)))
        {
            Some((l, at)) => (l, at),
            None => (layer, [0, 0]),
        };
        let (name, ramps) = match layer.split_once('@') {
            Some((map, ramps)) => (map, Some(ramps)),
            None => (layer, None),
        };
        let Some((w, px)) = read(name) else {
            log::warn!("look {look}: no texture {name}");
            continue;
        };
        let bytes = (w * w * 4) as usize;
        if w == 0 || px.len() < bytes {
            log::warn!("look {look}: {name} is not square");
            continue;
        }
        let (size, canvas) = out.get_or_insert_with(|| (w, vec![0; bytes]));
        // Each look is composed once, so this warns once per look.
        if *size != w {
            log::warn!("look {look}: {name} is {w} wide, the first layer {size}");
            continue;
        }
        let px = &px[..bytes];
        // Where pixel `i` of this layer lands on the canvas, if it's on it.
        let at = |i: usize| -> Option<usize> {
            let (x, y) = ((i as u32 % w) as i32 + shift[0], (i as u32 / w) as i32 + shift[1]);
            ((0..w as i32).contains(&x) && (0..w as i32).contains(&y)).then(|| (y as u32 * w + x as u32) as usize * 4)
        };
        let Some(ramps) = ramps else {
            for (i, p) in px.chunks_exact(4).enumerate() {
                if let (true, Some(o)) = (p[3] > 0, at(i)) {
                    canvas[o..o + 4].copy_from_slice(p);
                }
            }
            continue;
        };
        let ramps: Vec<Option<(u32, Vec<u8>)>> = ramps
            .split(',')
            .map(|r| {
                let r = r.trim();
                let ramp = read(r).filter(|(rw, rp)| *rw > 0 && rp.len() >= (*rw * 4) as usize);
                if ramp.is_none() {
                    log::warn!("look {look}: no ramp {r}");
                }
                ramp
            })
            .collect();
        for (i, p) in px.chunks_exact(4).enumerate() {
            let Some(o) = at(i).filter(|_| p[3] != 0) else { continue };
            // A region without a ramp leaves what is below.
            let Some(Some((rw, rp))) = ramps.get(p[1] as usize) else { continue };
            let x = (p[0] as u32 / 36).min(7).min(rw - 1) as usize;
            // The ramp gives the colour, the map where the layer is.
            canvas[o..o + 4].copy_from_slice(&[rp[x * 4], rp[x * 4 + 1], rp[x * 4 + 2], p[3]]);
        }
    }
    out
}

/// `dx,dy` in whole pixels.
fn parse_shift(at: &str) -> Option<[i32; 2]> {
    let (x, y) = at.split_once(',')?;
    Some([x.trim().parse().ok()?, y.trim().parse().ok()?])
}

#[cfg(test)]
mod tests {
    use super::compose;

    #[test]
    fn a_layer_moves() {
        // One pixel at (0, 0), moved one right and one down; off the canvas it's dropped.
        let one = vec![9, 9, 9, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let read = |n: &str| (n == "t:one").then(|| (2, one.clone()));
        let (_, px) = compose("t:one+1,1", read).unwrap();
        assert_eq!(&px[12..16], &[9, 9, 9, 255]);
        assert_eq!(&px[0..4], &[0, 0, 0, 0]);
        let (_, px) = compose("t:one+-1,0", read).unwrap();
        assert!(px.iter().all(|&b| b == 0));
    }

    #[test]
    fn layers_paint_over_each_other() {
        // A 2x2 map: step 0 region 0, step 7 region 1, a step past 7, and nothing.
        let map = vec![0, 0, 0, 255, 252, 1, 0, 255, 255, 0, 0, 255, 0, 0, 0, 0];
        let ramp = |c: u8| (8, (0..8u8).flat_map(|i| [c, i, 0, 255]).collect::<Vec<u8>>());
        let top = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 255];
        let read = |n: &str| match n {
            "t:map" => Some((2, map.clone())),
            "t:oak" => Some(ramp(10)),
            "t:iron" => Some(ramp(20)),
            "t:top" => Some((2, top.clone())),
            "t:big" => Some((4, vec![255; 64])),
            _ => None,
        };
        let (size, px) = compose("t:map@t:oak,t:iron;t:big;t:top", read).unwrap();
        assert_eq!(size, 2);
        assert_eq!(&px[0..4], &[10, 0, 0, 255]);
        assert_eq!(&px[4..8], &[20, 7, 0, 255]);
        assert_eq!(&px[8..12], &[10, 7, 0, 255]);
        assert_eq!(&px[12..16], &[9, 9, 9, 255]);
        // A missing ramp skips its region.
        let (_, px) = compose("t:map@t:oak", read).unwrap();
        assert_eq!(&px[4..8], &[0, 0, 0, 0]);
        assert!(compose("t:nothing", read).is_none());
    }
}
