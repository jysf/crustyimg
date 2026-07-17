//! Integration tests for the container-lane metadata commands (SPEC-026):
//! `meta strip` and `meta clean --gps` (grouped under `meta` in SPEC-087).
//! These drive the REAL compiled binary via
//! `std::process::Command` and assert exit codes + output bytes end-to-end.
//!
//! Fixtures are generated NATIVELY (no ImageMagick): pixels via the `image`
//! crate; EXIF (Orientation + Copyright + GPS refs) is a hand-assembled
//! little-endian TIFF block (SPEC-045/DEC-046 — `little_exif` is gone),
//! embedded via `img-parts` `set_exif`, mirroring `tests/common::
//! jpeg_with_orientation`'s existing hand-rolled-TIFF style. Read-back
//! assertions use `kamadak-exif` (the `exif` crate), the project's
//! read-side dependency. `.unwrap()` here is idiomatic test setup (the
//! `no-unwrap` constraint is scoped to `src/**`).

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageEXIF};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers (native; no ImageMagick) ──────────────────────────────────

/// A small deterministic 16×16 RGB image encoded to `format`, with no metadata.
fn base_bytes(format: ImageFormat) -> Vec<u8> {
    let mut img = RgbImage::new(16, 16);
    for (x, y, px) in img.enumerate_pixels_mut() {
        *px = image::Rgb([(x * 16) as u8, (y * 16) as u8, ((x + y) * 8) as u8]);
    }
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, format)
        .unwrap();
    buf.into_inner()
}

/// Append a TIFF ASCII (type 2) entry — value = UTF-8 bytes + trailing NUL —
/// to a hand-assembled directory. Values > 4 bytes go out-of-line via
/// `out_of_line`; the caller patches the offset once the directory's base is
/// known (see `build_tiff`).
struct RawEntry {
    tag: u16,
    ty: u16,
    count: u32,
    /// Inline value (padded to 4 bytes) or, if `out_of_line` is `Some`, a
    /// placeholder — the real bytes live in `out_of_line`.
    inline: [u8; 4],
    out_of_line: Option<Vec<u8>>,
}

fn ascii_raw(tag: u16, text: &str) -> RawEntry {
    let mut value = text.as_bytes().to_vec();
    value.push(0);
    let count = value.len() as u32;
    if value.len() <= 4 {
        let mut inline = [0u8; 4];
        inline[..value.len()].copy_from_slice(&value);
        RawEntry {
            tag,
            ty: 2,
            count,
            inline,
            out_of_line: None,
        }
    } else {
        RawEntry {
            tag,
            ty: 2,
            count,
            inline: [0; 4],
            out_of_line: Some(value),
        }
    }
}

fn short_raw(tag: u16, v: u16) -> RawEntry {
    let mut inline = [0u8; 4];
    inline[0..2].copy_from_slice(&v.to_le_bytes());
    RawEntry {
        tag,
        ty: 3,
        count: 1,
        inline,
        out_of_line: None,
    }
}

/// Hand-assemble a minimal little-endian TIFF block: a single IFD0 with
/// `entries` (sorted by tag, per TIFF 6.0), no sub-IFDs, no next-IFD. Mirrors
/// `tests/common::jpeg_with_orientation`'s existing hand-rolled-TIFF style,
/// generalized to N entries (this file needs Copyright/Orientation/GPS-ref
/// combinations `common` doesn't provide).
fn build_tiff(mut entries: Vec<RawEntry>) -> Vec<u8> {
    entries.sort_by_key(|e| e.tag);
    let mut out = vec![b'I', b'I', 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    let dir_start = out.len();
    out.resize(dir_start + entries.len() * 12 + 4, 0);
    for (i, e) in entries.iter().enumerate() {
        let slot = dir_start + i * 12;
        out[slot..slot + 2].copy_from_slice(&e.tag.to_le_bytes());
        out[slot + 2..slot + 4].copy_from_slice(&e.ty.to_le_bytes());
        out[slot + 4..slot + 8].copy_from_slice(&e.count.to_le_bytes());
        if let Some(value) = &e.out_of_line {
            let at = out.len() as u32;
            out.extend_from_slice(value);
            out[slot + 8..slot + 12].copy_from_slice(&at.to_le_bytes());
        } else {
            out[slot + 8..slot + 12].copy_from_slice(&e.inline);
        }
    }
    // next-IFD offset (already zero-initialized) = 0 — no IFD1.
    out
}

// IFD0 tag ids: Artist 0x013B, Copyright 0x8298, Orientation 0x0112.
const TAG_ARTIST: u16 = 0x013B;
const TAG_COPYRIGHT: u16 = 0x8298;
const TAG_ORIENTATION: u16 = 0x0112;
// GPS sub-tags (used inline in the GPS IFD below).
const TAG_GPS_LAT_REF: u16 = 0x0001;
const TAG_GPS_LON_REF: u16 = 0x0003;
const GPS_PTR: u16 = 0x8825;

/// A little-endian TIFF block for a JPEG seeded with Orientation + Copyright
/// (IFD0) and GPS{Latitude,Longitude}Ref (GPS sub-IFD).
fn tiff_with_gps() -> Vec<u8> {
    // Build the GPS sub-IFD first (it must be appended after IFD0 in the
    // final buffer; we do this by hand since there's no sub-IFD support in
    // this file's minimal `build_tiff`).
    let gps_entries = vec![
        ascii_raw(TAG_GPS_LAT_REF, "N"),
        ascii_raw(TAG_GPS_LON_REF, "E"),
    ];
    let gps_ifd = build_tiff(gps_entries);
    // `build_tiff` emits a full standalone TIFF (header + IFD); we only want
    // its IFD bytes (from offset 8 on) to append as a sub-IFD.
    let gps_ifd_bytes = &gps_ifd[8..];

    let ifd0 = vec![
        short_raw(TAG_ORIENTATION, 1),
        ascii_raw(TAG_COPYRIGHT, "crustyimg test"),
        // Placeholder GPS pointer; patched once we know where the sub-IFD lands.
        RawEntry {
            tag: GPS_PTR,
            ty: 4,
            count: 1,
            inline: [0; 4],
            out_of_line: None,
        },
    ];
    let mut out = build_tiff(ifd0);
    let gps_at = out.len() as u32;
    out.extend_from_slice(gps_ifd_bytes);

    // Patch the GPS pointer entry's inline offset slot. IFD0 has 3 entries;
    // find GPS_PTR's slot by scanning the directory we just built.
    let count = u16::from_le_bytes([out[8], out[9]]) as usize;
    let dir_start = 10;
    for i in 0..count {
        let slot = dir_start + i * 12;
        let tag = u16::from_le_bytes([out[slot], out[slot + 1]]);
        if tag == GPS_PTR {
            out[slot + 8..slot + 12].copy_from_slice(&gps_at.to_le_bytes());
            break;
        }
    }
    out
}

/// JPEG bytes seeded with Orientation + Copyright + GPS{Latitude,Longitude}Ref,
/// embedded via `img-parts` `set_exif` (bare TIFF, no `Exif\0\0` prefix).
fn jpeg_with_exif() -> Vec<u8> {
    let base = base_bytes(ImageFormat::Jpeg);
    let mut jpeg = Jpeg::from_bytes(Bytes::from(base)).unwrap();
    jpeg.set_exif(Some(Bytes::from(tiff_with_gps())));
    let mut out = Vec::new();
    jpeg.encoder().write_to(&mut out).unwrap();
    out
}

/// Write `bytes` to `dir/name` and return the path.
fn write_fixture(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Extract the bare TIFF/EXIF block from a JPEG via `img-parts`, if any.
fn jpeg_exif_block(bytes: &[u8]) -> Option<Bytes> {
    Jpeg::from_bytes(Bytes::from(bytes.to_vec())).ok()?.exif()
}

/// Whether a JPEG byte stream still carries any readable EXIF fields at all
/// (`kamadak-exif`) — an empty-but-parseable TIFF counts as "no EXIF" here,
/// matching `strip_all`'s intent (no tags survive).
fn jpeg_has_exif(bytes: &[u8]) -> bool {
    let Some(tiff) = jpeg_exif_block(bytes) else {
        return false;
    };
    match exif::Reader::new().read_raw(tiff.to_vec()) {
        Ok(parsed) => parsed.fields().len() > 0,
        Err(_) => false,
    }
}

/// Whether a JPEG byte stream carries the given (Context-qualified)
/// `exif::Tag` anywhere in its EXIF. Compares the full `Tag` (context +
/// number), not just the raw tag id, since GPS/Exif/Interop sub-IFD tag
/// numbers can collide with IFD0 tag numbers (e.g. GPS 0x0001 vs a
/// hypothetical generic 0x0001) — a bare-number compare would be a false
/// positive across contexts, exactly the kind of semantic slip this in-house
/// writer must not have (SPEC-045).
fn jpeg_has_tag(bytes: &[u8], tag: exif::Tag) -> bool {
    let Some(tiff) = jpeg_exif_block(bytes) else {
        return false;
    };
    let Ok(parsed) = exif::Reader::new().read_raw(tiff.to_vec()) else {
        return false;
    };
    let found = parsed.fields().any(|f| f.tag == tag);
    found
}

/// Whether a JPEG byte stream carries an IFD0 (primary) tag with the given
/// raw id. Used only for the two IFD0-only tags this file seeds (Artist,
/// Copyright) that don't collide with any sub-IFD tag number in these
/// fixtures.
fn jpeg_has_generic_tag(bytes: &[u8], tag_id: u16) -> bool {
    let Some(tiff) = jpeg_exif_block(bytes) else {
        return false;
    };
    let Ok(parsed) = exif::Reader::new().read_raw(tiff.to_vec()) else {
        return false;
    };
    let found = parsed
        .fields()
        .any(|f| f.tag.number() == tag_id && f.ifd_num == exif::In::PRIMARY);
    found
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[test]
fn strip_jpeg_to_stdout_has_no_exif() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["meta", "strip", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert!(output.status.success(), "strip should exit 0");
    assert!(
        !jpeg_has_exif(&output.stdout),
        "stripped JPEG on stdout should have no EXIF"
    );
}

#[test]
fn clean_gps_jpeg_removes_location_keeps_orientation() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "clean",
            input.to_str().unwrap(),
            "--gps",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "clean --gps should exit 0");

    let cleaned = std::fs::read(&out).unwrap();
    // GPS gone.
    assert!(
        !jpeg_has_tag(&cleaned, exif::Tag::GPSLatitudeRef),
        "GPS tags should be removed"
    );
    // Orientation (0x0112) survives.
    assert!(
        jpeg_has_generic_tag(&cleaned, TAG_ORIENTATION),
        "Orientation should be preserved"
    );
}

#[test]
fn clean_without_gps_flag_exits_2() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["meta", "clean", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "clean without --gps should exit 2"
    );
}

#[test]
fn strip_unsupported_format_exits_4() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.bmp", &base_bytes(ImageFormat::Bmp));

    let output = Command::new(BIN)
        .args(["meta", "strip", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(4),
        "strip on a BMP should exit 4 (unsupported format)"
    );
}

#[test]
fn strip_multi_input_requires_out_dir() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &jpeg_with_exif());
    let b = write_fixture(&dir, "b.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["meta", "strip", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "multiple inputs without --out-dir should exit 2"
    );
}

#[test]
fn strip_multi_input_fanout_writes_all() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &jpeg_with_exif());
    let b = write_fixture(&dir, "b.jpg", &jpeg_with_exif());
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "meta",
            "strip",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "fan-out strip should exit 0");

    for name in ["a.jpg", "b.jpg"] {
        let path = out_dir.path().join(name);
        assert!(path.exists(), "{name} should be written");
        let bytes = std::fs::read(&path).unwrap();
        assert!(!jpeg_has_exif(&bytes), "{name} should be stripped");
    }
}

#[test]
fn strip_batch_partial_failure_exits_6() {
    let dir = TempDir::new().unwrap();
    let good = write_fixture(&dir, "good.jpg", &jpeg_with_exif());
    let bad = write_fixture(&dir, "bad.bmp", &base_bytes(ImageFormat::Bmp));
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "meta",
            "strip",
            good.to_str().unwrap(),
            bad.to_str().unwrap(),
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(6),
        "a batch with one unsupported input should exit 6"
    );
    // The good input is still written.
    assert!(
        out_dir.path().join("good.jpg").exists(),
        "the good input should still be written"
    );
}

#[test]
fn strip_refuses_overwrite_without_yes() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");
    // Pre-create the output so the overwrite guard trips.
    std::fs::write(&out, b"existing").unwrap();

    let refused = Command::new(BIN)
        .args([
            "meta",
            "strip",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        refused.status.code(),
        Some(5),
        "overwrite without --yes should exit 5"
    );

    let allowed = Command::new(BIN)
        .args([
            "meta",
            "strip",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        allowed.status.success(),
        "overwrite with --yes should exit 0"
    );
}

// ── meta set (SPEC-027, folded from top-level `set` via SPEC-089) ─────────────

#[test]
fn set_writes_tags_to_output() {
    let dir = TempDir::new().unwrap();
    // Plain JPEG with no EXIF: set creates the tags.
    let input = write_fixture(&dir, "in.jpg", &base_bytes(ImageFormat::Jpeg));
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "Jane",
            "--copyright",
            "2026",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "meta set should exit 0");
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_ARTIST),
        "Artist should be written"
    );
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_COPYRIGHT),
        "Copyright should be written"
    );
}

/// SPEC-089: `meta set` with none of `--artist`/`--copyright`/`--description`
/// is a usage error (exit 2) with the updated `"meta set requires …"` message
/// (SPEC-089's deliberate divergence from SPEC-087, which left `clean`'s
/// message verbatim).
#[test]
fn meta_set_requires_a_tag() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());

    let output = Command::new(BIN)
        .args(["meta", "set", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(2),
        "meta set with no tag flags should exit 2"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("meta set requires"),
        "usage error should name the live command `meta set`, got:\n{stderr}"
    );
}

#[test]
fn set_preserves_other_metadata() {
    let dir = TempDir::new().unwrap();
    // jpeg_with_exif carries Orientation (0x0112).
    let input = write_fixture(&dir, "in.jpg", &jpeg_with_exif());
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--copyright",
            "X",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "meta set should exit 0");
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        jpeg_has_generic_tag(&bytes, TAG_ORIENTATION),
        "Orientation should be preserved"
    );
}

#[test]
fn set_unsupported_format_exits_4() {
    let dir = TempDir::new().unwrap();
    let input = write_fixture(&dir, "in.bmp", &base_bytes(ImageFormat::Bmp));

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "A",
            "-o",
            "-",
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(4),
        "meta set on a BMP should exit 4 (unsupported format)"
    );
}

#[test]
fn set_multi_input_fanout_writes_all() {
    let dir = TempDir::new().unwrap();
    let a = write_fixture(&dir, "a.jpg", &base_bytes(ImageFormat::Jpeg));
    let b = write_fixture(&dir, "b.jpg", &base_bytes(ImageFormat::Jpeg));
    let out_dir = TempDir::new().unwrap();

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            a.to_str().unwrap(),
            b.to_str().unwrap(),
            "--artist",
            "A",
            "--out-dir",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "fan-out meta set should exit 0");

    for name in ["a.jpg", "b.jpg"] {
        let path = out_dir.path().join(name);
        assert!(path.exists(), "{name} should be written");
        let bytes = std::fs::read(&path).unwrap();
        assert!(
            jpeg_has_generic_tag(&bytes, TAG_ARTIST),
            "{name} should be tagged with Artist"
        );
    }
}

// ── copy-metadata (SPEC-028, DEC-030) ─────────────────────────────────────────

/// JPEG bytes seeded with a single Copyright tag, embedded via `img-parts`
/// `set_exif` (no `little_exif` — SPEC-045/DEC-046).
fn jpeg_with_copyright(value: &str) -> Vec<u8> {
    let tiff = build_tiff(vec![ascii_raw(TAG_COPYRIGHT, value)]);
    let mut jpeg = Jpeg::from_bytes(Bytes::from(base_bytes(ImageFormat::Jpeg))).unwrap();
    jpeg.set_exif(Some(Bytes::from(tiff)));
    let mut out = Vec::new();
    jpeg.encoder().write_to(&mut out).unwrap();
    out
}

/// Decode `bytes` to an RGBA pixel buffer (for decode-equality assertions).
fn decode_rgba(bytes: &[u8]) -> Vec<u8> {
    image::load_from_memory(bytes)
        .unwrap()
        .to_rgba8()
        .into_raw()
}

#[test]
fn copy_metadata_to_explicit_output() {
    let dir = TempDir::new().unwrap();
    let src = write_fixture(&dir, "src.jpg", &jpeg_with_copyright("SRC owner"));
    let dst_bytes = base_bytes(ImageFormat::Jpeg);
    let dst = write_fixture(&dir, "dst.jpg", &dst_bytes);
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "copy-metadata should exit 0");

    // out carries SRC's Copyright.
    let out_bytes = std::fs::read(&out).unwrap();
    assert!(
        jpeg_has_generic_tag(&out_bytes, TAG_COPYRIGHT),
        "out should carry SRC's Copyright"
    );

    // DST on disk is untouched (no EXIF written to it).
    let dst_after = std::fs::read(&dst).unwrap();
    assert_eq!(dst_after, dst_bytes, "DST file should be unchanged on disk");
}

#[test]
fn copy_metadata_in_place_requires_yes() {
    let dir = TempDir::new().unwrap();
    let src = write_fixture(&dir, "src.jpg", &jpeg_with_copyright("SRC owner"));
    let dst = write_fixture(&dir, "dst.jpg", &base_bytes(ImageFormat::Jpeg));

    // No -o, no -y: in-place overwrite of the existing DST is refused → exit 5.
    let refused = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        refused.status.code(),
        Some(5),
        "in-place copy without --yes should exit 5"
    );

    // With -y: overwrites DST in place and DST now carries SRC's Copyright.
    let allowed = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(
        allowed.status.success(),
        "in-place copy with --yes should exit 0"
    );
    let dst_after = std::fs::read(&dst).unwrap();
    assert!(
        jpeg_has_generic_tag(&dst_after, TAG_COPYRIGHT),
        "dst.jpg should now carry SRC's Copyright"
    );
}

#[test]
fn copy_metadata_png_exits_4() {
    let dir = TempDir::new().unwrap();
    let src = write_fixture(&dir, "src.jpg", &jpeg_with_copyright("SRC owner"));
    let dst = write_fixture(&dir, "dst.png", &base_bytes(ImageFormat::Png));
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(4),
        "a PNG --to should exit 4 (JPEG-only, DEC-030)"
    );
}

#[test]
fn copy_metadata_preserves_pixels_e2e() {
    let dir = TempDir::new().unwrap();
    let src = write_fixture(&dir, "src.jpg", &jpeg_with_copyright("SRC owner"));
    let dst_bytes = base_bytes(ImageFormat::Jpeg);
    let dst = write_fixture(&dir, "dst.jpg", &dst_bytes);
    let pixels_before = decode_rgba(&dst_bytes);

    let output = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
            "--yes",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "in-place copy should exit 0");

    let dst_after = std::fs::read(&dst).unwrap();
    assert_eq!(
        decode_rgba(&dst_after),
        pixels_before,
        "DST's decoded pixels should be unchanged after copy"
    );
}

// ── SPEC-087: `meta` group is a pure surface move ─────────────────────────────

/// SPEC-087: `meta strip`, `meta clean --gps`, and `meta copy` are a *pure
/// surface move* of the old top-level `strip`/`clean`/`copy-metadata` verbs —
/// nothing changed but the path. Prove byte-identity: the CLI's output bytes
/// must equal the underlying container-lane op's output on the identical input
/// (the exact functions the old verbs dispatched to, and the new ones still do).
/// Capturing the "golden" bytes straight from the library op is the strongest
/// pre-move reference available once the old verbs are deleted.
#[test]
fn meta_subcommands_match_old_verbs() {
    let dir = TempDir::new().unwrap();
    let fixture = jpeg_with_exif();
    let input = write_fixture(&dir, "in.jpg", &fixture);

    // meta strip  ==  metadata::strip_all
    let golden_strip = crustyimg::metadata::strip_all(&fixture).expect("strip_all");
    let out = Command::new(BIN)
        .args(["meta", "strip", input.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "meta strip should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout, golden_strip,
        "`meta strip` bytes must be byte-identical to strip_all() (the old `strip`)"
    );

    // meta clean --gps  ==  metadata::clean_gps
    let golden_clean = crustyimg::metadata::clean_gps(&fixture).expect("clean_gps");
    let out = Command::new(BIN)
        .args(["meta", "clean", input.to_str().unwrap(), "--gps", "-o", "-"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "meta clean --gps should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout, golden_clean,
        "`meta clean --gps` bytes must be byte-identical to clean_gps() (the old `clean`)"
    );

    // meta copy --from SRC --to DST  ==  metadata::copy_metadata
    let src_bytes = jpeg_with_copyright("© Old Verb");
    let src = write_fixture(&dir, "src.jpg", &src_bytes);
    let dst_bytes = jpeg_with_exif();
    let dst = write_fixture(&dir, "dst.jpg", &dst_bytes);
    let golden_copy =
        crustyimg::metadata::copy_metadata(&src_bytes, &dst_bytes).expect("copy_metadata");
    let out = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            src.to_str().unwrap(),
            "--to",
            dst.to_str().unwrap(),
            "-o",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "meta copy should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout, golden_copy,
        "`meta copy` bytes must be byte-identical to copy_metadata() (the old `copy-metadata`)"
    );
}

// ── SPEC-089: `set` folded into `meta set` is a pure surface move ─────────────

/// SPEC-089: `meta set` is a *pure surface move* of the old top-level `set`
/// verb — nothing changed but the path. Prove byte-identity: the CLI's output
/// bytes must equal `metadata::set_tags`'s output on the identical input and
/// tags (the exact function the old `set` dispatched to, and `meta set` still
/// does), mirroring SPEC-087's proof for strip/clean/copy.
#[test]
fn meta_set_matches_old_set() {
    let dir = TempDir::new().unwrap();
    let fixture = jpeg_with_exif();
    let input = write_fixture(&dir, "in.jpg", &fixture);

    let tags = crustyimg::metadata::TagSet {
        artist: Some("Jane".to_string()),
        copyright: Some("2026".to_string()),
        description: None,
    };
    let golden = crustyimg::metadata::set_tags(&fixture, &tags).expect("set_tags");

    let out = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "Jane",
            "--copyright",
            "2026",
            "-o",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "meta set should exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.stdout, golden,
        "`meta set` bytes must be byte-identical to set_tags() (the old `set`)"
    );
}
