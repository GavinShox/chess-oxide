use crate::position::*;
use rand::seq::SliceRandom;

pub fn choose_move(pos: &Position) -> &Move {
    let (eval, mv) = negatedMax(pos, 4);
    println!("{}", eval);
    mv
    //*pos.get_legal_moves().choose(&mut rand::thread_rng()).unwrap_or_else(|| panic!("CHECKMATE"))
}

pub fn negatedMax(pos: &Position, depth: i32) -> (i32, &Move) {
    let mut max: i32 = i32::MIN;
    let mut max_move: &Move;
    max_move = &Move{ from: 0, to: 0, move_type: MoveType::Normal };
    if depth == 0 { return (-evaluate(pos), max_move) };  // TODO why does adding minus sign fix this for engine black player

    for mv in pos.get_legal_moves() {
        //println!("{:?}", mv);
        let new_pos = pos.new_position(mv);
        let score = -negatedMax(&new_pos, depth - 1).0;
        if score > max {
            max = score;
            max_move = mv;
        }
    }
    //println!("{:?}", max_move);
    (max, max_move) 
}

pub fn evaluate(position: &Position) -> i32 {
    let mut eval = 0;
    for s in &position.position {
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