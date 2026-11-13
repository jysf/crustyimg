//! `edit --crop`, `--rotate` and `--flip`: flag forms, refusals, typed order,
//! and `--save-recipe` replays.
//!
//! Every test drives the compiled binary. Fixtures are noise images written here
//! from a seeded xorshift, so the expected sizes and rectangles are literals, never
//! read back from the code under test. A replay row runs `edit … --save-recipe`,
//! then `apply` of that recipe on the same input, and compares the two decoded
//! images (colour type, dimensions, samples) and the saved step list in order.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, GrayImage, ImageEncoder, RgbImage, RgbaImage};
use tempfile::TempDir;

mod common;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// One xorshift64 step.
fn step(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

/// Seeded noise in one of four colour types: "rgb8", "rgba8", "l8", "rgb16".
/// One xorshift value per sample, truncated to the sample type.
fn noise(kind: &str, w: u32, h: u32, seed: u64) -> DynamicImage {
    let mut s = seed;
    let n = (w * h) as usize;
    match kind {
        "rgb8" => {
            let raw: Vec<u8> = (0..n * 3).map(|_| step(&mut s) as u8).collect();
            DynamicImage::ImageRgb8(RgbImage::from_raw(w, h, raw).unwrap())
        }
        "rgba8" => {
            let raw: Vec<u8> = (0..n * 4).map(|_| step(&mut s) as u8).collect();
            DynamicImage::ImageRgba8(RgbaImage::from_raw(w, h, raw).unwrap())
        }
        "l8" => {
            let raw: Vec<u8> = (0..n).map(|_| step(&mut s) as u8).collect();
            DynamicImage::ImageLuma8(GrayImage::from_raw(w, h, raw).unwrap())
        }
        "rgb16" => {
            let raw: Vec<u16> = (0..n * 3).map(|_| step(&mut s) as u16).collect();
            DynamicImage::ImageRgb16(image::ImageBuffer::from_raw(w, h, raw).unwrap())
        }
        other => panic!("unknown fixture kind {other}"),
    }
}

/// Write `img` as `dir/name` and return the path.
fn write_img(dir: &Path, name: &str, img: &DynamicImage) -> PathBuf {
    let p = dir.join(name);
    img.save(&p).unwrap();
    p
}

/// A 6×4 RGB noise PNG carrying an `eXIf` chunk with the given Orientation.
fn tagged_png(orientation: u8) -> Vec<u8> {
    let src = noise("rgb8", 6, 4, 5).to_rgb8();
    let mut out = Vec::new();
    let mut enc = PngEncoder::new(&mut out);
    enc.set_exif_metadata(common::tiff_with_orientation(orientation))
        .expect("the PNG encoder takes EXIF");
    enc.write_image(src.as_raw(), 6, 4, ExtendedColorType::Rgb8)
        .expect("encode tagged png");
    out
}

// ── Running the binary ────────────────────────────────────────────────────────

struct Ran {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(cwd: Option<&Path>, args: &[String]) -> Ran {
    let mut cmd = Command::new(BIN);
    cmd.args(args);
    if let Some(d) = cwd {
        cmd.current_dir(d);
    }
    let o = cmd.output().expect("failed to run crustyimg");
    Ran {
        code: o.status.code(),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn strs(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| (*s).to_owned()).collect()
}

/// `edit` with the extra args (any `{IN}` is replaced by the input path; if there
/// is none, the input follows `edit`), writing `-o out.png -y` and, when
/// `recipe` is true, `--save-recipe r.toml` in `dir`.
fn edit(dir: &Path, input: &Path, args: &[&str], recipe: bool) -> Ran {
    let mut a = vec!["edit".to_owned()];
    let mut placed = false;
    for s in args {
        if *s == "{IN}" {
            a.push(input.to_str().unwrap().to_owned());
            placed = true;
        } else {
            a.push((*s).to_owned());
        }
    }
    if !placed {
        a.insert(1, input.to_str().unwrap().to_owned());
    }
    a.extend(strs(&["-o", dir.join("e.png").to_str().unwrap(), "-y"]));
    if recipe {
        a.extend(strs(&[
            "--save-recipe",
            dir.join("r.toml").to_str().unwrap(),
        ]));
    }
    run(None, &a)
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

    let a = run(
        None,
        &strs(&[
            "apply",
            "--recipe",
            dir.path().join("r.toml").to_str().unwrap(),
            input.to_str().unwrap(),
            "-o",
            dir.path().join("a.png").to_str().unwrap(),
            "-y",
        ]),
    );
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

/// A fresh dir holding one 60×40 RGB noise PNG (seed 11).
fn fixture60() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let p = write_img(dir.path(), "in.png", &noise("rgb8", 60, 40, 11));
    (dir, p)
}

// ── Each flag alone ───────────────────────────────────────────────────────────

#[test]
fn replay_each_flag_alone() {
    let (_d, input) = fixture60();
    let rows: [(&[&str], (u32, u32), &str); 9] = [
        (&["--crop", "31x7+29+33"], (31, 7), "crop"),
        (&["--crop", "1:1"], (40, 40), "crop"),
        (
            &["--crop", "1:1", "--crop-gravity", "east"],
            (40, 40),
            "crop",
        ),
        (&["--rotate", "90"], (40, 60), "rotate"),
        (&["--rotate", "180"], (60, 40), "rotate"),
        (&["--rotate", "270"], (40, 60), "rotate"),
        (&["--rotate", "-90"], (40, 60), "rotate"),
        (&["--flip", "horizontal"], (60, 40), "flip"),
        (&["--flip", "vertical"], (60, 40), "flip"),
    ];
    for (args, dims, op) in rows {
        replay(&input, args, dims, &["auto-orient", op]);
    }
}

// ── Typed order ───────────────────────────────────────────────────────────────

#[test]
fn replay_pairs_both_orders() {
    let (_d, input) = fixture60();
    // (flag A, flag B, ops of A, ops of B, dims A-then-B, dims B-then-A)
    type Pair<'a> = (
        &'a [&'a str],
        &'a [&'a str],
        &'a str,
        &'a str,
        (u32, u32),
        (u32, u32),
    );
    let pairs: [Pair; 4] = [
        (
            &["--crop", "30x40+0+0"],
            &["--rotate", "90"],
            "crop",
            "rotate",
            (40, 30),
            (30, 40),
        ),
        (
            &["--crop", "20x40+5+0"],
            &["--flip", "horizontal"],
            "crop",
            "flip",
            (20, 40),
            (20, 40),
        ),
        (
            &["--crop", "30x20+0+0"],
            &["--resize-max", "30"],
            "crop",
            "resize",
            (30, 20),
            (30, 20),
        ),
        (
            &["--resize-max", "30"],
            &["--invert"],
            "resize",
            "invert",
            (30, 20),
            (30, 20),
        ),
    ];
    for (a, b, op_a, op_b, dims_ab, dims_ba) in pairs {
        let ab: Vec<&str> = a.iter().chain(b.iter()).copied().collect();
        let ba: Vec<&str> = b.iter().chain(a.iter()).copied().collect();
        let (img_ab, _) = replay(&input, &ab, dims_ab, &["auto-orient", op_a, op_b]);
        let (img_ba, _) = replay(&input, &ba, dims_ba, &["auto-orient", op_b, op_a]);
        assert!(
            img_ab.as_bytes() != img_ba.as_bytes(),
            "{ab:?} and {ba:?} must differ: the order is the point"
        );
    }
}

#[test]
fn replay_all_flags_two_orders() {
    let (_d, input) = fixture60();
    let fwd = [
        "--resize-max",
        "50",
        "--rotate",
        "90",
        "--flip",
        "vertical",
        "--crop",
        "20x30+3+4",
        "--invert",
    ];
    replay(
        &input,
        &fwd,
        (20, 30),
        &["auto-orient", "resize", "rotate", "flip", "crop", "invert"],
    );
    let rev = [
        "--invert",
        "--crop",
        "20x30+3+4",
        "--flip",
        "vertical",
        "--rotate",
        "90",
        "--resize-max",
        "50",
    ];
    replay(
        &input,
        &rev,
        (30, 20),
        &["auto-orient", "invert", "crop", "flip", "rotate", "resize"],
    );
}

#[test]
fn saved_recipe_keeps_crop_gravity() {
    let (_d, input) = fixture60();
    let (_, steps) = replay(
        &input,
        &["--crop", "1:1", "--crop-gravity", "east"],
        (40, 40),
        &["auto-orient", "crop"],
    );
    assert_eq!(steps[1]["aspect"].as_str(), Some("1:1"));
    assert_eq!(steps[1]["gravity"].as_str(), Some("east"));

    let (_, steps) = replay(
        &input,
        &["--crop", "1:1"],
        (40, 40),
        &["auto-orient", "crop"],
    );
    assert_eq!(steps[1]["aspect"].as_str(), Some("1:1"));
    assert!(
        !steps[1].contains_key("gravity"),
        "no --crop-gravity typed, so no gravity key: {:?}",
        steps[1]
    );
}

#[test]
fn typed_order_ignores_spelling_and_position() {
    let (_d, input) = fixture60();
    let shapes: [&[&str]; 4] = [
        &["{IN}", "--rotate", "90", "--crop", "30x40+0+0"],
        &["{IN}", "--rotate=90", "--crop=30x40+0+0"],
        &["--rotate", "90", "--crop", "30x40+0+0", "{IN}"],
        &["--rotate", "90", "{IN}", "--crop", "30x40+0+0"],
    ];
    let mut first: Option<DynamicImage> = None;
    for args in shapes {
        let (img, _) = replay(&input, args, (30, 40), &["auto-orient", "rotate", "crop"]);
        match &first {
            None => first = Some(img),
            Some(f) => assert!(f.as_bytes() == img.as_bytes(), "pixels differ for {args:?}"),
        }
    }
    // Crop typed first, then rotate: 30×40 turned is 40×30.
    replay(
        &input,
        &["{IN}", "--crop", "30x40+0+0", "--rotate", "90"],
        (40, 30),
        &["auto-orient", "crop", "rotate"],
    );
}

// ── Colour types ──────────────────────────────────────────────────────────────

#[test]
fn replay_colour_types() {
    type Form<'a> = (&'a [&'a str], (u32, u32), &'a [&'a str]);
    let forms: [Form; 6] = [
        (&["--crop", "31x7+29+33"], (31, 7), &["auto-orient", "crop"]),
        (
            &["--crop", "1:1", "--crop-gravity", "southwest"],
            (40, 40),
            &["auto-orient", "crop"],
        ),
        (&["--rotate", "270"], (40, 60), &["auto-orient", "rotate"]),
        (&["--flip", "vertical"], (60, 40), &["auto-orient", "flip"]),
        (
            &[
                "--rotate",
                "90",
                "--crop",
                "20x30+3+4",
                "--flip",
                "horizontal",
            ],
            (20, 30),
            &["auto-orient", "rotate", "crop", "flip"],
        ),
        (
            &["--resize-max", "30", "--crop", "10x10+5+5"],
            (10, 10),
            &["auto-orient", "resize", "crop"],
        ),
    ];
    for kind in ["rgb8", "rgba8", "l8", "rgb16"] {
        let dir = TempDir::new().unwrap();
        let img = noise(kind, 60, 40, 21);
        let input = write_img(dir.path(), "in.png", &img);
        for (args, dims, ops) in forms {
            let (out, _) = replay(&input, args, dims, ops);
            assert_eq!(
                out.color(),
                img.color(),
                "{kind} {args:?} keeps its colour type"
            );
        }
    }
}

// ── Pixel lengths ─────────────────────────────────────────────────────────────

/// The `crop` step's `(x, y, width, height)` floats, found by `op`, never by index.
fn crop_rect(steps: &[toml::Table]) -> (f64, f64, f64, f64) {
    let t = steps
        .iter()
        .find(|t| t["op"].as_str() == Some("crop"))
        .expect("a crop step was saved");
    let f = |k: &str| {
        t[k].as_float()
            .unwrap_or_else(|| panic!("{k} is a float: {t:?}"))
    };
    (f("x"), f("y"), f("width"), f("height"))
}

#[test]
fn pixel_crop_uses_the_received_size() {
    let (_d, input) = fixture60();
    // After --resize-max 30 the image is 30×20.
    let (_, steps) = replay(
        &input,
        &["--resize-max", "30", "--crop", "7x3+11+5"],
        (7, 3),
        &["auto-orient", "resize", "crop"],
    );
    assert_eq!(
        crop_rect(&steps),
        (11.0 / 30.0, 5.0 / 20.0, 7.0 / 30.0, 3.0 / 20.0)
    );
    // After --rotate 90 the image is 40×60.
    let (_, steps) = replay(
        &input,
        &["--rotate", "90", "--crop", "7x3+11+5"],
        (7, 3),
        &["auto-orient", "rotate", "crop"],
    );
    assert_eq!(
        crop_rect(&steps),
        (11.0 / 40.0, 5.0 / 60.0, 7.0 / 40.0, 3.0 / 60.0)
    );
}

#[test]
fn pixel_crop_writes_full_precision() {
    // The interior rectangle is noisy: 1/60 + 5/60 is not 6/60 bit for bit.
    assert_ne!(
        (1.0f64 / 60.0 + 5.0 / 60.0).to_bits(),
        (6.0f64 / 60.0).to_bits()
    );

    let dir = TempDir::new().unwrap();
    // 640×2: the one rectangle a six-decimal writer turns into a load refusal.
    let wide = write_img(dir.path(), "wide.png", &noise("rgb8", 640, 2, 3));
    replay(
        &wide,
        &["--crop", "631x2+9+0"],
        (631, 2),
        &["auto-orient", "crop"],
    );

    let (_d, input) = fixture60();
    replay(
        &input,
        &["--crop", "5x40+1+0"],
        (5, 40),
        &["auto-orient", "crop"],
    );
    replay(
        &input,
        &["--crop", "47x40+13+0"],
        (47, 40),
        &["auto-orient", "crop"],
    );
}

// ── Tagged input ──────────────────────────────────────────────────────────────

#[test]
fn replay_tagged_orientation_6_and_5() {
    for orientation in [6u8, 5u8] {
        let dir = TempDir::new().unwrap();
        let input = dir.path().join("t.png");
        std::fs::write(&input, tagged_png(orientation)).unwrap();
        replay(
            &input,
            &["--rotate", "90", "--crop", "4x3+1+0", "--auto-orient"],
            (4, 3),
            &["auto-orient", "rotate", "crop"],
        );
    }
}

// ── Apply failures (exit 1) ───────────────────────────────────────────────────

/// Run `edit` expecting `code` and exactly `stderr`; neither the output nor the
/// recipe may exist.
fn expect_failure(input: &Path, args: &[&str], code: i32, stderr: &str) {
    let dir = TempDir::new().unwrap();
    let r = edit(dir.path(), input, args, true);
    assert_eq!(r.code, Some(code), "{args:?}: {}", r.stderr);
    assert_eq!(r.stderr, stderr, "{args:?}");
    assert!(
        !dir.path().join("e.png").exists(),
        "{args:?}: output written"
    );
    assert!(
        !dir.path().join("r.toml").exists(),
        "{args:?}: recipe written"
    );
}

#[test]
fn past_the_edge_names_the_image_the_step_receives() {
    let (_d, input) = fixture60();
    // The image the step receives is 30×20, not the decoded 60×40.
    expect_failure(
        &input,
        &["--resize-max", "30", "--crop", "30x21+0+0"],
        1,
        "error: operation 'crop' failed: --crop 30x21+0+0 runs past the bottom edge of the \
         30\u{d7}20 px image it receives\n",
    );
    expect_failure(
        &input,
        &["--rotate", "90", "--crop", "41x10+0+0"],
        1,
        "error: operation 'crop' failed: --crop 41x10+0+0 runs past the right edge of the \
         40\u{d7}60 px image it receives\n",
    );
}

#[test]
fn aspect_crop_that_rounds_to_zero_fails() {
    let (_d, input) = fixture60();
    expect_failure(
        &input,
        &["--crop", "65535:1"],
        1,
        "error: operation 'crop' failed: the crop rounds to 60\u{d7}0 px on this 60\u{d7}40 px \
         image; it must keep at least 1 px each way\n",
    );
}

// ── Parse refusals (exit 2) ───────────────────────────────────────────────────

/// Run `edit` with `args` expecting exit 2 and `want` as the start of stderr (the
/// whole of it when `whole`); nothing written.
fn expect_usage(input: &Path, args: &[&str], want: &str, whole: bool) {
    let dir = TempDir::new().unwrap();
    let r = edit(dir.path(), input, args, true);
    assert_eq!(r.code, Some(2), "{args:?}: {}", r.stderr);
    if whole {
        assert_eq!(r.stderr, want, "{args:?}");
    } else {
        assert!(r.stderr.starts_with(want), "{args:?}: {}", r.stderr);
    }
    assert!(
        !dir.path().join("e.png").exists(),
        "{args:?}: output written"
    );
    assert!(
        !dir.path().join("r.toml").exists(),
        "{args:?}: recipe written"
    );
}

fn p1(got: &str) -> String {
    format!(
        "error: --crop must be WxH+X+Y in pixels, such as 800x600+10+20, or an aspect W:H, \
         such as 16:9; got {got:?}\n"
    )
}

#[test]
fn crop_pixel_form_refusals() {
    let (_d, input) = fixture60();
    // P1: seven spellings; `\u{d7}` is the multiplication sign, not the letter x.
    for bad in [
        "10X10+0+0",
        "10x10",
        "10\u{d7}10+0+0",
        "+10x10+0+0",
        "10x10+0+0+0",
        "99999999999x1+0+0",
    ] {
        expect_usage(&input, &["--crop", bad], &p1(bad), true);
    }
    // A leading '-' reaches the parser only in the `=` spelling.
    let eq = "--crop=-1x10+0+0";
    expect_usage(&input, &[eq], &p1("-1x10+0+0"), true);
    // P2.
    for bad in ["0x10+0+0", "10x0+0+0"] {
        expect_usage(
            &input,
            &["--crop", bad],
            &format!("error: --crop must keep at least 1 px each way; got {bad:?}\n"),
            true,
        );
    }
    // P3: space-separated, clap reads `-1x10+0+0` as a flag.
    expect_usage(
        &input,
        &["--crop", "-1x10+0+0"],
        "error: unexpected argument '-1' found\n",
        false,
    );
}

#[test]
fn crop_aspect_and_gravity_refusals() {
    let (_d, input) = fixture60();
    expect_usage(
        &input,
        &["--crop", "0:1"],
        "error: --crop must be two whole numbers from 1 to 65535 written W:H, such as \"16:9\"; \
         got \"0:1\"\n",
        true,
    );
    expect_usage(
        &input,
        &["--crop", "1.5:1"],
        "error: --crop must be two whole numbers written W:H; write \"1.5:1\" as \"3:2\"\n",
        true,
    );
    expect_usage(
        &input,
        &["--crop", "16:9", "--crop-gravity", "middle"],
        "error: --crop-gravity must be one of: center, north, south, east, west, northeast, \
         northwest, southeast, southwest; got \"middle\"\n",
        true,
    );
    expect_usage(
        &input,
        &["--crop", "10x10+0+0", "--crop-gravity", "north"],
        "error: --crop-gravity only places an aspect --crop W:H; a pixel --crop WxH+X+Y is \
         placed by its +X+Y\n",
        true,
    );
    // P8 is clap's own: assert both lines.
    expect_usage(
        &input,
        &["--crop-gravity", "north", "--invert"],
        "error: the following required arguments were not provided:\n  --crop <WxH+X+Y|W:H>\n",
        false,
    );
}

#[test]
fn rotate_refusals() {
    let (_d, input) = fixture60();
    expect_usage(
        &input,
        &["--rotate", "45"],
        "error: --rotate must be a multiple of 90, got 45; rotating by any other angle is not \
         supported yet\n",
        true,
    );
    expect_usage(
        &input,
        &["--rotate", "450"],
        "error: --rotate must be a number in -360.0..=360.0, got 450\n",
        true,
    );
    expect_usage(
        &input,
        &["--rotate", "90.0"],
        "error: invalid value '90.0' for '--rotate <DEGREES>': invalid digit found in string\n",
        false,
    );
    expect_usage(
        &input,
        &["--rotate", "abc"],
        "error: invalid value 'abc' for '--rotate <DEGREES>': invalid digit found in string\n",
        false,
    );
}

#[test]
fn flip_refusals() {
    let (_d, input) = fixture60();
    expect_usage(
        &input,
        &["--flip", "h"],
        "error: --flip must be one of: horizontal, vertical; got \"h\"\n",
        true,
    );
    // `--flip` with no value: run it last so `-o` is not read as its value.
    let r = run(None, &strs(&["edit", input.to_str().unwrap(), "--flip"]));
    assert_eq!(r.code, Some(2));
    assert!(
        r.stderr
            .starts_with("error: a value is required for '--flip <AXIS>' but none was supplied"),
        "{}",
        r.stderr
    );
}

#[test]
fn repeated_op_flags_are_refused() {
    let (_d, input) = fixture60();
    let rows: [(&[&str], &str); 3] = [
        (
            &["--rotate", "90", "--rotate", "90"],
            "'--rotate <DEGREES>'",
        ),
        (
            &["--crop", "1:1", "--crop", "2:1"],
            "'--crop <WxH+X+Y|W:H>'",
        ),
        (
            &["--flip", "vertical", "--flip", "vertical"],
            "'--flip <AXIS>'",
        ),
    ];
    for (args, name) in rows {
        expect_usage(
            &input,
            args,
            &format!("error: the argument {name} cannot be used multiple times\n"),
            false,
        );
    }
}

#[test]
fn negative_rotate_and_dash_input() {
    let (_d, input) = fixture60();
    for args in [&["--rotate", "-90"][..], &["--rotate=-90"][..]] {
        let dir = TempDir::new().unwrap();
        let r = edit(dir.path(), &input, args, false);
        assert_eq!(r.code, Some(0), "{args:?}: {}", r.stderr);
        let out = image::open(dir.path().join("e.png")).unwrap();
        assert_eq!((out.width(), out.height()), (40, 60), "{args:?}");
    }
    // An input named `-g.png`, after `--`.
    let dir = TempDir::new().unwrap();
    noise("rgb8", 60, 40, 11)
        .save(dir.path().join("-g.png"))
        .unwrap();
    let r = run(
        Some(dir.path()),
        &strs(&[
            "edit", "--rotate", "-90", "-o", "x.png", "-y", "--", "-g.png",
        ]),
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert!(dir.path().join("x.png").exists());
}

#[test]
fn no_op_flag_message_lists_every_flag() {
    let (_d, input) = fixture60();
    expect_usage(
        &input,
        &[],
        "error: edit requires at least one operation flag (--auto-orient, --resize-max, \
         --invert, --crop, --rotate, --flip, --pad, --trim)\n",
        true,
    );
}

#[test]
fn auto_orient_alone_is_accepted() {
    let (_d, input) = fixture60();
    let dir = TempDir::new().unwrap();
    let r = edit(dir.path(), &input, &["--auto-orient"], true);
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert_eq!(ops_of(&saved_steps(dir.path())), ["auto-orient"]);
}

#[test]
fn edit_help_states_typed_order() {
    let r = run(None, &strs(&["edit", "--help"]));
    assert_eq!(r.code, Some(0));
    for want in [
        "in the order",
        "you type them",
        "--crop <WxH+X+Y|W:H>",
        "--crop-gravity <GRAVITY>",
        "--rotate <DEGREES>",
        "--flip <AXIS>",
        "--pad <N|W:H>",
        "--pad-color <COLOR>",
        "--pad-gravity <GRAVITY>",
        "--trim",
        "--trim-color <COLOR>",
        "--trim-tolerance <N>",
        "one image or a batch",
    ] {
        assert!(
            r.stdout.contains(want),
            "help lacks {want:?}:\n{}",
            r.stdout
        );
    }
    assert!(
        !r.stdout.contains("canonical"),
        "help still says canonical:\n{}",
        r.stdout
    );
}

// ── Batches ───────────────────────────────────────────────────────────────────

/// A directory `in/` holding `big.png` (60×40) and `small.png` (20×50).
fn batch_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let d = dir.path().join("in");
    std::fs::create_dir(&d).unwrap();
    noise("rgb8", 60, 40, 1).save(d.join("big.png")).unwrap();
    noise("rgb8", 20, 50, 2).save(d.join("small.png")).unwrap();
    (dir, d)
}

fn dims_of(p: &Path) -> (u32, u32) {
    let i = image::open(p).unwrap();
    (i.width(), i.height())
}

#[test]
fn batch_pixel_crop_crops_each_image_in_its_own_pixels() {
    let (dir, d) = batch_dir();
    let out = dir.path().join("o");
    let r = run(
        None,
        &strs(&[
            "edit",
            d.to_str().unwrap(),
            "--crop",
            "15x15+0+0",
            "--out-dir",
            out.to_str().unwrap(),
            "-y",
        ]),
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    assert_eq!(dims_of(&out.join("big.png")), (15, 15));
    assert_eq!(dims_of(&out.join("small.png")), (15, 15));
}

#[test]
fn batch_pixel_crop_too_large_for_one_image_fails_that_input() {
    let (dir, d) = batch_dir();
    let out = dir.path().join("o");
    let r = run(
        None,
        &strs(&[
            "edit",
            d.to_str().unwrap(),
            "--crop",
            "30x30+0+0",
            "--out-dir",
            out.to_str().unwrap(),
            "-y",
        ]),
    );
    assert_eq!(r.code, Some(6), "{}", r.stderr);
    assert_eq!(dims_of(&out.join("big.png")), (30, 30));
    assert!(
        !out.join("small.png").exists(),
        "the failed input wrote nothing"
    );
    assert!(r.stderr.contains("small.png"), "{}", r.stderr);
    assert!(
        r.stderr
            .contains("runs past the right edge of the 20\u{d7}50 px image it receives"),
        "{}",
        r.stderr
    );
}

#[test]
fn batch_save_recipe_with_pixel_crop_is_refused() {
    let (dir, d) = batch_dir();
    let out = dir.path().join("o");
    let recipe = dir.path().join("r.toml");
    let dstr = d.to_str().unwrap();
    let r = run(
        None,
        &strs(&[
            "edit",
            dstr,
            "--crop",
            "10x10+0+0",
            "--out-dir",
            out.to_str().unwrap(),
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-y",
        ]),
    );
    assert_eq!(r.code, Some(2), "{}", r.stderr);
    assert_eq!(
        r.stderr,
        format!(
            "error: --save-recipe with a pixel --crop needs one input image, and {dstr:?} names \
             2: the crop is saved as fractions of the image it ran on; use an aspect --crop W:H, \
             or edit each image on its own\n"
        )
    );
    assert!(!out.exists(), "no out-dir");
    assert!(!recipe.exists(), "no recipe");

    // An aspect crop has no pixel length, so the same directory saves fine.
    let r = run(
        None,
        &strs(&[
            "edit",
            dstr,
            "--crop",
            "1:1",
            "--out-dir",
            out.to_str().unwrap(),
            "--save-recipe",
            recipe.to_str().unwrap(),
            "-y",
        ]),
    );
    assert_eq!(r.code, Some(0), "{}", r.stderr);
    let text = std::fs::read_to_string(&recipe).unwrap();
    let doc: toml::Table = text.parse().unwrap();
    let ops: Vec<&str> = doc["step"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["op"].as_str().unwrap())
        .collect();
    assert_eq!(ops, ["auto-orient", "crop"]);
}
