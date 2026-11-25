//! `crop` as a recipe step: a rectangle or an aspect, in fractions of the image.
//!
//! Every refusal row asserts the **whole** message with `==`, so a row that is
//! refused for another row's reason cannot pass. The tables run every row,
//! collect every mismatch and assert the row count, so one wrong message shows
//! up next to all the others instead of hiding behind the first.
//!
//! Fixtures are built here from seeded samples and encoded with the `image`
//! crate's PNG encoder. Expected pixels come from indexing the source buffer,
//! never from `crop_imm`.

use std::path::Path;
use std::process::Command;

use image::{DynamicImage, ImageBuffer, Luma, Rgb, Rgba, RgbaImage};
use tempfile::TempDir;

use crustyimg::image::Image;
use crustyimg::operation::OperationRegistry;
use crustyimg::recipe::{Recipe, RecipeError};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Helpers ─────────────────────────────────────────────────────────────────

/// A one-step version-2 recipe: `op = "crop"` plus the step body lines.
fn crop_recipe(body: &str) -> String {
    format!("version = \"2\"\n\n[[step]]\nop = \"crop\"\n{body}\n")
}

/// Parse and build a recipe the way `apply` does.
fn load(toml: &str) -> Result<crustyimg::pipeline::Pipeline, RecipeError> {
    Recipe::from_toml(toml)?.build_pipeline(&OperationRegistry::with_builtins())
}

struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// A seeded rgb8 image whose every pixel differs from its neighbours.
fn rgb8(w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0145 ^ (u64::from(w) << 20) ^ u64::from(h));
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |_, _| {
        let n = r.next();
        Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
    }))
}

fn rgba8(w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0146 ^ (u64::from(w) << 20) ^ u64::from(h));
    DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |_, _| {
        let n = r.next();
        Rgba([n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8])
    }))
}

fn rgba16(w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0147 ^ (u64::from(w) << 20) ^ u64::from(h));
    DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |_, _| {
        let n = r.next();
        Rgba([
            n as u16,
            (n >> 16) as u16,
            (n >> 32) as u16,
            (n >> 48) as u16,
        ])
    }))
}

fn png_bytes(img: &DynamicImage) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .expect("encode png");
    out.into_inner()
}

fn image_of(img: DynamicImage) -> Image {
    Image::from_bytes(&png_bytes(&img)).expect("fixture decodes")
}

/// Run a recipe over a decoded image, returning the pixels after the last step.
fn run(toml: &str, img: DynamicImage) -> DynamicImage {
    let pipeline = load(toml).expect("recipe builds");
    pipeline
        .run(image_of(img))
        .expect("pipeline runs")
        .pixels()
        .clone()
}

/// The same pixel read from two buffers of one colour type.
fn assert_cropped(src: &DynamicImage, out: &DynamicImage, left: u32, top: u32, what: &str) {
    assert_eq!(src.color(), out.color(), "{what}: colour type");
    let (ow, oh) = (out.width(), out.height());
    assert!(ow > 0 && oh > 0, "{what}: empty output");
    let bad = match (src, out) {
        (DynamicImage::ImageRgb8(s), DynamicImage::ImageRgb8(o)) => (0..oh)
            .flat_map(|j| (0..ow).map(move |i| (i, j)))
            .filter(|&(i, j)| o.get_pixel(i, j) != s.get_pixel(left + i, top + j))
            .count(),
        (DynamicImage::ImageRgba8(s), DynamicImage::ImageRgba8(o)) => (0..oh)
            .flat_map(|j| (0..ow).map(move |i| (i, j)))
            .filter(|&(i, j)| o.get_pixel(i, j) != s.get_pixel(left + i, top + j))
            .count(),
        (DynamicImage::ImageRgba16(s), DynamicImage::ImageRgba16(o)) => (0..oh)
            .flat_map(|j| (0..ow).map(move |i| (i, j)))
            .filter(|&(i, j)| o.get_pixel(i, j) != s.get_pixel(left + i, top + j))
            .count(),
        (DynamicImage::ImageLuma8(s), DynamicImage::ImageLuma8(o)) => (0..oh)
            .flat_map(|j| (0..ow).map(move |i| (i, j)))
            .filter(|&(i, j)| o.get_pixel(i, j) != s.get_pixel(left + i, top + j))
            .count(),
        _ => panic!("{what}: unexpected colour type {:?}", out.color()),
    };
    assert_eq!(bad, 0, "{what}: {bad} pixels differ from the source");
}

// ── Load refusals: R1–R32 ───────────────────────────────────────────────────

/// `'aspect'`'s generic refusal, with the value as written.
fn bad_aspect(got: &str) -> String {
    format!(
        "'aspect' must be two whole numbers from 1 to 65535 written W:H, such as \"16:9\"; got {got:?}"
    )
}

#[test]
fn every_load_refusal_names_its_reason() {
    let mixed = "'aspect' cannot be combined with 'x'; a crop is either a rectangle (x, y, width, height) or an aspect (aspect, gravity)".to_owned();
    let gravity =
        "'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y".to_owned();
    let rows: Vec<(&str, &str, String)> = vec![
        (
            "R1",
            "x=0.1\ny=0.1\nwidth=0.5\nheight=0.5\nleft=0.1",
            "unknown key 'left'; expected one of: x, y, width, height, aspect, gravity".into(),
        ),
        (
            "R2",
            "x=\"10%\"\ny=0\nwidth=0.5\nheight=0.5",
            "'x' must be a number in 0.0..=1.0, got the string \"10%\"".into(),
        ),
        (
            "R3",
            "x=0\ny=0\nwidth=true\nheight=0.5",
            "'width' must be a number > 0 and <= 1, got true".into(),
        ),
        (
            "R4",
            "x=-0.1\ny=0\nwidth=0.5\nheight=0.5",
            "'x' must be a number in 0.0..=1.0, got -0.1".into(),
        ),
        (
            "R5",
            "x=0\ny=1.5\nwidth=0.5\nheight=0.5",
            "'y' must be a number in 0.0..=1.0, got 1.5".into(),
        ),
        (
            "R6",
            "x=0\ny=0\nwidth=1.5\nheight=0.5",
            "'width' must be a number > 0 and <= 1, got 1.5".into(),
        ),
        (
            "R7",
            "x=0\ny=0\nwidth=800\nheight=0.5",
            "'width' must be a number > 0 and <= 1, got 800".into(),
        ),
        (
            "R8",
            "x=nan\ny=0\nwidth=0.5\nheight=0.5",
            "'x' must be a number in 0.0..=1.0, got nan".into(),
        ),
        (
            "R9",
            "x=0\ny=0\nwidth=0.5\nheight=inf",
            "'height' must be a number > 0 and <= 1, got inf".into(),
        ),
        (
            "R10",
            "x=0\ny=0\nwidth=0\nheight=0.5",
            "'width' must be a number > 0 and <= 1, got 0".into(),
        ),
        (
            "R11",
            "x=0\ny=0\nwidth=0.5\nheight=0.0",
            "'height' must be a number > 0 and <= 1, got 0.0".into(),
        ),
        (
            "R12",
            "x=0\ny=0\nwidth=1e-50\nheight=0.5",
            "'width' must be a number > 0 and <= 1, got 1e-50".into(),
        ),
        (
            "R13",
            "x=0.7\ny=0\nwidth=0.5\nheight=0.5",
            "'x' + 'width' is 0.7 + 0.5, past the right edge; together they must be at most 1"
                .into(),
        ),
        (
            "R14",
            "x=0\ny=0.6\nwidth=0.5\nheight=0.6",
            "'y' + 'height' is 0.6 + 0.6, past the bottom edge; together they must be at most 1"
                .into(),
        ),
        (
            "R15",
            "x=0\ny=0\nwidth=0.5",
            "a rectangle crop needs x, y, width and height; 'height' is missing".into(),
        ),
        (
            "R16",
            "width=0.5\nheight=0.5",
            "a rectangle crop needs x, y, width and height; 'x' is missing".into(),
        ),
        (
            "R17",
            "",
            "crop needs a rectangle (x, y, width, height) or an aspect (aspect, and optionally gravity)"
                .into(),
        ),
        (
            "R18",
            "aspect=\"16:9\"\nx=0\ny=0\nwidth=0.5\nheight=0.5",
            mixed,
        ),
        (
            "R19",
            "x=0\ny=0\nwidth=0.5\nheight=0.5\ngravity=\"north\"",
            gravity.clone(),
        ),
        ("R20", "gravity=\"north\"", gravity),
        ("R21", "aspect=\"16x9\"", bad_aspect("16x9")),
        ("R22", "aspect=\"16:\"", bad_aspect("16:")),
        ("R23", "aspect=\"a:b\"", bad_aspect("a:b")),
        ("R24", "aspect=\"+16:9\"", bad_aspect("+16:9")),
        ("R25", "aspect=\"0:9\"", bad_aspect("0:9")),
        ("R26", "aspect=\"-1:1\"", bad_aspect("-1:1")),
        ("R27", "aspect=\"70000:1\"", bad_aspect("70000:1")),
        (
            "R28",
            "aspect=\"2.39:1\"",
            "'aspect' must be two whole numbers written W:H; write \"2.39:1\" as \"239:100\""
                .into(),
        ),
        (
            "R29",
            "aspect=\"1.5:1\"",
            "'aspect' must be two whole numbers written W:H; write \"1.5:1\" as \"3:2\"".into(),
        ),
        ("R30", "aspect=\"1.00001:1\"", bad_aspect("1.00001:1")),
        (
            "R31",
            "aspect=1.78",
            "'aspect' must be a string, got the float 1.78".into(),
        ),
        (
            "R32",
            "aspect=\"16:9\"\ngravity=\"middle\"",
            "'gravity' must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest; got \"middle\"".into(),
        ),
    ];

    let mut mismatches = Vec::new();
    for (id, body, message) in &rows {
        let want = format!("recipe step 0 ('crop'): {message}");
        match load(&crop_recipe(body)) {
            Ok(_) => mismatches.push(format!("{id}: expected {want:?}, but the recipe loaded")),
            Err(e @ RecipeError::InvalidOperation { .. }) => {
                let got = e.to_string();
                if got != want {
                    mismatches.push(format!("{id}: expected {want:?}, got {got:?}"));
                }
            }
            Err(other) => mismatches.push(format!("{id}: expected {want:?}, got {other}")),
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} of {} rows wrong:\n{}",
        mismatches.len(),
        rows.len(),
        mismatches.join("\n")
    );
    assert_eq!(rows.len(), 32, "R1..R32 all ran");
}

// ── Apply refusals: A1–A4 ───────────────────────────────────────────────────

#[test]
fn every_zero_pixel_crop_fails_at_apply() {
    let rows: [(&str, &str, &str); 4] = [
        (
            "A1",
            "x=0.5\ny=0\nwidth=0.0001\nheight=1",
            "the crop rounds to 0×1067 px on this 1600×1067 px image; it must keep at least 1 px each way",
        ),
        (
            "A2",
            "x=0\ny=0.5\nwidth=1\nheight=0.0001",
            "the crop rounds to 1600×0 px on this 1600×1067 px image; it must keep at least 1 px each way",
        ),
        (
            "A3",
            "aspect=\"10000:1\"",
            "the crop rounds to 1600×0 px on this 1600×1067 px image; it must keep at least 1 px each way",
        ),
        (
            "A4",
            "aspect=\"1:10000\"",
            "the crop rounds to 0×1067 px on this 1600×1067 px image; it must keep at least 1 px each way",
        ),
    ];
    let mut mismatches = Vec::new();
    for (id, body, message) in rows {
        let want = format!("operation 'crop' failed: {message}");
        // A row that is refused at load would pass for the wrong reason.
        let pipeline = match load(&crop_recipe(body)) {
            Ok(p) => p,
            Err(e) => {
                mismatches.push(format!("{id}: refused at load, not at apply: {e}"));
                continue;
            }
        };
        match pipeline.run(image_of(rgb8(1600, 1067))) {
            Ok(_) => mismatches.push(format!("{id}: expected {want:?}, but it ran")),
            Err(e) => {
                let got = e.to_string();
                if got != want {
                    mismatches.push(format!("{id}: expected {want:?}, got {got:?}"));
                }
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} rows wrong:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    assert_eq!(rows.len(), 4, "A1..A4 all ran");
}

// ── The binary ──────────────────────────────────────────────────────────────

struct Applied {
    code: Option<i32>,
    stderr: String,
}

fn apply(dir: &Path, recipe: &str, input: &Path) -> Applied {
    let recipe_path = dir.join("crop.toml");
    std::fs::write(&recipe_path, recipe).expect("write recipe");
    let output = Command::new(BIN)
        .arg("apply")
        .arg("--recipe")
        .arg(&recipe_path)
        .arg(input)
        .arg("-o")
        .arg(dir.join("out.png"))
        .arg("-y")
        .output()
        .expect("run apply");
    Applied {
        code: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn write_input(dir: &Path, img: &DynamicImage) -> std::path::PathBuf {
    let path = dir.join("in.png");
    std::fs::write(&path, png_bytes(img)).expect("write input");
    path
}

#[test]
fn apply_refuses_a_zero_pixel_crop_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_input(dir.path(), &rgb8(1600, 1067));
    let r = apply(
        dir.path(),
        &crop_recipe("x=0.5\ny=0\nwidth=0.0001\nheight=1"),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "error: operation 'crop' failed: the crop rounds to 0×1067 px on this 1600×1067 px image; it must keep at least 1 px each way"
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_refuses_a_crop_past_the_right_edge_at_load() {
    let dir = TempDir::new().unwrap();
    let input = write_input(dir.path(), &rgb8(64, 48));
    let r = apply(
        dir.path(),
        &crop_recipe("x=0.7\ny=0\nwidth=0.5\nheight=0.5"),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "error: recipe step 0 ('crop'): 'x' + 'width' is 0.7 + 0.5, past the right edge; together they must be at most 1"
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_crops_a_png_end_to_end() {
    let dir = TempDir::new().unwrap();
    let src = rgb8(1200, 800);
    let input = write_input(dir.path(), &src);
    let r = apply(
        dir.path(),
        &crop_recipe("x=0.123\ny=0.2345\nwidth=0.5\nheight=0.4321"),
        &input,
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!((out.width(), out.height()), (600, 345));
    assert_cropped(&src, &out, 148, 188, "end to end");
}

// ── DEC-095: a crop moves pixels and computes none ──────────────────────────

#[test]
fn crop_preserves_colour_type_and_every_pixel() {
    let body = "x=0.25\ny=0.25\nwidth=0.5\nheight=0.5";
    for (name, src) in [
        ("rgb8", rgb8(64, 48)),
        ("rgba8", rgba8(64, 48)),
        ("rgba16", rgba16(64, 48)),
    ] {
        let out = run(&crop_recipe(body), src.clone());
        assert_eq!((out.width(), out.height()), (32, 24), "{name}: size");
        assert_cropped(&src, &out, 16, 12, name);
    }
    // Luma goes through the same path; the corpus fixtures carry no luma.
    let luma = DynamicImage::ImageLuma8(ImageBuffer::from_fn(64, 48, |x, y| {
        Luma([(x * 3 + y * 5) as u8])
    }));
    let out = run(&crop_recipe(body), luma.clone());
    assert_cropped(&luma, &out, 16, 12, "luma8");
}
