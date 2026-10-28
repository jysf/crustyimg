// Mutation sweep: truncations + byte flips; count Ok / Err / panic. An abort kills the process (seen as exit code).
fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let a: Vec<String> = std::env::args().skip(1).collect();
    let n: usize = a[0].parse().unwrap();
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut rnd = || { seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17; seed };
    for path in &a[1..] {
        let base = std::fs::read(path).unwrap();
        let (mut ok, mut err, mut panic) = (0, 0, 0);
        for i in 0..n {
            let mut b = base.clone();
            if i % 3 == 0 { let cut = (rnd() as usize) % b.len(); b.truncate(cut); }
            else { for _ in 0..(1 + rnd() % 8) { let p = (rnd() as usize) % b.len(); b[p] = rnd() as u8; } }
            match std::panic::catch_unwind(|| oxideav_avif::decode(&b)) { Ok(Ok(_)) => ok += 1, Ok(Err(_)) => err += 1, Err(_) => panic += 1 }
        }
        println!("{}: {n} mutants -> ok {ok}, err {err}, PANIC {panic}", path.rsplit('/').next().unwrap());
    }
}
