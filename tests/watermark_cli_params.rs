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
    ];
    assert_eq!(rows.len(), 10, "ten rows");

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
    let line = help
        .lines()
        .find(|l| l.contains("--size"))
        .expect("a --size line");
    // The description may wrap onto the next line; take both.
    let after = help.split(line).nth(1).unwrap_or("");
    let described = format!("{line}\n{}", after.lines().next().unwrap_or(""));
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
