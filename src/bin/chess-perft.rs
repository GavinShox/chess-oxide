use std::time::{Duration, Instant};

use env_logger::{Builder, Target};

fn main() {
    // initialise logger
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();

    let pos = chess::Position::new_starting();
    let board = chess::board::Board::new();

    let pos_perft_iterations = 10;
    let mut total_pos_perft_time = Duration::new(0, 0);
    for i in 0..pos_perft_iterations {
        let start = Instant::now();
        chess::perft(&pos, 5);
        let duration = start.elapsed();
        total_pos_perft_time += duration;
        println!(
            "Time elapsed in position perft iteration {}: {:?}",
            i + 1,
            duration
        );
    }

    let engine_iterations = 5;
    let mut total_engine_time = Duration::new(0, 0);
    for i in 0..engine_iterations {
        let mut tt = chess::TT::new();
        let start = Instant::now();
        chess::engine_perft(board.get_current_state(), 7, &mut tt);
        let duration = start.elapsed();
        total_engine_time += duration;
        println!(
            "Time elapsed in engine perft iteration {}: {:?}\n",
            i + 1,
            duration
        );
    }
    println!("Total time elapsed in position perft: {:?} (after {} iterations)\nAverage time per iteration: {:?}", total_pos_perft_time, pos_perft_iterations, total_pos_perft_time / pos_perft_iterations);
    println!();
    println!("Total time elapsed in engine perft: {:?} (after {} iterations)\nAverage time per iteration: {:?}", total_engine_time, engine_iterations, total_engine_time / engine_iterations);
}
