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
//! generic JPEG-quality binary search powering `shrink`'s auto-quality (DEC-019).
//! SPEC-026 adds the [`metadata`] module: the container lane (`strip` +
//! `clean --gps`), editing container bytes without re-decoding pixels (DEC-003,
//! DEC-029).
//! SPEC-030 adds the [`text`] module: pure glyph rasterization (`ab_glyph` +
//! bundled Go font) that turns a string into a transparent RGBA overlay for
//! `watermark --text`, then reuses the SPEC-029 compositing path (DEC-032).

pub mod cli;
pub mod error;
pub mod image;
pub mod metadata;
pub mod operation;
pub mod pipeline;
pub mod quality;
pub mod recipe;
pub mod sink;
pub mod source;
pub mod text;

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
