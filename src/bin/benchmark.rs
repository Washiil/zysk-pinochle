use rayon::prelude::*;
use std::time::Instant;
use zysk_pinochle::bench;

fn main() {
    const GAMES: usize = 50_000;

    println!("Benchmarking {} games...", GAMES);
    let start = Instant::now();

    (0..GAMES).into_par_iter().for_each(|_| {
        bench::simulate_one_game();
    });

    let elapsed = start.elapsed();
    let per_game = elapsed / GAMES as u32;
    let per_second = GAMES as f64 / elapsed.as_secs_f64();

    println!("Total: {:?}", elapsed);
    println!("Per game: {:?}", per_game);
    println!("Throughput: {:.0} games/sec", per_second);
}
