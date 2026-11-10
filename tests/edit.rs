//! Integration tests for `edit` + `--save-recipe` (SPEC-032).
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are synthesized in memory with the `image` crate — no committed
//! binary files. Outputs and recipes are written to a tempdir.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Generate a tiny solid-color RGB PNG and write it to `dir/name`. Returns the path.
fn write_png(dir: &TempDir, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_pixel(w, h, image::Rgb([42u8, 100u8, 200u8]));
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, buf.into_inner()).unwrap();
    path
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// `edit in.png --resize-max 8 -o out.png` → out.png exists, max edge == 8; exit 0.
#[test]
fn edit_resize_writes_output() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let out = dir.path().join("out.png");

    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit");

    assert!(
        status.success(),
        "edit --resize-max 8 must exit 0; got: {:?}",
        status.code()
    );
    assert!(out.exists(), "out.png must be created");

    let img = image::open(&out).unwrap();
    assert!(
        img.width() <= 8 && img.height() <= 8,
        "max edge must be ≤ 8, got {}×{}",
        img.width(),
        img.height()
    );
}

/// `edit in.png -o out.png` with NO op flag → exit 2.
#[test]
fn edit_no_ops_exits_2() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 16, 16);
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args(["edit", input.to_str().unwrap(), "-o", out.to_str().unwrap()])
        .output()
        .expect("failed to run edit no-ops");

    assert_eq!(
        output.status.code(),
        Some(2),
        "no op flags must exit 2; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `edit nope.png --invert -o out.png` (nonexistent input) → exit 3.
#[test]
fn edit_missing_input_exits_3() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args(["edit", "nope.png", "--invert", "-o", out.to_str().unwrap()])
        .output()
        .expect("failed to run edit missing-input");

    assert_eq!(
        output.status.code(),
        Some(3),
        "missing input must exit 3; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `edit in.png --resize-max 8 --save-recipe r.toml -o out.png` writes both
/// `out.png` and a valid `r.toml` containing `version = "1"`, `op = "resize"`,
/// and `width = 8`.
#[test]
fn edit_save_recipe_writes_parseable_toml() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let out = dir.path().join("out.png");
    let recipe = dir.path().join("r.toml");

    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit --save-recipe");

    assert!(
        status.success(),
        "edit --save-recipe must exit 0; got: {:?}",
        status.code()
    );
    assert!(out.exists(), "out.png must be created");
    assert!(recipe.exists(), "r.toml must be created");

    let toml_text = std::fs::read_to_string(&recipe).unwrap();
    assert!(
        toml_text.contains("version = \"1\""),
        "recipe must have version = \"1\"; got: {toml_text}"
    );
    assert!(
        toml_text.contains("op = \"resize\""),
        "recipe must have op = \"resize\"; got: {toml_text}"
    );
    assert!(
        toml_text.contains("width = 8"),
        "recipe must have width = 8; got: {toml_text}"
    );
}

/// Round-trip: `edit in.png --auto-orient --resize-max 8 --invert --save-recipe r.toml
/// -o edit_out.png` then `apply --recipe r.toml in.png -o apply_out.png` produces
/// the same bytes.
#[test]
fn edit_save_recipe_round_trips_through_apply() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 32, 32);
    let edit_out = dir.path().join("edit_out.png");
    let apply_out = dir.path().join("apply_out.png");
    let recipe = dir.path().join("r.toml");

    // Run edit with all three ops and save the recipe.
    let status = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--auto-orient",
            "--resize-max",
            "8",
            "--invert",
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-o",
            edit_out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run edit round-trip");

    assert!(
        status.success(),
        "edit (round-trip) must exit 0; got: {:?}",
        status.code()
    );

    // Apply the saved recipe to the same input.
    let status2 = Command::new(BIN)
        .args([
            "apply",
            "--recipe",
            recipe.to_str().unwrap(),
            input.to_str().unwrap(),
            "-o",
            apply_out.to_str().unwrap(),
            "-y",
        ])
        .status()
        .expect("failed to run apply round-trip");

    assert!(
        status2.success(),
        "apply (round-trip) must exit 0; got: {:?}",
        status2.code()
    );

    // Byte-for-byte equality: same ops, same order, same input → same output.
    let edit_bytes = std::fs::read(&edit_out).unwrap();
    let apply_bytes = std::fs::read(&apply_out).unwrap();
    assert_eq!(
        edit_bytes, apply_bytes,
        "edit output and apply-of-saved-recipe output must be byte-identical"
    );
}

// ── Byte pin: typed order and the surviving outputs of the original flags ─────

/// A seeded RGB8 noise image: xorshift64 from `seed`, one step per sample, each
/// sample `[0, 30, 255][s % 3]`, row-major, R then G then B. It is written here,
/// not taken from crustyimg, so the pins below are independent of the code under test.
fn noise_rgb8(w: u32, h: u32, seed: u64) -> RgbImage {
    let mut s = seed;
    let mut raw = Vec::with_capacity((w * h * 3) as usize);
    for _ in 0..(w * h * 3) {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        raw.push([0u8, 30, 255][(s % 3) as usize]);
    }
    RgbImage::from_raw(w, h, raw).unwrap()
}

/// FNV-1a 64 over a byte slice.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Run `edit n.png ARGS -o out.png -y --save-recipe r.toml` on the 61×41 noise
/// fixture (seed 7). Returns the decoded output and the saved recipe text.
fn edit_noise(args: &[&str]) -> (image::RgbImage, String) {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("n.png");
    noise_rgb8(61, 41, 7).save(&input).unwrap();
    let out = dir.path().join("out.png");
    let recipe = dir.path().join("r.toml");
    let output = Command::new(BIN)
        .arg("edit")
        .arg(&input)
        .args(args)
        .args(["-o", out.to_str().unwrap(), "-y", "--save-recipe"])
        .arg(&recipe)
        .output()
        .expect("failed to run edit");
    assert!(
        output.status.success(),
        "edit {args:?} must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let px = image::open(&out).unwrap().to_rgb8();
    (px, std::fs::read_to_string(&recipe).unwrap())
}

/// The four invocations whose output does not depend on the order rule keep their
/// decoded pixels and their saved recipe text byte for byte. The hashes were
/// recorded from the 0.8.2 release build, decoded by an independent decoder.
#[test]
fn edit_byte_pin_unchanged_invocations() {
    let head = "version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n";
    let resize = "\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 30\n";
    let invert = "\n[[step]]\nop = \"invert\"\n";
    let rows: [(&[&str], (u32, u32), u64, String); 4] = [
        (
            &["--resize-max", "30"],
            (30, 20),
            0x01a0_3151_2f8d_2eaf,
            format!("{head}{resize}"),
        ),
        (
            &["--invert"],
            (61, 41),
            0xfd42_fcdc_e5e3_c92e,
            format!("{head}{invert}"),
        ),
        (
            &["--resize-max", "30", "--invert"],
            (30, 20),
            0x62d0_5258_55cf_f6a7,
            format!("{head}{resize}{invert}"),
        ),
        (
            &["--auto-orient", "--resize-max", "30"],
            (30, 20),
            0x01a0_3151_2f8d_2eaf,
            format!("{head}{resize}"),
        ),
    ];
    for (args, dims, hash, text) in rows {
        let (px, recipe) = edit_noise(args);
        assert_eq!(px.dimensions(), dims, "pixel pin {args:?}: dimensions");
        assert_eq!(
            fnv1a64(px.as_raw()),
            hash,
            "pixel pin {args:?}: got {:#x}",
            fnv1a64(px.as_raw())
        );
        assert_eq!(recipe, text, "recipe pin {args:?}");
    }
}

/// Op flags run in the order typed: `--invert --resize-max 30` inverts, then
/// resizes. The old canonical order resized first and hashed `0x62d0525855cff6a7`.
/// The fixture is noise, where the two orders differ (a solid colour cannot tell
/// them apart).
#[test]
fn edit_invert_then_resize_runs_in_typed_order() {
    let (typed, recipe) = edit_noise(&["--invert", "--resize-max", "30"]);
    assert_eq!(typed.dimensions(), (30, 20));
    assert_eq!(
        fnv1a64(typed.as_raw()),
        0x5d24_8c13_6b8b_3ad7,
        "got {:#x}",
        fnv1a64(typed.as_raw())
    );
    let (other, _) = edit_noise(&["--resize-max", "30", "--invert"]);
    assert_ne!(
        typed.as_raw(),
        other.as_raw(),
        "the two orders must produce different pixels on noise"
    );
    assert_eq!(
        recipe,
        "version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n\n[[step]]\nop = \"invert\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 30\n"
    );
}

/// `--save-recipe` to an unwritable path (nonexistent directory) → exit 5.
#[test]
fn edit_save_recipe_unwritable_exits_5() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 16, 16);
    let out = dir.path().join("out.png");
    // Point save-recipe at a path whose parent directory does not exist.
    let bad_recipe = dir.path().join("no_such_dir").join("r.toml");

    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            bad_recipe.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run edit unwritable recipe");

    assert_eq!(
        output.status.code(),
        Some(5),
        "unwritable recipe path must exit 5; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ── SPEC-037: --save-recipe symlink guard (DEC-035) ──────────────────────────

/// `edit --save-recipe LINK` where LINK is a symlink → exits 5; target untouched.
///
/// A planted symlink at the recipe path is refused (DEC-035). The symlink's
/// target file must not be modified.
#[cfg(unix)]
#[test]
fn edit_save_recipe_through_symlink_is_rejected() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 8, 8);
    let out = dir.path().join("out.png");

    // Plant a symlink: link.toml → outside.toml
    let outside = dir.path().join("outside.toml");
    let sentinel = b"original sentinel bytes";
    std::fs::write(&outside, sentinel).unwrap();
    let link = dir.path().join("link.toml");
    std::os::unix::fs::symlink(&outside, &link).unwrap();

    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            link.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run edit with symlinked recipe path");

    assert_eq!(
        output.status.code(),
        Some(5),
        "symlinked recipe path must exit 5; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The symlink's target must be untouched.
    let after = std::fs::read(&outside).unwrap();
    assert_eq!(
        after, sentinel,
        "outside target must not be modified by the refused write"
    );
}

/// `edit --save-recipe r.toml` to a plain (non-symlink) path writes the recipe; exit 0.
///
/// Regression guard: the symlink guard must not block legitimate recipe saves.
#[test]
fn edit_save_recipe_normal_path_still_works() {
    let dir = TempDir::new().unwrap();
    let input = write_png(&dir, "in.png", 8, 8);
    let out = dir.path().join("out.png");
    let recipe = dir.path().join("r.toml");

    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "8",
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run edit with normal recipe path");

    assert_eq!(
        output.status.code(),
        Some(0),
        "normal --save-recipe must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(recipe.exists(), "recipe file must be written");
}
