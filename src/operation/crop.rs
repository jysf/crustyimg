#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ::image::{
        DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, ImageFormat, LumaA, Rgb, Rgba,
    };

    use super::*;
    use crate::image::Image;

    // ── Helpers ─────────────────────────────────────────────────────────────

    fn value_of(v: &str) -> toml::Value {
        // `v` is a TOML value as a recipe author writes it: `0.5`, `0`, `"16:9"`.
        let table: toml::Table = format!("v = {v}").parse().expect("a TOML value");
        table["v"].clone()
    }

    fn params(pairs: &[(&str, &str)]) -> OperationParams {
        OperationParams::from_map(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), value_of(v)))
                .collect::<BTreeMap<_, _>>(),
        )
    }

    fn crop(pairs: &[(&str, &str)]) -> Crop {
        Crop::from_params(&params(pairs)).expect("params are valid")
    }

    fn wrap(px: DynamicImage) -> Image {
        Image::from_parts(px, ImageFormat::Png, None)
    }

    /// An rgb8 image whose every pixel is unique (for dimensions under 4096).
    fn unique_rgb8(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |x, y| {
            Rgb([x as u8, y as u8, (((x >> 8) << 4) | (y >> 8)) as u8])
        }))
    }

    fn unique_rgb8_pixel(x: u32, y: u32) -> Rgb<u8> {
        Rgb([x as u8, y as u8, (((x >> 8) << 4) | (y >> 8)) as u8])
    }

    fn apply_to(c: &Crop, px: DynamicImage) -> DynamicImage {
        c.apply(wrap(px))
            .expect("crop applies")
            .pixels()
            .clone()
    }

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        /// A uniform draw in [0, 1).
        fn unit(&mut self) -> f64 {
            (self.next() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
        }
    }

    // ── Rectangle maths ─────────────────────────────────────────────────────

    #[test]
    fn rect_edges_exact_example() {
        assert_eq!(
            rect_edges(0.123, 0.2345, 0.5, 0.4321, 1200, 800),
            (148, 188, 748, 533)
        );
        assert_eq!(
            rect_edges(0.123, 0.2345, 0.5, 0.4321, 6000, 4000),
            (738, 938, 3738, 2666)
        );
    }

    #[test]
    fn crop_edges_within_half_a_pixel() {
        let mut cases = 0u64;
        for size in 1u32..=64 {
            for n in [97u32, 8] {
                for i in 0..=n {
                    for j in 1..=n {
                        let x = f64::from(i) / f64::from(n);
                        let width = f64::from(j) / f64::from(n);
                        if x + width > 1.0 {
                            continue;
                        }
                        cases += 1;
                        // Both axes through one call: the same fractions on a
                        // square image read the horizontal and vertical edges.
                        let (left, top, right, bottom) =
                            rect_edges(x, x, width, width, size, size);
                        let s = f64::from(size);
                        for (edge, exact, name) in [
                            (left, x * s, "left"),
                            (top, x * s, "top"),
                            (right, (x + width) * s, "right"),
                            (bottom, (x + width) * s, "bottom"),
                        ] {
                            assert!(
                                (f64::from(edge) - exact).abs() <= 0.5,
                                "{name}: x={x} width={width} size={size}: edge {edge}, exact {exact}"
                            );
                        }
                    }
                }
            }
        }
        // 64 sizes x (97·98/2 + 8·9/2) fraction pairs, counted once.
        assert_eq!(cases, 306_496, "every case ran");
    }

    #[test]
    fn crop_fraction_agrees_at_1200_and_6000() {
        let mut r = XorShift(0x5EED_0145);
        let mut pair = |r: &mut XorShift| loop {
            let (a, b) = (r.unit(), r.unit());
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            if hi - lo > 0.0 {
                return (lo, hi - lo);
            }
        };
        let mut cases = 0u32;
        let mut worst = 0.0f64;
        while cases < 200_000 {
            let (x, width) = pair(&mut r);
            let (y, height) = pair(&mut r);
            let small = rect_edges(x, y, width, height, 1200, 800);
            let large = rect_edges(x, y, width, height, 6000, 4000);
            for (s, l) in [
                (small.0, large.0),
                (small.1, large.1),
                (small.2, large.2),
                (small.3, large.3),
            ] {
                let drift = (f64::from(l) / 5.0 - f64::from(s)).abs();
                worst = worst.max(drift);
                assert!(
                    drift <= 0.6,
                    "x={x} y={y} width={width} height={height}: {s} at 1200, {l} at 6000"
                );
            }
            cases += 1;
        }
        assert_eq!(cases, 200_000);
        assert!(worst > 0.0, "the sweep saw rounding at all");
    }

    #[test]
    fn halves_round_up_and_fractions_read_as_f64() {
        // 0.0009375 * 1600 = 1.5: an f32 read gives 1.49999997 and rounds down.
        let c = crop(&[("x", "0.0009375"), ("y", "0"), ("width", "0.5"), ("height", "1")]);
        let src = unique_rgb8(1600, 1067);
        let out = apply_to(&c, src);
        assert_eq!((out.width(), out.height()), (800, 1067));
        assert_eq!(
            out.to_rgb8().get_pixel(0, 0),
            &unique_rgb8_pixel(2, 0),
            "left edge is 2, not 1"
        );

        // 0.25 * 1730 = 432.5 rounds up to 433.
        let c = crop(&[("x", "0.1"), ("y", "0.25"), ("width", "0.5"), ("height", "0.4321")]);
        let out = apply_to(&c, unique_rgb8(2600, 1730));
        assert_eq!((out.width(), out.height()), (1300, 747));
        assert_eq!(out.to_rgb8().get_pixel(0, 0), &unique_rgb8_pixel(260, 433));
    }

    // ── Aspect maths ────────────────────────────────────────────────────────

    #[test]
    fn aspect_fit_worked_example() {
        assert_eq!(aspect_size(5, 4, 2600, 1730), (2163, 1730));
        assert_eq!(aspect_size(5, 4, 1730, 2600), (1730, 1384));
        assert_eq!(aspect_size(16, 9, 2600, 1730), (2600, 1463));
        assert_eq!(aspect_size(32, 18, 2600, 1730), (2600, 1463));
        assert_eq!(aspect_size(10000, 1, 1600, 1067), (1600, 0));
        assert_eq!(aspect_size(1, 10000, 1600, 1067), (0, 1067));
    }

    #[test]
    fn aspect_never_exceeds_the_image() {
        let mut checked = 0u32;
        for (a, b) in [(1u32, 1u32), (16, 9), (9, 16), (239, 100), (65535, 1), (1, 65535)] {
            for iw in [1u32, 2, 3, 7, 64, 1067, 1600, 50_000] {
                for ih in [1u32, 2, 5, 48, 1067, 2600, 50_000] {
                    let (cw, ch) = aspect_size(a, b, iw, ih);
                    assert!(cw <= iw && ch <= ih, "{a}:{b} on {iw}x{ih} gave {cw}x{ch}");
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 6 * 8 * 7);
    }

    #[test]
    fn aspect_gravity_places_all_nine() {
        // (image w, image h, aspect, gravity, expected offset, expected size)
        let landscape = (2600u32, 1730u32, "5:4", (2163u32, 1730u32));
        let portrait = (1730u32, 2600u32, "5:4", (1730u32, 1384u32));
        let names = [
            "center", "north", "south", "east", "west", "northeast", "northwest", "southeast",
            "southwest",
        ];
        let landscape_x = [218, 218, 218, 437, 0, 437, 0, 437, 0];
        let portrait_y = [608, 0, 1216, 608, 608, 0, 0, 1216, 1216];

        let mut rows: Vec<(u32, u32, &str, &str, (u32, u32), (u32, u32))> = Vec::new();
        for (i, name) in names.iter().enumerate() {
            rows.push((
                landscape.0,
                landscape.1,
                landscape.2,
                name,
                (landscape_x[i], 0),
                landscape.3,
            ));
            rows.push((
                portrait.0,
                portrait.1,
                portrait.2,
                name,
                (0, portrait_y[i]),
                portrait.3,
            ));
        }
        rows.push((2600, 1730, "16:9", "center", (0, 133), (2600, 1463)));

        let mut mismatches = Vec::new();
        for (w, h, aspect, gravity, (left, top), (cw, ch)) in &rows {
            let aspect_toml = format!("\"{aspect}\"");
            let gravity_toml = format!("\"{gravity}\"");
            let c = crop(&[("aspect", aspect_toml.as_str()), ("gravity", gravity_toml.as_str())]);
            let out = apply_to(&c, unique_rgb8(*w, *h));
            let got_px = *out.to_rgb8().get_pixel(0, 0);
            if (out.width(), out.height()) != (*cw, *ch) || got_px != unique_rgb8_pixel(*left, *top)
            {
                mismatches.push(format!(
                    "{gravity} on {w}x{h}: wanted {cw}x{ch} at ({left}, {top}), got {}x{} starting {got_px:?}",
                    out.width(),
                    out.height()
                ));
            }
        }
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        assert_eq!(rows.len(), 19, "9 gravities x 2 orientations + 16:9");
    }

    // ── Pixels and colour types ─────────────────────────────────────────────

    #[test]
    fn luma_crops_are_exact() {
        let c = crop(&[("x", "0.25"), ("y", "0.25"), ("width", "0.5"), ("height", "0.5")]);

        let l8 = DynamicImage::ImageLuma8(GrayImage::from_fn(64, 48, |x, y| {
            ::image::Luma([(x * 3 + y * 5) as u8])
        }));
        let out = apply_to(&c, l8.clone());
        assert_eq!(out.color(), l8.color());
        assert_eq!((out.width(), out.height()), (32, 24));
        let (s, o) = (l8.to_luma8(), out.to_luma8());
        for j in 0..24 {
            for i in 0..32 {
                assert_eq!(o.get_pixel(i, j), s.get_pixel(16 + i, 12 + j), "L8 ({i}, {j})");
            }
        }

        let la8 = DynamicImage::ImageLumaA8(GrayAlphaImage::from_fn(64, 48, |x, y| {
            LumaA([(x * 3 + y * 5) as u8, (x + y * 7) as u8])
        }));
        let out = apply_to(&c, la8.clone());
        assert_eq!(out.color(), la8.color());
        let (s, o) = (la8.to_luma_alpha8(), out.to_luma_alpha8());
        for j in 0..24 {
            for i in 0..32 {
                assert_eq!(o.get_pixel(i, j), s.get_pixel(16 + i, 12 + j), "La8 ({i}, {j})");
            }
        }
    }

    #[test]
    fn identity_crop_is_byte_equal() {
        let c = crop(&[("x", "0"), ("y", "0"), ("width", "1"), ("height", "1")]);
        let rgb = unique_rgb8(37, 29);
        let rgba16 = DynamicImage::ImageRgba16(ImageBuffer::from_fn(37, 29, |x, y| {
            Rgba([x as u16 * 1000, y as u16 * 2000, (x + y) as u16 * 300, 65535 - x as u16])
        }));
        for (name, src) in [("rgb8", rgb), ("rgba16", rgba16)] {
            let out = apply_to(&c, src.clone());
            assert_eq!(out.color(), src.color(), "{name}: colour type");
            assert_eq!(out.as_bytes(), src.as_bytes(), "{name}: bytes");
        }
    }

    // ── The step as written ─────────────────────────────────────────────────

    #[test]
    fn params_round_trip_as_written() {
        let rect = params(&[("x", "0"), ("y", "0.25"), ("width", "1"), ("height", "0.5")]);
        let c = Crop::from_params(&rect).expect("rect builds");
        assert_eq!(c.params(), rect, "integers stay integers");

        let aspect = params(&[("aspect", "\"16:9\"")]);
        let c = Crop::from_params(&aspect).expect("aspect builds");
        assert_eq!(c.params(), aspect, "an absent gravity stays absent");
        assert!(!c.params().contains("gravity"));

        let with_gravity = params(&[("aspect", "\"16:9\""), ("gravity", "\"west\"")]);
        let c = Crop::from_params(&with_gravity).expect("aspect + gravity builds");
        assert_eq!(c.params(), with_gravity, "a given gravity is kept");
    }

    #[test]
    fn refusal_hook_is_not_used() {
        // A zero-pixel crop fails in `apply`; a skippable refusal would write
        // the uncropped image on every native pipeline.
        let c = crop(&[("x", "0.5"), ("y", "0"), ("width", "0.0001"), ("height", "1")]);
        let img = wrap(unique_rgb8(1600, 1067));
        assert!(c.refusal(&img).is_none());
        assert!(c.apply(img).is_err(), "and apply is where it fails");
    }
}
