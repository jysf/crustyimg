//! SPEC-095: the demo's honesty copy must state an ACCURATE claim, not a hedge that
//! was only true before this spec. Before SPEC-095 the wasm demo path encoded AVIF
//! at q80 while native `crustyimg web` encodes at q85 (DEC-069) — a real divergence,
//! honestly flagged as "approximates". SPEC-095 closes that divergence (the wasm
//! no-search AVIF default now anchors to `FAST_LOSSY_QUALITY`), so the same hedge
//! left in place would now be UNDER-claiming: the demo and the CLI agree on the
//! quality setting. The new copy must say so — precisely, not overclaiming
//! byte-identity the no-asm wasm `rav1e` build cannot promise.
//!
//! A plain string check on the shipped copy, in the spirit of
//! `tests/adoption_glue.rs`'s README/justfile checks — content correctness here
//! is a documentation claim, not a runtime behavior, so this is the mechanical
//! check that keeps it from drifting silently
//! ([[a-citation-looks-like-prose-not-a-claim]]).

use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn read(rel: &str) -> String {
    std::fs::read_to_string(Path::new(ROOT).join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

/// The old hedge — "approximates" — must be gone from the user-facing funnel copy:
/// it described a real quality divergence that SPEC-095 closed.
#[test]
fn demo_html_no_longer_hedges_approximates() {
    let html = read("demo/index.html");
    assert!(
        !html.contains("approximates"),
        "demo/index.html must not claim the demo only 'approximates' crustyimg web — \
         SPEC-095 closed the quality divergence that made that claim honest"
    );
}

/// The funnel copy states the accurate claim: same engine, same AVIF quality (q85).
#[test]
fn demo_html_states_same_engine_and_quality() {
    let html = read("demo/index.html");
    assert!(
        html.contains("q85"),
        "demo/index.html funnel copy should name the actual shared quality (q85)"
    );
    assert!(
        html.contains("same engine"),
        "demo/index.html funnel copy should claim the same engine as crustyimg web"
    );
}

/// Precise, not overclaiming: bytes are NOT guaranteed identical (the wasm build is
/// a no-asm `rav1e`), so the copy must say "same settings" rather than "identical
/// bytes" — the honesty this whole demo is built on cuts both ways.
#[test]
fn demo_html_does_not_overclaim_byte_identity() {
    let html = read("demo/index.html");
    assert!(
        !html.contains("byte-identical") || html.contains("not guaranteed"),
        "demo/index.html must not claim byte-identical output without qualifying it — \
         the wasm build is a no-asm rav1e and cannot guarantee that"
    );
}

/// The README states the same resolved claim (mirrors the funnel copy).
#[test]
fn demo_readme_no_longer_hedges_approximates() {
    let readme = read("demo/README.md");
    assert!(
        !readme.contains("**approximates**"),
        "demo/README.md must not claim the demo only approximates crustyimg web — \
         SPEC-095 closed the quality divergence that made that claim honest"
    );
    assert!(
        readme.contains("q85"),
        "demo/README.md should name the actual shared AVIF quality (q85)"
    );
}
