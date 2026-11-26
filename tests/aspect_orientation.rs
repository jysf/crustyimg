//! `orientation = "match"` on an aspect `crop` or `pad`, and the two `edit`
//! flags that write it.
//!
//! The key turns the aspect to follow each image: the aspect's long side runs
//! along the image's long side, strictly wider is landscape, strictly taller is
//! portrait, and a square keeps the aspect as written.
//!
//! Every test drives the compiled binary. Fixtures are noise PNGs written here
//! from a seeded xorshift; every expected size is a hand-derived literal, never
//! read back from the code under test. Refusal rows compare the whole stderr.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, RgbImage};
use tempfile::TempDir;

mod common;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// Seeded RGB noise, `w`×`h`.
fn noise(w: u32, h: u32) -> RgbImage {
    let mut s: u64 = 0x5EED_0163 ^ (u64::from(w) << 20) ^ u64::from(h);
    let raw: Vec<u8> = (0..(w * h * 3) as usize)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s as u8
        })
        .collect();
    RgbImage::from_raw(w, h, raw).unwrap()
}

/// Write a `w`×`h` noise PNG as `dir/name`.
fn write_png(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
    let p = dir.join(name);
    DynamicImage::ImageRgb8(noise(w, h)).save(&p).unwrap();
    p
}

/// A 64×48 noise PNG carrying an `eXIf` chunk with Orientation 6, so the stored
/// buffer is landscape and the displayed one is portrait.
fn write_tagged_png(dir: &Path, name: &str) -> PathBuf {
    let src = noise(64, 48);
    let mut out = Vec::new();
    let mut enc = PngEncoder::new(&mut out);
    enc.set_exif_metadata(common::tiff_with_orientation(6))
        .expect("the PNG encoder takes EXIF");
    enc.write_image(src.as_raw(), 64, 48, ExtendedColorType::Rgb8)
        .expect("encode tagged png");
    let p = dir.join(name);
    std::fs::write(&p, out).unwrap();
    p
}

fn dims(path: &Path) -> (u32, u32) {
    let img = image::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    (img.width(), img.height())
}

// ── Running the binary ────────────────────────────────────────────────────────

struct Ran {
    code: Option<i32>,
    stderr: String,
}

fn run(args: &[&str]) -> Ran {
    let o = Command::new(BIN)
        .args(args)
        .output()
        .expect("failed to run crustyimg");
    Ran {
        code: o.status.code(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

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

fn crop_step(body: &str) -> String {
    format!("op = \"crop\"\n{body}")
}

fn pad_step(body: &str) -> String {
    format!("op = \"pad\"\ncolor = \"#ffffff\"\n{body}")
}

/// Write `text` as `dir/r.toml` and `apply` it to `input`, writing `dir/o.png`.
fn apply_one(dir: &Path, text: &str, input: &Path) -> Ran {
    let r = dir.join("r.toml");
    std::fs::write(&r, text).unwrap();
    run(&[
        "apply",
        "--recipe",
        r.to_str().unwrap(),
        input.to_str().unwrap(),
        "-o",
        dir.join("o.png").to_str().unwrap(),
        "-y",
    ])
}

/// The size `apply` of one step gives a `w`×`h` image.
fn size_after(step: &str, w: u32, h: u32) -> (u32, u32) {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", w, h);
    let r = apply_one(dir.path(), &recipe(&[step]), &input);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    dims(&dir.path().join("o.png"))
}

/// `apply` over a folder, writing `dir/out/<name>`.
fn apply_folder(dir: &Path, text: &str, folder: &Path) -> Ran {
    let r = dir.join("r.toml");
    std::fs::write(&r, text).unwrap();
    run(&[
        "apply",
        "--recipe",
        r.to_str().unwrap(),
        folder.to_str().unwrap(),
        "--out-dir",
        dir.join("out").to_str().unwrap(),
    ])
}

// ── The size rows ─────────────────────────────────────────────────────────────

/// One row: the written aspect, the image's size, the size with the key on, and
/// the size with it off.
type Row = (&'static str, (u32, u32), (u32, u32), (u32, u32));

/// `crop`: (aspect, image, on, off). Hand-derived: the largest fit, the short
/// side rounded half up.
const CROP_ROWS: [Row; 10] = [
    ("16:9", (64, 48), (64, 36), (64, 36)),
    ("9:16", (64, 48), (64, 36), (27, 48)),
    ("16:9", (48, 64), (36, 64), (48, 27)),
    ("9:16", (48, 64), (36, 64), (36, 64)),
    ("16:9", (48, 48), (48, 27), (48, 27)),
    ("9:16", (48, 48), (27, 48), (27, 48)),
    ("16:9", (48, 49), (28, 49), (48, 27)),
    ("9:16", (48, 49), (28, 49), (28, 49)),
    ("16:9", (49, 48), (49, 28), (49, 28)),
    ("9:16", (49, 48), (49, 28), (27, 48)),
];

/// `pad`: the same ten rows, the smallest canvas that holds the image.
const PAD_ROWS: [Row; 10] = [
    ("16:9", (64, 48), (85, 48), (85, 48)),
    ("9:16", (64, 48), (85, 48), (64, 114)),
    ("16:9", (48, 64), (48, 85), (114, 64)),
    ("9:16", (48, 64), (48, 85), (48, 85)),
    ("16:9", (48, 48), (85, 48), (85, 48)),
    ("9:16", (48, 48), (48, 85), (48, 85)),
    ("16:9", (48, 49), (48, 85), (87, 49)),
    ("9:16", (48, 49), (48, 85), (48, 85)),
    ("16:9", (49, 48), (85, 48), (85, 48)),
    ("9:16", (49, 48), (85, 48), (49, 87)),
];

fn crop_with(aspect: &str, key: bool) -> String {
    let orientation = if key { "\norientation = \"match\"" } else { "" };
    crop_step(&format!("aspect = \"{aspect}\"{orientation}"))
}

fn pad_with(aspect: &str, key: bool) -> String {
    let orientation = if key { "\norientation = \"match\"" } else { "" };
    pad_step(&format!("aspect = \"{aspect}\"{orientation}"))
}

/// Run every row, collect every mismatch, and assert the row count.
fn check_rows(rows: &[Row], step: fn(&str, bool) -> String) {
    let mut bad = Vec::new();
    for (aspect, (w, h), on, off) in rows {
        let got_on = size_after(&step(aspect, true), *w, *h);
        let got_off = size_after(&step(aspect, false), *w, *h);
        if got_on != *on {
            bad.push(format!("{aspect} {w}x{h} on: want {on:?}, got {got_on:?}"));
        }
        if got_off != *off {
            bad.push(format!(
                "{aspect} {w}x{h} off: want {off:?}, got {got_off:?}"
            ));
        }
    }
    assert!(
        bad.is_empty(),
        "{} rows disagree:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert_eq!(rows.len(), 10);
}

#[test]
fn crop_follows_each_image_shape() {
    check_rows(&CROP_ROWS, crop_with);
}

#[test]
fn pad_follows_each_image_shape() {
    check_rows(&PAD_ROWS, pad_with);
}

/// The tables' own negative control: the key must change the size in exactly the
/// rows where the written order disagrees with the image's shape, so a table
/// whose "on" and "off" columns agreed everywhere could not catch a key that was
/// never read.
#[test]
fn the_rows_can_tell_on_from_off() {
    let differing = |rows: &[Row]| rows.iter().filter(|r| r.2 != r.3).count();
    assert_eq!(differing(&CROP_ROWS), 4);
    assert_eq!(differing(&PAD_ROWS), 4);
    // And they are the four the shape says: landscape 9:16, portrait 16:9,
    // 48x49 16:9 and 49x48 9:16.
    for rows in [&CROP_ROWS, &PAD_ROWS] {
        let who: Vec<(&str, (u32, u32))> = rows
            .iter()
            .filter(|r| r.2 != r.3)
            .map(|r| (r.0, r.1))
            .collect();
        assert_eq!(
            who,
            vec![
                ("9:16", (64, 48)),
                ("16:9", (48, 64)),
                ("16:9", (48, 49)),
                ("9:16", (49, 48)),
            ]
        );
    }
}

#[test]
fn one_to_one_is_unchanged_by_the_key() {
    // (image, crop, pad): the same with the key on and off.
    let rows = [
        ((64, 48), (48, 48), (64, 64)),
        ((48, 64), (48, 48), (64, 64)),
        ((48, 48), (48, 48), (48, 48)),
        ((48, 49), (48, 48), (49, 49)),
    ];
    let mut bad = Vec::new();
    for ((w, h), crop, pad) in rows {
        for key in [true, false] {
            let c = size_after(&crop_with("1:1", key), w, h);
            let p = size_after(&pad_with("1:1", key), w, h);
            if c != crop {
                bad.push(format!(
                    "crop 1:1 {w}x{h} key={key}: want {crop:?}, got {c:?}"
                ));
            }
            if p != pad {
                bad.push(format!(
                    "pad 1:1 {w}x{h} key={key}: want {pad:?}, got {p:?}"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn the_key_reads_the_image_the_step_receives() {
    let dir = TempDir::new().unwrap();
    let input = write_tagged_png(dir.path(), "tagged.png");
    let cases: [(&str, Vec<String>, (u32, u32)); 6] = [
        // Without `auto-orient` the step sees the stored 64x48 landscape.
        (
            "crop on, no auto-orient",
            vec![crop_with("16:9", true)],
            (64, 36),
        ),
        (
            "crop on, after auto-orient",
            vec!["op = \"auto-orient\"".to_owned(), crop_with("16:9", true)],
            (36, 64),
        ),
        (
            "crop off, after auto-orient",
            vec!["op = \"auto-orient\"".to_owned(), crop_with("16:9", false)],
            (48, 27),
        ),
        (
            "pad on, no auto-orient",
            vec![pad_with("16:9", true)],
            (85, 48),
        ),
        (
            "pad on, after auto-orient",
            vec!["op = \"auto-orient\"".to_owned(), pad_with("16:9", true)],
            (48, 85),
        ),
        (
            "pad off, after auto-orient",
            vec!["op = \"auto-orient\"".to_owned(), pad_with("16:9", false)],
            (114, 64),
        ),
    ];
    let mut bad = Vec::new();
    for (label, steps, want) in cases {
        let refs: Vec<&str> = steps.iter().map(String::as_str).collect();
        let r = apply_one(dir.path(), &recipe(&refs), &input);
        if r.code != Some(0) {
            bad.push(format!("{label}: exit {:?}: {}", r.code, r.stderr));
            continue;
        }
        let got = dims(&dir.path().join("o.png"));
        if got != want {
            bad.push(format!("{label}: want {want:?}, got {got:?}"));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// ── Load refusals ─────────────────────────────────────────────────────────────

/// Run each row through `apply` on a 64x48 input and compare the exit code and
/// the whole stderr. Collects every mismatch and asserts the row count.
fn check_refusals(rows: &[(String, String)], count: usize) {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png", 64, 48);
    let mut bad = Vec::new();
    for (body, want) in rows {
        let r = apply_one(dir.path(), &recipe(&[body]), &input);
        let want = format!("{want}\n");
        if r.code != Some(1) || r.stderr != want {
            bad.push(format!(
                "{body:?}\n  want exit 1, {want:?}\n  got  exit {:?}, {:?}",
                r.code, r.stderr
            ));
        }
        if dir.path().join("o.png").exists() {
            bad.push(format!("{body:?} wrote an output"));
        }
    }
    assert!(
        bad.is_empty(),
        "{} rows disagree:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert_eq!(rows.len(), count);
}

fn crop_msg(reason: &str) -> String {
    format!("error: recipe step 0 ('crop'): {reason}")
}

fn pad_msg(reason: &str) -> String {
    format!("error: recipe step 0 ('pad'): {reason}")
}

#[test]
fn crop_load_refusals() {
    let only_aspect =
        "'orientation' is only used with 'aspect'; a rectangle crop's shape is set by \
                       its width and height";
    let rect = "x = 0\ny = 0\nwidth = 0.5\nheight = 0.5\n";
    let rows: Vec<(String, String)> = vec![
        (
            crop_step(&format!("{rect}orientation = \"match\"")),
            crop_msg(only_aspect),
        ),
        (crop_step("orientation = \"match\""), crop_msg(only_aspect)),
        (
            crop_step("aspect = \"3:2\"\norientation = \"landscape\""),
            crop_msg("'orientation' must be one of: match; got \"landscape\""),
        ),
        (
            crop_step("aspect = \"3:2\"\norientation = \"Match\""),
            crop_msg("'orientation' must be one of: match; got \"Match\""),
        ),
        (
            crop_step("aspect = \"3:2\"\norientation = true"),
            crop_msg("'orientation' must be one of: match; got true"),
        ),
        (
            crop_step("aspect = \"3:2\"\norientation = 1"),
            crop_msg("'orientation' must be one of: match; got the integer 1"),
        ),
        // The table runs first, so a wrong type on a rectangle is the type error.
        (
            crop_step(&format!("{rect}orientation = true")),
            crop_msg("'orientation' must be one of: match; got true"),
        ),
        // Gravity's refusal comes first when both keys are on a rectangle.
        (
            crop_step(&format!(
                "{rect}gravity = \"north\"\norientation = \"match\""
            )),
            crop_msg("'gravity' is only used with 'aspect'; a rectangle crop is placed by x and y"),
        ),
    ];
    check_refusals(&rows, 8);
}

#[test]
fn pad_load_refusals() {
    let only_aspect = "'orientation' is only used with 'aspect'; a border pad adds the same width \
                       on every side";
    let rows: Vec<(String, String)> = vec![
        (
            pad_step("border = 0.1\norientation = \"match\""),
            pad_msg(only_aspect),
        ),
        (
            pad_step("aspect = \"3:2\"\norientation = \"landscape\""),
            pad_msg("'orientation' must be one of: match; got \"landscape\""),
        ),
        (
            pad_step("aspect = \"3:2\"\norientation = \"Match\""),
            pad_msg("'orientation' must be one of: match; got \"Match\""),
        ),
        (
            pad_step("aspect = \"3:2\"\norientation = true"),
            pad_msg("'orientation' must be one of: match; got true"),
        ),
        (
            pad_step("aspect = \"3:2\"\norientation = 1"),
            pad_msg("'orientation' must be one of: match; got the integer 1"),
        ),
        // Gravity's refusal comes first when both keys are on a border.
        (
            pad_step("border = 0.1\ngravity = \"north\"\norientation = \"match\""),
            pad_msg(
                "'gravity' is only used with 'aspect'; a border pad adds the same width on \
                 every side",
            ),
        ),
    ];
    check_refusals(&rows, 6);
}

#[test]
fn the_key_with_an_aspect_is_never_refused() {
    let mut bad = Vec::new();
    for (w, h) in [(64, 48), (48, 48), (48, 49)] {
        let steps = [
            crop_step("aspect = \"3:2\"\norientation = \"match\""),
            crop_step("aspect = \"3:2\"\ngravity = \"north\"\norientation = \"match\""),
            crop_step("aspect = \"1:1\"\norientation = \"match\""),
            pad_step("aspect = \"3:2\"\norientation = \"match\""),
            pad_step("aspect = \"3:2\"\ngravity = \"north\"\norientation = \"match\""),
            pad_step("aspect = \"1:1\"\norientation = \"match\""),
        ];
        let dir = TempDir::new().unwrap();
        let input = write_png(dir.path(), "in.png", w, h);
        for step in &steps {
            let r = apply_one(dir.path(), &recipe(&[step]), &input);
            if r.code != Some(0) {
                bad.push(format!("{w}x{h} {step:?}: exit {:?}: {}", r.code, r.stderr));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// ── A folder ──────────────────────────────────────────────────────────────────

/// A folder with landscape `l.png`, portrait `p.png` and square `s.png`.
fn mixed_folder(dir: &Path) -> PathBuf {
    let folder = dir.join("in");
    std::fs::create_dir(&folder).unwrap();
    write_png(&folder, "l.png", 64, 48);
    write_png(&folder, "p.png", 48, 64);
    write_png(&folder, "s.png", 48, 48);
    folder
}

#[test]
fn batch_load_refusal_writes_nothing() {
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("in");
    std::fs::create_dir(&folder).unwrap();
    write_png(&folder, "a.png", 64, 48);
    write_png(&folder, "b.png", 48, 64);
    let text = recipe(&[&crop_step("aspect = \"3:2\"\norientation = \"landscape\"")]);
    let r = apply_folder(dir.path(), &text, &folder);
    assert_eq!(r.code, Some(1), "stderr: {}", r.stderr);
    assert_eq!(
        r.stderr,
        format!(
            "{}\n",
            crop_msg("'orientation' must be one of: match; got \"landscape\"")
        )
    );
    assert!(!dir.path().join("out").exists(), "no output directory");
}

#[test]
fn batch_follows_each_image() {
    let want = |step: String, sizes: [(u32, u32); 3]| {
        let dir = TempDir::new().unwrap();
        let folder = mixed_folder(dir.path());
        let r = apply_folder(dir.path(), &recipe(&[&step]), &folder);
        assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
        let got = [
            dims(&dir.path().join("out/l.png")),
            dims(&dir.path().join("out/p.png")),
            dims(&dir.path().join("out/s.png")),
        ];
        assert_eq!(got, sizes, "{step}");
    };
    want(crop_with("16:9", true), [(64, 36), (36, 64), (48, 27)]);
    want(pad_with("16:9", true), [(85, 48), (48, 85), (85, 48)]);
}

// ── `edit`'s flags ────────────────────────────────────────────────────────────

/// `edit IN <args…> -o e.png -y --save-recipe r.toml`, all in `dir`.
fn edit(dir: &Path, input: &Path, args: &[&str]) -> Ran {
    let mut a = vec!["edit", input.to_str().unwrap()];
    a.extend_from_slice(args);
    let out = dir.join("e.png");
    let rec = dir.join("r.toml");
    a.extend(["-o", out.to_str().unwrap(), "-y"]);
    a.extend(["--save-recipe", rec.to_str().unwrap()]);
    run(&a)
}

#[test]
fn orientation_flag_refusals() {
    const CLAP: &str = "error: the following required arguments were not provided:";
    let wrong =
        |flag: &str, got: &str| format!("error: {flag} must be one of: match; got \"{got}\"\n");
    // (args, the stderr: whole when `exact`, else its first lines).
    let rows: Vec<(Vec<&str>, String, bool)> = vec![
        // 13: without --crop (clap).
        (
            vec!["--crop-orientation", "match"],
            format!("{CLAP}\n  --crop <WxH+X+Y|W:H>\n"),
            false,
        ),
        // 9: a pixel crop.
        (
            vec!["--crop", "20x20+0+0", "--crop-orientation", "match"],
            "error: --crop-orientation only turns an aspect --crop W:H; a pixel --crop WxH+X+Y \
             keeps the size it gives\n"
                .to_owned(),
            true,
        ),
        // 15: gravity's refusal comes first.
        (
            vec![
                "--crop",
                "20x20+0+0",
                "--crop-gravity",
                "north",
                "--crop-orientation",
                "match",
            ],
            "error: --crop-gravity only places an aspect --crop W:H; a pixel --crop WxH+X+Y is \
             placed by its +X+Y\n"
                .to_owned(),
            true,
        ),
        // 11: a bad value, named by the flag.
        (
            vec!["--crop", "3:2", "--crop-orientation", "landscape"],
            wrong("--crop-orientation", "landscape"),
            true,
        ),
        (
            vec!["--crop", "3:2", "--crop-orientation", "Match"],
            wrong("--crop-orientation", "Match"),
            true,
        ),
        // 14: without --pad (clap); --pad itself needs --pad-color.
        (
            vec!["--pad-orientation", "match"],
            format!("{CLAP}\n  --pad-color <COLOR>\n  --pad <N|W:H>\n"),
            false,
        ),
        // 10: a border pad.
        (
            vec![
                "--pad",
                "4",
                "--pad-color",
                "#ffffff",
                "--pad-orientation",
                "match",
            ],
            "error: --pad-orientation only turns an aspect --pad W:H; a border --pad N adds the \
             same width on every side\n"
                .to_owned(),
            true,
        ),
        // 12: a bad value, named by the flag.
        (
            vec![
                "--pad",
                "3:2",
                "--pad-color",
                "#ffffff",
                "--pad-orientation",
                "landscape",
            ],
            wrong("--pad-orientation", "landscape"),
            true,
        ),
        (
            vec![
                "--pad",
                "3:2",
                "--pad-color",
                "#ffffff",
                "--pad-orientation",
                "Match",
            ],
            wrong("--pad-orientation", "Match"),
            true,
        ),
    ];
    let mut bad = Vec::new();
    for (args, want, exact) in &rows {
        let dir = TempDir::new().unwrap();
        let input = write_png(dir.path(), "in.png", 48, 64);
        let r = edit(dir.path(), &input, args);
        let ok = if *exact {
            r.stderr == *want
        } else {
            r.stderr.starts_with(want.as_str())
        };
        if r.code != Some(2) || !ok {
            bad.push(format!(
                "{args:?}\n  want exit 2, {want:?}\n  got  exit {:?}, {:?}",
                r.code, r.stderr
            ));
        }
        for name in ["e.png", "r.toml"] {
            if dir.path().join(name).exists() {
                bad.push(format!("{args:?} wrote {name}"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} rows disagree:\n{}",
        bad.len(),
        bad.join("\n")
    );
    assert_eq!(rows.len(), 9);
}

/// The recipe `edit --save-recipe` writes for one aspect step after
/// `auto-orient`; `lines` are the step's keys, in saved (sorted) order.
fn saved(op: &str, lines: &[&str]) -> String {
    let mut out =
        format!("version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n\n[[step]]\nop = \"{op}\"\n");
    for l in lines {
        out.push_str(l);
        out.push('\n');
    }
    out
}

#[test]
fn save_recipe_writes_the_key_only_when_typed_and_replays() {
    let crop_on = saved("crop", &["aspect = \"3:2\"", "orientation = \"match\""]);
    let crop_off = saved("crop", &["aspect = \"3:2\""]);
    let pad_on = saved(
        "pad",
        &[
            "aspect = \"3:2\"",
            "color = \"#ffffff\"",
            "orientation = \"match\"",
        ],
    );
    let pad_off = saved("pad", &["aspect = \"3:2\"", "color = \"#ffffff\""]);
    // (label, flags, the saved text, the sizes on landscape 64x48, portrait
    // 48x64 and square 48x48). Hand-derived, halves up.
    let rows = vec![
        (
            "crop on",
            vec!["--crop", "3:2", "--crop-orientation", "match"],
            &crop_on,
            [(64, 43), (43, 64), (48, 32)],
        ),
        (
            "crop off",
            vec!["--crop", "3:2"],
            &crop_off,
            [(64, 43), (48, 32), (48, 32)],
        ),
        (
            "pad on",
            vec![
                "--pad",
                "3:2",
                "--pad-color",
                "#ffffff",
                "--pad-orientation",
                "match",
            ],
            &pad_on,
            [(72, 48), (48, 72), (72, 48)],
        ),
        (
            "pad off",
            vec!["--pad", "3:2", "--pad-color", "#ffffff"],
            &pad_off,
            [(72, 48), (96, 64), (72, 48)],
        ),
    ];
    let shapes = [(64, 48), (48, 64), (48, 48)];
    let mut bad = Vec::new();
    for (label, flags, text, sizes) in &rows {
        for ((w, h), want) in shapes.iter().zip(sizes) {
            let dir = TempDir::new().unwrap();
            let input = write_png(dir.path(), "in.png", *w, *h);
            let e = edit(dir.path(), &input, flags);
            let b = run(&[
                "apply",
                "--recipe",
                dir.path().join("r.toml").to_str().unwrap(),
                input.to_str().unwrap(),
                "-o",
                dir.path().join("b.png").to_str().unwrap(),
                "-y",
            ]);
            if e.code != Some(0) || b.code != Some(0) {
                bad.push(format!(
                    "{label} {w}x{h}: edit {:?} {}, apply {:?} {}",
                    e.code, e.stderr, b.code, b.stderr
                ));
                continue;
            }
            let ei = image::open(dir.path().join("e.png")).unwrap();
            let bi = image::open(dir.path().join("b.png")).unwrap();
            if (ei.width(), ei.height()) != *want {
                bad.push(format!(
                    "{label} {w}x{h}: want {want:?}, got {}x{}",
                    ei.width(),
                    ei.height()
                ));
            }
            if ei.color() != bi.color() || ei.as_bytes() != bi.as_bytes() {
                bad.push(format!(
                    "{label} {w}x{h}: apply of the saved recipe differs from edit"
                ));
            }
            let got = std::fs::read_to_string(dir.path().join("r.toml")).unwrap();
            if got != **text {
                bad.push(format!("{label} {w}x{h}: saved {got:?}, want {text:?}"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} rows disagree:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn batch_edit_with_the_key_saves_one_recipe() {
    let dir = TempDir::new().unwrap();
    let folder = dir.path().join("in");
    std::fs::create_dir(&folder).unwrap();
    write_png(&folder, "a.png", 64, 48);
    write_png(&folder, "b.png", 48, 64);
    let out = dir.path().join("out");
    let rec = dir.path().join("r.toml");
    let r = run(&[
        "edit",
        folder.to_str().unwrap(),
        "--crop",
        "3:2",
        "--crop-orientation",
        "match",
        "--out-dir",
        out.to_str().unwrap(),
        "--save-recipe",
        rec.to_str().unwrap(),
    ]);
    assert_eq!(r.code, Some(0), "stderr: {}", r.stderr);
    assert_eq!(dims(&out.join("a.png")), (64, 43));
    assert_eq!(dims(&out.join("b.png")), (43, 64));
    assert_eq!(
        std::fs::read_to_string(&rec).unwrap(),
        saved("crop", &["aspect = \"3:2\"", "orientation = \"match\""])
    );
}

#[test]
fn edit_help_lists_the_orientation_flags() {
    let o = Command::new(BIN).args(["edit", "--help"]).output().unwrap();
    let help = String::from_utf8_lossy(&o.stdout);
    for want in [
        "--crop-orientation <MATCH>",
        "--pad-orientation <MATCH>",
        "Turn an aspect --crop W:H to follow each image",
        "Turn an aspect --pad W:H to follow each image",
    ] {
        assert!(help.contains(want), "edit --help lacks {want:?}:\n{help}");
    }
}
