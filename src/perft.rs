use std::time::Instant;

use crate::position::Position;
use crate::{engine, movegen::*, transposition, BoardState};

fn get_all_legal_positions(
    pos: &Position,
    depth: u8,
    nodes: &mut u64,
    promotions: &mut u64,
    castles: &mut u64,
    en_passant: &mut u64,
    captures: &mut u64,
) {
    let moves = pos.get_legal_moves();
    if depth == 0 || moves.is_empty() {
        return;
    }
    for mv in moves {
        match mv.move_type {
            MoveType::EnPassant(_) => {
                *en_passant += 1;
            }
            MoveType::Promotion(_, capture) => {
                *promotions += 1;
                if capture.is_some() {
                    *captures += 1;
                }
            }
            MoveType::Castle(_) => {
                *castles += 1;
            }
            MoveType::Capture(_) => {
                *captures += 1;
            }
            _ => {}
        }
        if depth == 1 {
            *nodes += 1;
        } else {
            let p = pos.new_position(mv);
            //let unmake_move = pos.make_move(mv);
            get_all_legal_positions(
                &p,
                depth - 1,
                nodes,
                promotions,
                castles,
                en_passant,
                captures,
            );
        }
    }
}

pub fn perft(pos: &Position, depth: u8) -> u64 {
    let mut nodes: u64 = 0;
    let mut promotions: u64 = 0;
    let mut castles: u64 = 0;
    let mut en_passant: u64 = 0;
    let mut captures: u64 = 0;

    let start = Instant::now();

    get_all_legal_positions(
        pos,
        depth,
        &mut nodes,
        &mut promotions,
        &mut castles,
        &mut en_passant,
        &mut captures,
    );

    let duration = start.elapsed();

    println!(
        "Perft at depth {} (took {:?} to complete):",
        depth, duration
    );
    println!(" - Nodes: {}", nodes);
    println!(" - Move types breakdown: ");
    println!(" - Promotions: {}", promotions);
    println!(" - Castles: {}", castles);
    println!(" - En Passant: {}", en_passant);
    println!(" - Captures: {}", captures);
    println!();

    nodes
}

pub fn engine_perft(bs: &BoardState, depth: u8, tt: &mut transposition::TranspositionTable) {
    // let mut tt = transposition::TranspositionTable::new(); // not included in duration
    let start = Instant::now();
    let (eval, mv) = engine::choose_move(bs, depth, tt);
    let duration = start.elapsed();
    println!(
        "Engine perft at depth {} (took {:?} to complete):",
        depth, duration
    );
    println!(" - Eval: {}", eval);
    println!(" - Best move: {:?}", mv);
    println!();
}
