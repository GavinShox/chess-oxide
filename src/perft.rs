use std::time::Instant;

use crate::movegen::*;
use crate::position::Position;

fn get_all_legal_positions(
    pos: &Position,
    depth: i32,
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
            MoveType::Promotion(_) => {
                *promotions += 1;
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

pub fn perft(pos: &Position, depth: i32) -> u64 {
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
    println!("Nodes: {}", nodes);
    println!("Move types breakdown: ");
    println!("Promotions: {}", promotions);
    println!("Castles: {}", castles);
    println!("En Passant: {}", castles);
    println!("Captures: {}", captures);

    nodes
}
