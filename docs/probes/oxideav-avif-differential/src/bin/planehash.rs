fn main() {
    for p in std::env::args().skip(1) {
        let img = oxideav_avif::decode(&std::fs::read(&p).unwrap()).unwrap();
        let mut h: u64 = 0xcbf29ce484222325;
        for pl in &img.planes { for b in &pl.data { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); } }
        println!("(\"{}\", {}, {}, {}, 0x{h:016x}),", p.rsplit('/').next().unwrap(), img.width, img.height, img.planes.len());
    }
}
