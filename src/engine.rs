use crate::position::*;
use rand::seq::SliceRandom;
use std::cmp;

pub fn choose_move(pos: &Position) -> &Move {
    let mv = minimax(pos, 4, i32::MIN, i32::MAX, true, PieceColour::White).1.unwrap();
    mv
    //*pos.get_legal_moves().choose(&mut rand::thread_rng()).unwrap_or_else(|| panic!("CHECKMATE"))
}

pub fn sort_moves(pos: &Position, moves: &mut Vec<&Move>) {
    let mut moves_c = moves.clone();
    let mut move_scores: Vec<i32> = Vec::with_capacity(moves_c.len());

    for mv in &moves_c {
        let mut mv_score = 0;
        let mv_from_square = pos.position[mv.from];
        let mv_to_square = pos.position[mv.to];

        let mv_from_piece = match &mv_from_square {
            Square::Piece(p) => p,
            Square::Empty => panic!(),
        };

        // if mv is a capture
        match mv_to_square {
            Square::Piece(p) => {
                mv_score += 10 * get_piece_value(&p.ptype) - get_piece_value(&mv_from_piece.ptype);
            },
            Square::Empty => {},
        }

        // if pawn promotion
        match mv.move_type {
            MoveType::Promotion(promotion_type) => {
                mv_score += get_piece_value(&promotion_type);
            },
            _ => {}
        }
        move_scores.push(mv_score);
    }
    let mut move_score_pairs = moves_c.iter().zip(move_scores.iter()).collect::<Vec<_>>();
    move_score_pairs.sort_unstable_by(|a, b| a.1.cmp(b.1));
    moves_c = move_score_pairs.into_iter().map(|(a, _)| *a).collect::<Vec<_>>();
    *moves = moves_c;
}

pub fn minimax(pos: &Position, depth: i32, mut alpha: i32, mut beta: i32, is_maxi: bool, maxi_colour: PieceColour) -> (i32, Option<&Move>) {
    let mut moves = pos.get_legal_moves();
    sort_moves(pos, &mut moves);
    if depth == 0 || moves.len() == 0 {
        return (evaluate(pos, maxi_colour), None);
    }
    let mut best_move = moves[0];

    if is_maxi {
        let mut max_eval = i32::MIN;
        for mv in moves {
            let child_pos = pos.new_position(mv);
            let eval = minimax(&child_pos, depth - 1, alpha, beta, false, maxi_colour).0;
            if eval > max_eval {
                max_eval = eval;
                best_move = mv;
            }
            alpha = cmp::max(alpha, eval);
            if beta <= alpha {
                break;
            }
        }
        return (max_eval, Some(best_move));
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            let child_pos = pos.new_position(mv);
            let eval = minimax(&child_pos, depth - 1, alpha, beta, true, maxi_colour).0;
            if eval < min_eval {
                min_eval = eval;
                best_move = mv;
            }
            beta = cmp::min(beta, eval);
            if beta <= alpha {
                break;
            }
        }
        return (min_eval, Some(best_move));
    }
}

fn get_piece_value(ptype: &PieceType) -> i32 {
    match ptype {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 20000,
    }
}

pub fn evaluate(pos: &Position, maxi_colour: PieceColour) -> i32 {
    let mut w_eval: i32 = 0;
    let mut b_eval: i32 = 0;
    for s in &pos.position {
        match s {
            Square::Empty => { continue; }
            Square::Piece(p) => {
                let val = get_piece_value(&p.ptype);
                if p.pcolour == PieceColour::White {w_eval += val} else {b_eval += val};
            }
        }
    }
    let eval = w_eval - b_eval;
    return eval * (if maxi_colour == PieceColour::White {1} else {-1});
}




// pub fn aiminimax(pos: &Position, depth: i32, is_maxi: bool, maxi_colour: PieceColour, alpha: i32, beta: i32) -> (i32, Option<&Move>) {
//     let moves = pos.get_legal_moves();
//     if depth == 0 || moves.len() == 0 {
//         return (evaluate(pos, maxi_colour), None);
//     }
//     let mut best_move = moves[0];

//     if is_maxi {
//         let mut max_eval = i32::MIN;
//         for mv in moves {
//             let child_pos = pos.new_position(mv);
//             let eval = aiminimax(&child_pos, depth - 1, false, maxi_colour, alpha, beta).0;
//             if eval > max_eval {
//                 max_eval = eval;
//                 best_move = mv;
//             }
//             alpha = cmp::max(alpha, max_eval);
//             if beta <= alpha {
//                 break;
//             }
//         }
//         return (max_eval, Some(best_move));
//     } else {
//         let mut min_eval = i32::MAX;
//         for mv in moves {
//             let child_pos = pos.new_position(mv);
//             let eval = aiminimax(&child_pos, depth - 1, true, maxi_colour, alpha, beta).0;
//             if eval < min_eval {
//                 min_eval = eval;
//                 best_move = mv;
//             }
//             beta = cmp::min(beta, min_eval);
//             if beta <= alpha {
//                 break;
//             }
//         }
//         return (min
