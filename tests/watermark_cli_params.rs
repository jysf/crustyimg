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

/// The same gradient as 16-bit RGB: every 8-bit sample times 257, so the
/// 8-bit view is the 8-bit gradient exactly.
fn write_gradient16(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let img = image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_fn(w, h, |x, y| {
        Rgb([
            (x * 255 / w.max(1)) as u16 * 257,
            (y * 255 / h.max(1)) as u16 * 257,
            ((x + y) * 7 % 256) as u16 * 257,
        ])
    });
    let path = dir.join(name);
    DynamicImage::ImageRgb16(img).save(&path).unwrap();
    path
}

/// An all-white 8-bit RGB PNG, `w×h`, written to `dir/name`.
fn write_white(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let path = dir.join(name);
    DynamicImage::ImageRgb8(RgbImage::from_pixel(w, h, Rgb([255, 255, 255])))
        .save(&path)
        .unwrap();
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
// Twelve marks the CLI accepts, each also run as a one-step recipe. Their
// decoded output is hashed and compared with goldens captured from code that
// predates the flag checks, so a change to size, scale, resize filter or
// placement fails here. Regenerate only with `CRUSTYIMG_BLESS=1`, and only on
// the code the goldens describe.

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/watermark/cli_params.golden")
}

/// The base image a case marks.
#[derive(Clone, Copy)]
enum Base {
    /// 8-bit RGB gradient.
    Gradient(u32, u32),
    /// The same gradient as 16-bit RGB.
    Gradient16(u32, u32),
    /// 8-bit RGB, every sample 255.
    White(u32, u32),
}

impl Base {
    fn write(self, dir: &Path, name: &str) -> PathBuf {
        match self {
            Base::Gradient(w, h) => write_gradient(dir, name, w, h),
            Base::Gradient16(w, h) => write_gradient16(dir, name, w, h),
            Base::White(w, h) => write_white(dir, name, w, h),
        }
    }
}

/// `(case name, base, CLI args, recipe step body)`.
type AcceptedCase = (&'static str, Base, Vec<&'static str>, &'static str);

fn accepted_cases() -> Vec<AcceptedCase> {
    vec![
        (
            "text_default_size",
            Base::Gradient(400, 300),
            vec!["--text", "© crustyimg"],
            "text = \"© crustyimg\"",
        ),
        (
            "text_size_1024",
            Base::Gradient(1200, 1000),
            vec!["--text", ".", "--size", "1024"],
            "text = \".\"\nsize = 1024",
        ),
        (
            "image_scale_quarter",
            Base::Gradient(400, 300),
            vec!["--image", "logo.png", "--scale", "0.25"],
            "image = \"logo.png\"\nscale = 0.25",
        ),
        (
            "image_opacity_0_6",
            Base::Gradient(400, 300),
            vec!["--image", "logo.png", "--opacity", "0.6"],
            "image = \"logo.png\"\nopacity = 0.6",
        ),
        (
            "image_tile",
            Base::Gradient(400, 300),
            vec!["--image", "logo.png", "--tile"],
            "image = \"logo.png\"\ntile = true",
        ),
        (
            "image_scale_tenth",
            Base::Gradient(400, 300),
            vec!["--image", "logo.png", "--scale", "0.1"],
            "image = \"logo.png\"\nscale = 0.1",
        ),
        (
            "text_16bit",
            Base::Gradient16(400, 300),
            vec!["--text", "© crustyimg"],
            "text = \"© crustyimg\"",
        ),
        // The faint rows are marks with a visible pixel that a refusal of
        // "draws nothing" must not touch.
        (
            "faint_opacity",
            Base::Gradient16(400, 300),
            vec!["--image", "logo.png", "--opacity", "0.00001"],
            "image = \"logo.png\"\nopacity = 0.00001",
        ),
        (
            "faint_opacity_boundary",
            Base::Gradient16(400, 300),
            vec!["--image", "logo.png", "--opacity", "0.0000077"],
            "image = \"logo.png\"\nopacity = 0.0000077",
        ),
        (
            "faint_size",
            Base::Gradient16(400, 300),
            vec!["--text", "© crustyimg", "--size", "0.2"],
            "text = \"© crustyimg\"\nsize = 0.2",
        ),
        (
            "faint_8bit",
            Base::Gradient(400, 300),
            vec!["--image", "logo.png", "--opacity", "0.001"],
            "image = \"logo.png\"\nopacity = 0.001",
        ),
        (
            "white_on_white",
            Base::White(400, 300),
            vec!["--text", "© crustyimg"],
            "text = \"© crustyimg\"",
        ),
    ]
}

/// Runs one case through the CLI and through a one-step recipe; returns both
/// output hashes.
fn run_case(args: &[&str], recipe_body: &str, base: Base) -> (String, String) {
    let dir = TempDir::new().unwrap();
    write_logo(dir.path());
    let input = base.write(dir.path(), "in.png");

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
    assert_eq!(golden.lines().count(), 12, "twelve goldens");
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

// ─── CLI helpers ─────────────────────────────────────────────────────────────

fn error_lines(stderr: &str) -> Vec<&str> {
    stderr.lines().filter(|l| l.starts_with("error:")).collect()
}

fn files_in(dir: &Path) -> usize {
    std::fs::read_dir(dir).map(|e| e.count()).unwrap_or(0)
}

const SIZE_PHRASE: &str = "a number > 0 and <= 1024";
const SCALE_PHRASE: &str = "a finite number > 0";

/// Runs `crustyimg watermark <inputs…> <args…>` in `dir`.
fn watermark(dir: &Path, inputs: &[&str], args: &[&str]) -> std::process::Output {
    Command::new(BIN)
        .current_dir(dir)
        .arg("watermark")
        .args(inputs)
        .args(args)
        .output()
        .unwrap()
}

// ─── AC-4: a bad value is a usage error naming the flag ──────────────────────

#[test]
fn a_bad_flag_value_is_a_usage_error_naming_the_flag() {
    // (base size, args, expected stderr line)
    let rows: Vec<((u32, u32), Vec<&str>, String)> = vec![
        (
            (400, 300),
            vec!["--text", "hi", "--size", "nan"],
            format!("error: --size must be {SIZE_PHRASE}, got nan"),
        ),
        (
            (400, 300),
            vec!["--text", "hi", "--size", "inf"],
            format!("error: --size must be {SIZE_PHRASE}, got inf"),
        ),
        (
            (400, 300),
            vec!["--text", "hi", "--size", "1e39"],
            format!("error: --size must be {SIZE_PHRASE}, got inf"),
        ),
        (
            (1200, 1000),
            vec!["--text", ".", "--size", "1025"],
            format!("error: --size must be {SIZE_PHRASE}, got 1025"),
        ),
        (
            (400, 300),
            vec!["--text", "hi", "--size=-1"],
            format!("error: --size must be {SIZE_PHRASE}, got -1"),
        ),
        (
            // The longer string is load-bearing: `hi` at 100000 is fit-refused
            // on the old code, only this one reaches the rasteriser and panics.
            (400, 300),
            vec!["--text", "© crustyimg", "--size", "100000"],
            format!("error: --size must be {SIZE_PHRASE}, got 100000"),
        ),
        (
            (400, 300),
            vec!["--image", "logo.png", "--scale", "nan"],
            format!("error: --scale must be {SCALE_PHRASE}, got nan"),
        ),
        (
            (400, 300),
            vec!["--image", "logo.png", "--scale", "inf"],
            format!("error: --scale must be {SCALE_PHRASE}, got inf"),
        ),
        (
            (400, 300),
            vec!["--image", "logo.png", "--scale", "1e39"],
            format!("error: --scale must be {SCALE_PHRASE}, got inf"),
        ),
        (
            (400, 300),
            vec!["--image", "logo.png", "--opacity", "1.5"],
            "error: --opacity must be a number in 0.0..=1.0, got 1.5".to_owned(),
        ),
        (
            // 1.1 and 1024.1 are not exact in f32; the message must print the
            // value as parsed, not the widened f64 (1.100000023841858).
            (400, 300),
            vec!["--image", "logo.png", "--opacity", "1.1"],
            "error: --opacity must be a number in 0.0..=1.0, got 1.1".to_owned(),
        ),
        (
            (1200, 1000),
            vec!["--text", ".", "--size", "1024.1"],
            format!("error: --size must be {SIZE_PHRASE}, got 1024.1"),
        ),
    ];
    assert_eq!(rows.len(), 12, "twelve rows");

    for (base, args, expected) in rows {
        let dir = TempDir::new().unwrap();
        write_logo(dir.path());
        write_gradient(dir.path(), "in.png", base.0, base.1);
        let out = dir.path().join("out.png");
        let output = watermark(
            dir.path(),
            &["in.png"],
            &[&args[..], &["-o", "out.png"]].concat(),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
        assert_eq!(error_lines(&stderr), vec![expected.as_str()], "{args:?}");
        assert!(!out.exists(), "{args:?}: no output may be written");
    }
}

// ─── AC-5: the boundary, and the documented f32 residual ─────────────────────

#[test]
fn size_1024_is_accepted_and_1025_refused_on_the_cli() {
    let dir = TempDir::new().unwrap();
    let input = write_gradient(dir.path(), "in.png", 1200, 1000);

    let output = watermark(
        dir.path(),
        &["in.png"],
        &["--text", ".", "--size", "1024", "-o", "ok.png"],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(
        pixel_hash(&dir.path().join("ok.png")),
        pixel_hash(&input),
        "a mark is drawn at the cap"
    );

    let output = watermark(
        dir.path(),
        &["in.png"],
        &["--text", ".", "--size", "1025", "-o", "no.png"],
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!dir.path().join("no.png").exists());
}

#[test]
fn the_f32_residual_is_accepted_on_the_cli() {
    let dir = TempDir::new().unwrap();
    write_gradient(dir.path(), "in.png", 1200, 1000);
    for (size, out) in [("1024", "a.png"), ("1024.00001", "b.png")] {
        let output = watermark(
            dir.path(),
            &["in.png"],
            &["--text", ".", "--size", size, "-o", out],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{size}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_eq!(
        std::fs::read(dir.path().join("a.png")).unwrap(),
        std::fs::read(dir.path().join("b.png")).unwrap(),
        "1024.00001 reads as 1024 in an f32"
    );
}

// ─── AC-6: input arity does not change the answer ────────────────────────────

#[test]
fn a_bad_flag_is_refused_before_any_input_is_read() {
    use std::io::Write;
    use std::process::Stdio;

    // (flag args, expected line)
    let flags: [(Vec<&str>, String); 2] = [
        (
            vec!["--text", "hi", "--size", "nan"],
            format!("error: --size must be {SIZE_PHRASE}, got nan"),
        ),
        (
            vec!["--image", "logo.png", "--scale", "nan"],
            format!("error: --scale must be {SCALE_PHRASE}, got nan"),
        ),
    ];
    let mut rows = 0;

    for (flag_args, expected) in &flags {
        for shape in ["one", "batch", "glob", "stdin", "missing"] {
            rows += 1;
            let dir = TempDir::new().unwrap();
            write_logo(dir.path());
            write_gradient(dir.path(), "a1.png", 400, 300);
            let outdir = dir.path().join("outdir");
            let single = dir.path().join("single.png");

            let output = match shape {
                "one" => watermark(
                    dir.path(),
                    &["a1.png"],
                    &[&flag_args[..], &["-o", "single.png"]].concat(),
                ),
                "batch" => {
                    std::fs::create_dir(dir.path().join("batch")).unwrap();
                    write_gradient(&dir.path().join("batch"), "b1.png", 400, 300);
                    write_gradient(&dir.path().join("batch"), "b2.png", 400, 300);
                    watermark(
                        dir.path(),
                        &["batch"],
                        &[&flag_args[..], &["--out-dir", "outdir"]].concat(),
                    )
                }
                // One argument, no shell: the pattern is expanded by the verb.
                "glob" => watermark(
                    dir.path(),
                    &["a*.png"],
                    &[&flag_args[..], &["-o", "single.png"]].concat(),
                ),
                "missing" => watermark(
                    dir.path(),
                    &["/no/such.png"],
                    &[&flag_args[..], &["-o", "single.png"]].concat(),
                ),
                "stdin" => {
                    let mut png = std::io::Cursor::new(Vec::new());
                    DynamicImage::ImageRgb8(RgbImage::from_pixel(400, 300, Rgb([10, 20, 30])))
                        .write_to(&mut png, image::ImageFormat::Png)
                        .unwrap();
                    let mut child = Command::new(BIN)
                        .current_dir(dir.path())
                        .arg("watermark")
                        .arg("-")
                        .args(flag_args)
                        .args(["-o", "single.png"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .unwrap();
                    // The process may exit before reading; a broken pipe is fine.
                    let mut stdin = child.stdin.take().unwrap();
                    let _ = stdin.write_all(&png.into_inner());
                    drop(stdin);
                    child.wait_with_output().unwrap()
                }
                _ => unreachable!(),
            };
            let stderr = String::from_utf8_lossy(&output.stderr);
            let label = format!("{flag_args:?} on {shape}");
            assert_eq!(output.status.code(), Some(2), "{label}: {stderr}");
            assert_eq!(error_lines(&stderr), vec![expected.as_str()], "{label}");
            assert!(!single.exists(), "{label}: no -o output");
            assert_eq!(files_in(&outdir), 0, "{label}: no --out-dir output");
        }
    }
    assert_eq!(rows, 10, "ten rows");
}

// ─── AC-7: a bad flag wins over a missing font or overlay ────────────────────

#[test]
fn a_bad_flag_is_refused_before_the_font_or_overlay_is_read() {
    // (args, line it must print, or a word the line must contain)
    let size_line = format!("error: --size must be {SIZE_PHRASE}, got nan");
    let scale_line = format!("error: --scale must be {SCALE_PHRASE}, got nan");
    let opacity_line = "error: --opacity must be a number in 0.0..=1.0, got 2".to_owned();
    let rows: Vec<(Vec<&str>, Result<String, &str>)> = vec![
        (
            vec!["--text", "hi", "--font", "/no/f.ttf", "--size", "nan"],
            Ok(size_line),
        ),
        (
            vec!["--image", "/no/l.png", "--scale", "nan"],
            Ok(scale_line),
        ),
        (
            vec!["--image", "/no/l.png", "--opacity", "2"],
            Ok(opacity_line),
        ),
        (
            vec!["--image", "/no/l.png", "--gravity", "bogus"],
            Err("bogus"),
        ),
        (
            vec!["--text", "hi", "--font", "/no/f.ttf", "--color", "zzz"],
            Err("zzz"),
        ),
    ];
    assert_eq!(rows.len(), 5, "five rows");

    let dir = TempDir::new().unwrap();
    write_gradient(dir.path(), "in.png", 400, 300);
    for (args, want) in rows {
        let output = watermark(
            dir.path(),
            &["in.png"],
            &[&args[..], &["-o", "out.png"]].concat(),
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
        match want {
            Ok(line) => assert_eq!(error_lines(&stderr), vec![line.as_str()], "{args:?}"),
            Err(word) => assert!(stderr.contains(word), "{args:?}: {stderr}"),
        }
        assert!(!dir.path().join("out.png").exists());
    }
}

// ─── AC-8: the help states the cap ───────────────────────────────────────────

#[test]
fn size_help_states_the_cap() {
    let output = Command::new(BIN)
        .args(["watermark", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&output.stdout);
    // The option's own entry, not the prose that mentions the flag.
    let lines: Vec<&str> = help.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim_start().starts_with("--size"))
        .expect("a --size entry");
    // The description sits on the lines after the flag, up to the next entry.
    let described = lines[at..]
        .iter()
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(described.contains("1024"), "help: {described}");
}

// ─── AC-11: no refusal prints a saturated size ───────────────────────────────

#[test]
fn an_oversized_scale_names_no_saturated_size() {
    const LINE: &str =
        "error: watermark refused: the scaled mark does not fit inside the 64×48 px image";
    let dir = TempDir::new().unwrap();
    write_logo(dir.path());
    write_gradient(dir.path(), "in.png", 64, 48);

    // Two CLI rows.
    for scale in ["4294967296", "1e30"] {
        let output = watermark(
            dir.path(),
            &["in.png"],
            &["--image", "logo.png", "--scale", scale, "-o", "out.png"],
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "cli {scale}: {stderr}");
        assert_eq!(error_lines(&stderr), vec![LINE], "cli {scale}");
        assert!(!stderr.contains("4294967295"), "cli {scale}: {stderr}");
    }

    // Two recipe rows.
    for scale in ["4294967296", "1e30"] {
        let recipe = dir.path().join("r.toml");
        std::fs::write(
            &recipe,
            format!("version = \"1\"\n\n[[step]]\nop = \"watermark\"\nimage = \"logo.png\"\nscale = {scale}\n"),
        )
        .unwrap();
        let output = Command::new(BIN)
            .current_dir(dir.path())
            .args([
                "apply", "--recipe", "r.toml", "in.png", "-o", "out.png", "-y",
            ])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "recipe {scale}: {stderr}");
        assert_eq!(error_lines(&stderr), vec![LINE], "recipe {scale}");
        assert!(!stderr.contains("4294967295"), "recipe {scale}: {stderr}");
    }
    assert!(!dir.path().join("out.png").exists());

    // Batch: two inputs with different names (one name twice would collide in
    // `--out-dir`). Each is written without the mark, with a warning.
    std::fs::create_dir(dir.path().join("batch")).unwrap();
    write_gradient(&dir.path().join("batch"), "x1.png", 64, 48);
    write_gradient(&dir.path().join("batch"), "x2.png", 64, 48);
    let output = watermark(
        dir.path(),
        &["batch/x1.png", "batch/x2.png"],
        &[
            "--image",
            "logo.png",
            "--scale",
            "4294967296",
            "--out-dir",
            "outdir",
        ],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "batch: {stderr}");
    let warnings: Vec<&str> = stderr
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect();
    assert_eq!(warnings.len(), 2, "two warnings: {stderr}");
    for w in warnings {
        assert!(
            w.ends_with("the scaled mark does not fit inside the 64×48 px image"),
            "{w}"
        );
        assert!(!w.contains("4294967295"), "{w}");
    }
    assert_eq!(files_in(&dir.path().join("outdir")), 2);
}

// ─── A mark that draws nothing is refused ────────────────────────────────────
//
// "Draws nothing" is decided from the mark alone, at 16-bit precision, so the
// boundary rows run on a black 16-bit base: a white pixel that keeps any alpha
// changes a sample there, and one that keeps none changes nothing.

/// A black 16-bit RGB PNG, `w×h`. 400×300, because a 60×30 mark on a smaller
/// base trips the coverage cap.
fn write_black16(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let img = image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_pixel(w, h, Rgb([0, 0, 0]));
    let path = dir.join(name);
    DynamicImage::ImageRgb16(img).save(&path).unwrap();
    assert_eq!(
        image::open(&path).unwrap().color(),
        image::ColorType::Rgb16,
        "control: {name} is a 16-bit file"
    );
    path
}

/// A 60×30 white 8-bit RGBA overlay whose every pixel has alpha `alpha`.
fn write_white_overlay(dir: &Path, name: &str, alpha: u8) -> PathBuf {
    let img = RgbaImage::from_pixel(60, 30, Rgba([255, 255, 255, alpha]));
    let path = dir.join(name);
    DynamicImage::ImageRgba8(img).save(&path).unwrap();
    path
}

/// A 60×30 white 16-bit RGBA overlay whose every pixel has alpha16 4: no
/// alpha at 8 bits, a little at 16.
fn write_faint16(dir: &Path) -> PathBuf {
    let img = image::ImageBuffer::<Rgba<u16>, Vec<u16>>::from_pixel(
        60,
        30,
        Rgba([65535, 65535, 65535, 4]),
    );
    let path = dir.join("faint16.png");
    DynamicImage::ImageRgba16(img).save(&path).unwrap();
    let back = image::open(&path).unwrap();
    assert_eq!(
        back.color(),
        image::ColorType::Rgba16,
        "control: faint16 is a 16-bit file"
    );
    assert_eq!(
        back.to_rgba16().get_pixel(0, 0).0[3],
        4,
        "control: alpha16 4"
    );
    path
}

/// The fixtures every refusal row may name, written into `dir`.
fn write_refusal_fixtures(dir: &Path) {
    write_logo(dir);
    write_gradient(dir, "in.png", 400, 300);
    write_black16(dir, "black16.png", 400, 300);
    write_white_overlay(dir, "white.png", 255);
    write_white_overlay(dir, "logo1.png", 1);
    write_white_overlay(dir, "clear.png", 0);
    write_faint16(dir);
}

/// Runs `watermark <input> <args…> -o out.png -y` in `dir`.
fn watermark_to_out(dir: &Path, input: &str, args: &[&str]) -> std::process::Output {
    watermark(dir, &[input], &[args, &["-o", "out.png", "-y"]].concat())
}

/// Asserts a refusal: exit 2, exactly `expected` on stderr, nothing written and
/// the input untouched.
fn assert_refused_on_cli(dir: &Path, input: &str, args: &[&str], expected: &str) {
    let before = std::fs::read(dir.join(input)).ok();
    let output = watermark_to_out(dir, input, args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{args:?}: {stderr}");
    assert_eq!(error_lines(&stderr), vec![expected], "{args:?}");
    assert!(!dir.join("out.png").exists(), "{args:?}: nothing written");
    assert_eq!(std::fs::read(dir.join(input)).ok(), before, "{args:?}");
}

/// Asserts the mark is drawn: exit 0 and an output that differs from the input.
fn assert_drawn_on_cli(dir: &Path, input: &str, args: &[&str]) {
    let _ = std::fs::remove_file(dir.join("out.png"));
    let output = watermark_to_out(dir, input, args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{args:?}: {stderr}");
    assert_ne!(
        pixel_hash(&dir.join("out.png")),
        pixel_hash(&dir.join(input)),
        "{args:?}: the mark changes a sample"
    );
}

const OPACITY_ZERO_LINE: &str =
    "error: --opacity 0 draws no mark; to leave an image unmarked, do not run watermark";
const BLANK_TEXT_LINE: &str =
    "error: --text has no visible characters; use text with at least one visible character";
const TRANSPARENT_COLOR_LINE: &str =
    "error: --color ffffff00 is fully transparent; use an alpha above 00, or RRGGBB";
const TRANSPARENT_IMAGE_LINE: &str =
    "error: --image clear.png is fully transparent; use an image with visible pixels";

#[test]
fn a_mark_that_draws_nothing_is_refused_on_the_cli() {
    let faint = |opacity: &str| {
        format!(
            "error: --opacity {opacity} is too faint: the mark would draw no visible pixel; \
             use a larger --opacity"
        )
    };
    let rows: Vec<(Vec<&str>, String)> = vec![
        (
            vec!["--text", "hi", "--opacity", "0"],
            OPACITY_ZERO_LINE.into(),
        ),
        (
            vec!["--image", "logo.png", "--opacity", "0"],
            OPACITY_ZERO_LINE.into(),
        ),
        (
            vec!["--image", "white.png", "--opacity", "0.000001"],
            faint("0.000001"),
        ),
        (
            vec!["--image", "faint16.png", "--opacity", "0.1"],
            faint("0.1"),
        ),
        (vec!["--text", " "], BLANK_TEXT_LINE.into()),
        // A no-break space maps to an empty glyph, like a plain space.
        (vec!["--text", " \u{a0}"], BLANK_TEXT_LINE.into()),
        (
            vec!["--text", "© crustyimg", "--size", "0.01"],
            "error: --text draws no visible pixel at --size 0.01; use a larger --size".into(),
        ),
        (
            vec!["--text", "hi", "--color", "ffffff00"],
            TRANSPARENT_COLOR_LINE.into(),
        ),
        (
            vec!["--text", "hi", "--color", "#ffffff00"],
            "error: --color #ffffff00 is fully transparent; use an alpha above 00, or RRGGBB"
                .into(),
        ),
        (vec!["--image", "clear.png"], TRANSPARENT_IMAGE_LINE.into()),
    ];
    assert_eq!(rows.len(), 10, "ten rows");

    let dir = TempDir::new().unwrap();
    write_refusal_fixtures(dir.path());
    for (args, expected) in &rows {
        assert_refused_on_cli(dir.path(), "in.png", args, expected);
    }
}

#[test]
fn the_opacity_threshold_is_where_the_last_alpha_rounds_away_on_the_cli() {
    let dir = TempDir::new().unwrap();
    write_refusal_fixtures(dir.path());

    // An opaque mark keeps alpha16 1 from 0.0000077 (65535 × 0.0000077 = 0.505).
    assert_refused_on_cli(
        dir.path(),
        "black16.png",
        &["--image", "white.png", "--opacity", "0.0000076"],
        "error: --opacity 0.0000076 is too faint: the mark would draw no visible pixel; \
         use a larger --opacity",
    );
    assert_drawn_on_cli(
        dir.path(),
        "black16.png",
        &["--image", "white.png", "--opacity", "0.0000077"],
    );

    // A mark of alpha8 1 (alpha16 257) needs 0.002: 257 × 0.0019 = 0.488. A
    // threshold taken from 65535 alone would accept 0.0019.
    assert_refused_on_cli(
        dir.path(),
        "black16.png",
        &[
            "--text",
            "© crustyimg",
            "--color",
            "ffffff01",
            "--opacity",
            "0.0019",
        ],
        "error: --opacity 0.0019 is too faint: the mark would draw no visible pixel; \
         use a larger --opacity",
    );
    assert_drawn_on_cli(
        dir.path(),
        "black16.png",
        &[
            "--text",
            "© crustyimg",
            "--color",
            "ffffff01",
            "--opacity",
            "0.002",
        ],
    );
}

#[test]
fn the_size_threshold_is_where_the_text_first_draws() {
    let dir = TempDir::new().unwrap();
    write_refusal_fixtures(dir.path());
    assert_refused_on_cli(
        dir.path(),
        "black16.png",
        &["--text", "© crustyimg", "--size", "0.17"],
        "error: --text draws no visible pixel at --size 0.17; use a larger --size",
    );
    assert_drawn_on_cli(
        dir.path(),
        "black16.png",
        &["--text", "© crustyimg", "--size", "0.18"],
    );
}

#[test]
fn a_mark_that_draws_nothing_is_refused_before_any_input_is_read() {
    let dir = TempDir::new().unwrap();
    write_refusal_fixtures(dir.path());
    let mut rows = 0;

    // (input, args, expected exit, expected line)
    let single: Vec<(&str, Vec<&str>, i32, Option<&str>)> = vec![
        // Flag checks come before the input read…
        (
            "/no/such.png",
            vec!["--text", "hi", "--opacity", "0"],
            2,
            Some(OPACITY_ZERO_LINE),
        ),
        // …and before the overlay read…
        (
            "in.png",
            vec!["--image", "/no/l.png", "--opacity", "0"],
            2,
            Some(OPACITY_ZERO_LINE),
        ),
        // …and before the font read.
        (
            "in.png",
            vec!["--text", "hi", "--font", "/no/f.ttf", "--color", "ffffff00"],
            2,
            Some(TRANSPARENT_COLOR_LINE),
        ),
        // A mark check needs the rendered mark, and comes before the input.
        (
            "/no/such.png",
            vec!["--text", " "],
            2,
            Some(BLANK_TEXT_LINE),
        ),
        (
            "/no/such.png",
            vec!["--image", "clear.png"],
            2,
            Some(TRANSPARENT_IMAGE_LINE),
        ),
        // Blank text is not a flag check (a tab draws a box), so the font is
        // read first and a missing one is an input error.
        (
            "in.png",
            vec!["--text", " ", "--font", "/no/f.ttf"],
            3,
            None,
        ),
    ];
    for (input, args, code, line) in &single {
        rows += 1;
        let _ = std::fs::remove_file(dir.path().join("out.png"));
        let output = watermark_to_out(dir.path(), input, args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(*code), "{args:?}: {stderr}");
        if let Some(line) = line {
            assert_eq!(error_lines(&stderr), vec![*line], "{args:?}");
        }
        assert!(!dir.path().join("out.png").exists(), "{args:?}");
    }

    // A batch: refused as a whole, nothing written, every input untouched.
    rows += 1;
    let batch = dir.path().join("batch");
    std::fs::create_dir(&batch).unwrap();
    write_gradient(&batch, "b1.png", 400, 300);
    write_gradient(&batch, "b2.png", 400, 300);
    let before: Vec<Vec<u8>> = ["b1.png", "b2.png"]
        .iter()
        .map(|n| std::fs::read(batch.join(n)).unwrap())
        .collect();
    let output = watermark(
        dir.path(),
        &["batch"],
        &["--text", "hi", "--opacity", "0", "--out-dir", "out"],
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "batch: {stderr}");
    assert_eq!(error_lines(&stderr), vec![OPACITY_ZERO_LINE], "batch");
    assert_eq!(
        files_in(&dir.path().join("out")),
        0,
        "batch: nothing written"
    );
    for (n, bytes) in ["b1.png", "b2.png"].iter().zip(&before) {
        assert_eq!(&std::fs::read(batch.join(n)).unwrap(), bytes, "{n}");
    }
    assert_eq!(rows, 7, "seven rows");
}

#[test]
fn a_faint_or_odd_mark_that_draws_is_accepted() {
    let dir = TempDir::new().unwrap();
    write_refusal_fixtures(dir.path());
    write_gradient16(dir.path(), "g16.png", 400, 300);
    let mut rows = 0;

    // A faint mark that keeps alpha at 16 bits is drawn on a 16-bit image.
    rows += 1;
    assert_drawn_on_cli(
        dir.path(),
        "g16.png",
        &["--image", "logo.png", "--opacity", "0.001"],
    );
    // A 16-bit overlay with no alpha at 8 bits draws at opacity 1.
    rows += 1;
    assert_drawn_on_cli(dir.path(), "black16.png", &["--image", "faint16.png"]);
    // Whitespace the bundled font has no glyph for draws its fallback box.
    for text in ["\t", "\u{3000}", "\u{200d}"] {
        rows += 1;
        assert_drawn_on_cli(dir.path(), "in.png", &["--text", text]);
    }
    // White on white changes nothing, and is still drawn as asked.
    rows += 1;
    write_white(dir.path(), "white_base.png", 400, 300);
    let output = watermark_to_out(dir.path(), "white_base.png", &["--text", "© crustyimg"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(rows, 6, "six rows");
}

#[test]
fn opacity_help_states_the_range() {
    let output = Command::new(BIN)
        .args(["watermark", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = help.lines().collect();
    let at = lines
        .iter()
        .position(|l| l.trim_start().starts_with("--opacity"))
        .expect("an --opacity entry");
    let described = lines[at..]
        .iter()
        .take(3)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(described.contains("above 0"), "help: {described}");
}
