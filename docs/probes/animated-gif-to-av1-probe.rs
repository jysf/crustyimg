// Animated GIF -> AV1 sizing probe. Full pure-Rust path, no C, no ffmpeg.
use crustyimg::image::Image;
use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, ImageFormat};
use rav1e::prelude::*;
use std::fs::File;
use std::io::BufReader;

/// BT.709 full-range RGB -> YUV420, the layout a real AVIF sequence encoder uses.
fn to_yuv420(img: &DynamicImage) -> (Vec<u8>, Vec<u8>, Vec<u8>, usize, usize) {
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as usize, rgb.height() as usize);
    let (cw, ch) = ((w + 1) / 2, (h + 1) / 2);
    let mut y = vec![0u8; w * h];
    let mut u = vec![0u8; cw * ch];
    let mut v = vec![0u8; cw * ch];
    for j in 0..h {
        for i in 0..w {
            let p = rgb.get_pixel(i as u32, j as u32).0;
            let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
            y[j * w + i] = (0.2126 * r + 0.7152 * g + 0.0722 * b).round().clamp(0.0, 255.0) as u8;
        }
    }
    // 2x2 box-average chroma
    for j in 0..ch {
        for i in 0..cw {
            let (mut su, mut sv, mut n) = (0f32, 0f32, 0f32);
            for dy in 0..2 {
                for dx in 0..2 {
                    let (x, yy) = (i * 2 + dx, j * 2 + dy);
                    if x >= w || yy >= h { continue; }
                    let p = rgb.get_pixel(x as u32, yy as u32).0;
                    let (r, g, b) = (p[0] as f32, p[1] as f32, p[2] as f32);
                    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    su += (b - l) / 1.8556 + 128.0;
                    sv += (r - l) / 1.5748 + 128.0;
                    n += 1.0;
                }
            }
            u[j * cw + i] = (su / n).round().clamp(0.0, 255.0) as u8;
            v[j * cw + i] = (sv / n).round().clamp(0.0, 255.0) as u8;
        }
    }
    (y, u, v, w, h)
}

fn encode(frames: &[DynamicImage], q: usize, speed: u8) -> (usize, usize, usize, Vec<Vec<u8>>) {
    let (_, _, _, w, h) = to_yuv420(&frames[0]);
    let mut cfg = EncoderConfig::default();
    cfg.width = w;
    cfg.height = h;
    cfg.speed_settings = SpeedSettings::from_preset(speed);
    cfg.chroma_sampling = ChromaSampling::Cs420;
    cfg.quantizer = q;
    cfg.time_base = Rational::new(1, 20);
    let mut ctx: Context<u8> = Config::new().with_encoder_config(cfg).new_context().unwrap();
    for f in frames {
        let (y, u, v, _, _) = to_yuv420(f);
        let mut fr = ctx.new_frame();
        fr.planes[0].copy_from_raw_u8(&y, w, 1);
        fr.planes[1].copy_from_raw_u8(&u, (w + 1) / 2, 1);
        fr.planes[2].copy_from_raw_u8(&v, (w + 1) / 2, 1);
        ctx.send_frame(fr).unwrap();
    }
    ctx.flush();
    let (mut n, mut bytes, mut keys, mut obus) = (0, 0, 0, Vec::new());
    loop {
        match ctx.receive_packet() {
            Ok(p) => { n += 1; bytes += p.data.len();
                       if p.frame_type == FrameType::KEY { keys += 1 }
                       obus.push(p.data.to_vec()); }
            Err(EncoderStatus::Encoded) => continue,
            Err(EncoderStatus::LimitReached) => break,
            Err(e) => panic!("rav1e: {e:?}"),
        }
    }
    (n, bytes, keys, obus)
}

/// re_rav1d Picture (YUV420) -> RGB, BT.709 full range. Inverse of `to_yuv420`.
fn pic_to_rgb(pic: &re_rav1d::dav1d::Picture) -> DynamicImage {
    use re_rav1d::dav1d::PlanarImageComponent as C;
    let (w, h) = (pic.width() as usize, pic.height() as usize);
    let (yp, up, vp) = (pic.plane(C::Y), pic.plane(C::U), pic.plane(C::V));
    let (ys, us) = (pic.stride(C::Y) as usize, pic.stride(C::U) as usize);
    let mut out = vec![0u8; w * h * 3];
    for j in 0..h {
        for i in 0..w {
            let y = yp[j * ys + i] as f32;
            let u = up[(j / 2) * us + i / 2] as f32 - 128.0;
            let v = vp[(j / 2) * us + i / 2] as f32 - 128.0;
            let r = y + 1.5748 * v;
            let b = y + 1.8556 * u;
            let g = y - (0.2126 * 1.5748 * v + 0.0722 * 1.8556 * u) / 0.7152;
            let o = (j * w + i) * 3;
            out[o]     = r.round().clamp(0.0, 255.0) as u8;
            out[o + 1] = g.round().clamp(0.0, 255.0) as u8;
            out[o + 2] = b.round().clamp(0.0, 255.0) as u8;
        }
    }
    DynamicImage::ImageRgb8(image::RgbImage::from_raw(w as u32, h as u32, out).unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1) {
        let gif_bytes = std::fs::read(&path)?;
        let dec = GifDecoder::new(BufReader::new(File::open(&path)?))?;
        let frames: Vec<DynamicImage> = dec.into_frames().collect::<Result<Vec<_>, _>>()?
            .into_iter().map(|f| DynamicImage::ImageRgba8(f.into_buffer())).collect();

        // Prove the crustyimg Pipeline is on the path: identity recipe, every frame.
        let registry = crustyimg::operation::registry::OperationRegistry::with_builtins();
        let recipe = crustyimg::recipe::Recipe::from_toml(
            "version = \"1\"\n[[step]]\nop = \"identity\"\n")?;
        let mut piped = Vec::with_capacity(frames.len());
        for f in &frames {
            let img = Image::from_parts(f.clone(), ImageFormat::Gif, None);
            piped.push(recipe.build_pipeline(&registry)?.run(img)?.pixels().clone());
        }

        let name = std::path::Path::new(&path).file_name().unwrap().to_string_lossy();
        let (w, h) = (piped[0].width(), piped[0].height());
        println!("\n=== {name}  {w}x{h}  {} frames  GIF {} B ===", piped.len(), gif_bytes.len());

        // One variable at a time: hold speed, sweep quantizer; then hold quantizer, sweep speed.
        let matrix: Vec<(String, usize, u8)> =
            [60usize, 80, 100, 120, 140].iter().map(|q| (format!("s6  q{q}"), *q, 6u8))
            .chain([1u8, 4, 6, 8, 10].iter().map(|sp| (format!("q100 s{sp}"), 100usize, *sp)))
            .collect();
        for (label, q, speed) in matrix {
            let (label, q, speed) = (label.as_str(), q, speed);
            let (n, bytes, keys, obus) = encode(&piped, q, speed);
            // verify with a decoder we did not write
            let mut d = re_rav1d::dav1d::Decoder::with_settings(&re_rav1d::dav1d::Settings::new())?;
            let mut got = 0usize;
            for o in &obus {
                d.send_data(o.clone(), None, None, None)?;
                loop { match d.get_picture() { Ok(_) => got += 1, Err(e) if e.is_again() => break,
                                               Err(e) => return Err(format!("{e:?}").into()) } }
            }
            for _ in 0..32 { match d.get_picture() { Ok(_) => got += 1, Err(_) => break } }

            // Perceptual check: decode frame 0 back and score it against the source.
            let mut d2 = re_rav1d::dav1d::Decoder::with_settings(&re_rav1d::dav1d::Settings::new())?;
            d2.send_data(obus[0].clone(), None, None, None)?;
            let mut score = f64::NAN;
            for _ in 0..8 {
                match d2.get_picture() {
                    Ok(pic) => { score = crustyimg::quality::score(
                                     &piped[0], &pic_to_rgb(&pic))?; break }
                    Err(e) if e.is_again() => { d2.send_pending_data().ok(); }
                    Err(_) => break,
                }
            }
            let ratio = gif_bytes.len() as f64 / bytes as f64;
            println!("  {label:20} av1={bytes:>8} B  packets={n:>3} keys={keys:>2}  \
                      decoded_back={got:>3}  ssim2(frame0)={score:5.1}  -> {ratio:5.1}x smaller than GIF");
            assert_eq!(got, n, "frame count did not survive the round trip");
        }
    }
    Ok(())
}
