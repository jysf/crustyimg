//! Fuzz the AVIF decode path (SPEC-058, untrusted-input-hardening).
//!
//! `Image::from_bytes` dispatches AVIF-branded input through the container
//! parser (`avif-parse`) and the AV1 decoder (`re_rav1d`) + YUV→RGB conversion.
//! Seeding the corpus with a real `.avif` (see the `Seed:` line in
//! `fuzz/Cargo.toml`) drives libFuzzer's mutations through BOTH the container
//! parse and the decode/convert path. The contract is simply: never panic —
//! any malformed input must surface as a typed error, not a crash (the DEC-034
//! caps also bound allocation so a decompression-bomb header cannot OOM).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The result is intentionally ignored: Ok or Err are both acceptable; a
    // panic is the only failure this target hunts for.
    let _ = crustyimg::image::Image::from_bytes(data);
});
