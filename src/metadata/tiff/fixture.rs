//! Test-only TIFF block builder (SPEC-093).
//!
//! This exists because every pre-SPEC-093 fixture was seeded by calling
//! [`super::serialize`] — the very writer under test — which could only ever
//! emit little-endian blocks. A suite whose fixtures come from the code under
//! test cannot observe that code misreading a byte order, and a suite with no
//! big-endian fixture cannot exercise a byte-order bug at all. That is exactly
//! how the numeric-tag corruption shipped green for a month.
//!
//! So this builder is deliberately **independent** of `serialize`: it takes
//! *typed* values ([`V`]) and encodes them itself in the requested byte order.
//! It is the oracle, so it must not share the writer's assumptions.

/// A typed TIFF field value. [`build_tiff`] encodes each of these in the
/// target byte order — which is the whole point: the caller says
/// `V::Short(6)`, not "these two bytes", so a fixture cannot silently inherit
/// the writer's idea of what byte order means.
#[derive(Debug, Clone)]
pub(crate) enum V {
    /// Type 3, count 1 — e.g. Orientation. Inline (2 bytes).
    Short(u16),
    /// Type 4, count 1 — e.g. a plain LONG. Inline (4 bytes).
    Long(u32),
    /// Type 2 — UTF-8 + trailing NUL. Byte-order-immune (this is why
    /// ASCII-only fixtures never caught the bug).
    Ascii(&'static str),
    /// Type 5, count 1 — numerator/denominator. Out-of-line (8 bytes).
    Rational(u32, u32),
    /// Type 5, count 3 — e.g. a GPS coordinate (deg/min/sec). Out-of-line.
    RationalTriplet([(u32, u32); 3]),
    /// Type 7 — opaque bytes. Byte-order-immune.
    Undefined(&'static [u8]),
    /// A pointer tag (ExifIFD/GPS/Interop): type 4, value = sub-IFD offset.
    Sub(Vec<(u16, V)>),
}

fn u16_at(buf: &mut [u8], at: usize, le: bool, v: u16) {
    let b = if le { v.to_le_bytes() } else { v.to_be_bytes() };
    buf[at..at + 2].copy_from_slice(&b);
}

fn u32_at(buf: &mut [u8], at: usize, le: bool, v: u32) {
    let b = if le { v.to_le_bytes() } else { v.to_be_bytes() };
    buf[at..at + 4].copy_from_slice(&b);
}

fn u32_bytes(v: u32, le: bool) -> [u8; 4] {
    if le {
        v.to_le_bytes()
    } else {
        v.to_be_bytes()
    }
}

/// Encode one value to `(type, count, value-bytes)` in byte order `le`.
fn encode(v: &V, le: bool) -> (u16, u32, Vec<u8>) {
    match v {
        V::Short(x) => (
            3,
            1,
            if le {
                x.to_le_bytes().to_vec()
            } else {
                x.to_be_bytes().to_vec()
            },
        ),
        V::Long(x) => (4, 1, u32_bytes(*x, le).to_vec()),
        V::Ascii(s) => {
            let mut b = s.as_bytes().to_vec();
            b.push(0);
            let count = b.len() as u32;
            (2, count, b)
        }
        V::Rational(n, d) => {
            let mut b = Vec::with_capacity(8);
            b.extend_from_slice(&u32_bytes(*n, le));
            b.extend_from_slice(&u32_bytes(*d, le));
            (5, 1, b)
        }
        V::RationalTriplet(parts) => {
            let mut b = Vec::with_capacity(24);
            for (n, d) in parts {
                b.extend_from_slice(&u32_bytes(*n, le));
                b.extend_from_slice(&u32_bytes(*d, le));
            }
            (5, 3, b)
        }
        V::Undefined(x) => (7, x.len() as u32, x.to_vec()),
        // Placeholder — build_ifd patches the real sub-IFD offset in.
        V::Sub(_) => (4, 1, vec![0u8; 4]),
    }
}

/// Append one IFD (directory + out-of-line values + sub-IFDs) to `buf`,
/// encoding every field in byte order `le`.
fn build_ifd(buf: &mut Vec<u8>, le: bool, entries: &[(u16, V)]) {
    let dir_at = buf.len();
    buf.resize(dir_at + 2, 0);
    u16_at(buf, dir_at, le, entries.len() as u16);

    let dir_start = dir_at + 2;
    // Directory (12 bytes/entry) + the 4-byte next-IFD slot, zero-filled.
    buf.resize(dir_start + entries.len() * 12 + 4, 0);

    for (i, (tag, v)) in entries.iter().enumerate() {
        let slot = dir_start + i * 12;
        let (ty, count, bytes) = encode(v, le);
        u16_at(buf, slot, le, *tag);
        u16_at(buf, slot + 2, le, ty);
        u32_at(buf, slot + 4, le, count);

        match v {
            V::Sub(sub) => {
                let sub_at = buf.len();
                build_ifd(buf, le, sub);
                u32_at(buf, slot + 8, le, sub_at as u32);
            }
            _ if bytes.len() <= 4 => {
                // TIFF 6.0 §2: short values are left-justified in the slot.
                buf[slot + 8..slot + 8 + bytes.len()].copy_from_slice(&bytes);
            }
            _ => {
                if buf.len() % 2 == 1 {
                    buf.push(0);
                }
                let value_at = buf.len();
                buf.extend_from_slice(&bytes);
                u32_at(buf, slot + 8, le, value_at as u32);
            }
        }
    }
}

fn header(le: bool) -> Vec<u8> {
    if le {
        vec![b'I', b'I', 42, 0, 8, 0, 0, 0]
    } else {
        vec![b'M', b'M', 0, 42, 0, 0, 0, 8]
    }
}

/// Build a complete bare TIFF/EXIF block (the shape `img-parts` exposes via
/// `ImageEXIF::exif()`) whose IFD0 holds `entries`, in byte order `le`.
pub(crate) fn build_tiff(le: bool, entries: &[(u16, V)]) -> Vec<u8> {
    let mut buf = header(le);
    build_ifd(&mut buf, le, entries);
    buf
}

/// As [`build_tiff`], plus an IFD1 (the thumbnail directory) carrying `thumb`
/// located by `JPEGInterchangeFormat` (0x0201) / `…Length` (0x0202).
///
/// Worth its own fixture: the thumbnail's *length* is a LONG the serializer
/// carries over verbatim, so a mis-declared byte order turns a 6,430-byte
/// thumbnail into a 504,954,880-byte one and leaves the pointer dangling.
pub(crate) fn build_tiff_with_thumbnail(le: bool, entries: &[(u16, V)], thumb: &[u8]) -> Vec<u8> {
    let mut buf = header(le);
    build_ifd(&mut buf, le, entries);

    // Patch IFD0's next-IFD slot (zeroed by build_ifd) to point at IFD1.
    let ifd0_dir_start = 8 + 2;
    let next_slot = ifd0_dir_start + entries.len() * 12;
    if buf.len() % 2 == 1 {
        buf.push(0);
    }
    let ifd1_at = buf.len();
    u32_at(&mut buf, next_slot, le, ifd1_at as u32);

    let ifd1 = [
        (0x0201u16, V::Long(0)), // offset — patched once the blob lands
        (0x0202u16, V::Long(thumb.len() as u32)),
    ];
    build_ifd(&mut buf, le, &ifd1);

    let ifd1_dir_start = ifd1_at + 2;
    if buf.len() % 2 == 1 {
        buf.push(0);
    }
    let thumb_at = buf.len();
    buf.extend_from_slice(thumb);
    u32_at(&mut buf, ifd1_dir_start + 8, le, thumb_at as u32);

    buf
}
