//! Typed library errors (DEC-007).
//!
//! This is the crate's first typed error surface. The library returns these
//! matchable `thiserror` enums; `anyhow` context and exit-code mapping live at
//! the binary boundary (a later spec). No `unwrap`/`expect`/`panic!` on
//! recoverable paths (constraint `no-unwrap-on-recoverable-paths`).
//!
//! SPEC-002 only needs the image-loading surface (`ImageError`). A later spec
//! may widen this into a unified crate-wide `Error` (DEC-007); for now the
//! crate `Result` alias is over `ImageError`.

use thiserror::Error;

/// Errors that can occur while loading and decoding an image.
///
/// Messages name the *failure*, not the input path: the path is recoverable
/// context the binary boundary adds via `anyhow` (a later spec), so library
/// messages stay path-agnostic.
#[derive(Debug, Error)]
pub enum ImageError {
    /// Reading the input failed (e.g. the file does not exist or is
    /// unreadable). Only the path-based load entry produces this.
    #[error("could not read image input")]
    Io(#[from] std::io::Error),

    /// The byte stream was recognized as an image format but the pixels could
    /// not be decoded (corrupt, truncated, or otherwise invalid data).
    #[error("could not decode image: {0}")]
    Decode(String),

    /// The byte stream's format could not be detected, or it is not a format
    /// the configured pure-Rust codecs can decode.
    #[error("unsupported or undetectable image format")]
    UnsupportedFormat,
}

/// The crate `Result` alias over [`ImageError`].
///
/// Later specs may widen the error type to a unified crate `Error` (DEC-007);
/// SPEC-002 only needs the image surface.
pub type Result<T> = std::result::Result<T, ImageError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_converts_via_from() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "nope");
        let err: ImageError = io.into();
        assert!(matches!(err, ImageError::Io(_)));
    }

    #[test]
    fn decode_error_carries_message() {
        let err = ImageError::Decode("bad chunk".to_string());
        assert!(err.to_string().contains("bad chunk"));
    }

    #[test]
    fn unsupported_format_has_message() {
        let err = ImageError::UnsupportedFormat;
        assert!(!err.to_string().is_empty());
    }
}
