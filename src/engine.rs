use crate::position::*;
use rand::seq::SliceRandom;

pub fn choose_move(pos: &Position) -> &Move {
    *pos.get_legal_moves().choose(&mut rand::thread_rng()).unwrap_or_else(|| panic!("CHECKMATE"))
}