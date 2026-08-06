//! Integration tests for SPEC-110: EXIF Orientation is baked into pixels on
//! EVERY pixel-lane verb, not just `web`/`optimize`/`auto-orient`.
//!
//! All tests drive the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are synthesized in memory via `tests/common` — no committed binary
//! files (AGENTS.md §12).
//!
//! Every fixture here is deliberately NON-SQUARE (AC-4): a square input cannot
//! distinguish "baked" from "not baked" from "baked twice" (a 90°/270°
//! rotation is invisible on a square), which would make the whole spec
//! vacuous.

use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// The design's measured fixture: a JPEG stored 1200×800 with a real
/// one-entry `Orientation=6` IFD. Orientation 6 means the correct DISPLAY
/// size is 800×1200 — the exact numbers the spec's Context table uses, kept
/// here so test failures are directly comparable to that table.
const W: u32 = 1200;
const H: u32 = 800;
const ORIENT_6: u8 = 6;

fn write_bytes(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_dims(path: &std::path::Path, want_w: u32, want_h: u32, label: &str) {
    let decoded = image::open(path).unwrap_or_else(|e| panic!("{label}: not decodable: {e}"));
    assert_eq!(
        decoded.width(),
        want_w,
        "{label}: width mismatch (got {}x{}, want {}x{})",
        decoded.width(),
        decoded.height(),
        want_w,
        want_h
    );
    assert_eq!(
        decoded.height(),
        want_h,
        "{label}: height mismatch (got {}x{}, want {}x{})",
        decoded.width(),
        decoded.height(),
        want_w,
        want_h
    );
}

// ── AC-1: the seven previously-wrong verbs now bake ──────────────────────────

/// `convert --format png` and `convert --format jpeg` on a 1200×800
/// `Orientation=6` source both come back 800×1200 — the display-correct
/// dimensions. Fails today (1200×800, unbaked).
#[test]
fn convert_bakes_orientation_into_pixels() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );

    for fmt in ["png", "jpeg"] {
        let out = dir.path().join(format!("out.{fmt}"));
        let output = Command::new(BIN)
            .args([
                "convert",
                input.to_str().unwrap(),
                "--format",
                fmt,
                "-o",
                out.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run convert");
        assert_eq!(
            output.status.code(),
            Some(0),
            "convert --format {fmt} should exit 0; stderr: {}",
            stderr_str(&output)
        );
        assert_dims(&out, 800, 1200, &format!("convert --format {fmt}"));
    }
}

/// `resize --max 600` on the same source comes back 400×600 — NOT 600×400.
/// This is the worst case for users: today the `--max` bound is applied to
/// the WRONG AXIS, so the output isn't just mis-tagged, it's the wrong size.
#[test]
fn resize_bakes_orientation_before_bounding() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "resize",
            input.to_str().unwrap(),
            "--max",
            "600",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run resize");
    assert_eq!(
        output.status.code(),
        Some(0),
        "resize --max 600 should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert_dims(&out, 400, 600, "resize --max 600");
}

/// `thumbnail --size 300` on the same source comes back 200×300.
#[test]
fn thumbnail_bakes_orientation() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );
    let out = dir.path().join("out.jpg");

    let output = Command::new(BIN)
        .args([
            "thumbnail",
            input.to_str().unwrap(),
            "--size",
            "300",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run thumbnail");
    assert_eq!(
        output.status.code(),
        Some(0),
        "thumbnail --size 300 should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert_dims(&out, 200, 300, "thumbnail --size 300");
}

/// `responsive --widths 600` on the same source comes back 600×900 — width
/// PINNED, not a plain dimension swap. A test copied from the `resize` case
/// (asserting 400×600) would assert the WRONG thing here: `responsive`
/// compares the requested width against the (now oriented) source width
/// (800, not 1200) and scales height to match, landing on 900, not 400.
#[test]
fn responsive_bakes_orientation() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );
    let out_dir = dir.path().join("out");
    std::fs::create_dir(&out_dir).unwrap();

    let output = Command::new(BIN)
        .args([
            "responsive",
            input.to_str().unwrap(),
            "--widths",
            "600",
            "--out-dir",
            out_dir.to_str().unwrap(),
            "--no-snippet",
        ])
        .output()
        .expect("failed to run responsive");
    assert_eq!(
        output.status.code(),
        Some(0),
        "responsive --widths 600 should exit 0; stderr: {}",
        stderr_str(&output)
    );

    let variant = out_dir.join("in-600w.jpg");
    assert!(
        variant.exists(),
        "expected variant {variant:?} to exist; out_dir contents: {:?}",
        std::fs::read_dir(&out_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>()
    );
    assert_dims(&variant, 600, 900, "responsive --widths 600");
}

/// `edit --invert` (no `--auto-orient` flag) bakes orientation to 800×1200;
/// `edit --resize-max 600` (no `--auto-orient` flag) bakes-then-bounds to
/// 400×600. Neither flag combination opts into orientation handling anymore —
/// every `edit` invocation bakes unconditionally.
#[test]
fn edit_bakes_orientation_without_the_flag() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );

    let out_invert = dir.path().join("invert.jpg");
    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--invert",
            "-o",
            out_invert.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run edit --invert");
    assert_eq!(
        output.status.code(),
        Some(0),
        "edit --invert should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert_dims(&out_invert, 800, 1200, "edit --invert");

    let out_resize = dir.path().join("resize.jpg");
    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--resize-max",
            "600",
            "-o",
            out_resize.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run edit --resize-max");
    assert_eq!(
        output.status.code(),
        Some(0),
        "edit --resize-max 600 should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert_dims(&out_resize, 400, 600, "edit --resize-max 600");
}

/// `watermark --text hi` (no `--image`/`--auto-orient` flag — `watermark` has
/// neither) on the same source comes back 800×1200. Not part of AC-1's
/// measured table (`watermark` was the punch-list's missed pixel-lane site —
/// verify's 17-subcommand classification caught it after the build's own
/// `Pipeline`-construction sweep missed it), but the identical shape: a
/// `Pipeline::new().push(...)` site that skipped the shared prefix.
#[test]
fn watermark_bakes_orientation() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "watermark",
            input.to_str().unwrap(),
            "--text",
            "hi",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run watermark");
    assert_eq!(
        output.status.code(),
        Some(0),
        "watermark --text hi should exit 0; stderr: {}",
        stderr_str(&output)
    );
    assert_dims(&out, 800, 1200, "watermark --text hi");
}

// ── AC-2: the four already-correct verbs stay correct (no double rotation) ──

/// `web`, `optimize`, `auto-orient`, and `edit --auto-orient` all already
/// baked orientation before this spec — they must still produce 800×1200,
/// not 1200×800 (unbaked) or a 180°-off result (baked TWICE). This is the
/// over-correction guard: the obvious failure mode of adding a shared prefix
/// on top of `optimize_pipeline()`'s existing push. `-o out.png` pins the
/// output format so `web`/`optimize` skip their auto-decision entirely —
/// dimensions are what this test cares about, not the format choice.
#[test]
fn already_correct_verbs_do_not_double_rotate() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(
        &dir,
        "in.jpg",
        &common::jpeg_with_orientation(W, H, ORIENT_6),
    );

    let cases: &[&[&str]] = &[
        &["web"],
        &["optimize"],
        &["auto-orient"],
        &["edit", "--auto-orient"],
    ];

    for case in cases {
        let out = dir.path().join(format!("{}.png", case.join("_")));
        let mut args: Vec<&str> = case.to_vec();
        args.push(input.to_str().unwrap());
        args.push("-o");
        args.push(out.to_str().unwrap());

        let output = Command::new(BIN)
            .args(&args)
            .output()
            .unwrap_or_else(|e| panic!("failed to run {case:?}: {e}"));
        assert_eq!(
            output.status.code(),
            Some(0),
            "{case:?} should exit 0; stderr: {}",
            stderr_str(&output)
        );
        assert_dims(&out, 800, 1200, &format!("{case:?}"));
    }
}

// ── AC-3: orientation 1 / no EXIF is a genuine no-op (byte-identical) ───────

/// A source with NO EXIF at all and the same source wrapped with an explicit
/// `Orientation=1` (`NoTransforms`) tag must produce BYTE-IDENTICAL output on
/// every affected verb — not just matching dimensions. `AutoOrient::apply`
/// no-ops on both `None` and `Some(NoTransforms)`, returning the input image
/// completely unchanged, so re-encoding either source must land on the exact
/// same bytes. This is what makes baking safe for the overwhelming majority
/// of real-world inputs (most photos carry orientation 1 or no tag at all).
#[test]
fn orientation_1_and_no_exif_are_byte_identical() {
    let dir = TempDir::new().unwrap();
    // Same pixels, non-square (AC-4); one has no EXIF, the other an explicit
    // orientation=1 (NoTransforms) tag.
    let base = common::gradient_jpeg(300, 200);
    let orientation_1 = common::wrap_with_orientation_app1(&base, 1);

    let no_exif_in = write_bytes(&dir, "no_exif.jpg", &base);
    let o1_in = write_bytes(&dir, "o1.jpg", &orientation_1);

    // convert --format png
    {
        let a = dir.path().join("convert_a.png");
        let b = dir.path().join("convert_b.png");
        for (input, out) in [(&no_exif_in, &a), (&o1_in, &b)] {
            let output = Command::new(BIN)
                .args([
                    "convert",
                    input.to_str().unwrap(),
                    "--format",
                    "png",
                    "-o",
                    out.to_str().unwrap(),
                ])
                .output()
                .expect("failed to run convert");
            assert_eq!(output.status.code(), Some(0), "convert should exit 0");
        }
        assert_eq!(
            std::fs::read(&a).unwrap(),
            std::fs::read(&b).unwrap(),
            "convert: orientation=1 output must be byte-identical to no-EXIF output"
        );
    }

    // resize --max 150
    {
        let a = dir.path().join("resize_a.jpg");
        let b = dir.path().join("resize_b.jpg");
        for (input, out) in [(&no_exif_in, &a), (&o1_in, &b)] {
            let output = Command::new(BIN)
                .args([
                    "resize",
                    input.to_str().unwrap(),
                    "--max",
                    "150",
                    "-o",
                    out.to_str().unwrap(),
                ])
                .output()
                .expect("failed to run resize");
            assert_eq!(output.status.code(), Some(0), "resize should exit 0");
        }
        assert_eq!(
            std::fs::read(&a).unwrap(),
            std::fs::read(&b).unwrap(),
            "resize: orientation=1 output must be byte-identical to no-EXIF output"
        );
    }

    // thumbnail --size 100
    {
        let a = dir.path().join("thumb_a.jpg");
        let b = dir.path().join("thumb_b.jpg");
        for (input, out) in [(&no_exif_in, &a), (&o1_in, &b)] {
            let output = Command::new(BIN)
                .args([
                    "thumbnail",
                    input.to_str().unwrap(),
                    "--size",
                    "100",
                    "-o",
                    out.to_str().unwrap(),
                ])
                .output()
                .expect("failed to run thumbnail");
            assert_eq!(output.status.code(), Some(0), "thumbnail should exit 0");
        }
        assert_eq!(
            std::fs::read(&a).unwrap(),
            std::fs::read(&b).unwrap(),
            "thumbnail: orientation=1 output must be byte-identical to no-EXIF output"
        );
    }

    // edit --invert
    {
        let a = dir.path().join("edit_a.jpg");
        let b = dir.path().join("edit_b.jpg");
        for (input, out) in [(&no_exif_in, &a), (&o1_in, &b)] {
            let output = Command::new(BIN)
                .args([
                    "edit",
                    input.to_str().unwrap(),
                    "--invert",
                    "-o",
                    out.to_str().unwrap(),
                ])
                .output()
                .expect("failed to run edit");
            assert_eq!(output.status.code(), Some(0), "edit should exit 0");
        }
        assert_eq!(
            std::fs::read(&a).unwrap(),
            std::fs::read(&b).unwrap(),
            "edit: orientation=1 output must be byte-identical to no-EXIF output"
        );
    }

    // responsive --widths 150
    {
        let dir_a = dir.path().join("resp_a");
        let dir_b = dir.path().join("resp_b");
        std::fs::create_dir(&dir_a).unwrap();
        std::fs::create_dir(&dir_b).unwrap();
        for (input, out_dir) in [(&no_exif_in, &dir_a), (&o1_in, &dir_b)] {
            let output = Command::new(BIN)
                .args([
                    "responsive",
                    input.to_str().unwrap(),
                    "--widths",
                    "150",
                    "--out-dir",
                    out_dir.to_str().unwrap(),
                    "--no-snippet",
                ])
                .output()
                .expect("failed to run responsive");
            assert_eq!(output.status.code(), Some(0), "responsive should exit 0");
        }
        // Both inputs share the same file stem ("no_exif"/"o1" differ) —
        // compare by the actual variant each produced (there is exactly one).
        let variant_a = std::fs::read_dir(&dir_a)
            .unwrap()
            .next()
            .expect("dir_a should have one variant")
            .unwrap()
            .path();
        let variant_b = std::fs::read_dir(&dir_b)
            .unwrap()
            .next()
            .expect("dir_b should have one variant")
            .unwrap()
            .path();
        assert_eq!(
            std::fs::read(variant_a).unwrap(),
            std::fs::read(variant_b).unwrap(),
            "responsive: orientation=1 output must be byte-identical to no-EXIF output"
        );
    }
}

// ── AC-5: all eight orientation values, driven through one verb ─────────────

/// A source whose top-left QUADRANT is solid black and the rest solid white —
/// a content-level marker, independent of `common::gradient_jpeg` (which
/// varies only along X, so it cannot distinguish orientation 4's
/// vertical-only flip from a no-op: both leave a horizontal-only gradient's
/// values unchanged). Local to this test; not shared via `tests/common`
/// since no other test needs it.
fn quadrant_marker_jpeg(w: u32, h: u32) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let black = x < w / 2 && y < h / 2;
        *px = image::Rgb(if black { [0, 0, 0] } else { [255, 255, 255] });
    }
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut out, image::ImageFormat::Jpeg)
        .unwrap();
    out.into_inner()
}

/// Where the source's top-left quadrant marker lands in the DISPLAY-CORRECT
/// output, derived directly from the EXIF orientation spec's transform
/// definitions (not from crustyimg's implementation): 1=normal, 2=mirror
/// horizontal, 3=rotate 180, 4=mirror vertical, 5=transpose, 6=rotate 90 CW,
/// 7=transverse, 8=rotate 270 CW. Working through where a top-left source
/// point (small x, small y) lands under each transform gives a repeating
/// TL/TR/BR/BL cycle across {1,2,3,4} and again across {5,6,7,8} (the second
/// cycle in the swapped-dimension frame) — independent of AC-5's own
/// dimension-swap check, which this corner check complements rather than
/// duplicates.
#[derive(Clone, Copy, Debug)]
enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl Corner {
    fn for_orientation(o: u8) -> Self {
        match o {
            1 | 5 => Corner::TopLeft,
            2 | 6 => Corner::TopRight,
            3 | 7 => Corner::BottomRight,
            4 | 8 => Corner::BottomLeft,
            _ => unreachable!("orientation values are 1..=8"),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Corner::TopLeft => Corner::BottomRight,
            Corner::TopRight => Corner::BottomLeft,
            Corner::BottomRight => Corner::TopLeft,
            Corner::BottomLeft => Corner::TopRight,
        }
    }

    /// A sample point `inset` pixels in from this corner — far enough from
    /// the black/white boundary to be unaffected by JPEG block ringing on
    /// the (lossy) SOURCE encode.
    fn sample_xy(self, w: u32, h: u32, inset: u32) -> (u32, u32) {
        match self {
            Corner::TopLeft => (inset, inset),
            Corner::TopRight => (w - 1 - inset, inset),
            Corner::BottomRight => (w - 1 - inset, h - 1 - inset),
            Corner::BottomLeft => (inset, h - 1 - inset),
        }
    }
}

/// All eight EXIF orientation values (1–8), driven through `convert --format
/// png`. 5/6/7/8 imply a 90°/270° rotation and swap dimensions; 1/2/3/4 are
/// axis-preserving (identity/flip/180°) and do not. The current (unbaked)
/// code applies none of them — a fix that handles only 6 is not a fix.
///
/// Dimensions alone are vacuous for 1/2/3/4 (an UNBAKED build also produces
/// the source's own W×H for all four — four of eight assertions could never
/// fail). The quadrant-marker corner check below is a genuine content-level
/// assertion for every one of the eight: it fails if a flip/rotation is
/// silently skipped OR applied to the wrong axis/direction, not just if the
/// output has the wrong shape.
#[test]
fn all_eight_orientation_values_are_applied() {
    let dir = TempDir::new().unwrap();
    const BW: u32 = 40;
    const BH: u32 = 30;
    const INSET: u32 = 3;

    for o in 1u8..=8 {
        let input = write_bytes(
            &dir,
            &format!("in_{o}.jpg"),
            &common::wrap_with_orientation_app1(&quadrant_marker_jpeg(BW, BH), o),
        );
        let out = dir.path().join(format!("out_{o}.png"));

        let output = Command::new(BIN)
            .args([
                "convert",
                input.to_str().unwrap(),
                "--format",
                "png",
                "-o",
                out.to_str().unwrap(),
            ])
            .output()
            .unwrap_or_else(|e| panic!("failed to run convert for orientation {o}: {e}"));
        assert_eq!(
            output.status.code(),
            Some(0),
            "orientation {o} should exit 0; stderr: {}",
            stderr_str(&output)
        );

        let swaps = matches!(o, 5..=8);
        let (want_w, want_h) = if swaps { (BH, BW) } else { (BW, BH) };
        assert_dims(&out, want_w, want_h, &format!("orientation {o}"));

        let decoded = image::open(&out)
            .unwrap_or_else(|e| panic!("orientation {o}: not decodable: {e}"))
            .to_rgb8();
        let (ow, oh) = decoded.dimensions();
        let dark_corner = Corner::for_orientation(o);
        let light_corner = dark_corner.opposite();
        let (dx, dy) = dark_corner.sample_xy(ow, oh, INSET);
        let (lx, ly) = light_corner.sample_xy(ow, oh, INSET);
        let avg = |p: &image::Rgb<u8>| (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3;
        let dark_avg = avg(decoded.get_pixel(dx, dy));
        let light_avg = avg(decoded.get_pixel(lx, ly));
        assert!(
            dark_avg < 80,
            "orientation {o}: expected the baked quadrant marker at {dark_corner:?} \
             ({dx},{dy}) to be dark, got avg {dark_avg}"
        );
        assert!(
            light_avg > 175,
            "orientation {o}: expected {light_corner:?} ({lx},{ly}) to stay light, \
             got avg {light_avg}"
        );
    }
}

// ── AC-6: the frozen `--auto-orient` flag stays valid, exits 0 ──────────────

/// `edit --auto-orient` still parses and exits 0 — the CLI surface is frozen
/// (STAGE-030), so the flag cannot be removed even though it is now an
/// accepted no-op (every `edit` invocation already bakes orientation).
#[test]
fn edit_auto_orient_flag_still_exits_zero() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "in.png", &common::solid_png(8, 4, [10, 20, 30]));
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .args([
            "edit",
            input.to_str().unwrap(),
            "--auto-orient",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run edit --auto-orient");
    assert_eq!(
        output.status.code(),
        Some(0),
        "edit --auto-orient should exit 0; stderr: {}",
        stderr_str(&output)
    );
}
