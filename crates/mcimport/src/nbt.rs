//! Minecraft's NBT, as much as chunks need: big-endian tags, a root compound. Lists may be
//! heterogeneous (Minecraft 1.21.5+ wraps mixed elements in compounds with one empty-named
//! field), which `Tag::unwrap_list_item` takes off again.

#[derive(Debug, Clone)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bytes(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<(String, Tag)>),
    Ints(Vec<i32>),
    Longs(Vec<i64>),
}

impl Tag {
    pub fn get(&self, key: &str) -> Option<&Tag> {
        match self {
            Tag::Compound(f) => f.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn int(&self) -> Option<i64> {
        match self {
            Tag::Byte(v) => Some(*v as i64),
            Tag::Short(v) => Some(*v as i64),
            Tag::Int(v) => Some(*v as i64),
            Tag::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn list(&self) -> &[Tag] {
        match self {
            Tag::List(l) => l,
            _ => &[],
        }
    }

    pub fn longs(&self) -> &[i64] {
        match self {
            Tag::Longs(l) => l,
            _ => &[],
        }
    }

    pub fn fields(&self) -> &[(String, Tag)] {
        match self {
            Tag::Compound(f) => f,
            _ => &[],
        }
    }

    /// A heterogeneous list's element without its wrapper (`{"": value}`).
    pub fn unwrap_list_item(&self) -> &Tag {
        match self {
            Tag::Compound(f) if f.len() == 1 && f[0].0.is_empty() => &f[0].1,
            t => t,
        }
    }
}

struct R<'a> {
    b: &'a [u8],
    i: usize,
}

impl R<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let s = self.b.get(self.i..self.i + n)?;
        self.i += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn i16(&mut self) -> Option<i16> {
        Some(i16::from_be_bytes(self.take(2)?.try_into().ok()?))
    }
    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_be_bytes(self.take(4)?.try_into().ok()?))
    }
    fn i64(&mut self) -> Option<i64> {
        Some(i64::from_be_bytes(self.take(8)?.try_into().ok()?))
    }
    fn len(&mut self) -> Option<usize> {
        usize::try_from(self.i32()?).ok()
    }
    fn string(&mut self) -> Option<String> {
        let n = self.i16()? as u16 as usize;
        // Modified UTF-8; chunk names are ASCII.
        Some(String::from_utf8_lossy(self.take(n)?).into_owned())
    }

    fn tag(&mut self, t: u8, depth: u32) -> Option<Tag> {
        if depth > 64 {
            return None;
        }
        Some(match t {
            1 => Tag::Byte(self.u8()? as i8),
            2 => Tag::Short(self.i16()?),
            3 => Tag::Int(self.i32()?),
            4 => Tag::Long(self.i64()?),
            5 => Tag::Float(f32::from_bits(self.i32()? as u32)),
            6 => Tag::Double(f64::from_bits(self.i64()? as u64)),
            7 => {
                let n = self.len()?;
                Tag::Bytes(self.take(n)?.to_vec())
            }
            8 => Tag::String(self.string()?),
            9 => {
                let et = self.u8()?;
                let n = self.len()?;
                let mut out = Vec::with_capacity(n.min(1 << 16));
                for _ in 0..n {
                    out.push(self.tag(et, depth + 1)?);
                }
                Tag::List(out)
            }
            10 => {
                let mut f = Vec::new();
                loop {
                    let et = self.u8()?;
                    if et == 0 {
                        break;
                    }
                    let k = self.string()?;
                    f.push((k, self.tag(et, depth + 1)?));
                }
                Tag::Compound(f)
            }
            11 => {
                let n = self.len()?;
                let raw = self.take(n.checked_mul(4)?)?;
                Tag::Ints(
                    raw.as_chunks::<4>()
                        .0
                        .iter()
                        .map(|c| i32::from_be_bytes(*c))
                        .collect(),
                )
            }
            12 => {
                let n = self.len()?;
                let raw = self.take(n.checked_mul(8)?)?;
                Tag::Longs(
                    raw.as_chunks::<8>()
                        .0
                        .iter()
                        .map(|c| i64::from_be_bytes(*c))
                        .collect(),
                )
            }
            _ => return None,
        })
    }
}

/// The root tag of an uncompressed NBT document (its name dropped).
pub fn parse(b: &[u8]) -> Option<Tag> {
    let mut r = R { b, i: 0 };
    let t = r.u8()?;
    r.string()?;
    r.tag(t, 0)
}
