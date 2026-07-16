//! crustyimg — a fast Rust CLI to view and transform images.
//!
//! The library root. SPEC-001 shipped the std-only scaffold (`version()`);
//! SPEC-002 adds the canonical pixel core: typed [`error`]s and the [`image`]
//! module (the `Image` model + load/decode + metadata capture, DEC-002/003).
//! SPEC-003 adds the [`operation`] trait + concrete ops and the [`pipeline`]
//! executor (decode-once fold, DEC-002).
//! SPEC-004 adds the [`source`] module: CLI-argument → ordered [`source::Input`]
//! list (single file / glob / directory / stdin), with symlink-escape hardening.
//! SPEC-005 adds the [`sink`] module: encode a final [`image::Image`] to file /
//! dir+name-template / stdout / terminal display, with traversal + overwrite
//! hardening (DEC-002, DEC-004, DEC-007, DEC-011).
//! SPEC-006 adds the [`recipe`] module: TOML recipe (de)serialization +
//! operation registry; the keystone of "tune once, replay across many" (DEC-005).
//! SPEC-007 adds the [`cli`] module: the clap subcommand surface + dispatch +
//! exit-code mapping (DEC-012, DEC-007).
//! SPEC-016 adds the [`quality`] module: the SSIMULACRA2 perceptual metric + a
//! generic JPEG-quality binary search powering `optimize`'s auto-quality (DEC-019).
//! SPEC-026 adds the [`metadata`] module: the container lane (`strip` +
//! `clean --gps`), editing container bytes without re-decoding pixels (DEC-003,
//! DEC-029).
//! SPEC-030 adds the [`text`] module: pure glyph rasterization (`skrifa` +
//! `zeno` + bundled Go font, SPEC-044/DEC-045) that turns a string into a
//! transparent RGBA overlay for `watermark --text`, then reuses the SPEC-029
//! compositing path (DEC-032).
//! SPEC-063 adds the [`build`] module: the `crustyimg.build.toml` manifest
//! (versioned `[[target]]`s binding sources × a recipe → an out dir + name
//! template) that `crustyimg build` runs end to end (DEC-057).
//! SPEC-046 adds the [`analysis`] module: the computed-once `Analysis` feature
//! layer (histogram, entropy, edge density, alpha coverage, capped
//! unique-colours, dominant colour) that PROJ-002's optimization engine reads
//! (DEC-002, DEC-034). It lands standalone — no command consumes it yet.

//! SPEC-072 partitions the crate by TARGET (DEC-064): the modules below split
//! into the **pure engine** (decode → operations → encode), which compiles for
//! both native and `wasm32-unknown-unknown`, and the **shell** ([`cli`],
//! [`source`], [`build`], [`lint`]), which is filesystem/argv-bound and is
//! compiled only for native targets. The split is by `cfg(target_arch)`, not by
//! a cargo feature, so the native feature matrix (default / lean / `avif` /
//! `heic` / `webp-lossy`) is exactly as it was. [`wasm`] is the wasm-only
//! `wasm-bindgen` surface over the engine.

// ── The pure engine: native AND wasm32 ────────────────────────────────────────
pub mod analysis;
pub mod error;
pub mod image;
pub mod metadata;
pub mod operation;
pub mod pipeline;
pub mod quality;
pub mod recipe;
pub mod text;

// `sink` straddles the split: `encode_to_bytes` is the pure bytes-out encoder the
// wasm surface calls, while `Sink`'s file/stdout/display writers are native. The
// module compiles on both targets (its `std::fs` calls are simply never reached
// on wasm); only the viuer-backed `Display` arm is target-gated. Keeping it whole
// means the wasm build encodes through the exact same function as the CLI.
pub mod sink;

// ── The shell: native only ────────────────────────────────────────────────────
#[cfg(not(target_arch = "wasm32"))]
pub mod build;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod lint;
#[cfg(not(target_arch = "wasm32"))]
pub mod source;

// ── The wasm32 surface ────────────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Returns the crate's semantic version (from `Cargo.toml`).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_returns_cargo_pkg_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
