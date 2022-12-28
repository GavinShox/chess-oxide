// #![allow(warnings)]

// #[global_allocator]
// static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;


mod mailbox;
mod position;
mod engine;
mod board;
mod movegen;

mod test;

use std::{io};

use board::Player;
use movegen::MoveType;
use position::Position;
use std::time::{ Instant };


fn get_all_legal_positions(pos: &Position, depth: i32, nodes: &mut u64, promotions: &mut u64, castles: &mut u64, en_passant: &mut u64, captures: &mut u64) {
    let moves = pos.get_legal_moves();
    if depth == 0 || moves.is_empty() {
        return;
    }
    for mv in moves {

            if depth == 1 {
                match mv.move_type {
                    MoveType::EnPassant(_) => *en_passant += 1,
                    MoveType::Promotion(_) => *promotions += 1,
                    MoveType::Castle(_) => *castles += 1,
                    MoveType::Capture => *captures += 1,
                    _ => {}
                }
                *nodes += 1;
            } else {
                let p = pos.new_position(mv);
                //let unmake_move = pos.make_move(mv);
                get_all_legal_positions(&p, depth - 1, nodes, promotions, castles, en_passant, captures);
            }
        
    }
    
}

fn perft(pos: &Position, depth: i32) -> u64{
    let mut nodes: u64 = 0;
    let mut promotions: u64 = 0;
    let mut castles: u64 = 0;
    let mut en_passant: u64 = 0;
    let mut captures: u64 = 0;

    let start = Instant::now();

    get_all_legal_positions(pos, depth, &mut nodes, &mut promotions, &mut castles, &mut en_passant, &mut captures);

    let duration = start.elapsed();

    println!("Perft at depth {} (took {:?} to complete):", depth, duration);
    println!("Nodes: {}", nodes);
    println!("Move types breakdown: ");
    println!("Promotions: {}", promotions);
    println!("Castles: {}", castles);
    println!("En Passant: {}", castles);
    println!("Captures: {}", captures);

    nodes
}

struct EnginePlayer {
    depth: i32
}

impl board::Player for EnginePlayer {
    fn get_move(&self, board_state: &board::BoardState) -> movegen::Move {
        *engine::choose_move(&board_state.position, self.depth)
    }
}

struct HumanPlayer;

impl board::Player for HumanPlayer {
    fn get_move(&self, bstate: & board::BoardState) -> movegen::Move {
        let stdin = io::stdin();
        let mut input1 = String::new();
        let mut input2 = String::new();

        loop {
            println!("Move from:");
            stdin.read_line(&mut input1);
            println!("Move to:");
            stdin.read_line(&mut input2);
            let _illegal = true;
            let (i, j) = Position::move_as_notation(&input1, &input2);

            for mv in &bstate.legal_moves {
                if mv.from == i && mv.to == j {
                    return *mv
                }
            } 
            println!("Move isn't legal!");
            input1.clear();
            input2.clear();
            continue;
        }
    }
}

struct RandomPlayer;
impl Player for RandomPlayer {
    fn get_move(&self, bstate: &board::BoardState) -> movegen::Move {
        *engine::random_move(&bstate.position)
    }
}

fn game_loop() {
    let white_player = RandomPlayer;
    let black_player = EnginePlayer {depth: 4};
    let mut board = board::Board::new(Box::new(white_player), Box::new(black_player));

    loop {
        match board.make_move() {
            Ok(_) => {},
            Err(e) => {println!("{:?}", e); break;},
        }
        let game_state = board.current_state.get_gamestate();
        println!("Game state: {:?}", game_state);

        board.current_state.position.print_board();

        if game_state != board::GameState::Active && game_state != board::GameState::Check {
            println!("Game over, gamestate: {:?}", game_state);
            println!("{:#?}", board.current_state);
            break;
        }
    }
}

fn main() {
    let pos = Position::new_starting();
    //let mut pos = Position::new_position_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    pos.print_board();

    perft(&pos, 5);

    //game_loop();
}

