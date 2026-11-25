//! `trim` as a recipe step: remove a uniform border, against a stated colour or
//! the top-left pixel, within a per-channel tolerance.
//!
//! Every refusal row asserts the **whole** message with `==`, so a row that is
//! refused for another row's reason cannot pass. The tables run every row,
//! collect every mismatch and assert the row count, so one wrong message shows
//! up next to all the others instead of hiding behind the first.
//!
//! Fixtures are built here from 16-bit levels (an 8-bit sample is
//! `level / 257`, a float sample is `level / 65535` in `f64` then `f32`) with a
//! seeded xorshift for content, and never come from the code under test.
//! Expected boxes are literals. A trim computes no pixel, so every assertion
//! is equality with a sub-rectangle of the input, by raw sample (a float by
//! bit pattern).

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

fn trim_step(body: &str) -> String {
    format!("op = \"trim\"\n{body}")
}

fn pad_step(body: &str) -> String {
    format!("op = \"pad\"\n{body}")
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

/// A pixel as four 16-bit levels: red, green, blue, alpha. Grey images read
/// the first, and the alpha of a type without alpha is dropped.
type Lv = [u16; 4];

const WHITE: Lv = [65535, 65535, 65535, 65535];

/// An 8-bit sample for a level (exact for a multiple of 257).
fn s8(l: u16) -> u8 {
    (l / 257) as u8
}

/// A float sample for a level.
fn sf(l: u16) -> f32 {
    (f64::from(l) / 65535.0) as f32
}

/// A buffer of one colour type whose pixel `(x, y)` is `f(x, y)`.
fn build(kind: &str, w: u32, h: u32, f: &dyn Fn(u32, u32) -> Lv) -> DynamicImage {
    match kind {
        "l8" => DynamicImage::ImageLuma8(GrayImage::from_fn(w, h, |x, y| Luma([s8(f(x, y)[0])]))),
        "la8" => DynamicImage::ImageLumaA8(GrayAlphaImage::from_fn(w, h, |x, y| {
            let p = f(x, y);
            LumaA([s8(p[0]), s8(p[3])])
        })),
        "rgb8" => DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            Rgb([s8(p[0]), s8(p[1]), s8(p[2])])
        })),
        "rgba8" => DynamicImage::ImageRgba8(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            Rgba([s8(p[0]), s8(p[1]), s8(p[2]), s8(p[3])])
        })),
        "l16" => DynamicImage::ImageLuma16(ImageBuffer::from_fn(w, h, |x, y| Luma([f(x, y)[0]]))),
        "la16" => DynamicImage::ImageLumaA16(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            LumaA([p[0], p[3]])
        })),
        "rgb16" => DynamicImage::ImageRgb16(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            Rgb([p[0], p[1], p[2]])
        })),
        "rgba16" => DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |x, y| Rgba(f(x, y)))),
        "rgb32f" => DynamicImage::ImageRgb32F(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            Rgb([sf(p[0]), sf(p[1]), sf(p[2])])
        })),
        "rgba32f" => DynamicImage::ImageRgba32F(ImageBuffer::from_fn(w, h, |x, y| {
            let p = f(x, y);
            Rgba([sf(p[0]), sf(p[1]), sf(p[2]), sf(p[3])])
        })),
        other => panic!("unknown colour type {other}"),
    }
}

/// A seeded buffer of one colour type: every pixel random, opaque for a type
/// with alpha.
fn seeded(kind: &str, w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0152 ^ (u64::from(w) << 20) ^ u64::from(h) ^ kind.len() as u64);
    let mut lv = Vec::new();
    for _ in 0..(w * h) {
        let n = r.next();
        lv.push([n as u16, (n >> 16) as u16, (n >> 32) as u16, 65535]);
    }
    build(kind, w, h, &|x, y| lv[(y * w + x) as usize])
}

/// The name of a buffer's colour type, in `KINDS` spelling.
fn kind_of(img: &DynamicImage) -> &'static str {
    match img {
        DynamicImage::ImageLuma8(_) => "l8",
        DynamicImage::ImageLumaA8(_) => "la8",
        DynamicImage::ImageRgb8(_) => "rgb8",
        DynamicImage::ImageRgba8(_) => "rgba8",
        DynamicImage::ImageLuma16(_) => "l16",
        DynamicImage::ImageLumaA16(_) => "la16",
        DynamicImage::ImageRgb16(_) => "rgb16",
        DynamicImage::ImageRgba16(_) => "rgba16",
        DynamicImage::ImageRgb32F(_) => "rgb32f",
        DynamicImage::ImageRgba32F(_) => "rgba32f",
        _ => "other",
    }
}

/// Channels per pixel of a kind.
fn channels_of(kind: &str) -> usize {
    match kind {
        "l8" | "l16" => 1,
        "la8" | "la16" => 2,
        "rgb8" | "rgb16" | "rgb32f" => 3,
        "rgba8" | "rgba16" | "rgba32f" => 4,
        other => panic!("unknown colour type {other}"),
    }
}

/// Every sample of `img` as raw bits, row by row. A float is its bit pattern,
/// so equality is exact.
fn samples(img: &DynamicImage) -> Vec<u64> {
    let size = match img.color().bytes_per_pixel() as usize / channels_of(kind_of(img)) {
        1 => 1,
        2 => 2,
        _ => 4,
    };
    img.as_bytes()
        .chunks_exact(size)
        .map(|c| match size {
            1 => u64::from(c[0]),
            2 => u64::from(u16::from_ne_bytes([c[0], c[1]])),
            _ => u64::from(u32::from_ne_bytes([c[0], c[1], c[2], c[3]])),
        })
        .collect()
}

/// `None` when `out` is the `(left, top, w, h)` sub-rectangle of `input`, at
/// the input's colour type, sample for sample; otherwise what differs.
fn sub_rect_mismatch(
    input: &DynamicImage,
    out: &DynamicImage,
    (left, top, w, h): (u32, u32, u32, u32),
) -> Option<String> {
    if kind_of(out) != kind_of(input) {
        return Some(format!(
            "colour type {} became {}",
            kind_of(input),
            kind_of(out)
        ));
    }
    if (out.width(), out.height()) != (w, h) {
        return Some(format!(
            "size {}x{}, expected {w}x{h}",
            out.width(),
            out.height()
        ));
    }
    let ch = channels_of(kind_of(input));
    let src = samples(input);
    let got = samples(out);
    let mut want = Vec::new();
    for y in top..top + h {
        for x in left..left + w {
            let at = (y as usize * input.width() as usize + x as usize) * ch;
            want.extend_from_slice(&src[at..at + ch]);
        }
    }
    if got == want {
        None
    } else {
        Some("samples differ from the input's sub-rectangle".to_owned())
    }
}

/// `None` when `out` is `input` unchanged.
fn unchanged_mismatch(input: &DynamicImage, out: &DynamicImage) -> Option<String> {
    sub_rect_mismatch(input, out, (0, 0, input.width(), input.height()))
}

fn wrap(px: DynamicImage) -> Image {
    Image::from_parts(px, ImageFormat::Png, None)
}

/// Run a recipe over a buffer: the pixels after the last step, or the error's
/// text.
fn try_run(toml: &str, px: &DynamicImage) -> Result<DynamicImage, String> {
    let pipeline = load(toml).map_err(|e| format!("did not load: {e}"))?;
    pipeline
        .run(wrap(px.clone()))
        .map(|img| img.pixels().clone())
        .map_err(|e| e.to_string())
}

/// `trim` with `body` over `px`, expecting success.
fn trim(body: &str, px: &DynamicImage) -> DynamicImage {
    try_run(&recipe(&[&trim_step(body)]), px).expect("trim runs")
}

fn png_bytes(img: &DynamicImage) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png)
        .expect("encode png");
    out.into_inner()
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

const KEYS: &str = "expected one of: color, tolerance";
const NOT_HEX: &str = "'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA, got ";
const TOL: &str = "'tolerance' must be an integer in 0..=255, got ";

#[test]
fn every_trim_refusal_names_its_reason() {
    let rows: Vec<(&str, &str, String)> = vec![
        ("T1", "fuzz = 8", format!("unknown key 'fuzz'; {KEYS}")),
        (
            "T2",
            "threshold = 8",
            format!("unknown key 'threshold'; {KEYS}"),
        ),
        (
            "T3",
            "sides = \"all\"",
            format!("unknown key 'sides'; {KEYS}"),
        ),
        (
            "T4",
            "border = 0.01",
            format!("unknown key 'border'; {KEYS}"),
        ),
        ("T5", "margin = 2", format!("unknown key 'margin'; {KEYS}")),
        (
            "T6",
            "colour = \"#ffffff\"",
            format!("unknown key 'colour'; {KEYS}"),
        ),
        (
            "T7",
            "tolerence = 8",
            format!("unknown key 'tolerence'; {KEYS}"),
        ),
        ("T8", "tolerance = 256", format!("{TOL}256")),
        ("T9", "tolerance = -1", format!("{TOL}-1")),
        ("T10", "tolerance = 4294967296", format!("{TOL}4294967296")),
        ("T11", "tolerance = 16777217", format!("{TOL}16777217")),
        ("T12", "tolerance = 8.0", format!("{TOL}the float 8.0")),
        ("T13", "tolerance = 8.5", format!("{TOL}the float 8.5")),
        ("T14", "tolerance = \"8\"", format!("{TOL}the string \"8\"")),
        ("T15", "tolerance = true", format!("{TOL}true")),
        ("T16", "color = \"white\"", format!("{NOT_HEX}\"white\"")),
        ("T17", "color = \"#fff\"", format!("{NOT_HEX}\"#fff\"")),
        ("T18", "color = \"#ff00\"", format!("{NOT_HEX}\"#ff00\"")),
        ("T20", "color = \"\"", format!("{NOT_HEX}\"\"")),
        (
            "T21",
            "color = 0xffffff",
            "'color' must be a string, got the integer 16777215".to_owned(),
        ),
        (
            "T22",
            "color = \"#gggggg\"",
            format!("{NOT_HEX}\"#gggggg\""),
        ),
        ("T23", "color = \"aéaaa\"", format!("{NOT_HEX}\"aéaaa\"")),
    ];

    let mut mismatches = Vec::new();
    for (id, body, message) in &rows {
        let want = format!("recipe step 0 ('trim'): {message}");
        match load(&recipe(&[&trim_step(body)])) {
            Ok(_) => mismatches.push(format!("{id}: the step built; expected: {want}")),
            Err(RecipeError::InvalidOperation { .. }) => {
                let got = load(&recipe(&[&trim_step(body)]))
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_default();
                if got != want {
                    mismatches.push(format!("{id}: got {got:?}, expected {want:?}"));
                }
            }
            Err(other) => mismatches.push(format!("{id}: wrong error kind: {other}")),
        }
    }
    assert_no_mismatches(rows.len(), 22, mismatches);
}

#[test]
fn accepted_spellings_load() {
    let accepted = [
        "",
        "tolerance = 0",
        "tolerance = 255",
        "color = \"#00000000\"",
        "color = \"FFFFFF\"",
        "color = \"#123456ff\"",
        "color = \"#ffffff\"\ntolerance = 8",
        "color = \"transparent\"",
    ];
    let mut mismatches = Vec::new();
    for body in accepted {
        if let Err(e) = load(&recipe(&[&trim_step(body)])) {
            mismatches.push(format!("{body:?}: {e}"));
        }
    }
    assert_no_mismatches(accepted.len(), 8, mismatches);
}

/// `color` goes through `parse_color`, which used to slice a six-byte string
/// with a multi-byte character in the middle and panic.
#[test]
fn a_non_ascii_colour_is_refused_not_a_panic() {
    let rows = [
        ("trim", recipe(&[&trim_step("color = \"aéaaa\"")])),
        (
            "pad",
            recipe(&[&pad_step("border = 0.1\ncolor = \"aéaaa\"")]),
        ),
        (
            "watermark",
            recipe(&["op = \"watermark\"\ntext = \"hi\"\ncolor = \"aéaaa\""]),
        ),
    ];
    let mut mismatches = Vec::new();
    for (op, toml) in &rows {
        match load(toml) {
            Ok(_) => mismatches.push(format!("{op}: the step built")),
            Err(e) => {
                let shown = e.to_string();
                if !shown.contains("got \"aéaaa\"") {
                    mismatches.push(format!("{op}: message lacks the colour: {shown}"));
                }
            }
        }
    }
    assert_no_mismatches(rows.len(), 3, mismatches);

    // The CLI flag: exit 2 with the colour's own message, not a panic (101).
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("rgb8", 8, 8));
    let out = Command::new(BIN)
        .args(["watermark", "--text", "hi", "--color", "aéaaa"])
        .arg(&input)
        .arg("-o")
        .arg(dir.path().join("out.png"))
        .output()
        .expect("run watermark");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "stderr: {stderr}");
    assert_eq!(
        stderr.trim_end(),
        "error: invalid color 'aéaaa': expected RRGGBB, #RRGGBB, or RRGGBBAA"
    );
}

// ── Exactness ───────────────────────────────────────────────────────────────

/// One variant of the exactness matrix: label, tolerance line, the odd corner
/// pixel if any, and the expected box.
type Variant = (&'static str, &'static str, Option<Lv>, (u32, u32, u32, u32));

/// A shape: content size, the border `(left, top, right, bottom)`, and whether
/// only a stated colour applies (its top-left is content).
struct Shape {
    name: &'static str,
    content: (u32, u32),
    border: (u32, u32, u32, u32),
    stated_only: bool,
}

const SHAPES: [Shape; 4] = [
    Shape {
        name: "4x3 in (1,2,3,1)",
        content: (4, 3),
        border: (1, 2, 3, 1),
        stated_only: false,
    },
    Shape {
        name: "3x3 in (0,0,2,0)",
        content: (3, 3),
        border: (0, 0, 2, 0),
        stated_only: true,
    },
    Shape {
        name: "1x3 in (2,1,2,1)",
        content: (1, 3),
        border: (2, 1, 2, 1),
        stated_only: false,
    },
    Shape {
        name: "5x7 in (3,0,1,2)",
        content: (5, 7),
        border: (3, 0, 1, 2),
        stated_only: false,
    },
];

/// A bordered buffer: white border, seeded content below level 30000, and
/// optionally one different border pixel at the bottom-right corner.
fn bordered(kind: &str, shape: &Shape, corner: Option<Lv>) -> DynamicImage {
    let (cw, ch) = shape.content;
    let (l, t, r, b) = shape.border;
    let (w, h) = (cw + l + r, ch + t + b);
    let mut rng = XorShift(0xB0D3_0152 ^ u64::from(w * 31 + h));
    let mut content = Vec::new();
    for _ in 0..(cw * ch) {
        let n = rng.next();
        content.push([
            (n % 30000) as u16,
            ((n >> 16) % 30000) as u16,
            ((n >> 32) % 30000) as u16,
            65535,
        ]);
    }
    build(kind, w, h, &|x, y| {
        if let (Some(c), true) = (corner, x == w - 1 && y == h - 1) {
            return c;
        }
        if x >= l && x < l + cw && y >= t && y < t + ch {
            content[((y - t) * cw + (x - l)) as usize]
        } else {
            WHITE
        }
    })
}

#[test]
fn every_colour_type_trims_exactly() {
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        let eight = kind.ends_with('8');
        // Eight steps below white is 2056 levels at every depth; one past is
        // 9 steps (2313) at 8 bits and 2057 levels at 16 bits and float.
        let off8: u16 = 65535 - 2056;
        let off9: u16 = if eight { 65535 - 2313 } else { 65535 - 2057 };
        for shape in &SHAPES {
            let (cw, ch) = shape.content;
            let (l, t, r, b) = shape.border;
            let (w, h) = (cw + l + r, ch + t + b);
            let sources: &[(&str, &str)] = if shape.stated_only {
                &[("stated", "color = \"#ffffff\"\n")]
            } else {
                &[("top-left", ""), ("stated", "color = \"#ffffff\"\n")]
            };
            for (source, color) in sources {
                // (label, tolerance line, corner, expected box)
                let variants: [Variant; 3] = [
                    ("t=0 exact", "tolerance = 0", None, (l, t, cw, ch)),
                    (
                        "t=8 at 8 steps",
                        "tolerance = 8",
                        Some([off8, off8, off8, 65535]),
                        (l, t, cw, ch),
                    ),
                    (
                        "t=8 one past",
                        "tolerance = 8",
                        Some([off9, off9, off9, 65535]),
                        (l, t, w - l, h - t),
                    ),
                ];
                for (label, tol, corner, want) in variants {
                    cases += 1;
                    let input = bordered(kind, shape, corner);
                    let name = format!("{kind} {source} {} {label}", shape.name);
                    match try_run(&recipe(&[&trim_step(&format!("{color}{tol}"))]), &input) {
                        Ok(out) => {
                            if let Some(why) = sub_rect_mismatch(&input, &out, want) {
                                mismatches.push(format!("{name}: {why}"));
                            }
                        }
                        Err(e) => mismatches.push(format!("{name}: {e}")),
                    }
                }
            }
        }
    }
    assert_no_mismatches(cases, 210, mismatches);
}

/// A 7×6 `Rgb8`: 3×2 content at (2, 2) in a 2 px `#808080` border, with one
/// border pixel `delta` steps off in channel 0.
fn sided(off_at: (u32, u32), delta: u8) -> DynamicImage {
    build("rgb8", 7, 6, &|x, y| {
        let grey = 128 * 257;
        if (x, y) == off_at {
            return [(128 - delta as u16) * 257, grey, grey, 65535];
        }
        if (2..5).contains(&x) && (2..4).contains(&y) {
            [20 * 257, 30 * 257, 40 * 257, 65535]
        } else {
            [grey, grey, grey, 65535]
        }
    })
}

#[test]
fn each_side_is_scanned_on_its_own() {
    let body = "color = \"#808080\"\ntolerance = 8";
    let cases = [
        ((0, 3), (0, 2, 5, 2)),
        ((3, 0), (2, 0, 3, 4)),
        ((6, 3), (2, 2, 5, 2)),
        ((3, 5), (2, 2, 3, 4)),
    ];
    let mut mismatches = Vec::new();
    let mut rows = 0;
    for (at, want) in cases {
        rows += 1;
        let input = sided(at, 9);
        if let Some(why) = sub_rect_mismatch(&input, &trim(body, &input), want) {
            mismatches.push(format!("9 off at {at:?}: {why}"));
        }
        rows += 1;
        let input = sided(at, 8);
        if let Some(why) = sub_rect_mismatch(&input, &trim(body, &input), (2, 2, 3, 2)) {
            mismatches.push(format!("8 off at {at:?}: {why}"));
        }
    }
    assert_no_mismatches(rows, 8, mismatches);
}

#[test]
fn odd_shapes_and_survivors() {
    let mut mismatches = Vec::new();
    let other = [9 * 257, 9 * 257, 9 * 257, 65535];

    let img = build("rgb8", 5, 5, &|x, y| {
        if (x, y) == (2, 2) {
            other
        } else {
            WHITE
        }
    });
    if let Some(why) = sub_rect_mismatch(&img, &trim("", &img), (2, 2, 1, 1)) {
        mismatches.push(format!("5x5 one pixel: {why}"));
    }
    let img = build("rgb8", 1, 5, &|_, y| if y == 3 { other } else { WHITE });
    if let Some(why) = sub_rect_mismatch(&img, &trim("", &img), (0, 3, 1, 1)) {
        mismatches.push(format!("1x5 column: {why}"));
    }
    let img = build("rgb8", 1, 1, &|_, _| other);
    if let Some(why) = unchanged_mismatch(&img, &trim("color = \"#ffffff\"", &img)) {
        mismatches.push(format!("1x1 stated other colour: {why}"));
    }
    assert_no_mismatches(3, 3, mismatches);
}

#[test]
fn grey_compares_as_three_equal_channels() {
    let mut mismatches = Vec::new();
    for (kind, border, centre) in [("l8", 128 * 257, 16 * 257), ("l16", 32896, 4112)] {
        let img = build(kind, 3, 3, &|x, y| {
            let v = if (x, y) == (1, 1) { centre } else { border };
            [v, v, v, 65535]
        });
        if let Some(why) = sub_rect_mismatch(&img, &trim("color = \"#808080\"", &img), (1, 1, 1, 1))
        {
            mismatches.push(format!("{kind} grey colour: {why}"));
        }
        if let Some(why) = unchanged_mismatch(&img, &trim("color = \"#800000\"", &img)) {
            mismatches.push(format!("{kind} chromatic colour: {why}"));
        }
    }
    assert_no_mismatches(4, 4, mismatches);
}

#[test]
fn alpha_zero_matches_whatever_is_hidden() {
    let mut mismatches = Vec::new();
    let mut rows = 0;
    // 12×10, opaque green at (1, 2, 8, 7), every border pixel alpha 0 with its
    // own hidden colour.
    let mut rng = XorShift(0xA1FA_0152);
    let mut hidden = Vec::new();
    for _ in 0..(12 * 10) {
        let n = rng.next();
        hidden.push([n as u16, (n >> 16) as u16, (n >> 32) as u16, 0]);
    }
    for kind in ["rgba8", "la16", "rgba32f"] {
        let img = build(kind, 12, 10, &|x, y| {
            if (1..9).contains(&x) && (2..9).contains(&y) {
                [0, 40000, 0, 65535]
            } else {
                hidden[(y * 12 + x) as usize]
            }
        });
        for (label, body) in [("default", ""), ("stated", "color = \"#00000000\"")] {
            rows += 1;
            if let Some(why) = sub_rect_mismatch(&img, &trim(body, &img), (1, 2, 8, 7)) {
                mismatches.push(format!("{kind} {label}: {why}"));
            }
        }
    }

    // Alpha 3 is not alpha 0: all four channels are compared.
    for (corner, want) in [
        ([200u16, 10, 90, 3], (1, 1, 2, 2)),
        ([5, 5, 5, 3], (1, 1, 1, 1)),
    ] {
        rows += 1;
        let img = build("rgba8", 3, 3, &|x, y| match (x, y) {
            (1, 1) => [100 * 257, 100 * 257, 100 * 257, 65535],
            (2, 2) => [
                corner[0] * 257,
                corner[1] * 257,
                corner[2] * 257,
                corner[3] * 257,
            ],
            _ => [200 * 257, 100 * 257, 50 * 257, 0],
        });
        let out = trim("color = \"#00000000\"\ntolerance = 8", &img);
        // The corner is built from 8-bit values: alpha 3 is 3 * 257 levels.
        if let Some(why) = sub_rect_mismatch(&img, &out, want) {
            mismatches.push(format!("corner {corner:?}: {why}"));
        }
    }
    assert_no_mismatches(rows, 8, mismatches);
}

#[test]
fn nan_and_out_of_range_floats() {
    let mut mismatches = Vec::new();
    let mut check =
        |label: &str, input: &DynamicImage, body: &str, want: Option<(u32, u32, u32, u32)>| {
            let out = trim(body, input);
            let why = match want {
                Some(b) => sub_rect_mismatch(input, &out, b),
                None => unchanged_mismatch(input, &out),
            };
            if let Some(why) = why {
                mismatches.push(format!("{label}: {why}"));
            }
        };
    let white = [1.0f32, 1.0, 1.0];
    let float3 = |edge: [f32; 3], centre: [f32; 3], corner: Option<[f32; 3]>| {
        DynamicImage::ImageRgb32F(ImageBuffer::from_fn(3, 3, |x, y| {
            let p = match (x, y) {
                (1, 1) => centre,
                (2, 2) => corner.unwrap_or(edge),
                _ => edge,
            };
            Rgb(p)
        }))
    };
    let centre = [0.25, 0.5, 0.75];
    let nan_corner = float3(white, centre, Some([f32::NAN, 1.0, 1.0]));
    check(
        "nan corner",
        &nan_corner,
        "color = \"#ffffff\"",
        Some((1, 1, 2, 2)),
    );
    check(
        "nan corner at t=255",
        &nan_corner,
        "color = \"#ffffff\"\ntolerance = 255",
        Some((2, 2, 1, 1)),
    );
    let all_nan = float3([f32::NAN; 3], [f32::NAN; 3], None);
    check("all nan, no keys", &all_nan, "", None);
    let over = float3([1.5, 1.0, 7.0], centre, None);
    check(
        "over range",
        &over,
        "color = \"#ffffff\"",
        Some((1, 1, 1, 1)),
    );
    let under = float3([-0.25, 0.0, -9.0], centre, None);
    check(
        "under range",
        &under,
        "color = \"#000000\"",
        Some((1, 1, 1, 1)),
    );
    assert_no_mismatches(5, 5, mismatches);
}

// ── No-op, whole match, refusal hook ────────────────────────────────────────

#[test]
fn no_op_returns_the_input() {
    let mut mismatches = Vec::new();
    for kind in ["rgb8", "rgba16", "rgba32f"] {
        let img = seeded(kind, 4, 3);
        if let Some(why) = unchanged_mismatch(&img, &trim("", &img)) {
            mismatches.push(format!("{kind}: {why}"));
        }
    }
    assert_no_mismatches(3, 3, mismatches);
}

const W1_1X1: &str = "operation 'trim' failed: every pixel of this 1×1 px image matches its top-left pixel within 'tolerance' = 0, so a trim would leave nothing";

#[test]
fn whole_match_fails_at_apply() {
    let one = build("rgb16", 1, 1, &|_, _| [7, 8, 9, 65535]);
    assert_eq!(
        try_run(&recipe(&[&trim_step("")]), &one).unwrap_err(),
        W1_1X1
    );

    let near = build("rgb8", 3, 2, &|_, _| {
        [250 * 257, 255 * 257, 255 * 257, 65535]
    });
    assert_eq!(
        try_run(
            &recipe(&[&trim_step("color = \"#FFFFFF\"\ntolerance = 5")]),
            &near
        )
        .unwrap_err(),
        "operation 'trim' failed: every pixel of this 3×2 px image matches 'color' = \"#FFFFFF\" within 'tolerance' = 5, so a trim would leave nothing"
    );

    let seeded43 = seeded("rgb8", 4, 3);
    assert_eq!(
        try_run(&recipe(&[&trim_step("tolerance = 255")]), &seeded43).unwrap_err(),
        "operation 'trim' failed: every pixel of this 4×3 px image matches its top-left pixel within 'tolerance' = 255, so a trim would leave nothing"
    );
}

/// A 4×4 RGBA8 whose outer ring is `(255, 0, 0, 0)` (fully transparent, with a
/// colour under it) and whose inner 2×2 is opaque and different from the ring.
fn transparent_ring() -> DynamicImage {
    DynamicImage::ImageRgba8(image::ImageBuffer::from_fn(4, 4, |x, y| {
        if (1..3).contains(&x) && (1..3).contains(&y) {
            image::Rgba([10 + x as u8, 20 + y as u8, 30, 255])
        } else {
            image::Rgba([255, 0, 0, 0])
        }
    }))
}

#[test]
fn transparent_keyword_matches_like_00000000() {
    let src = transparent_ring();
    let mut mismatches = Vec::new();
    for spelling in ["transparent", "#00000000"] {
        let out = match try_run(
            &recipe(&[&trim_step(&format!("color = \"{spelling}\""))]),
            &src,
        ) {
            Ok(o) => o,
            Err(e) => {
                mismatches.push(format!("{spelling}: {e}"));
                continue;
            }
        };
        if (out.width(), out.height()) != (2, 2) {
            mismatches.push(format!("{spelling}: {}x{}", out.width(), out.height()));
        }
    }
    assert_no_mismatches(2, 2, mismatches);

    // An image that is all transparent matches the keyword everywhere, and the
    // message prints the keyword as written, not `#00000000`.
    let clear = DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
        4,
        4,
        image::Rgba([9, 9, 9, 0]),
    ));
    assert_eq!(
        try_run(&recipe(&[&trim_step("color = \"transparent\"")]), &clear).unwrap_err(),
        "operation 'trim' failed: every pixel of this 4×4 px image matches 'color' = \"transparent\" within 'tolerance' = 0, so a trim would leave nothing"
    );
}

// ── The inverse of pad ──────────────────────────────────────────────────────

#[test]
fn pad_then_trim_is_the_identity() {
    // (kind, size, pad body, colour)
    let cases: [(&str, (u32, u32), &str, &str); 5] = [
        (
            "rgba16",
            (37, 23),
            "aspect = \"16:9\"\ngravity = \"southeast\"",
            "#123456",
        ),
        (
            "rgb8",
            (23, 37),
            "aspect = \"3:2\"\ngravity = \"northwest\"",
            "#123456",
        ),
        ("l8", (7, 5), "border = 0.3", "#808080"),
        ("rgba32f", (5, 9), "aspect = \"1:1\"", "#123456"),
        ("rgb16", (1, 1), "border = 1", "#123456"),
    ];
    let mut mismatches = Vec::new();
    for (kind, (w, h), pad, color) in cases {
        let input = seeded(kind, w, h);
        let toml = recipe(&[
            &pad_step(&format!("{pad}\ncolor = \"{color}\"")),
            &trim_step(&format!("color = \"{color}\"")),
        ]);
        match try_run(&toml, &input) {
            Ok(out) => {
                if let Some(why) = unchanged_mismatch(&input, &out) {
                    mismatches.push(format!("{kind} {pad}: {why}"));
                }
            }
            Err(e) => mismatches.push(format!("{kind} {pad}: {e}")),
        }
    }

    // A grey image padded with a colour widens to RGB, and trim keeps that.
    let grey = seeded("l8", 7, 5);
    let toml = recipe(&[
        &pad_step("border = 0.3\ncolor = \"#123456\""),
        &trim_step("color = \"#123456\""),
    ]);
    match try_run(&toml, &grey) {
        Ok(out) => {
            let widened = DynamicImage::ImageRgb8(grey.to_rgb8());
            if let Some(why) = unchanged_mismatch(&widened, &out) {
                mismatches.push(format!("l8 widened: {why}"));
            }
        }
        Err(e) => mismatches.push(format!("l8 widened: {e}")),
    }

    // The intended failure: an all-white top row goes with the pad.
    let mut r = XorShift(0x1A57_0152);
    let mut lv = Vec::new();
    for _ in 0..12 {
        let n = r.next();
        lv.push([
            (n % 60000) as u16,
            ((n >> 16) % 60000) as u16,
            ((n >> 32) % 60000) as u16,
            65535,
        ]);
    }
    let topwhite = build("rgb8", 4, 3, &|x, y| {
        if y == 0 {
            WHITE
        } else {
            lv[(y * 4 + x) as usize]
        }
    });
    let toml = recipe(&[
        &pad_step("border = 0.5\ncolor = \"#ffffff\""),
        &trim_step(""),
    ]);
    match try_run(&toml, &topwhite) {
        Ok(out) => {
            if let Some(why) = sub_rect_mismatch(&topwhite, &out, (0, 1, 4, 2)) {
                mismatches.push(format!("white top row: {why}"));
            }
        }
        Err(e) => mismatches.push(format!("white top row: {e}")),
    }
    assert_no_mismatches(cases.len() + 2, 7, mismatches);
}

// ── The binary ──────────────────────────────────────────────────────────────

struct Applied {
    code: Option<i32>,
    stderr: String,
}

fn run_apply(args: &[&Path], dir: &Path, toml: &str) -> Applied {
    let recipe_path = dir.join("trim.toml");
    std::fs::write(&recipe_path, toml).expect("write recipe");
    let output = Command::new(BIN)
        .arg("apply")
        .arg("--recipe")
        .arg(&recipe_path)
        .args(args)
        .arg("-y")
        .output()
        .expect("run apply");
    Applied {
        code: output.status.code(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// `apply --recipe <toml> <input> -o <dir>/out.png`.
fn apply(dir: &Path, toml: &str, input: &Path) -> Applied {
    let out = dir.join("out.png");
    run_apply(&[input, Path::new("-o"), &out], dir, toml)
}

/// `apply --recipe <toml> <inputs…> --out-dir <dir>/out`.
fn apply_batch(dir: &Path, toml: &str, inputs: &[&Path]) -> Applied {
    let out = dir.join("out");
    let mut args: Vec<&Path> = inputs.to_vec();
    args.push(Path::new("--out-dir"));
    args.push(&out);
    run_apply(&args, dir, toml)
}

fn write_png(dir: &Path, name: &str, img: &DynamicImage) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, png_bytes(img)).expect("write input");
    path
}

/// The reason a whole match gives for AC-43's 1×1 input.
const W1_REASON: &str = "every pixel of this 1×1 px image matches its top-left pixel within 'tolerance' = 0, so a trim would leave nothing";

/// An 8×6 `Rgb8`: 4×3 seeded content in a white border (1, 2, 3, 1).
fn b_png() -> DynamicImage {
    bordered("rgb8", &SHAPES[0], None)
}

#[test]
fn apply_no_op_trim_writes_the_input_pixels() {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 4, 3);
    let input = write_png(dir.path(), "in.png", &src);
    let r = apply(dir.path(), &recipe(&[&trim_step("")]), &input);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!((out.width(), out.height()), (4, 3));
    assert_eq!(samples(&out), samples(&src));
}

#[test]
fn apply_refuses_an_unknown_key_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("rgb8", 6, 4));
    let r = apply(dir.path(), &recipe(&[&trim_step("fuzz = 8")]), &input);
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: recipe step 0 ('trim'): unknown key 'fuzz'; {KEYS}")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_refuses_a_float_tolerance_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("rgb8", 6, 4));
    let r = apply(
        dir.path(),
        &recipe(&[&trim_step("tolerance = 8.0")]),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: recipe step 0 ('trim'): {TOL}the float 8.0")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_fails_when_every_pixel_matches_with_exit_1() {
    let dir = TempDir::new().unwrap();
    let one = build("rgb8", 1, 1, &|_, _| [7 * 257, 8 * 257, 9 * 257, 65535]);
    let input = write_png(dir.path(), "in.png", &one);
    let r = apply(dir.path(), &recipe(&[&trim_step("")]), &input);
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: operation 'trim' failed: {W1_REASON}")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn batch_with_a_uniform_image_fails_that_input_with_exit_6() {
    let dir = TempDir::new().unwrap();
    let one = build("rgb8", 1, 1, &|_, _| [7 * 257, 8 * 257, 9 * 257, 65535]);
    let a = write_png(dir.path(), "a.png", &one);
    let b = write_png(dir.path(), "b.png", &b_png());
    let r = apply_batch(dir.path(), &recipe(&[&trim_step("")]), &[&a, &b]);
    assert_eq!(r.code, Some(6), "stderr: {}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("a.png: operation 'trim' failed: {W1_REASON}")),
        "stderr: {}",
        r.stderr
    );
    assert!(
        r.stderr.contains("error: 1 of 2 inputs failed"),
        "stderr: {}",
        r.stderr
    );
    assert!(
        !dir.path().join("out/a.png").exists(),
        "a.png writes nothing"
    );
    let out = image::open(dir.path().join("out/b.png")).expect("b.png is written");
    assert_eq!((out.width(), out.height()), (4, 3));
}

#[test]
fn apply_trims_a_png_end_to_end() {
    let dir = TempDir::new().unwrap();
    let src = b_png();
    let input = write_png(dir.path(), "in.png", &src);
    let r = apply(dir.path(), &recipe(&[&trim_step("")]), &input);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!(sub_rect_mismatch(&src, &out, (1, 2, 4, 3)), None);
}

// ── Metadata: trim acts on the stored pixels and writes no tag ──────────────

/// A 7×5 `Rgb8` with a 1 px white frame round 5×3 seeded content.
fn framed() -> DynamicImage {
    let mut r = XorShift(0x7A66_0152);
    let mut lv = Vec::new();
    for _ in 0..35 {
        let n = r.next();
        lv.push([
            (n % 50000) as u16,
            ((n >> 16) % 50000) as u16,
            ((n >> 32) % 50000) as u16,
            65535,
        ]);
    }
    build("rgb8", 7, 5, &|x, y| {
        if x == 0 || y == 0 || x == 6 || y == 4 {
            WHITE
        } else {
            lv[(y * 7 + x) as usize]
        }
    })
}

/// The framed image as a PNG carrying an `eXIf` chunk with Orientation 6.
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

fn apply_to_tagged(steps: &[&str]) -> (Applied, Option<DynamicImage>, Vec<u8>) {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("in.png");
    std::fs::write(&input, tagged_png(&framed())).expect("write input");
    let r = apply(dir.path(), &recipe(steps), &input);
    let out_path = dir.path().join("out.png");
    let bytes = std::fs::read(&out_path).unwrap_or_default();
    let decoded = image::open(&out_path).ok();
    (r, decoded, bytes)
}

#[test]
fn trim_writes_no_orientation_tag_and_keeps_the_bundle() {
    // Control A: the reader sees the tag on the input bytes.
    assert_eq!(
        common::orientation_of(&tagged_png(&framed())),
        Some(6),
        "control A: the test's PNG does not carry Orientation 6"
    );

    // Control B: the tag reaches the pipeline, so auto-orient turns the image.
    let (r, out, _) = apply_to_tagged(&["op = \"auto-orient\""]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("auto-orient output decodes");
    assert_eq!(
        (out.width(), out.height()),
        (5, 7),
        "control B: the input's Orientation tag never reached the pipeline"
    );

    let trim = trim_step("");
    let (r, out, bytes) = apply_to_tagged(&[&trim]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("trim output decodes");
    assert_eq!(
        (out.width(), out.height()),
        (5, 3),
        "trim alone: {}",
        r.stderr
    );
    match common::orientation_of(&bytes) {
        None | Some(1) => {}
        Some(other) => panic!(
            "the pixel lane carried the source's Orientation tag through ({other}); it must \
             write 1 or nothing"
        ),
    }

    // The bundle survived the trim, so a later auto-orient still turns it.
    let (r, out, _) = apply_to_tagged(&[&trim, "op = \"auto-orient\""]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("trim then auto-orient decodes");
    assert_eq!(
        (out.width(), out.height()),
        (3, 5),
        "trim dropped the metadata bundle, so the later auto-orient found no tag"
    );
}
