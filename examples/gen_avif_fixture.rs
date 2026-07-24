//! Regenerate the committed AVIF decode fixtures (SPEC-058, SPEC-094).
//!
//! Produces `tests/fixtures/avif/solid_16x16.avif` (opaque) and
//! `tests/fixtures/avif/solid_16x16_alpha.avif` (real alpha channel), using
//! crustyimg's OWN AVIF encoder (the `image` crate's `avif` feature → `ravif`).
//! AVIF cannot be produced natively without an encoder feature, so the
//! fixtures are generated once and committed as static assets (AGENTS §12
//! forbids shelling out to ImageMagick).
//!
//! Regen (from the repo root):
//!
//! ```sh
//! cargo run --example gen_avif_fixture
//! ```
//!
//! `avif` is a default feature (SPEC-102, DEC-081), so a plain `cargo run` picks
//! it up with no flag. Only a build that drops it (e.g. `--no-default-features`)
//! makes this a no-op that prints how to run it (so the example still compiles
//! in the default `--all-targets` build either way).

fn main() {
    #[cfg(feature = "avif")]
    {
        use ::image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
        use std::io::Cursor;

        let img = RgbImage::from_pixel(16, 16, Rgb([200, 100, 50]));
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Avif)
            .expect("encode avif");
        let path = "tests/fixtures/avif/solid_16x16.avif";
        std::fs::write(path, buf.into_inner()).expect("write fixture");
        eprintln!("wrote {path}");

        // A real (non-empty) alpha channel, semi-transparent so decode must
        // actually merge the alpha OBU stream (SPEC-094's "valid AVIF with
        // alpha decodes unchanged" test needs a genuine alpha payload).
        let img = RgbaImage::from_pixel(16, 16, Rgba([200, 100, 50, 128]));
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ImageFormat::Avif)
            .expect("encode avif with alpha");
        let path = "tests/fixtures/avif/solid_16x16_alpha.avif";
        std::fs::write(path, buf.into_inner()).expect("write alpha fixture");
        eprintln!("wrote {path}");
    }
    #[cfg(not(feature = "avif"))]
    {
        eprintln!("run with: cargo run --example gen_avif_fixture --features avif");
    }
}
