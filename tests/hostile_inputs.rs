//! Native CLI harness for the committed hostile/edge input corpus (SPEC-107).
//!
//! Mirrors `tests/fuzz_regressions.rs`'s shape at the BINARY level: that file
//! sweeps the never-panics contract through `Image::from_bytes`/`raw_preview`
//! in-process; this drives the real compiled `crustyimg` binary end-to-end and
//! asserts on exit code + stderr — the surface an actual user (or an HN visitor
//! feeding the live demo garbage) sees. `tests/fixtures/hostile/` is a separate,
//! hand-built corpus from `tests/fixtures/fuzz/`'s fuzzer-found crash
//! reproducers (DEC-062's provenance rule is one file per fuzzer finding; these
//! are constructed edge cases) — see `tests/fixtures/hostile/README.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

mod common;

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// Absolute path to the committed hostile corpus, robust to the test binary's
/// working directory.
fn hostile_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hostile")
}

fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

/// One fixture's declared contract. The expected code is hardcoded here, NOT
/// derived from running the binary — a fixture that can only "fail" by
/// asserting against its own measured output can never fail
/// [[fixtures-from-the-code-under-test-cannot-fail]].
struct Expectation {
    file: &'static str,
    exit: i32,
    /// A substring that MUST appear on stderr — one of OUR error prefixes
    /// (`could not decode image:`, `image exceeds decode limits:`,
    /// `unsupported or undetectable image format`) or the F1 truncation
    /// warning, never upstream decoder wording (Notes: "do not assert on
    /// upstream decoder wording").
    stderr_contains: &'static str,
}

// AC-1 roster, each ≤ 4 KB (see README.md for provenance/build recipe).
const EXPECTATIONS: &[Expectation] = &[
    Expectation {
        file: "zero_byte.png",
        exit: 4,
        stderr_contains: "unsupported or undetectable image format",
    },
    Expectation {
        file: "text_as.jpg",
        exit: 4,
        stderr_contains: "unsupported or undetectable image format",
    },
    Expectation {
        file: "text_as.png",
        exit: 4,
        stderr_contains: "unsupported or undetectable image format",
    },
    // F1 fix (AC-5): warns, still exits 0.
    Expectation {
        file: "truncated.jpg",
        exit: 0,
        stderr_contains: "truncated",
    },
    Expectation {
        file: "truncated.png",
        exit: 1,
        stderr_contains: "could not decode image:",
    },
    Expectation {
        file: "truncated.avif",
        exit: 1,
        stderr_contains: "could not decode image:",
    },
    // The 20000x20000 pixel-count-cap gap the coverage matrix names.
    Expectation {
        file: "pixel_count_bomb.png",
        exit: 1,
        stderr_contains: "image exceeds decode limits:",
    },
    // AC-4: the empty-alpha-OBU AVIF, committed so the CLI and wasm drive the
    // SAME artifact the SPEC-094 guard is pinned against.
    Expectation {
        file: "empty_alpha_obu.avif",
        exit: 1,
        stderr_contains: "could not decode image:",
    },
];

fn expectation_for(name: &str) -> Option<&'static Expectation> {
    EXPECTATIONS.iter().find(|e| e.file == name)
}

fn run_info(path: &Path) -> std::process::Output {
    Command::new(BIN)
        .args(["info", path.to_str().expect("utf8 path")])
        .output()
        .unwrap_or_else(|e| panic!("failed to run crustyimg info {}: {e}", path.display()))
}

/// AC-3: enumerate the fixture directory and fail on any file with no declared
/// expectation. Adding a fixture without wiring its expectation must turn the
/// suite red, not be silently skipped
/// [[a-harness-that-exercises-nothing-reports-green]].
#[test]
fn every_hostile_fixture_has_a_declared_expectation() {
    let dir = hostile_dir();
    let mut seen: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read tests/fixtures/hostile") {
        let path = entry.expect("read dir entry").path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("utf8 filename")
            .to_owned();
        if name == "README.md" {
            continue;
        }
        assert!(
            expectation_for(&name).is_some(),
            "{name} is in tests/fixtures/hostile/ but has no declared expectation in \
             EXPECTATIONS — a corpus file with no wired expectation is not actually checked"
        );
        seen.push(name);
    }
    for exp in EXPECTATIONS {
        assert!(
            seen.iter().any(|s| s == exp.file),
            "EXPECTATIONS declares {}, but that file is missing from tests/fixtures/hostile/",
            exp.file
        );
    }
    assert_eq!(
        seen.len(),
        EXPECTATIONS.len(),
        "the corpus directory and EXPECTATIONS must match 1:1 (got {} files, {} expectations)",
        seen.len(),
        EXPECTATIONS.len()
    );
}

/// AC-2: every fixture through `info` gets its EXACT declared exit code.
/// `Command::output()` blocks until the process exits, so a hang here would
/// hang the test binary itself — there is no configured `nextest`/`libtest`
/// timeout in this project (plain `cargo test`), so nothing bounds that wait.
#[test]
fn hostile_corpus_exit_codes_are_as_declared() {
    for exp in EXPECTATIONS {
        let path = hostile_dir().join(exp.file);
        let output = run_info(&path);
        assert_eq!(
            output.status.code(),
            Some(exp.exit),
            "{}: expected exit {}, got {:?}; stderr: {}",
            exp.file,
            exp.exit,
            output.status.code(),
            stderr_str(&output)
        );
    }
}

/// The AC-2/AC-6 stderr-cleanliness check for the hand-built corpus: non-empty,
/// carries the declared prefix, and free of `panicked`/`RUST_BACKTRACE`/
/// `unwrap`. None of these eight fixtures are expected to trip the F3
/// debug-profile divergence (that is `meta_parser_state.avif`, a DIFFERENT,
/// pre-existing fuzz-corpus file — see
/// [`meta_parser_state_avif_debug_leak_is_the_only_carve_out`] below), so this
/// stays a plain, unconditional check: a real regression here should fail on
/// every profile.
#[test]
fn hostile_corpus_stderr_is_non_empty_and_not_a_panic() {
    for exp in EXPECTATIONS {
        let path = hostile_dir().join(exp.file);
        let output = run_info(&path);
        let stderr = stderr_str(&output);
        assert!(!stderr.is_empty(), "{}: stderr must be non-empty", exp.file);
        assert!(
            stderr.contains(exp.stderr_contains),
            "{}: stderr must contain {:?}, got: {stderr}",
            exp.file,
            exp.stderr_contains
        );
        for bad in ["panicked", "RUST_BACKTRACE", "unwrap"] {
            assert!(
                !stderr.contains(bad),
                "{}: stderr contains {bad:?} — a raw panic/backtrace leaked: {stderr}",
                exp.file
            );
        }
    }
}

/// AC-6 (F3, recorded not fixed): `meta_parser_state.avif` — the existing
/// fuzz-corpus file at `tests/fixtures/fuzz/avif_decode/` (read in place, NOT
/// duplicated into `tests/fixtures/hostile/`, which has its own hand-built
/// provenance rule) — is the one input that leaks upstream `avif-parse` panic
/// text on the **debug** profile: the default panic hook prints
/// "thread 'main' ... panicked at ..." BEFORE `decode_avif`'s `catch_unwind`
/// (`src/image/avif.rs:127`) converts the unwind into a typed error. Release
/// never panics here (the triggering `debug_assert!` compiles out), so its
/// stderr is exactly our one-line error.
///
/// The carve-out is narrow **by name**: it recognizes only the specific known
/// banner text, and asserts the LAST stderr line is always our own typed
/// error — so a genuinely new/different panic (different text, or appearing on
/// release) still fails this test.
#[test]
fn meta_parser_state_avif_debug_leak_is_the_only_carve_out() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/fuzz/avif_decode/meta_parser_state.avif");
    let output = run_info(&path);
    assert_eq!(
        output.status.code(),
        Some(1),
        "meta_parser_state.avif must exit 1; stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(!stderr.is_empty());

    let last_line = stderr.lines().last().unwrap_or("");
    assert!(
        last_line.starts_with("error: could not decode image:"),
        "the final stderr line must always be our own typed error, got: {last_line}"
    );

    if cfg!(debug_assertions) {
        let lines: Vec<&str> = stderr.lines().collect();
        if lines.len() > 1 {
            // F3's known upstream banner, by name and by POSITION — the extra
            // lines must be EXACTLY this 5-line block, not merely contain its
            // key phrase. `contains()` alone would still pass if a second,
            // unrelated panic-shaped leak appeared alongside the known banner
            // on the same input [[test-a-carve-out-additively-not-just-by-replacement]];
            // requiring an exact-length, line-by-line match closes that hole.
            let banner = &lines[..lines.len() - 1];
            assert_eq!(
                banner.len(),
                5,
                "debug profile's known F3 banner is exactly 5 lines before our typed \
                 error — got {} extra line(s), which means either the banner changed or \
                 a second leak is coexisting with it: {stderr}",
                banner.len()
            );
            assert!(
                banner[0].starts_with("thread 'main' (")
                    && banner[0].contains("panicked at")
                    && banner[0].contains("avif-parse-2.1.0/src/lib.rs:921:9:"),
                "banner line 1 (panic header) is not the known F3 shape: {stderr}"
            );
            assert_eq!(
                banner[1], "assertion `left == right` failed: bad parser state bytes left",
                "banner line 2 is not the known F3 assertion text: {stderr}"
            );
            assert_eq!(
                banner[2], "  left: 0",
                "banner line 3 is not the known F3 left-value: {stderr}"
            );
            assert_eq!(
                banner[3], " right: 1768517057",
                "banner line 4 is not the known F3 right-value: {stderr}"
            );
            assert_eq!(
                banner[4],
                "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace",
                "banner line 5 is not the known F3 backtrace note: {stderr}"
            );
        }
    } else {
        assert_eq!(
            stderr.lines().count(),
            1,
            "release stderr must be exactly our one-line error, got: {stderr}"
        );
    }
}

/// AC-4: driven through the CLI, the committed empty-alpha-OBU AVIF exits 1 —
/// the `debug_assertions` profile is the leg that actually proves the
/// SPEC-094 guard fires before `re_rav1d`'s `debug_abort()` can
/// ([[a-thread-boundary-does-not-catch-abort]]: a `debug_abort()` is not an
/// unwind, so a `cargo test` run where the guard regressed would show the
/// whole test BINARY die with `SIGABRT`, not a failed assertion — the
/// `Some(1)` below is what a passing run looks like on both profiles).
#[test]
fn empty_alpha_obu_avif_exits_1_through_the_cli() {
    let path = hostile_dir().join("empty_alpha_obu.avif");
    let output = run_info(&path);
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(stderr.contains("could not decode image:"), "got: {stderr}");
}

/// The 20000x20000 pixel-count-cap gap the coverage matrix names as a CLI gap
/// (the per-dimension 70000x1 cap already had `info_on_oversized_image_exits_1_not_panic`
/// in `tests/cli.rs`; the pixel-COUNT cap did not). The behaviour is already
/// correct on `main` — this is coverage for an existing guard, not a fix.
#[test]
fn pixel_count_bomb_png_exits_1() {
    let path = hostile_dir().join("pixel_count_bomb.png");
    let output = run_info(&path);
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        stderr_str(&output)
    );
    let stderr = stderr_str(&output);
    assert!(
        stderr.contains("image exceeds decode limits:"),
        "got: {stderr}"
    );
}

// ── AC-5: F1 fixed — a truncated JPEG warns, still exits 0 ────────────────────

/// Run one verb over `input_path`, writing to `out_path` when the verb needs an
/// output sink (`convert`/`resize`; `info` takes no output). Returns the raw
/// `Output` so the caller inspects exit code and stderr.
fn run_verb(verb: &str, input_path: &Path, out_path: &Path) -> std::process::Output {
    let mut cmd = Command::new(BIN);
    match verb {
        "info" => {
            cmd.args(["info", input_path.to_str().unwrap()]);
        }
        "web" => {
            cmd.args([
                "web",
                input_path.to_str().unwrap(),
                "-o",
                out_path.to_str().unwrap(),
            ]);
        }
        "convert" => {
            cmd.args([
                "convert",
                input_path.to_str().unwrap(),
                "--format",
                "png",
                "-o",
                out_path.to_str().unwrap(),
            ]);
        }
        "resize" => {
            cmd.args([
                "resize",
                input_path.to_str().unwrap(),
                "--max",
                "16",
                "-o",
                out_path.to_str().unwrap(),
            ]);
        }
        other => panic!("unknown verb {other}"),
    }
    cmd.output()
        .unwrap_or_else(|e| panic!("failed to run crustyimg {verb}: {e}"))
}

/// A truncated JPEG through `info`, `web`, `convert`, and `resize` prints a
/// one-line stderr warning naming truncation, still writes its output, and
/// still exits 0 (AC-5; the maintainer decision recorded in
/// `decisions/DEC-085-*.md`: warn, don't fail — a truncated JPEG opens in
/// every viewer that ever shipped, and the exit-code surface is frozen,
/// STAGE-030).
#[test]
fn truncated_jpeg_warns_on_stderr_and_still_exits_0() {
    let dir = tempfile::tempdir().expect("tempdir");
    let input = hostile_dir().join("truncated.jpg");
    for verb in ["info", "web", "convert", "resize"] {
        let out_path = dir.path().join(format!("out_{verb}.png"));
        let output = run_verb(verb, &input, &out_path);
        let stderr = stderr_str(&output);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{verb}: truncated JPEG must still exit 0; stderr: {stderr}"
        );
        assert!(
            !stderr.is_empty() && stderr.to_lowercase().contains("truncat"),
            "{verb}: expected a stderr warning naming truncation, got: {stderr:?}"
        );
        if verb != "info" {
            assert!(
                out_path.exists(),
                "{verb}: output must still be written despite the warning"
            );
        }
    }
}

/// AC-5's negative control: a well-formed JPEG through the same four verbs
/// prints NO truncation warning. Without this, `truncated_jpeg_warns_on_stderr…`
/// could pass on a check that fires unconditionally
/// [[a-plausible-test-result-is-not-a-checked-one]].
#[test]
fn well_formed_jpeg_emits_no_truncation_warning() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bytes = common::gradient_jpeg(64, 64);
    let input = dir.path().join("real.jpg");
    std::fs::write(&input, &bytes).expect("write real.jpg");

    for verb in ["info", "web", "convert", "resize"] {
        let out_path = dir.path().join(format!("out_{verb}.png"));
        let output = run_verb(verb, &input, &out_path);
        let stderr = stderr_str(&output);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{verb}: a well-formed JPEG must exit 0; stderr: {stderr}"
        );
        assert!(
            !stderr.to_lowercase().contains("truncat"),
            "{verb}: a well-formed JPEG must not warn about truncation, got: {stderr:?}"
        );
    }
}

// ── Corpus builder (AC-1) ──────────────────────────────────────────────────────
//
// The committed bytes under `tests/fixtures/hostile/` are generated by THIS
// code, not hand-crafted opaque blobs (Notes: "Build the corpus
// deterministically and commit the builder"). Regenerate after touching this
// module with:
//   cargo test --test hostile_inputs -- --ignored regenerate_hostile_corpus
// then diff/inspect before committing — the checked-in files are expected to
// be byte-identical to this output.
mod build_corpus {
    use super::hostile_dir;

    fn write(name: &str, bytes: &[u8]) {
        let path = hostile_dir().join(name);
        assert!(
            bytes.len() <= 4096,
            "{name} is {} bytes, over the AC-1 4 KB cap",
            bytes.len()
        );
        std::fs::write(&path, bytes).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    }

    #[test]
    #[ignore = "regenerates the committed hostile corpus; run manually, not part of the suite"]
    fn regenerate_hostile_corpus() {
        // Zero-byte file: the empty input itself.
        write("zero_byte.png", &[]);

        // Text bytes renamed to image extensions: not an image at all, by
        // construction — no header a guesser could ever match.
        const TEXT: &[u8] = b"this is not an image, just plain text bytes\n";
        write("text_as.jpg", TEXT);
        write("text_as.png", TEXT);

        // Truncated JPEG: a real generated JPEG, cut well into the entropy-coded
        // scan data. The design cycle's probe cut its `real.jpg` to about a
        // third and the decoder tolerated it; empirically, on THIS 96x96
        // gradient JPEG, anything below ~50% cuts before the decoder has
        // enough of the scan to produce a full frame (a hard `Decode` error,
        // not F1's silent-success case) — confirmed while building this
        // fixture (SPEC-107). 60% leaves a comfortable margin on the
        // "still decodes" side while still losing the EOI marker.
        let jpeg = super::common::gradient_jpeg(96, 96);
        let cut = jpeg.len() * 6 / 10;
        write("truncated.jpg", &jpeg[..cut]);

        // Truncated PNG: a real generated PNG, kept to its first third — long
        // enough to carry a valid IHDR, short enough to be missing IDAT/IEND.
        let png = super::common::solid_png(96, 96, [42, 100, 200]);
        let cut = png.len() / 3;
        write("truncated.png", &png[..cut]);

        // Truncated AVIF: the committed 16x16 solid AVIF fixture, kept to its
        // first half (mirrors the design probe's truncation ratio).
        let avif: &[u8] = include_bytes!("fixtures/avif/solid_16x16.avif");
        let cut = avif.len() / 2;
        write("truncated.avif", &avif[..cut]);

        // Forged pixel-count-bomb PNG: a real CRC32'd IHDR declaring
        // 20000x20000 (400,000,000 px) — the pixel-count cap (DEC-063), as
        // opposed to `tests/cli.rs`'s existing per-dimension 70000x1 case
        // (DEC-034). Shape: `tests/common::png_header_declaring`, which
        // mirrors `wasm_roundtrip.rs`'s private copy (Notes).
        write(
            "pixel_count_bomb.png",
            &super::common::png_header_declaring(20_000, 20_000),
        );

        // empty_alpha_obu.avif is generated by src/image/avif.rs's own
        // `generate_empty_alpha_obu_fixture` (AC-4) — not duplicated here, so
        // the CLI/wasm-driven artifact and the SPEC-094 unit test can never
        // drift apart (one source of truth for those exact bytes).
    }
}
