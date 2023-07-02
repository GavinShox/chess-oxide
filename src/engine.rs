use crate::position::*;
use rand::seq::SliceRandom;
use std::cmp;
use crate::movegen::*;

const MIN: i32 = i32::MIN + 1;  // avoid int overflow when negating
const MAX: i32 = i32::MAX;

pub fn random_move(pos: &Position) -> &Move {
    pos.get_legal_moves().choose(&mut rand::thread_rng()).unwrap_or(&&NULL_MOVE)
}

// TODO SHould use board_state so it can factor in checkmate and stalemate easier
pub fn choose_move(pos: &Position, depth: i32) -> (i32, &Move) {
    // TODO add check if position is in endgame, for different evaluation
    negamax_root(pos, depth, pos.side)
}

pub fn quiescence(pos: &Position, depth: i32, mut alpha: i32, beta: i32, maxi_colour: PieceColour) -> i32 {
    let mut max_eval = evaluate(pos, maxi_colour);
    if max_eval >= beta || depth == 0 {
        return max_eval;
    }
    alpha = cmp::max(alpha, max_eval);
    let moves = pos.get_legal_moves().into_iter().filter(|mv| matches!(mv.move_type, MoveType::Capture(_))).collect::<Vec<_>>();
    for i in sorted_move_indexes(&moves) {
        let mv = moves[i];
        let child_pos = pos.new_position(mv);
        let eval = -quiescence(&child_pos, depth - 1, -beta, -alpha, !maxi_colour);
        max_eval = cmp::max(max_eval, eval);
        alpha = cmp::max(alpha, eval);
        if beta <= alpha {
            break;
        }
    }
    max_eval
}
pub fn negamax_root(pos: &Position, depth: i32, maxi_colour: PieceColour) -> (i32, &Move) {
    let moves = pos.get_legal_moves();
    if moves.is_empty() && pos.is_in_check() {
        return (if pos.side == maxi_colour {MIN} else {MAX}, &NULL_MOVE);  
    } else if moves.is_empty() {
        // stalemate
        return (0, &NULL_MOVE); 
    } 
    let mut alpha = MIN;
    let beta = MAX;
    let mut best_move = moves[0];
    let mut max_eval = MIN;
    for i in sorted_move_indexes(&moves) {
        let mv = moves[i];
        let child_pos = pos.new_position(mv);
        let eval = -negamax(&child_pos, depth - 1, -beta, -alpha, !maxi_colour);
        if eval > max_eval {
            max_eval = eval;
            best_move = mv;
        }
        alpha = cmp::max(alpha, eval);
        if beta <= alpha {
            break;
        }
    }
    (max_eval, best_move)
}
const QUIECENCE_DEPTH: i32 = 5;
// todo maybe return depth reached for fastest checkmate
pub fn negamax(pos: &Position, depth: i32, mut alpha: i32, beta: i32, maxi_colour: PieceColour) -> i32 {
    let moves = pos.get_legal_moves();
    //sort_moves(pos, &mut moves);
    // TODO different checks for stalemate and checkmate. not sure if below works correctly, need to test
    if moves.is_empty() && pos.is_in_check() {
        return if pos.side == maxi_colour {MIN} else {MAX};  // checkmate TODO im not sure if min is the correct return here. maybe account for maxi_colour?
    } else if moves.is_empty() {
        return 0;  // stalemate
    } else if depth == 0 {
        return quiescence(pos, QUIECENCE_DEPTH, alpha, beta, maxi_colour);
    }

    let mut max_eval = MIN;
    for i in sorted_move_indexes(&moves) {
        let mv = moves[i];
        let child_pos = pos.new_position(mv);
        let colour = !maxi_colour;
        let eval = -negamax(&child_pos, depth - 1, -beta, -alpha, colour);
        if eval > max_eval {
            max_eval = eval;
        }
        alpha = cmp::max(alpha, eval);
        if beta <= alpha {
            break;
        }
    }
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


pub fn sorted_move_indexes(moves: &[&Move]) -> Vec<usize> {
    let mut move_scores: Vec<i32> = Vec::with_capacity(moves.len());
    let mut move_indexes: Vec<usize> = (0..moves.len()).collect();

    for mv in moves {
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

    sorted_move_indexes.into_iter().map(|a| *a.0).collect::<Vec<_>>()
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
pub fn evaluate(pos: &Position, maxi_colour: PieceColour) -> i32 {
    let mut w_eval: i32 = 0;
    let mut b_eval: i32 = 0;
    for (i, s) in pos.position.iter().enumerate() {
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
