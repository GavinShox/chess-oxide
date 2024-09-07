use std::{array, process::exit, time::{Duration, Instant}};

use env_logger::{Builder, Target};

use chess::magic;

fn main() {
    // pos_table: [[PositionHash; 12]; 64],
    // en_passant_table: [PositionHash; 8], // 8 possible files that an en passant move can be made
    // black_to_move: PositionHash,
    // white_castle_long: PositionHash,
    // black_castle_long: PositionHash,
    // white_castle_short: PositionHash,
    // black_castle_short: PositionHash,

    // let mut pos_table: [[u64; 12]; 64] = [[0; 12]; 64];
    // // offset_piece=64*kind_of_piece+ 8 * row + file;
    // for (i, s) in magic::POLYGLOT_ZOBRIST_KEYS.chunks(64).enumerate() {
    //     for (j, key) in s.iter().enumerate() {
    //         pos_table[j][i] = *key;
    //     }
    //     if i == 11 {
    //         break;
    //     }
    // }
    // let mut new_pos_table: [[u64; 12]; 64] = pos_table.clone();
    // for (i, a) in pos_table.iter().enumerate() {
    //     new_pos_table[i][0] = a[1];
    //     new_pos_table[i][1] = a[3];
    //     new_pos_table[i][2] = a[5];
    //     new_pos_table[i][3] = a[7];
    //     new_pos_table[i][4] = a[9];
    //     new_pos_table[i][5] = a[11];
    //     new_pos_table[i][6] = a[0];
    //     new_pos_table[i][7] = a[2];
    //     new_pos_table[i][8] = a[4];
    //     new_pos_table[i][9] = a[6];
    //     new_pos_table[i][10] = a[8];
    //     new_pos_table[i][11] = a[10];

    // }

    // let mut final_pos_table = new_pos_table.clone();
    // for (i, a) in new_pos_table.iter().enumerate() {
    //     let converted_idx = magic::polyglot_index_to_index(i);
    //     final_pos_table[converted_idx] = a.clone();
    // }

    // println!("{:?}", final_pos_table);

    // let white_castle_long = magic::POLYGLOT_ZOBRIST_KEYS[769];
    // let black_castle_long = magic::POLYGLOT_ZOBRIST_KEYS[771];
    // let white_castle_short = magic::POLYGLOT_ZOBRIST_KEYS[768];
    // let black_castle_short = magic::POLYGLOT_ZOBRIST_KEYS[770];

    // let ep_table: [u64; 8] = [
    //     magic::POLYGLOT_ZOBRIST_KEYS[772],
    //     magic::POLYGLOT_ZOBRIST_KEYS[773],
    //     magic::POLYGLOT_ZOBRIST_KEYS[774],
    //     magic::POLYGLOT_ZOBRIST_KEYS[775],
    //     magic::POLYGLOT_ZOBRIST_KEYS[776],
    //     magic::POLYGLOT_ZOBRIST_KEYS[777],
    //     magic::POLYGLOT_ZOBRIST_KEYS[778],
    //     magic::POLYGLOT_ZOBRIST_KEYS[779],
    // ];

    // let white_to_move = magic::POLYGLOT_ZOBRIST_KEYS[780];

    // println!("white_castle_long: {}", white_castle_long);
    // println!("black_castle_long: {}", black_castle_long);
    // println!("white_castle_short: {}", white_castle_short);
    // println!("black_castle_short: {}", black_castle_short);
    // println!("ep_table: {:?}", ep_table);
    // println!("white_to_move: {}", white_to_move);
    // exit(0);
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
        let start = Instant::now();
        //chess::perft(&pos, 5);
        chess::engine_perft(&board.current_state, 7);
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
