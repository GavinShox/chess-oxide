use std::{rc::Rc, collections::HashMap};

use crate::position::*;

enum GameState {
    Check,
    Checkmate,
    Stalemate,
    Repetition,
    FiftyMove,
    Active
}

struct BoardState {
    position: Position,
    move_count: u32,
    halfmove_count: u32,
    hash: u64,
    side_to_move: PieceColour,
    game_state: GameState

}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        let position_hash = position.pos_hash();
        let side_to_move = position.side;
        let game_state = GameState::Active;
        BoardState { position, move_count: 0, halfmove_count: 0, hash: position_hash, side_to_move, game_state }
    }
}

struct BoardTree {
    
}

pub struct Board {
    current_position: usize,
    positions: Vec<Position>,
    position_occurences: HashMap<u64, usize>,
    move_count: u32,
    halfmove_count: u32
}

impl Board {
    pub fn new() -> Self {
        let init_pos = Position::new_starting();
        let init_pos_hash = init_pos.pos_hash();
        let mut positions = Vec::new();
        let mut position_hashes = HashMap::new();
        positions.push(init_pos);
        position_hashes.insert(init_pos_hash, 1);
        Board { current_position: 0, positions, position_occurences: position_hashes, move_count: 0, halfmove_count: 0 }
    }
    pub fn get_gamestate(&self) -> GameState {
        let legal_move_len = self.positions[self.current_position].get_legal_moves().len();
        let is_in_check = self.positions[self.current_position].is_in_check();

        return if is_in_check && legal_move_len == 0 {
            GameState::Checkmate
        } else if !is_in_check && legal_move_len == 0 {
            GameState::Stalemate
        } else if is_in_check{
            GameState::Check
        } else {
            GameState::Active
        }
    }

}