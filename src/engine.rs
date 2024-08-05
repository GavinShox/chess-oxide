use std::cmp;

use crate::movegen::*;
use crate::board::*;

// avoid int overflows when operating on these values i.e. negating, +/- checkmate depth etc.
const MIN: i32 = i32::MIN + 1000;  
const MAX: i32 = i32::MAX - 1000;


pub fn choose_move(bs: &BoardState, depth: i32) -> (i32, &Move) {
    // TODO add check if position is in endgame, for different evaluation
    negamax_root(bs, depth, bs.side_to_move)
}

pub fn quiescence(bs: &BoardState, depth: i32, mut alpha: i32, beta: i32, maxi_colour: PieceColour) -> i32 {
    let mut max_eval = evaluate(bs, maxi_colour);
    if max_eval >= beta || depth == 0 {
        return max_eval;
    }
    alpha = cmp::max(alpha, max_eval);
    let moves = &bs.legal_moves;
    for i in sorted_move_indexes(&moves, true) {
        let mv = moves[i];
        let child_bs = bs.next_state(&mv).unwrap();
        let eval = -quiescence(&child_bs, depth - 1, -beta, -alpha, !maxi_colour);
        max_eval = cmp::max(max_eval, eval);
        alpha = cmp::max(alpha, max_eval);
        if beta <= alpha {
            break;
        }
    }
    max_eval
}

pub fn negamax_root(bs: &BoardState, depth: i32, maxi_colour: PieceColour) -> (i32, &Move) {
    if bs.is_in_check() && bs.legal_moves.is_empty() {
        return (if bs.side_to_move == maxi_colour {MIN} else {MAX}, &NULL_MOVE);  
    } else if bs.legal_moves.is_empty() || bs.get_occurences_of_current_position() == 3 {
        // stalemate
        return (0, &NULL_MOVE); 
    } 
    let mut alpha = MIN;
    let beta = MAX;
    
    let mut best_move = &bs.legal_moves[0];
    let mut max_eval = MIN;
    for i in sorted_move_indexes(&bs.legal_moves, false) {
        let mv = &bs.legal_moves[i];
        let child_bs = bs.next_state(mv).unwrap();
        let eval = -negamax(&child_bs, depth - 1, -beta, -alpha, !maxi_colour, 1);
        if eval > max_eval {
            max_eval = eval;
            best_move = &mv;
        }
        alpha = cmp::max(alpha, max_eval);
        if beta <= alpha {
            break;
        }
    }
    unsafe { println!("NODES: {}, PRUNES  {}", NODES, PRUNES); }

    (max_eval, best_move)
}

const QUIECENCE_DEPTH: i32 = 4;
static mut NODES: usize = 0;
static mut PRUNES: usize = 0;

//todo maybe no need for BoardState here, only for root negamax?
pub fn negamax(bs: &BoardState, depth: i32, mut alpha: i32, beta: i32, maxi_colour: PieceColour, root_depth: i32) -> i32 {
    if bs.is_in_check() && bs.legal_moves.is_empty() {
        return if bs.side_to_move == maxi_colour {MIN + root_depth} else {MAX  - root_depth};
    } else if bs.legal_moves.is_empty() || bs.get_occurences_of_current_position() == 3 {
        return 0;  // stalemate
    } else if depth == 0 {
        return quiescence(bs, QUIECENCE_DEPTH, alpha, beta, maxi_colour);
    }

    let mut max_eval = MIN;
    for i in sorted_move_indexes(&bs.legal_moves, false) {
        let mv = &bs.legal_moves[i];
        let child_bs = bs.next_state(mv).unwrap();
        let eval = -negamax(&child_bs, depth - 1, -beta, -alpha, !maxi_colour, root_depth + 1);
        if eval > max_eval {
            max_eval = eval;
        }
        alpha = cmp::max(alpha, max_eval);
        if beta <= alpha {
            unsafe {
                PRUNES += 1;
            };
            break;
        }
    }
    unsafe {
        NODES += 1;
    };
    max_eval
}

// pub fn quiescence_sort_move_indexes(moves: &[Move]) -> Vec<usize> {
//     let mut move_scores: Vec<i32> = Vec::with_capacity(moves.len());
//     let mut move_indexes: Vec<usize> = (0..moves.len()).collect();
//     for mv in moves {
//         let mut mv_score = 0;
//         match mv.move_type {
//             MoveType::Capture(capture_type) => {
//                 mv_score += get_piece_value(&capture_type) - get_piece_value(&mv.piece.ptype);
//             },
//             _ => {},
//         }
//         move_scores.push(mv_score);
//     }
//     let mut sorted_move_indexes = 
//         move_indexes
//         .iter()
//         .zip(move_scores.iter())
//         .collect::<Vec<_>>();

//     // sort the moves in descending order of scores
//     sorted_move_indexes.sort_unstable_by(|a, b| b.1.cmp(a.1));

//     sorted_move_indexes.into_iter().map(|a| *a.0).collect::<Vec<_>>()
// }


pub fn sorted_move_indexes(moves: &[Move], captures_only: bool) -> Vec<usize> {
    let mut move_scores: Vec<i32> = Vec::with_capacity(moves.len());
    let move_indexes: Vec<usize> = (0..moves.len()).collect();

    for mv in moves {
        // non captures = -1 so they can be filtered out
        if captures_only && !matches!(mv.move_type, MoveType::Capture(_)) { 
            move_scores.push(-1);
            continue; 
        }

        let mut mv_score = 0;

        // if mv is a capture
        match mv.move_type {
            MoveType::Capture(capture_type) => {
                mv_score += get_piece_value(&capture_type) - get_piece_value(&mv.piece.ptype);
                if mv_score < 0 {
                    mv_score = 1;  // prioritise captures, even when capturing with a more valuable piece. After trades it could still be good
                }
            },
            MoveType::Promotion(promotion_type) => {
                mv_score += get_piece_value(&promotion_type);
            },
            _ => {},
        }
        move_scores.push(mv_score);
    }
    let mut sorted_move_indexes = 
        move_indexes
        .iter()
        .zip(move_scores.iter())
        .collect::<Vec<_>>();

    // sort the moves in descending order of scores
    sorted_move_indexes.sort_unstable_by(|a, b| b.1.cmp(a.1));
    // filter out negative scores here
    sorted_move_indexes.into_iter().filter(|&x| *x.1 >= 0).map(|a| *a.0).collect::<Vec<_>>()
}
// values in centipawns
fn get_piece_value(ptype: &PieceType) -> i32 {
    match ptype {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
        PieceType::None => panic!("None piece type"),
    }
}

fn get_piece_pos_value(i: usize, piece: &Piece, is_endgame: bool) -> i32 {
    // all pos values are from whites perspective (a8 = index 0, h1 = index 63)
    const PAWN_POS_VALUES: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
        5,  5,  10, 25, 25, 10,  5,  5,
        0,  0,  0,  20, 20, 0,  0,  0,
        5, -5,-10,  0,  0,-10, -5,  5,
        5, 10, 10, -20,-20, 10, 10,  5,
        0,  0,  0,  0,  0,  0,  0,  0
    ];
    const KNIGHT_POS_VALUES: [i32; 64] = [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -30,  5, 15, 20, 20, 15,  5,-30,
        -30,  0, 15, 20, 20, 15,  0,-30,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ];
    const BISHOP_POS_VALUES: [i32; 64] = [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10,  5,  0,  0,  0,  0,  5,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ];
    const ROOK_POS_VALUES: [i32; 64] = [
        0,  0,  0,  0,  0,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0
    ];
    const QUEEN_POS_VALUES: [i32; 64] = [
        -20,-10,-10, -5, -5,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5,  5,  5,  5,  0,-10,
        -5,  0,  5,  5,  5,  5,  0, -5,
         0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  0,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20
    ];
    const KING_MIDDLE_POS_VALUES: [i32; 64] = [
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -30,-40,-40,-50,-50,-40,-40,-30,
        -20,-30,-30,-40,-40,-30,-30,-20,
        -10,-20,-20,-20,-20,-20,-20,-10,
         20, 20,  0,  0,  0,  0, 20, 20,
         20, 30, 10,  0,  0, 10, 30, 20
    ];
    const KING_END_POS_VALUES: [i32; 64] = [
        -50,-40,-30,-20,-20,-30,-40,-50,
        -30,-20,-10,  0,  0,-10,-20,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 30, 40, 40, 30,-10,-30,
        -30,-10, 20, 30, 30, 20,-10,-30,
        -30,-30,  0,  0,  0,  0,-30,-30,
        -50,-30,-30,-30,-30,-30,-30,-50
    ];

    let side_adjusted_idx = if piece.pcolour == PieceColour::White {i} else {63 - i};
    match piece.ptype {
        PieceType::Pawn => PAWN_POS_VALUES[side_adjusted_idx],
        PieceType::Knight => KNIGHT_POS_VALUES[side_adjusted_idx],
        PieceType::Bishop => BISHOP_POS_VALUES[side_adjusted_idx],
        PieceType::Rook => ROOK_POS_VALUES[side_adjusted_idx],
        PieceType::Queen => QUEEN_POS_VALUES[side_adjusted_idx],
        PieceType::King => if is_endgame {KING_END_POS_VALUES[side_adjusted_idx]} else {KING_MIDDLE_POS_VALUES[side_adjusted_idx]},
        PieceType::None => panic!("PieceType::None is not a valid piece type"),
    }
}

// adapted piece eval scores from here -> https://www.chessprogramming.org/Simplified_Evaluation_Function
pub fn evaluate(bs: &BoardState, maxi_colour: PieceColour) -> i32 {
    let mut w_eval: i32 = 0;
    let mut b_eval: i32 = 0;
    //tTODO add getter function for position
    for (i, s) in bs.get_pos64().iter().enumerate() {
        match s {
            Square::Empty => { continue; }
            Square::Piece(p) => {
                let val = get_piece_value(&p.ptype) + get_piece_pos_value(i, p, false);
                if p.pcolour == PieceColour::White {w_eval += val} else {b_eval += val};
            }
        }
    }
    let eval = w_eval - b_eval;
    eval * (if maxi_colour == PieceColour::White {1} else {-1})
}
