use crate::position::*;
use rand::seq::SliceRandom;

pub fn choose_move(pos: &mut Position) -> Move {
    *pos.gen_legal_moves().choose(&mut rand::thread_rng()).unwrap()
}