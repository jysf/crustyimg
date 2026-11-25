//! Verifies the in-repo CI adoption glue (SPEC-057): the `.pre-commit-hooks.yaml`
//! hook, the `just lint-images` recipe, and the README Continuous-integration
//! section. Behavior of the linter the recipe wraps is confirmed by driving the
//! real binary over fixture trees (the same exit codes `just lint-images` yields).
//!
//! The two composite Actions (`setup-crustyimg`, `crustyimg-action`) are separate
//! repos validated by their own 3-OS self-test workflows; only the in-repo glue is
//! testable here.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

mod common;
use common::{gradient_jpeg, solid_png};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// A JPEG whose GPS IFD holds a real position (lat + lon with refs), hand-built.
/// The shared `common::jpeg_with_gps` carries only a `GPSLatitudeRef`, which is
/// not a location, so the exit-7 check below needs a coordinate of its own.
fn jpeg_with_gps_coordinates(w: u32, h: u32) -> Vec<u8> {
    let base = gradient_jpeg(w, h);
    let rat = |v: &[(u32, u32)]| -> Vec<u8> {
        v.iter()
            .flat_map(|(n, d)| n.to_le_bytes().into_iter().chain(d.to_le_bytes()))
            .collect()
    };
    // IFD0 (1 entry) ends at 26; the GPS IFD (4 entries) is 2 + 48 + 4 = 54 bytes,
    // so its data area starts at 80. Latitude takes 24 bytes, longitude follows.
    let (lat_off, lon_off) = (80u32, 104u32);
    let mut tiff: Vec<u8> = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    tiff.extend_from_slice(&[0x01, 0x00, 0x25, 0x88, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00]);
    tiff.extend_from_slice(&26u32.to_le_bytes());
    tiff.extend_from_slice(&[0, 0, 0, 0]);
    tiff.extend_from_slice(&4u16.to_le_bytes());
    for (tag, ty, count, value) in [
        (1u16, 2u16, 2u32, [b'N', 0, 0, 0]),
        (2, 5, 3, lat_off.to_le_bytes()),
        (3, 2, 2, [b'E', 0, 0, 0]),
        (4, 5, 3, lon_off.to_le_bytes()),
    ] {
        tiff.extend_from_slice(&tag.to_le_bytes());
        tiff.extend_from_slice(&ty.to_le_bytes());
        tiff.extend_from_slice(&count.to_le_bytes());
        tiff.extend_from_slice(&value);
    }
    tiff.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(tiff.len(), 80, "GPS IFD layout");
    tiff.extend_from_slice(&rat(&[(50, 1), (29, 1), (1113, 25)]));
    tiff.extend_from_slice(&rat(&[(30, 1), (31, 1), (0, 1)]));

    let mut payload = b"Exif\0\0".to_vec();
    payload.extend_from_slice(&tiff);
    let mut out = Vec::new();
    out.extend_from_slice(&base[0..2]);
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]);
    out
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

#[test]
fn pre_commit_hooks_defines_the_crustyimg_lint_hook() {
    let y = read(".pre-commit-hooks.yaml");
    assert!(y.contains("id: crustyimg-lint"), "hook id");
    assert!(y.contains("entry: crustyimg lint"), "runs `crustyimg lint`");
    assert!(y.contains("language: rust"), "builds via cargo");
    assert!(y.contains("types: [image]"), "runs on image files");
}

#[test]
fn justfile_has_a_lint_images_recipe() {
    let j = read("justfile");
    // The recipe is declared with a `paths` parameter and wraps `crustyimg lint`.
    assert!(
        j.contains("lint-images *paths"),
        "lint-images recipe with a paths parameter"
    );
    assert!(
        j.contains("-- lint {{paths}}"),
        "lint-images must invoke `crustyimg lint`"
    );
}

#[test]
fn readme_documents_the_ci_on_ramp() {
    let r = read("README.md");
    assert!(r.contains("## Continuous integration"), "CI section");
    assert!(r.contains("jysf/setup-crustyimg"), "setup Action");
    assert!(r.contains("jysf/crustyimg-action"), "lint Action");
    assert!(r.contains("crustyimg lint"), "a lint invocation");
    assert!(r.contains(".pre-commit-hooks.yaml"), "pre-commit reference");
}

/// The exit-code contract `just lint-images` relies on: `0` on a clean tree, `7`
/// on an error finding (a GPS leak). Driven through the real binary — the recipe
/// is a thin `cargo run -- lint {{paths}}` wrapper over exactly this.
#[test]
fn lint_gate_exit_codes_over_fixture_trees() {
    let clean = TempDir::new().unwrap();
    std::fs::write(clean.path().join("a.png"), solid_png(4, 4, [1, 2, 3])).unwrap();
    let clean_code = Command::new(BIN)
        .arg("lint")
        .arg(clean.path())
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert_eq!(clean_code, 0, "clean tree → exit 0");

    let leaky = TempDir::new().unwrap();
    std::fs::write(
        leaky.path().join("leak.jpg"),
        jpeg_with_gps_coordinates(16, 16),
    )
    .unwrap();
    let leaky_code = Command::new(BIN)
        .arg("lint")
        .arg(leaky.path())
        .output()
        .unwrap()
        .status
        .code()
        .unwrap_or(-1);
    assert_eq!(leaky_code, 7, "GPS-leaking tree → exit 7");
}
