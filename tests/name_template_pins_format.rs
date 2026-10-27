//! A recognised literal extension in `--name-template` pins the output format
//! on every verb that encodes, exactly as a recognised `-o` extension does.
//!
//! One table drives every verb surface. Each row runs the real binary in a fresh
//! temp dir and identifies every written file by its MAGIC BYTES — never by its
//! name, because the name is exactly what is in question. All failing rows are
//! collected before the assertion, so one run reports every wrong row.
//!
//! The JPEG sources are built with the `image` crate's own encoder from a
//! committed photograph, never with crustyimg: a fixture made by the code under
//! test cannot fail.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use image::{DynamicImage, ImageFormat};
use tempfile::TempDir;

mod common;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// A plain pixel recipe: one `resize`, no `format`.
const PLAIN_RECIPE: &str =
    "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 64\n";

/// The same step on a recipe that declares its own `format`.
const WEBP_RECIPE: &str =
    "version = \"2\"\nformat = \"webp\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 64\n";

// ── Fixtures ──────────────────────────────────────────────────────────────────

/// A photograph-like JPEG: the committed colour photo, downscaled and encoded
/// by the `image` crate. Photographic content is what makes `web`'s automatic
/// choice pick AVIF (the gated guard below checks that before relying on it).
fn photo_jpeg() -> Vec<u8> {
    let png =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/classify/color_photo_fuji.png");
    let img = image::open(&png)
        .expect("the committed photo fixture decodes")
        .resize(224, 224, image::imageops::FilterType::Triangle);
    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img.to_rgb8())
        .write_to(&mut buf, ImageFormat::Jpeg)
        .unwrap();
    buf.into_inner()
}

/// A fresh working dir holding every source any row needs.
fn workdir(jpeg: &[u8]) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    std::fs::write(root.join("in.jpg"), jpeg).unwrap();
    std::fs::write(root.join("in2.jpg"), jpeg).unwrap();
    std::fs::write(root.join("a.png"), common::detailed_png(64, 64)).unwrap();
    std::fs::write(root.join("r.toml"), PLAIN_RECIPE).unwrap();
    std::fs::write(root.join("r2.toml"), WEBP_RECIPE).unwrap();
    dir
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("binary should run")
}

// ── Format sniffing (independent of crustyimg and of `image`) ─────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fmt {
    Jpeg,
    Png,
    WebP,
    Avif,
    Other,
}

fn sniff(bytes: &[u8]) -> Fmt {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Fmt::Jpeg
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        Fmt::Png
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Fmt::WebP
    } else if bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && matches!(&bytes[8..12], b"avif" | b"avis")
    {
        Fmt::Avif
    } else {
        Fmt::Other
    }
}

fn sniff_file(path: &Path) -> Option<Fmt> {
    std::fs::read(path).ok().map(|b| sniff(&b))
}

// ── The table ─────────────────────────────────────────────────────────────────

/// Where a row's output is checked.
#[derive(Clone, Copy)]
enum Out {
    /// A file under the working dir.
    File(&'static str, Fmt),
    /// The process's stdout.
    Stdout(Fmt),
}

struct Row {
    label: &'static str,
    args: &'static [&'static str],
    expect_exit: i32,
    expect: &'static [Out],
    expect_absent: &'static [&'static str],
    stderr_has: &'static [&'static str],
    stderr_lacks: Option<&'static str>,
}

use Fmt::{Jpeg, Png, WebP};
use Out::{File, Stdout};

/// The decision line every automatic choice prints (`in.jpg: jpeg → avif · …`).
const DECISION_ARROW: &str = "→";

const ROWS: &[Row] = &[
    // ── AC-1: the pixel verbs ────────────────────────────────────────────────
    Row {
        label: "AC-1 resize 1",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 resize N",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 thumbnail 1",
        args: &[
            "thumbnail",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 thumbnail N",
        args: &[
            "thumbnail",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 auto-orient 1",
        args: &[
            "auto-orient",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 auto-orient N",
        args: &[
            "auto-orient",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 watermark 1",
        args: &[
            "watermark",
            "--text",
            "hi",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 watermark N",
        args: &[
            "watermark",
            "--text",
            "hi",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-1 edit 1",
        args: &[
            "edit",
            "--invert",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-2: apply, plain recipe ────────────────────────────────────────────
    Row {
        label: "AC-2 apply 1",
        args: &[
            "apply",
            "--recipe",
            "r.toml",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-2 apply N",
        args: &[
            "apply",
            "--recipe",
            "r.toml",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-3: apply --recipe web (the automatic choice is skipped) ───────────
    Row {
        label: "AC-3 apply web 1",
        args: &[
            "apply",
            "--recipe",
            "web",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "AC-3 apply web N",
        args: &[
            "apply",
            "--recipe",
            "web",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    // ── AC-4: optimize ───────────────────────────────────────────────────────
    Row {
        label: "AC-4 optimize 1",
        args: &[
            "optimize",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "AC-4 optimize N",
        args: &[
            "optimize",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "AC-4 optimize --profile preserve 1",
        args: &[
            "optimize",
            "--profile",
            "preserve",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-5: web ────────────────────────────────────────────────────────────
    Row {
        label: "AC-5 web 1 .webp",
        args: &[
            "web",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.webp",
        ],
        expect_exit: 0,
        expect: &[File("d/in.webp", WebP)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "AC-5 web N .png",
        args: &[
            "web",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    // ── AC-6: --json has no automatic choice to report on a pin ──────────────
    Row {
        label: "AC-6 optimize --json 1",
        args: &[
            "optimize",
            "--json",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 2,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["unavailable"],
        stderr_lacks: None,
    },
    Row {
        label: "AC-6 web --json 1",
        args: &[
            "web",
            "--json",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 2,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["unavailable"],
        stderr_lacks: None,
    },
    Row {
        label: "AC-6 apply web --json 1",
        args: &[
            "apply",
            "--recipe",
            "web",
            "--json",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 2,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["unavailable"],
        stderr_lacks: None,
    },
    // ── AC-7: --format beats the template ────────────────────────────────────
    Row {
        label: "AC-7 resize --format webp 1",
        args: &[
            "resize",
            "--max",
            "64",
            "--format",
            "webp",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", WebP)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-7 apply --format webp N",
        args: &[
            "apply",
            "--recipe",
            "r.toml",
            "--format",
            "webp",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", WebP), File("d/in2.png", WebP)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-7 convert --format webp 1",
        args: &[
            "convert",
            "--format",
            "webp",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", WebP)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-7 convert --format webp N",
        args: &[
            "convert",
            "--format",
            "webp",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", WebP), File("d/in2.png", WebP)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-8: the template beats recipe.format ───────────────────────────────
    Row {
        label: "AC-8 apply r2 1",
        args: &[
            "apply",
            "--recipe",
            "r2.toml",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-8 apply r2 N",
        args: &[
            "apply",
            "--recipe",
            "r2.toml",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-9: -o beats the template (the template is not the output's name) ──
    Row {
        label: "AC-9 resize -o o.jpg",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "-o",
            "o.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("o.jpg", Jpeg)],
        expect_absent: &["d"],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-9 web -o o.jpg",
        args: &[
            "web",
            "in.jpg",
            "-o",
            "o.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("o.jpg", Jpeg)],
        expect_absent: &["d/in.png"],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-11: no recognised literal extension names the file only ───────────
    Row {
        label: "AC-11 resize {stem}.txt",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.txt",
        ],
        expect_exit: 0,
        expect: &[File("d/in.txt", Jpeg)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-11 resize {stem}",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}",
        ],
        expect_exit: 0,
        expect: &[File("d/in", Jpeg)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-11 resize --format png {name}",
        args: &[
            "resize",
            "--max",
            "64",
            "--format",
            "png",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{name}",
        ],
        expect_exit: 0,
        expect: &[File("d/in.jpg", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-11 resize 1 no --out-dir (stdout)",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[Stdout(Jpeg)],
        expect_absent: &["in.png"],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-11 resize N {stem}.{ext}",
        args: &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.{ext}",
        ],
        expect_exit: 0,
        expect: &[File("d/in.jpg", Jpeg), File("d/in2.jpg", Jpeg)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── AC-12: meta refuses a template naming another format ─────────────────
    Row {
        label: "AC-12 meta strip 1",
        args: &[
            "meta",
            "strip",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 4,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["in.jpg", "PNG", "JPEG", "convert"],
        stderr_lacks: None,
    },
    Row {
        label: "AC-12 meta clean --gps 1",
        args: &[
            "meta",
            "clean",
            "--gps",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 4,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["in.jpg", "PNG", "JPEG", "convert"],
        stderr_lacks: None,
    },
    Row {
        label: "AC-12 meta set --artist 1",
        args: &[
            "meta",
            "set",
            "--artist",
            "x",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 4,
        expect: &[],
        expect_absent: &["d/in.png"],
        stderr_has: &["in.jpg", "PNG", "JPEG", "convert"],
        stderr_lacks: None,
    },
    // ── AC-13: the batch is refused whole, before any write ──────────────────
    Row {
        label: "AC-13 meta strip N",
        args: &[
            "meta",
            "strip",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 4,
        expect: &[],
        expect_absent: &["d/in.png", "d/in2.png"],
        stderr_has: &["convert"],
        stderr_lacks: None,
    },
    Row {
        label: "AC-13 meta strip N mixed (a.png first)",
        args: &[
            "meta",
            "strip",
            "a.png",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 4,
        expect: &[],
        expect_absent: &["d/a.png", "d/in.png"],
        stderr_has: &["in.jpg", "convert"],
        stderr_lacks: None,
    },
    // ── AC-14: same format, or {ext}, is unchanged ───────────────────────────
    Row {
        label: "AC-14 meta strip {stem}.jpg",
        args: &[
            "meta",
            "strip",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.jpg",
        ],
        expect_exit: 0,
        expect: &[File("d/in.jpg", Jpeg)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "AC-14 meta strip {stem}_c.{ext}",
        args: &[
            "meta",
            "strip",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}_c.{ext}",
        ],
        expect_exit: 0,
        expect: &[File("d/in_c.jpg", Jpeg)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: None,
    },
    // ── N inputs ignore -o: the pin reads the template (the name written) ─────
    Row {
        label: "N-input apply web -o x.jpg",
        args: &[
            "apply",
            "--recipe",
            "web",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input web -o x.jpg",
        args: &[
            "web",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input apply web -o x.bin",
        args: &[
            "apply",
            "--recipe",
            "web",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.bin",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input web -o x.bin",
        args: &[
            "web",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.bin",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input optimize -o x.bin",
        args: &[
            "optimize",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.bin",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &[],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    // ── N-input pin where ONE glob argument resolves to several inputs ───────
    Row {
        label: "N-input glob web -o x.bin",
        args: &[
            "web",
            "in*.jpg",
            "-o",
            "x.bin",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &["x.bin"],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input glob apply web -o x.jpg",
        args: &[
            "apply",
            "--recipe",
            "web",
            "in*.jpg",
            "-o",
            "x.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &["x.jpg"],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input glob optimize -o x.bin",
        args: &[
            "optimize",
            "in*.jpg",
            "-o",
            "x.bin",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.png",
        ],
        expect_exit: 0,
        expect: &[File("d/in.png", Png), File("d/in2.png", Png)],
        expect_absent: &["x.bin"],
        stderr_has: &[],
        stderr_lacks: Some(DECISION_ARROW),
    },
    Row {
        label: "N-input meta strip -o x.png ignored",
        args: &[
            "meta",
            "strip",
            "in.jpg",
            "in2.jpg",
            "-o",
            "x.png",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.jpg",
        ],
        expect_exit: 0,
        expect: &[File("d/in.jpg", Jpeg), File("d/in2.jpg", Jpeg)],
        expect_absent: &["x.png"],
        stderr_has: &[],
        stderr_lacks: None,
    },
    Row {
        label: "N-input glob meta strip -o x.png ignored",
        args: &[
            "meta",
            "strip",
            "in*.jpg",
            "-o",
            "x.png",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.jpg",
        ],
        expect_exit: 0,
        expect: &[File("d/in.jpg", Jpeg), File("d/in2.jpg", Jpeg)],
        expect_absent: &["x.png"],
        stderr_has: &[],
        stderr_lacks: None,
    },
];

/// Run one row; `Some(report)` when it is wrong.
fn check_row(row: &Row, jpeg: &[u8]) -> Option<String> {
    let dir = workdir(jpeg);
    let root = dir.path();
    let out = run(root, row.args);
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let mut problems: Vec<String> = Vec::new();

    if out.status.code() != Some(row.expect_exit) {
        problems.push(format!(
            "exit {:?}, expected {}",
            out.status.code(),
            row.expect_exit
        ));
    }
    for expected in row.expect {
        match *expected {
            File(rel, fmt) => match sniff_file(&root.join(rel)) {
                Some(got) if got == fmt => {}
                Some(got) => problems.push(format!("{rel}: {got:?}, expected {fmt:?}")),
                None => problems.push(format!("{rel}: not written, expected {fmt:?}")),
            },
            Stdout(fmt) => {
                let got = sniff(&out.stdout);
                if got != fmt {
                    problems.push(format!("stdout: {got:?}, expected {fmt:?}"));
                }
            }
        }
    }
    for rel in row.expect_absent {
        let path: PathBuf = root.join(rel);
        if path.exists() {
            problems.push(format!(
                "{rel}: exists ({:?}), expected absent",
                sniff_file(&path)
            ));
        }
    }
    for needle in row.stderr_has {
        if !stderr.contains(needle) {
            problems.push(format!("stderr lacks {needle:?}"));
        }
    }
    if let Some(needle) = row.stderr_lacks {
        if stderr.contains(needle) {
            problems.push(format!("stderr has {needle:?}"));
        }
    }
    if stderr.contains("panicked") {
        problems.push("panicked".to_owned());
    }

    if problems.is_empty() {
        None
    } else {
        Some(format!(
            "✗ {}\n    args: {:?}\n    {}\n    stderr: {}",
            row.label,
            row.args,
            problems.join("\n    "),
            stderr.trim()
        ))
    }
}

/// AC-1 through AC-14 (except AC-10's byte-identity, below): one rule over
/// every verb surface. Rows run in parallel; every failing row is reported.
#[test]
fn name_template_extension_pins_the_format_on_every_verb() {
    assert_eq!(
        ROWS.len(),
        51,
        "the table's row count is part of the contract"
    );

    let jpeg = photo_jpeg();
    assert_eq!(sniff(&jpeg), Jpeg, "the fixture is a JPEG");

    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .clamp(1, 8);
    let chunk = ROWS.len().div_ceil(workers);
    let mut failures: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = ROWS
            .chunks(chunk)
            .map(|rows| {
                let jpeg = &jpeg;
                s.spawn(move || {
                    rows.iter()
                        .filter_map(|row| check_row(row, jpeg))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("a row worker panicked"))
            .collect()
    });
    failures.sort();
    assert!(
        failures.is_empty(),
        "{} of {} rows wrong:\n{}",
        failures.len(),
        ROWS.len(),
        failures.join("\n")
    );
}

// ── AC-10: {ext} is byte-identical to no template ─────────────────────────────

/// Run `args` (an output under `d`) with and without a template; assert the
/// named outputs are byte-identical.
fn assert_same_bytes(jpeg: &[u8], args: &[&str], template: &str, pairs: &[(&str, &str)]) {
    let plain = workdir(jpeg);
    let templated = workdir(jpeg);
    let a = run(plain.path(), args);
    assert_eq!(
        a.status.code(),
        Some(0),
        "{args:?}: {}",
        String::from_utf8_lossy(&a.stderr)
    );
    let mut targs: Vec<&str> = args.to_vec();
    targs.extend(["--name-template", template]);
    let b = run(templated.path(), &targs);
    assert_eq!(
        b.status.code(),
        Some(0),
        "{targs:?}: {}",
        String::from_utf8_lossy(&b.stderr)
    );
    for (default_rel, templated_rel) in pairs {
        let want = std::fs::read(plain.path().join(default_rel))
            .unwrap_or_else(|e| panic!("{args:?}: {default_rel} missing: {e}"));
        let got = std::fs::read(templated.path().join(templated_rel))
            .unwrap_or_else(|e| panic!("{targs:?}: {templated_rel} missing: {e}"));
        assert!(
            want == got,
            "{targs:?}: {templated_rel} differs from the no-template {default_rel}"
        );
    }
}

#[test]
fn name_template_ext_placeholder_is_byte_identical_to_the_default() {
    let jpeg = photo_jpeg();
    assert_same_bytes(
        &jpeg,
        &[
            "resize",
            "--max",
            "64",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
        ],
        "{stem}.{ext}",
        &[("d/in.jpg", "d/in.jpg"), ("d/in2.jpg", "d/in2.jpg")],
    );
    assert_same_bytes(
        &jpeg,
        &[
            "apply",
            "--recipe",
            "r.toml",
            "in.jpg",
            "in2.jpg",
            "--out-dir",
            "d",
        ],
        "{stem}.{ext}",
        &[("d/in.jpg", "d/in.jpg"), ("d/in2.jpg", "d/in2.jpg")],
    );
    assert_same_bytes(
        &jpeg,
        &["meta", "strip", "in.jpg", "--out-dir", "d"],
        "{stem}.{ext}",
        &[("d/in.jpg", "d/in.jpg")],
    );
}

/// The automatic choice survives `{ext}` and an unrecognised extension. Needs
/// the AVIF encoder, and no lossy WebP (which can win the choice instead).
#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
#[test]
fn name_template_ext_placeholder_keeps_the_automatic_choice() {
    let jpeg = photo_jpeg();

    // Prove the fixture first: with no template, `web` picks AVIF.
    let control = workdir(&jpeg);
    let out = run(control.path(), &["web", "in.jpg", "--out-dir", "d"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        sniff_file(&control.path().join("d/in.avif")),
        Some(Fmt::Avif),
        "the fixture must auto-decide to AVIF for this guard to mean anything"
    );

    // web N with {stem}_w.{ext}: AVIF, byte-identical to the no-template run.
    assert_same_bytes(
        &jpeg,
        &["web", "in.jpg", "in2.jpg", "--out-dir", "d"],
        "{stem}_w.{ext}",
        &[("d/in.avif", "d/in_w.avif"), ("d/in2.avif", "d/in2_w.avif")],
    );

    // apply --recipe web 1 with {stem}.{ext}: AVIF, and the decision line prints.
    let dir = workdir(&jpeg);
    let out = run(
        dir.path(),
        &[
            "apply",
            "--recipe",
            "web",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.{ext}",
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "{stderr}");
    assert_eq!(sniff_file(&dir.path().join("d/in.avif")), Some(Fmt::Avif));
    assert!(stderr.contains("jpeg → avif"), "stderr: {stderr}");

    // optimize --json with {stem}.{ext} still reports the decision.
    let dir = workdir(&jpeg);
    let out = run(
        dir.path(),
        &[
            "optimize",
            "--json",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.{ext}",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // web with an unrecognised extension names the file only: AVIF in in.txt.
    let dir = workdir(&jpeg);
    let out = run(
        dir.path(),
        &[
            "web",
            "in.jpg",
            "--out-dir",
            "d",
            "--name-template",
            "{stem}.txt",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sniff_file(&dir.path().join("d/in.txt")), Some(Fmt::Avif));
}

// ── AC-22: meta refuses an -o naming another format ───────────────────────────

#[test]
fn meta_dash_o_naming_another_format_refuses() {
    let jpeg = photo_jpeg();
    let dir = workdir(&jpeg);
    let out = run(dir.path(), &["meta", "strip", "in.jpg", "-o", "E.png"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "stderr: {stderr}");
    assert!(
        !dir.path().join("E.png").exists(),
        "a refused meta write must write nothing (got {:?})",
        sniff_file(&dir.path().join("E.png"))
    );
    for needle in ["in.jpg", "PNG", "JPEG", "convert"] {
        assert!(stderr.contains(needle), "stderr lacks {needle:?}: {stderr}");
    }
}

#[test]
fn meta_dash_o_same_or_unrecognised_extension_is_unchanged() {
    let jpeg = photo_jpeg();
    for name in ["E.jpg", "E.bin"] {
        let dir = workdir(&jpeg);
        let out = run(dir.path(), &["meta", "strip", "in.jpg", "-o", name]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            sniff_file(&dir.path().join(name)),
            Some(Fmt::Jpeg),
            "{name}"
        );
    }
}
