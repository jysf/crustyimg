// Compare oxideav-avif decoded planes to re_rav1d decoded planes, byte for byte.
use re_rav1d::dav1d::{Decoder, Picture, PlanarImageComponent, Settings};
use std::io::Cursor;
use std::time::Instant;

fn rav1d(obus: &[u8]) -> Result<Picture, String> {
    let mut s = Settings::new();
    s.set_n_threads(1);
    s.set_max_frame_delay(1);
    let mut d = Decoder::with_settings(&s).map_err(|e| e.to_string())?;
    d.send_data(obus.to_vec(), None, None, None).map_err(|e| e.to_string())?;
    for _ in 0..8 {
        match d.get_picture() {
            Ok(p) => return Ok(p),
            Err(e) if e.is_again() => { d.send_pending_data().map_err(|e| e.to_string())?; }
            Err(e) => return Err(e.to_string()),
        }
    }
    Err("no frame".into())
}

/// Rows of `w*bps` bytes from a strided plane.
fn rows(data: &[u8], stride: usize, w: usize, h: usize, bps: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(w * h * bps);
    for y in 0..h { v.extend_from_slice(&data[y * stride..y * stride + w * bps]); }
    v
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mutate = args.iter().any(|a| a == "--mutate"); // negative control
    let dump = args.iter().position(|a| a == "--dump").map(|i| args[i + 1].clone());
    for path in args.iter().filter(|a| a.ends_with(".avif")) {
        let name = path.rsplit('/').next().unwrap();
        let bytes = std::fs::read(path).unwrap();
        let t0 = Instant::now();
        let ox = std::panic::catch_unwind(|| oxideav_avif::decode(&bytes));
        let t_ox = t0.elapsed().as_secs_f64() * 1e3;
        let ox = match ox {
            Err(_) => { println!("{name}: OXIDEAV PANIC"); continue; }
            Ok(Err(e)) => { println!("{name}: oxideav error: {e}"); continue; }
            Ok(Ok(i)) => i,
        };
        if let Some(dir) = &dump { std::fs::write(format!("{dir}/{name}.ox.y"), &ox.planes[0].data).ok(); }
        let t1 = Instant::now();
        let parsed = std::panic::catch_unwind(|| avif_parse::read_avif(&mut Cursor::new(&bytes)));
        let parsed = match parsed { Ok(Ok(p)) => p, Ok(Err(e)) => { println!("{name}: ox {}x{} {:?} ok ({t_ox:.1} ms) | avif-parse error: {e}", ox.width, ox.height, ox.format); continue; } Err(_) => { println!("{name}: avif-parse panic"); continue; } };
        let pic = match rav1d(&parsed.primary_item) { Ok(p) => p, Err(e) => { println!("{name}: rav1d error {e}"); continue; } };
        let alpha = parsed.alpha_item.as_ref().map(|a| rav1d(a).unwrap());
        let t_rd = t1.elapsed().as_secs_f64() * 1e3;
        let (w, h) = (pic.width() as usize, pic.height() as usize);
        let depth = pic.bit_depth();
        let bps = if depth > 8 { 2 } else { 1 };
        let lay = pic.pixel_layout();
        let (sx, sy) = match format!("{lay:?}").as_str() { "I420" => (1, 1), "I422" => (1, 0), "I444" => (0, 0), _ => (9, 9) };
        let mut comps = vec![(PlanarImageComponent::Y, w, h, &pic)];
        if sx != 9 {
            let (cw, ch) = ((w + sx) >> sx, (h + sy) >> sy);
            comps.push((PlanarImageComponent::U, cw, ch, &pic));
            comps.push((PlanarImageComponent::V, cw, ch, &pic));
        }
        // (bytes, row_bytes, rows)
        let mut ref_planes: Vec<(Vec<u8>, usize, usize)> = comps.iter().map(|(c, pw, ph, p)| (rows(&p.plane(*c), p.stride(*c) as usize, *pw, *ph, bps), pw * bps, *ph)).collect();
        if let Some(a) = &alpha {
            let ab = if a.bit_depth() > 8 { 2 } else { 1 };
            ref_planes.push((rows(&a.plane(PlanarImageComponent::Y), a.stride(PlanarImageComponent::Y) as usize, w, h, ab), w * ab, h));
        }
        let geo_ok = ox.width as usize == w && ox.height as usize == h;
        let mut verdict = Vec::new();
        let mut all_exact = geo_ok && ox.planes.len() == ref_planes.len();
        for (i, (rp, rb, rh)) in ref_planes.iter().enumerate() {
            let Some(op) = ox.planes.get(i) else { verdict.push(format!("p{i}:MISSING")); all_exact = false; continue; };
            if op.stride < *rb || op.data.len() < op.stride * (rh - 1) + rb { verdict.push(format!("p{i}:geom stride {} len {} want row {rb}x{rh}", op.stride, op.data.len())); all_exact = false; continue; }
            let mut o = rows(&op.data, op.stride, *rb, *rh, 1);
            if mutate && i == 0 && !o.is_empty() { let m = o.len() / 2; o[m] ^= 1; }
            if o.len() != rp.len() { verdict.push(format!("p{i}:len {}≠{}", o.len(), rp.len())); all_exact = false; continue; }
            let diff = o.iter().zip(rp).filter(|(a, b)| a != b).count();
            let maxd = o.iter().zip(rp).map(|(a, b)| (*a as i32 - *b as i32).abs()).max().unwrap_or(0);
            if diff > 0 { all_exact = false; verdict.push(format!("p{i}:{diff} bytes differ (max {maxd})")); }
        }
        println!("{name}: {w}x{h} {lay:?} {depth}b alpha={} | {} | ox {t_ox:.1} ms, rav1d {t_rd:.1} ms {}",
            alpha.is_some(), if all_exact { "BYTE-EXACT".to_string() } else { format!("MISMATCH geo_ok={geo_ok} ox_planes={} ref_planes={}", ox.planes.len(), ref_planes.len()) }, verdict.join(" "));
    }
}
