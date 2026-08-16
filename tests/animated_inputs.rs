//! Integration tests for SPEC-119: an animated GIF/APNG/WebP is never
//! silently flattened to its first frame on `convert`/`optimize`/`web`.
//!
//! Drives the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! Fixtures are synthesized in memory via `tests/common` — no committed
//! binary files (AGENTS.md §12). A separate file from
//! `tests/hostile_inputs.rs` (which is about malformed/adversarial bytes):
//! every fixture here is a genuinely VALID, well-formed animated image —
//! this defect is about silent data loss on GOOD input, not hardening
//! against bad input.
//!
//! **AC-6's trap, restated as a comment because it is easy to forget while
//! writing a new test here**: SSIMULACRA2 compares decoded-source to output,
//! and both are frame 1 — the score this bug produces is HIGH, not low. Any
//! assertion here that leans on `diff`/a quality score would be vacuous by
//! construction. Every positive assertion below is structural: stderr text,
//! exit code, or a frame count from an independent decode.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

fn write_bytes(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Run one of the three affected verbs on `input`, writing to `out`.
/// `web`/`optimize` auto-decide the output format; `convert` is pinned to
/// WebP so every verb's output is independently decodable.
fn run_verb(verb: &str, input: &Path, out: &Path, extra: &[&str]) -> std::process::Output {
    let mut args: Vec<&str> = vec![verb, input.to_str().unwrap()];
    if verb == "convert" {
        args.extend(["--format", "webp"]);
    }
    args.extend(extra);
    args.extend(["-o", out.to_str().unwrap()]);
    Command::new(BIN)
        .args(&args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run crustyimg {verb}: {e}"))
}

const PIXEL_VERBS: &[&str] = &["convert", "optimize", "web"];

// ── AC-1 / AC-2: warns on every pixel verb, exit stays 0, frame 1 is written ──

#[test]
fn animated_gif_warns_on_every_pixel_verb() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "anim.gif", &common::animated_gif(4, 4));

    for verb in PIXEL_VERBS {
        let out = dir.path().join(format!("out_{verb}.bin"));
        let output = run_verb(verb, &input, &out, &[]);
        let stderr = stderr_str(&output);
        assert!(
            output.status.success(),
            "{verb}: must still exit 0 (AC-2); stderr:\n{stderr}"
        );
        assert!(
            stderr.contains("warning: ") && stderr.contains("anim.gif"),
            "{verb}: must warn on stderr, naming the input (AC-1); stderr:\n{stderr}"
        );
        assert!(
            stderr.contains("discarded"),
            "{verb}: the warning must say frames were discarded, not just that \
             SOMETHING happened (AC-1 — assert the message, not non-empty stderr); \
             stderr:\n{stderr}"
        );
        assert!(
            out.exists() && std::fs::metadata(&out).unwrap().len() > 0,
            "{verb}: frame 1 must still be written (AC-2)"
        );
    }
}

/// AC-3: not `--quiet`-gated (DEC-085's sibling) — pinned separately from
/// AC-1 because the adjacent cache warning in `build.rs` IS `--quiet`-gated,
/// which is exactly the trap a coarser test would miss
/// [[a-criterion-nobody-claims-is-a-criterion-nobody-checks]].
#[test]
fn animated_warning_survives_quiet() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "anim.gif", &common::animated_gif(4, 4));
    let out = dir.path().join("out.webp");

    let output = Command::new(BIN)
        .args([
            "convert",
            input.to_str().unwrap(),
            "--format",
            "webp",
            "--quiet",
            "-o",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run convert --quiet");
    assert!(output.status.success(), "stderr:\n{}", stderr_str(&output));
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("discarded"),
        "--quiet must not suppress the animated-input warning; stderr:\n{stderr}"
    );
}

/// AC-4: the did-not-break-it control. Without this, "always warn" would
/// pass AC-1 and ruin the verb
/// [[a-harness-that-exercises-nothing-reports-green]].
#[test]
fn static_gif_emits_no_animation_warning() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "still.gif", &common::static_gif(4, 4));
    let out = dir.path().join("out.webp");

    let output = run_verb("convert", &input, &out, &[]);
    assert!(output.status.success(), "stderr:\n{}", stderr_str(&output));
    let stderr = stderr_str(&output);
    assert!(
        !stderr.contains("discarded"),
        "a static GIF must never warn about discarded frames; stderr:\n{stderr}"
    );
}

// ── AC-5: APNG and animated WebP warn too, with their own static controls ────

#[test]
fn apng_warns_on_every_pixel_verb() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "anim.png", &common::animated_apng(4, 4));

    for verb in PIXEL_VERBS {
        let out = dir.path().join(format!("out_{verb}.bin"));
        let output = run_verb(verb, &input, &out, &[]);
        let stderr = stderr_str(&output);
        assert!(
            output.status.success(),
            "{verb}: must still exit 0; stderr:\n{stderr}"
        );
        assert!(
            stderr.contains("warning: ")
                && stderr.contains("anim.png")
                && stderr.contains("discarded"),
            "{verb}: must warn on the APNG, naming the input; stderr:\n{stderr}"
        );
    }
}

#[test]
fn animated_webp_warns_on_every_pixel_verb() {
    let dir = TempDir::new().unwrap();
    let input = write_bytes(&dir, "anim.webp", &common::animated_webp(4, 4));

    for verb in PIXEL_VERBS {
        let out = dir.path().join(format!("out_{verb}.bin"));
        let output = run_verb(verb, &input, &out, &[]);
        let stderr = stderr_str(&output);
        assert!(
            output.status.success(),
            "{verb}: must still exit 0; stderr:\n{stderr}"
        );
        assert!(
            stderr.contains("warning: ")
                && stderr.contains("anim.webp")
                && stderr.contains("discarded"),
            "{verb}: must warn on the animated WebP, naming the input; stderr:\n{stderr}"
        );
    }
}

/// AC-5's controls: a static PNG and a static (lossless) WebP must never
/// warn — the sibling of `static_gif_emits_no_animation_warning`, one per
/// newly covered format.
#[test]
fn static_png_and_static_webp_emit_no_animation_warning() {
    let dir = TempDir::new().unwrap();

    let png_in = write_bytes(&dir, "still.png", &common::solid_png(4, 4, [10, 20, 30]));
    let png_out = dir.path().join("still_out.webp");
    let png_result = run_verb("convert", &png_in, &png_out, &[]);
    assert!(
        png_result.status.success(),
        "stderr:\n{}",
        stderr_str(&png_result)
    );
    assert!(
        !stderr_str(&png_result).contains("discarded"),
        "a static PNG must never warn about discarded frames; stderr:\n{}",
        stderr_str(&png_result)
    );

    let webp_in = write_bytes(&dir, "still.webp", &common::webp_lossless(4, 4));
    let webp_out = dir.path().join("still_webp_out.webp");
    let webp_result = run_verb("convert", &webp_in, &webp_out, &[]);
    assert!(
        webp_result.status.success(),
        "stderr:\n{}",
        stderr_str(&webp_result)
    );
    assert!(
        !stderr_str(&webp_result).contains("discarded"),
        "a static WebP must never warn about discarded frames; stderr:\n{}",
        stderr_str(&webp_result)
    );
}

// ── AC-6: the assertion is structural, never the quality score ──────────────

/// Decodes the SOURCE independently (via `image`'s own `AnimationDecoder`,
/// not any crustyimg code path) to prove it really is multi-frame, then
/// decodes crustyimg's OUTPUT and asserts it carries exactly one frame.
/// Never asserts on a quality score — see the module doc's trap note.
#[test]
fn animated_output_frame_count_is_asserted_structurally() {
    let dir = TempDir::new().unwrap();
    let gif_bytes = common::animated_gif(4, 4);
    let input = write_bytes(&dir, "anim.gif", &gif_bytes);

    let source_frame_count = {
        use image::codecs::gif::GifDecoder;
        use image::AnimationDecoder;
        let dec = GifDecoder::new(std::io::Cursor::new(&gif_bytes[..])).unwrap();
        dec.into_frames().count()
    };
    assert_eq!(
        source_frame_count, 2,
        "fixture sanity check: the source must genuinely be 2 frames"
    );

    let out = dir.path().join("out.webp");
    let output = run_verb("convert", &input, &out, &[]);
    assert!(output.status.success(), "stderr:\n{}", stderr_str(&output));

    let out_bytes = std::fs::read(&out).unwrap();
    use image::codecs::webp::WebPDecoder;
    let dec = WebPDecoder::new(std::io::Cursor::new(&out_bytes)).expect("output must decode");
    assert!(
        !dec.has_animation(),
        "the flattened output must structurally carry no animation (AC-6) — \
         asserting a quality score here would be vacuous by construction, since \
         SSIMULACRA2 compares decoded-source (frame 1) to output (also frame 1)"
    );
}
