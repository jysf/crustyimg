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

pub mod error;
pub mod image;
pub mod operation;
pub mod pipeline;
pub mod sink;
pub mod source;

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
