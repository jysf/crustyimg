//! A minimal, bounded, panic-free binary TIFF-IFD reader + normalizing
//! little-endian writer (SPEC-045, DEC-046).
//!
//! This replaces `little_exif` for the two tag-level edits the container lane
//! needs: [`crate::metadata::set_tags`] (add/replace IFD0 ASCII tags) and
//! [`crate::metadata::clean_gps`] (drop the GPS sub-IFD pointer). It operates
//! on the **bare TIFF block** that `img-parts` exposes via `ImageEXIF::exif()`
//! (`II*\0`/`MM\0*`, no `"Exif\0\0"` prefix) and re-serializes it for
//! `set_exif()`.
//!
//! The EXIF block is attacker-influenced (untrusted input, DEC-034/036/038),
//! so **every** offset/length/index read here is bounds-checked and returns
//! [`Error`] instead of panicking. Sub-IFD recursion (`ExifIFD`/`GPS`/
//! `Interop` pointer tags) and the IFD0→IFD1 chain are both depth/cycle
//! guarded ([`MAX_IFD_DEPTH`]) to kill cycles built from self-referential
//! offsets.
//!
//! Only what `set`/`clean --gps` need is modeled: every entry's raw
//! `(tag, type, count, value-bytes)` is preserved opaquely (unknown tags
//! round-trip verbatim); pointer tags additionally carry a parsed sub-[`Ifd`].
//! IFD0's next-IFD link (IFD1 — the thumbnail directory) is followed, and
//! IFD1's thumbnail blob (`JPEGInterchangeFormat` 0x0201 / `…Length` 0x0202)
//! is captured as [`Ifd::thumbnail`] and relocated on serialize.

use std::collections::HashSet;

/// A tag-level error from the TIFF-IFD parser or serializer. Every variant
/// maps to `MetadataError::Exif` in `src/metadata/mod.rs`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("TIFF block too short ({0} bytes)")]
    Truncated(usize),
    #[error("bad TIFF byte-order marker")]
    BadByteOrder,
    #[error("bad TIFF magic number")]
    BadMagic,
    #[error("IFD offset/length out of bounds")]
    OutOfBounds,
    #[error("sub-IFD recursion exceeded depth {0} (cyclic or malicious offsets)")]
    RecursionLimit(usize),
    #[error("integer overflow while computing an offset/length")]
    Overflow,
}

type Result<T> = std::result::Result<T, Error>;

/// Cap on IFD recursion/chaining (sub-IFDs *and* the IFD0→IFD1 next-link),
/// so a self-referential or deeply nested offset chain can't blow the stack
/// or spin forever. Real EXIF nests at most 2-3 deep (IFD0 → ExifIFD →
/// Interop) plus one IFD1; 8 is a generous, still-bounded ceiling.
pub const MAX_IFD_DEPTH: usize = 8;

/// Pointer tags whose value is a `LONG` offset to a sub-IFD.
pub const EXIF_PTR: u16 = 0x8769;
pub const GPS_PTR: u16 = 0x8825;
pub const INTEROP_PTR: u16 = 0xA005;

/// IFD0 ASCII attribution tags `set_tags` writes (SPEC-027).
pub const TAG_IMAGE_DESCRIPTION: u16 = 0x010E;
pub const TAG_ARTIST: u16 = 0x013B;
pub const TAG_COPYRIGHT: u16 = 0x8298;

/// IFD1 thumbnail location tags (TIFF 6.0 / EXIF 2.3 §4.6.5).
const TAG_THUMB_OFFSET: u16 = 0x0201;
const TAG_THUMB_LENGTH: u16 = 0x0202;

/// TIFF field type codes referenced by name for clarity at call sites.
const TY_LONG: u16 = 4;
const TY_ASCII: u16 = 2;

/// Byte width of one element of TIFF field type `ty` (TIFF 6.0 §2):
/// 1=BYTE 2=ASCII 3=SHORT 4=LONG 5=RATIONAL 6=SBYTE 7=UNDEFINED 8=SSHORT
/// 9=SLONG 10=SRATIONAL 11=FLOAT 12=DOUBLE. Unknown codes are treated as
/// byte-sized: `vlen` is still computed from `count` and every read of it is
/// bounds-checked, so an unrecognized type can't cause an OOB read — worst
/// case we mis-size an opaque blob that we pass through verbatim anyway.
fn type_size(ty: u16) -> usize {
    match ty {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => 1,
    }
}

/// One IFD entry, kept as an opaque `(tag, type, count, value-bytes)` tuple
/// so unknown tags round-trip byte-for-byte. Pointer tags additionally carry
/// their parsed sub-IFD in `sub` (the raw `value` is ignored on serialize
/// for these — the offset is recomputed from where `sub` ends up).
#[derive(Debug, Clone)]
pub struct Entry {
    pub tag: u16,
    pub ty: u16,
    pub count: u32,
    /// The raw value bytes (length = `type_size(ty) * count`).
    pub value: Vec<u8>,
    /// Parsed sub-IFD for pointer tags (`EXIF_PTR`/`GPS_PTR`/`INTEROP_PTR`).
    pub sub: Option<Box<Ifd>>,
}

/// One IFD (directory): its entries, an optional next-IFD link (IFD0's
/// `next` is IFD1, the thumbnail directory), and — only meaningful on IFD1
/// — the thumbnail bytes pointed to by `0x0201`/`0x0202`, extracted during
/// parse so they can be relocated (not left dangling at a stale offset) on
/// serialize.
#[derive(Debug, Clone, Default)]
pub struct Ifd {
    pub entries: Vec<Entry>,
    pub next: Option<Box<Ifd>>,
    pub thumbnail: Option<Vec<u8>>,
}

// ── Bounds-checked byte readers ─────────────────────────────────────────────

fn get_slice(buf: &[u8], start: usize, len: usize) -> Result<&[u8]> {
    let end = start.checked_add(len).ok_or(Error::Overflow)?;
    buf.get(start..end).ok_or(Error::OutOfBounds)
}

fn read_u16(buf: &[u8], at: usize, le: bool) -> Result<u16> {
    let s = get_slice(buf, at, 2)?;
    Ok(if le {
        u16::from_le_bytes([s[0], s[1]])
    } else {
        u16::from_be_bytes([s[0], s[1]])
    })
}

fn read_u32(buf: &[u8], at: usize, le: bool) -> Result<u32> {
    let s = get_slice(buf, at, 4)?;
    Ok(if le {
        u32::from_le_bytes([s[0], s[1], s[2], s[3]])
    } else {
        u32::from_be_bytes([s[0], s[1], s[2], s[3]])
    })
}

/// Decode a 4-byte inline/offset value slot as a `u32` per the TIFF header's
/// byte order. Used both for real offsets and for reading a LONG value that
/// happens to be stored inline (`vlen == 4`).
fn value_as_u32(bytes: &[u8; 4], le: bool) -> u32 {
    if le {
        u32::from_le_bytes(*bytes)
    } else {
        u32::from_be_bytes(*bytes)
    }
}

// ── Parse ────────────────────────────────────────────────────────────────────

/// A parsed TIFF block: byte order + IFD0 (with its sub-IFDs and IFD1
/// chain).
#[derive(Debug, Clone)]
pub struct Tiff {
    pub ifd0: Ifd,
}

/// Parse a bare TIFF/EXIF block (as returned by `img-parts`'
/// `ImageEXIF::exif()` — no `"Exif\0\0"` prefix). Fully bounds-checked:
/// malformed, truncated, or cyclic input returns [`Error`], never panics.
pub fn parse(buf: &[u8]) -> Result<Tiff> {
    if buf.len() < 8 {
        return Err(Error::Truncated(buf.len()));
    }
    let little_endian = match &buf[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return Err(Error::BadByteOrder),
    };
    let magic = read_u16(buf, 2, little_endian)?;
    if magic != 42 {
        return Err(Error::BadMagic);
    }
    let ifd0_offset = read_u32(buf, 4, little_endian)? as usize;

    let mut seen = HashSet::new();
    let ifd0 = parse_ifd(buf, ifd0_offset, little_endian, 0, &mut seen)?;
    Ok(Tiff { ifd0 })
}

/// Parse one IFD at `offset`, recursing into pointer-tag sub-IFDs and
/// following the next-IFD link. `depth` bounds recursion
/// ([`MAX_IFD_DEPTH`]); `seen` (offsets already visited in this parse)
/// catches a cycle (an IFD, sub-IFD, or next-link pointing back at an
/// offset already on the path) that a naive depth counter alone might not
/// bound quickly enough.
fn parse_ifd(
    buf: &[u8],
    offset: usize,
    le: bool,
    depth: usize,
    seen: &mut HashSet<usize>,
) -> Result<Ifd> {
    if depth > MAX_IFD_DEPTH {
        return Err(Error::RecursionLimit(MAX_IFD_DEPTH));
    }
    if !seen.insert(offset) {
        return Err(Error::RecursionLimit(MAX_IFD_DEPTH));
    }

    let count = read_u16(buf, offset, le)? as usize;
    let dir_start = offset.checked_add(2).ok_or(Error::Overflow)?;
    let entries_len = count.checked_mul(12).ok_or(Error::Overflow)?;
    // Bounds-check the whole directory (entries + the next-IFD offset slot
    // right after it) up front.
    let next_offset_pos = dir_start.checked_add(entries_len).ok_or(Error::Overflow)?;
    get_slice(buf, dir_start, entries_len)?;
    get_slice(buf, next_offset_pos, 4)?;

    let mut entries = Vec::with_capacity(count);
    let mut thumb_offset: Option<u32> = None;
    let mut thumb_len: Option<u32> = None;

    for i in 0..count {
        let entry_off = dir_start + i * 12;
        let tag = read_u16(buf, entry_off, le)?;
        let ty = read_u16(buf, entry_off + 2, le)?;
        let raw_count = read_u32(buf, entry_off + 4, le)?;
        let elem_size = type_size(ty);
        let vlen = elem_size
            .checked_mul(raw_count as usize)
            .ok_or(Error::Overflow)?;

        let inline_slot: [u8; 4] = get_slice(buf, entry_off + 8, 4)?
            .try_into()
            .expect("get_slice(_, _, 4) yields exactly 4 bytes");

        let value = if vlen <= 4 {
            inline_slot[..vlen].to_vec()
        } else {
            let value_offset = value_as_u32(&inline_slot, le) as usize;
            get_slice(buf, value_offset, vlen)?.to_vec()
        };

        // Thumbnail location tags (only meaningful in IFD1, but harmless to
        // record wherever seen — IFD0 won't carry them in practice).
        if tag == TAG_THUMB_OFFSET && vlen == 4 {
            thumb_offset = Some(value_as_u32(&inline_slot, le));
        }
        if tag == TAG_THUMB_LENGTH && vlen == 4 {
            thumb_len = Some(value_as_u32(&inline_slot, le));
        }

        let sub = if matches!(tag, EXIF_PTR | GPS_PTR | INTEROP_PTR) && ty == TY_LONG && vlen == 4 {
            let sub_offset = value_as_u32(&inline_slot, le) as usize;
            Some(Box::new(parse_ifd(buf, sub_offset, le, depth + 1, seen)?))
        } else {
            None
        };

        entries.push(Entry {
            tag,
            ty,
            count: raw_count,
            value,
            sub,
        });
    }

    let next_raw = read_u32(buf, next_offset_pos, le)?;
    let next = if next_raw == 0 {
        None
    } else {
        Some(Box::new(parse_ifd(
            buf,
            next_raw as usize,
            le,
            depth + 1,
            seen,
        )?))
    };

    let thumbnail = match (thumb_offset, thumb_len) {
        (Some(off), Some(len)) => Some(get_slice(buf, off as usize, len as usize)?.to_vec()),
        _ => None,
    };

    Ok(Ifd {
        entries,
        next,
        thumbnail,
    })
}

// ── Edits ────────────────────────────────────────────────────────────────────

/// Set (add or overwrite) an ASCII (type 2) IFD0 entry: `value` = UTF-8
/// bytes + a trailing NUL (`count = value.len() + 1`), per TIFF 6.0 ASCII
/// semantics. Overwrites in place if `tag` already exists (no duplicate
/// entries emitted); otherwise appends and re-sorts by tag (TIFF requires
/// ascending tag order within a directory).
pub fn set_ascii_tag(ifd0: &mut Ifd, tag: u16, text: &str) {
    let mut value = text.as_bytes().to_vec();
    value.push(0);
    let count = value.len() as u32;

    if let Some(existing) = ifd0.entries.iter_mut().find(|e| e.tag == tag) {
        existing.ty = TY_ASCII;
        existing.count = count;
        existing.value = value;
        existing.sub = None;
    } else {
        ifd0.entries.push(Entry {
            tag,
            ty: TY_ASCII,
            count,
            value,
            sub: None,
        });
        ifd0.entries.sort_by_key(|e| e.tag);
    }
}

/// Drop the IFD0 GPS pointer entry (`GPS_PTR`), orphaning its sub-IFD (it is
/// simply not re-emitted on serialize). A no-op if there is no GPS entry.
pub fn remove_gps(ifd0: &mut Ifd) {
    ifd0.entries.retain(|e| e.tag != GPS_PTR);
}

/// Build a minimal TIFF with an empty IFD0 (the "no existing EXIF" fallback
/// for `set_tags` — callers then apply [`set_ascii_tag`]).
pub fn minimal() -> Tiff {
    Tiff {
        ifd0: Ifd::default(),
    }
}

// ── Serialize ────────────────────────────────────────────────────────────────

/// Serialize a [`Tiff`] to a normalized **little-endian** TIFF block
/// (regardless of the input's original byte order — matches prior
/// `little_exif` output behavior; readers handle either order per the TIFF
/// spec).
pub fn serialize(tiff: &Tiff) -> Vec<u8> {
    let mut out = vec![b'I', b'I', 42, 0, 8, 0, 0, 0];
    put_ifd(&mut out, &tiff.ifd0);
    out
}

/// Append `ifd` (its directory, sub-IFDs, out-of-line values, thumbnail
/// blob, and its `next` chain) to the end of `out`, patching offset slots
/// for the region that was reserved for this call as it goes.
fn put_ifd(out: &mut Vec<u8>, ifd: &Ifd) {
    let dir_at = out.len();
    let count = ifd.entries.len() as u16;
    out.extend_from_slice(&count.to_le_bytes());

    // Reserve the directory (12 bytes/entry) + the 4-byte next-IFD offset
    // slot; entry/offset/sub-IFD patches below write into this reserved
    // region by absolute index, while out-of-line data is appended after.
    let dir_start = dir_at + 2;
    out.resize(dir_start + ifd.entries.len() * 12 + 4, 0);

    for (i, entry) in ifd.entries.iter().enumerate() {
        let slot = dir_start + i * 12;
        out[slot..slot + 2].copy_from_slice(&entry.tag.to_le_bytes());
        out[slot + 2..slot + 4].copy_from_slice(&entry.ty.to_le_bytes());
        out[slot + 4..slot + 8].copy_from_slice(&entry.count.to_le_bytes());

        if entry.tag == TAG_THUMB_OFFSET && ifd.thumbnail.is_some() {
            // Placeholder — patched below once the thumbnail blob (which
            // must be appended after ALL directory entries, including any
            // sub-IFDs walked in this same loop) has an address. Zero for
            // now; corrected in the thumbnail-append step.
            continue;
        }

        if let Some(sub) = &entry.sub {
            let sub_at = out.len();
            put_ifd(out, sub);
            out[slot + 8..slot + 12].copy_from_slice(&(sub_at as u32).to_le_bytes());
        } else if entry.value.len() <= 4 {
            let mut inline = [0u8; 4];
            inline[..entry.value.len()].copy_from_slice(&entry.value);
            out[slot + 8..slot + 12].copy_from_slice(&inline);
        } else {
            if out.len() % 2 == 1 {
                out.push(0); // keep out-of-line values word-aligned
            }
            let value_at = out.len();
            out.extend_from_slice(&entry.value);
            out[slot + 8..slot + 12].copy_from_slice(&(value_at as u32).to_le_bytes());
        }
    }

    // Relocate the thumbnail blob (if any) now that every sub-IFD/value in
    // this directory has been appended, and patch 0x0201's offset slot (its
    // 0x0202 length slot was already written verbatim above — the length
    // doesn't change on relocation).
    if let Some(thumb) = &ifd.thumbnail {
        if out.len() % 2 == 1 {
            out.push(0);
        }
        let thumb_at = out.len();
        out.extend_from_slice(thumb);
        if let Some(i) = ifd.entries.iter().position(|e| e.tag == TAG_THUMB_OFFSET) {
            let slot = dir_start + i * 12;
            out[slot + 8..slot + 12].copy_from_slice(&(thumb_at as u32).to_le_bytes());
        }
    }

    let next_slot = dir_start + ifd.entries.len() * 12;
    if let Some(next) = &ifd.next {
        if out.len() % 2 == 1 {
            out.push(0);
        }
        let next_at = out.len();
        out[next_slot..next_slot + 4].copy_from_slice(&(next_at as u32).to_le_bytes());
        put_ifd(out, next);
    }
    // else: next_slot stays zero (already zero-initialized above) — end of chain.
}
