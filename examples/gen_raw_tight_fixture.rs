//! Regenerate the committed "tight preview" RAW fixture (SPEC-113).
//!
//! Produces `tests/fixtures/raw/tight_preview.nef`: a little-endian TIFF header,
//! filler, then one embedded JPEG preview. Close to `synthetic_preview.nef` (see
//! `gen_raw_fixture.rs`) but deliberately thumbnail-free, and it exists for a
//! size relationship rather than for its shape.
//!
//! `optimize <file>.nef -o out.jpg` pins the output to the SAME format the RAW
//! preview's `source_format` reports (`Jpeg`), so it reaches the never-bigger
//! guard. The guard only consults the raw-bytes sniff after it has decided the
//! re-encode did not beat the source, i.e. only when
//!
//! ```text
//! re-encode bytes  >=  the whole .nef container's bytes
//! ```
//!
//! `synthetic_preview.nef` cannot satisfy that: its preview is a solid colour
//! encoded at the SAME default quality the re-encode uses, so the re-encode
//! comes back about the size of the preview alone (~712 B) while the container
//! carries a thumbnail and a header too (~1365 B). The comparison short-circuits
//! on size and the sniff is never reached — a test written against it passes
//! whether or not the sniff exists.
//!
//! This fixture inverts that: a high-frequency (incompressible) preview stored
//! at a LOW quality keeps the container small, while the default-quality
//! re-encode of those same pixels is much larger. The invariant is asserted at
//! generation time below, so a future regen that breaks it fails loudly here
//! instead of silently turning the test back into a no-op.
//!
//! Regen (from the repo root):
//!
//! ```sh
//! cargo run --example gen_raw_tight_fixture
//! ```

use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};

/// Candidate `(width, height, stored quality)` preview configurations, tried in
/// order; the first that clears [`REQUIRED_MARGIN`] is committed. Ordered
/// smallest-first so the fixture stays as small as it can be while still
/// exercising the branch.
///
/// The search exists because the winning ratio is not predictable by hand — a
/// fixed guess landed at 0.99x, just under the bar. Two measured findings shape
/// the list:
/// - Stored quality below ~2 is a dead lever: the encoder floors out and q1 and
///   q2 produce byte-identical output. So these vary dimensions, not quality.
/// - Every byte of container that is not the preview is charged to the
///   denominator and not to the re-encode, which is why this fixture carries no
///   thumbnail (see [`build_blob`]) and why larger dimensions help — they dilute
///   the header and filler.
const CANDIDATES: &[(u32, u32, u8)] = &[
    (96, 72, 2),
    (160, 120, 2),
    (224, 168, 2),
    (320, 240, 2),
    (448, 336, 2),
];

/// How much bigger the re-encode must be than the container for the fixture to
/// be worth committing. Margin, not equality, so encoder-version drift does not
/// silently walk the fixture back to a no-op.
const REQUIRED_MARGIN: f64 = 1.25;

/// A deterministic high-frequency RGB pattern — an integer hash of the pixel
/// coordinates, no `rand` dependency. Noise is the point: it is what makes a
/// default-quality re-encode expensive relative to a low-quality store.
fn detailed(w: u32, h: u32) -> RgbImage {
    RgbImage::from_fn(w, h, |x, y| {
        let mut v = x
            .wrapping_mul(374_761_393)
            .wrapping_add(y.wrapping_mul(668_265_263));
        v ^= v >> 13;
        v = v.wrapping_mul(1_274_126_177);
        v ^= v >> 16;
        Rgb([
            (v & 0xFF) as u8,
            ((v >> 8) & 0xFF) as u8,
            ((v >> 16) & 0xFF) as u8,
        ])
    })
}

/// Assemble the RAW-shaped container around one embedded preview.
///
/// Deliberately **no thumbnail**, unlike `synthetic_preview.nef`. A second
/// embedded JPEG is ~600 B of fixed overhead charged to the container but not to
/// the re-encode, and the achievable ratio ceiling here is only about 1.33x, so
/// that overhead is the difference between a fixture that works and one that
/// does not. Proving the scan skips a thumbnail in favour of the larger preview
/// is `synthetic_preview.nef`'s job and is tested separately; this fixture has a
/// different one. What it must keep is the `II*\0` TIFF header — that is what
/// makes `guess_format` report Tiff, which is precisely the mismatch against the
/// adopted `Jpeg` label that the sniff has to catch.
fn build_blob(preview: &[u8]) -> Vec<u8> {
    // Little-endian TIFF header: `II` + magic 42 + first-IFD offset (unused).
    let mut blob: Vec<u8> = vec![0x49, 0x49, 0x2A, 0x00, 0x08, 0x00, 0x00, 0x00];
    blob.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // filler
    blob.extend_from_slice(preview);
    blob.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]); // trailing junk
    blob
}

/// Encode `img` to JPEG at an explicit quality.
fn jpeg_at(img: &DynamicImage, quality: u8) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_with_encoder(JpegEncoder::new_with_quality(&mut out, quality))
        .expect("encode jpeg at quality");
    out.into_inner()
}

/// Encode `img` to JPEG at the encoder DEFAULT quality — the same call the sink
/// makes when the pinned path passes `quality: None`.
fn jpeg_default(img: &DynamicImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Jpeg)
        .expect("encode jpeg at default quality");
    out.into_inner()
}

fn main() {
    eprintln!("  dims      q   preview  container  re-encode   ratio");
    let mut chosen: Option<(u32, u32, u8, Vec<u8>)> = None;

    for &(w, h, q) in CANDIDATES {
        let preview = jpeg_at(&DynamicImage::ImageRgb8(detailed(w, h)), q);
        let blob = build_blob(&preview);

        // What the pinned path will actually produce: decode the STORED preview,
        // then re-encode at the default quality. Decoding first matters — the
        // re-encode sees the quantized pixels, not the originals.
        let decoded = image::load_from_memory(&preview).expect("stored preview must decode");
        let reencoded = jpeg_default(&decoded);
        let ratio = reencoded.len() as f64 / blob.len() as f64;

        eprintln!(
            "  {w:>3}x{h:<3} q{q}  {:>6} B  {:>7} B  {:>7} B   {ratio:.2}x{}",
            preview.len(),
            blob.len(),
            reencoded.len(),
            if ratio >= REQUIRED_MARGIN {
                "  <- chosen"
            } else {
                ""
            },
        );

        if ratio >= REQUIRED_MARGIN {
            chosen = Some((w, h, q, blob));
            break;
        }
    }

    let (w, h, q, blob) = chosen.unwrap_or_else(|| {
        panic!(
            "no candidate cleared {REQUIRED_MARGIN}x: the default-quality re-encode must \
             exceed the WHOLE container, else optimize's never-bigger guard short-circuits \
             on size and never reaches the raw-bytes sniff — the exact branch this fixture \
             exists to cover. Add a larger / lower-quality candidate to CANDIDATES."
        )
    });

    let path = "tests/fixtures/raw/tight_preview.nef";
    std::fs::create_dir_all("tests/fixtures/raw").expect("create fixture dir");
    std::fs::write(path, &blob).expect("write fixture");
    eprintln!(
        "\nwrote {path} ({} B) — preview {w}x{h} at q{q}\n\
         tests/input_raw.rs asserts these dimensions; update it if they change.",
        blob.len(),
    );
}
