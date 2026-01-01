//! Command-line interface for Pinochle bot simulation and benchmarking.

use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "benchmark" => run_benchmark(),
        "compare" => run_comparison(&args[2..]),
        "simulate" => run_simulation(&args[2..]),
        _ => {
            println!("Unknown command: {}", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Pinochle Bot - High Performance Simulation Framework");
    println!("\nUsage:");
    println!("  pinochle-cli benchmark              - Run performance benchmarks");
    println!("  pinochle-cli compare <a1> <a2>      - Compare two agents");
    println!("  pinochle-cli simulate <games>       - Run simulations");
    println!("\nAgent types: random, heuristic, mcts");
}

fn run_benchmark() {
    println!("=== Pinochle Bot Benchmark ===\n");

}

fn run_comparison(args: &[String]) {
    if args.len() < 2 {
        println!("Usage: pinochle-cli compare <agent1> <agent2>");
        println!("Agent types: random, heuristic, mcts");
        return;
    }
}

fn run_simulation(args: &[String]) {
    let num_games = if !args.is_empty() {
        args[0].parse().unwrap_or(1000)
    } else {
        1000
    };

    println!("Running {} games...\n", num_games);
}
