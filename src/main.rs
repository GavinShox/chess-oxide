// #![allow(warnings)]
#![allow(unused_must_use)]

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};
use chess::*;
// use std::{ io };
// use std::time::{ Instant };


// struct HumanPlayer;

// impl Player for HumanPlayer {
//     fn get_move(&self, bstate: &BoardState) -> Move {
//         let stdin = io::stdin();
//         let mut input1 = String::new();
//         let mut input2 = String::new();

//         loop {
//             println!("Move from:");
//             stdin.read_line(&mut input1);
//             println!("Move to:");
//             stdin.read_line(&mut input2);
//             let _illegal = true;
//             let (i, j) = Position::move_as_notation(&input1, &input2);

//             for mv in &bstate.legal_moves {
//                 if mv.from == i && mv.to == j {
//                     return *mv;
//                 }
//             }
//             println!("Move isn't legal!");
//             input1.clear();
//             input2.clear();
//             continue;
//         }
//     }
// }



// fn game_loop() {
//     let white_player = RandomPlayer;
//     let black_player = EnginePlayer { depth: 4 };
//     let mut board = Board::new(Box::new(white_player), Box::new(black_player));

//     loop {
//         match board.make_move() {
//             Ok(_) => {}
//             Err(e) => {
//                 println!("{:?}", e);
//                 break;
//             }
//         }
//         let game_state = board.current_state.get_gamestate();
//         println!("Game state: {:?}", game_state);

//         board.current_state.position.print_board();

//         if game_state != GameState::Active && game_state != GameState::Check {
//             println!("Game over, gamestate: {:?}", game_state);
//             println!("{:#?}", board.current_state);
//             break;
//         }
//     }
// }

// fn main() {
//     let pos = Position::new_starting();
//     //let mut pos = Position::new_position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
//     //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
//     pos.print_board();

//     perft(&pos, 5);

//     //game_loop();
// }


fn main() {

}