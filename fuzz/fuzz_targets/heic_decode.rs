//! Fuzz the HEIC decode path (SPEC-062, untrusted-input-hardening).
//!
//! Only meaningful with the `heic` feature ON — build it with
//! `cargo +nightly fuzz run --features heic heic_decode` and a system libheif
//! present. Without the feature the target still builds and runs, but it only
//! exercises `is_heic` brand detection and the `CodecNotBuilt` rejection.
//!
//! `Image::from_bytes` dispatches HEVC-branded input into `libheif-rs` and the
//! system libheif **C** decoder, then packs the interleaved plane (honoring its
//! stride) into an `RgbImage`/`RgbaImage`. Seeding the corpus with a real `.heic`
//! (see the `Seed:` line in `fuzz/Cargo.toml`) drives libFuzzer's mutations
//! through both the container parse and the decode/copy path. The contract is
//! simply: never panic — any malformed input must surface as a typed error, not a
//! crash (the DEC-034 caps, checked from the image handle before decode, also
//! bound allocation so a decompression-bomb header cannot OOM).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The result is intentionally ignored: Ok or Err are both acceptable; a
    // panic is the only failure this target hunts for.
    let _ = crustyimg::image::Image::from_bytes(data);
});
