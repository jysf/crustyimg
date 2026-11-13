//! `edit --pad` and `--trim`: flag forms, refusals, typed order, the alpha rule
//! for a translucent fill, `--save-recipe` replays, and a hex colour with a sign.
//!
//! Every test drives the compiled binary. Fixtures are seeded noise written here,
//! so each expected size, colour type and message is a literal worked out by
//! hand, never read back from the code under test. A replay row runs
//! `edit … --save-recipe`, then `apply` of that recipe on the same input, and
//! compares the two decoded images and the saved step list.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use image::{ColorType, DynamicImage, GrayImage, Luma, LumaA, Rgb, RgbImage, Rgba};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// One xorshift64 step.
fn step(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn is_wide(kind: &str) -> bool {
    kind.ends_with("16")
}

/// One pixel's four levels from one xorshift value. The alpha level is ORed with
/// 128 (or 0x8000) so no pixel is fully transparent.
fn levels(kind: &str, v: u64) -> [u16; 4] {
    if is_wide(kind) {
        [
            v as u16,
            (v >> 16) as u16,
            (v >> 32) as u16,
            ((v >> 48) as u16) | 0x8000,
        ]
    } else {
        [
            v as u8 as u16,
            (v >> 8) as u8 as u16,
            (v >> 16) as u8 as u16,
            (((v >> 24) as u8) | 128) as u16,
        ]
    }
}

/// The opaque #101010 frame colour as levels: 16, or 16 × 257 at 16 bits.
fn frame_levels(kind: &str) -> [u16; 4] {
    if is_wide(kind) {
        [4112, 4112, 4112, 65535]
    } else {
        [16, 16, 16, 255]
    }
}

/// An image of colour type `kind` ("l8", "la8", "rgb8", "rgba8", "l16",
/// "rgba16"), with each pixel's levels from `px`. Written per colour type:
/// `DynamicImage::put_pixel` would round-trip a 16-bit pixel through RGBA8.
fn build(kind: &str, w: u32, h: u32, px: impl Fn(u32, u32) -> [u16; 4]) -> DynamicImage {
    match kind {
        "l8" => {
            DynamicImage::ImageLuma8(GrayImage::from_fn(w, h, |x, y| Luma([px(x, y)[0] as u8])))
        }
        "la8" => DynamicImage::ImageLumaA8(image::ImageBuffer::from_fn(w, h, |x, y| {
            let p = px(x, y);
            LumaA([p[0] as u8, p[3] as u8])
        })),
        "rgb8" => DynamicImage::ImageRgb8(RgbImage::from_fn(w, h, |x, y| {
            let p = px(x, y);
            Rgb([p[0] as u8, p[1] as u8, p[2] as u8])
        })),
        "rgba8" => DynamicImage::ImageRgba8(image::RgbaImage::from_fn(w, h, |x, y| {
            let p = px(x, y);
            Rgba([p[0] as u8, p[1] as u8, p[2] as u8, p[3] as u8])
        })),
        "l16" => DynamicImage::ImageLuma16(image::ImageBuffer::from_fn(w, h, |x, y| {
            Luma([px(x, y)[0]])
        })),
        "rgba16" => {
            DynamicImage::ImageRgba16(image::ImageBuffer::from_fn(w, h, |x, y| Rgba(px(x, y))))
        }
        other => panic!("unknown fixture kind {other}"),
    }
}

/// Seeded noise: one xorshift value per pixel, in row order.
fn noise(kind: &str, w: u32, h: u32, seed: u64) -> DynamicImage {
    let mut s = seed;
    let all: Vec<[u16; 4]> = (0..w * h).map(|_| levels(kind, step(&mut s))).collect();
    build(kind, w, h, |x, y| all[(y * w + x) as usize])
}

/// 60×40: 52×32 noise at (3, 3) in an opaque #101010 frame (3 px top and left,
/// 5 px bottom and right), so `--trim` has a border to take and the trimmed size
/// is the literal 52×32.
fn framed(kind: &str) -> DynamicImage {
    let mut s = 77u64;
    let inner: Vec<[u16; 4]> = (0..52 * 32).map(|_| levels(kind, step(&mut s))).collect();
    let frame = frame_levels(kind);
    build(kind, 60, 40, |x, y| {
        if (3..55).contains(&x) && (3..35).contains(&y) {
            inner[((y - 3) * 52 + (x - 3)) as usize]
        } else {
            frame
        }
    })
}

/// A 4×4 RGB image of one colour, (200, 100, 50).
fn flat() -> DynamicImage {
    DynamicImage::ImageRgb8(RgbImage::from_pixel(4, 4, Rgb([200, 100, 50])))
}

fn write_img(dir: &Path, name: &str, img: &DynamicImage) -> PathBuf {
    let p = dir.join(name);
    img.save(&p).unwrap();
    p
}

// ── Running the binary ────────────────────────────────────────────────────────

struct Ran {
    code: Option<i32>,
    stderr: String,
}

fn run(args: &[String]) -> Ran {
    let o = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run crustyimg");
    Ran {
        code: o.status.code(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn strs(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_owned()).collect()
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap()
}

/// `crustyimg edit ARGS`, as given.
fn ed(args: &[&str]) -> Ran {
    let mut a = vec!["edit".to_owned()];
    a.extend(strs(args));
    run(&a)
}

/// `edit` with the extra args (any `{IN}` is replaced by the input path; if there
/// is none, the input follows `edit`), writing `-o e.png -y` and, when `recipe`
/// is true, `--save-recipe r.toml` in `dir`.
fn edit(dir: &Path, input: &Path, args: &[&str], recipe: bool) -> Ran {
    let mut a = vec!["edit".to_owned()];
    let mut placed = false;
    for arg in args {
        if *arg == "{IN}" {
            a.push(s(input).to_owned());
            placed = true;
        } else {
            a.push((*arg).to_owned());
        }
    }
    if !placed {
        a.insert(1, s(input).to_owned());
    }
    a.extend(strs(&["-o", s(&dir.join("e.png")), "-y"]));
    if recipe {
        a.extend(strs(&["--save-recipe", s(&dir.join("r.toml"))]));
    }
    run(&a)
}

/// The saved recipe's `[[step]]` tables, in order.
fn saved_steps(dir: &Path) -> Vec<toml::Table> {
    let text = std::fs::read_to_string(dir.join("r.toml")).expect("recipe was written");
    let doc: toml::Table = text.parse().expect("recipe parses");
    doc["step"]
        .as_array()
        .expect("step array")
        .iter()
        .map(|v| v.as_table().unwrap().clone())
        .collect()
}

fn ops_of(steps: &[toml::Table]) -> Vec<String> {
    steps
        .iter()
        .map(|t| t["op"].as_str().unwrap().to_owned())
        .collect()
}

/// Run `edit IN ARGS --save-recipe`, then `apply` the saved recipe on the same
/// input. Asserts both exit 0, the two outputs agree (colour type, dimensions,
/// samples), the output has `dims`, and the saved ops equal `ops` in order.
/// Returns the edit's output and the saved steps.
fn replay(
    input: &Path,
    args: &[&str],
    dims: (u32, u32),
    ops: &[&str],
) -> (DynamicImage, Vec<toml::Table>) {
    let dir = TempDir::new().unwrap();
    let e = edit(dir.path(), input, args, true);
    assert_eq!(e.code, Some(0), "edit {args:?}: {}", e.stderr);
    let steps = saved_steps(dir.path());
    assert_eq!(ops_of(&steps), ops, "saved steps for {args:?}");

    let a = run(&strs(&[
        "apply",
        "--recipe",
        s(&dir.path().join("r.toml")),
        s(input),
        "-o",
        s(&dir.path().join("a.png")),
        "-y",
    ]));
    assert_eq!(
        a.code,
        Some(0),
        "apply of the recipe saved by {args:?}: {}",
        a.stderr
    );

    let ed = image::open(dir.path().join("e.png")).unwrap();
    let ap = image::open(dir.path().join("a.png")).unwrap();
    assert_eq!(ed.color(), ap.color(), "colour type, {args:?}");
    assert_eq!(
        (ed.width(), ed.height()),
        (ap.width(), ap.height()),
        "replay dimensions, {args:?}"
    );
    assert_eq!((ed.width(), ed.height()), dims, "edit dimensions, {args:?}");
    assert!(
        ed.as_bytes() == ap.as_bytes(),
        "pixels differ between edit and replay {args:?}"
    );
    (ed, steps)
}

// ── FT-1: replay, six colour types × five shapes ─────────────────────────────

#[test]
fn replay_pad_and_trim_on_six_colour_types() {
    // (kind, colour type after an opaque chromatic pad, after a transparent pad,
    // after the aspect pad with alpha 80). Grey widens to RGB for a chromatic
    // fill and gains alpha for a translucent one (DEC-095).
    let kinds = [
        ("l8", ColorType::Rgb8, ColorType::La8, ColorType::Rgba8),
        ("la8", ColorType::Rgba8, ColorType::La8, ColorType::Rgba8),
        ("rgb8", ColorType::Rgb8, ColorType::Rgba8, ColorType::Rgba8),
        (
            "rgba8",
            ColorType::Rgba8,
            ColorType::Rgba8,
            ColorType::Rgba8,
        ),
        ("l16", ColorType::Rgb16, ColorType::La16, ColorType::Rgba16),
        (
            "rgba16",
            ColorType::Rgba16,
            ColorType::Rgba16,
            ColorType::Rgba16,
        ),
    ];
    for (kind, opaque_type, clear_type, aspect_type) in kinds {
        let dir = TempDir::new().unwrap();
        let src = framed(kind);
        let input = write_img(dir.path(), "in.png", &src);
        let src16 = src.to_rgba16();

        // --pad 3, opaque chromatic: 66×46, border 3/60 = 0.05.
        let (out, steps) = replay(
            &input,
            &["--pad", "3", "--pad-color", "336699"],
            (66, 46),
            &["auto-orient", "pad"],
        );
        assert_eq!(out.color(), opaque_type, "{kind}: opaque pad type");
        assert_eq!(steps[1]["border"].as_float(), Some(0.05), "{kind}");
        let o16 = out.to_rgba16();
        assert_eq!(
            o16.get_pixel(0, 0).0,
            [0x3333, 0x6666, 0x9999, 0xffff],
            "{kind}: fill"
        );
        assert_eq!(
            o16.get_pixel(6, 6),
            src16.get_pixel(3, 3),
            "{kind}: the image sits 3 px in"
        );

        // --pad 3, transparent: 66×46.
        let (out, _) = replay(
            &input,
            &["--pad", "3", "--pad-color", "transparent"],
            (66, 46),
            &["auto-orient", "pad"],
        );
        assert_eq!(out.color(), clear_type, "{kind}: transparent pad type");
        assert_eq!(out.to_rgba16().get_pixel(0, 0).0, [0, 0, 0, 0], "{kind}");

        // The aspect form, southwest, alpha 80: 60×60, the image at the bottom.
        let (out, steps) = replay(
            &input,
            &[
                "--pad",
                "1:1",
                "--pad-gravity",
                "southwest",
                "--pad-color",
                "ff000080",
            ],
            (60, 60),
            &["auto-orient", "pad"],
        );
        assert_eq!(out.color(), aspect_type, "{kind}: aspect pad type");
        assert_eq!(steps[1]["gravity"].as_str(), Some("southwest"), "{kind}");
        assert_eq!(steps[1]["aspect"].as_str(), Some("1:1"), "{kind}");
        let o16 = out.to_rgba16();
        assert_eq!(
            o16.get_pixel(0, 0).0,
            [0xffff, 0, 0, 0x8080],
            "{kind}: fill"
        );
        assert_eq!(
            o16.get_pixel(5, 25),
            src16.get_pixel(5, 5),
            "{kind}: the image sits at the bottom"
        );

        // --trim: the top-left colour, type kept, no keys saved.
        let (out, steps) = replay(&input, &["--trim"], (52, 32), &["auto-orient", "trim"]);
        assert_eq!(out.color(), src.color(), "{kind}: trim keeps the type");
        assert!(
            !steps[1].contains_key("color") && !steps[1].contains_key("tolerance"),
            "{kind}: plain --trim saves no keys: {:?}",
            steps[1]
        );
        assert_eq!(
            out.to_rgba16().get_pixel(0, 0),
            src16.get_pixel(3, 3),
            "{kind}: trim starts at the noise"
        );

        // --trim-color one step off the frame, with tolerance 1 (×257 at 16 bits).
        let (out, steps) = replay(
            &input,
            &["--trim", "--trim-color", "111111", "--trim-tolerance", "1"],
            (52, 32),
            &["auto-orient", "trim"],
        );
        assert_eq!(out.color(), src.color(), "{kind}");
        assert_eq!(steps[1]["tolerance"].as_integer(), Some(1), "{kind}");
        assert_eq!(steps[1]["color"].as_str(), Some("111111"), "{kind}");

        // The control: the same colour without the tolerance trims nothing.
        let (out, _) = replay(
            &input,
            &["--trim", "--trim-color", "111111"],
            (60, 40),
            &["auto-orient", "trim"],
        );
        assert_eq!(out.to_rgba16(), src16, "{kind}: nothing trimmed");
    }
}

// ── FT-2: the saved text ──────────────────────────────────────────────────────

#[test]
fn saved_recipe_text_is_exact() {
    let dir = TempDir::new().unwrap();
    let input = write_img(dir.path(), "in.png", &noise("rgb8", 60, 40, 11));

    let w = TempDir::new().unwrap();
    let r = edit(
        w.path(),
        &input,
        &["--pad", "7", "--pad-color", "AbCdEf"],
        true,
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    let text = std::fs::read_to_string(w.path().join("r.toml")).unwrap();
    let steps = saved_steps(w.path());
    let mut keys: Vec<&str> = steps[1].keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["border", "color", "op"]);
    assert!(
        text.contains("\nborder = 0.11666666666666667\n"),
        "7/60 printed in full:\n{text}"
    );
    assert!(text.contains("\ncolor = \"AbCdEf\"\n"), "{text}");
    assert!(!text.contains("gravity"), "{text}");

    let w = TempDir::new().unwrap();
    let r = edit(
        w.path(),
        &input,
        &["--pad", "4:5", "--pad-color", "ff0000"],
        true,
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    let text = std::fs::read_to_string(w.path().join("r.toml")).unwrap();
    assert!(text.contains("\naspect = \"4:5\"\n"), "{text}");
    assert!(
        !text.contains("gravity") && !text.contains("border"),
        "{text}"
    );
}

// ── FT-3: typed order ─────────────────────────────────────────────────────────

#[test]
fn pad_and_trim_follow_typed_order() {
    let dir = TempDir::new().unwrap();
    let input = write_img(dir.path(), "in.png", &framed("rgb8"));
    // (args A then B, ops, dims, args B then A, ops, dims)
    #[allow(clippy::type_complexity)]
    let pairs: [(
        &[&str],
        [&str; 3],
        (u32, u32),
        &[&str],
        [&str; 3],
        (u32, u32),
    ); 3] = [
        (
            &["--pad", "4:5", "--pad-color", "ff0000", "--rotate", "90"],
            ["auto-orient", "pad", "rotate"],
            (75, 60),
            &["--rotate", "90", "--pad", "4:5", "--pad-color", "ff0000"],
            ["auto-orient", "rotate", "pad"],
            (48, 60),
        ),
        (
            &["--pad", "2", "--pad-color", "101010", "--trim"],
            ["auto-orient", "pad", "trim"],
            (52, 32),
            &["--trim", "--pad", "2", "--pad-color", "101010"],
            ["auto-orient", "trim", "pad"],
            (56, 36),
        ),
        (
            &["--pad", "3", "--pad-color", "ff0000", "--resize-max", "30"],
            ["auto-orient", "pad", "resize"],
            (30, 21),
            &["--resize-max", "30", "--pad", "3", "--pad-color", "ff0000"],
            ["auto-orient", "resize", "pad"],
            (36, 26),
        ),
    ];
    for (a, ops_a, dims_a, b, ops_b, dims_b) in pairs {
        let (img_a, _) = replay(&input, a, dims_a, &ops_a);
        let (img_b, steps_b) = replay(&input, b, dims_b, &ops_b);
        assert!(
            img_a.as_bytes() != img_b.as_bytes(),
            "{a:?} and {b:?} must differ: the order is the point"
        );
        if b[0] == "--resize-max" {
            // 3 px of the 30 px image the step receives, not of the 60 px original.
            assert_eq!(steps_b[2]["border"].as_float(), Some(0.1));
        }
    }
}

// ── FT-4: a switch before the input path keeps its position ──────────────────

#[test]
fn typed_order_with_trim_before_the_input() {
    let dir = TempDir::new().unwrap();
    let input = write_img(dir.path(), "in.png", &framed("rgb8"));
    // trim 60×40 → 52×32, rotate → 32×52, pad 1 → 34×54.
    replay(
        &input,
        &[
            "--trim",
            "--rotate=90",
            "{IN}",
            "--pad=1",
            "--pad-color",
            "000000",
        ],
        (34, 54),
        &["auto-orient", "trim", "rotate", "pad"],
    );
}

// ── The alpha rule ────────────────────────────────────────────────────────────

const JPEG_TAIL: &str =
    "and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque";
const GIF_TAIL: &str =
    "and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead";

fn jpeg_line(written: &str, alpha: &str) -> String {
    format!("error: --pad-color {written:?} has alpha {alpha}, {JPEG_TAIL}\n")
}

fn gif_line(written: &str, alpha: &str) -> String {
    format!("error: --pad-color {written:?} has alpha {alpha}, {GIF_TAIL}\n")
}

// ── FT-5: one input ───────────────────────────────────────────────────────────

#[test]
fn translucent_pad_into_jpeg_or_gif_one_input() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let png = write_img(d, "in.png", &flat());
    let jpg = write_img(d, "in.jpg", &flat());

    // Both pad forms, after a step that makes the step index differ from 0.
    let rows: [(&[&str], &str, String); 4] = [
        (
            &["--rotate", "90", "--pad", "2", "--pad-color", "transparent"],
            "o.jpg",
            jpeg_line("transparent", "00"),
        ),
        (
            &[
                "--rotate",
                "90",
                "--pad",
                "4:5",
                "--pad-color",
                "transparent",
            ],
            "o.jpg",
            jpeg_line("transparent", "00"),
        ),
        (
            &["--rotate", "90", "--pad", "2", "--pad-color", "#ff000080"],
            "o.gif",
            gif_line("#ff000080", "80"),
        ),
        (
            &["--rotate", "90", "--pad", "4:5", "--pad-color", "#ff000080"],
            "o.gif",
            gif_line("#ff000080", "80"),
        ),
    ];
    for (args, out, want) in rows {
        let o = d.join(out);
        let mut a = vec![s(&png)];
        a.extend(args);
        a.extend(["-o", s(&o), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(4), "{args:?} -> {out}: {}", r.stderr);
        assert_eq!(r.stderr, want, "{args:?} -> {out}");
        assert!(!o.exists(), "{args:?} -> {out}: output written");
    }

    // A kept JPEG source into --out-dir: refused, and no directory made.
    let od = d.join("od");
    let r = ed(&[
        s(&jpg),
        "--pad",
        "2",
        "--pad-color",
        "transparent",
        "--out-dir",
        s(&od),
        "-y",
    ]);
    assert_eq!(r.code, Some(4), "{}", r.stderr);
    assert_eq!(r.stderr, jpeg_line("transparent", "00"));
    assert!(!od.exists(), "out-dir made");

    // Controls: opaque into JPEG, transparent into GIF (stores 00), alpha 80 into PNG.
    let o = d.join("c1.jpg");
    let r = ed(&[
        s(&png),
        "--pad",
        "2",
        "--pad-color",
        "336699",
        "-o",
        s(&o),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(o.exists());
    let o = d.join("c2.gif");
    let r = ed(&[
        s(&png),
        "--pad",
        "2",
        "--pad-color",
        "transparent",
        "-o",
        s(&o),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(o.exists());
    let o = d.join("c3.png");
    let r = ed(&[
        s(&png),
        "--pad",
        "2",
        "--pad-color",
        "#ff000080",
        "-o",
        s(&o),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    let got = image::open(&o).unwrap().to_rgba8();
    assert_eq!(got.get_pixel(0, 0).0, [255, 0, 0, 128]);
}

// ── FT-6: a format the flags fix for the whole batch ─────────────────────────

/// A directory `in/` holding `a.jpg` (4×4) and `b.png` (6×4).
fn jpg_png_batch() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let d = dir.path().join("in");
    std::fs::create_dir(&d).unwrap();
    write_img(&d, "a.jpg", &flat());
    write_img(
        &d,
        "b.png",
        &DynamicImage::ImageRgb8(RgbImage::from_pixel(6, 4, Rgb([9, 99, 199]))),
    );
    (dir, d)
}

#[test]
fn translucent_pad_batch_wide_format_refused_once() {
    let (dir, d) = jpg_png_batch();
    let od = dir.path().join("od");
    let rows: [(&[&str], String); 3] = [
        (
            &[
                "--pad",
                "1",
                "--pad-color",
                "transparent",
                "--format",
                "jpeg",
            ],
            jpeg_line("transparent", "00"),
        ),
        (
            &[
                "--pad",
                "1",
                "--pad-color",
                "transparent",
                "--name-template",
                "{stem}.jpg",
            ],
            jpeg_line("transparent", "00"),
        ),
        (
            &[
                "--pad",
                "1:1",
                "--pad-color",
                "#ff000080",
                "--name-template",
                "{stem}.gif",
            ],
            gif_line("#ff000080", "80"),
        ),
    ];
    for (args, want) in rows {
        let mut a = vec![s(&d)];
        a.extend(args);
        a.extend(["--out-dir", s(&od), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(4), "{args:?}: {}", r.stderr);
        assert_eq!(r.stderr, want, "{args:?}");
        assert!(!od.exists(), "{args:?}: out-dir made");
    }
    // transparent into GIF is stored.
    let r = ed(&[
        s(&d),
        "--pad",
        "1",
        "--pad-color",
        "transparent",
        "--name-template",
        "{stem}.gif",
        "--out-dir",
        s(&od),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(od.join("a.gif").exists() && od.join("b.gif").exists());
}

// ── FT-7: a batch that keeps each input's format ─────────────────────────────

#[test]
fn translucent_pad_kept_format_batch_fails_that_input() {
    let (dir, d) = jpg_png_batch();
    let od = dir.path().join("od");
    let r = ed(&[
        s(&d),
        "--pad",
        "1",
        "--pad-color",
        "transparent",
        "--out-dir",
        s(&od),
        "-y",
    ]);
    assert_eq!(r.code, Some(6), "{}", r.stderr);
    let lines: Vec<&str> = r.stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{}", r.stderr);
    assert!(lines[0].starts_with("error: "), "{}", lines[0]);
    // The resolver prints a canonical path, so match the tail.
    assert!(
        lines[0].ends_with(&format!(
            "/in/a.jpg: --pad-color \"transparent\" has alpha 00, {JPEG_TAIL}"
        )),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "error: 1 of 2 inputs failed");
    assert!(od.join("b.png").exists(), "the PNG is written");
    assert!(!od.join("a.jpg").exists(), "the JPEG is not");
}

// ── FT-8: exit codes that must not move ──────────────────────────────────────

#[test]
fn exit_codes_unchanged_without_added_alpha() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let input = write_img(d, "in.png", &flat());
    let o = d.join("o.png");
    let rows: [(Vec<&str>, i32); 4] = [
        // The crop fails first, so --format bogus is never reached.
        (
            vec![
                "edit",
                s(&input),
                "--crop",
                "10x10+0+0",
                "--format",
                "bogus",
                "-o",
                s(&o),
            ],
            1,
        ),
        (
            vec![
                "resize",
                s(&input),
                "--exact",
                "60000x10",
                "--format",
                "bogus",
                "-o",
                s(&o),
            ],
            1,
        ),
        // An opaque pad adds no alpha, so it takes the old path and fails at apply.
        (
            vec![
                "edit",
                s(&input),
                "--pad",
                "9",
                "--pad-color",
                "ff0000",
                "--format",
                "bogus",
                "-o",
                s(&o),
            ],
            1,
        ),
        (
            vec![
                "watermark",
                s(&input),
                "--text",
                "hi",
                "--size",
                "500",
                "--format",
                "bogus",
                "-o",
                s(&o),
            ],
            2,
        ),
    ];
    for (args, code) in rows {
        let r = run(&strs(&args));
        assert_eq!(r.code, Some(code), "{args:?}: {}", r.stderr);
        assert!(!o.exists(), "{args:?}: output written");
    }
}

// ── FT-9: --pad N failures speak pixels ──────────────────────────────────────

#[test]
fn pad_n_failures_speak_pixels() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let small = write_img(d, "small.png", &flat());
    let tall = write_img(
        d,
        "tall.png",
        &DynamicImage::ImageLuma8(GrayImage::from_pixel(2, 49990, Luma([7]))),
    );
    let long = write_img(
        d,
        "long.png",
        &DynamicImage::ImageLuma8(GrayImage::from_pixel(2, 11000, Luma([7]))),
    );
    let o = d.join("o.png");
    let rows: [(&Path, &[&str], &str); 4] = [
        (
            &small,
            &["--pad", "5", "--pad-color", "ff0000"],
            "error: operation 'pad' failed: --pad 5 is wider than the 4 px long edge of the 4×4 px image it receives; a border can be at most as wide as the long edge\n",
        ),
        (
            &tall,
            &["--pad", "6", "--pad-color", "ff0000"],
            "error: operation 'pad' failed: --pad 6 on this 2×49990 px image needs a 14×50002 px canvas, which exceeds the maximum edge limit of 50000 px\n",
        ),
        (
            &long,
            &["--pad", "5000", "--pad-color", "ff0000"],
            "error: operation 'pad' failed: --pad 5000 on this 2×11000 px image needs a 10002×21000 px canvas = 210042000 px, which exceeds the maximum area of 134217728 px\n",
        ),
        (
            &small,
            &["--pad", "65535:1", "--pad-color", "ff0000"],
            "error: operation 'pad' failed: --pad \"65535:1\" on this 4×4 px image needs a 262140×4 px canvas, which exceeds the maximum edge limit of 50000 px\n",
        ),
    ];
    for (input, args, want) in rows {
        let mut a = vec![s(input)];
        a.extend(args);
        a.extend(["-o", s(&o), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(1), "{args:?}: {}", r.stderr);
        assert_eq!(r.stderr, want, "{args:?}");
        assert!(!o.exists(), "{args:?}: output written");
    }

    // N equal to the long edge is a border of 1: allowed.
    let r = ed(&[
        s(&small),
        "--pad",
        "4",
        "--pad-color",
        "ff0000",
        "-o",
        s(&o),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    let got = image::open(&o).unwrap();
    assert_eq!((got.width(), got.height()), (12, 12));

    // The same --pad 5 in a batch with a 6×4 image: the 4×4 fails (6), the 6×4 is written.
    let b = d.join("batch");
    std::fs::create_dir(&b).unwrap();
    write_img(&b, "a.png", &flat());
    write_img(
        &b,
        "b.png",
        &DynamicImage::ImageRgb8(RgbImage::from_pixel(6, 4, Rgb([1, 2, 3]))),
    );
    let od = d.join("od");
    let r = ed(&[
        s(&b),
        "--pad",
        "5",
        "--pad-color",
        "ff0000",
        "--out-dir",
        s(&od),
        "-y",
    ]);
    assert_eq!(r.code, Some(6), "{}", r.stderr);
    let lines: Vec<&str> = r.stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{}", r.stderr);
    assert!(
        lines[0].ends_with("/a.png: operation 'pad' failed: --pad 5 is wider than the 4 px long edge of the 4×4 px image it receives; a border can be at most as wide as the long edge"),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "error: 1 of 2 inputs failed");
    assert!(!od.join("a.png").exists());
    let got = image::open(od.join("b.png")).unwrap();
    assert_eq!((got.width(), got.height()), (16, 14));
}

// ── FT-10: --trim on a whole match ───────────────────────────────────────────

#[test]
fn trim_whole_match_names_the_flags() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let input = write_img(d, "in.png", &flat());
    let o = d.join("o.png");

    let r = ed(&[s(&input), "--trim", "-o", s(&o), "-y"]);
    assert_eq!(r.code, Some(1), "{}", r.stderr);
    assert_eq!(
        r.stderr,
        "error: operation 'trim' failed: every pixel of this 4×4 px image matches its top-left pixel within --trim-tolerance 0, so a trim would leave nothing\n"
    );
    let r = ed(&[
        s(&input),
        "--trim",
        "--trim-color",
        "#C86432",
        "--trim-tolerance",
        "8",
        "-o",
        s(&o),
        "-y",
    ]);
    assert_eq!(r.code, Some(1), "{}", r.stderr);
    assert_eq!(
        r.stderr,
        "error: operation 'trim' failed: every pixel of this 4×4 px image matches --trim-color \"#C86432\" within --trim-tolerance 8, so a trim would leave nothing\n"
    );
    assert!(!o.exists());

    // A batch: the uniform image fails, the framed one is written.
    let b = d.join("batch");
    std::fs::create_dir(&b).unwrap();
    write_img(&b, "flat.png", &flat());
    write_img(&b, "framed.png", &framed("rgb8"));
    let od = d.join("od");
    let r = ed(&[s(&b), "--trim", "--out-dir", s(&od), "-y"]);
    assert_eq!(r.code, Some(6), "{}", r.stderr);
    let lines: Vec<&str> = r.stderr.lines().collect();
    assert_eq!(lines.len(), 2, "{}", r.stderr);
    assert!(
        lines[0].ends_with("/flat.png: operation 'trim' failed: every pixel of this 4×4 px image matches its top-left pixel within --trim-tolerance 0, so a trim would leave nothing"),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "error: 1 of 2 inputs failed");
    assert!(!od.join("flat.png").exists());
    let got = image::open(od.join("framed.png")).unwrap();
    assert_eq!((got.width(), got.height()), (52, 32));
}

// ── FT-11: parse-time refusals ───────────────────────────────────────────────

const COLOR_MSG: &str = "must be a hex colour RRGGBB, #RRGGBB or RRGGBBAA";

#[test]
fn parse_refusals() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let input = write_img(d, "in.png", &noise("rgb8", 60, 40, 11));
    let o = d.join("e.png");
    let r_toml = d.join("r.toml");

    let pad_syntax = |got: &str| {
        format!(
            "error: --pad must be N, a whole number of pixels of at least 1, or an aspect W:H, such as 4:5; got {got:?}\n"
        )
    };
    // Whole-line refusals from `edit` itself.
    let whole: Vec<(Vec<&str>, String)> = vec![
        (vec!["--pad=-5", "--pad-color", "ff0000"], pad_syntax("-5")),
        (vec!["--pad", "0", "--pad-color", "ff0000"], pad_syntax("0")),
        (vec!["--pad", "10px", "--pad-color", "ff0000"], pad_syntax("10px")),
        (
            vec!["--pad", "4294967296", "--pad-color", "ff0000"],
            pad_syntax("4294967296"),
        ),
        (vec!["--pad", "+3", "--pad-color", "ff0000"], pad_syntax("+3")),
        (
            vec!["--pad", "10", "--pad-gravity", "north", "--pad-color", "ff0000"],
            "error: --pad-gravity only places an aspect --pad W:H; a border --pad N adds the same width on every side\n".to_owned(),
        ),
        (
            vec!["--pad", "4:", "--pad-color", "ff0000"],
            "error: --pad must be two whole numbers from 1 to 65535 written W:H, such as \"16:9\"; got \"4:\"\n".to_owned(),
        ),
        (
            vec!["--pad", "4:5", "--pad-gravity", "up", "--pad-color", "ff0000"],
            "error: --pad-gravity must be one of: center, north, south, east, west, northeast, northwest, southeast, southwest; got \"up\"\n".to_owned(),
        ),
        (
            vec!["--pad", "2", "--pad-color", "red"],
            format!("error: --pad-color {COLOR_MSG}, got \"red\"\n"),
        ),
        (
            vec!["--trim", "--trim-color", "zz"],
            format!("error: --trim-color {COLOR_MSG}, got \"zz\"\n"),
        ),
        (
            vec![],
            "error: edit requires at least one operation flag (--auto-orient, --resize-max, --invert, --crop, --rotate, --flip, --pad, --trim)\n".to_owned(),
        ),
    ];
    for (args, want) in whole {
        let mut a = vec![s(&input)];
        a.extend(&args);
        a.extend(["-o", s(&o), "-y", "--save-recipe", s(&r_toml)]);
        let r = ed(&a);
        assert_eq!(r.code, Some(2), "{args:?}: {}", r.stderr);
        assert_eq!(r.stderr, want, "{args:?}");
        assert!(!o.exists() && !r_toml.exists(), "{args:?}: wrote something");
    }

    // clap's own refusals, matched by the first line and the named flags.
    const MISSING: &str = "error: the following required arguments were not provided:";
    let clap_rows: Vec<(Vec<&str>, Vec<&str>)> = vec![
        (vec!["--pad", "2"], vec![MISSING, "--pad-color <COLOR>"]),
        (
            vec!["--pad-color", "ff0000", "--rotate", "90"],
            vec![MISSING, "--pad <N|W:H>"],
        ),
        (
            vec!["--pad-gravity", "north", "--rotate", "90"],
            vec![MISSING, "--pad-color <COLOR>", "--pad <N|W:H>"],
        ),
        (
            vec!["--trim-color", "ff0000", "--rotate", "90"],
            vec![MISSING, "--trim"],
        ),
        (
            vec!["--trim-tolerance", "8", "--rotate", "90"],
            vec![MISSING, "--trim"],
        ),
        (
            vec!["--pad", "2", "--pad-color", "ff0000", "--pad", "3"],
            vec!["error: the argument '--pad <N|W:H>' cannot be used multiple times"],
        ),
        (
            vec!["--trim", "--trim"],
            vec!["error: the argument '--trim' cannot be used multiple times"],
        ),
        (
            vec!["--trim", "--trim-tolerance", "256"],
            vec!["error: invalid value '256' for '--trim-tolerance <N>': 256 is not in 0..=255"],
        ),
        (
            vec!["--trim", "--trim-tolerance", "-1"],
            vec!["error: unexpected argument '-1' found"],
        ),
        (
            vec!["--pad", "-5", "--pad-color", "ff0000"],
            vec!["error: unexpected argument '-5' found"],
        ),
    ];
    for (args, wants) in clap_rows {
        let mut a = vec![s(&input)];
        a.extend(&args);
        a.extend(["-o", s(&o), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(2), "{args:?}: {}", r.stderr);
        for want in wants {
            assert!(
                r.stderr.contains(want),
                "{args:?} lacks {want:?}:\n{}",
                r.stderr
            );
        }
        assert!(!o.exists(), "{args:?}: wrote something");
    }
}

// ── FT-12: --save-recipe with --pad N on a batch ─────────────────────────────

#[test]
fn batch_save_recipe_with_pad_n_is_refused() {
    let (dir, d) = jpg_png_batch();
    let od = dir.path().join("od");
    let rec = dir.path().join("r.toml");
    let pad_line = format!(
        "error: --save-recipe with a border --pad N needs one input image, and {:?} names 2: the border is saved as a fraction of the image it ran on; use an aspect --pad W:H, or edit each image on its own\n",
        s(&d)
    );
    let crop_line = format!(
        "error: --save-recipe with a pixel --crop needs one input image, and {:?} names 2: the crop is saved as fractions of the image it ran on; use an aspect --crop W:H, or edit each image on its own\n",
        s(&d)
    );
    // Typed first, the border pad names itself; a crop typed first keeps its line.
    let refused: [(&[&str], &String); 2] = [
        (&["--pad", "2", "--pad-color", "ff0000"], &pad_line),
        (
            &["--pad", "2", "--pad-color", "ff0000", "--crop", "2x2+0+0"],
            &pad_line,
        ),
    ];
    for (args, want) in refused {
        let mut a = vec![s(&d)];
        a.extend(args);
        a.extend(["--out-dir", s(&od), "--save-recipe", s(&rec), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(2), "{args:?}: {}", r.stderr);
        assert_eq!(&r.stderr, want, "{args:?}");
        assert!(!od.exists() && !rec.exists(), "{args:?}: wrote something");
    }
    let r = ed(&[
        s(&d),
        "--crop",
        "2x2+0+0",
        "--pad",
        "2",
        "--pad-color",
        "ff0000",
        "--out-dir",
        s(&od),
        "--save-recipe",
        s(&rec),
        "-y",
    ]);
    assert_eq!(r.code, Some(2));
    assert_eq!(r.stderr, crop_line);

    // The aspect form and --trim are size-independent: they save from a batch.
    // (--trim needs images with something to keep, so it gets a noise batch.)
    let nd = dir.path().join("noisy");
    std::fs::create_dir(&nd).unwrap();
    write_img(&nd, "a.png", &framed("rgb8"));
    write_img(&nd, "b.png", &framed("rgba8"));
    for (src, args) in [
        (&d, &["--pad", "1:1", "--pad-color", "ff0000"][..]),
        (&nd, &["--trim", "--trim-tolerance", "3"][..]),
    ] {
        let od2 = dir.path().join("od2");
        let mut a = vec![s(src)];
        a.extend(args);
        a.extend(["--out-dir", s(&od2), "--save-recipe", s(&rec), "-y"]);
        let r = ed(&a);
        assert_eq!(r.code, Some(0), "{args:?}: {}", r.stderr);
        assert!(rec.exists(), "{args:?}: no recipe");
        std::fs::remove_file(&rec).unwrap();
        std::fs::remove_dir_all(&od2).unwrap();
    }

    // One file saves a border pad.
    let one = d.join("b.png");
    let o = dir.path().join("o.png");
    let r = ed(&[
        s(&one),
        "--pad",
        "2",
        "--pad-color",
        "ff0000",
        "-o",
        s(&o),
        "--save-recipe",
        s(&rec),
        "-y",
    ]);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(rec.exists());
    std::fs::remove_file(&rec).unwrap();

    // Stdin `-` is one image: allowed, and the recipe is written.
    let mut child = Command::new(BIN)
        .args([
            "edit",
            "-",
            "--pad",
            "2",
            "--pad-color",
            "ff0000",
            "-o",
            s(&o),
            "--save-recipe",
            s(&rec),
            "-y",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut bytes = Vec::new();
    flat()
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(rec.exists(), "stdin: no recipe");
}

// ── FT-13: a hex colour with a sign ──────────────────────────────────────────

#[test]
fn a_signed_hex_colour_is_refused_on_every_route() {
    let dir = TempDir::new().unwrap();
    let d = dir.path();
    let big = write_img(d, "big.png", &noise("rgb8", 256, 256, 3));
    let o = d.join("o.png");

    // A recipe with one step, run through `apply`.
    let apply = |step: &str| -> Ran {
        let rec = d.join("x.toml");
        std::fs::write(&rec, format!("version = \"1\"\n\n[[step]]\n{step}\n")).unwrap();
        run(&strs(&[
            "apply",
            "--recipe",
            s(&rec),
            s(&big),
            "-o",
            s(&o),
            "-y",
        ]))
    };

    for signed in ["+f+f+f", "#+f+f+f", "+f+f+f+f", "-f-f-f"] {
        for (op, step) in [
            (
                "pad",
                format!("op = \"pad\"\nborder = 0.25\ncolor = \"{signed}\""),
            ),
            ("trim", format!("op = \"trim\"\ncolor = \"{signed}\"")),
            (
                "watermark",
                format!("op = \"watermark\"\ntext = \"hi\"\ncolor = \"{signed}\""),
            ),
        ] {
            let _ = std::fs::remove_file(&o);
            let r = apply(&step);
            assert_eq!(r.code, Some(1), "recipe {op} {signed}: {}", r.stderr);
            assert_eq!(
                r.stderr,
                format!("error: recipe step 0 ('{op}'): 'color' {COLOR_MSG}, got {signed:?}\n"),
                "recipe {op} {signed}"
            );
            assert!(!o.exists(), "recipe {op} {signed}: output written");
        }

        // `watermark --color=` is a usage error (exit 2).
        let r = run(&strs(&[
            "watermark",
            s(&big),
            "--text",
            "hi",
            &format!("--color={signed}"),
            "-o",
            s(&o),
            "-y",
        ]));
        assert_eq!(r.code, Some(2), "watermark --color {signed}: {}", r.stderr);
        assert_eq!(
            r.stderr,
            format!("error: invalid color '{signed}': expected RRGGBB, #RRGGBB, or RRGGBBAA\n"),
            "watermark --color {signed}"
        );

        let r = ed(&[
            s(&big),
            "--pad",
            "2",
            &format!("--pad-color={signed}"),
            "-o",
            s(&o),
            "-y",
        ]);
        assert_eq!(r.code, Some(2), "--pad-color {signed}: {}", r.stderr);
        assert_eq!(
            r.stderr,
            format!("error: --pad-color {COLOR_MSG}, got {signed:?}\n"),
            "--pad-color {signed}"
        );
        let r = ed(&[
            s(&big),
            "--trim",
            &format!("--trim-color={signed}"),
            "-o",
            s(&o),
            "-y",
        ]);
        assert_eq!(r.code, Some(2), "--trim-color {signed}: {}", r.stderr);
        assert_eq!(
            r.stderr,
            format!("error: --trim-color {COLOR_MSG}, got {signed:?}\n"),
            "--trim-color {signed}"
        );
        assert!(!o.exists(), "{signed}: output written");
    }

    // Controls: an all-hex colour loads on the same routes, either case.
    for ok in ["#0f0f0f", "0F0F0F"] {
        let r = apply(&format!("op = \"pad\"\nborder = 0.25\ncolor = \"{ok}\""));
        assert_eq!(r.code, Some(0), "recipe pad {ok}: {}", r.stderr);
        let r = run(&strs(&[
            "watermark",
            s(&big),
            "--text",
            "hi",
            &format!("--color={ok}"),
            "-o",
            s(&o),
            "-y",
        ]));
        assert_eq!(r.code, Some(0), "watermark --color {ok}: {}", r.stderr);
        let r = ed(&[
            s(&big),
            "--pad",
            "2",
            &format!("--pad-color={ok}"),
            "-o",
            s(&o),
            "-y",
        ]);
        assert_eq!(r.code, Some(0), "--pad-color {ok}: {}", r.stderr);
    }
}
