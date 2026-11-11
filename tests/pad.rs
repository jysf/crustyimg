//! `pad` as a recipe step: grow the canvas to an aspect or by a border, with an
//! opaque fill.
//!
//! Every refusal row asserts the **whole** message with `==`, so a row that is
//! refused for another row's reason cannot pass. The tables run every row,
//! collect every mismatch and assert the row count, so one wrong message shows
//! up next to all the others instead of hiding behind the first.
//!
//! Fixtures are seeded here and never come from the code under test. Expected
//! pixels come from hand-written rules (an offset function written from the
//! gravity names, the colour-type table's fills), never from `image::imageops`
//! or the crate's own placement: a pad computes no pixel, so every assertion is
//! an equality on raw samples.

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

/// A seeded buffer of one colour type, every pixel different from its
/// neighbours (to the extent the sample range allows).
fn seeded(kind: &str, w: u32, h: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0151 ^ (u64::from(w) << 20) ^ u64::from(h) ^ kind.len() as u64);
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

/// `(channels, bits per sample, has alpha)` of a kind.
fn shape_of(kind: &str) -> (usize, u32, bool) {
    match kind {
        "l8" => (1, 8, false),
        "la8" => (2, 8, true),
        "rgb8" => (3, 8, false),
        "rgba8" => (4, 8, true),
        "l16" => (1, 16, false),
        "la16" => (2, 16, true),
        "rgb16" => (3, 16, false),
        "rgba16" => (4, 16, true),
        "rgb32f" => (3, 32, false),
        "rgba32f" => (4, 32, true),
        other => panic!("unknown colour type {other}"),
    }
}

/// Every pixel of `img` as raw samples, `channels` per pixel, row by row. A
/// float sample is its bit pattern, so equality is exact.
fn samples(img: &DynamicImage) -> Vec<u64> {
    let (channels, bits, _) = shape_of(kind_of(img));
    let size = (bits / 8) as usize;
    assert_eq!(
        img.color().bytes_per_pixel() as usize,
        channels * size,
        "sample size"
    );
    img.as_bytes()
        .chunks_exact(size)
        .map(|c| match size {
            1 => u64::from(c[0]),
            2 => u64::from(u16::from_ne_bytes([c[0], c[1]])),
            _ => u64::from(u32::from_ne_bytes([c[0], c[1], c[2], c[3]])),
        })
        .collect()
}

/// One 8-bit fill channel as a sample of `bits` width: `c` at 8 bits,
/// `c × 257` at 16, and `c / 255.0` as an `f32` bit pattern at 32.
fn scale(c: u8, bits: u32) -> u64 {
    match bits {
        8 => u64::from(c),
        16 => u64::from(c) * 257,
        _ => u64::from((f32::from(c) / 255.0).to_bits()),
    }
}

/// The maximum alpha sample at `bits` width.
fn alpha_max(bits: u32) -> u64 {
    match bits {
        8 => 255,
        16 => 65535,
        _ => u64::from(1.0f32.to_bits()),
    }
}

/// The `#RRGGBB` text of a fill.
fn hex(fill: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", fill.0, fill.1, fill.2)
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

fn assert_no_mismatches(rows: usize, want: usize, mismatches: Vec<String>) {
    assert!(
        mismatches.is_empty(),
        "{} rows wrong:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
    assert_eq!(rows, want, "every row ran");
}

const GRAVITIES: [&str; 9] = [
    "center",
    "north",
    "south",
    "east",
    "west",
    "northeast",
    "northwest",
    "southeast",
    "southwest",
];

/// Where a `w`×`h` image sits in a `cw`×`ch` canvas at a gravity: written from
/// the gravity's name, `0`, the whole remainder, or half of it rounded down.
/// Never from the crate's own placement.
fn offset(gravity: &str, cw: u32, ch: u32, w: u32, h: u32) -> (u32, u32) {
    let (rx, ry) = (cw - w, ch - h);
    let left = if gravity.ends_with("west") {
        0
    } else if gravity.ends_with("east") {
        rx
    } else {
        rx / 2
    };
    let top = if gravity.starts_with("north") {
        0
    } else if gravity.starts_with("south") {
        ry
    } else {
        ry / 2
    };
    (left, top)
}

/// The canvas an `a:b` aspect makes for a `w`×`h` image, as integer maths:
/// the axis that is at least as wide as the aspect keeps its length, and the
/// other grows to the aspect, rounded to the nearest pixel with halves up.
fn aspect_canvas(a: u64, b: u64, w: u64, h: u64) -> (u32, u32) {
    if w * b >= h * a {
        (w as u32, ((2 * w * b + a) / (2 * a)) as u32)
    } else {
        (((2 * h * a + b) / (2 * b)) as u32, h as u32)
    }
}

// ── Load refusals ───────────────────────────────────────────────────────────

const BORDER: &str = "'border' must be a number > 0 and <= 1";
const NOT_HEX: &str = "'color' must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA";
const ASPECT: &str =
    "'aspect' must be two whole numbers from 1 to 65535 written W:H, such as \"16:9\"";
const GRAVITY_ONE_OF: &str = "'gravity' must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest";
const KEYS: &str = "expected one of: aspect, gravity, border, color";
const NO_FORM: &str = "pad needs an aspect (aspect, and optionally gravity) or a border";
const NO_COLOR: &str =
    "missing required key 'color'; a pad needs a fill colour such as \"#ffffff\"";

#[test]
fn every_pad_refusal_names_its_reason() {
    let w = "color = \"#ffffff\"";
    let rows: Vec<(&str, String, String)> = vec![
        (
            "P1",
            format!("aspect = \"1:1\"\n{w}\nwidth = 1080"),
            format!("unknown key 'width'; {KEYS}"),
        ),
        (
            "P2",
            format!("aspect = \"1:1\"\n{w}\nheight = 1080"),
            format!("unknown key 'height'; {KEYS}"),
        ),
        (
            "P3",
            format!("aspect = \"1:1\"\n{w}\nmode = \"aspect\""),
            format!("unknown key 'mode'; {KEYS}"),
        ),
        (
            "P4",
            format!("border = 0\n{w}"),
            format!("{BORDER}, got 0"),
        ),
        (
            "P5",
            format!("border = -0.1\n{w}"),
            format!("{BORDER}, got -0.1"),
        ),
        (
            "P6",
            format!("border = 1.5\n{w}"),
            format!("{BORDER}, got 1.5"),
        ),
        (
            "P7",
            format!("border = 800\n{w}"),
            format!("{BORDER}, got 800"),
        ),
        (
            "P8",
            format!("border = nan\n{w}"),
            format!("{BORDER}, got nan"),
        ),
        (
            "P9",
            format!("border = inf\n{w}"),
            format!("{BORDER}, got inf"),
        ),
        (
            "P10",
            format!("border = -inf\n{w}"),
            format!("{BORDER}, got -inf"),
        ),
        (
            "P11",
            format!("border = 1e-50\n{w}"),
            format!("{BORDER}, got 1e-50"),
        ),
        (
            "P12",
            format!("border = \"0.05\"\n{w}"),
            format!("{BORDER}, got the string \"0.05\""),
        ),
        (
            "P13",
            format!("border = true\n{w}"),
            format!("{BORDER}, got true"),
        ),
        (
            "P14",
            format!("aspect = \"1:1\"\ngravity = \"middle\"\n{w}"),
            format!("{GRAVITY_ONE_OF}; got \"middle\""),
        ),
        (
            "P15",
            format!("aspect = \"1:1\"\ngravity = 1\n{w}"),
            format!("{GRAVITY_ONE_OF}; got the integer 1"),
        ),
        (
            "P16",
            format!("aspect = 16\n{w}"),
            "'aspect' must be a string, got the integer 16".into(),
        ),
        (
            "P17",
            "aspect = \"1:1\"\ncolor = 16777215".into(),
            "'color' must be a string, got the integer 16777215".into(),
        ),
        ("P18", w.to_string(), NO_FORM.into()),
        ("P19", format!("gravity = \"north\"\n{w}"), NO_FORM.into()),
        (
            "P20",
            format!("aspect = \"1:1\"\nborder = 0.1\n{w}"),
            "'aspect' cannot be combined with 'border'; a pad is either an aspect (aspect, gravity) or a border".into(),
        ),
        (
            "P21",
            format!("border = 0.1\ngravity = \"north\"\n{w}"),
            "'gravity' is only used with 'aspect'; a border pad adds the same width on every side".into(),
        ),
        ("P22", "aspect = \"1:1\"".into(), NO_COLOR.into()),
        ("P23", "border = 0.1".into(), NO_COLOR.into()),
        (
            "P24",
            "aspect = \"1:1\"\ncolor = \"red\"".into(),
            format!("{NOT_HEX}, got \"red\""),
        ),
        (
            "P25",
            "aspect = \"1:1\"\ncolor = \"#ff00\"".into(),
            format!("{NOT_HEX}, got \"#ff00\""),
        ),
        (
            "P26",
            "aspect = \"1:1\"\ncolor = \"#gggggg\"".into(),
            format!("{NOT_HEX}, got \"#gggggg\""),
        ),
        (
            "P27",
            "aspect = \"1:1\"\ncolor = \"\"".into(),
            format!("{NOT_HEX}, got \"\""),
        ),
        (
            "P28",
            "aspect = \"1:1\"\ncolor = \"Transparent\"".into(),
            format!("{NOT_HEX}, got \"Transparent\""),
        ),
        (
            "P28b",
            "aspect = \"1:1\"\ncolor = \"TRANSPARENT\"".into(),
            format!("{NOT_HEX}, got \"TRANSPARENT\""),
        ),
        (
            "P33",
            format!("aspect = \"16x9\"\n{w}"),
            format!("{ASPECT}; got \"16x9\""),
        ),
        (
            "P34",
            format!("aspect = \"0:9\"\n{w}"),
            format!("{ASPECT}; got \"0:9\""),
        ),
        (
            "P35",
            format!("aspect = \"-1:1\"\n{w}"),
            format!("{ASPECT}; got \"-1:1\""),
        ),
        (
            "P36",
            format!("aspect = \"65536:1\"\n{w}"),
            format!("{ASPECT}; got \"65536:1\""),
        ),
        (
            "P37",
            format!("aspect = \"1.5:1\"\n{w}"),
            "'aspect' must be two whole numbers written W:H; write \"1.5:1\" as \"3:2\"".into(),
        ),
    ];

    let mut mismatches = Vec::new();
    for (id, body, message) in &rows {
        let want = format!("recipe step 0 ('pad'): {message}");
        match load(&recipe(&[&pad_step(body)])) {
            Ok(_) => mismatches.push(format!("{id}: expected {want:?}, but it built")),
            Err(e) => {
                let got = e.to_string();
                if got != want {
                    mismatches.push(format!("{id}: expected {want:?}, got {got:?}"));
                }
            }
        }
    }
    assert_no_mismatches(rows.len(), 34, mismatches);
}

#[test]
fn accepted_colour_and_value_spellings_load() {
    let bodies = [
        "aspect = \"1:1\"\ncolor = \"#FFFFFF\"",
        "aspect = \"1:1\"\ncolor = \"ffffff\"",
        "aspect = \"1:1\"\ncolor = \"#123456ff\"",
        "aspect = \"1:1\"\ncolor = \"123456FF\"",
        "border = 1\ncolor = \"#ffffff\"",
        "aspect = \"65535:1\"\ncolor = \"#ffffff\"",
        "aspect = \"1:1\"\ncolor = \"transparent\"",
        "aspect = \"1:1\"\ncolor = \"#ff000080\"",
        "border = 0.1\ncolor = \"ff000000\"",
        "aspect = \"1:1\"\ncolor = \"#ffffffFE\"",
    ];
    let mut mismatches = Vec::new();
    for body in bodies {
        if let Err(e) = load(&recipe(&[&pad_step(body)])) {
            mismatches.push(format!("{:?} was refused: {e}", body.replace('\n', " ")));
        }
    }
    assert_no_mismatches(bodies.len(), 10, mismatches);
}

// ── Exactness in every colour type ──────────────────────────────────────────

/// The output kind for an input kind and a fill, by the colour-type table: a
/// grey input widens to RGB (RGBA) only for a chromatic fill, at the same
/// depth; everything else keeps its type.
fn out_kind(kind: &str, fill: (u8, u8, u8)) -> &'static str {
    let chromatic = fill.0 != fill.1 || fill.1 != fill.2;
    match (kind, chromatic) {
        ("l8", true) => "rgb8",
        ("la8", true) => "rgba8",
        ("l16", true) => "rgb16",
        ("la16", true) => "rgba16",
        ("l8", false) => "l8",
        ("la8", false) => "la8",
        ("l16", false) => "l16",
        ("la16", false) => "la16",
        ("rgb8", _) => "rgb8",
        ("rgba8", _) => "rgba8",
        ("rgb16", _) => "rgb16",
        ("rgba16", _) => "rgba16",
        ("rgb32f", _) => "rgb32f",
        ("rgba32f", _) => "rgba32f",
        other => panic!("unknown colour type {other:?}"),
    }
}

/// The samples of one new pixel of kind `out`, filled with `fill`.
fn fill_pixel(out: &str, fill: (u8, u8, u8)) -> Vec<u64> {
    let (channels, bits, has_alpha) = shape_of(out);
    let color_channels = channels - usize::from(has_alpha);
    let mut px = if color_channels == 1 {
        // A grey output only ever gets a grey fill, so any channel will do.
        vec![scale(fill.0, bits)]
    } else {
        vec![
            scale(fill.0, bits),
            scale(fill.1, bits),
            scale(fill.2, bits),
        ]
    };
    if has_alpha {
        px.push(alpha_max(bits));
    }
    px
}

/// One input pixel (`in_kind`'s samples) as `out_kind`'s samples: copied, or
/// replicated across the colour channels where a grey pixel widens.
fn widen_pixel(px: &[u64], in_kind: &str, out: &str) -> Vec<u64> {
    if in_kind == out {
        return px.to_vec();
    }
    let (_, _, has_alpha) = shape_of(in_kind);
    let l = px[0];
    let mut v = vec![l, l, l];
    if has_alpha {
        v.push(px[1]);
    }
    v
}

/// Compare `out` against the expectation: kind, size, every input pixel at
/// `(x + left, y + top)` and every other pixel the fill.
fn check_pad(
    src: &DynamicImage,
    out: &DynamicImage,
    fill: (u8, u8, u8),
    canvas: (u32, u32),
    at: (u32, u32),
) -> Result<(), String> {
    let in_kind = kind_of(src);
    let want_kind = out_kind(in_kind, fill);
    if kind_of(out) != want_kind {
        return Err(format!(
            "colour type {} , expected {want_kind}",
            kind_of(out)
        ));
    }
    if (out.width(), out.height()) != canvas {
        return Err(format!(
            "size {}x{}, expected {}x{}",
            out.width(),
            out.height(),
            canvas.0,
            canvas.1
        ));
    }
    let (in_ch, _, _) = shape_of(in_kind);
    let (out_ch, _, _) = shape_of(want_kind);
    let (sin, sout) = (samples(src), samples(out));
    let fill_px = fill_pixel(want_kind, fill);
    let mut wrong = 0u32;
    let mut first = None;
    for y in 0..canvas.1 {
        for x in 0..canvas.0 {
            let inside =
                x >= at.0 && x < at.0 + src.width() && y >= at.1 && y < at.1 + src.height();
            let want = if inside {
                let i = ((y - at.1) as usize * src.width() as usize + (x - at.0) as usize) * in_ch;
                widen_pixel(&sin[i..i + in_ch], in_kind, want_kind)
            } else {
                fill_px.clone()
            };
            let o = (y as usize * canvas.0 as usize + x as usize) * out_ch;
            if sout[o..o + out_ch] != want[..] {
                wrong += 1;
                first.get_or_insert((x, y));
            }
        }
    }
    match first {
        None => Ok(()),
        Some((x, y)) => Err(format!("{wrong} pixels wrong, first at ({x}, {y})")),
    }
}

#[test]
fn every_colour_type_pads_exactly() {
    let fills = [(0x80u8, 0x80u8, 0x80u8), (0x12, 0x34, 0x56)];
    let shapes = [(7u32, 4u32), (6, 4), (4, 7), (4, 6)];
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for kind in KINDS {
        for fill in fills {
            for (w, h) in shapes {
                let src = seeded(kind, w, h);
                let side = w.max(h);
                for gravity in GRAVITIES {
                    cases += 1;
                    let step = pad_step(&format!(
                        "aspect = \"1:1\"\ngravity = \"{gravity}\"\ncolor = \"{}\"",
                        hex(fill)
                    ));
                    let out = run(&recipe(&[&step]), &src);
                    let at = offset(gravity, side, side, w, h);
                    if let Err(e) = check_pad(&src, &out, fill, (side, side), at) {
                        mismatches.push(format!(
                            "{kind} fill {} aspect 1:1 {gravity} {w}x{h}: {e}",
                            hex(fill)
                        ));
                    }
                }
                // round(0.25 × long edge), halves up, in integers.
                cases += 1;
                let px = (side + 2) / 4;
                let step = pad_step(&format!("border = 0.25\ncolor = \"{}\"", hex(fill)));
                let out = run(&recipe(&[&step]), &src);
                if let Err(e) = check_pad(&src, &out, fill, (w + 2 * px, h + 2 * px), (px, px)) {
                    mismatches.push(format!(
                        "{kind} fill {} border 0.25 {w}x{h}: {e}",
                        hex(fill)
                    ));
                }
            }
        }
    }
    assert_no_mismatches(cases, 800, mismatches);
}

/// One row of the colour-type table: an input kind, a fill, the output kind
/// and the fill's samples in it.
struct Row {
    kind: &'static str,
    fill: &'static str,
    out: &'static str,
    fill_samples: Vec<u64>,
}

fn f32_bits(c: u8) -> u64 {
    u64::from((f32::from(c) / 255.0).to_bits())
}

#[test]
fn colour_type_table_row_by_row() {
    let one = u64::from(1.0f32.to_bits());
    let rows = vec![
        Row {
            kind: "l8",
            fill: "#808080",
            out: "l8",
            fill_samples: vec![0x80],
        },
        Row {
            kind: "l8",
            fill: "#123456",
            out: "rgb8",
            fill_samples: vec![0x12, 0x34, 0x56],
        },
        Row {
            kind: "l8",
            fill: "#808081",
            out: "rgb8",
            fill_samples: vec![0x80, 0x80, 0x81],
        },
        Row {
            kind: "la8",
            fill: "#808080",
            out: "la8",
            fill_samples: vec![0x80, 0xff],
        },
        Row {
            kind: "la8",
            fill: "#123456",
            out: "rgba8",
            fill_samples: vec![0x12, 0x34, 0x56, 0xff],
        },
        Row {
            kind: "rgb8",
            fill: "#808080",
            out: "rgb8",
            fill_samples: vec![0x80, 0x80, 0x80],
        },
        Row {
            kind: "rgb8",
            fill: "#123456",
            out: "rgb8",
            fill_samples: vec![0x12, 0x34, 0x56],
        },
        Row {
            kind: "rgba8",
            fill: "#808080",
            out: "rgba8",
            fill_samples: vec![0x80, 0x80, 0x80, 0xff],
        },
        Row {
            kind: "rgba8",
            fill: "#123456",
            out: "rgba8",
            fill_samples: vec![0x12, 0x34, 0x56, 0xff],
        },
        Row {
            kind: "l16",
            fill: "#808080",
            out: "l16",
            fill_samples: vec![0x8080],
        },
        Row {
            kind: "l16",
            fill: "#123456",
            out: "rgb16",
            fill_samples: vec![0x1212, 0x3434, 0x5656],
        },
        Row {
            kind: "la16",
            fill: "#808080",
            out: "la16",
            fill_samples: vec![0x8080, 0xffff],
        },
        Row {
            kind: "la16",
            fill: "#123456",
            out: "rgba16",
            fill_samples: vec![0x1212, 0x3434, 0x5656, 0xffff],
        },
        Row {
            kind: "rgb16",
            fill: "#808080",
            out: "rgb16",
            fill_samples: vec![0x8080, 0x8080, 0x8080],
        },
        Row {
            kind: "rgb16",
            fill: "#123456",
            out: "rgb16",
            fill_samples: vec![0x1212, 0x3434, 0x5656],
        },
        Row {
            kind: "rgba16",
            fill: "#808080",
            out: "rgba16",
            fill_samples: vec![0x8080, 0x8080, 0x8080, 0xffff],
        },
        Row {
            kind: "rgba16",
            fill: "#123456",
            out: "rgba16",
            fill_samples: vec![0x1212, 0x3434, 0x5656, 0xffff],
        },
        Row {
            kind: "rgb32f",
            fill: "#808080",
            out: "rgb32f",
            fill_samples: vec![f32_bits(0x80); 3],
        },
        Row {
            kind: "rgb32f",
            fill: "#123456",
            out: "rgb32f",
            fill_samples: vec![f32_bits(0x12), f32_bits(0x34), f32_bits(0x56)],
        },
        Row {
            kind: "rgba32f",
            fill: "#808080",
            out: "rgba32f",
            fill_samples: vec![f32_bits(0x80), f32_bits(0x80), f32_bits(0x80), one],
        },
        Row {
            kind: "rgba32f",
            fill: "#123456",
            out: "rgba32f",
            fill_samples: vec![f32_bits(0x12), f32_bits(0x34), f32_bits(0x56), one],
        },
    ];

    let mut mismatches = Vec::new();
    for row in &rows {
        let (ch, bits, has_alpha) = shape_of(row.kind);
        // A 2×1 seeded buffer whose first pixel has alpha 0 where there is alpha.
        let mut src = seeded(row.kind, 2, 1);
        if has_alpha {
            src = zero_first_alpha(src);
        }
        let step = pad_step(&format!("border = 1\ncolor = \"{}\"", row.fill));
        let out = run(&recipe(&[&step]), &src);
        let label = format!("{} + {}", row.kind, row.fill);
        if kind_of(&out) != row.out {
            mismatches.push(format!(
                "{label}: became {}, expected {}",
                kind_of(&out),
                row.out
            ));
            continue;
        }
        // border = 1 on a 2-px long edge adds 2 px on every side: 6×5.
        if (out.width(), out.height()) != (6, 5) {
            mismatches.push(format!("{label}: size {}x{}", out.width(), out.height()));
            continue;
        }
        let (out_ch, _, _) = shape_of(row.out);
        let so = samples(&out);
        if so[..out_ch] != row.fill_samples[..] {
            mismatches.push(format!(
                "{label}: fill at (0, 0) is {:?}, expected {:?}",
                &so[..out_ch],
                row.fill_samples
            ));
        }
        // The input's first pixel is at (2, 2), alpha untouched.
        let si = samples(&src);
        let want = widen_pixel(&si[..ch], row.kind, row.out);
        let at = (2 * 6 + 2) * out_ch;
        if so[at..at + out_ch] != want[..] {
            mismatches.push(format!(
                "{label}: input pixel at (2, 2) is {:?}, expected {want:?}",
                &so[at..at + out_ch]
            ));
        }
        let _ = bits;
    }
    assert_no_mismatches(rows.len(), 21, mismatches);
}

/// `src` with its first pixel's alpha set to 0.
fn zero_first_alpha(mut src: DynamicImage) -> DynamicImage {
    match &mut src {
        DynamicImage::ImageLumaA8(b) => b.get_pixel_mut(0, 0).0[1] = 0,
        DynamicImage::ImageRgba8(b) => b.get_pixel_mut(0, 0).0[3] = 0,
        DynamicImage::ImageLumaA16(b) => b.get_pixel_mut(0, 0).0[1] = 0,
        DynamicImage::ImageRgba16(b) => b.get_pixel_mut(0, 0).0[3] = 0,
        DynamicImage::ImageRgba32F(b) => b.get_pixel_mut(0, 0).0[3] = 0.0,
        other => panic!("no alpha in {}", kind_of(other)),
    }
    src
}

#[test]
fn sixteen_bit_fill_is_times_257() {
    let cases: [(&str, &str, Vec<u64>); 4] = [
        ("rgb16", "#123456", vec![0x1212, 0x3434, 0x5656]),
        ("rgba16", "#123456", vec![0x1212, 0x3434, 0x5656, 0xffff]),
        ("l16", "#ffffff", vec![65535]),
        ("l16", "#000000", vec![0]),
    ];
    let mut mismatches = Vec::new();
    for (kind, fill, want) in &cases {
        let out = run(
            &recipe(&[&pad_step(&format!("border = 1\ncolor = \"{fill}\""))]),
            &seeded(kind, 2, 1),
        );
        let got = samples(&out);
        if got[..want.len()] != want[..] {
            mismatches.push(format!(
                "{kind} + {fill}: fill is {:?}, expected {want:?}",
                &got[..want.len()]
            ));
        }
    }
    assert_no_mismatches(cases.len(), 4, mismatches);
}

#[test]
fn float_images_keep_their_variant() {
    let mut mismatches = Vec::new();
    for kind in ["rgb32f", "rgba32f"] {
        let src = seeded(kind, 3, 2);
        // border = 1 on a 3-px long edge adds 3 px: 9×8, the input at (3, 3).
        let out = run(
            &recipe(&[&pad_step("border = 1\ncolor = \"#ff8000\"")]),
            &src,
        );
        if kind_of(&out) != kind {
            mismatches.push(format!("{kind}: became {}", kind_of(&out)));
            continue;
        }
        let (ch, _, has_alpha) = shape_of(kind);
        let mut want_fill = vec![
            u64::from(1.0f32.to_bits()),
            u64::from((128.0f32 / 255.0).to_bits()),
            u64::from(0.0f32.to_bits()),
        ];
        if has_alpha {
            want_fill.push(u64::from(1.0f32.to_bits()));
        }
        let (so, si) = (samples(&out), samples(&src));
        if so[..ch] != want_fill[..] {
            mismatches.push(format!(
                "{kind}: fill {:?}, expected {want_fill:?}",
                &so[..ch]
            ));
        }
        for y in 0..2usize {
            for x in 0..3usize {
                let o = ((y + 3) * 9 + (x + 3)) * ch;
                let i = (y * 3 + x) * ch;
                if so[o..o + ch] != si[i..i + ch] {
                    mismatches.push(format!("{kind}: input pixel ({x}, {y}) is not bit-equal"));
                }
            }
        }
    }
    assert_no_mismatches(2, 2, mismatches);
}

// ── Gravity, by absolute position ───────────────────────────────────────────

/// `(gravity, offset on a 5×2 image, offset on a 2×5 image)`, both padded to
/// `1:1` (a 5×5 canvas), as literals.
type Offset = (u32, u32);

const GRAVITY_TABLE: [(&str, Offset, Offset); 9] = [
    ("center", (0, 1), (1, 0)),
    ("north", (0, 0), (1, 0)),
    ("south", (0, 3), (1, 0)),
    ("east", (0, 1), (3, 0)),
    ("west", (0, 1), (0, 0)),
    ("northeast", (0, 0), (3, 0)),
    ("northwest", (0, 0), (0, 0)),
    ("southeast", (0, 3), (3, 0)),
    ("southwest", (0, 3), (0, 0)),
];

#[test]
fn gravity_corners_land_by_absolute_position() {
    let mut mismatches = Vec::new();
    let mut cases = 0;
    for (gravity, wide, tall) in GRAVITY_TABLE {
        for ((w, h), (left, top)) in [((5u32, 2u32), wide), ((2, 5), tall)] {
            cases += 1;
            let src = seeded("rgb8", w, h);
            let out = run(
                &recipe(&[&pad_step(&format!(
                    "aspect = \"1:1\"\ngravity = \"{gravity}\"\ncolor = \"#123456\""
                ))]),
                &src,
            );
            if (out.width(), out.height()) != (5, 5) {
                mismatches.push(format!(
                    "{gravity} {w}x{h}: canvas {}x{}",
                    out.width(),
                    out.height()
                ));
                continue;
            }
            let (so, si) = (samples(&out), samples(&src));
            let px = |buf: &[u64], width: u32, x: u32, y: u32| -> Vec<u64> {
                let i = (y as usize * width as usize + x as usize) * 3;
                buf[i..i + 3].to_vec()
            };
            if px(&so, 5, left, top) != px(&si, w, 0, 0) {
                mismatches.push(format!(
                    "{gravity} {w}x{h}: input (0, 0) is not at ({left}, {top})"
                ));
            }
            if px(&so, 5, left + w - 1, top + h - 1) != px(&si, w, w - 1, h - 1) {
                mismatches.push(format!(
                    "{gravity} {w}x{h}: input ({}, {}) is not at ({}, {})",
                    w - 1,
                    h - 1,
                    left + w - 1,
                    top + h - 1
                ));
            }
        }
    }
    assert_no_mismatches(cases, 18, mismatches);
}

// ── Pad then crop is the identity ───────────────────────────────────────────

#[test]
fn pad_then_crop_is_the_identity() {
    // (width, height, pad body, the aspect or border that makes the canvas)
    enum Form {
        Aspect(u64, u64, &'static str),
        Border(f64),
    }
    let cases: [(u32, u32, Form); 6] = [
        (37, 23, Form::Aspect(16, 9, "southeast")),
        (23, 37, Form::Aspect(3, 2, "northwest")),
        (1201, 801, Form::Aspect(3, 2, "center")),
        (7, 501, Form::Aspect(1, 1, "east")),
        (501, 7, Form::Border(0.3)),
        (1, 1, Form::Border(1.0)),
    ];
    let mut mismatches = Vec::new();
    for (w, h, form) in &cases {
        let (w, h) = (*w, *h);
        let (body, canvas, at) = match form {
            Form::Aspect(a, b, gravity) => {
                let canvas = aspect_canvas(*a, *b, u64::from(w), u64::from(h));
                (
                    format!("aspect = \"{a}:{b}\"\ngravity = \"{gravity}\"\ncolor = \"#102030\""),
                    canvas,
                    offset(gravity, canvas.0, canvas.1, w, h),
                )
            }
            Form::Border(f) => {
                let px = (f * f64::from(w.max(h))).round() as u32;
                (
                    format!("border = {f:?}\ncolor = \"#102030\""),
                    (w + 2 * px, h + 2 * px),
                    (px, px),
                )
            }
        };
        let label = format!("{w}x{h} {}", body.replace('\n', " "));
        let (cw, ch) = (f64::from(canvas.0), f64::from(canvas.1));
        let crop = format!(
            "op = \"crop\"\nx = {:?}\ny = {:?}\nwidth = {:?}\nheight = {:?}",
            f64::from(at.0) / cw,
            f64::from(at.1) / ch,
            f64::from(w) / cw,
            f64::from(h) / ch
        );
        let src = seeded("rgba16", w, h);
        let toml = recipe(&[&pad_step(&body), &crop]);
        let padded = run(&recipe(&[&pad_step(&body)]), &src);
        if (padded.width(), padded.height()) != canvas {
            mismatches.push(format!(
                "{label}: pad made {}x{}, expected {}x{}",
                padded.width(),
                padded.height(),
                canvas.0,
                canvas.1
            ));
            continue;
        }
        let out = run(&toml, &src);
        if kind_of(&out) != "rgba16" || out.as_bytes() != src.as_bytes() {
            mismatches.push(format!("{label}: crop did not give the input back"));
        }
    }
    assert_no_mismatches(cases.len(), 6, mismatches);
}

// ── The binary ──────────────────────────────────────────────────────────────

struct Applied {
    code: Option<i32>,
    stderr: String,
}

fn run_apply(args: &[&Path], dir: &Path, toml: &str) -> Applied {
    let recipe_path = dir.join("pad.toml");
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

/// A1 on 40×30 at `border = 0.001`.
const A1: &str = "the border rounds to 0 px on this 40×30 px image ('border' = 0.001 of its 40 px long edge); it must add at least 1 px";
/// A2 on a 1×1 image at `aspect = "65535:1"`.
const A2: &str = "'aspect' = \"65535:1\" on this 1×1 px image needs a 65535×1 px canvas, which exceeds the maximum edge limit of 50000 px";
/// A3 on a 1×11586 image at `aspect = "1:1"`.
const A3: &str = "'aspect' = \"1:1\" on this 1×11586 px image needs a 11586×11586 px canvas = 134235396 px, which exceeds the maximum area of 134217728 px";

#[test]
fn apply_pads_a_png_end_to_end() {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 4, 3);
    let input = write_png(dir.path(), "in.png", &src);
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step("aspect = \"1:1\"\ncolor = \"#123456\"")]),
        &input,
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!((out.width(), out.height()), (4, 4));
    let (so, si) = (samples(&out), samples(&src));
    assert_eq!(&so[..4 * 3 * 3], &si[..], "rows 0 to 2 are the input");
    for x in 0..4usize {
        let i = (3 * 4 + x) * 3;
        assert_eq!(&so[i..i + 3], &[0x12, 0x34, 0x56], "row 3, column {x}");
    }
}

#[test]
fn apply_reads_the_border_as_f64() {
    // 0.01 of a 50 px long edge is exactly half a pixel in f64 (rounds to 1 px) but
    // 0.49999999 in f32 (rounds to 0 px), so a narrowed read changes the result.
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 50, 10);
    let input = write_png(dir.path(), "in.png", &src);
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step("border = 0.01\ncolor = \"#ffffff\"")]),
        &input,
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!((out.width(), out.height()), (52, 12));
    let (so, si) = (samples(&out), samples(&src));
    for y in 0..12usize {
        for x in 0..52usize {
            let i = (y * 52 + x) * 3;
            if (1..11).contains(&y) && (1..51).contains(&x) {
                let j = ((y - 1) * 50 + (x - 1)) * 3;
                assert_eq!(&so[i..i + 3], &si[j..j + 3], "input pixel ({x}, {y})");
            } else {
                assert_eq!(&so[i..i + 3], &[0xff; 3], "frame pixel ({x}, {y})");
            }
        }
    }
}

#[test]
fn apply_no_op_pad_writes_the_input_pixels() {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 6, 4);
    let input = write_png(dir.path(), "in.png", &src);
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step("aspect = \"3:2\"\ncolor = \"#123456\"")]),
        &input,
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("out.png")).expect("output decodes");
    assert_eq!((out.width(), out.height()), (6, 4));
    assert_eq!(samples(&out), samples(&src));
}

#[test]
fn apply_refuses_a_pixel_width_key_and_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("rgb8", 7, 5));
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step(
            "aspect = \"1:1\"\ncolor = \"#ffffff\"\nwidth = 1080",
        )]),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: recipe step 0 ('pad'): unknown key 'width'; {KEYS}")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn apply_fails_a_0px_border_with_exit_1() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("rgb8", 40, 30));
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step("border = 0.001\ncolor = \"#ffffff\"")]),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: operation 'pad' failed: {A1}")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn batch_with_a_0px_border_fails_that_input_with_exit_6() {
    let dir = TempDir::new().unwrap();
    let a = write_png(dir.path(), "a.png", &seeded("rgb8", 40, 30));
    let b = write_png(dir.path(), "b.png", &seeded("rgb8", 4000, 3));
    let r = apply_batch(
        dir.path(),
        &recipe(&[&pad_step("border = 0.001\ncolor = \"#ffffff\"")]),
        &[&a, &b],
    );
    assert_eq!(r.code, Some(6), "stderr: {}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("a.png: operation 'pad' failed: {A1}")),
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
    assert_eq!((out.width(), out.height()), (4008, 11));
}

#[test]
fn apply_fails_a_canvas_past_the_edge_cap_with_exit_1() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", &seeded("l8", 1, 1));
    let r = apply(
        dir.path(),
        &recipe(&[&pad_step("aspect = \"65535:1\"\ncolor = \"#ffffff\"")]),
        &input,
    );
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr.trim_end(),
        format!("error: operation 'pad' failed: {A2}")
    );
    assert!(!dir.path().join("out.png").exists(), "no file is written");
}

#[test]
fn batch_with_a_canvas_past_the_area_cap_fails_that_input_with_exit_6() {
    let dir = TempDir::new().unwrap();
    let a = write_png(dir.path(), "a.png", &seeded("l8", 1, 11586));
    let b = write_png(dir.path(), "b.png", &seeded("l8", 1, 100));
    let r = apply_batch(
        dir.path(),
        &recipe(&[&pad_step("aspect = \"1:1\"\ncolor = \"#ffffff\"")]),
        &[&a, &b],
    );
    assert_eq!(r.code, Some(6), "stderr: {}", r.stderr);
    assert!(
        r.stderr
            .contains(&format!("a.png: operation 'pad' failed: {A3}")),
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
    assert_eq!((out.width(), out.height()), (100, 100));
}

// ── Metadata: pad acts on the stored pixels and writes no tag ───────────────

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

fn apply_to_tagged(steps: &[&str]) -> (Applied, Option<DynamicImage>, Vec<u8>) {
    let dir = TempDir::new().unwrap();
    let src = seeded("rgb8", 7, 5);
    let input = dir.path().join("in.png");
    std::fs::write(&input, tagged_png(&src)).expect("write input");
    let r = apply(dir.path(), &recipe(steps), &input);
    let out_path = dir.path().join("out.png");
    let bytes = std::fs::read(&out_path).unwrap_or_default();
    let decoded = image::open(&out_path).ok();
    (r, decoded, bytes)
}

#[test]
fn pad_writes_no_orientation_tag_and_keeps_the_bundle() {
    let src = seeded("rgb8", 7, 5);

    // Control A: the reader sees the tag on the input bytes.
    assert_eq!(
        common::orientation_of(&tagged_png(&src)),
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

    // pad border = 0.2 on a 7-px long edge adds round(1.4) = 1 px: 9×7.
    let pad = pad_step("border = 0.2\ncolor = \"#ffffff\"");
    let (r, out, bytes) = apply_to_tagged(&[&pad]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("pad output decodes");
    assert_eq!(
        (out.width(), out.height()),
        (9, 7),
        "pad alone: {}",
        r.stderr
    );
    match common::orientation_of(&bytes) {
        None | Some(1) => {}
        Some(other) => panic!(
            "the pixel lane carried the source's Orientation tag through ({other}); it must \
             write 1 or nothing"
        ),
    }

    // The bundle survived the pad, so a later auto-orient still turns it.
    let (r, out, _) = apply_to_tagged(&[&pad, "op = \"auto-orient\""]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = out.expect("pad then auto-orient decodes");
    assert_eq!(
        (out.width(), out.height()),
        (7, 9),
        "pad dropped the metadata bundle, so the later auto-orient found no tag"
    );
}

// ── The byte pin: an opaque pad's output does not change ───────────────────

/// FNV-1a 64 of a byte string.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A 5×3 PNG whose samples come from a 64-bit xorshift, one draw per sample in
/// pixel-then-channel order, masked to the sample width. Built from raw samples
/// by `image`'s encoder: no crustyimg code is involved.
fn pin_fixture(kind: &str, seed: u64) -> DynamicImage {
    let mut r = XorShift(seed);
    let (channels, bits, _) = shape_of(kind);
    let count = 5 * 3 * channels;
    match bits {
        8 => {
            let raw: Vec<u8> = (0..count).map(|_| r.next() as u8).collect();
            match kind {
                "l8" => DynamicImage::ImageLuma8(GrayImage::from_raw(5, 3, raw).unwrap()),
                "rgb8" => DynamicImage::ImageRgb8(ImageBuffer::from_raw(5, 3, raw).unwrap()),
                _ => DynamicImage::ImageRgba8(ImageBuffer::from_raw(5, 3, raw).unwrap()),
            }
        }
        _ => {
            let raw: Vec<u16> = (0..count).map(|_| r.next() as u16).collect();
            DynamicImage::ImageRgb16(ImageBuffer::from_raw(5, 3, raw).unwrap())
        }
    }
}

#[test]
fn opaque_pad_byte_pin() {
    // (kind, seed, fill, byte length, FNV-1a 64 of the output's samples),
    // recorded from the output of a build that predates translucent fills and
    // matched there by an independent model of the expected bytes.
    let rows: [(&str, u64, &str, usize, u64); 4] = [
        ("l8", 0x159_0000, "#808080", 35, 0xc9a6_1194_fef5_1c54),
        ("rgb8", 0x159_0001, "#123456", 105, 0xd66d_9899_5f85_0ac0),
        ("rgba8", 0x159_0002, "#123456", 140, 0xb8cb_45d4_dd54_0cc3),
        ("rgb16", 0x159_0003, "#123456", 210, 0x3107_3686_7184_2126),
    ];
    let mut mismatches = Vec::new();
    for (kind, seed, color, len, want) in rows {
        let dir = TempDir::new().unwrap();
        let input = write_png(dir.path(), "n.png", &pin_fixture(kind, seed));
        let out = dir.path().join("n.out.png");
        let toml = recipe(&[&pad_step(&format!("border = 0.2\ncolor = \"{color}\""))]);
        let r = run_apply(&[&input, Path::new("-o"), &out], dir.path(), &toml);
        if r.code != Some(0) {
            mismatches.push(format!("{kind}: exit {:?}, stderr {}", r.code, r.stderr));
            continue;
        }
        let img = image::open(&out).expect("output decodes");
        let bytes = img.as_bytes();
        if kind_of(&img) != kind || (img.width(), img.height()) != (7, 5) {
            mismatches.push(format!("{kind}: came out {}", kind_of(&img)));
        }
        if bytes.len() != len || fnv1a64(bytes) != want {
            mismatches.push(format!(
                "{kind}: {} bytes, hash {:#018x}; expected {len} bytes, {want:#018x}",
                bytes.len(),
                fnv1a64(bytes)
            ));
        }
    }
    assert_no_mismatches(rows.len(), 4, mismatches);
}

// ── Translucent fills: the fill, the widening, the rule ────────────────────

/// A fill as `[r, g, b, a]`, written from the spec's rules, never parsed by the
/// crate under test.
type Fill4 = [u8; 4];

/// The five fills every colour type is driven with: `transparent` as the
/// keyword, a coloured alpha-0 fill, a coloured half-alpha fill, an opaque
/// colour, and a grey half-alpha fill (the one that splits grey + alpha from
/// RGBA).
const FILLS: [(&str, Fill4); 5] = [
    ("transparent", [0, 0, 0, 0]),
    ("#ff000000", [255, 0, 0, 0]),
    ("#ff000080", [255, 0, 0, 128]),
    ("#ff0000", [255, 0, 0, 255]),
    ("#80808080", [128, 128, 128, 128]),
];

/// The output kind: a grey input widens to RGB(A) only for a fill that is not
/// grey; an input without alpha gains it only for a fill below `ff`; an alpha
/// input keeps it. A grey image with a grey translucent fill is grey + alpha.
fn out_kind4(kind: &str, fill: Fill4) -> &'static str {
    let chromatic = fill[0] != fill[1] || fill[1] != fill[2];
    let translucent = fill[3] != 255;
    match (kind, chromatic, translucent) {
        ("l8", false, false) => "l8",
        ("l8", true, false) => "rgb8",
        ("l8", false, true) => "la8",
        ("l8", true, true) => "rgba8",
        ("l16", false, false) => "l16",
        ("l16", true, false) => "rgb16",
        ("l16", false, true) => "la16",
        ("l16", true, true) => "rgba16",
        ("la8", false, _) => "la8",
        ("la8", true, _) => "rgba8",
        ("la16", false, _) => "la16",
        ("la16", true, _) => "rgba16",
        ("rgb8", _, false) => "rgb8",
        ("rgb8", _, true) => "rgba8",
        ("rgb16", _, false) => "rgb16",
        ("rgb16", _, true) => "rgba16",
        ("rgb32f", _, false) => "rgb32f",
        ("rgb32f", _, true) => "rgba32f",
        ("rgba8", _, _) => "rgba8",
        ("rgba16", _, _) => "rgba16",
        ("rgba32f", _, _) => "rgba32f",
        other => panic!("unknown colour type {other:?}"),
    }
}

/// The samples of one new pixel of kind `out`: the fill's own alpha, not the
/// type's maximum.
fn fill_px4(out: &str, fill: Fill4) -> Vec<u64> {
    let (channels, bits, has_alpha) = shape_of(out);
    let colour = channels - usize::from(has_alpha);
    let mut px = if colour == 1 {
        vec![scale(fill[0], bits)]
    } else {
        vec![
            scale(fill[0], bits),
            scale(fill[1], bits),
            scale(fill[2], bits),
        ]
    };
    if has_alpha {
        px.push(scale(fill[3], bits));
    }
    px
}

#[test]
fn translucent_fill_by_colour_type() {
    let mut mismatches = Vec::new();
    let mut cells = 0;
    for kind in KINDS {
        for (text, fill) in FILLS {
            cells += 1;
            let src = seeded(kind, 3, 3);
            // 0.34 of a 3 px edge rounds to 1 px: a 5×5 canvas, the source at (1, 1).
            let step = pad_step(&format!("border = 0.34\ncolor = \"{text}\""));
            let pipeline = load(&recipe(&[&step]));
            let out = match pipeline {
                Ok(p) => p.run(wrap(src.clone())).expect("runs").pixels().clone(),
                Err(e) => {
                    mismatches.push(format!("{kind} + {text}: did not load: {e}"));
                    continue;
                }
            };
            let want_kind = out_kind4(kind, fill);
            if kind_of(&out) != want_kind {
                mismatches.push(format!(
                    "{kind} + {text}: {}, expected {want_kind}",
                    kind_of(&out)
                ));
                continue;
            }
            let (in_ch, _, _) = shape_of(kind);
            let (out_ch, _, _) = shape_of(want_kind);
            let (sin, sout) = (samples(&src), samples(&out));
            let fill_px = fill_px4(want_kind, fill);
            let mut wrong = 0;
            for y in 0..5usize {
                for x in 0..5usize {
                    let inside = (1..4).contains(&x) && (1..4).contains(&y);
                    let want = if inside {
                        let i = ((y - 1) * 3 + (x - 1)) * in_ch;
                        widen_pixel(&sin[i..i + in_ch], kind, want_kind)
                    } else {
                        fill_px.clone()
                    };
                    let o = (y * 5 + x) * out_ch;
                    if sout[o..o + out_ch] != want[..] {
                        wrong += 1;
                    }
                }
            }
            if wrong > 0 {
                mismatches.push(format!(
                    "{kind} + {text}: {wrong} of 25 pixels wrong (fill expected {fill_px:?})"
                ));
            }
        }
    }
    assert_no_mismatches(cells, 50, mismatches);
}

#[test]
fn at_aspect_translucent_fill_keeps_type_and_bytes() {
    let mut mismatches = Vec::new();
    for kind in KINDS {
        let src = seeded(kind, 4, 4);
        let step = pad_step("aspect = \"1:1\"\ncolor = \"transparent\"");
        let out = match load(&recipe(&[&step])) {
            Ok(p) => p.run(wrap(src.clone())).expect("runs").pixels().clone(),
            Err(e) => {
                mismatches.push(format!("{kind}: did not load: {e}"));
                continue;
            }
        };
        if kind_of(&out) != kind || out.as_bytes() != src.as_bytes() {
            mismatches.push(format!("{kind}: came out {}", kind_of(&out)));
        }
    }
    assert_no_mismatches(KINDS.len(), 10, mismatches);
}

// ── Pad, then trim, with a translucent fill ─────────────────────────────────

/// A 3×3 RGB buffer of `bits` width whose every channel is odd, so no sample
/// equals a fill channel of 0 and none equals 65535 at 16 bits.
fn odd_rgb(bits: u32) -> DynamicImage {
    let mut r = XorShift(0x5EED_0159);
    if bits == 8 {
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(3, 3, |_, _| {
            let n = r.next();
            Rgb([(n as u8) | 1, ((n >> 8) as u8) | 1, ((n >> 16) as u8) | 1])
        }))
    } else {
        DynamicImage::ImageRgb16(ImageBuffer::from_fn(3, 3, |_, _| {
            let n = r.next();
            Rgb([
                ((n as u16) | 1) & 0x7fff,
                (((n >> 16) as u16) | 1) & 0x7fff,
                (((n >> 32) as u16) | 1) & 0x7fff,
            ])
        }))
    }
}

/// The colour channels of every pixel, alpha dropped.
fn colour_samples(img: &DynamicImage) -> Vec<u64> {
    let (channels, _, has_alpha) = shape_of(kind_of(img));
    let colour = channels - usize::from(has_alpha);
    samples(img)
        .chunks_exact(channels)
        .flat_map(|px| px[..colour].to_vec())
        .collect()
}

#[test]
fn pad_then_trim_transparent_round_trip() {
    // (source bits, colour written to both steps, the output kind)
    let cases = [(8, "transparent", "rgba8"), (16, "#ff000080", "rgba16")];
    let mut mismatches = Vec::new();
    for (bits, color, want_kind) in cases {
        let src = odd_rgb(bits);
        let steps = [
            pad_step(&format!("border = 0.67\ncolor = \"{color}\"")),
            format!("op = \"trim\"\ncolor = \"{color}\""),
        ];
        let toml = recipe(&[&steps[0], &steps[1]]);
        let out = match load(&toml) {
            Ok(p) => p.run(wrap(src.clone())).expect("runs").pixels().clone(),
            Err(e) => {
                mismatches.push(format!("{bits}-bit {color}: did not load: {e}"));
                continue;
            }
        };
        if kind_of(&out) != want_kind || (out.width(), out.height()) != (3, 3) {
            mismatches.push(format!(
                "{bits}-bit {color}: {} {}x{}",
                kind_of(&out),
                out.width(),
                out.height()
            ));
            continue;
        }
        if colour_samples(&out) != samples(&src) {
            mismatches.push(format!("{bits}-bit {color}: the colour channels changed"));
        }
    }
    assert_no_mismatches(cases.len(), 2, mismatches);
}

// ── The binary: the output format rule ──────────────────────────────────────

struct Ran {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run the binary with `dir` as its working directory, so relative paths (and
/// so input labels) are the ones a user types.
fn bin(dir: &Path, args: &[&str]) -> Ran {
    let output = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run crustyimg");
    Ran {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Write `recipe.toml` into `dir` and run `apply --recipe recipe.toml <args> -y`.
fn apply_in(dir: &Path, toml: &str, args: &[&str]) -> Ran {
    std::fs::write(dir.join("recipe.toml"), toml).expect("write recipe");
    let mut all = vec!["apply", "--recipe", "recipe.toml"];
    all.extend_from_slice(args);
    all.push("-y");
    bin(dir, &all)
}

/// The recipe with a top-level `format`.
fn with_format(toml: &str, format: &str) -> String {
    toml.replacen(
        "version = \"2\"\n",
        &format!("version = \"2\"\nformat = \"{format}\"\n"),
        1,
    )
}

fn rgb4() -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(4, 4, |x, y| {
        Rgb([(x * 50) as u8, (y * 50) as u8, 90])
    }))
}

/// A 4×4 RGBA8: the left half opaque red, the right half alpha 0.
fn rgba4() -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(4, 4, |x, _| {
        if x < 2 {
            Rgba([255, 0, 0, 255])
        } else {
            Rgba([0, 255, 0, 0])
        }
    }))
}

/// A 64×48 RGB8 xorshift photo.
fn photo() -> DynamicImage {
    let mut r = XorShift(0x159a070);
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(64, 48, |_, _| {
        let n = r.next();
        Rgb([n as u8, (n >> 8) as u8, (n >> 16) as u8])
    }))
}

fn write_as(path: &Path, img: &DynamicImage, format: ImageFormat) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, format).expect("encode fixture");
    std::fs::write(path, out.into_inner()).expect("write fixture");
}

/// `rgb.png`, `src.jpg` (both 4×4 RGB8) and `rgba.png` in `dir`.
fn fixtures(dir: &Path) {
    write_as(&dir.join("rgb.png"), &rgb4(), ImageFormat::Png);
    write_as(&dir.join("src.jpg"), &rgb4(), ImageFormat::Jpeg);
    write_as(&dir.join("rgba.png"), &rgba4(), ImageFormat::Png);
}

/// A border pad with a fill, as a one-step recipe.
fn pad_recipe(color: &str) -> String {
    recipe(&[&pad_step(&format!("border = 0.25\ncolor = \"{color}\""))])
}

/// The refusal text for a fill into JPEG, written out from the spec.
fn jpeg_line(step: usize, written: &str, aa: &str) -> String {
    format!("recipe step {step} ('pad'): 'color' = \"{written}\" has alpha {aa}, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque")
}

/// The refusal text for a fill into GIF, written out from the spec.
fn gif_line(step: usize, written: &str, aa: &str) -> String {
    format!("recipe step {step} ('pad'): 'color' = \"{written}\" has alpha {aa}, and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead")
}

#[test]
fn apply_one_input_refuses_on_every_format_rung() {
    let transparent = pad_recipe("transparent");
    let half = pad_recipe("#ff000080");
    let rows: Vec<(&str, Vec<&str>, String, String, &str)> = vec![
        (
            "-o with a .jpg extension",
            vec!["rgb.png", "-o", "o.jpg"],
            transparent.clone(),
            jpeg_line(0, "transparent", "00"),
            "o.jpg",
        ),
        (
            "--format jpeg over a .bin name",
            vec!["rgb.png", "--format", "jpeg", "-o", "o.bin"],
            half.clone(),
            jpeg_line(0, "#ff000080", "80"),
            "o.bin",
        ),
        (
            "recipe.format with a bare -o",
            vec!["rgb.png", "-o", "o2"],
            with_format(&transparent, "jpeg"),
            jpeg_line(0, "transparent", "00"),
            "o2",
        ),
        (
            "a JPEG source keeps its format",
            vec!["src.jpg", "-o", "o3"],
            transparent.clone(),
            jpeg_line(0, "transparent", "00"),
            "o3",
        ),
        (
            "-o with a .gif extension",
            vec!["rgb.png", "-o", "o.gif"],
            half.clone(),
            gif_line(0, "#ff000080", "80"),
            "o.gif",
        ),
    ];
    let mut mismatches = Vec::new();
    for (label, args, toml, line, written) in &rows {
        let dir = TempDir::new().unwrap();
        fixtures(dir.path());
        let r = apply_in(dir.path(), toml, args);
        if r.code != Some(4) || r.stderr != format!("error: {line}\n") {
            mismatches.push(format!(
                "{label}: exit {:?}, stderr {:?}, expected exit 4 and {line:?}",
                r.code, r.stderr
            ));
        }
        if dir.path().join(written).exists() {
            mismatches.push(format!("{label}: {written} was written"));
        }
    }
    assert_no_mismatches(rows.len(), 5, mismatches);
}

#[test]
fn gif_takes_alpha_00_and_ff_only() {
    let mut mismatches = Vec::new();
    let mut rows = 0;
    for aa in ["00", "01", "80", "fe", "ff"] {
        rows += 1;
        let dir = TempDir::new().unwrap();
        fixtures(dir.path());
        let color = format!("#ff0000{aa}");
        let out = format!("g{aa}.gif");
        let r = apply_in(dir.path(), &pad_recipe(&color), &["rgb.png", "-o", &out]);
        let stored = aa == "00" || aa == "ff";
        let wrote = dir.path().join(&out).exists();
        if stored {
            if r.code != Some(0) || !wrote {
                mismatches.push(format!(
                    "{aa}: exit {:?}, wrote {wrote}: {}",
                    r.code, r.stderr
                ));
            }
        } else if r.code != Some(4)
            || r.stderr != format!("error: {}\n", gif_line(0, &color, aa))
            || wrote
        {
            mismatches.push(format!(
                "{aa}: exit {:?}, wrote {wrote}: {}",
                r.code, r.stderr
            ));
        }
    }
    // The keyword is alpha 00 and is stored.
    rows += 1;
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let r = apply_in(
        dir.path(),
        &pad_recipe("transparent"),
        &["rgb.png", "-o", "t.gif"],
    );
    if r.code != Some(0) || !dir.path().join("t.gif").exists() {
        mismatches.push(format!("transparent: exit {:?}: {}", r.code, r.stderr));
    }
    assert_no_mismatches(rows, 6, mismatches);

    // Read back: a `#ff000000` border is (255, 0, 0, 0) in the GIF, and
    // trimming `transparent` gives the 4×4 original's colour back.
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let made = apply_in(
        dir.path(),
        &pad_recipe("#ff000000"),
        &["rgb.png", "-o", "g.gif"],
    );
    assert_eq!(made.code, Some(0), "stderr: {}", made.stderr);
    let gif = image::open(dir.path().join("g.gif")).expect("gif decodes");
    assert_eq!((gif.width(), gif.height()), (6, 6));
    assert_eq!(gif.to_rgba8().get_pixel(0, 0).0, [255, 0, 0, 0]);
    let trim = recipe(&["op = \"trim\"\ncolor = \"transparent\""]);
    let back = apply_in(dir.path(), &trim, &["g.gif", "-o", "t.png"]);
    assert_eq!(back.code, Some(0), "stderr: {}", back.stderr);
    let out = image::open(dir.path().join("t.png")).expect("trimmed decodes");
    assert_eq!((out.width(), out.height()), (4, 4));
    assert_eq!(out.to_rgb8().as_raw(), rgb4().to_rgb8().as_raw());
}

#[test]
fn batch_with_one_format_refuses_before_reading() {
    let transparent = pad_recipe("transparent");
    let line = jpeg_line(0, "transparent", "00");
    let rows: Vec<(&str, String, Vec<&str>)> = vec![
        (
            "--format jpeg",
            transparent.clone(),
            vec!["src.jpg", "rgb.png", "--format", "jpeg", "--out-dir", "out"],
        ),
        (
            "--name-template with .jpg",
            transparent.clone(),
            vec![
                "src.jpg",
                "rgb.png",
                "--name-template",
                "{stem}.jpg",
                "--out-dir",
                "out",
            ],
        ),
        (
            "recipe.format",
            with_format(&transparent, "jpeg"),
            vec!["src.jpg", "rgb.png", "--out-dir", "out"],
        ),
    ];
    let mut mismatches = Vec::new();
    for (label, toml, args) in &rows {
        let dir = TempDir::new().unwrap();
        fixtures(dir.path());
        let r = apply_in(dir.path(), toml, args);
        if r.code != Some(4) || r.stderr != format!("error: {line}\n") {
            mismatches.push(format!("{label}: exit {:?}, stderr {:?}", r.code, r.stderr));
        }
        if dir.path().join("out").exists() {
            mismatches.push(format!("{label}: the out dir was created"));
        }
    }
    assert_no_mismatches(rows.len(), 3, mismatches);
}

#[test]
fn batch_keeping_formats_fails_that_input() {
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let r = apply_in(
        dir.path(),
        &pad_recipe("transparent"),
        &["src.jpg", "rgb.png", "--out-dir", "out"],
    );
    let line = jpeg_line(0, "transparent", "00");
    assert_eq!(r.code, Some(6), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: src.jpg: {line}\nerror: 1 of 2 inputs failed\n")
    );
    assert!(
        dir.path().join("out/rgb.png").exists(),
        "the PNG is written"
    );
    assert!(!dir.path().join("out/src.jpg").exists(), "the JPEG is not");
}

/// A recipe that pads, then ends in the terminal `optimize` step.
fn pad_then_optimize(pad_body: &str) -> String {
    recipe(&[&pad_step(pad_body), "op = \"optimize\""])
}

#[test]
fn terminal_optimize_pinned_refuses() {
    let toml = pad_then_optimize("border = 0.25\ncolor = \"transparent\"");
    let line = jpeg_line(0, "transparent", "00");

    let one = TempDir::new().unwrap();
    fixtures(one.path());
    let r = apply_in(one.path(), &toml, &["rgb.png", "-o", "p.jpg"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(r.stderr, format!("error: {line}\n"));
    assert!(!one.path().join("p.jpg").exists(), "nothing is written");

    let two = TempDir::new().unwrap();
    fixtures(two.path());
    let r = apply_in(
        two.path(),
        &toml,
        &["src.jpg", "rgb.png", "--format", "jpeg", "--out-dir", "tb"],
    );
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(r.stderr, format!("error: {line}\n"));
    assert!(
        !two.path().join("tb").exists(),
        "the out dir is not created"
    );
}

/// The one file in `dir`, or a panic naming what is there.
fn only_file(dir: &Path) -> PathBuf {
    let names: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        names.len(),
        1,
        "expected one output in {}: {names:?}",
        dir.display()
    );
    names[0].clone()
}

/// The decided output keeps the alpha the pad brought: not a JPEG, a colour
/// type with alpha, and the corner pixel fully transparent.
fn assert_alpha_output(path: &Path) {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    assert_ne!(ext, "jpg", "{} must not be a JPEG", path.display());
    let img = image::open(path).expect("output decodes");
    assert!(img.color().has_alpha(), "{} has no alpha", path.display());
    assert_eq!(
        img.to_rgba8().get_pixel(0, 0).0[3],
        0,
        "corner is transparent"
    );
}

#[test]
fn auto_decided_format_keeps_the_alpha() {
    let padded = pad_then_optimize("border = 0.25\ncolor = \"transparent\"");
    let at_aspect = pad_then_optimize("aspect = \"4:3\"\ncolor = \"transparent\"");

    // apply: the format is chosen after the pipeline.
    let dir = TempDir::new().unwrap();
    write_as(&dir.path().join("photo.jpg"), &photo(), ImageFormat::Jpeg);
    let r = apply_in(dir.path(), &padded, &["photo.jpg", "--out-dir", "ao"]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    assert_alpha_output(&only_file(&dir.path().join("ao")));

    // An image already at the aspect ships unchanged: no fill pixel, no alpha.
    let r = apply_in(dir.path(), &at_aspect, &["photo.jpg", "--out-dir", "ap"]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let kept = only_file(&dir.path().join("ap"));
    assert_eq!(kept.extension().and_then(|e| e.to_str()), Some("jpg"));
    assert_eq!(
        std::fs::read(&kept).unwrap(),
        std::fs::read(dir.path().join("photo.jpg")).unwrap()
    );
    assert!(
        format!("{}{}", r.stdout, r.stderr).contains("kept jpeg"),
        "stdout {:?} stderr {:?}",
        r.stdout,
        r.stderr
    );

    // build: a fresh project each time, because a stale cache replays old bytes.
    for (toml, label) in [(&padded, "padded"), (&at_aspect, "at-aspect")] {
        let dir = TempDir::new().unwrap();
        write_as(
            &dir.path().join("in/photo.jpg"),
            &photo(),
            ImageFormat::Jpeg,
        );
        std::fs::write(dir.path().join("t.toml"), toml).unwrap();
        std::fs::write(
            dir.path().join("crustyimg.build.toml"),
            "version = 1\n\n[[target]]\nsource = \"in/*.jpg\"\nrecipe = \"t.toml\"\nout = \"bd\"\n",
        )
        .unwrap();
        let r = bin(dir.path(), &["build"]);
        assert_eq!(r.code, Some(0), "{label}: stderr: {}", r.stderr);
        let out = only_file(&dir.path().join("bd"));
        if label == "padded" {
            assert_alpha_output(&out);
        } else {
            assert_eq!(
                std::fs::read(&out).unwrap(),
                std::fs::read(dir.path().join("in/photo.jpg")).unwrap(),
                "an at-aspect image is shipped unchanged"
            );
        }
    }
}

#[test]
fn build_refuses_pinned_once_and_kept_formats_per_input() {
    let line = jpeg_line(0, "transparent", "00");
    let project = |target: &str| {
        let dir = TempDir::new().unwrap();
        fixtures(dir.path());
        write_as(&dir.path().join("in/src.jpg"), &rgb4(), ImageFormat::Jpeg);
        write_as(&dir.path().join("in/rgb.png"), &rgb4(), ImageFormat::Png);
        std::fs::write(dir.path().join("t.toml"), pad_recipe("transparent")).unwrap();
        std::fs::write(
            dir.path().join("crustyimg.build.toml"),
            format!(
                "version = 1\n\n[[target]]\nsource = \"in/*\"\nrecipe = \"t.toml\"\n{target}\n"
            ),
        )
        .unwrap();
        dir
    };

    // Pinned by the name's extension: refused once, before any input.
    let dir = project("out = \"b1\"\nname = \"{stem}.jpg\"");
    let r = bin(dir.path(), &["build"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: target #0 (recipe t.toml, out b1)\nerror: {line}\n")
    );
    assert!(!dir.path().join("b1").exists(), "nothing is created");

    // Each input keeps its format: the JPEG fails, the PNG is written.
    let dir = project("out = \"b2\"");
    let r = bin(dir.path(), &["build"]);
    assert_eq!(r.code, Some(6), "stderr: {}", r.stderr);
    assert!(
        r.stderr
            .starts_with(&format!("error: in/src.jpg: {line}\n")),
        "stderr: {}",
        r.stderr
    );
    assert!(dir.path().join("b2/rgb.png").exists(), "the PNG is written");
    assert!(!dir.path().join("b2/src.jpg").exists(), "the JPEG is not");
}

#[test]
fn static_rule_at_aspect_and_two_pads() {
    // An image already at the aspect writes no fill pixel, and is refused anyway.
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let at = recipe(&[&pad_step("aspect = \"1:1\"\ncolor = \"transparent\"")]);
    let r = apply_in(dir.path(), &at, &["rgb.png", "-o", "a.jpg"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: {}\n", jpeg_line(0, "transparent", "00"))
    );
    assert!(!dir.path().join("a.jpg").exists());
    let r = apply_in(dir.path(), &at, &["rgb.png", "-o", "a.png"]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    let out = image::open(dir.path().join("a.png")).unwrap();
    assert_eq!(
        kind_of(&out),
        "rgb8",
        "no fill pixel, so the type is unchanged"
    );

    // Two pads: the first step the format cannot store is the one named.
    let pad = |c: &str| pad_step(&format!("border = 0.25\ncolor = \"{c}\""));
    let opaque_then_clear = recipe(&[&pad("#ffffff"), &pad("transparent")]);
    let r = apply_in(dir.path(), &opaque_then_clear, &["rgb.png", "-o", "s.jpg"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: {}\n", jpeg_line(1, "transparent", "00"))
    );

    let clear_then_half = recipe(&[&pad("#ff000000"), &pad("#ff000080")]);
    let r = apply_in(dir.path(), &clear_then_half, &["rgb.png", "-o", "s.gif"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: {}\n", gif_line(1, "#ff000080", "80"))
    );
    let r = apply_in(dir.path(), &clear_then_half, &["rgb.png", "-o", "s2.jpg"]);
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!("error: {}\n", jpeg_line(0, "#ff000000", "00"))
    );
}

#[test]
fn format_is_resolved_before_the_pipeline() {
    // A border that rounds to 0 px AND a format that is wrong: the format is
    // reported first, with its own exit code.
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let zero = recipe(&[&pad_step("border = 0.001\ncolor = \"#ffffff\"")]);
    let r = apply_in(
        dir.path(),
        &zero,
        &["--format", "bogus", "-o", "z.png", "rgb.png"],
    );
    assert_eq!(r.code, Some(4), "stderr: {}", r.stderr);
    assert_eq!(r.stderr, "error: unsupported output extension: bogus\n");
    assert!(!dir.path().join("z.png").exists());
}

#[test]
fn source_alpha_and_opaque_pad_into_jpeg_unchanged() {
    // Alpha the SOURCE carries is dropped by the JPEG encoder, silently, as
    // before: only alpha a pad brings is refused.
    let dir = TempDir::new().unwrap();
    fixtures(dir.path());
    let flip = recipe(&["op = \"flip\"\naxis = \"horizontal\""]);
    let r = apply_in(dir.path(), &flip, &["rgba.png", "-o", "f.jpg"]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    assert_eq!(r.stderr, "");
    assert!(dir.path().join("f.jpg").exists());

    let r = apply_in(
        dir.path(),
        &pad_recipe("#ffffff"),
        &["rgb.png", "-o", "o.jpg"],
    );
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    assert_eq!(r.stderr, "");
    assert!(dir.path().join("o.jpg").exists());
}
