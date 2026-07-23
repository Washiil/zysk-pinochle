use zysk_pinochle::bench;

/// Minimum acceptable single-threaded throughput (games/sec) in release mode.
/// If a change drops below this, investigate before proceeding.
const MIN_THROUGHPUT: f64 = 400_000.0;

/// Warmup games to run before measuring (cache warmup).
const WARMUP_GAMES: usize = 1_000;

/// Measured sample size.
const SAMPLE_GAMES: usize = 10_000;

#[test]
#[ignore = "run with: cargo test --release --test regression -- --ignored --nocapture"]
fn throughput_regression() {
    // Warmup
    bench::throughput(WARMUP_GAMES);

    // Measure
    let games_per_sec = bench::throughput(SAMPLE_GAMES);

    eprintln!("Throughput: {:.0} games/sec (minimum: {:.0})", games_per_sec, MIN_THROUGHPUT);

    assert!(
        games_per_sec >= MIN_THROUGHPUT,
        "Performance regression: {:.0} games/sec is below the minimum of {:.0} games/sec.\n\
         Run `cargo run --release --bin benchmark` for a throughput measurement.",
        games_per_sec,
        MIN_THROUGHPUT,
    );
}
