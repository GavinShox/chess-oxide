use std::time::{Duration, Instant};

use crate::{board, engine, movegen::*, position::Position, transposition, BoardState};

#[derive(Debug, Default)]
pub struct PerftNodes {
    pub nodes: u64,
    pub captures: u64,
    pub en_passant: u64,
    pub castles: u64,
    pub promotions: u64,
}

pub fn perft(pos_iterations: u32, engine_iterations: u32) {
    let pos = Position::new_starting();
    let board = board::Board::new();

    let mut total_pos_perft_time = Duration::new(0, 0);
    for i in 0..pos_iterations {
        let start = Instant::now();
        pos_perft(&pos, 5);
        let duration = start.elapsed();
        total_pos_perft_time += duration;
        println!(
            "Time elapsed in position perft iteration {}: {:?}",
            i + 1,
            duration
        );
    }

    let mut total_engine_time = Duration::new(0, 0);
    for i in 0..engine_iterations {
        let mut tt = transposition::TT::new();
        let start = Instant::now();
        engine_perft(board.get_current_state(), 7, &mut tt);
        let duration = start.elapsed();
        total_engine_time += duration;
        println!(
            "Time elapsed in engine perft iteration {}: {:?}\n",
            i + 1,
            duration
        );
    }
    println!("Total time elapsed in position perft: {:?} (after {} iterations)\nAverage time per iteration: {:?}", total_pos_perft_time, pos_iterations, total_pos_perft_time / pos_iterations);
    println!();
    println!("Total time elapsed in engine perft: {:?} (after {} iterations)\nAverage time per iteration: {:?}", total_engine_time, engine_iterations, total_engine_time / engine_iterations);
}

pub fn pos_perft(pos: &Position, depth: u8) -> PerftNodes {
    let mut nodes = PerftNodes::default();

    let start = Instant::now();

    get_all_legal_positions(pos, depth, &mut nodes);

    let duration = start.elapsed();

    println!(
        "Perft at depth {} (took {:?} to complete):",
        depth, duration
    );
    println!(" - Nodes: {}", nodes.nodes);
    println!(" - Move types breakdown: ");
    println!(" - Promotions: {}", nodes.promotions);
    println!(" - Castles: {}", nodes.castles);
    println!(" - En Passant: {}", nodes.en_passant);
    println!(" - Captures: {}", nodes.captures);
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

#[inline]
fn get_all_legal_positions(pos: &Position, depth: u8, nodes: &mut PerftNodes) {
    let moves = pos.get_legal_moves();
    if depth == 0 || moves.is_empty() {
        return;
    }
    for mv in moves {
        if depth == 1 {
            match mv.move_type {
                MoveType::EnPassant(_) => {
                    nodes.en_passant += 1;
                    nodes.captures += 1;
                }
                MoveType::Promotion(_, capture) => {
                    nodes.promotions += 1;
                    if capture.is_some() {
                        nodes.captures += 1;
                    }
                }
                MoveType::Castle(_) => {
                    nodes.castles += 1;
                }
                MoveType::Capture(_) => {
                    nodes.captures += 1;
                }
                _ => {}
            }
            nodes.nodes += 1;
        } else {
            let p = pos.new_position(mv);
            //let unmake_move = pos.make_move(mv);
            get_all_legal_positions(&p, depth - 1, nodes);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fen::FEN;

    #[test]
    fn test_perft() {
        // https://www.chessprogramming.org/Perft_Results
        // Assert perft results equal those in chessprogramming.org
        let pos1 = Position::new_starting();

        let pos2 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos3 = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos4 = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos4mirrored = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos5 = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos6 = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            .parse::<FEN>()
            .unwrap()
            .into();

        let pos1_nodes = pos_perft(&pos1, 5);
        assert_eq!(pos1_nodes.nodes, 4865609);
        assert_eq!(pos1_nodes.captures, 82719);
        assert_eq!(pos1_nodes.en_passant, 258);
        assert_eq!(pos1_nodes.castles, 0);
        assert_eq!(pos1_nodes.promotions, 0);

        let pos2_nodes = pos_perft(&pos2, 4);
        assert_eq!(pos2_nodes.nodes, 4085603);
        assert_eq!(pos2_nodes.captures, 757163);
        assert_eq!(pos2_nodes.en_passant, 1929);
        assert_eq!(pos2_nodes.castles, 128013);
        assert_eq!(pos2_nodes.promotions, 15172);

        let pos3_nodes = pos_perft(&pos3, 5);
        assert_eq!(pos3_nodes.nodes, 674624);
        assert_eq!(pos3_nodes.captures, 52051);
        assert_eq!(pos3_nodes.en_passant, 1165);
        assert_eq!(pos3_nodes.castles, 0);
        assert_eq!(pos3_nodes.promotions, 0);

        let pos4_nodes = pos_perft(&pos4, 5);
        assert_eq!(pos4_nodes.nodes, 15833292);
        assert_eq!(pos4_nodes.captures, 2046173);
        assert_eq!(pos4_nodes.en_passant, 6512);
        assert_eq!(pos4_nodes.castles, 0);
        assert_eq!(pos4_nodes.promotions, 329464);

        let pos4mirrored_nodes = pos_perft(&pos4mirrored, 5);
        assert_eq!(pos4mirrored_nodes.nodes, 15833292);
        assert_eq!(pos4mirrored_nodes.captures, 2046173);
        assert_eq!(pos4mirrored_nodes.en_passant, 6512);
        assert_eq!(pos4mirrored_nodes.castles, 0);
        assert_eq!(pos4mirrored_nodes.promotions, 329464);

        let pos5_nodes = pos_perft(&pos5, 4);
        assert_eq!(pos5_nodes.nodes, 2103487);

        let pos6_nodes = pos_perft(&pos6, 4);
        assert_eq!(pos6_nodes.nodes, 3894594);
    }
}
