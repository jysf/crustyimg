//! Fuzz the RAW embedded-preview extraction path (SPEC-061, untrusted-input-hardening).
//!
//! `crustyimg::image::raw_preview` scans untrusted RAW bytes for embedded JPEG
//! streams (`FF D8 FF`), prunes on a plausible marker, and decodes each
//! candidate through the DEC-034-capped `image` JPEG decoder, keeping the
//! largest. RAW is routed by extension in `Image::load`, so `from_bytes` never
//! reaches this path — the fuzz target calls the byte entry directly. Seeding
//! the corpus with the synthetic fixture (see the `Seed:` line in
//! `fuzz/Cargo.toml`) drives libFuzzer's mutations through the scan + decode. The
//! contract is simply: never panic — any malformed input must surface as a typed
//! error, not a crash (the caps bound allocation and MAX_PREVIEW_CANDIDATES
//! bounds the decode work, so neither a bomb preview nor a file stuffed with fake
//! SOIs can OOM or hang).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Ok or Err are both acceptable; a panic is the only failure this hunts for.
    let _ = crustyimg::image::raw_preview(data);
});
