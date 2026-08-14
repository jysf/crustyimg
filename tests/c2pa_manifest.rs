//! Integration tests for SPEC-114: `meta set`/`meta clean --gps`/`meta copy`
//! must never write a file carrying a C2PA/Content Credentials manifest that
//! no longer matches its own bytes, and `meta strip`'s removal of one is
//! pinned as intentional (not an accident of the `0xE1..=0xEF` sweep).
//!
//! Drives the REAL compiled binary via `std::process::Command`, matching
//! `tests/metadata.rs`'s existing pattern. Manifest presence/absence is
//! checked with a hand-rolled structural walk (JPEG APP11 `0xEB` marker / PNG
//! `caBX` chunk) written independently of both `img-parts` and the crate's
//! own `metadata::has_manifest`, so a broken production scan can't pass this
//! suite just by agreeing with itself. Validity (not just presence) was
//! independently confirmed against every fixture with `c2patool` 0.27.9 (an
//! external validator) during the build session — see SPEC-114's `## Build
//! Completion` for that record. `cargo test` never shells out to c2patool,
//! so CI does not need it installed.
//!
//! ## Fixtures (`tests/fixtures/c2pa/`)
//!
//! - `signed_gps.jpg` — a plain 16x16 JPEG (no prior EXIF), given GPS +
//!   Orientation=6 via `exiftool` 13.55, then signed with `c2patool` 0.27.9
//!   using the `contentauth/c2pa-rs` project's own ES256 **test** certificate
//!   (`sdk/tests/fixtures/certs/es256.{pem,pub}` — not a production cert).
//!   Signed with an EXTERNAL tool against a test cert, **never with
//!   crustyimg itself** — a fixture from the code under test cannot fail.
//! - `signed.png` — the PNG mirror of the same fixture: same GPS/Orientation
//!   via exiftool, same c2patool/cert signing, so `meta`'s PNG lane can be
//!   driven against something real instead of assumed symmetric with JPEG.
//!
//! Regenerate from the repo root:
//! ```sh
//! curl -o /tmp/es256.pem https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/certs/es256.pem
//! curl -o /tmp/es256.pub https://raw.githubusercontent.com/contentauth/c2pa-rs/main/sdk/tests/fixtures/certs/es256.pub
//! cat > /tmp/manifest.json <<'EOF'
//! {"alg": "es256", "private_key": "/tmp/es256.pem", "sign_cert": "/tmp/es256.pub"}
//! EOF
//! magick -size 16x16 xc:"rgb(120,80,200)" -depth 8 /tmp/base.jpg
//! exiftool -overwrite_original "-Orientation#=6" -GPSLatitude=50.4957 -GPSLatitudeRef=N \
//!   -GPSLongitude=4.4778 -GPSLongitudeRef=E /tmp/base.jpg
//! c2patool /tmp/base.jpg -m /tmp/manifest.json -o tests/fixtures/c2pa/signed_gps.jpg -f
//!
//! magick -size 16x16 xc:"rgb(200,50,50)" -type TrueColor -depth 8 PNG24:/tmp/base.png
//! exiftool -overwrite_original "-Orientation#=6" -GPSLatitude=50.4957 -GPSLatitudeRef=N \
//!   -GPSLongitude=4.4778 -GPSLongitudeRef=E /tmp/base.png
//! c2patool /tmp/base.png -m /tmp/manifest.json -o tests/fixtures/c2pa/signed.png -f
//! ```

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use img_parts::jpeg::Jpeg;
use img_parts::{Bytes, ImageEXIF};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/c2pa")
        .join(name)
}

fn write_fixture(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A small deterministic 16x16 RGB image encoded to `format`, with no
/// metadata at all — a plain, never-signed fixture built natively (no
/// external tools), matching `tests/metadata.rs::base_bytes`.
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

/// Independent structural walk for a JPEG APP11 (`0xEB`) segment — hand-
/// written directly against the marker-segment format (SOI, then `FF xx`
/// markers with a 2-byte big-endian length), NOT via `img-parts` and NOT via
/// the crate's own `metadata::has_manifest`. This is one of the two
/// independent checks SPEC-114 requires; c2patool is the other (run outside
/// `cargo test`, see the module doc).
fn jpeg_has_app11(bytes: &[u8]) -> bool {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return false;
    }
    let mut i = 2;
    while i + 4 <= bytes.len() {
        if bytes[i] != 0xFF {
            return false; // not a marker; malformed or truncated — not our fixtures
        }
        let marker = bytes[i + 1];
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            i += 2; // markers with no length/payload (restart markers, SOI/EOI)
            continue;
        }
        if marker == 0xDA {
            return false; // start of scan: no APP segment follows in our fixtures
        }
        let len = u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]) as usize;
        if marker == 0xEB {
            return true;
        }
        i += 2 + len;
    }
    false
}

/// Independent structural walk for a PNG `caBX` chunk (not via `img-parts`).
fn png_has_cabx(bytes: &[u8]) -> bool {
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
    if bytes.len() < 8 || bytes[..8] != SIG {
        return false;
    }
    let mut i = 8;
    while i + 8 <= bytes.len() {
        let len = u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize;
        let ctype = &bytes[i + 4..i + 8];
        if ctype == b"caBX" {
            return true;
        }
        i += 8 + len + 4;
        if i > bytes.len() {
            break;
        }
    }
    false
}

// ── AC-1 / AC-2: `meta set` ───────────────────────────────────────────────

#[test]
fn meta_set_drops_a_manifest_rather_than_invalidating_it() {
    let dir = TempDir::new().unwrap();
    let input = fixture("signed_gps.jpg");
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "Test",
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "meta set should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        !jpeg_has_app11(&bytes),
        "output must not carry a JUMBF/APP11 manifest segment"
    );
}

#[test]
fn meta_set_warns_when_it_drops_a_manifest() {
    let dir = TempDir::new().unwrap();
    let input = fixture("signed_gps.jpg");
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "Test",
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning:"),
        "stderr should carry a warning: {stderr}"
    );
    assert!(
        stderr.contains("Content Credentials"),
        "warning should name what was removed: {stderr}"
    );
}

// ── AC-3: `meta clean --gps` ───────────────────────────────────────────────

#[test]
fn meta_clean_drops_a_manifest_rather_than_invalidating_it() {
    let dir = TempDir::new().unwrap();
    let input = fixture("signed_gps.jpg");
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "clean",
            input.to_str().unwrap(),
            "--gps",
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "meta clean --gps should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        !jpeg_has_app11(&bytes),
        "clean --gps must drop the manifest, not just break it"
    );

    // clean's own contract still holds on the way past the manifest fix:
    // GPS is actually gone, Orientation survives exactly.
    let jpeg = Jpeg::from_bytes(Bytes::from(bytes)).unwrap();
    let tiff = jpeg.exif().expect("EXIF (Orientation) should survive");
    let exif = exif::Reader::new().read_raw(tiff.to_vec()).unwrap();
    assert!(
        exif.get_field(exif::Tag::GPSLatitudeRef, exif::In::PRIMARY)
            .is_none(),
        "GPS should be gone"
    );
    assert_eq!(
        exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
            .and_then(|f| f.value.get_uint(0)),
        Some(6),
        "Orientation should survive clean --gps exactly"
    );
}

// ── AC-4: `meta copy`, both directions ─────────────────────────────────────

#[test]
fn meta_copy_never_retains_an_invalidated_manifest() {
    let dir = TempDir::new().unwrap();
    let signed = fixture("signed_gps.jpg");
    let plain = write_fixture(&dir, "plain.jpg", &base_bytes(ImageFormat::Jpeg));

    // Direction A: FROM = signed donor, TO = plain recipient. `copy` only
    // grafts EXIF/ICC, so the donor's manifest must never be transplanted —
    // this was already true pre-fix; pinned here as a control.
    let out_a = dir.path().join("out_a.jpg");
    let output = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            signed.to_str().unwrap(),
            "--to",
            plain.to_str().unwrap(),
            "-o",
            out_a.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let bytes_a = std::fs::read(&out_a).unwrap();
    assert!(
        !jpeg_has_app11(&bytes_a),
        "copy must never transplant SRC's manifest onto DST"
    );

    // Direction B: FROM = plain donor, TO = signed recipient. The graft
    // rewrites DST's own EXIF, which invalidates DST's own manifest — this
    // is the actual bug: DST's manifest must be dropped, not retained-and-
    // broken.
    let out_b = dir.path().join("out_b.jpg");
    let output = Command::new(BIN)
        .args([
            "meta",
            "copy",
            "--from",
            plain.to_str().unwrap(),
            "--to",
            signed.to_str().unwrap(),
            "-o",
            out_b.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "copy should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes_b = std::fs::read(&out_b).unwrap();
    assert!(
        !jpeg_has_app11(&bytes_b),
        "copy must drop DST's own manifest when the graft would invalidate it"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning:"),
        "copy should also warn when it drops DST's manifest: {stderr}"
    );
}

// ── AC-5: `meta strip` (JPEG), pinned by name ──────────────────────────────

#[test]
fn meta_strip_removes_the_jumbf_segment_by_name() {
    let dir = TempDir::new().unwrap();
    let input = fixture("signed_gps.jpg");
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "meta",
            "strip",
            input.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let bytes = std::fs::read(&out).unwrap();
    assert!(
        !jpeg_has_app11(&bytes),
        "strip must remove the JUMBF/APP11 manifest segment, driven against a REAL signed \
         fixture (not just an arbitrary marker in the 0xE1..=0xEF sweep)"
    );
    // strip's contract is silent, total removal — unlike set/clean/copy, which
    // warn because they drop a manifest as a side effect of a surgical edit.
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "strip should not print a manifest-drop warning"
    );
}

// ── AC-6: PNG, driven — `caBX` is not in PNG_METADATA_CHUNKS pre-fix ───────

#[test]
fn meta_strip_and_set_remove_the_png_cabx_manifest_chunk() {
    let dir = TempDir::new().unwrap();
    let input = fixture("signed.png");

    let out_strip = dir.path().join("stripped.png");
    let output = Command::new(BIN)
        .args([
            "meta",
            "strip",
            input.to_str().unwrap(),
            "-o",
            out_strip.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        !png_has_cabx(&std::fs::read(&out_strip).unwrap()),
        "strip must remove PNG's caBX manifest chunk (pre-fix it survives — caBX isn't in \
         PNG_METADATA_CHUNKS)"
    );

    let out_set = dir.path().join("set.png");
    let output = Command::new(BIN)
        .args([
            "meta",
            "set",
            input.to_str().unwrap(),
            "--artist",
            "PNG Test",
            "-o",
            out_set.to_str().unwrap(),
            "-y",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        !png_has_cabx(&std::fs::read(&out_set).unwrap()),
        "set must drop PNG's caBX manifest rather than retain-and-break it"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("warning:"));
}
