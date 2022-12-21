// #![allow(warnings)]

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;


mod mailbox;
mod position;
mod engine;
mod board;
mod movegen;

use std::{io, env};

use movegen::MoveType;
use position::Position;
use std::time::{ Duration, Instant };


fn get_all_legal_positions(pos: &Position, depth: i32, nodes: &mut u64, promotions: &mut u64, castles: &mut u64, en_passant: &mut u64, captures: &mut u64) -> () {
    let moves = pos.get_legal_moves();
    if depth == 0 || moves.len() == 0 {
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
                get_all_legal_positions(&p, depth - 1, nodes, promotions, castles, en_passant, captures);
            }
        
    }
    return;
}

fn perft(pos: &Position, depth: i32) {
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
            let mut illegal = true;
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

fn move_pos(p: &Position) -> io::Result<()> {
    let mut pos = p.clone();
    let stdin = io::stdin();
    let mut input1 = String::new();
    let mut input2 = String::new();

    loop {
        // let engine_mv = engine::choose_move(&pos);
        // pos = pos.new_position(engine_mv);
        // pos.print_board();
        // input1.clear();
        // input2.clear();

        println!("Move from:");
        stdin.read_line(&mut input1)?;
        println!("Move to:");
        stdin.read_line(&mut input2)?;
        let mut illegal = true;
        let (i, j) = Position::move_as_notation(&input1, &input2);

        for mv in pos.get_legal_moves() {
            if mv.from == i && mv.to == j {
                pos = pos.new_position(&mv);
                pos.print_board();
                illegal = false;
                break;
            }
        } 
        if illegal {
            println!("Move isn't legal!");
            input1.clear();
            input2.clear();
            continue;
        }
        // let engine_mv = engine::choose_move(&pos);
        // pos = pos.new_position(engine_mv);
        // pos.print_board();
        // input1.clear();
        // input2.clear();
    }

    Ok(())
}

fn game_loop() {
    let white_player = HumanPlayer;
    let black_player = EnginePlayer {depth: 4};
    let mut board = board::Board::new(Box::new(white_player), Box::new(black_player));

    loop {
        board.make_move();
        println!("{:?}", board.current_state.get_gamestate());
        board.current_state.position.print_board();
    }
}

fn main() {
    let mut pos = Position::new_starting();
//    let mut pos = Position::new_position_from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1");
    //let mut pos = Position::new_position_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    pos.print_board();
    // println!("{:?}", pos.defend_map);
    //println!("{:#?}", pos);
    // let idx: usize = 62;
    // let s = &pos.position[idx];
    // match s {
    //     position::Square::Piece(p) => {
    //         pos.print_board(pos.attack_map.get(&idx).unwrap());
    //         println!("{:p}", p)

    //     }
    //     _ => {}
    // }
    // todo move king to 31 and see if it allows a legal move

    let start = Instant::now();
    let mut nodes: u64 = 0;
    let mut promotions: u64 = 0;
    //println!("{:x}", pos.pos_hash());
    perft(&pos, 5);

    // let legal_moves = &pos.legal_moves;

    // for (i, mv) in legal_moves {
    //     for j in mv {
    //         let p = pos.new_move(*i, *j);
    //         positions.push(p);
    //     }
    // }

    // for p in &positions {
    //     p.print_board(&Vec::new());
    //     println!();
    // }
    //println!("{}", engine::negatedMax(&pos, 4));
    let duration = start.elapsed();
    println!("nodes: {}", nodes);
    println!("promotions: {}", promotions);

    println!("Time elapsed is: {:?}", duration);

    game_loop();
    //move_pos(&pos);
}