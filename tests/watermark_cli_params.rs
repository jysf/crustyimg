//! The `watermark` verb checks `--size`, `--scale` and `--opacity` against the
//! same key table a recipe step is checked against, and reports a bad flag
//! before it reads any file. A mark that is accepted draws exactly as before.
//!
//! These tests drive the real binary, because "nothing was read" and "nothing
//! was written" are observables of the process. Inputs are generated in-test
//! with the `image` crate, never with crustyimg. `.unwrap()` is idiomatic test
//! setup here (the `no-unwrap` constraint is scoped to `src/**`).

use std::path::{Path, PathBuf};
use std::process::Command;

use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// An RGB gradient PNG, `w×h`, written to `dir/name`.
fn write_gradient(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let img = RgbImage::from_fn(w, h, |x, y| {
        Rgb([
            (x * 255 / w.max(1)) as u8,
            (y * 255 / h.max(1)) as u8,
            ((x + y) * 7 % 256) as u8,
        ])
    });
    let path = dir.join(name);
    DynamicImage::ImageRgb8(img).save(&path).unwrap();
    path
}

/// A 60×30 opaque RGBA *gradient* overlay. A flat colour would hide a change
/// of resize filter, so every pixel differs from its neighbours.
fn write_logo(dir: &Path) -> PathBuf {
    let img = RgbaImage::from_fn(60, 30, |x, y| {
        Rgba([
            (x * 4 % 256) as u8,
            (y * 8 % 256) as u8,
            ((x * y) % 256) as u8,
            255,
        ])
    });
    let path = dir.join("logo.png");
    DynamicImage::ImageRgba8(img).save(&path).unwrap();
    path
}

/// SHA-256 over the decoded output's size, colour type and raw samples (the
/// `tests/watermark_damage.rs` scheme); PNG file bytes are not hashed.
fn pixel_hash(path: &Path) -> String {
    let img = image::open(path).unwrap();
    let mut h = Sha256::new();
    h.update(img.width().to_le_bytes());
    h.update(img.height().to_le_bytes());
    h.update(format!("{:?}", img.color()).as_bytes());
    h.update(img.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

// ─── Byte identity of a mark that is accepted ────────────────────────────────
//
// Three marks the CLI accepts, each also run as a one-step recipe. Their
// decoded output is hashed and compared with goldens captured from code that
// predates the flag checks, so a change to size, scale, resize filter or
// placement fails here. Regenerate only with `CRUSTYIMG_BLESS=1`, and only on
// the code the goldens describe.

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/watermark/cli_params.golden")
}

/// `(case name, base size, CLI args, recipe step body)`.
fn accepted_cases() -> Vec<(&'static str, (u32, u32), Vec<&'static str>, &'static str)> {
    vec![
        (
            "text_default_size",
            (400, 300),
            vec!["--text", "© crustyimg"],
            "text = \"© crustyimg\"",
        ),
        (
            "text_size_1024",
            (1200, 1000),
            vec!["--text", ".", "--size", "1024"],
            "text = \".\"\nsize = 1024",
        ),
        (
            "image_scale_quarter",
            (400, 300),
            vec!["--image", "logo.png", "--scale", "0.25"],
            "image = \"logo.png\"\nscale = 0.25",
        ),
    ]
}

/// Runs one case through the CLI and through a one-step recipe; returns both
/// output hashes.
fn run_case(args: &[&str], recipe_body: &str, base: (u32, u32)) -> (String, String) {
    let dir = TempDir::new().unwrap();
    write_logo(dir.path());
    let input = write_gradient(dir.path(), "in.png", base.0, base.1);

    let cli_out = dir.path().join("cli.png");
    let status = Command::new(BIN)
        .current_dir(dir.path())
        .arg("watermark")
        .arg(&input)
        .args(args)
        .arg("-o")
        .arg(&cli_out)
        .arg("-y")
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "cli {args:?}: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    let recipe = dir.path().join("r.toml");
    std::fs::write(
        &recipe,
        format!("version = \"1\"\n\n[[step]]\nop = \"watermark\"\n{recipe_body}\n"),
    )
    .unwrap();
    let recipe_out = dir.path().join("recipe.png");
    let status = Command::new(BIN)
        .current_dir(dir.path())
        .arg("apply")
        .arg("--recipe")
        .arg(&recipe)
        .arg(&input)
        .arg("-o")
        .arg(&recipe_out)
        .arg("-y")
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "recipe {recipe_body:?}: {}",
        String::from_utf8_lossy(&status.stderr)
    );

    (pixel_hash(&cli_out), pixel_hash(&recipe_out))
}

/// `name hash` lines, one per case, in case order (the CLI's hash).
fn current_hashes() -> String {
    accepted_cases()
        .into_iter()
        .map(|(name, base, args, body)| {
            let (cli, recipe) = run_case(&args, body, base);
            assert_eq!(cli, recipe, "{name}: the recipe draws what the CLI draws");
            format!("{name} {cli}\n")
        })
        .collect()
}

#[test]
fn accepted_marks_are_byte_identical_to_main() {
    let golden = std::fs::read_to_string(golden_path())
        .expect("golden file")
        .replace("\r\n", "\n");
    assert_eq!(golden.lines().count(), 3, "three goldens");
    assert_eq!(current_hashes(), golden);
}

/// Writes the golden file. Not an assertion: run it with
/// `CRUSTYIMG_BLESS=1 cargo test --test watermark_cli_params -- --ignored bless`
/// only when the goldens should describe the current code.
#[test]
#[ignore]
fn bless_cli_params_goldens() {
    if std::env::var_os("CRUSTYIMG_BLESS").is_none() {
        eprintln!("CRUSTYIMG_BLESS is not set; not writing goldens");
        return;
    }
    let path = golden_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, current_hashes()).unwrap();
}
