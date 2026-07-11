//! Deterministic regressions + always-on smoke for the decoder fuzz gate
//! (SPEC-069, DEC-062).
//!
//! The `cargo fuzz` targets in `fuzz/` run on a **nightly** toolchain, off the
//! per-PR path. This file is the *durable* half of the gate: it runs in the
//! ordinary `cargo test` suite (so 3-OS CI exercises it every PR, without the
//! fuzzer), and it has two parts:
//!
//! 1. **Regressions** — one deterministic test per crash the fuzzer found. Each
//!    feeds the **minimized** crashing bytes (committed under
//!    `tests/fixtures/fuzz/<target>/`) to the real entry point
//!    (`Image::from_bytes` for AVIF/SVG/HEIC, `raw_preview` for RAW) and asserts
//!    a **typed error and no panic**. Each must fail before its guard/fix and
//!    pass after (mutation-checked, SPEC-064 style).
//!
//! 2. **`fuzz_corpus_never_panics`** — the cheap, always-on smoke: it sweeps
//!    every committed seed fixture (`tests/fixtures/{avif,svg,raw,heic}`) **and**
//!    every committed crash reproducer (`tests/fixtures/fuzz/**`) through the
//!    matching entry point and asserts none of them panics (`Ok` or a typed
//!    `Err` are both fine). This is what keeps the crash corpus non-panicking on
//!    macOS/Linux/Windows after this spec ships.
//!
//! Note on build profiles: `cargo test` runs with **debug-assertions on**, so an
//! upstream `debug_assert!` reached over untrusted bytes (e.g. `avif-parse`'s
//! `check_parser_state`) *would* panic here without our boundary guard — which is
//! exactly why the AVIF decoder wraps the third-party parse/decode in
//! `catch_unwind` (see `src/image/avif.rs`). These tests pin that guard.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use crustyimg::error::ImageError;
use crustyimg::image::{raw_preview, Image};

/// Absolute path to a directory under the crate's `tests/` tree, robust to the
/// test binary's working directory.
fn tests_dir(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(rel)
}

/// Run `entry` on `bytes` inside `catch_unwind`, returning `Err(())` if it
/// panicked. The decode result itself is discarded — the contract under test is
/// "never panics", so `Ok` and a typed `Err` are equally acceptable.
fn ran_without_panic(bytes: &[u8], entry: fn(&[u8]) -> ()) -> Result<(), ()> {
    catch_unwind(AssertUnwindSafe(|| entry(bytes))).map_err(|_| ())
}

/// Feed every regular file in `dir` (if it exists) through `entry`, asserting
/// none panics. Missing directories are skipped (a clean target has no
/// `fuzz/<target>/` crash dir — that is expected). Returns the count exercised.
fn assert_dir_never_panics(rel: &str, entry: fn(&[u8]) -> ()) -> usize {
    let dir = tests_dir(rel);
    let read = match std::fs::read_dir(&dir) {
        Ok(r) => r,
        // A clean target legitimately has no committed crash corpus.
        Err(_) => return 0,
    };
    let mut count = 0;
    for entry_res in read {
        let path = entry_res.expect("read dir entry").path();
        // Skip subdirectories and dotfiles (e.g. a `.gitkeep`).
        if !path.is_file() {
            continue;
        }
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'))
        {
            continue;
        }
        let bytes = std::fs::read(&path).expect("read fixture file");
        assert!(
            ran_without_panic(&bytes, entry).is_ok(),
            "decoder PANICKED on corpus file {} — the never-panic contract is broken",
            path.display()
        );
        count += 1;
    }
    count
}

/// `Image::from_bytes` adaptor for the content-sniffed formats (AVIF/SVG/HEIC).
fn from_bytes_entry(bytes: &[u8]) {
    let _ = Image::from_bytes(bytes);
}

/// `raw_preview` adaptor for the extension-routed RAW path.
fn raw_preview_entry(bytes: &[u8]) {
    let _ = raw_preview(bytes);
}

// ---------------------------------------------------------------------------
// Regressions (one per crash the SPEC-069 fuzz gate found)
// ---------------------------------------------------------------------------

/// Assert `from_bytes(fixture)` returns a typed `Decode` error and does not
/// panic — even under `cargo test`'s debug-assertions. `catch_unwind` here turns
/// a regressed panic into a clear failure message instead of aborting the binary.
fn assert_avif_fixture_is_decode_error(rel: &str) {
    let bytes = std::fs::read(tests_dir(rel)).expect("read avif crash fixture");
    let result = catch_unwind(AssertUnwindSafe(|| Image::from_bytes(&bytes)));
    let decoded = result.unwrap_or_else(|_| panic!("from_bytes PANICKED on {rel}"));
    assert!(
        matches!(decoded, Err(ImageError::Decode(_))),
        "expected a typed Decode error for {rel}, got {decoded:?}"
    );
}

/// avif_decode / finding 1 (upstream `avif-parse`, bucket c) — this 32-byte
/// container carries an inner box whose size field overruns the buffer. Root
/// cause: `avif-parse` 2.1.0 trusts declared box sizes; the guard is
/// `box_sizes_fit` (a top-level size sanity check before `read_avif`). Pre-fix
/// this reached `check_parser_state`'s `debug_assert!` and panicked under
/// debug-assertions; post-fix it is a clean typed error.
#[test]
fn avif_bad_parser_state_is_typed_error_not_panic() {
    assert_avif_fixture_is_decode_error("fixtures/fuzz/avif_decode/bad_parser_state.avif");
}

/// avif_decode / finding 2 (upstream `avif-parse`, bucket b/c — the OOM) — a
/// 286-byte container whose `ftyp` size field reads `0xB8000018` ≈ 3.09 GB.
/// Pre-fix `read_avif` allocated ~3 GB before any cap ran (libFuzzer OOM);
/// post-fix `box_sizes_fit` rejects the inflated header up front. Mutation-check
/// was done under the fuzzer (`-O`): the original code OOMs, the fixed code runs
/// clean.
#[test]
fn avif_container_box_size_bomb_is_typed_error_not_oom() {
    assert_avif_fixture_is_decode_error("fixtures/fuzz/avif_decode/container_box_size_bomb.avif");
}

/// avif_decode / finding 3 (upstream `avif-parse`, bucket c) — a ~235-byte
/// container with *valid* top-level box sizes (so it passes `box_sizes_fit`) that
/// trips a **different** `check_parser_state` `debug_assert!` deep in meta-box
/// parsing (`avif-parse` `src/lib.rs:921`). This is the input that pins the
/// `catch_unwind` boundary in `decode_avif`: a `--release`/`-O` build compiles the
/// assert out (clean `Err`), but `cargo test` runs debug-assertions ON, so
/// without the guard this panics here. (Mutation-check: delete the `catch_unwind`
/// in `src/image/avif.rs` and this test panics.)
#[test]
fn avif_meta_parser_state_is_typed_error_not_panic() {
    assert_avif_fixture_is_decode_error("fixtures/fuzz/avif_decode/meta_parser_state.avif");
}

// ---------------------------------------------------------------------------
// Always-on corpus smoke
// ---------------------------------------------------------------------------

/// Sweep every committed seed fixture and every committed crash reproducer
/// through the matching decoder entry point and assert **no panic**. This is the
/// per-PR guard (runs in normal `cargo test`, no nightly/fuzzer) that the crash
/// corpus stays non-panicking on all three CI OSes.
#[test]
fn fuzz_corpus_never_panics() {
    let mut total = 0;

    // Content-sniffed default formats: seeds + any committed crash reproducers.
    total += assert_dir_never_panics("fixtures/avif", from_bytes_entry);
    total += assert_dir_never_panics("fixtures/svg", from_bytes_entry);
    total += assert_dir_never_panics("fixtures/fuzz/avif_decode", from_bytes_entry);
    total += assert_dir_never_panics("fixtures/fuzz/svg_decode", from_bytes_entry);

    // Extension-routed RAW path (`from_bytes` never reaches it — call directly).
    total += assert_dir_never_panics("fixtures/raw", raw_preview_entry);
    total += assert_dir_never_panics("fixtures/fuzz/raw_preview", raw_preview_entry);

    // HEIC only decodes with the `heic` feature (system libheif); without it,
    // `from_bytes` short-circuits at brand detection with `CodecNotBuilt` and
    // never reaches a decoder, so exercising the HEIC corpus is only meaningful
    // (and only reaches libheif's C code) with the feature on.
    #[cfg(feature = "heic")]
    {
        total += assert_dir_never_panics("fixtures/heic", from_bytes_entry);
        total += assert_dir_never_panics("fixtures/fuzz/heic_decode", from_bytes_entry);
    }

    // Guard against the sweep silently exercising nothing (e.g. a fixtures move):
    // the seed corpus alone guarantees several files.
    assert!(
        total >= 3,
        "corpus smoke exercised only {total} files — expected the seed fixtures at minimum"
    );
}
