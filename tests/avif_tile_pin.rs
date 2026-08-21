//! Integration tests for SPEC-124 — pin the AVIF encoder's tile count.
//!
//! `ravif` derives the AV1 tile count from `threads.unwrap_or_else(rayon::
//! current_num_threads)` (`av1encoder.rs:653-654`), and with the shipped
//! build's `threading` feature off, that `current_num_threads` is `ravif`'s
//! own `rayoff` shim — `std::thread::available_parallelism()`, the machine's
//! core count (DEC-094). Pinning `with_num_threads(Some(N))` makes `threads`
//! always `Some`, so that branch can never run: tile count becomes a
//! compile-time constant instead of a value read from the OS.
//!
//! Two things can break that guarantee, and this file drives both:
//!
//! - **Partial application.** DEC-019/DEC-068 require `sink::
//!   encode_to_bytes_with` and `quality::encode_candidate_bytes_with` to stay
//!   byte-identical at the same (image, quality, speed). Pinning one and not
//!   the other breaks that lockstep — [`both_encode_paths_set_the_thread_count`].
//! - **The mechanism itself.** [`avif_output_is_identical_across_ambient_core_counts`]
//!   proves the OS core-count reading is unreachable, not merely that it
//!   currently agrees. The shipped (non-`threading`) build is deaf to every
//!   user-space lever *before* this spec too (DEC-094 leg A/B) — a test that
//!   only sweeps `RAYON_NUM_THREADS` against the shipped binary would pass
//!   whether or not the pin exists, which is not a test. So this drives the
//!   ONE lever DEC-094 established actually reaches `ravif`'s tile
//!   computation: a probe binary built with `--features image/rayon`
//!   (`ravif/threading` ON), where `current_num_threads` becomes real
//!   rayon's and DOES respond to `RAYON_NUM_THREADS` (DEC-094 leg E). Sweeping
//!   it there is this repo's only available proxy for "what would a
//!   differently-cored machine compute" (DEC-094 itself could not drive a
//!   second physical host either — see its "Not measured here"). Building
//!   that probe is the one-time cost this file pays; see `probe_binary()`.

#![cfg(feature = "avif")]

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use ::image::{DynamicImage, ImageFormat, RgbImage};
use crustyimg::image::Image;
use crustyimg::quality::auto_under_size_at_speed;
use crustyimg::sink::{encode_to_bytes_with, AVIF_DEFAULT_QUALITY};

// ── Fixtures ─────────────────────────────────────────────────────────────────

/// A flat, few-colour "graphic" fixture (mirrors `examples/gen_bench_corpus.rs`'s
/// `graphic()`) — DEC-094 measured this content class as the one where tile
/// count costs the most compression (+47.9% at 14 tiles vs 1), so a broken
/// lockstep or a reopened core-count dependency shows up as a large,
/// unmistakable byte delta here rather than a coin-flip on noisy content.
fn graphic_fixture(w: u32, h: u32) -> DynamicImage {
    let palette = [
        [20u8, 30, 40],
        [220, 50, 47],
        [38, 139, 210],
        [133, 153, 0],
        [245, 245, 245],
    ];
    let mut img = RgbImage::new(w, h);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let bx = (x * 4 / w.max(1)) as usize;
        let by = (y * 3 / h.max(1)) as usize;
        *px = ::image::Rgb(palette[(bx + by) % palette.len()]);
    }
    DynamicImage::ImageRgb8(img)
}

fn graphic_png_bytes(w: u32, h: u32) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    graphic_fixture(w, h)
        .write_to(&mut out, ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

// ── AC-1: the two AVIF encode arms stay in lockstep ────────────────────────────

/// A speed faster than the production [`AVIF_SPEED`] (6), matching
/// `size_search_speed_parity`'s own precedent — this test cares about byte
/// PARITY, not the production speed, and `min_tile_size`'s speed term is
/// already 128 for every `speed >= 5` (`av1encoder.rs`'s `SpeedTweaks::
/// from_my_preset`), so 10 exercises the identical tile-count math faster.
const TEST_SPEED: u8 = 10;

/// DEC-019/DEC-068's byte-parity contract, driven at the crate's public
/// boundary rather than re-asserted as a comment: `quality::
/// auto_under_size_at_speed`'s byte-budget search (which encodes candidates
/// through the private `encode_candidate_bytes_with`) and `sink::
/// encode_to_bytes_with` must land on the exact same byte length for the same
/// (image, quality, speed) — which is only true if BOTH arms set the SAME
/// thread pin. Pin one and not the other and this goes red: the two arms
/// derive different tile counts (`Some(N)` vs the ambient core count) and
/// therefore different byte lengths, on this graphic fixture by a wide margin.
#[test]
fn both_encode_paths_set_the_thread_count() {
    // 512x512, matching DEC-094's `graphic_large.png`: the size term
    // `(w*h)/min_tile_size^2` is 16 at this resolution, well above the
    // ambient core count on any host DEC-094 or this build measured, so
    // "pinned N=1" vs "ambient N" never coincidentally saturate to the same
    // tile count the way a smaller fixture's size-term clamp can.
    let png = graphic_png_bytes(512, 512);
    let img = Image::from_bytes(&png).expect("fixture PNG decodes");

    let sink_bytes = encode_to_bytes_with(
        &img,
        ImageFormat::Avif,
        Some(AVIF_DEFAULT_QUALITY),
        Some(TEST_SPEED),
    )
    .expect("sink AVIF encode");
    let budget = sink_bytes.len() as u64;

    // The search's own probe goes through the private `encode_candidate_bytes_with`
    // — its `choice.score` is that probe's byte length at `choice.quality`. Do NOT
    // assert `choice.quality == AVIF_DEFAULT_QUALITY`: this fixture is flat enough
    // that many qualities tie on byte count (AVIF saturates well below 100 on
    // near-solid content), so the search can legitimately land on any member of
    // that tie set, not necessarily the lowest.
    let choice =
        auto_under_size_at_speed(img.pixels(), ImageFormat::Avif, budget, Some(TEST_SPEED))
            .expect("AVIF byte-budget search");

    // The dispositive check: re-encode through the SINK at the search's own
    // winning quality and confirm it produces exactly the length the search
    // probed. If only one AVIF arm carries the thread-count pin, this quality's
    // probe and the sink's re-encode resolve different tile counts and diverge —
    // on this graphic fixture, DEC-094 measured that gap at up to +47.9%, so a
    // broken lockstep cannot hide inside search noise.
    let emitted = encode_to_bytes_with(
        &img,
        ImageFormat::Avif,
        Some(choice.quality),
        Some(TEST_SPEED),
    )
    .expect("sink AVIF re-encode at the search's winning quality");

    assert_eq!(
        emitted.len() as u64,
        choice.score as u64,
        "sink::encode_to_bytes_with emitted {} bytes at q{} but quality::\
         encode_candidate_bytes_with (via the byte-budget search) probed {} bytes at the SAME \
         quality/speed — the two AVIF encode arms have drifted on the thread-count pin, which \
         DEC-019/DEC-068 require to stay in lockstep",
        emitted.len(),
        choice.quality,
        choice.score
    );
}

// ── AC-2 / AC-5: the pin makes the OS core-count reading unreachable ───────────

/// Build (once, memoized for the process) a probe `crustyimg` with
/// `ravif/threading` reachable (`--features image/rayon`) in an isolated
/// `CARGO_TARGET_DIR` — never the shared one
/// ([[concurrent-differently-featured-builds-corrupt-a-shared-target-dir]]).
/// A debug build is enough: this drives byte-identity, not speed.
fn probe_binary() -> &'static PathBuf {
    static PROBE: OnceLock<PathBuf> = OnceLock::new();
    PROBE.get_or_init(|| {
        let target_dir = std::env::temp_dir().join(format!(
            "crustyimg-spec124-probe-target-{}",
            std::process::id()
        ));
        let status = Command::new("cargo")
            .args(["build", "--features", "image/rayon"])
            .env("CARGO_TARGET_DIR", &target_dir)
            .status()
            .expect("cargo must be on PATH to build the determinism probe");
        assert!(
            status.success(),
            "probe build (--features image/rayon) failed"
        );
        let bin = target_dir.join("debug").join(if cfg!(windows) {
            "crustyimg.exe"
        } else {
            "crustyimg"
        });
        assert!(bin.exists(), "probe binary not found at {}", bin.display());
        bin
    })
}

/// Runs the probe binary's `convert --format avif` on `input`, with
/// `RAYON_NUM_THREADS` set to `threads` — DEC-094 leg E's proxy for "what a
/// machine with this many cores would compute" — and returns the output
/// bytes' SHA-256.
fn probe_encode_hash(input: &std::path::Path, threads: u32) -> String {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.avif");
    let status = Command::new(probe_binary())
        .env("RAYON_NUM_THREADS", threads.to_string())
        .args(["convert"])
        .arg(input)
        .arg("-o")
        .arg(&out)
        .args(["--format", "avif"])
        .status()
        .expect("probe binary must run");
    assert!(
        status.success(),
        "probe encode failed at RAYON_NUM_THREADS={threads}"
    );
    let bytes = std::fs::read(&out).expect("probe must write output");
    sha2_hex(&bytes)
}

fn sha2_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// AC-2: with the pin in place, `threads.unwrap_or_else(current_num_threads)`
/// never evaluates the fallback, so the value `current_num_threads` WOULD
/// have returned — real rayon's pool size on this probe, standing in for the
/// OS core-count reading `rayoff` uses on the shipped build — cannot reach
/// the encoder. Sweeping `RAYON_NUM_THREADS` on the probe and finding the
/// output invariant is this repo's available proof of that (see the module
/// doc). AC-5's negative control: temporarily revert the pin and re-run —
/// DEC-094 leg E already measured that this exact lever moves the bytes
/// (threads 1/4/14 → three distinct hashes) when the pin is absent, so this
/// test goes red the moment the pin is removed.
#[test]
fn avif_output_is_identical_across_ambient_core_counts() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("graphic.png");
    std::fs::write(&input, graphic_png_bytes(256, 256)).unwrap();

    let simulated_core_counts = [1, 2, 4, 8, 14];
    let hashes: Vec<(u32, String)> = simulated_core_counts
        .iter()
        .map(|&t| (t, probe_encode_hash(&input, t)))
        .collect();

    let first = &hashes[0].1;
    for (threads, hash) in &hashes {
        assert_eq!(
            hash, first,
            "AVIF output changed between a simulated {}-core machine and a {}-core one \
             (hashes: {hashes:?}) — the thread-count pin no longer makes the tile count \
             independent of the ambient core count",
            hashes[0].0, threads
        );
    }
}
