use wasm_bindgen_test::*;
fn fnv(img: &oxideav_avif::AvifImage) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for pl in &img.planes { for b in &pl.data { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); } }
    h
}
// Expected values computed by a NATIVE oxideav decode, itself byte-exact vs re_rav1d (and libaom on gps_grid2x2).
const CASES: &[(&str, &[u8], u32, u32, usize, u64)] = &[
    ("fuji_d8_420_q60", include_bytes!("fuji_d8_420_q60.avif"), 448, 448, 3, 0x7c2fc50c4a908f40),
    ("fuji_alpha_10b", include_bytes!("fuji_alpha_10b.avif"), 448, 448, 4, 0xf9c5641b8e76bd6d),
    ("gps_grid2x2", include_bytes!("gps_grid2x2.avif"), 2100, 1500, 1, 0x29a0affec847daa7),
    ("ravif_fuji_q40", include_bytes!("ravif_fuji_q40.avif"), 448, 448, 3, 0x4bf162da143302de),
];
#[wasm_bindgen_test]
fn decodes_byte_identical_to_native() {
    for (name, bytes, w, h, np, want) in CASES {
        let img = oxideav_avif::decode(bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!((img.width, img.height, img.planes.len()), (*w, *h, *np), "{name} geometry");
        assert_eq!(fnv(&img), *want, "{name} plane hash");
        assert_eq!(img.to_rgba8().len(), (*w * *h * 4) as usize);
    }
}
#[wasm_bindgen_test]
fn negative_control_hash_detects_one_byte() {
    let mut img = oxideav_avif::decode(CASES[0].1).unwrap();
    img.planes[0].data[1000] ^= 1;
    assert_ne!(fnv(&img), CASES[0].5);
}
