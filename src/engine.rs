use crate::position::*;
use rand::seq::SliceRandom;
use std::cmp;

pub fn choose_move(pos: &Position) -> &Move {
    let mv = minimax_root(pos, 5);
    mv
    //*pos.get_legal_moves().choose(&mut rand::thread_rng()).unwrap_or_else(|| panic!("CHECKMATE"))
}

pub fn minimax_root(pos: &Position, depth: i32) -> &Move {
    let moves = pos.get_legal_moves();
    let mut max_eval = i32::MIN;
    let mut max_mv = moves[0];
    for mv in moves {
        let child_pos = pos.new_position(mv);
        let eval = minimax(&child_pos, depth - 1, i32::MIN, i32::MAX, false);
        if eval > max_eval {
            max_eval = eval;
            max_mv = mv;
        }
    }
    max_mv
}

pub fn minimax(pos: &Position, depth: i32, mut a: i32, mut b: i32, is_maxi: bool) -> i32 {
    let moves = pos.get_legal_moves();


    if depth == 0 {
        return -evaluate(pos);
    }
    if is_maxi {
        let mut max_eval = i32::MIN;

        for mv in moves {
            let child_pos = pos.new_position(mv);
            let eval = minimax(&child_pos, depth - 1, a, b, !is_maxi);
            if eval > max_eval {
                max_eval = eval;
            }

            a = cmp::max(a, eval);
            if b <= a {
                println!("break max");
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in moves {
            let child_pos = pos.new_position(mv);
            let eval = minimax(&child_pos, depth - 1, a, b, !is_maxi);
            if eval < min_eval {
                min_eval = eval;
            }

            b = cmp::min(b, eval);
            if b <= a {
                println!("break min");
                break;
            }
        }
        min_eval
    }
}

// pub fn negatedMax(pos: &Position, depth: i32, mut a: i32, mut b: i32, ev_sign: i32) -> (i32, &Move) {
//     let mut max: i32 = i32::MIN + 1;
//     let mut max_move: &Move;
//     max_move = &Move{ from: 0, to: 0, move_type: MoveType::Normal };
//     if depth == 0 { return (ev_sign * evaluate(pos), max_move) };  // TODO why does adding minus sign fix this for engine black player
//     let mut score = i32::MIN + 1;
//     for mv in pos.get_legal_moves() {
//         //println!("{:?}", mv);
//         let new_pos = pos.new_position(mv);
//         score = std::cmp::max(score, -negatedMax(&new_pos, depth - 1, -b, -a, -ev_sign).0);
//         a = std::cmp::max(a, score);
//         if score > max {
//             max = score;
//             max_move = mv;
//         }
//         if a >= b {
//             break;
//         }

//     }
//     //println!("{:?}", max_move);
//     (max, max_move) 
// }

pub fn evaluate(pos: &Position) -> i32 {
    let mut eval = 0;
    for s in &pos.position {
        match s {
            Square::Empty => { continue; }
            Square::Piece(p) => {
                let pcolour_sign = if p.pcolour == PieceColour::White {1} else {-1};
                match p.ptype {
                    PieceType::Pawn => {eval += 10 * pcolour_sign},
                    PieceType::Knight => {eval += 30 * pcolour_sign},
                    PieceType::Bishop => {eval += 30 * pcolour_sign},
                    PieceType::Rook => {eval += 50 * pcolour_sign},
                    PieceType::Queen => {eval += 90 * pcolour_sign},
                    PieceType::King => {eval += 900 * pcolour_sign}
                }
            }
        }
    }
    eval
}