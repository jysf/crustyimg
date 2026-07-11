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

    /// The image exceeds the configured decode resource limits (e.g. a dimension
    /// or allocation cap — DEC-034). The input is rejected before decoding.
    #[error("image exceeds decode limits: {0}")]
    LimitsExceeded(String),

    /// The input's format was recognized, but its DECODER is not compiled into
    /// this build (a feature-gated codec — today only HEIC, behind `--features
    /// heic`, DEC-052/DEC-056). Maps to exit 4. Leads with the fix most users can
    /// act on — pre-convert to a widely supported format — since the common case
    /// is a released/`cargo install` binary a user cannot easily rebuild (HEIC is
    /// patent/AGPL-walled off the default path); the `--features` hint is kept for
    /// those building from source. The encode-side twin is
    /// [`crate::sink::SinkError::CodecNotBuilt`].
    #[error(
        "{codec} decoding isn't built into this crustyimg; convert the file to a \
         supported format (JPEG, PNG, or WebP) first, or rebuild with --features {feature}"
    )]
    CodecNotBuilt {
        codec: &'static str,
        feature: &'static str,
    },
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

    #[test]
    fn limits_exceeded_carries_message() {
        let err = ImageError::LimitsExceeded("too big".to_string());
        assert!(err.to_string().contains("too big"));
    }

    /// The message must (1) name the codec, (2) lead with the actionable fix for
    /// the common case — pre-convert to a supported format — and (3) keep the
    /// `--features` hint for source builders. A released binary's user cannot
    /// rebuild, so "convert first" is the advice that actually helps them.
    #[test]
    fn codec_not_built_leads_with_convert_and_keeps_feature() {
        let err = ImageError::CodecNotBuilt {
            codec: "HEIC",
            feature: "heic",
        };
        let msg = err.to_string();
        assert!(msg.contains("HEIC"), "got {msg}");
        assert!(msg.contains("convert"), "got {msg}");
        assert!(msg.contains("--features heic"), "got {msg}");
    }
}
