//! Regenerate the committed AVIF decode fixture (SPEC-058).
//!
//! Produces `tests/fixtures/avif/solid_16x16.avif`, a 16×16 solid image, using
//! crustyimg's OWN AVIF encoder (the `image` crate's `avif` feature → `ravif`).
//! AVIF cannot be produced natively without an encoder feature, so the fixture
//! is generated once and committed as a static asset (AGENTS §12 forbids
//! shelling out to ImageMagick).
//!
//! Regen (from the repo root):
//!
//! ```sh
//! cargo run --example gen_avif_fixture --features avif
//! ```
//!
//! Without `--features avif` this is a no-op that prints how to run it (so the
//! example still compiles in the default `--all-targets` build).

fn main() {
    #[cfg(feature = "avif")]
    {
        use ::image::{DynamicImage, ImageFormat, Rgb, RgbImage};
        use std::io::Cursor;

        let img = RgbImage::from_pixel(16, 16, Rgb([200, 100, 50]));
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Avif)
            .expect("encode avif");
        let path = "tests/fixtures/avif/solid_16x16.avif";
        std::fs::write(path, buf.into_inner()).expect("write fixture");
        eprintln!("wrote {path}");
    }
    #[cfg(not(feature = "avif"))]
    {
        eprintln!("run with: cargo run --example gen_avif_fixture --features avif");
    }
}
