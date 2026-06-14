//! crustyimg — a fast Rust CLI to view and transform images.
//!
//! The library root. SPEC-001 shipped the std-only scaffold (`version()`);
//! SPEC-002 adds the canonical pixel core: typed [`error`]s and the [`image`]
//! module (the `Image` model + load/decode + metadata capture, DEC-002/003).
//! Further modules (`operation`, `pipeline`, …) land in later specs.

pub mod error;
pub mod image;

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
