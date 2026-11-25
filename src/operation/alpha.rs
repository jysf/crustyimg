//! What alpha an output format can store, and the refusal for a step whose
//! alpha it can't.
//!
//! A step that adds pixels with alpha (a `pad` with a translucent fill) says so
//! through [`Operation::introduced_alpha`](super::Operation::introduced_alpha).
//! [`Pipeline::unstorable_alpha`](crate::pipeline::Pipeline::unstorable_alpha)
//! compares that with the output format's [`AlphaCapacity`] before anything
//! runs. The check is static: it reads the recipe and the format, never the
//! pixels, so an image already at a pad's aspect is refused the same way as one
//! that gets a border.
//!
//! Only alpha a step *introduces* is checked. Alpha the source image carried
//! and a format drops (an RGBA PNG written as JPEG) is out of scope here.
//!
//! Uses only `image::ImageFormat`, so it compiles in every feature set.

use std::fmt;

use ::image::ImageFormat;

/// How much alpha an output format keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaCapacity {
    /// No alpha channel: only a fully opaque pixel is stored as given (JPEG).
    None,
    /// One bit: alpha 0 and fully opaque are stored exactly, and any alpha in
    /// between is written opaque (GIF).
    Binary,
    /// Every alpha value is kept. AVIF is lossy in the value but keeps it.
    Full,
}

/// The alpha capacity of an output format.
pub fn alpha_capacity(format: ImageFormat) -> AlphaCapacity {
    match format {
        ImageFormat::Jpeg => AlphaCapacity::None,
        ImageFormat::Gif => AlphaCapacity::Binary,
        _ => AlphaCapacity::Full,
    }
}

/// Whether a format with this capacity stores `alpha` exactly as given.
pub fn stores(capacity: AlphaCapacity, alpha: u8) -> bool {
    match capacity {
        AlphaCapacity::None => alpha == u8::MAX,
        AlphaCapacity::Binary => alpha == 0 || alpha == u8::MAX,
        AlphaCapacity::Full => true,
    }
}

/// The alpha a step gives the pixels it adds, and the key that set it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntroducedAlpha {
    /// The alpha of every added pixel, 0 to 255.
    pub alpha: u8,
    /// The step key that set it, such as `color`.
    pub key: &'static str,
    /// That key's value as the recipe wrote it, such as `transparent`.
    pub written: String,
    /// The CLI flag that set the key, such as `--pad-color`, when the step was
    /// built from a flag; `None` for a recipe step.
    pub flag: Option<&'static str>,
}

/// A step's introduced alpha that the output format can't store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnstorableAlpha {
    /// The recipe step's index.
    pub step: usize,
    /// The step's op name.
    pub op: &'static str,
    pub introduced: IntroducedAlpha,
    pub format: ImageFormat,
}

impl fmt::Display for UnstorableAlpha {
    /// One line. A step built from a CLI flag names that flag, since the user
    /// typed no step; a recipe step names its index, op and key, and no flag,
    /// because the same line serves `apply`, `build` and the browser build.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let IntroducedAlpha {
            alpha,
            key,
            written,
            flag,
        } = &self.introduced;
        match flag {
            Some(flag) => write!(f, "{flag} {written:?} has alpha {alpha:02x}, and ")?,
            None => write!(
                f,
                "recipe step {} ('{}'): '{key}' = {written:?} has alpha {alpha:02x}, and ",
                self.step, self.op
            )?,
        }
        match alpha_capacity(self.format) {
            AlphaCapacity::Binary => write!(
                f,
                "GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP \
                 or AVIF instead"
            ),
            // `Full` formats are never refused; the same wording as `None` is
            // the honest fallback if one ever were.
            AlphaCapacity::None | AlphaCapacity::Full => write!(
                f,
                "{} output has no alpha channel; write PNG, WebP or AVIF instead, or make the \
                 colour opaque",
                format_name(self.format)
            ),
        }
    }
}

impl std::error::Error for UnstorableAlpha {}

fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Gif => "GIF",
        _ => "this format's",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHAS: [u8; 5] = [0x00, 0x01, 0x80, 0xfe, 0xff];

    #[test]
    fn capacity_and_stores_table() {
        // (format, which of the five alphas it stores)
        let rows: [(ImageFormat, [bool; 5]); 8] = [
            (ImageFormat::Png, [true; 5]),
            (ImageFormat::Jpeg, [false, false, false, false, true]),
            (ImageFormat::Gif, [true, false, false, false, true]),
            (ImageFormat::Bmp, [true; 5]),
            (ImageFormat::Tiff, [true; 5]),
            (ImageFormat::Ico, [true; 5]),
            (ImageFormat::WebP, [true; 5]),
            (ImageFormat::Avif, [true; 5]),
        ];
        for (format, want) in rows {
            let got = ALPHAS.map(|a| stores(alpha_capacity(format), a));
            assert_eq!(got, want, "{format:?}");
        }
    }

    fn refused(step: usize, written: &str, alpha: u8, format: ImageFormat) -> String {
        UnstorableAlpha {
            step,
            op: "pad",
            introduced: IntroducedAlpha {
                alpha,
                key: "color",
                written: written.to_owned(),
                flag: None,
            },
            format,
        }
        .to_string()
    }

    #[test]
    fn messages_verbatim() {
        assert_eq!(
            refused(0, "transparent", 0, ImageFormat::Jpeg),
            "recipe step 0 ('pad'): 'color' = \"transparent\" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque"
        );
        assert_eq!(
            refused(0, "#ff000080", 0x80, ImageFormat::Jpeg),
            "recipe step 0 ('pad'): 'color' = \"#ff000080\" has alpha 80, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque"
        );
        assert_eq!(
            refused(0, "#ff000080", 0x80, ImageFormat::Gif),
            "recipe step 0 ('pad'): 'color' = \"#ff000080\" has alpha 80, and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead"
        );
    }

    #[test]
    fn messages_name_the_flag_when_a_flag_set_it() {
        let line = |written: &str, alpha: u8, format: ImageFormat| {
            UnstorableAlpha {
                step: 1,
                op: "pad",
                introduced: IntroducedAlpha {
                    alpha,
                    key: "color",
                    written: written.to_owned(),
                    flag: Some("--pad-color"),
                },
                format,
            }
            .to_string()
        };
        assert_eq!(
            line("transparent", 0, ImageFormat::Jpeg),
            "--pad-color \"transparent\" has alpha 00, and JPEG output has no alpha channel; write PNG, WebP or AVIF instead, or make the colour opaque"
        );
        assert_eq!(
            line("#ff000080", 0x80, ImageFormat::Gif),
            "--pad-color \"#ff000080\" has alpha 80, and GIF output stores only alpha 00 or ff; use one of those, or write PNG, WebP or AVIF instead"
        );
    }
}
