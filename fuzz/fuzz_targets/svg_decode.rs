//! Fuzz the SVG rasterize path (SPEC-060, untrusted-input-hardening).
//!
//! `Image::from_bytes` content-sniffs `<svg`/`<?xml` and rasterizes the input
//! through `usvg` (XML parse) + `tiny-skia` (raster). Seeding the corpus with
//! the plain-text fixture (see the `Seed:` line in `fuzz/Cargo.toml`) drives
//! libFuzzer's mutations through both the parse and the render. The contract is
//! simply: never panic — any malformed SVG must surface as a typed error, not a
//! crash (the DEC-034 caps also bound allocation so a huge-`viewBox` bomb cannot
//! OOM). The hardened options refuse external file/URL refs, so the fuzzer
//! cannot make the target touch the filesystem or network.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Ok or Err are both acceptable; a panic is the only failure this hunts for.
    let _ = crustyimg::image::Image::from_bytes(data);
});
