use std::hash::Hash;
use std::{ rc::Rc, collections::HashMap };

use crate::position;
use crate::position::*;
use crate::engine;
use crate::movegen::*;

pub trait Player {
    fn get_move(&self, _: &BoardState) -> Move;
}

#[derive(Debug)]
pub enum BoardStateError {
    IllegalMove,
    NullMove,
    NoLegalMoves
}

#[derive(Debug, PartialEq)]
pub enum GameState {
    Check,
    Checkmate,
    Stalemate,
    Repetition,
    FiftyMove,
    Active,
}

pub struct BoardState {
    pub position: Position,
    position_hash: u64,
    move_count: u32,
    halfmove_count: u32,
    side_to_move: PieceColour,
    last_move: Move,
    pub legal_moves: Vec<Move>,
    position_occurences: HashMap<PositionHash, u8>,
}

impl BoardState {
    pub fn new_starting() -> Self {
        let position = Position::new_starting();
        let position_hash: PositionHash = position.pos_hash();
        let side_to_move = position.side;
        // deref all legal moves, performance isn't as important here, so avoid lifetime specifiers to make things easier to look at
        let legal_moves = position.get_legal_moves().into_iter().cloned().collect();
        let mut position_occurences = HashMap::new();
        *position_occurences.entry(position_hash).or_insert(0) += 1;
        BoardState {
            position,
            move_count: 0,
            halfmove_count: 0,
            position_hash,
            side_to_move,
            last_move: NULL_MOVE,
            legal_moves,
            position_occurences,
        }
    }

    pub fn next_state(&self, mv: &Move) -> Result<Self, BoardStateError> {
        if mv == &NULL_MOVE {
            return Err(BoardStateError::NullMove);
        }
        if !self.legal_moves.contains(mv) {
            return Err(BoardStateError::IllegalMove);
        }

        let current_game_state = self.get_gamestate();
        
        if
            current_game_state == GameState::Checkmate ||
            current_game_state == GameState::Stalemate ||
            current_game_state == GameState::FiftyMove ||
            current_game_state == GameState::Repetition
        {
            return Err(BoardStateError::NoLegalMoves);
        };

        let position = self.position.new_position(mv);
        let position_hash = position.pos_hash();
        let side_to_move = position.side;
        let last_move = *mv;
        // deref all legal moves
        let legal_moves = position
            .get_legal_moves()
            .into_iter()
            .map(|x| *x)
            .collect();

        let move_count = if side_to_move == PieceColour::White {
            self.move_count + 1
        } else {
            self.move_count
        };

        let halfmove_reset =
            mv.move_type == MoveType::PawnPush ||
            mv.move_type == MoveType::DoublePawnPush ||
            mv.move_type == MoveType::Capture;
        let halfmove_count = if halfmove_reset { 0 } else { self.halfmove_count + 1 };

        let mut position_occurences = self.position_occurences.clone();
        *position_occurences.entry(position_hash).or_insert(0) += 1;

        Ok(Self {
            position,
            position_hash,
            move_count,
            halfmove_count,
            side_to_move,
            last_move,
            legal_moves,
            position_occurences,
        })
    }

    pub fn get_occurences_of_current_position(&self) -> u8 {
        *self.position_occurences.get(&self.position_hash).unwrap_or(&1)
    }

    pub fn get_gamestate(&self) -> GameState {
        let legal_move_len = self.legal_moves.len();
        let is_in_check = self.position.is_in_check();
        let occurence_of_current_pos = self.get_occurences_of_current_position();

        // checkmate has to be checked for first, as it supercedes other states like the 50 move rule
        return if is_in_check && legal_move_len == 0 {
            GameState::Checkmate
        } else if !is_in_check && legal_move_len == 0 {
            GameState::Stalemate
        } else if self.halfmove_count >= 100 {
            GameState::FiftyMove
        } else if occurence_of_current_pos >= 3 {
            GameState::Repetition
        } else if is_in_check {
            GameState::Check
        } else {
            GameState::Active
        };
    }
}

pub struct Board {
    pub current_state: Rc<BoardState>,
    pub state_history: Vec<Rc<BoardState>>,
    pub white_player: Box<dyn Player>,
    pub black_player: Box<dyn Player>,
}

impl Board {
    pub fn new(white_player: Box<dyn Player>, black_player: Box<dyn Player>) -> Self {
        let current_state = Rc::new(BoardState::new_starting());

        let mut state_history: Vec<Rc<BoardState>> = Vec::new();
        state_history.push(current_state.clone());

        Board {
            current_state,
            state_history,
            white_player,
            black_player,
        }
    }
    pub fn branch(&self, branch_state: Rc<BoardState>) -> Self {
        // TODO, clone from specific state in state_history. Will probably need to store data differently like position_occurences
        // probably will have to go through all position hashes after the branch node, and remove occurences one by one
        todo!()
    }

    pub fn make_move(&mut self) -> Result<GameState, BoardStateError> {
        let current_player = if self.current_state.side_to_move == PieceColour::White {
            &self.white_player
        } else {
            &self.black_player
        };
        let mv = current_player.get_move(&self.current_state);
        let next_state = self.current_state.next_state(&mv)?;

        self.current_state = Rc::new(next_state);
        self.state_history.push(self.current_state.clone());

        let game_state = self.current_state.get_gamestate();

        Ok(game_state)
    }
    pub fn unmake_move(&mut self) -> Result<Rc<BoardState>, BoardStateError> {
        todo!()
    }
}