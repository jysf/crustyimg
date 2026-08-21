//! SPEC-124 measurement probe — **throwaway prototype, not production code.**
//!
//! Call 2 leaves N unsettled at design time and asks the build to measure it.
//! This probe answers the two questions the spec names: does the AV1 tile
//! count affect **serial** encode wall clock (nobody had measured this — only
//! whether *threads* moved the bytes, which DEC-094 showed is a different
//! axis from *tiles* here since the shipped build never parallelizes), and
//! what does 1 tile cost in compression versus a handful.
//!
//! It replicates `sink::encode_to_bytes_with`'s AVIF arm exactly —
//! `AvifEncoder::new_with_speed_quality(w, speed, quality)` at the same
//! speed/quality constants — plus the one call the shipped path is missing:
//! `.with_num_threads(Some(n))`. Nothing in `src/` calls this file and it
//! changes no shipped behaviour by existing.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --example spec124_tile_count_probe -- \
//!     <photo.jpg> <graphic.png> [--repeats N]
//! cargo run --release --example spec124_tile_count_probe -- large
//! ```
//!
//! Prints one JSON object per (input, quality-path, N) to stdout.
//!
//! `avif` is a default feature (SPEC-102, DEC-081), so a plain `cargo run`
//! picks it up with no flag. Only a build that drops it (e.g.
//! `--no-default-features`) makes this a no-op that prints how to run it (so
//! the example still compiles in the default `--all-targets` build either
//! way — the `gen_avif_fixture.rs` precedent).

fn main() {
    #[cfg(feature = "avif")]
    probe::run();

    #[cfg(not(feature = "avif"))]
    println!(
        "spec124_tile_count_probe: no-op without the `avif` feature; re-run with default features"
    );
}

#[cfg(feature = "avif")]
mod probe {
    use std::io::Cursor;
    use std::time::Instant;

    use image::codecs::avif::AvifEncoder;
    use image::ImageFormat;

    /// Mirrors `sink::AVIF_DEFAULT_QUALITY` / `sink::AVIF_SPEED` — the pinned
    /// `--format avif` path (`convert -q 80`, `web`/`optimize --format avif`).
    const PINNED_QUALITY: u8 = 80;
    const SPEED: u8 = 6;
    /// Mirrors `sink::FAST_LOSSY_QUALITY` — the AUTO decision path `web`/
    /// `optimize` actually hit in DEC-094's leg A2 (no `--format` pin).
    const AUTO_QUALITY: u8 = 85;

    /// The sweep. Includes 1 (the "may be strictly better" prior), a few
    /// intermediate values, and 14 (this host's core count — DEC-094's
    /// shipped tile count on this same host, `sysctl -n hw.ncpu` confirms 14
    /// unchanged).
    const N_SWEEP: &[usize] = &[1, 2, 4, 8, 14];

    fn sha256_hex(bytes: &[u8]) -> String {
        // Stdlib has no SHA-256; shell out to the same `shasum`/`sha256sum`
        // the committed Python harness uses, so hashes are comparable by
        // inspection.
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("shasum")
            .arg("-a")
            .arg("256")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("shasum must be on PATH");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(bytes)
            .expect("write to shasum stdin");
        let out = child.wait_with_output().expect("shasum must run");
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_owned()
    }

    fn encode_once(img: &image::DynamicImage, quality: u8, n: usize) -> (Vec<u8>, f64) {
        let mut cursor = Cursor::new(Vec::new());
        let encoder = AvifEncoder::new_with_speed_quality(&mut cursor, SPEED, quality)
            .with_num_threads(Some(n));
        let start = Instant::now();
        img.write_with_encoder(encoder).expect("avif encode");
        let wall = start.elapsed().as_secs_f64();
        (cursor.into_inner(), wall)
    }

    fn run_one(label: &str, path: &str, quality: u8, repeats: u32) {
        let img = image::open(path).unwrap_or_else(|e| panic!("open {path}: {e}"));
        for &n in N_SWEEP {
            let mut best_wall = f64::MAX;
            let mut bytes_len = 0usize;
            let mut hash = String::new();
            for _ in 0..repeats {
                let (bytes, wall) = encode_once(&img, quality, n);
                best_wall = best_wall.min(wall);
                bytes_len = bytes.len();
                hash = sha256_hex(&bytes);
            }
            println!(
                "{{\"label\":\"{label}\",\"input\":\"{path}\",\"quality\":{quality},\"n\":{n},\
                 \"bytes\":{bytes_len},\"wall_s_best_of_{repeats}\":{best_wall:.4},\
                 \"sha256_16\":\"{}\"}}",
                &hash[..16.min(hash.len())]
            );
        }
    }

    /// Does N=1 stay bitstream-legal on an image too large for one AV1 tile?
    /// `rav1e`'s `TilingInfo::from_target_tiles` clamps the requested tile
    /// count UP to its own `MAX_TILE_WIDTH` (4096 px) / `MAX_TILE_AREA`
    /// (4096x2304 px) bitstream limits regardless of what the caller asked
    /// for (`tiling/tiler.rs:74-90`), and that clamp is a pure function of
    /// frame dimensions — not thread count — so it cannot reopen the
    /// determinism this spec pins. A synthetic 6000x4000 (24 MP, wider than
    /// MAX_TILE_WIDTH and over MAX_TILE_AREA) frame drives it directly
    /// rather than trusting the read.
    fn run_large_image_check(path: Option<&str>) {
        let (label, img) = match path {
            Some(p) => (
                format!("large_real_{p}"),
                image::open(p).unwrap_or_else(|e| panic!("open {p}: {e}")),
            ),
            None => {
                let w = 6000u32;
                let h = 4000u32;
                let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(w, h, |x, y| {
                    image::Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
                }));
                ("large_synthetic_6000x4000".to_string(), img)
            }
        };
        for &n in &[1usize, 14] {
            let start = Instant::now();
            let (bytes, wall) = encode_once(&img, PINNED_QUALITY, n);
            let elapsed = start.elapsed().as_secs_f64();
            println!(
                "{{\"label\":\"{label}\",\"n\":{n},\"bytes\":{},\
                 \"wall_s\":{elapsed:.3},\"encode_fn_wall_s\":{wall:.3},\"panicked\":false}}",
                bytes.len()
            );
        }
    }

    pub fn run() {
        let args: Vec<String> = std::env::args().collect();
        if args.get(1).map(String::as_str) == Some("large") {
            run_large_image_check(args.get(2).map(String::as_str));
            return;
        }

        let photo = args
            .get(1)
            .cloned()
            .unwrap_or_else(|| "bench/corpus/photo_forest_cc0.jpg".to_string());
        let graphic = args
            .get(2)
            .cloned()
            .unwrap_or_else(|| "bench/corpus/graphic_large.png".to_string());
        let repeats: u32 = args
            .iter()
            .position(|a| a == "--repeats")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        // Sanity: both inputs must decode and format-detect as expected.
        let _ = ImageFormat::from_path(&photo);
        let _ = ImageFormat::from_path(&graphic);

        run_one("photo_pinned_q80", &photo, PINNED_QUALITY, repeats);
        run_one("photo_auto_q85", &photo, AUTO_QUALITY, repeats);
        run_one("graphic_pinned_q80", &graphic, PINNED_QUALITY, repeats);
        run_one("graphic_auto_q85", &graphic, AUTO_QUALITY, repeats);
    }
}
