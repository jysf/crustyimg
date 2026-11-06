//! `rotate` (quarter turns) and `flip` as recipe steps.
//!
//! Every refusal row asserts the **whole** message with `==`, so a row that is
//! refused for another row's reason cannot pass. The tables run every row,
//! collect every mismatch and assert the row count, so one wrong message shows
//! up next to all the others instead of hiding behind the first.
//!
//! Fixtures are seeded here and never come from the code under test. Expected
//! pixels come from hand-written index maps over the source bytes, never from
//! `image::imageops`: a pixel is `bytes_per_pixel` bytes, so one comparison
//! serves every colour type.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::codecs::png::PngEncoder;
use image::{
    DynamicImage, ExtendedColorType, GrayAlphaImage, GrayImage, ImageBuffer, ImageEncoder,
    ImageFormat, Luma, LumaA, Rgb, Rgba,
};
use tempfile::TempDir;

use crustyimg::image::Image;
use crustyimg::operation::OperationRegistry;
use crustyimg::recipe::{Recipe, RecipeError};

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Helpers ─────────────────────────────────────────────────────────────────

/// A version-2 recipe: each entry is the body of one `[[step]]`.
fn recipe(steps: &[&str]) -> String {
    let mut out = String::from("version = \"2\"\n");
    for body in steps {
        out.push_str("\n[[step]]\n");
        out.push_str(body);
        out.push('\n');
    }
    out
}

fn rotate_step(angle: &str) -> String {
    format!("op = \"rotate\"\nangle = {angle}")
}

fn flip_step(axis: &str) -> String {
    format!("op = \"flip\"\naxis = \"{axis}\"")
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

/// The ten colour types a `DynamicImage` can hold.
const KINDS: [&str; 10] = [
    "l8", "la8", "rgb8", "rgba8", "l16", "la16", "rgb16", "rgba16", "rgb32f", "rgba32f",
];

/// A seeded buffer of one colour type, every pixel different from its
/// neighbours (to the extent the sample range allows).
fn seeded(kind: &str, w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0147 ^ (u64::from(w) << 20) ^ u64::from(h) ^ kind.len() as u64);
    // A float sample that is not a multiple of anything convenient.
    let f = |n: u64| (n % 100_003) as f32 / 977.0;
    match kind {
        "l8" => DynamicImage::ImageLuma8(GrayImage::from_fn(w, h, |_, _| Luma([r.next() as u8]))),
        "la8" => DynamicImage::ImageLumaA8(GrayAlphaImage::from_fn(w, h, |_, _| {
            let n = r.next();
            LumaA([n as u8, (n >> 8) as u8])
        })),
        "rgb8" => DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
        })),
        "rgba8" => DynamicImage::ImageRgba8(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgba([n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8])
        })),
        "l16" => {
            DynamicImage::ImageLuma16(ImageBuffer::from_fn(w, h, |_, _| Luma([r.next() as u16])))
        }
        "la16" => DynamicImage::ImageLumaA16(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            LumaA([n as u16, (n >> 16) as u16])
        })),
        "rgb16" => DynamicImage::ImageRgb16(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgb([n as u16, (n >> 16) as u16, (n >> 32) as u16])
        })),
        "rgba16" => DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgba([
                n as u16,
                (n >> 16) as u16,
                (n >> 32) as u16,
                (n >> 48) as u16,
            ])
        })),
        "rgb32f" => DynamicImage::ImageRgb32F(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgb([f(n), f(n >> 20), f(n >> 40)])
        })),
        "rgba32f" => DynamicImage::ImageRgba32F(ImageBuffer::from_fn(w, h, |_, _| {
            let n = r.next();
            Rgba([f(n), f(n >> 16), f(n >> 32), f(n >> 48)])
        })),
        other => panic!("unknown colour type {other}"),
    }
}

fn wrap(px: DynamicImage) -> Image {
    Image::from_parts(px, ImageFormat::Png, None)
}

/// Run a recipe over a buffer, returning the pixels after the last step.
fn run(toml: &str, px: &DynamicImage) -> DynamicImage {
    let pipeline = load(toml).expect("recipe builds");
    pipeline
        .run(wrap(px.clone()))
        .expect("pipeline runs")
        .pixels()
        .clone()
}

fn png_bytes(img: &DynamicImage) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png)
        .expect("encode png");
    out.into_inner()
}

// ── Hand-written index maps ─────────────────────────────────────────────────
//
// On a W×H input, output pixel (ox, oy) is this input pixel. Each returns the
// output size and the map.

type Map = Box<dyn Fn(u32, u32) -> (u32, u32)>;

/// The output size and index map of `turns` clockwise quarter turns.
fn turn_map(turns: u8, w: u32, h: u32) -> ((u32, u32), Map) {
    match turns % 4 {
        0 => ((w, h), Box::new(|x, y| (x, y))),
        1 => ((h, w), Box::new(move |ox, oy| (oy, h - 1 - ox))),
        2 => ((w, h), Box::new(move |ox, oy| (w - 1 - ox, h - 1 - oy))),
        _ => ((h, w), Box::new(move |ox, oy| (w - 1 - oy, ox))),
    }
}

fn flip_map(axis: &str, w: u32, h: u32) -> ((u32, u32), Map) {
    match axis {
        "horizontal" => ((w, h), Box::new(move |ox, oy| (w - 1 - ox, oy))),
        "vertical" => ((w, h), Box::new(move |ox, oy| (ox, h - 1 - oy))),
        other => panic!("unknown axis {other}"),
    }
}

/// Compare `out` to `src` under an index map. `Ok` when the size, colour type
/// and every pixel's bytes agree; otherwise a one-line reason.
fn check_map(
    src: &DynamicImage,
    out: &DynamicImage,
    size: (u32, u32),
    map: &dyn Fn(u32, u32) -> (u32, u32),
) -> Result<(), String> {
    if src.color() != out.color() {
        return Err(format!(
            "colour type {:?} became {:?}",
            src.color(),
            out.color()
        ));
    }
    if (out.width(), out.height()) != size {
        return Err(format!(
            "size {}x{}, expected {}x{}",
            out.width(),
            out.height(),
            size.0,
            size.1
        ));
    }
    let bpp = src.color().bytes_per_pixel() as usize;
    let (sb, ob) = (src.as_bytes(), out.as_bytes());
    let mut wrong = 0u32;
    let mut first = None;
    for oy in 0..size.1 {
        for ox in 0..size.0 {
            let (ix, iy) = map(ox, oy);
            let o = (oy as usize * size.0 as usize + ox as usize) * bpp;
            let i = (iy as usize * src.width() as usize + ix as usize) * bpp;
            if ob[o..o + bpp] != sb[i..i + bpp] {
                wrong += 1;
                first.get_or_insert((ox, oy));
            }
        }
    }
    match first {
        None => Ok(()),
        Some((x, y)) => Err(format!(
            "{wrong} pixels differ from the index map, first at ({x}, {y})"
        )),
    }
}

/// Bytes and colour type equal: the strictest "same buffer" test.
fn same(a: &DynamicImage, b: &DynamicImage) -> bool {
    a.color() == b.color()
        && (a.width(), a.height()) == (b.width(), b.height())
        && a.as_bytes() == b.as_bytes()
}

fn assert_no_mismatches(rows: usize, want: usize, mismatches: Vec<String>) {
    assert!(
        mismatches.is_empty(),
        "{} rows wrong:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    assert_eq!(rows, want, "every row ran");
}

// ── Load refusals ───────────────────────────────────────────────────────────

/// Run a `(id, body, message)` table through the loader for `op`.
fn run_refusal_table(op: &str, rows: &[(&str, &str, &str)]) -> Vec<String> {
    let mut mismatches = Vec::new();
    for (id, body, message) in rows {
        let want = format!("recipe step 0 ('{op}'): {message}");
        let toml = recipe(&[&format!("op = \"{op}\"\n{body}")]);
        match load(&toml) {
            Ok(_) => mismatches.push(format!("{id}: expected {want:?}, but it built")),
            Err(e) => {
                let got = e.to_string();
                if got != want {
                    mismatches.push(format!("{id}: expected {want:?}, got {got:?}"));
                }
            }
        }
    }
    mismatches
}

const RANGE: &str = "'angle' must be a number in -360.0..=360.0";
const NOT_90: &str = "rotating by any other angle is not supported yet";

#[test]
fn every_rotate_refusal_names_its_reason() {
    let rows: Vec<(&str, String, String)> = vec![
        ("R1", String::new(), "missing required key 'angle'".into()),
        (
            "R2",
            "angel = 90".into(),
            "unknown key 'angel'; expected one of: angle".into(),
        ),
        (
            "R3",
            "angle = 90\naxis = \"horizontal\"".into(),
            "unknown key 'axis'; expected one of: angle".into(),
        ),
        (
            "R4",
            "angle = \"90\"".into(),
            format!("{RANGE}, got the string \"90\""),
        ),
        ("R5", "angle = true".into(), format!("{RANGE}, got true")),
        ("R6", "angle = 450".into(), format!("{RANGE}, got 450")),
        ("R7", "angle = -450".into(), format!("{RANGE}, got -450")),
        ("R8", "angle = 360.5".into(), format!("{RANGE}, got 360.5")),
        ("R9", "angle = nan".into(), format!("{RANGE}, got nan")),
        ("R10", "angle = inf".into(), format!("{RANGE}, got inf")),
        ("R11", "angle = -inf".into(), format!("{RANGE}, got -inf")),
        (
            "R12",
            "angle = 45".into(),
            format!("'angle' must be a multiple of 90, got 45; {NOT_90}"),
        ),
        (
            "R13",
            "angle = 45.0".into(),
            format!("'angle' must be a multiple of 90, got 45.0; {NOT_90}"),
        ),
        (
            "R14",
            "angle = 90.5".into(),
            format!("'angle' must be a multiple of 90, got 90.5; {NOT_90}"),
        ),
        (
            "R15",
            "angle = 1.1".into(),
            format!("'angle' must be a multiple of 90, got 1.1; {NOT_90}"),
        ),
        (
            "R16",
            "angle = 1e-300".into(),
            format!("'angle' must be a multiple of 90, got 1e-300; {NOT_90}"),
        ),
    ];
    let borrowed: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|(a, b, c)| (*a, b.as_str(), c.as_str()))
        .collect();
    let mismatches = run_refusal_table("rotate", &borrowed);
    assert_no_mismatches(rows.len(), 16, mismatches);
}

#[test]
fn every_flip_refusal_names_its_reason() {
    const ONE_OF: &str = "'axis' must be one of: horizontal, vertical";
    let rows: Vec<(&str, String, String)> = vec![
        ("F1", String::new(), "missing required key 'axis'".into()),
        (
            "F2",
            "axis = \"horizontal\"\nangle = 90".into(),
            "unknown key 'angle'; expected one of: axis".into(),
        ),
        (
            "F3",
            "axis = \"diagonal\"".into(),
            format!("{ONE_OF}; got \"diagonal\""),
        ),
        ("F4", "axis = \"h\"".into(), format!("{ONE_OF}; got \"h\"")),
        (
            "F5",
            "axis = \"Horizontal\"".into(),
            format!("{ONE_OF}; got \"Horizontal\""),
        ),
        (
            "F6",
            "axis = 1".into(),
            format!("{ONE_OF}; got the integer 1"),
        ),
        ("F7", "axis = true".into(), format!("{ONE_OF}; got true")),
    ];
    let borrowed: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|(a, b, c)| (*a, b.as_str(), c.as_str()))
        .collect();
    let mismatches = run_refusal_table("flip", &borrowed);
    assert_no_mismatches(rows.len(), 7, mismatches);
}

// ── Exactness across colour types ───────────────────────────────────────────

#[test]
fn round_trips_are_exact_in_every_colour_type() {
    let trips: [(&str, Vec<String>); 4] = [
        ("rotate 90 x4", vec![rotate_step("90"); 4]),
        (
            "rotate -90 then 90",
            vec![rotate_step("-90"), rotate_step("90")],
        ),
        (
            "flip horizontal x2",
            vec![flip_step("horizontal"), flip_step("horizontal")],
        ),
        (
            "flip vertical x2",
            vec![flip_step("vertical"), flip_step("vertical")],
        ),
    ];
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        let src = seeded(kind, 7, 5);
        for (name, steps) in &trips {
            cases += 1;
            let refs: Vec<&str> = steps.iter().map(String::as_str).collect();
            let out = run(&recipe(&refs), &src);
            if !same(&src, &out) {
                mismatches.push(format!("{kind}: {name} did not return the input"));
            }
        }
    }
    assert_no_mismatches(cases, 40, mismatches);
}

#[test]
fn rotate_180_is_both_flips_in_every_colour_type() {
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        cases += 1;
        let src = seeded(kind, 7, 5);
        let turned = run(&recipe(&[&rotate_step("180")]), &src);
        let flipped = run(
            &recipe(&[&flip_step("horizontal"), &flip_step("vertical")]),
            &src,
        );
        if !same(&turned, &flipped) {
            mismatches.push(format!("{kind}: rotate 180 differs from the two flips"));
        }
        // Not vacuous: it is not the input.
        if same(&turned, &src) {
            mismatches.push(format!("{kind}: rotate 180 left the buffer unchanged"));
        }
    }
    assert_no_mismatches(cases, 10, mismatches);
}

#[test]
fn quarter_turns_swap_the_dimensions() {
    let src = seeded("rgb8", 7, 5);
    let mut mismatches = Vec::new();
    let mut rows = 0;
    for (angle, dims) in [
        ("90", (5, 7)),
        ("270", (5, 7)),
        ("-90", (5, 7)),
        ("-270", (5, 7)),
        ("180", (7, 5)),
        ("-180", (7, 5)),
    ] {
        rows += 1;
        let out = run(&recipe(&[&rotate_step(angle)]), &src);
        if (out.width(), out.height()) != dims {
            mismatches.push(format!("rotate {angle}: {}x{}", out.width(), out.height()));
        }
    }
    for axis in ["horizontal", "vertical"] {
        rows += 1;
        let out = run(&recipe(&[&flip_step(axis)]), &src);
        if (out.width(), out.height()) != (7, 5) {
            mismatches.push(format!("flip {axis}: {}x{}", out.width(), out.height()));
        }
    }
    assert_no_mismatches(rows, 8, mismatches);
}

#[test]
fn single_steps_keep_colour_type_and_depth() {
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        let src = seeded(kind, 7, 5);
        for step in [
            rotate_step("90"),
            rotate_step("180"),
            flip_step("horizontal"),
            flip_step("vertical"),
        ] {
            cases += 1;
            let out = run(&recipe(&[&step]), &src);
            if out.color() != src.color() {
                mismatches.push(format!(
                    "{kind}: {} turned {:?} into {:?}",
                    step.replace('\n', " "),
                    src.color(),
                    out.color()
                ));
            }
        }
    }
    assert_no_mismatches(cases, 40, mismatches);
}

/// Every step against its hand-written map, in every colour type.
#[test]
fn every_step_matches_its_index_map_in_every_colour_type() {
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        let src = seeded(kind, 7, 5);
        for (angle, turns) in [("90", 1u8), ("180", 2), ("270", 3), ("-90", 3), ("0", 0)] {
            cases += 1;
            let ((ow, oh), map) = turn_map(turns, 7, 5);
            let out = run(&recipe(&[&rotate_step(angle)]), &src);
            if let Err(e) = check_map(&src, &out, (ow, oh), &*map) {
                mismatches.push(format!("{kind}: rotate {angle}: {e}"));
            }
        }
        for axis in ["horizontal", "vertical"] {
            cases += 1;
            let ((ow, oh), map) = flip_map(axis, 7, 5);
            let out = run(&recipe(&[&flip_step(axis)]), &src);
            if let Err(e) = check_map(&src, &out, (ow, oh), &*map) {
                mismatches.push(format!("{kind}: flip {axis}: {e}"));
            }
        }
    }
    assert_no_mismatches(cases, 70, mismatches);
}

// ── The binary ──────────────────────────────────────────────────────────────

struct Applied {
    code: Option<i32>,
    stderr: String,
}

fn apply(dir: &Path, toml: &str, input: &Path) -> Applied {
    let recipe_path = dir.join("geometry.toml");
    std::fs::write(&recipe_path, toml).expect("write recipe");
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

fn write_input(dir: &Path, bytes: &[u8]) -> PathBuf {
    let path = dir.join("in.png");
    std::fs::write(&path, bytes).expect("write input");
    path
}

#[test]
fn apply_refuses_a_45_degree_rotate_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_input(dir.path(), &png_bytes(&seeded("rgb8", 7, 5)));
    let r = apply(dir.path(), &recipe(&[&rotate_step("45")]), &input);
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "error: recipe step 0 ('rotate'): 'angle' must be a multiple of 90, got 45; rotating by any other angle is not supported yet"
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_refuses_a_diagonal_flip_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_input(dir.path(), &png_bytes(&seeded("rgb8", 7, 5)));
    let r = apply(dir.path(), &recipe(&[&flip_step("diagonal")]), &input);
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        "error: recipe step 0 ('flip'): 'axis' must be one of: horizontal, vertical; got \"diagonal\""
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_rotate_0_writes_the_input_pixels() {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 7, 5);
    let input = write_input(dir.path(), &png_bytes(&src));
    let r = apply(dir.path(), &recipe(&[&rotate_step("0")]), &input);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert!(same(&src, &out), "a zero-degree rotate changed the image");
}

#[test]
fn apply_flips_a_png_end_to_end() {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 7, 5);
    let input = write_input(dir.path(), &png_bytes(&src));
    let r = apply(dir.path(), &recipe(&[&flip_step("horizontal")]), &input);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    let ((ow, oh), map) = flip_map("horizontal", 7, 5);
    check_map(&src, &out, (ow, oh), &*map).expect("the horizontal map");
}

// ── Orientation: a geometry step acts on the stored pixels ──────────────────

/// A seeded 7×5 rgb8 PNG carrying an `eXIf` chunk with Orientation 6.
fn tagged_png(src: &DynamicImage) -> Vec<u8> {
    let rgb = src.to_rgb8();
    let mut out = Vec::new();
    let mut enc = PngEncoder::new(&mut out);
    enc.set_exif_metadata(common::tiff_with_orientation(6))
        .expect("the PNG encoder takes EXIF");
    enc.write_image(rgb.as_raw(), 7, 5, ExtendedColorType::Rgb8)
        .expect("encode tagged png");
    out
}

/// The Orientation value kamadak-exif finds in `bytes`, if any.
fn orientation_of(bytes: &[u8]) -> Option<u32> {
    let exif = exif::Reader::new()
        .read_from_container(&mut std::io::Cursor::new(bytes))
        .ok()?;
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
}

fn apply_to_tagged(steps: &[&str]) -> (Applied, Option<DynamicImage>, Vec<u8>) {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 7, 5);
    let input = write_input(dir.path(), &tagged_png(&src));
    let r = apply(dir.path(), &recipe(steps), &input);
    let out_path = dir.path().join("out.png");
    let bytes = std::fs::read(&out_path).unwrap_or_default();
    let decoded = image::open(&out_path).ok();
    (r, decoded, bytes)
}

#[test]
fn rotate_acts_on_stored_pixels_and_writes_no_orientation_tag() {
    let src = seeded("rgb8", 7, 5);

    // Control A: the reader sees the tag on the input bytes.
    assert_eq!(
        orientation_of(&tagged_png(&src)),
        Some(6),
        "control A: the test's PNG does not carry Orientation 6"
    );

    // Control B: the tag reaches the pipeline, so auto-orient turns the image.
    let (r, out, _) = apply_to_tagged(&["op = \"auto-orient\""]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("auto-orient output decodes");
    let ((ow, oh), map) = turn_map(1, 7, 5);
    assert_eq!(
        (out.width(), out.height()),
        (5, 7),
        "control B: the input's Orientation tag never reached the pipeline"
    );
    check_map(&src, &out, (ow, oh), &*map).expect("control B: auto-orient is the 1-turn map");

    // The step under test: rotate alone turns the stored pixels, and the
    // output must not claim an orientation.
    let (r, out, bytes) = apply_to_tagged(&[&rotate_step("90")]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("rotate output decodes");
    check_map(&src, &out, (ow, oh), &*map).expect("rotate 90 is the 1-turn map of stored pixels");
    match orientation_of(&bytes) {
        None | Some(1) => {}
        Some(other) => panic!(
            "the pixel lane carried the source's Orientation tag through ({other}); it must \
             write 1 or nothing"
        ),
    }
}

#[test]
fn auto_orient_then_rotate_turns_twice() {
    let src = seeded("rgb8", 7, 5);
    let (r, out, _) = apply_to_tagged(&["op = \"auto-orient\"", &rotate_step("90")]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("output decodes");
    let ((ow, oh), map) = turn_map(2, 7, 5);
    check_map(&src, &out, (ow, oh), &*map).expect("the 2-turn map of the stored pixels");
}

#[test]
fn rotate_then_auto_orient_matches_auto_orient_then_rotate() {
    let src = seeded("rgb8", 7, 5);
    let (r, first, _) = apply_to_tagged(&["op = \"auto-orient\"", &rotate_step("90")]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let first = first.expect("auto-orient then rotate decodes");

    let (r, second, _) = apply_to_tagged(&[&rotate_step("90"), "op = \"auto-orient\""]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let second = second.expect("rotate then auto-orient decodes");

    assert_eq!(
        (second.width(), second.height()),
        (7, 5),
        "rotate dropped the metadata bundle, so the later auto-orient found no tag"
    );
    let ((ow, oh), map) = turn_map(2, 7, 5);
    check_map(&src, &second, (ow, oh), &*map).expect("the 2-turn map");
    assert!(
        same(&first, &second),
        "the two step orders decode to different pixels"
    );
}
