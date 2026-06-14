//! crustyimg — a fast Rust CLI to view and transform images.
//!
//! This is the SPEC-001 scaffold: a minimal library root exposing the
//! package version. The pixel core (`image`, `operation`, `pipeline`, …)
//! lands in later specs (DEC-002); keep this tiny.

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
