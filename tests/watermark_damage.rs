//! A watermark that would not fit inside an image, or would cover too much of
//! it, is refused instead of drawn; a mark that passes draws exactly as before.
//!
//! The library half builds `Watermark` ops in memory. Bases are generated
//! natively as gradients (not solid colours), so "the output equals the input"
//! and the golden hashes both depend on every pixel. `.unwrap()` is idiomatic
//! test setup here (the `no-unwrap` constraint is scoped to `src/**`).

use std::path::{Path, PathBuf};

use crustyimg::image::Image;
use crustyimg::operation::{Gravity, Operation, Watermark};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, RgbaImage};
use sha2::{Digest, Sha256};

// ─── Fixtures ────────────────────────────────────────────────────────────────

/// An opaque RGBA gradient base, `w×h`.
fn gradient(w: u32, h: u32) -> RgbaImage {
    RgbaImage::from_fn(w, h, |x, y| {
        Rgba([
            (x * 255 / w.max(1)) as u8,
            (y * 255 / h.max(1)) as u8,
            ((x + y) * 7 % 256) as u8,
            255,
        ])
    })
}

fn image_of(pixels: DynamicImage) -> Image {
    Image::from_parts(pixels, ImageFormat::Png, None)
}

/// An overlay whose every pixel is visible, with varied colour and alpha so a
/// change in compositing would show in the hash.
fn opaque_logo(w: u32, h: u32) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
        Rgba([
            (200 - (x * 3 % 200)) as u8,
            (y * 5 % 256) as u8,
            ((x * y) % 256) as u8,
            255,
        ])
    }))
}

fn image_mark(overlay: DynamicImage, opacity: f32, scale: Option<f32>, tile: bool) -> Watermark {
    Watermark::new_image(
        overlay,
        "logo.png".to_owned(),
        Gravity::SouthEast,
        opacity,
        scale,
        0,
        tile,
    )
}

/// The mark `watermark --text "© crustyimg"` draws with every default.
fn default_text_mark() -> Watermark {
    let color = [255, 255, 255, 255];
    let rendered =
        crustyimg::text::render_text(crustyimg::text::DEFAULT_FONT, "© crustyimg", 32.0, color)
            .unwrap();
    Watermark::new_text(
        DynamicImage::ImageRgba8(rendered),
        "© crustyimg".to_owned(),
        None,
        32.0,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    )
}

// ─── Byte identity of a mark that passes ─────────────────────────────────────
//
// Six marks the damage rule lets through. Their decoded output is hashed and
// compared with goldens captured from code that predates the rule, so a
// change to placement, scaling, opacity or narrowing fails here. Regenerate
// only with `CRUSTYIMG_BLESS=1`, and only on the code the goldens describe.

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/watermark/passing_marks.golden")
}

/// `(case name, op, base)` for every passing case.
fn passing_cases() -> Vec<(&'static str, Watermark, Image)> {
    // A 16-bit base whose low bytes vary, with a part-transparent mark.
    let base16: ImageBuffer<Rgba<u16>, Vec<u16>> = ImageBuffer::from_fn(100, 100, |x, y| {
        Rgba([
            (x * 655 + 17) as u16,
            (y * 641 + 3) as u16,
            ((x * y) % 65536) as u16,
            65535,
        ])
    });
    let soft_logo = DynamicImage::ImageRgba8(RgbaImage::from_fn(10, 10, |x, y| {
        Rgba([250, (x * 20) as u8, (y * 20) as u8, (x * 25 + y) as u8])
    }));
    vec![
        (
            "text_default_on_256",
            default_text_mark(),
            image_of(DynamicImage::ImageRgba8(gradient(256, 256))),
        ),
        (
            "text_default_on_800x600",
            default_text_mark(),
            image_of(DynamicImage::ImageRgba8(gradient(800, 600))),
        ),
        (
            "opaque_64_on_800x600_half_opacity",
            image_mark(opaque_logo(64, 64), 0.5, None, false),
            image_of(DynamicImage::ImageRgba8(gradient(800, 600))),
        ),
        (
            "opaque_64_tiled_on_256",
            image_mark(opaque_logo(64, 64), 1.0, None, true),
            image_of(DynamicImage::ImageRgba8(gradient(256, 256))),
        ),
        (
            "overlay_256_scaled_half_on_24",
            image_mark(opaque_logo(256, 256), 1.0, Some(0.5), false),
            image_of(DynamicImage::ImageRgba8(gradient(24, 24))),
        ),
        (
            "soft_10_on_16_bit_100",
            image_mark(soft_logo, 1.0, None, false),
            image_of(DynamicImage::ImageRgba16(base16)),
        ),
    ]
}

/// SHA-256 over the output's size, colour type and raw samples.
fn pixel_hash(img: &Image) -> String {
    let px = img.pixels();
    let mut h = Sha256::new();
    h.update(px.width().to_le_bytes());
    h.update(px.height().to_le_bytes());
    h.update(format!("{:?}", px.color()).as_bytes());
    h.update(px.as_bytes());
    h.finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

/// `name hash` lines, one per case, in case order.
fn current_hashes() -> String {
    passing_cases()
        .into_iter()
        .map(|(name, op, base)| {
            let out = op
                .apply(base)
                .unwrap_or_else(|e| panic!("{name}: a passing mark must apply: {e}"));
            format!("{name} {}\n", pixel_hash(&out))
        })
        .collect()
}

#[test]
fn passing_marks_are_byte_identical_to_main() {
    let golden = std::fs::read_to_string(golden_path())
        .expect("golden file")
        .replace("\r\n", "\n");
    assert_eq!(golden.lines().count(), 6, "six goldens");
    assert_eq!(current_hashes(), golden);
}

/// Writes the golden file. Not an assertion: run it with
/// `CRUSTYIMG_BLESS=1 cargo test --test watermark_damage -- --ignored bless`
/// only when the goldens should describe the current code.
#[test]
#[ignore]
fn bless_watermark_goldens() {
    if std::env::var_os("CRUSTYIMG_BLESS").is_none() {
        eprintln!("CRUSTYIMG_BLESS is not set; not writing goldens");
        return;
    }
    let path = golden_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, current_hashes()).unwrap();
}

// ─── The recipe: text mode refuses `scale` and `tile = true` ─────────────────

use crustyimg::operation::OperationRegistry;
use crustyimg::recipe::{Recipe, RecipeError};

/// Parse and build a recipe the way `apply` does (an `image` key gets
/// stand-in overlay bytes, as the CLI's resolver attaches).
fn load(toml: &str) -> Result<crustyimg::pipeline::Pipeline, RecipeError> {
    let mut recipe = Recipe::from_toml(toml)?;
    for step in &mut recipe.steps {
        if step.params.get_str("image").is_some() {
            step.params
                .set_resolved_bytes("image", png_bytes(&gradient(4, 4)));
        }
    }
    recipe.build_pipeline(&OperationRegistry::with_builtins())
}

fn watermark_step(body: &str) -> String {
    format!("version = \"1\"\n\n[[step]]\nop = \"watermark\"\n{body}\n")
}

#[test]
fn text_step_refuses_scale_and_tile_true() {
    let rows = [
        (
            "text = \"x\"\ntile = true",
            "'tile = true' only applies to an image watermark (this step sets 'text')",
        ),
        (
            "text = \"x\"\nscale = 0.9",
            "'scale' only applies to an image watermark (this step sets 'text')",
        ),
    ];
    for (body, expected) in rows {
        match load(&watermark_step(body)) {
            Err(RecipeError::InvalidOperation { name, reason, .. }) => {
                assert_eq!(name, "watermark");
                assert_eq!(reason, expected, "for `{body}`");
            }
            Err(other) => panic!("`{body}`: expected InvalidOperation, got {other:?}"),
            Ok(_) => panic!("`{body}`: a text step must refuse it"),
        }
    }
}

#[test]
fn text_step_accepts_tile_false_and_image_steps_keep_scale_and_tile() {
    load(&watermark_step("text = \"x\"\ntile = false")).expect("tile = false is the default");
    load(&watermark_step(
        "image = \"logo.png\"\nscale = 0.5\ntile = true",
    ))
    .expect("an image step takes scale and tile");
    // A text mark as the CLI builds it survives a recipe round trip.
    let (op, _) = text_op("hi", 32.0);
    let ops: Vec<Box<dyn Operation>> = vec![Box::new(op)];
    let recipe = Recipe::from_ops(&ops);
    let toml = recipe.to_toml().unwrap();
    load(&toml).unwrap_or_else(|e| panic!("crustyimg's own text recipe must load: {e}\n{toml}"));
}

fn text_op(text: &str, size: f32) -> (Watermark, (u32, u32)) {
    let color = [255, 255, 255, 255];
    let rendered =
        crustyimg::text::render_text(crustyimg::text::DEFAULT_FONT, text, size, color).unwrap();
    let dims = rendered.dimensions();
    let op = Watermark::new_text(
        DynamicImage::ImageRgba8(rendered),
        text.to_owned(),
        None,
        size,
        color,
        Gravity::SouthEast,
        1.0,
        None,
        0,
        false,
    );
    (op, dims)
}

// ─── The CLI ─────────────────────────────────────────────────────────────────

use std::process::{Command, Output};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

fn png_bytes(img: &RgbaImage) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img.clone())
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

/// Write an opaque gradient PNG `w×h` at `dir/name`.
fn write_base(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, png_bytes(&gradient(w, h))).unwrap();
    path
}

fn write_text(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn decoded(path: &Path) -> RgbaImage {
    image::open(path)
        .unwrap_or_else(|e| panic!("{} should decode: {e}", path.display()))
        .to_rgba8()
}

/// The lines of `stderr` that are skip warnings.
fn skip_lines(err: &str) -> Vec<&str> {
    err.lines()
        .filter(|l| l.contains("watermark skipped, image written without it"))
        .collect()
}

/// Exactly one skip warning, and it names `name`.
fn assert_one_skip_line(err: &str, name: &str) {
    let lines = skip_lines(err);
    assert_eq!(lines.len(), 1, "one skip line expected, stderr:\n{err}");
    assert!(
        lines[0].starts_with("warning: ") && lines[0].contains(name),
        "the skip line must be a warning naming {name}: {}",
        lines[0]
    );
    assert!(
        lines[0].contains("does not fit inside"),
        "the skip line names the reason: {}",
        lines[0]
    );
}

const ROW_A_STEP: &str = "[[step]]\nop = \"watermark\"\ntext = \"© crustyimg\"\nsize = 200.0\n";
const RESIZE_HALF: &str = "[[step]]\nop = \"resize\"\nmode = \"percent\"\npercent = 50\n";
const DEFAULT_TEXT_STEP: &str = "[[step]]\nop = \"watermark\"\ntext = \"© crustyimg\"\n";
const OPTIMIZE_STEP: &str = "[[step]]\nop = \"optimize\"\n";

fn recipe(steps: &[&str]) -> String {
    format!("version = \"1\"\n\n{}", steps.join("\n"))
}

/// An `error:` refusal on one input: exit 2, one `error:` line naming the
/// reason, and nothing written.
fn assert_refused_exit_2(out: &Output) {
    let err = stderr(out);
    assert_eq!(out.status.code(), Some(2), "stderr:\n{err}");
    let errors: Vec<&str> = err.lines().filter(|l| l.starts_with("error:")).collect();
    assert_eq!(errors.len(), 1, "one error line, stderr:\n{err}");
    assert!(errors[0].contains("watermark refused"), "{}", errors[0]);
    assert!(errors[0].contains("does not fit"), "{}", errors[0]);
    assert!(errors[0].contains("24×24 px"), "{}", errors[0]);
}

// ── Single explicit input → exit 2 ──

#[test]
fn watermark_one_input_refused_exits_2() {
    let dir = TempDir::new().unwrap();
    write_base(dir.path(), "b24.png", 24, 24);
    let out = run(
        dir.path(),
        &[
            "watermark",
            "b24.png",
            "--text",
            "© crustyimg",
            "--size",
            "200",
            "-o",
            "out.png",
        ],
    );
    assert_refused_exit_2(&out);
    assert!(!dir.path().join("out.png").exists(), "nothing written");
}

#[test]
fn watermark_one_input_over_the_cap_exits_2() {
    let dir = TempDir::new().unwrap();
    write_base(dir.path(), "b172.png", 172, 172);
    let out = run(
        dir.path(),
        &[
            "watermark",
            "b172.png",
            "--text",
            "© crustyimg",
            "-o",
            "out.png",
        ],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(2), "stderr:\n{err}");
    assert!(err.contains("watermark refused"), "{err}");
    assert!(err.contains("5.61 %"), "{err}");
    assert!(err.contains("(1661 of 29584 px)"), "{err}");
    assert!(err.contains("the limit is 5 %"), "{err}");
    assert!(!dir.path().join("out.png").exists(), "nothing written");
}

#[test]
fn apply_one_input_refused_exits_2() {
    let dir = TempDir::new().unwrap();
    write_base(dir.path(), "b24.png", 24, 24);
    write_text(dir.path(), "r.toml", &recipe(&[ROW_A_STEP]));
    let out = run(
        dir.path(),
        &["apply", "--recipe", "r.toml", "b24.png", "-o", "out.png"],
    );
    assert_refused_exit_2(&out);
    assert!(!dir.path().join("out.png").exists(), "nothing written");
}

#[test]
fn apply_optimize_pinned_one_input_refused_exits_2() {
    let dir = TempDir::new().unwrap();
    write_base(dir.path(), "b24.png", 24, 24);
    write_text(dir.path(), "r.toml", &recipe(&[ROW_A_STEP, OPTIMIZE_STEP]));
    let out = run(
        dir.path(),
        &["apply", "--recipe", "r.toml", "b24.png", "-o", "out.png"],
    );
    assert_refused_exit_2(&out);
    assert!(!dir.path().join("out.png").exists(), "nothing written");
}

#[test]
fn apply_optimize_autodecide_one_input_refused_exits_2() {
    let dir = TempDir::new().unwrap();
    write_base(dir.path(), "b24.png", 24, 24);
    write_text(dir.path(), "r.toml", &recipe(&[ROW_A_STEP, OPTIMIZE_STEP]));
    std::fs::create_dir(dir.path().join("d")).unwrap();
    let out = run(
        dir.path(),
        &["apply", "--recipe", "r.toml", "b24.png", "--out-dir", "d"],
    );
    assert_refused_exit_2(&out);
    assert_eq!(
        std::fs::read_dir(dir.path().join("d")).unwrap().count(),
        0,
        "nothing written to d"
    );
}

// ── Batch → skip, written without the mark ──

#[test]
fn watermark_batch_skips_and_reports() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let b24 = write_base(root, "in/b24.png", 24, 24);
    write_base(root, "in/b256.png", 256, 256);

    let out = run(
        root,
        &[
            "watermark",
            "in/b24.png",
            "in/b256.png",
            "--text",
            "© crustyimg",
            "--out-dir",
            "out",
        ],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    assert_eq!(
        decoded(&root.join("out/b24.png")),
        decoded(&b24),
        "b24 written unmarked"
    );
    // The image that fits is marked exactly as a single-input run marks it.
    let single = run(
        root,
        &[
            "watermark",
            "in/b256.png",
            "--text",
            "© crustyimg",
            "-o",
            "single.png",
        ],
    );
    assert!(single.status.success(), "{}", stderr(&single));
    assert_eq!(
        std::fs::read(root.join("out/b256.png")).unwrap(),
        std::fs::read(root.join("single.png")).unwrap()
    );
    assert_one_skip_line(&err, "b24.png");
    assert!(err.contains("watermark skipped on 1 of 2 images"), "{err}");

    // `--quiet` keeps the warning and drops the closing count.
    let quiet = run(
        root,
        &[
            "watermark",
            "in/b24.png",
            "in/b256.png",
            "--text",
            "© crustyimg",
            "--out-dir",
            "q",
            "--quiet",
        ],
    );
    let err = stderr(&quiet);
    assert_eq!(quiet.status.code(), Some(0), "stderr:\n{err}");
    assert_one_skip_line(&err, "b24.png");
    assert!(!err.contains("watermark skipped on"), "{err}");
}

/// AC-18's fixture: a 48 px and a 512 px input, halved, then marked.
fn small_and_large(root: &Path) {
    write_base(root, "in/small.png", 48, 48);
    write_base(root, "in/large.png", 512, 512);
}

#[test]
fn apply_batch_skip_keeps_the_other_steps() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_and_large(root);
    write_text(
        root,
        "marked.toml",
        &recipe(&[RESIZE_HALF, DEFAULT_TEXT_STEP]),
    );
    write_text(root, "plain.toml", &recipe(&[RESIZE_HALF]));

    let out = run(
        root,
        &[
            "apply",
            "--recipe",
            "marked.toml",
            "in/small.png",
            "in/large.png",
            "--out-dir",
            "m",
        ],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    let plain = run(
        root,
        &[
            "apply",
            "--recipe",
            "plain.toml",
            "in/small.png",
            "in/large.png",
            "--out-dir",
            "p",
        ],
    );
    assert!(plain.status.success(), "{}", stderr(&plain));
    assert_eq!(
        std::fs::read(root.join("m/small.png")).unwrap(),
        std::fs::read(root.join("p/small.png")).unwrap(),
        "the resize still applied; only the mark was skipped"
    );
    assert_ne!(
        std::fs::read(root.join("m/large.png")).unwrap(),
        std::fs::read(root.join("p/large.png")).unwrap(),
        "the large input is marked"
    );
    assert_one_skip_line(&err, "small.png");
    assert!(err.contains("watermark skipped on 1 of 2 images"), "{err}");
}

fn apply_optimize_batch(extra: &[&str]) {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_and_large(root);
    write_text(
        root,
        "r.toml",
        &recipe(&[RESIZE_HALF, DEFAULT_TEXT_STEP, OPTIMIZE_STEP]),
    );
    let mut args = vec![
        "apply",
        "--recipe",
        "r.toml",
        "in/small.png",
        "in/large.png",
        "--out-dir",
        "d",
    ];
    args.extend_from_slice(extra);
    let out = run(root, &args);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    assert_eq!(
        std::fs::read_dir(root.join("d")).unwrap().count(),
        2,
        "two outputs"
    );
    assert_one_skip_line(&err, "small.png");
    assert!(err.contains("watermark skipped on 1 of 2 images"), "{err}");
}

#[test]
fn apply_optimize_pinned_batch_skips() {
    apply_optimize_batch(&["--format", "png"]);
}

#[test]
fn apply_optimize_autodecide_batch_skips() {
    apply_optimize_batch(&[]);
}

// ── build ──

const MANIFEST: &str =
    "version = 1\n\n[[target]]\nsource = \"in/*.png\"\nrecipe = \"r.toml\"\nout = \"dist\"\n";

fn build_project(steps: &[&str]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_and_large(root);
    write_text(root, "r.toml", &recipe(steps));
    write_text(root, "crustyimg.build.toml", MANIFEST);
    dir
}

fn lock_records(root: &Path) -> usize {
    std::fs::read_to_string(root.join("crustyimg.build.lock"))
        .unwrap()
        .matches("[[output]]")
        .count()
}

const SKIP_SUFFIX: &str = ", watermark skipped on 1";

#[test]
fn build_skip_writes_unmarked_output_and_the_lockfile() {
    let dir = build_project(&[RESIZE_HALF, DEFAULT_TEXT_STEP]);
    let root = dir.path();
    let out = run(root, &["build", "--no-cache"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    assert_one_skip_line(&err, "small.png");
    assert!(
        err.contains("built 1 target, 2 outputs (0 cached, 2 rebuilt), watermark skipped on 1\n"),
        "{err}"
    );
    assert_eq!(lock_records(root), 2);

    // The skipped output is the watermark-free recipe's output.
    let marked = std::fs::read(root.join("dist/small.png")).unwrap();
    let plain = TempDir::new().unwrap();
    small_and_large(plain.path());
    write_text(plain.path(), "r.toml", &recipe(&[RESIZE_HALF]));
    write_text(plain.path(), "crustyimg.build.toml", MANIFEST);
    let p = run(plain.path(), &["build", "--no-cache"]);
    assert!(p.status.success(), "{}", stderr(&p));
    assert_eq!(
        marked,
        std::fs::read(plain.path().join("dist/small.png")).unwrap()
    );

    let check = run(root, &["build", "--check", "--no-cache"]);
    assert_eq!(check.status.code(), Some(0), "stderr:\n{}", stderr(&check));
}

#[test]
fn build_decide_plan_skips() {
    let dir = build_project(&[RESIZE_HALF, DEFAULT_TEXT_STEP, OPTIMIZE_STEP]);
    let root = dir.path();
    let out = run(root, &["build", "--no-cache"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    assert_one_skip_line(&err, "small.png");
    assert!(
        err.contains("(0 cached, 2 rebuilt), watermark skipped on 1\n"),
        "{err}"
    );
    assert_eq!(lock_records(root), 2);
}

#[test]
fn build_skip_is_not_counted_as_a_failure() {
    let dir = build_project(&[RESIZE_HALF, DEFAULT_TEXT_STEP]);
    let root = dir.path();
    write_text(root, "in/corrupt.png", "this is not a png");
    let out = run(root, &["build", "--no-cache"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(6), "stderr:\n{err}");
    assert_one_skip_line(&err, "small.png");
    assert!(err.contains("watermark skipped on 1, 1 failed\n"), "{err}");
    assert!(err.contains("1 of 3 inputs failed"), "{err}");
    assert!(
        !root.join("crustyimg.build.lock").exists(),
        "no lockfile on a failure"
    );
}

#[test]
fn build_cached_rerun_does_not_repeat_the_skip() {
    let dir = build_project(&[RESIZE_HALF, DEFAULT_TEXT_STEP]);
    let root = dir.path();
    let first = run(root, &["build"]);
    let err = stderr(&first);
    assert_eq!(first.status.code(), Some(0), "stderr:\n{err}");
    assert_one_skip_line(&err, "small.png");
    let small = std::fs::read(root.join("dist/small.png")).unwrap();
    let large = std::fs::read(root.join("dist/large.png")).unwrap();

    // A cache hit never runs the pipeline, so the skip is not reported again.
    let cached = run(root, &["build"]);
    let err = stderr(&cached);
    assert_eq!(cached.status.code(), Some(0), "stderr:\n{err}");
    assert!(err.contains("(2 cached, 0 rebuilt)\n"), "{err}");
    assert!(skip_lines(&err).is_empty(), "{err}");
    assert!(!err.contains(SKIP_SUFFIX), "{err}");
    assert_eq!(std::fs::read(root.join("dist/small.png")).unwrap(), small);
    assert_eq!(std::fs::read(root.join("dist/large.png")).unwrap(), large);

    // Bypassing the cache lists the skip again.
    let fresh = run(root, &["build", "--no-cache"]);
    let err = stderr(&fresh);
    assert_eq!(fresh.status.code(), Some(0), "stderr:\n{err}");
    assert_one_skip_line(&err, "small.png");
    assert!(err.contains(SKIP_SUFFIX), "{err}");
}

// ── The rule judges the image entering the step ──

/// The default text mark on a 48×48 input, preceded by an enlarging resize:
/// the mark does not fit the 48 px input but fits the 192 px image the step
/// actually receives, so it is drawn.
#[test]
fn rule_judges_the_image_entering_the_step_when_it_grew() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_base(root, "small.png", 48, 48);
    let grow = "[[step]]\nop = \"resize\"\nmode = \"percent\"\npercent = 400\n";
    write_text(root, "marked.toml", &recipe(&[grow, DEFAULT_TEXT_STEP]));
    write_text(root, "plain.toml", &recipe(&[grow]));

    let out = run(
        root,
        &[
            "apply",
            "--recipe",
            "marked.toml",
            "small.png",
            "-o",
            "m.png",
        ],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    assert!(skip_lines(&err).is_empty(), "{err}");
    let plain = run(
        root,
        &[
            "apply",
            "--recipe",
            "plain.toml",
            "small.png",
            "-o",
            "p.png",
        ],
    );
    assert!(plain.status.success(), "{}", stderr(&plain));
    let (marked, unmarked) = (decoded(&root.join("m.png")), decoded(&root.join("p.png")));
    assert_eq!(marked.dimensions(), (192, 192));
    assert_ne!(marked, unmarked, "the mark is drawn");
}

/// The converse: a mark that fits the 400 px input stops fitting after a
/// shrinking resize, and the refusal names the shrunken image.
#[test]
fn rule_judges_the_image_entering_the_step_when_it_shrank() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_base(root, "big.png", 400, 400);
    let shrink = "[[step]]\nop = \"resize\"\nmode = \"percent\"\npercent = 25\n";
    write_text(root, "r.toml", &recipe(&[shrink, DEFAULT_TEXT_STEP]));

    let out = run(
        root,
        &["apply", "--recipe", "r.toml", "big.png", "-o", "out.png"],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(2), "stderr:\n{err}");
    assert!(err.contains("watermark refused"), "{err}");
    assert!(err.contains("100×100 px"), "{err}");
    assert!(!root.join("out.png").exists(), "nothing written");
}

// ── K counts images, not steps ──

#[test]
fn skipped_count_is_per_image_not_per_step() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    small_and_large(root);
    write_text(
        root,
        "two.toml",
        &recipe(&[DEFAULT_TEXT_STEP, DEFAULT_TEXT_STEP]),
    );

    let out = run(
        root,
        &[
            "apply",
            "--recipe",
            "two.toml",
            "in/small.png",
            "in/large.png",
            "--out-dir",
            "o",
        ],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "stderr:\n{err}");
    let lines = skip_lines(&err);
    assert_eq!(lines.len(), 2, "one warning per skipped step:\n{err}");
    assert!(lines.iter().all(|l| l.contains("small.png")), "{err}");
    assert!(err.contains("watermark skipped on 1 of 2 images"), "{err}");
}

// ── The input count is taken after globs and directories resolve ──

#[test]
fn a_directory_resolving_to_one_file_is_refused_with_exit_2() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_base(root, "one/b24.png", 24, 24);
    let out = run(
        root,
        &[
            "watermark",
            "one/",
            "--text",
            "© crustyimg",
            "--size",
            "200",
            "--out-dir",
            "o",
        ],
    );
    assert_refused_exit_2(&out);
    assert!(!root.join("o/b24.png").exists(), "nothing written");
}
